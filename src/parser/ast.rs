use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub alphabet: Vec<Spanned<char>>,
    pub automata: Vec<Automaton>,
    pub simulations: Vec<Simulation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomatonKind {
    Dfa,
    Nfa,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automaton {
    pub kind: AutomatonKind,
    pub name: Spanned<String>,
    pub states: Vec<Spanned<String>>,
    pub initial: Spanned<String>,
    pub finals: Vec<Spanned<String>>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Char(char),
    Epsilon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub from: Spanned<String>,
    pub to: Spanned<String>,
    pub symbol: Spanned<Symbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simulation {
    pub automaton: Spanned<String>,
    pub input: String,
}
