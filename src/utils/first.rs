use crate::utils::builder::FirstFollowBuilder;
use bnf::{Grammar, Production, Term};
use once_cell::sync::OnceCell;
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

    pub fn epsilon() -> &'static Term {
        static EPSILON: OnceCell<Term> = OnceCell::new();
        EPSILON.get_or_init(|| Term::Terminal(String::from("ε")))
    }

    fn first(&mut self) -> HashMap<&Term, HashSet<&Term>> {
        let mut builder: FirstFollowBuilder = FirstFollowBuilder::new(&self.grammar);

        // initialize the first table
        self.symbols().into_iter().for_each(|t| {
            match t {
                Term::Terminal(s) => {
                    // Rule1: If X is a terminal, then First(X) = { X }
                    builder.insert_term(t, t);
                    println!("Rule1: Push {} to First({})", s, t.to_string());
                }
                Term::Nonterminal(_) => { /* skip */ }
            };

            if self.produce_epsilon(t) {
                // Rule2: If X is an ε-production, then add ε to First(X)
                builder.insert_epsilon(t);
                println!("Rule2: Push ε to First({})", t.to_string());
            }
        });

        loop {
            let mut changed = false;

            self.symbols()
                .iter()
                .filter(|term| matches!(*term, Term::Nonterminal(_)))
                .for_each(|lhs| {
                    println!("===> Checking Symbol: {}", lhs.to_string());
                    let production = self.lookup.get(lhs).unwrap();
                    // Rule3: If X is a non-terminal and X → Y1 Y2 ... Yk,
                    // then add First(Y1) ∖ {ε} to First(X)
                    for expr in production.rhs_iter() {
                        for term in expr
                            .terms_iter()
                            .filter(|term| term != &&Term::Terminal("ε".to_string()))
                        {
                            // First(Y1) ∖ {ε} to First(X)
                            changed |= builder.insert_first_no_epsilon(&production.lhs, term);
                            println!(
                                "Rule3/4: Push First({}) \\ ε to First({})",
                                term,
                                production.lhs.to_string()
                            );
                            // terminate (check next expression) if X does NOT produce ε
                            if !self.produce_epsilon(term) {
                                println!("{} does NOT produce ε", term.to_string());
                                break;
                            }
                        }
                        // Rule 5: If X is a non-terminal and X -> Y1 Y2 ... Yk,
                        // and First(Yi) produce ε for all i, then add ε to First(X)
                        if expr.terms_iter().all(|term| self.produce_epsilon(term)) {
                            println!("Rule5: Push ε to First({})", production.lhs.to_string());
                            changed = builder.insert_epsilon(&production.lhs);
                        }
                    }
                });

            if !changed {
                println!("Unchanged, break!");
                break;
            }
        } // End of loop

        builder.build()
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
    use crate::utils::first::First;
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
        first.first().iter().for_each(|(lhs, rhs)| match lhs {
            Term::Terminal(_) => {
                assert_eq!(rhs.len(), 1)
            }
            Term::Nonterminal(_) => {
                assert_eq!(rhs.len(), 2)
            }
        })
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
