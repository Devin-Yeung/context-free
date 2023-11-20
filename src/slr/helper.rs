use bnf::{Expression, Grammar, Production, Term};
use indexmap::IndexMap;

pub struct IndexedGrammar<'grammar> {
    /// rhs -> lhs
    grammar: IndexMap<&'grammar Expression, &'grammar Term>,
}

impl<'grammar> IndexedGrammar<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> IndexedGrammar<'grammar> {
        let grammar = grammar
            .productions_iter()
            .flat_map(|production| production.rhs_iter().map(|expr| (expr, &production.lhs)))
            .collect::<IndexMap<&Expression, &Term>>();

        IndexedGrammar { grammar }
    }

    pub fn get_index_of(&self, expr: &Expression) -> Option<usize> {
        self.grammar.get_index_of(expr)
    }

    pub(crate) fn get(&self, expr: &Expression) -> Option<Production> {
        let lhs = self.grammar.get(expr)?;
        Some(Production::from_parts(Term::clone(lhs), vec![expr.clone()]))
    }
}
