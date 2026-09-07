//! Testes do simulador, que percorre um caminho único do autômato.
//!
//! Os casos usam os arquivos de `examples/`, de modo que os exemplos
//! distribuídos com a ferramenta funcionem como massa de teste.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path as FsPath, PathBuf};

use autosim::lexer::tokenize_spanned;
use autosim::parser::{Automaton, Program, Symbol, parse};
use autosim::runtime::{Path, Simulator, Verdict};
use autosim::sema::analyse;

fn examples_dir() -> PathBuf {
    FsPath::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn load(name: &str) -> Program {
    let source = fs::read_to_string(examples_dir().join(name)).expect("leitura do exemplo");
    let spanned = tokenize_spanned(&source).expect("lex falhou");
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
    let program = parse(&tokens, &spans).expect("parse falhou");
    analyse(&program, &spans).expect("sema falhou");
    program
}

fn automaton<'a>(program: &'a Program, name: &str) -> &'a Automaton {
    program
        .automata
        .iter()
        .find(|a| a.name.value == name)
        .expect("autômato não encontrado")
}

fn simulate(file: &str, automaton_name: &str, input: &str) -> Simulator {
    let program = load(file);
    let mut sim = Simulator::new(automaton(&program, automaton_name).clone(), input);
    sim.run_to_end();
    sim
}

/// Descreve o caminho na forma `q0 -a-> q1 -ε-> q2`.
fn render(path: &Path) -> String {
    let mut out = path.start.clone();
    for step in &path.steps {
        match step.symbol {
            Some(c) => write!(out, " -{c}-> {}", step.to),
            None => write!(out, " -ε-> {}", step.to),
        }
        .expect("escrita em String não falha");
    }
    out
}

/// Verifica as invariantes que todo caminho deve satisfazer, seja ele
/// aceitador ou não.
fn assert_well_formed(sim: &Simulator, input: &str) {
    let path = sim.path();
    let automaton = sim.automaton();

    assert_eq!(
        path.start, automaton.initial.value,
        "caminho deve partir do estado inicial"
    );

    let mut cursor = path.start.as_str();
    let mut consumed = 0;
    for step in &path.steps {
        assert_eq!(step.from, cursor, "caminho deve ser contíguo");

        let exists = automaton.transitions.iter().any(|tr| {
            tr.from.value == step.from
                && tr.to.value == step.to
                && match (&tr.symbol.value, step.symbol) {
                    (Symbol::Epsilon, None) => true,
                    (Symbol::Char(a), Some(b)) => *a == b,
                    _ => false,
                }
        });
        assert!(
            exists,
            "passo {step:?} não corresponde a nenhuma transição do autômato"
        );

        if step.symbol.is_some() {
            consumed += 1;
        }
        cursor = step.to.as_str();
    }

    assert_eq!(consumed, path.consumed(), "contagem de símbolos consumidos");
    assert_eq!(cursor, path.last_state(), "estado final do caminho");
    assert!(
        consumed <= input.chars().count(),
        "caminho não pode consumir mais do que a entrada"
    );

    let is_final = automaton.finals.iter().any(|s| s.value == *path.last_state());
    assert_eq!(
        path.accepting,
        consumed == input.chars().count() && is_final,
        "veredito deve derivar do caminho"
    );
}

#[test]
fn caminho_aceitador_consome_toda_a_entrada_e_termina_em_estado_final() {
    let sim = simulate("afn_caminhos_multiplos.asl", "terceiro_do_fim_a", "abaab");

    assert_well_formed(&sim, "abaab");
    assert!(sim.path().accepting);
    assert_eq!(sim.path().consumed(), 5);
    assert_eq!(sim.path().last_state(), "q3");
    assert_eq!(render(sim.path()), "q0 -a-> q0 -b-> q0 -a-> q1 -a-> q2 -b-> q3");
    assert_eq!(sim.stuck_at(), None);
}

#[test]
fn cada_configuracao_exibe_um_unico_estado() {
    let sim = simulate("afn_caminhos_multiplos.asl", "terceiro_do_fim_a", "abaab");

    for config in sim.history() {
        assert_eq!(
            config.current.len(),
            1,
            "simulador deve exibir um estado por vez, e não a nuvem de estados"
        );
    }
}

#[test]
fn caminho_usa_transicao_epsilon_quando_necessario() {
    let sim = simulate("afn_epsilon.asl", "exemplo_afn_eps", "aabb");

    assert_well_formed(&sim, "aabb");
    assert!(sim.path().accepting);
    assert_eq!(render(sim.path()), "q0 -a-> q0 -a-> q0 -ε-> q1 -b-> q1 -b-> q1");
}

#[test]
fn cadeia_vazia_aceita_quando_o_estado_inicial_e_final() {
    let sim = simulate("afn_epsilon.asl", "exemplo_afn_eps", "");

    assert_well_formed(&sim, "");
    assert!(sim.path().accepting);
    assert!(sim.path().steps.is_empty());
    assert_eq!(sim.path().last_state(), "q0");
}

#[test]
fn rejeicao_por_falta_de_transicao_avanca_o_maximo_possivel() {
    // "ba" sai de L = a* b*: o 'b' leva a q1 via ε, mas q1 não lê 'a'.
    let sim = simulate("afn_epsilon.asl", "exemplo_afn_eps", "ba");

    assert_well_formed(&sim, "ba");
    assert!(!sim.path().accepting);
    assert_eq!(sim.path().consumed(), 1, "deve consumir o 'b'");
    assert_eq!(sim.stuck_at(), Some(1), "trava no 'a', de índice 1");
}

#[test]
fn rejeicao_com_entrada_consumida_para_no_estado_mais_proximo_do_final() {
    // "abab" tem 'b' como terceiro símbolo do fim, então é rejeitada. Ainda
    // assim o caminho consome a cadeia inteira e termina em q2, a uma única
    // transição de q3, o estado final.
    let sim = simulate("afn_caminhos_multiplos.asl", "terceiro_do_fim_a", "abab");

    assert_well_formed(&sim, "abab");
    assert!(!sim.path().accepting);
    assert_eq!(sim.path().consumed(), 4, "consome a entrada inteira");
    assert_eq!(sim.path().last_state(), "q2");
    assert_eq!(sim.stuck_at(), None, "não travou: a entrada acabou");
}

#[test]
fn navegacao_passo_a_passo_percorre_o_mesmo_caminho() {
    let program = load("afn_epsilon.asl");
    let mut sim = Simulator::new(automaton(&program, "exemplo_afn_eps").clone(), "aabb");

    let mut visited = vec![sim.config().current.clone()];
    while sim.step_forward() {
        visited.push(sim.config().current.clone());
    }
    assert_eq!(visited.len(), sim.path().steps.len() + 1);

    // volta ao início e confirma que o histórico se desfaz por completo
    while sim.step_back() {}
    assert_eq!(sim.step_count(), 0);
    assert_eq!(sim.config().current.iter().next().unwrap(), "q0");

    sim.run_to_end();
    sim.reset();
    assert_eq!(sim.step_count(), 0);
}

#[test]
fn todos_os_exemplos_validos_produzem_caminhos_bem_formados() {
    let mut checked = 0;
    let mut files: Vec<_> = fs::read_dir(examples_dir())
        .expect("pasta examples")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "asl"))
        .collect();
    files.sort();

    for file in files {
        let source = fs::read_to_string(&file).expect("leitura");
        let Ok(spanned) = tokenize_spanned(&source) else {
            continue; // arquivo de erro léxico proposital
        };
        let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();
        let Ok(program) = parse(&tokens, &spans) else {
            continue; // arquivo de erro sintático proposital
        };
        if analyse(&program, &spans).is_err() {
            continue; // arquivo de erro semântico proposital
        }

        for declaration in &program.simulations {
            let aut = automaton(&program, &declaration.automaton.value);
            let mut sim = Simulator::new(aut.clone(), &declaration.input);
            sim.run_to_end();
            assert_well_formed(&sim, &declaration.input);
            checked += 1;
        }
    }

    assert!(
        checked >= 12,
        "esperava exercitar as simulações dos exemplos, mas rodou {checked}"
    );
}

#[test]
fn veredito_dos_exemplos_confere_com_a_linguagem_descrita() {
    // AFN: terceiro símbolo a partir do fim é 'a'
    for (input, expected) in [
        ("abaab", Verdict::Accepted),
        ("abab", Verdict::Rejected),
        ("ab", Verdict::Rejected),
    ] {
        let mut sim = simulate("afn_caminhos_multiplos.asl", "terceiro_do_fim_a", input);
        assert_eq!(sim.run_to_end(), expected, "cadeia {input:?}");
    }

    // AFN-ε: L = a* b*
    for (input, expected) in [
        ("", Verdict::Accepted),
        ("aaa", Verdict::Accepted),
        ("bbb", Verdict::Accepted),
        ("aabb", Verdict::Accepted),
        ("ba", Verdict::Rejected),
    ] {
        let mut sim = simulate("afn_epsilon.asl", "exemplo_afn_eps", input);
        assert_eq!(sim.run_to_end(), expected, "cadeia {input:?}");
    }
}
