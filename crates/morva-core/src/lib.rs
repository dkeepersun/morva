mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod semantic;
mod simulate;

pub use ast::*;
pub use diagnostic::Diagnostic;
pub use simulate::{
    PhaseResult, SimulationFailure, SimulationPhase, SimulationReport, StateChange, Value, simulate,
};

pub fn parse(source: &str) -> Result<Document, Vec<Diagnostic>> {
    parser::parse(source)
}

pub fn check(document: &Document) -> Vec<Diagnostic> {
    semantic::check(document)
}
