use bnf::{Grammar, Production, Term};
use std::collections::{HashMap, HashSet};

pub struct First<'grammar> {
    grammar: &'grammar Grammar,
    lookup: HashMap<&'grammar Term, &'grammar Production>,
}

impl<'grammar> First<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> First<'grammar> {
        let lookup = grammar
            .productions_iter()
            .map(|production| (&production.lhs, production))
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
                        // Rule3: If X is a non-terminal and X → Y1 Y2 ... Yk,
                        // then add First(Y1) ∖ {𝜀} to First(X)
                        let production = self.lookup.get(term).unwrap();
                        production.rhs_iter().for_each(|expr| {
                            // take the first
                            expr.terms_iter().take(1).for_each(|term0| {
                                match term0 {
                                    Term::Terminal(_) => { /* skip */ }
                                    Term::Nonterminal(_) => {
                                        // First (Y1) ∖ {ε}
                                        let mut set = first.get(term0).unwrap().clone();
                                        set.remove("ε");
                                        first.get_mut(term).unwrap().extend(&set);
                                    }
                                }
                            })
                        })
                    }
                })
            });
        });
    }

    fn produce_epsilon(&self, term: &Term) -> bool {
        let production = self.lookup.get(&term).unwrap();
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
