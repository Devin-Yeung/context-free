use bnf::Term;
use std::collections::{HashMap, HashSet};

mod builder;

pub struct First<'grammar> {
    pub(crate) first: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}
