use bnf::Term;
use std::collections::{HashMap, HashSet};

mod builder;

pub struct Follow<'grammar> {
    pub(crate) follow: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}
