mod blocks;
mod parts;
mod statement;

pub use parts::{is_identifier, parse_dialogue, parse_value};

pub(crate) use parts::identifier;

pub use crate::condition::parse_condition;

use crate::diagnostics::Diagnostic;
use crate::types::parser::{Stmt, Token};

use blocks::Parser;

pub fn parse_program(tokens: &[Token]) -> (Vec<Stmt>, Vec<Diagnostic>) {
    let mut parser = Parser {
        tokens,
        diagnostics: Vec::new(),
    };
    let scenes = parser.program();
    (scenes, parser.diagnostics)
}
