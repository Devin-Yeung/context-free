use bnf::{Grammar, Term};
use once_cell::sync::OnceCell;
use std::collections::HashSet;

pub mod builder;
pub mod first_v1;
pub mod follow_v1;

pub mod first;
pub fn symbols(grammar: &Grammar) -> HashSet<&Term> {
    grammar
        .productions_iter()
        .flat_map(|production| production.rhs_iter())
        .flat_map(|expr| expr.terms_iter())
        .collect::<HashSet<_>>()
}

pub fn epsilon() -> &'static Term {
    static EPSILON: OnceCell<Term> = OnceCell::new();
    EPSILON.get_or_init(|| Term::Terminal(String::from("ε")))
}
