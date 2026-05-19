use super::error::LexError;
use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
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
    #[token("AFN", ignore(case))]
    NFA,
    #[token("epsilon", ignore(case))]
    #[token("eps", ignore(case))]
    Epsilon,
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Alphabet => write!(f, "alfabeto"),
            Token::Automaton => write!(f, "automato"),
            Token::DFA => write!(f, "AFD"),
            Token::NFA => write!(f, "AFN"),
            Token::Epsilon => write!(f, "epsilon"),
            Token::States => write!(f, "estados"),
            Token::Initial => write!(f, "inicial"),
            Token::Final => write!(f, "finais"),
            Token::Transitions => write!(f, "transicoes"),
            Token::Simulate => write!(f, "simular"),
            Token::With => write!(f, "com"),
            Token::BraceOpen => write!(f, "{{"),
            Token::BraceClose => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Arrow => write!(f, "->"),
            Token::CharLiteral(c) => write!(f, "'{c}'"),
            Token::StringLiteral(s) => write!(f, "\"{s}\""),
            Token::Ident(s) => write!(f, "{s}"),
        }
    }
}
