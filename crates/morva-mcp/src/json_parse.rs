//! Minimal std-only JSON parser for incoming JSON-RPC messages.
//!
//! Accepts one complete RFC 8259 value; enforces a nesting-depth bound so a
//! hostile client cannot trigger unbounded recursion.

const MAX_DEPTH: usize = 128;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedJson {
    Null,
    Bool(bool),
    /// Raw, syntax-validated number token, preserved verbatim so request ids
    /// round-trip exactly.
    Number(String),
    Str(String),
    Array(Vec<ParsedJson>),
    Object(Vec<(String, ParsedJson)>),
}

impl ParsedJson {
    pub fn get(&self, key: &str) -> Option<&ParsedJson> {
        match self {
            Self::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[ParsedJson]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }
}

pub fn parse(input: &str) -> Result<ParsedJson, String> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        input,
        cursor: 0,
    };
    parser.skip_whitespace();
    let value = parser.value(0)?;
    parser.skip_whitespace();
    if parser.cursor != parser.bytes.len() {
        return Err("trailing content after the JSON value".to_owned());
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    input: &'a str,
    cursor: usize,
}

impl Parser<'_> {
    fn skip_whitespace(&mut self) {
        while matches!(
            self.bytes.get(self.cursor),
            Some(b' ' | b'\t' | b'\n' | b'\r')
        ) {
            self.cursor += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Result<ParsedJson, String> {
        if depth > MAX_DEPTH {
            return Err("JSON nesting is too deep".to_owned());
        }
        match self.bytes.get(self.cursor) {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(ParsedJson::Str(self.string()?)),
            Some(b't') => self.literal("true", ParsedJson::Bool(true)),
            Some(b'f') => self.literal("false", ParsedJson::Bool(false)),
            Some(b'n') => self.literal("null", ParsedJson::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err("expected a JSON value".to_owned()),
        }
    }

    fn literal(&mut self, keyword: &str, value: ParsedJson) -> Result<ParsedJson, String> {
        if self.input[self.cursor..].starts_with(keyword) {
            self.cursor += keyword.len();
            Ok(value)
        } else {
            Err(format!("invalid JSON literal, expected '{keyword}'"))
        }
    }

    fn number(&mut self) -> Result<ParsedJson, String> {
        let start = self.cursor;
        if self.bytes.get(self.cursor) == Some(&b'-') {
            self.cursor += 1;
        }
        match self.bytes.get(self.cursor) {
            Some(b'0') => self.cursor += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.bytes.get(self.cursor), Some(b'0'..=b'9')) {
                    self.cursor += 1;
                }
            }
            _ => return Err("invalid JSON number".to_owned()),
        }
        if self.bytes.get(self.cursor) == Some(&b'.') {
            self.cursor += 1;
            if !matches!(self.bytes.get(self.cursor), Some(b'0'..=b'9')) {
                return Err("invalid JSON number fraction".to_owned());
            }
            while matches!(self.bytes.get(self.cursor), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        if matches!(self.bytes.get(self.cursor), Some(b'e' | b'E')) {
            self.cursor += 1;
            if matches!(self.bytes.get(self.cursor), Some(b'+' | b'-')) {
                self.cursor += 1;
            }
            if !matches!(self.bytes.get(self.cursor), Some(b'0'..=b'9')) {
                return Err("invalid JSON number exponent".to_owned());
            }
            while matches!(self.bytes.get(self.cursor), Some(b'0'..=b'9')) {
                self.cursor += 1;
            }
        }
        Ok(ParsedJson::Number(
            self.input[start..self.cursor].to_owned(),
        ))
    }

    fn string(&mut self) -> Result<String, String> {
        debug_assert_eq!(self.bytes.get(self.cursor), Some(&b'"'));
        self.cursor += 1;
        let mut value = String::new();
        loop {
            let Some(&byte) = self.bytes.get(self.cursor) else {
                return Err("unterminated JSON string".to_owned());
            };
            match byte {
                b'"' => {
                    self.cursor += 1;
                    return Ok(value);
                }
                b'\\' => {
                    self.cursor += 1;
                    let Some(&escape) = self.bytes.get(self.cursor) else {
                        return Err("unterminated JSON escape".to_owned());
                    };
                    self.cursor += 1;
                    match escape {
                        b'"' => value.push('"'),
                        b'\\' => value.push('\\'),
                        b'/' => value.push('/'),
                        b'b' => value.push('\u{8}'),
                        b'f' => value.push('\u{c}'),
                        b'n' => value.push('\n'),
                        b'r' => value.push('\r'),
                        b't' => value.push('\t'),
                        b'u' => {
                            let unit = self.hex_unit()?;
                            if (0xD800..=0xDBFF).contains(&unit) {
                                if self.bytes.get(self.cursor) != Some(&b'\\')
                                    || self.bytes.get(self.cursor + 1) != Some(&b'u')
                                {
                                    return Err("unpaired JSON surrogate".to_owned());
                                }
                                self.cursor += 2;
                                let low = self.hex_unit()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err("invalid JSON low surrogate".to_owned());
                                }
                                let combined = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00);
                                value.push(
                                    char::from_u32(combined)
                                        .ok_or_else(|| "invalid JSON code point".to_owned())?,
                                );
                            } else if (0xDC00..=0xDFFF).contains(&unit) {
                                return Err("unpaired JSON surrogate".to_owned());
                            } else {
                                value.push(
                                    char::from_u32(unit)
                                        .ok_or_else(|| "invalid JSON code point".to_owned())?,
                                );
                            }
                        }
                        _ => return Err("invalid JSON escape".to_owned()),
                    }
                }
                0x00..=0x1F => return Err("unescaped control character in JSON string".to_owned()),
                _ => {
                    let character = self.input[self.cursor..]
                        .chars()
                        .next()
                        .ok_or_else(|| "invalid UTF-8 in JSON string".to_owned())?;
                    value.push(character);
                    self.cursor += character.len_utf8();
                }
            }
        }
    }

    fn hex_unit(&mut self) -> Result<u32, String> {
        let end = self.cursor + 4;
        let digits = self
            .input
            .get(self.cursor..end)
            .ok_or_else(|| "truncated JSON unicode escape".to_owned())?;
        let unit = u32::from_str_radix(digits, 16)
            .map_err(|_| "invalid JSON unicode escape".to_owned())?;
        self.cursor = end;
        Ok(unit)
    }

    fn array(&mut self, depth: usize) -> Result<ParsedJson, String> {
        self.cursor += 1;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.bytes.get(self.cursor) == Some(&b']') {
            self.cursor += 1;
            return Ok(ParsedJson::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value(depth + 1)?);
            self.skip_whitespace();
            match self.bytes.get(self.cursor) {
                Some(b',') => self.cursor += 1,
                Some(b']') => {
                    self.cursor += 1;
                    return Ok(ParsedJson::Array(items));
                }
                _ => return Err("expected ',' or ']' in JSON array".to_owned()),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<ParsedJson, String> {
        self.cursor += 1;
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.bytes.get(self.cursor) == Some(&b'}') {
            self.cursor += 1;
            return Ok(ParsedJson::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.bytes.get(self.cursor) != Some(&b'"') {
                return Err("expected a JSON object key".to_owned());
            }
            let key = self.string()?;
            self.skip_whitespace();
            if self.bytes.get(self.cursor) != Some(&b':') {
                return Err("expected ':' in JSON object".to_owned());
            }
            self.cursor += 1;
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            members.push((key, value));
            self.skip_whitespace();
            match self.bytes.get(self.cursor) {
                Some(b',') => self.cursor += 1,
                Some(b'}') => {
                    self.cursor += 1;
                    return Ok(ParsedJson::Object(members));
                }
                _ => return Err("expected ',' or '}' in JSON object".to_owned()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_values_and_escapes() {
        let parsed = parse(r#"{"a": [1, -2.5e3, "x\n\u00e9\ud83d\ude00", true, null], "b": {}}"#)
            .expect("valid JSON parses");
        let items = parsed
            .get("a")
            .and_then(ParsedJson::as_array)
            .expect("array");
        assert_eq!(items[0], ParsedJson::Number("1".to_owned()));
        assert_eq!(items[1], ParsedJson::Number("-2.5e3".to_owned()));
        assert_eq!(items[2], ParsedJson::Str("x\n\u{e9}\u{1F600}".to_owned()));
        assert_eq!(parsed.get("b"), Some(&ParsedJson::Object(Vec::new())));
    }

    #[test]
    fn rejects_malformed_input() {
        for input in [
            "",
            "{",
            "[1,]",
            "\"unterminated",
            "{\"a\" 1}",
            "01",
            "1 2",
            "\"\\ud800\"",
            "\u{1}",
        ] {
            assert!(parse(input).is_err(), "{input:?} must fail");
        }
        let deep = format!("{}1{}", "[".repeat(200), "]".repeat(200));
        assert!(parse(&deep).is_err(), "depth bound holds");
    }
}
