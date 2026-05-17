use autosim::lexer::tokenize_spanned;
use autosim::parser::{parse, AutomatonKind, ParseError, Program, Simulation, Symbol, Transition};

fn parse_ok(input: &str) -> Program {
    let spanned = tokenize_spanned(input).expect("lex falhou");
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
    parse(&tokens, &spans).expect("parse falhou")
}

fn parse_err(input: &str) -> Vec<ParseError> {
    let spanned = tokenize_spanned(input).expect("lex falhou");
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
    parse(&tokens, &spans).expect_err("esperava erro de parse")
}

#[test]
fn alphabet_only() {
    let program = parse_ok("alfabeto { 'a', 'b' }");
    assert_eq!(program.alphabet, vec!['a', 'b']);
    assert!(program.automata.is_empty());
    assert!(program.simulations.is_empty());
}

#[test]
fn empty_alphabet() {
    let program = parse_ok("alfabeto { }");
    assert!(program.alphabet.is_empty());
}

#[test]
fn alphabet_trailing_comma() {
    let program = parse_ok("alfabeto { 'a', 'b', }");
    assert_eq!(program.alphabet, vec!['a', 'b']);
}

#[test]
fn full_dfa() {
    let input = r#"
        alfabeto { 'a', 'b' }

        automato AFD exemplo {
            estados { q0, q1 }
            inicial q0
            finais { q1 }

            transicoes {
                q0 -> q1 com 'a'
                q1 -> q1 com 'b'
            }
        }
    "#;

    let program = parse_ok(input);
    assert_eq!(program.alphabet, vec!['a', 'b']);
    assert_eq!(program.automata.len(), 1);

    let automaton = &program.automata[0];
    assert_eq!(automaton.kind, AutomatonKind::Dfa);
    assert_eq!(automaton.name, "exemplo");
    assert_eq!(automaton.states, vec!["q0", "q1"]);
    assert_eq!(automaton.initial, "q0");
    assert_eq!(automaton.finals, vec!["q1"]);
    assert_eq!(
        automaton.transitions,
        vec![
            Transition { from: "q0".into(), to: "q1".into(), symbol: Symbol::Char('a') },
            Transition { from: "q1".into(), to: "q1".into(), symbol: Symbol::Char('b') },
        ]
    );
}

#[test]
fn nfa_with_epsilon() {
    let input = r#"
        alfabeto { 'a', 'b' }

        automato AFN exemplo {
            estados { q0, q1 }
            inicial q0
            finais { q1 }

            transicoes {
                q0 -> q1 com eps
                q1 -> q1 com 'a'
            }
        }
    "#;

    let program = parse_ok(input);
    let automaton = &program.automata[0];
    assert_eq!(automaton.kind, AutomatonKind::Nfa);
    assert_eq!(automaton.transitions[0].symbol, Symbol::Epsilon);
    assert_eq!(automaton.transitions[1].symbol, Symbol::Char('a'));
}

#[test]
fn empty_transitions() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD vazio {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
    "#;

    let program = parse_ok(input);
    assert!(program.automata[0].transitions.is_empty());
}

#[test]
fn multiple_automata() {
    let input = r#"
        alfabeto { 'a' }

        automato AFD m1 {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }

        automato AFN m2 {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
    "#;

    let program = parse_ok(input);
    assert_eq!(program.automata.len(), 2);
    assert_eq!(program.automata[0].name, "m1");
    assert_eq!(program.automata[0].kind, AutomatonKind::Dfa);
    assert_eq!(program.automata[1].name, "m2");
    assert_eq!(program.automata[1].kind, AutomatonKind::Nfa);
}

#[test]
fn simulations() {
    let input = r#"
        alfabeto { 'a', 'b' }

        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }

        simular m com "aab"
        simular m com ""
    "#;

    let program = parse_ok(input);
    assert_eq!(
        program.simulations,
        vec![
            Simulation { automaton: "m".into(), input: "aab".into() },
            Simulation { automaton: "m".into(), input: "".into() },
        ]
    );
}

#[test]
fn case_insensitive_keywords() {
    let input = r#"
        ALFABETO { 'a' }
        AUTOMATO afd m {
            ESTADOS { q0 }
            INICIAL q0
            FINAIS { q0 }
            TRANSICOES { q0 -> q0 COM 'a' }
        }
        SIMULAR m COM "a"
    "#;

    let program = parse_ok(input);
    assert_eq!(program.automata[0].kind, AutomatonKind::Dfa);
    assert_eq!(program.simulations.len(), 1);
}

#[test]
fn err_missing_initial_keyword() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            q0
            finais { q0 }
            transicoes { }
        }
    "#;

    let errors = parse_err(input);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("inicial"));
}

#[test]
fn err_ident_in_alphabet() {
    let errors = parse_err("alfabeto { a }");
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("símbolo do alfabeto"));
}

#[test]
fn err_missing_kind() {
    let input = r#"
        alfabeto { 'a' }
        automato m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
    "#;

    let errors = parse_err(input);
    assert!(!errors.is_empty());
}

#[test]
fn err_unterminated_block() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
    "#;

    let errors = parse_err(input);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("arquivo terminou") || errors[0].message.contains("fim do arquivo"));
}

#[test]
fn err_invalid_transition_symbol() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0, q1 }
            inicial q0
            finais { q1 }
            transicoes {
                q0 -> q1 com q2
            }
        }
    "#;

    let errors = parse_err(input);
    assert!(!errors.is_empty());
}
