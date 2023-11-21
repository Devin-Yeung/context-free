use crate::lr0::core::LR0Closure;
use crate::slr::core::SLRInstruction;
use crate::slr::helper::IndexedGrammar;
use crate::utils::follow::Follow;
use bnf::{Grammar, Production, Term};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::repeat;

pub struct SLRTable<'grammar> {
    table: Vec<HashMap<&'grammar Term, SLRInstruction>>,
}

impl<'grammar> SLRTable<'grammar> {}

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
        SLRTableBuilder {
            grammar,
            follow,
            closure,
            table,
        }
    }

    fn shift(&self, from: usize, via: &'grammar Term) {
        let to = self.closure.transition(from, via).unwrap();
        println!("Shift: goto(I_{}, {}) = I_{}", from, via, to);
        let mut table = self.table.borrow_mut();
        table
            .get_mut(from)
            .unwrap()
            .insert(via, SLRInstruction::Shift(to));
    }

    pub fn build(self) {
        self.closure.enumerate_lr0().for_each(|(i, lr0)| {
            match lr0.expect() {
                None => {
                    /* Reduce */
                    // let grammar_index = self.grammar.get_index_of(&lr0.rhs).unwrap();
                    let prod = self.grammar.get(lr0.rhs).unwrap();
                    let follow = self.follow.follow_of(&prod.lhs).collect::<Vec<_>>();
                    println!("Reduce: Follow({}) = ({:?}) -> {}", lr0, follow, prod);
                }
                Some(t) => {
                    if matches!(t, Term::Terminal(_)) {
                        // Shift
                        self.shift(i, t);
                    }
                }
            }
        })
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
        builder.build();
    }
}
