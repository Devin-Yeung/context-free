use bnf::{Grammar, Production, Term};
use std::collections::{HashMap, HashSet};

pub struct First<'grammar> {
    grammar: &'grammar Grammar,
    lookup: HashMap<&'grammar str, &'grammar Production>,
}

impl<'grammar> First<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> First<'grammar> {
        let lookup = grammar
            .productions_iter()
            .map(|production| {
                let key = match production.lhs {
                    Term::Terminal(ref s) => s.as_str(),
                    Term::Nonterminal(ref s) => s.as_str(),
                };

                let val = production;
                (key, val)
            })
            .collect::<HashMap<_, _>>();
        First { grammar, lookup }
    }

    fn first(&mut self) {
        let mut first: HashMap<&Term, HashSet<&str>> = HashMap::new();

        // initialize the first table
        self.symbols()
            .into_iter()
            .filter(|t| matches!(*t, Term::Terminal(_)))
            .for_each(|t| {
                if self.produce_epsilon(t) {
                    // Rule2: If X is an ε-production, then add ε to First(X)
                    first.get_mut(t).unwrap().insert("ε");
                }

                match t {
                    Term::Terminal(s) => {
                        // Rule1: If X is a terminal, then First(X) = { X }
                        first.insert(t, HashSet::from([s.as_str()]));
                    }
                    Term::Nonterminal(_) => {
                        first.insert(t, HashSet::new());
                    }
                };
            });

        self.grammar.productions_iter().for_each(|production| {
            production.rhs_iter().for_each(|expr| {
                expr.terms_iter().for_each(|term| {
                    if matches!(*term, Term::Nonterminal(_)) {
                        // Rule3: If 𝑋 is a non-terminal and 𝑋 → 𝑌1 𝑌2 ... 𝑌k,
                        // then add 𝐹𝑖𝑟𝑠𝑡(𝑌1) ∖ {𝜀} to 𝐹𝑖𝑟𝑠𝑡(𝑋)
                    }
                })
            });
        });
    }

    fn produce_epsilon(&self, term: &Term) -> bool {
        match term {
            Term::Terminal(_) => false,
            Term::Nonterminal(nt) => {
                let production = self.lookup.get(nt.as_str()).unwrap();
                production
                    .rhs_iter()
                    .map(|expr| {
                        expr.terms_iter().all(|term| match term {
                            Term::Terminal(_) => false,
                            Term::Nonterminal(nt) => nt == "ε",
                        })
                    })
                    .any(|v| v)
            }
        }
    }

    fn symbols(&'grammar self) -> HashSet<&'grammar Term> {
        let mut symbols = HashSet::new();
        self.grammar.productions_iter().for_each(|production| {
            production.rhs_iter().for_each(|expr| {
                expr.terms_iter().for_each(|term| match term {
                    Term::Terminal(ref s) | Term::Nonterminal(ref s) => {
                        if !s.is_empty() {
                            symbols.insert(term);
                        }
                    }
                });
            });
        });
        symbols
    }
}

#[cfg(test)]
mod tests {
    use crate::first::First;
    use bnf::{Grammar, Term};
    use std::collections::HashSet;

    pub fn grammar() -> Grammar {
        let input = r#"
        <E> ::= <T> <E'>
        <E'> ::= '+' <T> <E'> | <ε>
        <T> ::= <F> <T'>
        <T'> ::= '*' <F> <T'> | <ε>
        <F> ::= '(' <E> ')' | 'id'
        <ε> ::= ''
        "#;
        let grammar: Grammar = input.parse().unwrap();
        grammar
    }

    #[test]
    fn symbols() {
        let grammar = grammar();
        let first = First::new(&grammar);
        assert_eq!(
            first
                .symbols()
                .into_iter()
                .map(|s| match s {
                    Term::Terminal(s) | Term::Nonterminal(s) => {
                        s.as_str()
                    }
                })
                .collect::<HashSet<_>>(),
            ["+", "*", "(", ")", "id", "ε", "F", "E", "E'", "T'", "T"].into()
        );
    }

    #[test]
    fn produce_epsilon() {
        let grammar = grammar();
        let first = First::new(&grammar);
        assert!(first.produce_epsilon(&Term::Nonterminal(String::from("E'"))));
        assert!(first.produce_epsilon(&Term::Nonterminal(String::from("T'"))));
        assert!(!first.produce_epsilon(&Term::Nonterminal(String::from("T"))));
        assert!(!first.produce_epsilon(&Term::Nonterminal(String::from("E"))));
    }
}
