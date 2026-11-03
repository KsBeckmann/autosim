use std::collections::BTreeSet;

use super::path::{Path, best_path};
use crate::parser::Automaton;

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

/// Configuração exibida em um instante da simulação.
///
/// O conjunto `current` mantém sempre um único estado, porque o simulador
/// percorre um caminho determinado do autômato em vez da nuvem de estados
/// simultâneos. O tipo permanece um conjunto para preservar a interface
/// consumida pela camada de apresentação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub current: BTreeSet<String>,
    pub consumed: usize,
    pub last_step: LastStep,
}

pub struct Simulator {
    automaton: Automaton,
    input: Vec<char>,
    path: Path,
    history: Vec<Configuration>,
}

impl Simulator {
    #[must_use]
    pub fn new(automaton: Automaton, input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        let path = best_path(&automaton, &input);
        let history = vec![Configuration {
            current: BTreeSet::from([path.start.clone()]),
            consumed: 0,
            last_step: LastStep::None,
        }];
        Self {
            automaton,
            input,
            path,
            history,
        }
    }

    #[must_use]
    pub const fn automaton(&self) -> &Automaton {
        &self.automaton
    }

    #[must_use]
    pub fn input(&self) -> &[char] {
        &self.input
    }

    #[must_use]
    pub const fn path(&self) -> &Path {
        &self.path
    }

    /// Retorna a configuração atual do simulador.
    ///
    /// # Panics
    ///
    /// Causa panic se o histórico estiver vazio, o que nunca acontece porque a
    /// configuração inicial está sempre presente.
    #[must_use]
    pub fn config(&self) -> &Configuration {
        self.history
            .last()
            .expect("histórico do simulador nunca está vazio")
    }

    #[must_use]
    pub fn prev_config(&self) -> Option<&Configuration> {
        if self.history.len() >= 2 {
            Some(&self.history[self.history.len() - 2])
        } else {
            None
        }
    }

    #[must_use]
    pub fn history(&self) -> &[Configuration] {
        &self.history
    }

    #[must_use]
    pub const fn step_count(&self) -> usize {
        self.history.len() - 1
    }

    #[must_use]
    pub fn next_step(&self) -> NextStep {
        match self.path.steps.get(self.step_count()) {
            None => NextStep::Done,
            Some(step) if step.symbol.is_none() => NextStep::Closure,
            Some(_) => NextStep::Char,
        }
    }

    #[must_use]
    pub fn status(&self) -> Status {
        if matches!(self.next_step(), NextStep::Done) {
            Status::Done(if self.path.accepting {
                Verdict::Accepted
            } else {
                Verdict::Rejected
            })
        } else {
            Status::Running
        }
    }

    /// Posição do símbolo em que o caminho ficou preso, quando a cadeia é
    /// rejeitada por falta de transição disponível.
    ///
    /// Retorna `None` quando a cadeia é aceita ou quando ela é consumida por
    /// inteiro sem alcançar um estado final.
    #[must_use]
    pub fn stuck_at(&self) -> Option<usize> {
        let consumed = self.path.consumed();
        (!self.path.accepting && consumed < self.input.len()).then_some(consumed)
    }

    pub fn step_forward(&mut self) -> bool {
        let Some(step) = self.path.steps.get(self.step_count()).cloned() else {
            return false;
        };
        let consumed = self.config().consumed + usize::from(step.symbol.is_some());
        let last_step = step.symbol.map_or(LastStep::Closure, LastStep::Char);
        self.history.push(Configuration {
            current: BTreeSet::from([step.to]),
            consumed,
            last_step,
        });
        true
    }

    pub fn step_back(&mut self) -> bool {
        if self.history.len() <= 1 {
            return false;
        }
        self.history.pop();
        true
    }

    /// Reinicia o simulador para a configuração inicial.
    ///
    /// # Panics
    ///
    /// Causa panic se o histórico estiver vazio, o que nunca acontece porque a
    /// configuração inicial está sempre presente.
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
}
