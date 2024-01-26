use crate::slr::helper::IndexedGrammar;
use crate::utils::dollar;
use bnf::Term;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::once;
use tabled::builder::Builder;
use tabled::Table;

#[derive(Clone, Serialize)]
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

fn strip_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

impl<'grammar> Serialize for SLRTable<'grammar> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = s.serialize_struct("SLRTable", 2)?;

        state.serialize_field("grammar", &self.grammar)?;

        let serialized_table: Vec<IndexMap<String, &SLRInstruction>> = self
            .table
            .iter()
            .map(|map| {
                map.iter()
                    .sorted_by_key(|(term, _)| *term)
                    .map(|(&term, instr)| (strip_quotes(&term.to_string()).to_string(), instr))
                    .collect::<IndexMap<_, _>>()
            })
            .collect();
        state.serialize_field("table", &serialized_table)?;
        state.end()
    }
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

#[cfg(test)]
mod tests {
    use crate::slr::builder::SLRTableBuilder;
    use bnf::Production;
    use std::str::FromStr;

    #[test]
    fn slr_to_json() {
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
        let json = serde_json::to_string_pretty(&slr).unwrap();
        insta::assert_display_snapshot!(json);
    }
}
