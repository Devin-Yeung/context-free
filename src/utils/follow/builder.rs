use crate::utils::first::First;
use crate::utils::follow::Follow;
use crate::utils::symbols;
use crate::utils::{dollar, epsilon};
use bnf::{Grammar, Term};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

pub struct FollowBuilder<'grammar> {
    pub(crate) grammar: &'grammar Grammar,
    pub(crate) follow: RefCell<HashMap<&'grammar Term, HashSet<&'grammar Term>>>,
    pub(crate) first: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> FollowBuilder<'grammar> {
    pub(crate) fn new(grammar: &'grammar Grammar) -> FollowBuilder<'grammar> {
        let mut follow = HashMap::new();

        // initialize the table
        symbols(&grammar)
            .into_iter()
            .filter(|term| term != &epsilon()) // epsilon is a special non-terminal
            .for_each(|term| {
                follow.insert(term, HashSet::new());
            });

        let follow = RefCell::new(follow);
        let first = First::new(&grammar).first;

        FollowBuilder {
            grammar,
            follow,
            first,
        }
    }

    pub fn build_follow(&mut self, start: &'grammar Term) {
        // Rule 1: If X is a start symbol, then Follow(X) = { $ }
        self.insert_dollar(start);
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
                            changed |= self.insert_set(term, first_yi);
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
                        changed |= self.insert_follow(term, &production.lhs);

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
    }

    pub fn first_produce_epsilon(&self, term: &Term) -> bool {
        self.first
            .get(term)
            .map_or(false, |first| first.contains(&epsilon()))
    }

    // Insert term to Follow(x)
    ///
    /// return true if the Follow(x) changes
    /// otherwise return false
    pub(crate) fn insert_term(&self, x: &'grammar Term, term: &'grammar Term) -> bool {
        let mut follow = self.follow.borrow_mut();
        // Follow(x)
        let follow_x = follow.get_mut(x).unwrap();

        // Insert term to Follow(x)
        let before = follow_x.len();
        follow_x.insert(term);
        let after = follow_x.len();

        // check if set changes
        before != after
    }

    // Insert epsilon to First(x)
    pub(crate) fn insert_epsilon(&self, x: &'grammar Term) -> bool {
        self.insert_term(x, epsilon())
    }

    // Insert dollar to Follow(x)
    pub(crate) fn insert_dollar(&self, x: &'grammar Term) -> bool {
        self.insert_term(x, dollar())
    }

    // Insert set to Follow(x)
    pub(crate) fn insert_set(&self, x: &'grammar Term, set: HashSet<&'grammar Term>) -> bool {
        let mut follow = self.follow.borrow_mut();
        // Follow(x)
        let follow_x = follow.get_mut(x).unwrap();

        // Insert set into Follow(x)
        let before = follow_x.len();
        follow_x.extend(set);
        let after = follow_x.len();

        // check if set changes
        return before != after;
    }

    pub(crate) fn follow(&self, x: &Term) -> HashSet<&'grammar Term> {
        self.follow
            .borrow()
            .get(x)
            .map_or_else(|| HashSet::new(), |set| set.clone())
    }

    /// Insert Follow(tx) into Follow(rx)
    pub(crate) fn insert_follow(&mut self, rx: &'grammar Term, tx: &'grammar Term) -> bool {
        // Follow(tx)
        let follow_tx = self.follow(tx);
        println!("Insert {:?} to Follow({})", follow_tx, rx);
        // Insert Follow(tx) into Follow(rx)
        self.insert_set(rx, follow_tx)
    }

    pub(crate) fn build(mut self, start: &'grammar Term) -> Follow<'grammar> {
        self.build_follow(start);
        Follow {
            follow: self.follow.into_inner(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::follow::builder::FollowBuilder;
    use crate::utils::follow::Follow;
    use bnf::{Grammar, Term};
    use std::collections::HashSet;

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

    fn get_follow<'a>(follow: &'a Follow, term: &str) -> HashSet<&'a Term> {
        let term = Term::from_str(term).unwrap();
        follow.follow.get(&term).unwrap().clone()
    }

    #[test]
    fn it_works() {
        let grammar = grammar();
        let start = Term::Nonterminal("E".to_string());
        let follow = FollowBuilder::new(&grammar).build(&start);

        assert_eq!(get_follow(&follow, "<E>").len(), 2);
        assert_eq!(get_follow(&follow, "<E'>").len(), 2);
        assert_eq!(get_follow(&follow, "<T>").len(), 3);
        assert_eq!(get_follow(&follow, "<T'>").len(), 3);
        assert_eq!(get_follow(&follow, "<F>").len(), 4);
    }
}
