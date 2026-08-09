use crate::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Identifier(String),
    Integer(String),
    Newline,
    Symbol(char),
    Operator(&'static str),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub(crate) fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let start = cursor;
        match bytes[cursor] {
            b' ' | b'\t' | b'\r' => cursor += 1,
            b'\n' => {
                cursor += 1;
                tokens.push(token(TokenKind::Newline, start, cursor));
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
            }
            byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                cursor += 1;
                while cursor < bytes.len()
                    && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
                {
                    cursor += 1;
                }
                tokens.push(token(
                    TokenKind::Identifier(source[start..cursor].to_owned()),
                    start,
                    cursor,
                ));
            }
            byte if byte.is_ascii_digit() => {
                cursor += 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                    cursor += 1;
                }
                tokens.push(token(
                    TokenKind::Integer(source[start..cursor].to_owned()),
                    start,
                    cursor,
                ));
            }
            b'=' | b'!' | b'>' | b'<' | b'+' | b'-' => {
                cursor += 1;
                let combined = match (bytes[start], bytes.get(cursor)) {
                    (b'=', Some(b'=')) => Some("=="),
                    (b'!', Some(b'=')) => Some("!="),
                    (b'>', Some(b'=')) => Some(">="),
                    (b'<', Some(b'=')) => Some("<="),
                    (b'+', Some(b'=')) => Some("+="),
                    (b'-', Some(b'=')) => Some("-="),
                    _ => None,
                };
                if let Some(operator) = combined {
                    cursor += 1;
                    tokens.push(token(TokenKind::Operator(operator), start, cursor));
                } else {
                    tokens.push(token(
                        TokenKind::Symbol(bytes[start] as char),
                        start,
                        cursor,
                    ));
                }
            }
            byte if byte.is_ascii() && !byte.is_ascii_control() => {
                cursor += 1;
                tokens.push(token(TokenKind::Symbol(byte as char), start, cursor));
            }
            byte if byte.is_ascii_control() => {
                return Err(vec![Diagnostic::new(
                    "MORVA1001",
                    format!("unsupported control character 0x{byte:02X}"),
                    Span {
                        start,
                        end: start + 1,
                    },
                )]);
            }
            _ => {
                let width = source[start..]
                    .chars()
                    .next()
                    .expect("cursor points into source")
                    .len_utf8();
                return Err(vec![Diagnostic::new(
                    "MORVA1002",
                    "non-ASCII identifiers are not supported yet",
                    Span {
                        start,
                        end: start + width,
                    },
                )]);
            }
        }
    }
    tokens.push(token(TokenKind::Eof, source.len(), source.len()));
    Ok(tokens)
}

fn token(kind: TokenKind, start: usize, end: usize) -> Token {
    Token {
        kind,
        span: Span { start, end },
    }
}
