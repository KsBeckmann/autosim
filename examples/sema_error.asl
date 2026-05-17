// Exemplo de erros SEMANTICOS (passa pelo lexer e parser, falha na analise).
//
// Erros esperados:
//   1. estado inicial 'q5' nao declarado
//   2. estado final 'q9' nao declarado
//   3. simbolo 'z' nao esta no alfabeto
//   4. transicao usa estado destino 'q9' nao declarado

alfabeto { 'a', 'b' }

automato AFD exemplo {
    estados { q0, q1 }
    inicial q5
    finais { q9 }

    transicoes {
        q0 -> q0 com 'z'
        q1 -> q9 com 'a'
    }
}

simular naoexiste com "ab"
