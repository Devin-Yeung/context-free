use crate::utils::first::builder::FirstBuilder;
use bnf::{Grammar, Term};
use std::collections::{HashMap, HashSet};

mod builder;

pub struct First<'grammar> {
    pub(crate) first: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> First<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> First<'grammar> {
        FirstBuilder::new(grammar).build()
    }
}
