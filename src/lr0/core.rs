use crate::lr0::builder::LR0Builder;
use crate::lr0::lookup::Lookup;
use bnf::{Expression, Grammar, Production, Term};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;

use tabled::builder::Builder;
use tabled::Table;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LR0Item<'grammar> {
    pub(crate) lhs: &'grammar Term,
    pub(crate) rhs: &'grammar Expression,
    pub(crate) delimiter: usize,
}

pub struct LR0Closure<'grammar> {
    pub(crate) closures: Vec<LR0ItemSet<'grammar>>,
    pub(crate) transitions: HashMap<(usize, &'grammar Term), usize>,
}

impl<'grammar> LR0Closure<'grammar> {
    pub fn new(
        grammar: &'grammar Grammar,
        augmentation: &'grammar Production,
    ) -> LR0Closure<'grammar> {
        LR0Builder::new(grammar).build(augmentation)
    }

    pub fn closures(&self) -> &Vec<LR0ItemSet<'grammar>> {
        &self.closures
    }

    pub fn closure_table(&self) -> Table {
        let mut builder = Builder::default();
        for set in self.closures.iter() {
            builder.push_record([set.to_string()]);
        }
        builder.index().build()
    }

    /// enumerate all the `from` states
    /// which means `(from, term) = to`
    pub fn enumerate_states(&self) -> impl Iterator<Item = usize> {
        self.transitions
            .keys()
            .map(|(i, _)| *i)
            .chain(self.transitions.values().map(|i| *i))
            .unique()
            .sorted()
    }

    pub fn transition_table(&self) -> Table {
        let mut builder = Builder::default();
        let header = self
            .transitions
            .keys()
            .map(|(_, t)| t)
            .unique()
            .sorted()
            .collect::<Vec<_>>();
        // header
        builder.set_header(
            std::iter::once(String::from("Closure/Symbol"))
                .chain(header.iter().map(|t| t.to_string())),
        );

        self.enumerate_states().for_each(|from| {
            let row = std::iter::once(format!("I_{}", from))
                .chain(header.iter().map(|term| {
                    match self.transition(from, term) {
                        None => String::new(), /* ∅ */
                        Some(to) => format!("I_{}", to),
                    }
                }))
                .collect::<Vec<_>>();
            builder.push_record(row);
        });

        builder.build()
    }

    pub fn enumerate_lr0<'a>(&'a self) -> impl Iterator<Item = (usize, &'a LR0Item<'grammar>)> {
        self.closures
            .iter()
            .enumerate()
            .flat_map(|(i, set)| set.items.iter().map(move |item| (i, item)))
    }

    /// number of states in the closure
    pub fn len(&self) -> usize {
        self.closures.len()
    }

    pub fn transition<'a>(&self, from: usize, via: &Term) -> Option<usize> {
        self.transitions.get(&(from, via)).copied()
    }
}

impl<'grammar> LR0Item<'grammar> {
    pub fn from_production(production: &'grammar Production) -> Option<LR0Item<'grammar>> {
        if production.rhs_iter().count() != 1 {
            return None;
        }
        let rhs = production.rhs_iter().next().unwrap();
        Some(LR0Item {
            lhs: &production.lhs,
            rhs,
            delimiter: 0,
        })
    }
}

impl<'grammar> Display for LR0Item<'grammar> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = Vec::new();
        s.push(self.lhs.to_string());
        s.push("->".to_string());
        let mut rhs = self
            .rhs
            .terms_iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>();
        rhs.insert(self.delimiter, "•".to_string());
        s.extend(rhs);
        f.write_str(&s.join(" "))
    }
}

impl<'grammar> Display for LR0ItemSet<'grammar> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self
            .items
            .iter()
            .map(|i| i.to_string())
            .sorted() // make display result deterministic
            .collect::<Vec<_>>()
            .join(", ");
        f.write_fmt(format_args!("[{}]", s))
    }
}

impl<'grammar> LR0Item<'grammar> {
    pub fn expect(&self) -> Option<&'grammar Term> {
        self.rhs.terms_iter().nth(self.delimiter)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LR0ItemSet<'grammar> {
    pub(crate) items: HashSet<LR0Item<'grammar>>,
}

impl<'grammar> FromIterator<LR0Item<'grammar>> for LR0ItemSet<'grammar> {
    fn from_iter<T: IntoIterator<Item = LR0Item<'grammar>>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<HashSet<_>>(),
        }
    }
}

impl<'grammar> LR0ItemSet<'grammar> {
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }
    pub fn closure(&self, grammar: &'grammar Grammar) -> LR0ItemSet<'grammar> {
        let lookup = Lookup::new(grammar);

        let mut closure = self.clone();

        loop {
            let mut extend = HashSet::new();

            for item in &closure.items {
                if let Some(x) = item.rhs.terms_iter().nth(item.delimiter) {
                    // x is the term after dot
                    for production in lookup.get(x) {
                        let lr0_item = LR0Item {
                            lhs: production.0,
                            rhs: production.1,
                            delimiter: 0,
                        };
                        if !closure.contains(&lr0_item) {
                            extend.insert(lr0_item);
                        }
                    }
                }
            }

            // check if closure change or not
            if extend.is_empty() {
                break;
            } else {
                closure.items.extend(extend);
            }
        }

        closure
    }

    pub fn goto(&self, grammar: &'grammar Grammar, term: &Term) -> LR0ItemSet<'grammar> {
        let items = self
            .items
            .iter()
            .filter_map(|item| {
                if item.expect() == Some(term) {
                    let mut bump = item.clone();
                    bump.delimiter += 1;
                    Some(bump)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let set = LR0ItemSet { items };
        set.closure(grammar)
    }

    pub fn contains(&self, item: &LR0Item<'grammar>) -> bool {
        self.items.contains(item)
    }
}

#[cfg(test)]
mod test {
    use crate::lr0::core::{LR0Closure, LR0Item, LR0ItemSet};
    use bnf::{Expression, Grammar, Production, Term};
    use std::str::FromStr;

    pub fn grammar() -> Grammar {
        let input = r#"
        <E'> ::= <E>
        <E> ::= <E> '+' <T> | <T>
        <T> ::= <T> '*' <F> | <F>
        <F> ::= '(' <E> ')' | 'id'
        "#;
        let grammar: Grammar = input.parse().unwrap();
        grammar
    }

    #[test]
    fn it_works() {
        let grammar = grammar();
        let augmentation = Production::from_str("<E'> ::= <E>").unwrap();
        let closure = LR0Closure::new(&grammar, &augmentation).closure_table();
        insta::assert_display_snapshot!(closure);
    }

    #[test]
    fn deterministic_transition_table() {
        let grammar = grammar();
        let augmentation = Production::from_str("<E'> ::= <E>").unwrap();
        let transitions = LR0Closure::new(&grammar, &augmentation).transition_table();
        insta::assert_display_snapshot!(transitions);
    }

    #[test]
    fn more_items() {
        let set = [("<E'>", "<E>", 1usize), ("<E>", "<E> '+' <T>", 1usize)];
        let set = set
            .into_iter()
            .map(|(lhs, rhs, delimiter)| {
                let lhs = Term::from_str(lhs).unwrap();
                let rhs = Expression::from_str(rhs).unwrap();
                (lhs, rhs, delimiter)
            })
            .collect::<Vec<_>>();

        let lr0_set: LR0ItemSet =
            LR0ItemSet::from_iter(set.iter().map(|(lhs, rhs, delimiter)| LR0Item {
                lhs,
                rhs,
                delimiter: *delimiter,
            }));
        assert_eq!(lr0_set.closure(&grammar()).items.len(), 2);
    }

    #[test]
    fn goto() {
        let grammar = grammar();
        let lhs = Term::from_str("<E'>").unwrap();
        let rhs = Expression::from_str("<E>").unwrap();

        let lr0_item = LR0Item {
            lhs: &lhs,
            rhs: &rhs,
            delimiter: 0,
        };

        let set = LR0ItemSet::from_iter(vec![lr0_item]);
        let I_0 = set.closure(&grammar);
        [
            ("<E>", 2usize),
            ("<T>", 2),
            ("<F>", 1),
            ("'('", 7),
            ("'id'", 1),
            ("<E'>", 0),
            ("'+'", 0),
            ("'*'", 0),
            ("')'", 0),
        ]
        .iter()
        .for_each(|(t, cnt)| {
            let term = Term::from_str(t).unwrap();
            let goto = I_0.goto(&grammar, &term);
            assert_eq!(goto.items.len(), *cnt)
        });
    }
}
