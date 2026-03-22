// Comentários com barras duplas

alfabeto { a, b }

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

// Simulações
simular exemplo_afd com "ab"       // Deve aceitar
simular exemplo_afd com "aab"      // Deve aceitar
simular exemplo_afd com "bbb"      // Deve rejeitar
simular exemplo_afd com ""         // Cadeia vazia