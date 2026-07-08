pub mod ast;
mod error;
mod grammar;

pub use ast::*;
pub use error::ParseError;

use crate::lexer::Token;
use chumsky::Parser;
use std::ops::Range;

/// Analisa um fluxo de tokens e constrói a AST de um [`Program`].
///
/// # Errors
///
/// Retorna uma lista de [`ParseError`] se os tokens não formam um programa válido.
pub fn parse(tokens: &[Token], token_spans: &[Range<usize>]) -> Result<Program, Vec<ParseError>> {
    grammar::program()
        .parse(tokens)
        .into_result()
        .map_err(|errs| {
            errs.iter()
                .map(|e| ParseError::from_rich(e, token_spans))
                .collect()
        })
}
