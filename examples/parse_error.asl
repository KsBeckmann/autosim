// Erro de parse: faltou a palavra-chave `inicial` antes de q0.
// O lexer aceita (todos os tokens sao validos), mas o parser
// espera `inicial <estado>` apos `estados { ... }`.

alfabeto { 'a', 'b' }

automato AFD exemplo {
    estados { q0, q1 }
    q0
    finais { q1 }

    transicoes {
        q0 -> q1 com 'a'
    }
}
