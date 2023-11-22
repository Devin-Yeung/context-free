use crate::utils::follow::builder::FollowBuilder;
use bnf::{Grammar, Term};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use tabled::builder::Builder;
use tabled::Table;

mod builder;

pub struct Follow<'grammar> {
    pub(crate) follow: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> Follow<'grammar> {
    pub fn new(grammar: &'grammar Grammar, start: &'grammar Term) -> Follow<'grammar> {
        FollowBuilder::new(grammar).build(start)
    }

    pub fn follow_of(&self, x: &Term) -> impl Iterator<Item = &&'grammar Term> {
        // TODO: remove the unwrap?
        self.follow.get(x).unwrap().iter()
    }

    pub fn tabled(&self) -> Table {
        let mut table = Builder::new();
        table.set_header(["Term", "Follow(X)"]);
        for (term, first) in self
            .follow
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
mod tests {
    use crate::utils::follow::Follow;
    use bnf::Term;
    use std::str::FromStr;

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
        let start = Term::from_str("<E>").unwrap();
        let follow = Follow::new(&grammar, &start);
        insta::assert_display_snapshot!(follow.tabled());
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
        let start = Term::from_str("<P>").unwrap();
        let follow = Follow::new(&grammar, &start);
        insta::assert_display_snapshot!(follow.tabled());
    }
}
