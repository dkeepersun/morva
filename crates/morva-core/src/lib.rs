mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod semantic;

pub use ast::*;
pub use diagnostic::Diagnostic;

pub fn parse(source: &str) -> Result<Document, Vec<Diagnostic>> {
    parser::parse(source)
}

pub fn check(document: &Document) -> Vec<Diagnostic> {
    semantic::check(document)
}
