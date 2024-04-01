use crate::utils::first::builder::FirstBuilder;
use bnf::{Grammar, Term};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use tabled::builder::Builder;
use tabled::Table;

mod builder;

pub struct First<'grammar> {
    pub(crate) first: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> First<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> First<'grammar> {
        FirstBuilder::new(grammar).build()
    }

    pub fn tabled(&self) -> Table {
        let mut table = Builder::new();
        table.push_record(["Term", "First(X)"]);
        for (term, first) in self
            .first
            .iter()
            .filter(|(term, _)| matches!(term, Term::Nonterminal(_)))
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            table.push_record([term.to_string(), first.iter().sorted().join(", ")]);
        }
        table.build()
    }
}

#[cfg(test)]
mod test {
    use crate::utils::first::First;

    #[test]
    fn it_works() {
        let grammar = r#"
        <E> ::= <T> <E'>
        <E'> ::= '+' <T> <E'> | 'ε'
        <T> ::= <F> <T'>
        <T'> ::= '*' <F> <T'> | 'ε'
        <F> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();

        let first = First::new(&grammar);

        insta::assert_snapshot!(first.tabled());
    }

    #[test]
    fn test_case_1() {
        let grammar = r#"
        <P> ::= <Q> 'id' <R>
        <Q> ::= '∃' | '∀'
        <R> ::= <E> '=' <E>
        <E> ::= <T> <E'>
        <E'> ::= '+' <T> <E'> | 'ε'
        <T> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();
        let first = First::new(&grammar);
        insta::assert_snapshot!(first.tabled());
    }
}
