use std::{error, fmt, ops};
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f\r]+")]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
#[logos(error = LexError)]
pub enum Token {
    // Keywords
    #[token("alfabeto", ignore(case))]
    Alphabet,
    #[token("automato", ignore(case))]
    Automaton,
    #[token("AFD", ignore(case))]
    DFA,
    #[token("estados", ignore(case))]
    States,
    #[token("inicial", ignore(case))]
    Initial,
    #[token("finais", ignore(case))]
    Final,
    #[token("transicoes", ignore(case))]
    Transitions,
    #[token("simular", ignore(case))]
    Simulate,
    #[token("com", ignore(case))]
    With,

    // Symbols
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token(",")]
    Comma,
    #[token("->")]
    Arrow,

    // Literals
    #[regex(r"'[^']'", |lex| lex.slice().chars().nth(1))]
    CharLiteral(char),
    #[regex(r#""[^"]*""#, |lex| { let s = lex.slice(); s[1..s.len()-1].to_string() })]
    StringLiteral(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LexError {
    pub span: ops::Range<usize>,
    pub text: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
    pub file_path: String,
}

impl Default for LexError {
    fn default() -> Self {
        Self {
            span: 0..0,
            text: String::new(),
            line: 0,
            column: 0,
            source_line: String::new(),
            file_path: String::new(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line_num = self.line.to_string();
        let gutter = " ".repeat(line_num.len());
        let padding = " ".repeat(self.column - 1);
        let carets = "^".repeat(self.text.len().max(1));
        write!(
            f,
            "\x1b[1;31merror\x1b[0m: token inválido `{}`\n \x1b[1;34m-->\x1b[0m {}:{}:{}\n  \x1b[1;34m{} |\x1b[0m\n  \x1b[1;34m{} |\x1b[0m {}\n  \x1b[1;34m{} |\x1b[0m {}\x1b[1;31m{}\x1b[0m",
            self.text,
            self.file_path, self.line, self.column,
            gutter,
            line_num, self.source_line,
            gutter, padding, carets,
        )
    }
}

impl error::Error for LexError {}

#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        tokenize(input, "<test>").expect("lexing failed")
    }

    #[test]
    fn keywords() {
        assert_eq!(lex("alfabeto"), vec![Token::Alphabet]);
        assert_eq!(lex("automato"), vec![Token::Automaton]);
        assert_eq!(lex("AFD"), vec![Token::DFA]);
        assert_eq!(lex("estados"), vec![Token::States]);
        assert_eq!(lex("inicial"), vec![Token::Initial]);
        assert_eq!(lex("finais"), vec![Token::Final]);
        assert_eq!(lex("transicoes"), vec![Token::Transitions]);
        assert_eq!(lex("simular"), vec![Token::Simulate]);
        assert_eq!(lex("com"), vec![Token::With]);
    }

    #[test]
    fn keywords_case_insensitive() {
        assert_eq!(lex("ALFABETO"), vec![Token::Alphabet]);
        assert_eq!(lex("Automato"), vec![Token::Automaton]);
        assert_eq!(lex("afd"), vec![Token::DFA]);
        assert_eq!(lex("SIMULAR"), vec![Token::Simulate]);
    }

    #[test]
    fn symbols() {
        assert_eq!(lex("{ } , ->"), vec![
            Token::BraceOpen,
            Token::BraceClose,
            Token::Comma,
            Token::Arrow,
        ]);
    }

    #[test]
    fn char_literal() {
        assert_eq!(lex("'a'"), vec![Token::CharLiteral('a')]);
        assert_eq!(lex("'b'"), vec![Token::CharLiteral('b')]);
    }

    #[test]
    fn string_literal() {
        assert_eq!(lex(r#""ab""#), vec![Token::StringLiteral("ab".to_string())]);
        assert_eq!(lex(r#""""#), vec![Token::StringLiteral("".to_string())]);
    }

    #[test]
    fn identifiers() {
        assert_eq!(lex("q0"), vec![Token::Ident("q0".to_string())]);
        assert_eq!(lex("exemplo_afd"), vec![Token::Ident("exemplo_afd".to_string())]);
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(lex("// isto é um comentário"), vec![]);
        assert_eq!(lex("alfabeto // comentário"), vec![Token::Alphabet]);
    }

    #[test]
    fn whitespace_is_skipped() {
        assert_eq!(lex("   alfabeto   {   }   "), vec![
            Token::Alphabet,
            Token::BraceOpen,
            Token::BraceClose,
        ]);
    }

    #[test]
    fn full_program() {
        let input = r#"
            // Exemplo completo
            alfabeto { a, b }

            automato AFD exemplo {
                estados { q0, q1 }
                inicial q0
                finais { q1 }

                transicoes {
                    q0 -> q1 com 'a'
                }
            }

            simular exemplo com "ab"
        "#;

        let tokens = lex(input);
        assert_eq!(tokens[0], Token::Alphabet);
        assert_eq!(tokens[1], Token::BraceOpen);
        assert_eq!(tokens[2], Token::Ident("a".to_string()));
        assert_eq!(tokens[3], Token::Comma);
        assert_eq!(tokens[4], Token::Ident("b".to_string()));
        assert_eq!(tokens[5], Token::BraceClose);
        assert_eq!(tokens[6], Token::Automaton);
        assert_eq!(tokens[7], Token::DFA);
        assert!(tokens.contains(&Token::Arrow));
        assert!(tokens.contains(&Token::CharLiteral('a')));
        assert!(tokens.contains(&Token::StringLiteral("ab".to_string())));
    }

    #[test]
    fn invalid_token() {
        let err = tokenize("alfabeto %", "<test>").unwrap_err();
        assert_eq!(err.span, 9..10);
    }
}
