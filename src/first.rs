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

    fn first(&mut self) -> HashMap<&Term, HashSet<&str>> {
        let mut first: HashMap<&Term, HashSet<&str>> = HashMap::new();

        // initialize the first table
        self.symbols().into_iter().for_each(|t| {
            match t {
                Term::Terminal(s) => {
                    // Rule1: If X is a terminal, then First(X) = { X }
                    first.insert(t, HashSet::from([s.as_str()]));
                    println!("Push {} to First({})", s, t.to_string());
                }
                Term::Nonterminal(_) => {
                    first.insert(t, HashSet::new());
                }
            };

            if self.produce_epsilon(t) {
                // Rule2: If X is an ε-production, then add ε to First(X)
                first.get_mut(t).unwrap().insert("ε");
                println!("Push ε to First({})", t.to_string());
            }
        });

        loop {
            let mut changed = false;

            // Rule3: If X is a non-terminal and X → Y1 Y2 ... Yk,
            // then add First(Y1) ∖ {ε} to First(X)
            self.symbols()
                .iter()
                .filter(|term| matches!(*term, Term::Nonterminal(_)))
                .for_each(|term| {
                    println!("Checking Symbol: {}", term.to_string());
                    let production = self.lookup.get(term).unwrap();
                    for expr in production.rhs_iter() {
                        for term in expr
                            .terms_iter()
                            .filter(|term| term != &&Term::Terminal("ε".to_string()))
                        {
                            // First(Y1) ∖ {ε} to First(X)
                            let mut set = first
                                .get(term)
                                .map_or_else(|| HashSet::new(), |set| set.clone());
                            set.remove("ε");
                            // Push into First(X) and check if change or not
                            let before = first.get(term).unwrap().len();
                            first.get_mut(&production.lhs).unwrap().extend(&set);
                            println!("Push {:?} to First({})", set, production.lhs.to_string());
                            let after = first.get(term).unwrap().len();
                            if before != after {
                                changed = true;
                            }
                            // terminate (check next expression) if X does NOT produce ε
                            if !self.produce_epsilon(term) {
                                println!("{} does NOT produce ε", term.to_string());
                                break;
                            }
                        }
                    }
                });

            if !changed {
                break;
            }
        } // End of loop

        first
    }

    fn produce_epsilon(&self, term: &Term) -> bool {
        let production = self.lookup.get(&term);
        if production.is_none() {
            return false;
        }

        let production = production.unwrap();

        match &production.lhs {
            Term::Terminal(t) => {
                if t == "ε" {
                    return true;
                }
            }
            Term::Nonterminal(nt) => {
                if nt == "ε" {
                    return true;
                }
            }
        }

        production
            .rhs_iter()
            .map(|expr| {
                expr.terms_iter().all(|term| match term {
                    Term::Terminal(t) => t == "ε",
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
                        if !s.is_empty() && s != "ε" {
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
        <E'> ::= '+' <T> <E'> | 'ε'
        <T> ::= <F> <T'>
        <T'> ::= '*' <F> <T'> | 'ε'
        <F> ::= '(' <E> ')' | 'id'
        "#;
        let grammar: Grammar = input.parse().unwrap();
        grammar
    }

    #[test]
    fn first() {
        let grammar = grammar();
        let mut first = First::new(&grammar);
        dbg!(first.first());
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
            ["+", "*", "(", ")", "id", "F", "E", "E'", "T'", "T"].into()
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
