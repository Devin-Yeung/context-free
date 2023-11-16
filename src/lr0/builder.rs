use crate::lr0::core::{LR0Item, LR0ItemSet};
use crate::utils::symbols;
use bnf::{Grammar, Production, Term};
use std::collections::{HashMap, VecDeque};

pub struct LR0Builder<'grammar> {
    grammar: &'grammar Grammar,
    closures: Vec<LR0ItemSet<'grammar>>,
    transitions: HashMap<(usize, Term), usize>,
}

impl<'grammar> LR0Builder<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> LR0Builder<'grammar> {
        LR0Builder {
            grammar,
            closures: Vec::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn build(&mut self, augmentation: &'grammar Production) {
        let initial = LR0ItemSet::from_iter(vec![LR0Item::from_production(&augmentation).unwrap()]);
        self.build_closure(&initial);
        self.build_transition();
    }

    fn build_closure(&mut self, initial: &LR0ItemSet<'grammar>) {
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

    fn build_transition(&mut self) {
        for closure in &self.closures {
            for term in symbols(&self.grammar) {
                let goto = closure.goto(self.grammar, term);
                if goto.items.is_empty() {
                    continue;
                }
                let goto_index = self.index_of(&goto);
                let cur_index = self.index_of(&closure);
                self.transitions
                    .insert((cur_index, Term::clone(term)), goto_index);
                println!(
                    "goto(I_{}, {}) = I_{}",
                    cur_index,
                    term.to_string(),
                    goto_index
                );
            }
        }
    }

    fn contains(&self, set: &LR0ItemSet<'grammar>) -> bool {
        self.closures.iter().any(|v| v == set)
    }

    fn index_of(&self, set: &LR0ItemSet<'grammar>) -> usize {
        self.closures.iter().position(|v| v == set).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::lr0::builder::LR0Builder;
    use crate::lr0::core::{LR0Item, LR0ItemSet};
    use bnf::{Expression, Grammar, Production, Term};
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
        builder.build_closure(&set);
        assert_eq!(builder.closures.len(), 12);
        for closure in &builder.closures {
            println!("{}", closure);
        }
        builder.build_transition();
    }

    #[test]
    fn exercise() {
        let grammar = r#"
        <S> ::= <O> 'v' <C>
        <O> ::= 'n'
        <C> ::= <S>
        <C> ::= 'n'
        "#
        .parse()
        .unwrap();

        let mut builder = LR0Builder::new(&grammar);
        let augmentation = Production::from_str("<S'> ::= <S>").unwrap();
        builder.build(&augmentation);
        assert_eq!(builder.closures.len(), 8);
        for closure in &builder.closures {
            println!("{}", closure);
        }
    }

    #[test]
    fn check() {
        let grammar = r#"
        <P'> ::= <Q> 'id' <R>
        <Q> ::= 'forall' | 'exist'
        <R> ::= <E> '=' <E>
        <F> ::= <E> '+' <T> | <T>
        <T> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();

        let lhs = Term::from_str("<X>").unwrap();
        let rhs = Expression::from_str("<P'>").unwrap();

        let lr0_item = LR0Item {
            lhs: &lhs,
            rhs: &rhs,
            delimiter: 0,
        };

        let set = LR0ItemSet::from_iter(vec![lr0_item]);

        let mut builder = LR0Builder::new(&grammar);
        dbg!(builder.closures.len());
        for closure in &builder.closures {
            println!("{}", closure);
        }
        builder.build_closure(&set);
    }
}
