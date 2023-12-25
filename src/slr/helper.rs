use bnf::{Expression, Grammar, Production, Term};
use indexmap::IndexMap;
use itertools::Itertools;
use tabled::builder::Builder;
use tabled::Table;

pub struct IndexedGrammar<'grammar> {
    /// rhs -> lhs
    grammar: IndexMap<&'grammar Expression, &'grammar Term>,
    /// original
    original: &'grammar Grammar,
}

impl<'grammar> IndexedGrammar<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> IndexedGrammar<'grammar> {
        let original = grammar;
        let grammar = grammar
            .productions_iter()
            .flat_map(|production| production.rhs_iter().map(|expr| (expr, &production.lhs)))
            .collect::<IndexMap<&Expression, &Term>>();

        IndexedGrammar { grammar, original }
    }

    pub fn grammar_table(&self) -> Table {
        let mut builder = Builder::default();
        builder.push_record(["Rule"]);
        self.grammar.iter().for_each(|(rhs, lhs)| {
            builder.push_record([format!("{} -> {}", lhs, rhs)]);
        });
        builder.index().build()
    }

    pub fn get_index_of(&self, expr: &Expression) -> Option<usize> {
        self.grammar.get_index_of(expr)
    }

    pub(crate) fn get(&self, expr: &Expression) -> Option<Production> {
        let lhs = self.grammar.get(expr)?;
        Some(Production::from_parts(Term::clone(lhs), vec![expr.clone()]))
    }

    pub(crate) fn non_terminals<'a>(&'a self) -> impl Iterator<Item = &'grammar Term> {
        self.original
            .productions_iter()
            .map(|prod| &prod.lhs)
            .filter(|term| matches!(*term, Term::Nonterminal(_)))
            .unique()
    }

    pub(crate) fn terminals<'a>(&'a self) -> impl Iterator<Item = &'grammar Term> {
        self.original
            .productions_iter()
            .flat_map(|prod| prod.rhs_iter())
            .flat_map(|expr| expr.terms_iter())
            .filter(|term| matches!(*term, Term::Terminal(_)))
            .unique()
    }
}
