use bnf::{Grammar, Term};
use std::collections::HashSet;

pub fn symbols(grammar: &Grammar) -> HashSet<&Term> {
    grammar
        .productions_iter()
        .flat_map(|production| production.rhs_iter())
        .flat_map(|expr| expr.terms_iter())
        .collect::<HashSet<_>>()
}
