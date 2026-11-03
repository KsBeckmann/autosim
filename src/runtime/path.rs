use std::collections::{HashMap, HashSet, VecDeque};

use crate::parser::{Automaton, Symbol};

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
pub(super) fn best_path(automaton: &Automaton, input: &[char]) -> Path {
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
