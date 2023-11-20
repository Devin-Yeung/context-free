use crate::lr0::core::{LR0Closure, LR0Item, LR0ItemSet};
use crate::utils::symbols;
use bnf::{Grammar, Production, Term};
use std::collections::{HashMap, VecDeque};

pub struct LR0Builder<'grammar> {
    grammar: &'grammar Grammar,
    closures: Vec<LR0ItemSet<'grammar>>,
    transitions: HashMap<(usize, &'grammar Term), usize>,
}

impl<'grammar> LR0Builder<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> LR0Builder<'grammar> {
        LR0Builder {
            grammar,
            closures: Vec::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn build(mut self, augmentation: &'grammar Production) -> LR0Closure {
        let initial = LR0ItemSet::from_iter(vec![LR0Item::from_production(augmentation).unwrap()]);
        self.build_closure(&initial);
        self.build_transition();
        LR0Closure {
            closures: self.closures,
            transitions: self.transitions,
        }
    }

    fn build_closure(&mut self, initial: &LR0ItemSet<'grammar>) {
        let mut waiting = VecDeque::from([initial.closure(self.grammar)]);
        self.closures.push(initial.closure(self.grammar));

        while !waiting.is_empty() {
            let cur = waiting.pop_front().unwrap();
            symbols(self.grammar).iter().for_each(|term| {
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
            for term in symbols(self.grammar) {
                let goto = closure.goto(self.grammar, term);
                if goto.items.is_empty() {
                    continue;
                }
                let goto_index = self.index_of(&goto);
                let cur_index = self.index_of(closure);
                self.transitions.insert((cur_index, term), goto_index);
                println!("goto(I_{}, {}) = I_{}", cur_index, term, goto_index);
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
    use crate::lr0::core::LR0Closure;
    use bnf::Production;
    use std::str::FromStr;

    #[test]
    fn it_works() {
        let grammar = r#"
        <E'> ::= <E>
        <E> ::= <E> '+' <T> | <T>
        <T> ::= <T> '*' <F> | <F>
        <F> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();
        let augmentation = Production::from_str("<E'> ::= <E>").unwrap();

        assert_eq!(
            LR0Closure::new(&grammar, &augmentation).closures().len(),
            12
        );
    }

    #[test]
    fn object_clause() {
        let grammar = r#"
        <S> ::= <O> 'v' <C>
        <O> ::= 'n'
        <C> ::= <S>
        <C> ::= 'n'
        "#
        .parse()
        .unwrap();
        let augmentation = Production::from_str("<S'> ::= <S>").unwrap();

        assert_eq!(LR0Closure::new(&grammar, &augmentation).closures().len(), 8);
    }
}
