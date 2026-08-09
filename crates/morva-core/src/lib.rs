use std::collections::HashSet;
use std::fmt;

const DECLARATION_KINDS: &[&str] = &[
    "system",
    "module",
    "entity",
    "enum",
    "service",
    "action",
    "event",
    "flow",
    "lifecycle",
    "scenario",
    "policy",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub kind: String,
    pub name: String,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub offset: Option<usize>,
}

impl Diagnostic {
    fn at(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset: Some(offset),
        }
    }

    fn semantic(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} (byte {offset})", self.message),
            None => f.write_str(&self.message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Word(String),
    Symbol(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    offset: usize,
}

pub fn parse(source: &str) -> Result<Document, Vec<Diagnostic>> {
    let tokens = lex(source)?;
    Parser { tokens, cursor: 0 }.parse_document()
}

pub fn check(document: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let systems = document
        .declarations
        .iter()
        .filter(|item| item.kind == "system")
        .count();
    if systems != 1 {
        diagnostics.push(Diagnostic::semantic(format!(
            "expected exactly one top-level system declaration, found {systems}"
        )));
    }
    check_scope(&document.declarations, "document", &mut diagnostics);
    diagnostics
}

fn check_scope(declarations: &[Declaration], scope: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut names = HashSet::new();
    for declaration in declarations {
        if !names.insert(declaration.name.as_str()) {
            diagnostics.push(Diagnostic::semantic(format!(
                "duplicate declaration '{}' in {scope}",
                declaration.name
            )));
        }
        check_scope(
            &declaration.declarations,
            &format!("{} '{}'", declaration.kind, declaration.name),
            diagnostics,
        );
    }
}

fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte.is_ascii_whitespace() {
            cursor += 1;
        } else if byte == b'/' && bytes.get(cursor + 1) == Some(&b'/') {
            cursor += 2;
            while cursor < bytes.len() && bytes[cursor] != b'\n' {
                cursor += 1;
            }
        } else if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Word(source[start..cursor].to_owned()),
                offset: start,
            });
        } else if byte.is_ascii_digit() {
            let start = cursor;
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Word(source[start..cursor].to_owned()),
                offset: start,
            });
        } else if byte.is_ascii() {
            tokens.push(Token {
                kind: TokenKind::Symbol(byte as char),
                offset: cursor,
            });
            cursor += 1;
        } else {
            return Err(vec![Diagnostic::at(
                "non-ASCII identifiers are not supported yet",
                cursor,
            )]);
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn parse_document(mut self) -> Result<Document, Vec<Diagnostic>> {
        let declarations = self.parse_scope(false)?;
        Ok(Document { declarations })
    }

    fn parse_scope(&mut self, closes: bool) -> Result<Vec<Declaration>, Vec<Diagnostic>> {
        let mut declarations = Vec::new();
        while let Some(token) = self.tokens.get(self.cursor) {
            if token.kind == TokenKind::Symbol('}') {
                if closes {
                    self.cursor += 1;
                    return Ok(declarations);
                }
                return Err(vec![Diagnostic::at(
                    "unexpected closing brace",
                    token.offset,
                )]);
            }

            let is_declaration = matches!(&token.kind, TokenKind::Word(word) if DECLARATION_KINDS.contains(&word.as_str()));
            if is_declaration {
                declarations.push(self.parse_declaration()?);
            } else {
                self.skip_non_declaration()?;
            }
        }
        if closes {
            Err(vec![Diagnostic::semantic("unclosed declaration block")])
        } else {
            Ok(declarations)
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, Vec<Diagnostic>> {
        let kind = match &self.tokens[self.cursor].kind {
            TokenKind::Word(word) => word.clone(),
            _ => unreachable!(),
        };
        self.cursor += 1;
        let Some(name_token) = self.tokens.get(self.cursor) else {
            return Err(vec![Diagnostic::semantic(format!(
                "missing name after {kind}"
            ))]);
        };
        let name = match &name_token.kind {
            TokenKind::Word(word) => word.clone(),
            _ => {
                return Err(vec![Diagnostic::at(
                    format!("missing name after {kind}"),
                    name_token.offset,
                )]);
            }
        };
        self.cursor += 1;

        let mut paren_depth = 0usize;
        while let Some(token) = self.tokens.get(self.cursor) {
            match token.kind {
                TokenKind::Symbol('(') => paren_depth += 1,
                TokenKind::Symbol(')') if paren_depth > 0 => paren_depth -= 1,
                TokenKind::Symbol('{') if paren_depth == 0 => {
                    self.cursor += 1;
                    let declarations = self.parse_scope(true)?;
                    return Ok(Declaration {
                        kind,
                        name,
                        declarations,
                    });
                }
                TokenKind::Symbol('}') if paren_depth == 0 => {
                    return Err(vec![Diagnostic::at(
                        format!("missing block for {kind} {name}"),
                        token.offset,
                    )]);
                }
                _ => {}
            }
            self.cursor += 1;
        }
        Err(vec![Diagnostic::semantic(format!(
            "missing block for {kind} {name}"
        ))])
    }

    fn skip_non_declaration(&mut self) -> Result<(), Vec<Diagnostic>> {
        let token = &self.tokens[self.cursor];
        match token.kind {
            TokenKind::Symbol('{') => {
                self.cursor += 1;
                self.skip_balanced_block()
            }
            _ => {
                self.cursor += 1;
                Ok(())
            }
        }
    }

    fn skip_balanced_block(&mut self) -> Result<(), Vec<Diagnostic>> {
        let mut depth = 1usize;
        while let Some(token) = self.tokens.get(self.cursor) {
            match token.kind {
                TokenKind::Symbol('{') => depth += 1,
                TokenKind::Symbol('}') => {
                    depth -= 1;
                    if depth == 0 {
                        self.cursor += 1;
                        return Ok(());
                    }
                }
                _ => {}
            }
            self.cursor += 1;
        }
        Err(vec![Diagnostic::semantic("unclosed nested block")])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_declarations() {
        let document =
            parse("system Shop { module Orders { entity Order {} action Confirm(x: Order) {} } }")
                .unwrap();
        assert_eq!(document.declarations[0].name, "Shop");
        assert_eq!(document.declarations[0].declarations[0].name, "Orders");
        assert!(check(&document).is_empty());
    }

    #[test]
    fn reports_duplicate_names_in_a_scope() {
        let document = parse("system Shop { entity Order {} enum Order {} }").unwrap();
        assert!(
            check(&document)
                .iter()
                .any(|item| item.message.contains("duplicate"))
        );
    }

    #[test]
    fn rejects_unclosed_blocks() {
        assert!(parse("system Shop {").is_err());
    }
}
