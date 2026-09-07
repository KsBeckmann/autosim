use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

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

/// Passo elementar do caminho percorrido pelo autômato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathStep {
    pub from: String,
    pub to: String,
    /// `None` identifica uma transição ε, que não consome símbolo da entrada.
    pub symbol: Option<char>,
}

/// Caminho único que o simulador percorre sobre o autômato.
///
/// Quando a cadeia pertence à linguagem, o caminho é aceitador: parte do estado
/// inicial, consome todos os símbolos da entrada e termina em um estado final.
/// Caso contrário, é o caminho que mais se aproxima da aceitação, segundo o
/// critério descrito em [`best_path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub start: String,
    pub steps: Vec<PathStep>,
    pub accepting: bool,
}

impl Path {
    /// Quantidade de símbolos da entrada consumidos ao longo do caminho.
    #[must_use]
    pub fn consumed(&self) -> usize {
        self.steps.iter().filter(|s| s.symbol.is_some()).count()
    }

    /// Estado em que o caminho termina.
    #[must_use]
    pub fn last_state(&self) -> &str {
        self.steps
            .last()
            .map_or(self.start.as_str(), |s| s.to.as_str())
    }
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

fn is_final(automaton: &Automaton, state: &str) -> bool {
    automaton.finals.iter().any(|s| s.value == state)
}

/// Nó do espaço de busca: um estado do autômato somado à quantidade de símbolos
/// já consumidos da entrada.
type Node = (String, usize);

/// Determina o caminho que o simulador exibe.
///
/// A busca em largura percorre todos os pares (estado, símbolos consumidos)
/// alcançáveis a partir do estado inicial e escolhe o melhor deles por dois
/// critérios, nesta ordem:
///
/// 1. maior número de símbolos consumidos da entrada;
/// 2. menor distância até um estado final no grafo do autômato.
///
/// Quando a cadeia pertence à linguagem, esses critérios elegem necessariamente
/// um nó que consumiu a entrada inteira e repousa sobre um estado final, de modo
/// que o caminho reconstruído é aceitador. Quando não pertence, eles elegem o
/// caminho que chega mais longe na entrada e, entre os que empatam, o que
/// termina mais perto de um estado final.
///
/// A busca em largura garante ainda que o caminho escolhido seja o mais curto
/// entre os que levam ao nó vencedor.
fn best_path(automaton: &Automaton, input: &[char]) -> Path {
    let distances = distances_to_final(automaton);
    let start = automaton.initial.value.clone();

    let mut parent: HashMap<Node, (Node, Option<char>)> = HashMap::new();
    let mut seen: HashSet<Node> = HashSet::new();
    let mut queue: VecDeque<Node> = VecDeque::new();

    let origin = (start.clone(), 0);
    seen.insert(origin.clone());
    queue.push_back(origin.clone());
    let mut best = origin;

    while let Some(node) = queue.pop_front() {
        if is_better(&node, &best, &distances) {
            best = node.clone();
        }

        let (state, consumed) = &node;
        for tr in &automaton.transitions {
            if tr.from.value != *state {
                continue;
            }
            let advance = match tr.symbol.value {
                Symbol::Epsilon => Some(((tr.to.value.clone(), *consumed), None)),
                Symbol::Char(c) if *consumed < input.len() && c == input[*consumed] => {
                    Some(((tr.to.value.clone(), consumed + 1), Some(c)))
                }
                Symbol::Char(_) => None,
            };

            if let Some((child, symbol)) = advance
                && seen.insert(child.clone())
            {
                parent.insert(child.clone(), (node.clone(), symbol));
                queue.push_back(child);
            }
        }
    }

    let mut steps = Vec::new();
    let mut cursor = best.clone();
    while let Some((previous, symbol)) = parent.get(&cursor) {
        steps.push(PathStep {
            from: previous.0.clone(),
            to: cursor.0.clone(),
            symbol: *symbol,
        });
        cursor = previous.clone();
    }
    steps.reverse();

    let accepting = best.1 == input.len() && is_final(automaton, &best.0);
    Path {
        start,
        steps,
        accepting,
    }
}

/// Compara dois nós segundo os critérios documentados em [`best_path`].
fn is_better(candidate: &Node, current: &Node, distances: &HashMap<String, usize>) -> bool {
    if candidate.1 != current.1 {
        return candidate.1 > current.1;
    }
    let reach = |state: &String| distances.get(state).copied().unwrap_or(usize::MAX);
    reach(&candidate.0) < reach(&current.0)
}

/// Distância, em número de transições, de cada estado até o estado final mais
/// próximo.
///
/// Calculada por busca em largura sobre o grafo invertido do autômato, partindo
/// simultaneamente de todos os estados finais. Estados ausentes do resultado não
/// alcançam nenhum estado final.
fn distances_to_final(automaton: &Automaton) -> HashMap<String, usize> {
    let mut incoming: HashMap<&str, Vec<&str>> = HashMap::new();
    for tr in &automaton.transitions {
        incoming
            .entry(tr.to.value.as_str())
            .or_default()
            .push(tr.from.value.as_str());
    }

    let mut distances: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    for state in &automaton.finals {
        if distances.insert(state.value.clone(), 0).is_none() {
            queue.push_back((state.value.clone(), 0));
        }
    }

    while let Some((state, distance)) = queue.pop_front() {
        let Some(sources) = incoming.get(state.as_str()) else {
            continue;
        };
        for source in sources {
            if !distances.contains_key(*source) {
                distances.insert((*source).to_string(), distance + 1);
                queue.push_back(((*source).to_string(), distance + 1));
            }
        }
    }

    distances
}
