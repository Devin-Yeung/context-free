use crate::utils::follow::builder::FollowBuilder;
use bnf::{Grammar, Term};
use std::collections::{HashMap, HashSet};

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
}
