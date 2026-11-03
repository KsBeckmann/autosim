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
    let spanned = tokenize_spanned(input)?;
    Ok(spanned.into_iter().map(|(token, _)| token).collect())
}

/// Converte a entrada em tokens, mantendo o intervalo de origem de cada um.
///
/// # Errors
///
/// Retorna um [`LexError`] se a entrada contém uma sequência que não casa com
/// nenhum token.
pub fn tokenize_spanned(input: &str) -> Result<Vec<(Token, Range<usize>)>, LexError> {
    Token::lexer(input)
        .spanned()
        .map(|(result, span)| match result {
            Ok(token) => Ok((token, span)),
            Err(_) => Err(LexError {
                text: input[span.clone()].to_string(),
                span,
            }),
        })
        .collect()
}
