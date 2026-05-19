use std::collections::BTreeSet;

use crate::parser::{Automaton, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Running,
    Done(Verdict),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastStep {
    None,
    Closure,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextStep {
    Closure,
    Char,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub current: BTreeSet<String>,
    pub consumed: usize,
    pub last_step: LastStep,
}

pub struct Simulator {
    automaton: Automaton,
    input: Vec<char>,
    history: Vec<Configuration>,
}

impl Simulator {
    pub fn new(automaton: Automaton, input: &str) -> Self {
        let initial = BTreeSet::from([automaton.initial.value.clone()]);
        let history = vec![Configuration {
            current: initial,
            consumed: 0,
            last_step: LastStep::None,
        }];
        Self {
            automaton,
            input: input.chars().collect(),
            history,
        }
    }

    pub fn automaton(&self) -> &Automaton {
        &self.automaton
    }

    pub fn input(&self) -> &[char] {
        &self.input
    }

    pub fn config(&self) -> &Configuration {
        self.history
            .last()
            .expect("histórico do simulador nunca está vazio")
    }

    pub fn prev_config(&self) -> Option<&Configuration> {
        if self.history.len() >= 2 {
            Some(&self.history[self.history.len() - 2])
        } else {
            None
        }
    }

    pub fn history(&self) -> &[Configuration] {
        &self.history
    }

    pub fn step_count(&self) -> usize {
        self.history.len() - 1
    }

    pub fn next_step(&self) -> NextStep {
        let cfg = self.config();
        if cfg.current.is_empty() {
            return NextStep::Done;
        }

        match cfg.last_step {
            LastStep::None | LastStep::Char(_) => {
                if self.closure_would_expand(&cfg.current) {
                    NextStep::Closure
                } else if cfg.consumed < self.input.len() {
                    NextStep::Char
                } else {
                    NextStep::Done
                }
            }
            LastStep::Closure => {
                if cfg.consumed < self.input.len() {
                    NextStep::Char
                } else {
                    NextStep::Done
                }
            }
        }
    }

    pub fn status(&self) -> Status {
        match self.next_step() {
            NextStep::Done => {
                let cfg = self.config();
                if cfg.current.iter().any(|s| self.is_final(s)) {
                    Status::Done(Verdict::Accepted)
                } else {
                    Status::Done(Verdict::Rejected)
                }
            }
            _ => Status::Running,
        }
    }

    pub fn step_forward(&mut self) -> bool {
        match self.next_step() {
            NextStep::Done => false,
            NextStep::Closure => {
                let cfg = self.config().clone();
                let closed = epsilon_closure(&self.automaton, cfg.current);
                self.history.push(Configuration {
                    current: closed,
                    consumed: cfg.consumed,
                    last_step: LastStep::Closure,
                });
                true
            }
            NextStep::Char => {
                let cfg = self.config().clone();
                let symbol = self.input[cfg.consumed];
                let mut next = BTreeSet::new();
                for state in &cfg.current {
                    for tr in &self.automaton.transitions {
                        if tr.from.value == *state
                            && let Symbol::Char(c) = tr.symbol.value
                            && c == symbol
                        {
                            next.insert(tr.to.value.clone());
                        }
                    }
                }
                self.history.push(Configuration {
                    current: next,
                    consumed: cfg.consumed + 1,
                    last_step: LastStep::Char(symbol),
                });
                true
            }
        }
    }

    pub fn step_back(&mut self) -> bool {
        if self.history.len() <= 1 {
            return false;
        }
        self.history.pop();
        true
    }

    pub fn reset(&mut self) {
        let first = self
            .history
            .first()
            .cloned()
            .expect("histórico do simulador nunca está vazio");
        self.history.clear();
        self.history.push(first);
    }

    pub fn run_to_end(&mut self) -> Verdict {
        loop {
            match self.status() {
                Status::Done(v) => return v,
                Status::Running => {
                    self.step_forward();
                }
            }
        }
    }

    fn is_final(&self, state: &str) -> bool {
        self.automaton.finals.iter().any(|s| s.value == state)
    }

    fn closure_would_expand(&self, set: &BTreeSet<String>) -> bool {
        for state in set {
            for tr in &self.automaton.transitions {
                if tr.from.value == *state
                    && matches!(tr.symbol.value, Symbol::Epsilon)
                    && !set.contains(&tr.to.value)
                {
                    return true;
                }
            }
        }
        false
    }
}

fn epsilon_closure(automaton: &Automaton, mut set: BTreeSet<String>) -> BTreeSet<String> {
    let mut changed = true;
    while changed {
        changed = false;
        let snapshot: Vec<String> = set.iter().cloned().collect();
        for state in &snapshot {
            for tr in &automaton.transitions {
                if tr.from.value == *state
                    && tr.symbol.value == Symbol::Epsilon
                    && set.insert(tr.to.value.clone())
                {
                    changed = true;
                }
            }
        }
    }
    set
}
