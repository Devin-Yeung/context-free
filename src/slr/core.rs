use crate::slr::helper::IndexedGrammar;
use crate::utils::dollar;
use bnf::Term;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::once;
use tabled::builder::Builder;
use tabled::Table;

#[derive(Clone)]
pub enum SLRInstruction {
    Reduce(usize),
    Shift(usize),
    Goto(usize),
    Empty,
}

impl ToString for SLRInstruction {
    fn to_string(&self) -> String {
        match self {
            SLRInstruction::Reduce(i) => format!("r{}", i),
            SLRInstruction::Shift(i) => format!("s{}", i),
            SLRInstruction::Goto(i) => format!("g{}", i),
            SLRInstruction::Empty => String::new(),
        }
    }
}

impl From<&SLRInstruction> for String {
    fn from(val: &SLRInstruction) -> Self {
        val.to_string()
    }
}

pub struct SLRTable<'grammar> {
    pub(crate) grammar: IndexedGrammar<'grammar>,
    pub(crate) table: Vec<HashMap<&'grammar Term, SLRInstruction>>,
}

impl<'grammar> SLRTable<'grammar> {
    pub fn grammar_table(&self) -> Table {
        self.grammar.grammar_table()
    }

    pub fn parsing_table(&self) -> Table {
        let mut builder = Builder::default();

        let header = self
            .grammar
            .terminals()
            .chain(once(dollar()))
            .chain(self.grammar.non_terminals())
            .collect::<Vec<_>>();

        builder.push_record(header.iter().map(|t| t.to_string()));

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

impl<'grammar> Display for SLRTable<'grammar> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Grammar: \n{}\n", self.grammar_table()))?;
        f.write_fmt(format_args!("Table: \n{}", self.parsing_table()))
    }
}
