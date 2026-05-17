// AFN-epsilon que reconhece a linguagem L = a* b*
// (sequencias de a's seguidas de sequencias de b's, incluindo vazio)

alfabeto { 'a', 'b' }

automato AFN exemplo_afn_eps {
    estados { q0, q1 }
    inicial q0
    finais { q0, q1 }

    transicoes {
        q0 -> q0 com 'a'
        q0 -> q1 com eps
        q1 -> q1 com 'b'
    }
}

// Simulacoes
simular exemplo_afn_eps com ""        // aceita (q0 inicial e final)
simular exemplo_afn_eps com "aaa"     // aceita (loop em q0)
simular exemplo_afn_eps com "bbb"     // aceita (eps para q1, depois b's)
simular exemplo_afn_eps com "aabb"    // aceita (a's em q0, eps, b's em q1)
simular exemplo_afn_eps com "ba"      // rejeita (b em q0 nao tem transicao)
