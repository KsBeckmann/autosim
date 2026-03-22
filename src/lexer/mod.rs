mod token;

use logos::Logos;
pub use token::{Token, LexError};

pub fn tokenize(input: &str, file_path: &str) -> Result<Vec<Token>, LexError> {
    Token::lexer(input)
        .spanned()
        .map(|(tok, span)| {
            tok.map_err(|mut e| {
                let line = input[..span.start].matches('\n').count() + 1;
                let last_newline = input[..span.start].rfind('\n').map_or(0, |p| p + 1);
                let column = span.start - last_newline + 1;
                let line_end = input[span.start..].find('\n').map_or(input.len(), |p| span.start + p);
                e.source_line = input[last_newline..line_end].to_string();
                e.span = span.clone();
                e.text = input[span].to_string();
                e.line = line;
                e.column = column;
                e.file_path = file_path.to_string();
                e
            })
        })
        .collect()
}
