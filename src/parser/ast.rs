#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub alphabet: Vec<char>,
    pub automata: Vec<Automaton>,
    pub simulations: Vec<Simulation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutomatonKind {
    Dfa,
    Nfa,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Automaton {
    pub kind: AutomatonKind,
    pub name: String,
    pub states: Vec<String>,
    pub initial: String,
    pub finals: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Char(char),
    Epsilon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Simulation {
    pub automaton: String,
    pub input: String,
}
