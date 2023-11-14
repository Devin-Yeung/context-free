use crate::lr0::lookup::Lookup;
use bnf::{Expression, Grammar, Term};
use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LR0Item<'grammar> {
    lhs: &'grammar Term,
    rhs: &'grammar Expression,
    delimiter: usize,
}

#[derive(Debug, Clone)]
pub struct LR0ItemSet<'grammar> {
    items: HashSet<LR0Item<'grammar>>,
}

impl<'grammar> FromIterator<LR0Item<'grammar>> for LR0ItemSet<'grammar> {
    fn from_iter<T: IntoIterator<Item = LR0Item<'grammar>>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<HashSet<_>>(),
        }
    }
}

impl<'grammar> LR0ItemSet<'grammar> {
    pub fn closure(&self, grammar: &'grammar Grammar) -> LR0ItemSet<'grammar> {
        let lookup = Lookup::new(&grammar);

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

    pub fn contains(&self, item: &LR0Item<'grammar>) -> bool {
        self.items.contains(item)
    }
}

#[cfg(test)]
mod test {
    use crate::lr0::core::{LR0Item, LR0ItemSet};
    use bnf::{Expression, Grammar, Term};
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
        let lhs = Term::from_str("<E'>").unwrap();
        let rhs = Expression::from_str("<E>").unwrap();

        let lr0_item = LR0Item {
            lhs: &lhs,
            rhs: &rhs,
            delimiter: 0,
        };

        let set = LR0ItemSet::from_iter(vec![lr0_item]);

        assert_eq!(set.closure(&grammar()).items.len(), 7);
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
                lhs: &lhs,
                rhs: &rhs,
                delimiter: *delimiter,
            }));
        assert_eq!(lr0_set.closure(&grammar()).items.len(), 2);
    }
}
