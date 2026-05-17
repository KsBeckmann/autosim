// Erro semantico: AFD com transicao epsilon.
// Transicoes epsilon (vazias) so sao permitidas em AFN.
// Se voce precisa de epsilon, declare o automato como AFN.

alfabeto { 'a' }

automato AFD exemplo {
    estados { q0, q1 }
    inicial q0
    finais { q1 }

    transicoes {
        q0 -> q1 com eps
        q1 -> q1 com 'a'
    }
}
