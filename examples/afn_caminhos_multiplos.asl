// AFN que reconhece as cadeias sobre {a, b} cujo terceiro simbolo
// contado a partir do fim e' 'a'.
//
// O objetivo deste exemplo e' tornar visivel o paralelismo do AFN: a
// nuvem de estados chega a conter tres estados simultaneos e, em varios
// passos, tres transicoes diferentes sao percorridas ao mesmo tempo.
//
// Note que q0 tem duas transicoes com 'a' (para q0 e para q1). E' esse
// nao-determinismo que faz o automato "apostar" que o simbolo lido e' o
// terceiro a partir do fim, mantendo em paralelo a aposta de que ainda
// nao chegou a hora.

alfabeto { 'a', 'b' }

automato AFN terceiro_do_fim_a {
    estados { q0, q1, q2, q3 }
    inicial q0
    finais { q3 }

    transicoes {
        q0 -> q0 com 'a'    // permanece esperando
        q0 -> q0 com 'b'    // permanece esperando
        q0 -> q1 com 'a'    // aposta: este 'a' e' o terceiro do fim
        q1 -> q2 com 'a'
        q1 -> q2 com 'b'
        q2 -> q3 com 'a'
        q2 -> q3 com 'b'
    }
}

// Aceita: a b a a b  ->  terceiro do fim e' 'a'
// Evolucao da nuvem:
//   {q0} -a-> {q0,q1} -b-> {q0,q2} -a-> {q0,q1,q3} -a-> {q0,q1,q2} -b-> {q0,q2,q3}
// Nos tres ultimos passos, tres transicoes acendem simultaneamente.
simular terceiro_do_fim_a com "abaab"

// Rejeita: a b a b  ->  terceiro do fim e' 'b'
// Observe que q3 entra na nuvem apos o prefixo "aba", mas o simbolo
// seguinte mata esse ramo: q3 nao tem transicao de saida.
simular terceiro_do_fim_a com "abab"

// Rejeita: cadeia curta demais para ter um terceiro simbolo do fim.
simular terceiro_do_fim_a com "ab"
