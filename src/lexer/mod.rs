mod error;
mod token;

pub use error::LexError;
pub use token::Token;

use logos::Logos;
use std::ops::Range;

/// Converte a entrada em uma lista de tokens.
///
/// # Errors
///
/// Retorna um [`LexError`] se a entrada contém uma sequência que não casa com
/// nenhum token.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    Token::lexer(input)
        .spanned()
        .map(|(tok, span)| {
            tok.map_err(|mut e| {
                e.text = input[span.clone()].to_string();
                e.span = span;
                e
            })
        })
        .collect()
}

/// Converte a entrada em tokens, mantendo o intervalo de origem de cada um.
///
/// # Errors
///
/// Retorna um [`LexError`] se a entrada contém uma sequência que não casa com
/// nenhum token.
pub fn tokenize_spanned(input: &str) -> Result<Vec<(Token, Range<usize>)>, LexError> {
    let mut out = Vec::new();
    let mut lex = Token::lexer(input);
    while let Some(result) = lex.next() {
        match result {
            Ok(tok) => out.push((tok, lex.span())),
            Err(_) => {
                return Err(LexError {
                    span: lex.span(),
                    text: lex.slice().to_string(),
                });
            }
        }
    }
    Ok(out)
}
