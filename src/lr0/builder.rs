use crate::lr0::core::LR0ItemSet;
use crate::utils::symbols;
use bnf::Grammar;
use std::collections::VecDeque;

pub struct LR0Builder<'grammar> {
    grammar: &'grammar Grammar,
    closures: Vec<LR0ItemSet<'grammar>>,
}

impl<'grammar> LR0Builder<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> LR0Builder<'grammar> {
        LR0Builder {
            grammar,
            closures: Vec::new(),
        }
    }

    fn build_transition(&mut self, initial: &LR0ItemSet<'grammar>) {
        let mut waiting = VecDeque::from([initial.closure(self.grammar)]);
        self.closures.push(initial.closure(self.grammar));

        while !waiting.is_empty() {
            let cur = waiting.pop_front().unwrap();
            symbols(&self.grammar).iter().for_each(|term| {
                let goto = cur.goto(self.grammar, term);
                if !goto.items.is_empty() && !self.contains(&goto) {
                    self.closures.push(goto);
                    waiting.push_back(self.closures.last().unwrap().clone());
                }
            });
        }
    }

    fn contains(&self, set: &LR0ItemSet<'grammar>) -> bool {
        self.closures.iter().any(|v| v == set)
    }
}

#[cfg(test)]
mod tests {
    use crate::lr0::builder::LR0Builder;
    use crate::lr0::core::{LR0Item, LR0ItemSet};
    use bnf::{Expression, Grammar, Term};
    use std::str::FromStr;

    pub fn grammar() -> Grammar {
        let input = r#"
        <E'> ::= <E>
        <E> ::= <E> '+' <T> | <T>
        <T> ::= <T> '*' <F> | <F>
        <F> ::= '(' <E> ')' | 'id'
        "#;
        let grammar: Grammar = input.parse().unwrap();
        grammar
    }

    #[test]
    fn it_works() {
        let grammar = grammar();
        let lhs = Term::from_str("<E'>").unwrap();
        let rhs = Expression::from_str("<E>").unwrap();

        let lr0_item = LR0Item {
            lhs: &lhs,
            rhs: &rhs,
            delimiter: 0,
        };

        let set = LR0ItemSet::from_iter(vec![lr0_item]);

        let mut builder = LR0Builder::new(&grammar);
        builder.build_transition(&set);
        assert_eq!(builder.closures.len(), 12);
    }
}
