use crate::lr0::lookup::Lookup;
use bnf::{Expression, Term};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LR0Item<'grammar> {
    lhs: &'grammar Term,
    rhs: &'grammar Expression,
    delimiter: usize,
}

pub struct LR0ItemSet<'grammar> {
    items: HashMap<Term, LR0Item<'grammar>>,
}

impl<'grammar> LR0ItemSet<'grammar> {
    pub fn closure(&self, grammar: &Lookup) -> LR0ItemSet<'grammar> {
        todo!()
    }
}
