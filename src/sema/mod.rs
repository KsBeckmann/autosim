mod error;
mod table;

use std::collections::HashMap;
use std::ops::Range;

pub use error::SemaError;
pub use table::{AutomatonEntry, SymbolTable};

use crate::parser::{Automaton, AutomatonKind, Program, Symbol};
use crate::sema::error::to_char_span;

pub fn analyse(
    program: &Program,
    token_spans: &[Range<usize>],
) -> Result<SymbolTable, Vec<SemaError>> {
    let mut errors = Vec::new();
    let table = build_table(program, token_spans, &mut errors);
    check_references(program, &table, token_spans, &mut errors);
    check_dfa_determinism(program, token_spans, &mut errors);

    if errors.is_empty() {
        Ok(table)
    } else {
        Err(errors)
    }
}

fn build_table(
    program: &Program,
    token_spans: &[Range<usize>],
    errors: &mut Vec<SemaError>,
) -> SymbolTable {
    let mut table = SymbolTable::new();

    for sym in &program.alphabet {
        let span = to_char_span(&sym.span, token_spans);
        if let Some(prev) = table.alphabet.get(&sym.value) {
            errors.push(
                SemaError::new(
                    format!("símbolo '{}' duplicado no alfabeto", sym.value),
                    span,
                    "declarado novamente aqui",
                )
                .with_note(prev.clone(), "primeira declaração"),
            );
        } else {
            table.alphabet.insert(sym.value, span);
        }
    }

    for aut in &program.automata {
        let entry = build_automaton_entry(aut, token_spans, errors);
        let name_span = to_char_span(&aut.name.span, token_spans);
        if let Some(prev) = table.automata.get(&aut.name.value) {
            errors.push(
                SemaError::new(
                    format!("autômato '{}' declarado duas vezes", aut.name.value),
                    name_span,
                    "redeclarado aqui",
                )
                .with_note(prev.declared_at.clone(), "primeira declaração"),
            );
        } else {
            table.automata.insert(aut.name.value.clone(), entry);
        }
    }

    table
}

fn build_automaton_entry(
    aut: &Automaton,
    token_spans: &[Range<usize>],
    errors: &mut Vec<SemaError>,
) -> AutomatonEntry {
    let mut states = HashMap::new();
    for st in &aut.states {
        let span = to_char_span(&st.span, token_spans);
        if let Some(prev) = states.get(&st.value) {
            errors.push(
                SemaError::new(
                    format!("estado '{}' declarado duas vezes", st.value),
                    span,
                    "redeclarado aqui",
                )
                .with_note(
                    (prev as &Range<usize>).clone(),
                    "primeira
  declaração",
                ),
            );
        } else {
            states.insert(st.value.clone(), span);
        }
    }

    let mut finals = HashMap::new();
    for f in &aut.finals {
        let span = to_char_span(&f.span, token_spans);
        finals.insert(f.value.clone(), span);
    }

    AutomatonEntry {
        kind: aut.kind.clone(),
        declared_at: to_char_span(&aut.name.span, token_spans),
        states,
        initial: aut.initial.value.clone(),
        finals,
    }
}

fn check_references(
    program: &Program,
    table: &SymbolTable,
    token_spans: &[Range<usize>],
    errors: &mut Vec<SemaError>,
) {
    for aut in &program.automata {
        let Some(entry) = table.automata.get(&aut.name.value) else {
            continue;
        };

        // inicial
        if !entry.states.contains_key(&aut.initial.value) {
            errors.push(SemaError::new(
                format!("estado inicial '{}' não declarado", aut.initial.value),
                to_char_span(&aut.initial.span, token_spans),
                "não está em `estados`",
            ));
        }

        // finais
        for f in &aut.finals {
            if !entry.states.contains_key(&f.value) {
                errors.push(SemaError::new(
                    format!("estado final '{}' não declarado", f.value),
                    to_char_span(&f.span, token_spans),
                    "não está em `estados`",
                ));
            }
        }

        // transições
        for t in &aut.transitions {
            if !entry.states.contains_key(&t.from.value) {
                errors.push(SemaError::new(
                    format!("estado '{}' não declarado", t.from.value),
                    to_char_span(&t.from.span, token_spans),
                    "origem desconhecida",
                ));
            }
            if !entry.states.contains_key(&t.to.value) {
                errors.push(SemaError::new(
                    format!("estado '{}' não declarado", t.to.value),
                    to_char_span(&t.to.span, token_spans),
                    "destino desconhecido",
                ));
            }
            if let Symbol::Char(c) = t.symbol.value
                && !table.alphabet.contains_key(&c)
            {
                errors.push(SemaError::new(
                    format!("símbolo '{c}' não está no alfabeto"),
                    to_char_span(&t.symbol.span, token_spans),
                    "fora do alfabeto declarado",
                ));
            }
        }
    }

    // simulações
    for sim in &program.simulations {
        if !table.automata.contains_key(&sim.automaton.value) {
            errors.push(SemaError::new(
                format!("autômato '{}' não declarado", sim.automaton.value),
                to_char_span(&sim.automaton.span, token_spans),
                "não existe",
            ));
        }
    }
}

fn check_dfa_determinism(
    program: &Program,
    token_spans: &[Range<usize>],
    errors: &mut Vec<SemaError>,
) {
    for aut in &program.automata {
        if !matches!(aut.kind, AutomatonKind::Dfa) {
            continue;
        }

        let mut seen: HashMap<(String, char), Range<usize>> = HashMap::new();
        for t in &aut.transitions {
            let Symbol::Char(c) = t.symbol.value else {
                errors.push(SemaError::new(
                    "AFD não pode ter transição epsilon",
                    to_char_span(&t.symbol.span, token_spans),
                    "use AFN se precisar de epsilon",
                ));
                continue;
            };

            let key = (t.from.value.clone(), c);
            let span = to_char_span(&t.from.span, token_spans);
            if let Some(prev) = seen.get(&key) {
                errors.push(
                    SemaError::new(
                        format!(
                            "AFD '{}' não-determinístico: duas saídas de '{}'
  com '{}'",
                            aut.name.value, t.from.value, c
                        ),
                        span,
                        "segunda transição",
                    )
                    .with_note(prev.clone(), "primeira transição"),
                );
            } else {
                seen.insert(key, span);
            }
        }
    }
}
