// Erro semantico: AFD nao-deterministico.
// Existem duas transicoes saindo de q0 com o mesmo simbolo 'a',
// o que e proibido em um AFD (cada par (estado, simbolo) tem
// no maximo uma saida). Use AFN se precisar de nao-determinismo.

alfabeto { 'a' }

automato AFD exemplo {
    estados { q0, q1, q2 }
    inicial q0
    finais { q2 }

    transicoes {
        q0 -> q1 com 'a'
        q0 -> q2 com 'a'
    }
}
