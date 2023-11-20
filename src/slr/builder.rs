use crate::slr::core::SLRInstruction;
use crate::slr::helper::IndexedGrammar;
use crate::utils::follow::Follow;
use bnf::Term;
use std::collections::HashMap;

pub struct SLRTable<'grammar> {
    table: Vec<HashMap<&'grammar Term, SLRInstruction>>,
}

impl<'grammar> SLRTable<'grammar> {}

pub struct SLRTableBuilder<'grammar> {
    grammar: IndexedGrammar<'grammar>,
    follow: Follow<'grammar>,
    table: Vec<HashMap<&'grammar Term, SLRInstruction>>,
}
