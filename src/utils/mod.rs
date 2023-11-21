use bnf::{Grammar, Term};
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::Iter;

pub mod first;
pub mod follow;

pub fn symbols(grammar: &Grammar) -> HashSet<&Term> {
    grammar
        .productions_iter()
        .flat_map(|production| {
            production
                .rhs_iter()
                .flat_map(|expr| expr.terms_iter())
                .chain(std::iter::once(&production.lhs))
        })
        .collect::<HashSet<_>>()
}

pub fn terminals(grammar: &Grammar) -> impl Iterator<Item = &Term> {
    symbols(&grammar)
        .into_iter()
        .filter(|term| term != &epsilon())
        .filter(|term| matches!(*term, Term::Terminal(_)))
}

pub fn nonterminals(grammar: &Grammar) -> impl Iterator<Item = &Term> {
    symbols(&grammar)
        .into_iter()
        .filter(|term| term != &epsilon())
        .filter(|term| matches!(*term, Term::Nonterminal(_)))
}

pub fn epsilon() -> &'static Term {
    static EPSILON: OnceCell<Term> = OnceCell::new();
    EPSILON.get_or_init(|| Term::Terminal(String::from("ε")))
}

pub fn dollar() -> &'static Term {
    static DOLLAR: OnceCell<Term> = OnceCell::new();
    DOLLAR.get_or_init(|| Term::Terminal(String::from("$")))
}

#[cfg(test)]
mod tests {
    use super::symbols;

    #[test]
    fn count_symbols() {
        let grammar = r#"
        <P> ::= <Q> 'id' <R>
        <Q> ::= '∃' | '∀'
        <R> ::= <E> '=' <E>
        <E> ::= <E> '+' <T> | <T>
        <T> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();

        assert_eq!(symbols(&grammar).len(), 12);
    }
}
