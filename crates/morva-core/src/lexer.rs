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
            b' ' | b'\t' => cursor += 1,
            b'\r' => {
                cursor += 1;
                if bytes.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
                tokens.push(token(TokenKind::Newline, start, cursor));
            }
            b'\n' => {
                cursor += 1;
                tokens.push(token(TokenKind::Newline, start, cursor));
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor += 2;
                while cursor < bytes.len() && !matches!(bytes[cursor], b'\r' | b'\n') {
                    cursor += 1;
                }
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                let split_left = tokens.last().and_then(|previous| {
                    (previous.span.end == start).then_some(match &previous.kind {
                        TokenKind::Identifier(_) => SplitLeft::Identifier,
                        TokenKind::Integer(_) => SplitLeft::Integer,
                        TokenKind::Symbol('=' | '!' | '>' | '<' | '+' | '-') => {
                            SplitLeft::CompoundOperator
                        }
                        _ => SplitLeft::Other,
                    })
                });
                let (end, has_newline) = scan_block_comment_run(bytes, start, &mut tokens)?;
                if !has_newline
                    && bytes.get(end).is_some_and(|right| {
                        matches!(split_left, Some(SplitLeft::Identifier))
                            && (right.is_ascii_alphanumeric() || *right == b'_')
                            || matches!(split_left, Some(SplitLeft::Integer))
                                && right.is_ascii_digit()
                            || matches!(split_left, Some(SplitLeft::CompoundOperator))
                                && *right == b'='
                    })
                {
                    return Err(vec![Diagnostic::new(
                        "MORVA1025",
                        "comment cannot split a token",
                        Span {
                            start,
                            end: start + 2,
                        },
                    )]);
                }
                cursor = end;
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

#[derive(Clone, Copy)]
enum SplitLeft {
    Identifier,
    Integer,
    CompoundOperator,
    Other,
}

fn scan_block_comment_run(
    bytes: &[u8],
    start: usize,
    tokens: &mut Vec<Token>,
) -> Result<(usize, bool), Vec<Diagnostic>> {
    let mut cursor = start;
    let mut has_newline = false;
    while matches!(&bytes[cursor..], [b'/', b'*', ..]) {
        let outer_start = cursor;
        cursor += 2;
        let mut depth = 1usize;
        while cursor < bytes.len() && depth > 0 {
            if matches!(&bytes[cursor..], [b'/', b'*', ..]) {
                depth += 1;
                cursor += 2;
            } else if matches!(&bytes[cursor..], [b'*', b'/', ..]) {
                depth -= 1;
                cursor += 2;
            } else if bytes[cursor] == b'\r' {
                let newline_start = cursor;
                cursor += 1;
                if bytes.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
                tokens.push(token(TokenKind::Newline, newline_start, cursor));
                has_newline = true;
            } else if bytes[cursor] == b'\n' {
                let newline_start = cursor;
                cursor += 1;
                tokens.push(token(TokenKind::Newline, newline_start, cursor));
                has_newline = true;
            } else {
                cursor += 1;
            }
        }
        if depth > 0 {
            return Err(vec![Diagnostic::new(
                "MORVA1024",
                "unterminated block comment",
                Span {
                    start: outer_start,
                    end: outer_start + 2,
                },
            )]);
        }
    }
    Ok((cursor, has_newline))
}

fn token(kind: TokenKind, start: usize, end: usize) -> Token {
    Token {
        kind,
        span: Span { start, end },
    }
}
