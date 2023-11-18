use crate::utils::builder::{epsilon, FirstFollowBuilder};
use crate::utils::first::First;
use bnf::{Grammar, Production, Term};
use std::collections::{HashMap, HashSet};

pub struct Follow<'grammar> {
    grammar: &'grammar Grammar,
    lookup: HashMap<&'grammar Term, &'grammar Production>,
    first: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> Follow<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> Follow<'grammar> {
        let lookup = grammar
            .productions_iter()
            .map(|production| (&production.lhs, production))
            .collect::<HashMap<_, _>>();

        let first = First::new(grammar).first();

        Follow {
            grammar,
            lookup,
            first,
        }
    }

    pub fn first_produce_epsilon(&self, term: &Term) -> bool {
        self.first
            .get(term)
            .map_or(false, |first| first.contains(&epsilon()))
    }

    pub fn follow(
        &self,
        start: &'grammar Term,
    ) -> HashMap<&'grammar Term, HashSet<&'grammar Term>> {
        let mut follow = FirstFollowBuilder::new(&self.grammar);
        // Rule 1: If X is a start symbol, then Follow(X) = { $ }
        follow.insert_dollar(start);
        println!("Rule 1: Push $ to Follow({})", start.to_string());

        loop {
            let mut changed = false;
            for production in self.grammar.productions_iter() {
                println!("==> Checking production {}", production.lhs.to_string());
                for expr in production.rhs_iter() {
                    // Rule 2 checking
                    let mut prev: Option<&'grammar Term> = None;
                    for (idx, term) in expr
                        .terms_iter()
                        .collect::<Vec<_>>()
                        .iter()
                        .rev()
                        .enumerate()
                    {
                        if idx != 0 && matches!(term, Term::Nonterminal(_)) {
                            let mut first_yi = self
                                .first
                                .get(prev.unwrap())
                                .map_or_else(|| HashSet::new(), |set| set.clone());
                            first_yi.remove(epsilon());
                            println!(
                                "Rule 2: Push First({}) \\ ε to Follow({})",
                                prev.unwrap().to_string(),
                                term.to_string()
                            );
                            changed |= follow.insert_set(term, first_yi);
                        }

                        prev = Some(term);
                    } // Rule 2 checking End

                    // Rule 3 checking
                    for term in expr.terms_iter().collect::<Vec<_>>().iter().rev() {
                        // if
                        if matches!(term, Term::Terminal(_)) {
                            break;
                        }
                        // Rule 3: If X -> Y1 Y2 ... Yk,
                        // then add Follow(X) to Follow(Yk)
                        println!(
                            "Rule 3: Push Follow({}) to Follow({})",
                            production.lhs.to_string(),
                            term.to_string()
                        );
                        changed |= follow.insert_follow(term, &production.lhs);

                        if !self.first_produce_epsilon(term) {
                            println!("{} does not produce ε, break", term.to_string());
                            break;
                        }
                    } // Rule 3 checking End
                }
            }

            if !changed {
                break;
            }
        }

        follow.build()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::follow::Follow;
    use bnf::{Grammar, Term};
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

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

    fn get_follow<'a>(
        follow: &HashMap<&'a Term, HashSet<&'a Term>>,
        term: &str,
    ) -> HashSet<&'a Term> {
        let term = Term::from_str(term).unwrap();
        follow.get(&term).unwrap().clone()
    }

    #[test]
    fn it_works() {
        let grammar = grammar();
        let start = Term::Nonterminal("E".to_string());
        let follow = Follow::new(&grammar).follow(&start);

        assert_eq!(get_follow(&follow, "<E>").len(), 2);
        assert_eq!(get_follow(&follow, "<E'>").len(), 2);
        assert_eq!(get_follow(&follow, "<T>").len(), 3);
        assert_eq!(get_follow(&follow, "<T'>").len(), 3);
        assert_eq!(get_follow(&follow, "<F>").len(), 4);
    }
}
