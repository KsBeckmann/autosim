// AFD com 14 estados para demonstrar a navegação do diagrama:
// o círculo de estados cresce além da janela e pode ser explorado
// arrastando com o mouse (pan) e usando a rodinha (zoom).
// Reconhece cadeias de 13 a's, com b's opcionais no início.

alfabeto { 'a', 'b' }

automato AFD grande {
    estados { q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11, q12, q13 }
    inicial q0
    finais { q13 }

    transicoes {
        q0 -> q1 com 'a'
        q1 -> q2 com 'a'
        q2 -> q3 com 'a'
        q3 -> q4 com 'a'
        q4 -> q5 com 'a'
        q5 -> q6 com 'a'
        q6 -> q7 com 'a'
        q7 -> q8 com 'a'
        q8 -> q9 com 'a'
        q9 -> q10 com 'a'
        q10 -> q11 com 'a'
        q11 -> q12 com 'a'
        q12 -> q13 com 'a'
        q0 -> q0 com 'b'
        q13 -> q0 com 'b'
    }
}

// Simulações
simular grande com "aaaaaaaaaaaaa"     // aceita (13 a's chegam em q13)
simular grande com "bbaaaaaaaaaaaaa"   // aceita (b's ficam em q0)
simular grande com "aaa"               // rejeita (para em q3)
