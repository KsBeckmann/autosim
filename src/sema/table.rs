use crate::parser::AutomatonKind;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug)]
pub struct SymbolTable {
    pub alphabet: HashMap<char, Range<usize>>,
    pub automata: HashMap<String, AutomatonEntry>,
}

#[derive(Debug)]
pub struct AutomatonEntry {
    pub kind: AutomatonKind,
    pub declared_at: Range<usize>,
    pub states: HashMap<String, Range<usize>>,
    pub initial: String,
    pub finals: HashMap<String, Range<usize>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            alphabet: HashMap::new(),
            automata: HashMap::new(),
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
