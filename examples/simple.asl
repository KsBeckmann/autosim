// simple.asl

alfabeto { 'a' }

automato AFD simple {
    estados { q0, q1 }
    inicial q0
    finais { q1 }

    transicoes {
        q0 -> q0 com 'a'
        q0 -> q1 com 'a'
    }
}

// Simulações
simular simple com "aaa"
