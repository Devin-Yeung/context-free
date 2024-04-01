use crate::lr0::core::{LR0Closure, LR0Item};
use crate::slr::core::{SLRInstruction, SLRTable};
use crate::slr::helper::IndexedGrammar;

use crate::utils::follow::Follow;
use bnf::{Grammar, Production, Term};
use std::cell::RefCell;
use std::collections::HashMap;

use log::{debug, info};
use std::iter::repeat;

pub struct SLRTableBuilder<'grammar> {
    grammar: IndexedGrammar<'grammar>,
    follow: Follow<'grammar>,
    closure: LR0Closure<'grammar>,
    table: RefCell<Vec<HashMap<&'grammar Term, SLRInstruction>>>,
}

impl<'grammar> SLRTableBuilder<'grammar> {
    pub fn new(
        grammar: &'grammar Grammar,
        augmentation: &'grammar Production,
    ) -> SLRTableBuilder<'grammar> {
        let follow = Follow::new(grammar, &augmentation.lhs);
        let closure = LR0Closure::new(grammar, augmentation);
        let grammar = IndexedGrammar::new(grammar);
        let table = RefCell::new(
            repeat(HashMap::<&Term, SLRInstruction>::new())
                .take(closure.len())
                .collect(),
        );
        info!("LR0 Closure:\n{}", closure.closure_table());
        info!("LR0 Transition Table:\n{}", closure.transition_table());
        SLRTableBuilder {
            grammar,
            follow,
            closure,
            table,
        }
    }

    fn shift(&self, from: usize, via: &'grammar Term) {
        let to = self.closure.transition(from, via).unwrap(); // TODO: really not sure whether this unwrap is safe?
        debug!("[SLR Builder] Shift: goto(I_{}, {}) = I_{}", from, via, to);
        let mut table = self.table.borrow_mut();
        table
            .get_mut(from)
            .unwrap()
            .insert(via, SLRInstruction::Shift(to));
    }

    fn reduce(&self, index: usize, lr0: &LR0Item) {
        let grammar_index = self.grammar.get_index_of(lr0.rhs).unwrap();
        let prod = self.grammar.get(lr0.rhs).unwrap();
        let mut table = self.table.borrow_mut();
        for term in self.follow.follow_of(&prod.lhs).collect::<Vec<_>>() {
            table[index].insert(term, SLRInstruction::Reduce(grammar_index));
            debug!(
                "[SLR Builder] Reduce: set (I_{}, {}) = r{}",
                index, term, grammar_index
            );
        }
    }

    fn goto(&self, from: usize, via: &'grammar Term) {
        debug_assert!(matches!(via, Term::Nonterminal(_)));
        let to = self.closure.transition(from, via).unwrap(); // TODO: really not sure whether this unwrap is safe?
        debug!("[SLR Builder] Goto: goto(I_{}, {}) = I_{}", from, via, to);
        let mut table = self.table.borrow_mut();
        table
            .get_mut(from)
            .unwrap()
            .insert(via, SLRInstruction::Goto(to));
    }

    pub fn build(self) -> SLRTable<'grammar> {
        self.closure.enumerate_lr0().for_each(|(i, lr0)| {
            // take a look at the char after the •
            match lr0.expect() {
                // lr0 expect non character, which is the form of A -> 𝛼 •
                None => {
                    /* Reduce */
                    self.reduce(i, lr0)
                }
                Some(t) => {
                    match t {
                        Term::Terminal(_) => self.shift(i, t),   /* Shift */
                        Term::Nonterminal(_) => self.goto(i, t), /* Goto */
                    }
                }
            }
        });
        SLRTable {
            grammar: self.grammar,
            table: self.table.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::slr::builder::SLRTableBuilder;
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

        let builder = SLRTableBuilder::new(&grammar, &augmentation);
        let slr = builder.build();
        insta::assert_snapshot!(slr);
    }
}
