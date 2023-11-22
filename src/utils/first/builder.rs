use crate::utils::first::First;
use crate::utils::{epsilon, symbols};
use bnf::{Grammar, Production, Term};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

pub struct FirstBuilder<'grammar> {
    pub(crate) grammar: &'grammar Grammar,
    pub(crate) first: RefCell<HashMap<&'grammar Term, HashSet<&'grammar Term>>>,
    pub(crate) lookup: HashMap<&'grammar Term, &'grammar Production>,
}

impl<'grammar> FirstBuilder<'grammar> {
    pub(crate) fn new(grammar: &'grammar Grammar) -> FirstBuilder<'grammar> {
        let mut first = HashMap::new();

        let lookup = grammar
            .productions_iter()
            .map(|production| (&production.lhs, production))
            .collect::<HashMap<_, _>>();

        // initialize the table
        symbols(grammar)
            .filter(|term| term != &epsilon()) // epsilon is a special non-terminal
            .for_each(|term| {
                first.insert(term, HashSet::new());
            });

        let first = RefCell::new(first);

        FirstBuilder {
            grammar,
            first,
            lookup,
        }
    }

    fn build_first(&mut self) {
        symbols(self.grammar)
            .filter(|term| term != &epsilon())
            .for_each(|t| {
                match t {
                    Term::Terminal(s) => {
                        // Rule1: If X is a terminal, then First(X) = { X }
                        self.insert_term(t, t);
                        println!("Rule1: Push {} to First({})", s, t);
                    }
                    Term::Nonterminal(_) => { /* skip */ }
                };

                if self.produce_epsilon(t) {
                    // Rule2: If X is an ε-production, then add ε to First(X)
                    self.insert_epsilon(t);
                    println!("Rule2: Push ε to First({})", t);
                }
            });

        loop {
            let mut changed = false;

            symbols(self.grammar)
                .filter(|term| matches!(*term, Term::Nonterminal(_)))
                .for_each(|lhs| {
                    println!("===> Checking Symbol: {}", lhs);
                    let production = self.lookup.get(lhs).unwrap();
                    // Rule3: If X is a non-terminal and X → Y1 Y2 ... Yk,
                    // then add First(Y1) ∖ {ε} to First(X)
                    for expr in production.rhs_iter() {
                        for term in expr
                            .terms_iter()
                            .filter(|term| term != &&Term::Terminal("ε".to_string()))
                        {
                            // First(Y1) ∖ {ε} to First(X)
                            changed |= self.insert_first_no_epsilon(&production.lhs, term);
                            println!(
                                "Rule3/4: Push First({}) \\ ε to First({})",
                                term, production.lhs
                            );
                            // terminate (check next expression) if X does NOT produce ε
                            if !self.produce_epsilon(term) {
                                println!("{} does NOT produce ε", term);
                                break;
                            }
                        }
                        // Rule 5: If X is a non-terminal and X -> Y1 Y2 ... Yk,
                        // and First(Yi) produce ε for all i, then add ε to First(X)
                        if expr.terms_iter().all(|term| self.produce_epsilon(term)) {
                            println!("Rule5: Push ε to First({})", production.lhs);
                            changed = self.insert_epsilon(&production.lhs);
                        }
                    }
                });

            if !changed {
                println!("Unchanged, break!");
                break;
            }
        } // End of loop
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

    // Insert term to First(x)
    ///
    /// return true if the First(x) changes
    /// otherwise return false
    fn insert_term(&self, x: &'grammar Term, term: &'grammar Term) -> bool {
        let mut first = self.first.borrow_mut();
        // First(x)
        dbg!(x);
        let first_x = first.get_mut(x).unwrap();

        // Insert term to First(x)
        let before = first_x.len();
        first_x.insert(term);
        let after = first_x.len();

        // check if set changes
        before != after
    }

    // Insert epsilon to First(x)
    fn insert_epsilon(&self, x: &'grammar Term) -> bool {
        self.insert_term(x, epsilon())
    }

    /// First(x)
    fn first(&self, x: &Term) -> HashSet<&'grammar Term> {
        self.first
            .borrow()
            .get(x)
            .map_or_else(HashSet::new, |set| set.clone())
    }

    fn insert_set(&self, x: &'grammar Term, set: HashSet<&'grammar Term>) -> bool {
        let mut first = self.first.borrow_mut();
        // First(x)
        let first_x = first.get_mut(x).unwrap();

        // Insert set into First(x)
        let before = first_x.len();
        first_x.extend(set);
        let after = first_x.len();

        // check if set changes
        before != after
    }

    /// Insert First(y) \ { ε } into First(x)
    ///
    /// return true if the First(x) changes
    /// otherwise return false
    fn insert_first_no_epsilon(&self, x: &'grammar Term, y: &'grammar Term) -> bool {
        // First(y)
        let mut first_y = self.first(y);
        // First(y) \ { ε }
        first_y.remove(epsilon());
        // Insert First(y) \ { ε } into First(x)
        self.insert_set(x, first_y)
    }

    pub(crate) fn build(mut self) -> First<'grammar> {
        self.build_first();
        First {
            first: self.first.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::first::builder::FirstBuilder;
    use bnf::Term;

    #[test]
    fn first() {
        let grammar = r#"
        <E> ::= <T> <E'>
        <E'> ::= '+' <T> <E'> | 'ε'
        <T> ::= <F> <T'>
        <T'> ::= '*' <F> <T'> | 'ε'
        <F> ::= '(' <E> ')' | 'id'
        "#
        .parse()
        .unwrap();

        let first = FirstBuilder::new(&grammar).build();
        first.first.iter().for_each(|(lhs, rhs)| match lhs {
            Term::Terminal(_) => {
                assert_eq!(rhs.len(), 1)
            }
            Term::Nonterminal(_) => {
                assert_eq!(rhs.len(), 2)
            }
        })
    }
}
