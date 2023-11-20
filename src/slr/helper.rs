use bnf::{Expression, Grammar, Term};
use indexmap::IndexMap;

pub struct IndexedGrammar<'grammar> {
    /// rhs -> lhs
    grammar: IndexMap<&'grammar Expression, &'grammar Term>,
}

impl<'grammar> IndexedGrammar<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> IndexedGrammar<'grammar> {
        let grammar = grammar
            .productions_iter()
            .map(|production| production.rhs_iter().map(|expr| (expr, &production.lhs)))
            .flatten()
            .collect::<IndexMap<&Expression, &Term>>();

        IndexedGrammar { grammar }
    }
}
