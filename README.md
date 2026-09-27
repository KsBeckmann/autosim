# Autosim — DSL para Ensino de Autômatos Finitos com Simulação Visual

Ferramenta educacional do TCC **“Ferramenta Baseada em DSL para o Apoio ao Ensino de
Autômatos Finitos com Simulação Visual Interativa”**
(UTP 2026 — João Thomaz Vieira · Klaus Siegfried Beckmann · Orientador: Prof. Diógenes Cogo Furlan).

Monografia e slides: **https://github.com/KsBeckmann/tcc-utp**

A Autosim permite descrever **AFDs e AFNs com ε** em texto (português, versionável no Git),
valida com diagnósticos localizados em português e anima a simulação passo a passo.

## Exemplo (`.asl`)

```asl
alfabeto { 'a', 'b' }

automato AFD exemplo_afd {
    estados { q0, q1, q2 }
    inicial q0
    finais { q2 }
    transicoes {
        q0 -> q1 com 'a'
        q0 -> q0 com 'b'
        q1 -> q2 com 'b'
        q1 -> q1 com 'a'
        q2 -> q2 com 'a'
        q2 -> q2 com 'b'
    }
}

simular exemplo_afd com "ab"   // aceita
simular exemplo_afd com "bbb"  // rejeita
```

Mais exemplos em `examples/`: `afn_epsilon.asl` (a\*b\*), `grande.asl` (14 estados, pan/zoom),
`sema_error.asl`, `parse_error.asl`, `error_line3.asl` (casos de erro propositais).

## Pré-requisitos

* Rust 1.94+ (`cargo --version`)
* Ambiente gráfico (X11/Wayland) para a GUI em Iced

## Como rodar

```bash
cargo test                          # 51 testes: 12 léxico + 15 sintático + 15 semântico + 9 simulador
cargo run -- --path examples/example1.asl
cargo run -- --path examples/afn_epsilon.asl
cargo run -- --path examples/grande.asl
cargo build --release               # binário otimizado em target/release/autosim
```

Uso da GUI: escolha a simulação no topo → `Passo →` / `← Voltar` / `Rodar-Pausar`
(auto a cada 800 ms) / `Reset` / `Centralizar`. Verde = aceita, vermelho = rejeita.
Diagrama com pan (arrastar) e zoom (rodinha, 20%–400%).

## Arquitetura (pipeline)

```text
.asl → Léxico (Logos: Token) → Sintático (Chumsky: AST em parser/ast.rs)
    → Semântico (sema/: tabela de símbolos + 3 fases)
    → Simulador (runtime/: BFS por caminho único) → GUI (Iced Elm em ui/)
```

* `src/lexer/` — `token.rs` (11 palavras reservadas case-insensitive, `epsilon/eps`, literais, `//` comentários), erros via Ariadne
* `src/parser/` — `grammar.rs` (EBNF: alfabeto único + `automato AFD/AFN` + `simular X com "..."`), AST com spans
* `src/sema/` — duplicatas, referências inválidas, determinismo de AFD (sem ε, sem conflito)
* `src/runtime/` — BFS sobre `(estado, consumidos)`; elege maior consumo + menor distância até final; histórico bidirecional
* `src/ui/` — canvas Iced: layout circular, transições agrupadas, entrada + histórico fixos (HUD)

## Demonstração em vídeo

`demo/demo-autosim.mp4` (1min44s): simulação aceita/rejeita, diagnósticos de erro,
AFN-ε e navegação pan/zoom no diagrama grande.
