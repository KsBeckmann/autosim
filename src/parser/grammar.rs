use super::ast::{Automaton, AutomatonKind, Program, Simulation, Spanned, Symbol, Transition};
use crate::lexer::Token;
use chumsky::prelude::*;

pub(super) type Err<'a> = extra::Err<Rich<'a, Token>>;

fn spanned<'a, T: 'a, P>(p: P) -> impl Parser<'a, &'a [Token], Spanned<T>, Err<'a>> + Clone
where
    P: Parser<'a, &'a [Token], T, Err<'a>> + Clone,
{
    p.map_with(|v, e| Spanned {
        value: v,
        span: e.span().into(),
    })
}

pub(super) fn ident<'a>() -> impl Parser<'a, &'a [Token], Spanned<String>, Err<'a>> + Clone {
    spanned(select! { Token::Ident(s) => s }).labelled("nome")
}

pub(super) fn char_lit<'a>() -> impl Parser<'a, &'a [Token], Spanned<char>, Err<'a>> + Clone {
    spanned(select! { Token::CharLiteral(c) => c }).labelled("símbolo do alfabeto")
}

pub(super) fn string_lit<'a>() -> impl Parser<'a, &'a [Token], String, Err<'a>> + Clone {
    select! { Token::StringLiteral(s) => s }.labelled("texto entre aspas")
}

pub(super) fn braced_list<'a, T: 'a>(
    item: impl Parser<'a, &'a [Token], T, Err<'a>> + Clone,
) -> impl Parser<'a, &'a [Token], Vec<T>, Err<'a>> + Clone {
    item.separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::BraceOpen), just(Token::BraceClose))
}

pub(super) fn alphabet_decl<'a>()
-> impl Parser<'a, &'a [Token], Vec<Spanned<char>>, Err<'a>> + Clone {
    just(Token::Alphabet).ignore_then(braced_list(char_lit()))
}

pub(super) fn transition<'a>() -> impl Parser<'a, &'a [Token], Transition, Err<'a>> + Clone {
    let symbol = spanned(choice((
        select! { Token::CharLiteral(c) => Symbol::Char(c) },
        just(Token::Epsilon).to(Symbol::Epsilon),
    )));

    ident()
        .then_ignore(just(Token::Arrow))
        .then(ident())
        .then_ignore(just(Token::With))
        .then(symbol)
        .map(|((from, to), symbol)| Transition { from, to, symbol })
}

pub(super) fn simulation<'a>() -> impl Parser<'a, &'a [Token], Simulation, Err<'a>> + Clone {
    just(Token::Simulate)
        .ignore_then(ident())
        .then_ignore(just(Token::With))
        .then(string_lit())
        .map(|(automaton, input)| Simulation { automaton, input })
}

pub(super) fn automaton_decl<'a>() -> impl Parser<'a, &'a [Token], Automaton, Err<'a>> + Clone {
    let kind = choice((
        just(Token::DFA).to(AutomatonKind::Dfa),
        just(Token::NFA).to(AutomatonKind::Nfa),
    ));

    let states_section = just(Token::States).ignore_then(braced_list(ident()));
    let initial_section = just(Token::Initial).ignore_then(ident());
    let finals_section = just(Token::Final).ignore_then(braced_list(ident()));
    let transitions_section = just(Token::Transitions).ignore_then(
        transition()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::BraceOpen), just(Token::BraceClose)),
    );

    let body = states_section
        .then(initial_section)
        .then(finals_section)
        .then(transitions_section)
        .delimited_by(just(Token::BraceOpen), just(Token::BraceClose));

    just(Token::Automaton)
        .ignore_then(kind)
        .then(ident())
        .then(body)
        .map(
            |((kind, name), (((states, initial), finals), transitions))| Automaton {
                kind,
                name,
                states,
                initial,
                finals,
                transitions,
            },
        )
}

pub(super) fn program<'a>() -> impl Parser<'a, &'a [Token], Program, Err<'a>> + Clone {
    #[derive(Clone)]
    enum Item {
        Auto(Automaton),
        Sim(Simulation),
    }

    let item = choice((
        automaton_decl().map(Item::Auto),
        simulation().map(Item::Sim),
    ));

    alphabet_decl()
        .then(item.repeated().collect::<Vec<_>>())
        .map(|(alphabet, items)| {
            let mut automata = Vec::new();
            let mut simulations = Vec::new();
            for it in items {
                match it {
                    Item::Auto(a) => automata.push(a),
                    Item::Sim(s) => simulations.push(s),
                }
            }
            Program {
                alphabet,
                automata,
                simulations,
            }
        })
}
