use crate::lr0::core::{LR0Closure, LR0Item};
use crate::slr::core::SLRInstruction;
use crate::slr::helper::IndexedGrammar;
use crate::utils::dollar;
use crate::utils::follow::Follow;
use bnf::{Grammar, Production, Term};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::{once, repeat};
use tabled::builder::Builder;
use tabled::Table;

pub struct SLRTable<'grammar> {
    grammar: IndexedGrammar<'grammar>,
    table: Vec<HashMap<&'grammar Term, SLRInstruction>>,
}

impl<'grammar> SLRTable<'grammar> {
    pub fn table(&self) -> Table {
        let mut builder = Builder::default();

        let header = self
            .grammar
            .terminals()
            .chain(once(dollar()))
            .chain(self.grammar.non_terminals())
            .collect::<Vec<_>>();

        builder.set_header(header.iter().map(|t| t.to_string()));

        self.table.iter().enumerate().for_each(|(_, table)| {
            let row = header
                .iter()
                .map(|t| table.get(t).unwrap_or(&SLRInstruction::Empty))
                .collect::<Vec<_>>();
            builder.push_record(row);
        });

        builder.index().build()
    }
}

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

    fn reduce(&self, index: usize, lr0: &LR0Item) {
        let grammar_index = self.grammar.get_index_of(&lr0.rhs).unwrap();
        let prod = self.grammar.get(lr0.rhs).unwrap();
        let mut table = self.table.borrow_mut();
        for term in self.follow.follow_of(&prod.lhs).collect::<Vec<_>>() {
            table[index].insert(term, SLRInstruction::Reduce(grammar_index));
            println!(
                "Reduce: set (I_{}, {}) = r{}",
                index,
                term.to_string(),
                grammar_index
            );
        }
    }

    pub fn build(self) -> SLRTable<'grammar> {
        self.closure.enumerate_lr0().for_each(|(i, lr0)| {
            match lr0.expect() {
                // lr0 expect non character, which is the form of A -> 𝛼 •
                None => {
                    /* Reduce */
                    self.reduce(i, lr0)
                }
                Some(t) => {
                    if matches!(t, Term::Terminal(_)) {
                        // Shift
                        self.shift(i, t);
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
        slr.table();
    }
}