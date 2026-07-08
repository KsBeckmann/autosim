use super::error::LexError;
use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
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
            Self::Alphabet => write!(f, "alfabeto"),
            Self::Automaton => write!(f, "automato"),
            Self::DFA => write!(f, "AFD"),
            Self::NFA => write!(f, "AFN"),
            Self::Epsilon => write!(f, "epsilon"),
            Self::States => write!(f, "estados"),
            Self::Initial => write!(f, "inicial"),
            Self::Final => write!(f, "finais"),
            Self::Transitions => write!(f, "transicoes"),
            Self::Simulate => write!(f, "simular"),
            Self::With => write!(f, "com"),
            Self::BraceOpen => write!(f, "{{"),
            Self::BraceClose => write!(f, "}}"),
            Self::Comma => write!(f, ","),
            Self::Arrow => write!(f, "->"),
            Self::CharLiteral(c) => write!(f, "'{c}'"),
            Self::StringLiteral(s) => write!(f, "\"{s}\""),
            Self::Ident(s) => write!(f, "{s}"),
        }
    }
}
