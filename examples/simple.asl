// simple.asl
// AFN: de q0 saem duas transicoes com 'a' (uma volta para q0 e outra
// vai para q1), o que so e permitido em um AFN.

alfabeto { 'a' }

automato AFN simple {
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
