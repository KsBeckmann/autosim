use autosim::lexer::tokenize_spanned;
use autosim::parser::parse;
use autosim::sema::{SemaError, SymbolTable, analyse};

fn analyse_ok(input: &str) -> SymbolTable {
    let spanned = tokenize_spanned(input).expect("lex falhou");
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
    let program = parse(&tokens, &spans).expect("parse falhou");
    analyse(&program, &spans).expect("sema falhou")
}

fn analyse_err(input: &str) -> Vec<SemaError> {
    let spanned = tokenize_spanned(input).expect("lex falhou");
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
    let program = parse(&tokens, &spans).expect("parse falhou");
    analyse(&program, &spans).expect_err("esperava erro semântico")
}

fn assert_any(errors: &[SemaError], needle: &str) {
    assert!(
        errors.iter().any(|e| e.message.contains(needle)),
        "nenhum erro contém {:?}. erros: {:#?}",
        needle,
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

#[test]
fn ok_dfa_completo() {
    let input = r#"
        alfabeto { 'a', 'b' }
        automato AFD m {
            estados { q0, q1 }
            inicial q0
            finais { q1 }
            transicoes {
                q0 -> q1 com 'a'
                q1 -> q1 com 'b'
            }
        }
    "#;

    let table = analyse_ok(input);
    assert_eq!(table.alphabet.len(), 2);
    assert!(table.alphabet.contains_key(&'a'));
    assert!(table.alphabet.contains_key(&'b'));
    assert!(table.automata.contains_key("m"));

    let entry = &table.automata["m"];
    assert_eq!(entry.states.len(), 2);
    assert_eq!(entry.initial, "q0");
    assert_eq!(entry.finals.len(), 1);
}

#[test]
fn ok_afn_com_epsilon() {
    let input = r#"
        alfabeto { 'a' }
        automato AFN m {
            estados { q0, q1 }
            inicial q0
            finais { q1 }
            transicoes {
                q0 -> q1 com eps
                q1 -> q1 com 'a'
            }
        }
    "#;

    analyse_ok(input);
}

#[test]
fn ok_simulacao_valida() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
        simular m com "a"
    "#;

    analyse_ok(input);
}

#[test]
fn err_simbolo_duplicado_no_alfabeto() {
    let errors = analyse_err("alfabeto { 'a', 'a' }");
    assert_any(&errors, "duplicado no alfabeto");
}

#[test]
fn err_estado_duplicado() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0, q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "declarado duas vezes");
}

#[test]
fn err_automato_duplicado() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
        automato AFN m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "autômato 'm' declarado duas vezes");
}

#[test]
fn err_inicial_nao_declarado() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q5
            finais { q0 }
            transicoes { }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "estado inicial 'q5' não declarado");
}

#[test]
fn err_final_nao_declarado() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q9 }
            transicoes { }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "estado final 'q9' não declarado");
}

#[test]
fn err_transicao_origem_desconhecida() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes {
                qX -> q0 com 'a'
            }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "estado 'qX' não declarado");
}

#[test]
fn err_simbolo_fora_do_alfabeto() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes {
                q0 -> q0 com 'z'
            }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "símbolo 'z' não está no alfabeto");
}

#[test]
fn err_simular_automato_inexistente() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q0
            finais { q0 }
            transicoes { }
        }
        simular naoexiste com "a"
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "autômato 'naoexiste' não declarado");
}

#[test]
fn err_afd_nao_deterministico() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0, q1, q2 }
            inicial q0
            finais { q2 }
            transicoes {
                q0 -> q1 com 'a'
                q0 -> q2 com 'a'
            }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "não-determinístico");
}

#[test]
fn err_afd_com_epsilon() {
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0, q1 }
            inicial q0
            finais { q1 }
            transicoes {
                q0 -> q1 com eps
            }
        }
    "#;
    let errors = analyse_err(input);
    assert_any(&errors, "AFD não pode ter transição epsilon");
}

#[test]
fn afn_pode_ter_nao_determinismo() {
    // AFN permite múltiplas saídas com o mesmo símbolo — não deve gerar erro.
    let input = r#"
        alfabeto { 'a' }
        automato AFN m {
            estados { q0, q1, q2 }
            inicial q0
            finais { q2 }
            transicoes {
                q0 -> q1 com 'a'
                q0 -> q2 com 'a'
            }
        }
    "#;
    analyse_ok(input);
}

#[test]
fn varios_erros_acumulados() {
    // Garante que sema não para no primeiro erro.
    let input = r#"
        alfabeto { 'a' }
        automato AFD m {
            estados { q0 }
            inicial q5
            finais { q9 }
            transicoes {
                q0 -> q0 com 'z'
            }
        }
    "#;
    let errors = analyse_err(input);
    assert!(
        errors.len() >= 3,
        "esperava ao menos 3 erros, got {}",
        errors.len()
    );
    assert_any(&errors, "q5");
    assert_any(&errors, "q9");
    assert_any(&errors, "'z'");
}
