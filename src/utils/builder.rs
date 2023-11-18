use crate::utils::symbols;
use bnf::{Grammar, Term};
use once_cell::sync::OnceCell;
use std::collections::{HashMap, HashSet};

pub fn epsilon() -> &'static Term {
    static EPSILON: OnceCell<Term> = OnceCell::new();
    EPSILON.get_or_init(|| Term::Terminal(String::from("ε")))
}

fn dollar() -> &'static Term {
    static DOLLAR: OnceCell<Term> = OnceCell::new();
    DOLLAR.get_or_init(|| Term::Terminal(String::from("$")))
}

pub struct FirstFollowBuilder<'grammar> {
    pub(crate) inner: HashMap<&'grammar Term, HashSet<&'grammar Term>>,
}

impl<'grammar> FirstFollowBuilder<'grammar> {
    pub(crate) fn new(grammar: &'grammar Grammar) -> FirstFollowBuilder<'grammar> {
        let mut inner = HashMap::new();

        // initialize the table
        symbols(&grammar)
            .into_iter()
            .filter(|term| term != &epsilon()) // epsilon is a special non-terminal
            .for_each(|term| {
                inner.insert(term, HashSet::new());
            });

        FirstFollowBuilder { inner }
    }

    // Insert term to First(x)
    ///
    /// return true if the First(x) changes
    /// otherwise return false
    pub(crate) fn insert_term(&mut self, x: &'grammar Term, term: &'grammar Term) -> bool {
        // First(x)
        let first_x = self.inner.get_mut(x).unwrap();

        // Insert term to First(x)
        let before = first_x.len();
        first_x.insert(term);
        let after = first_x.len();

        // check if set changes
        before != after
    }

    // Insert epsilon to First(x)
    pub(crate) fn insert_epsilon(&mut self, x: &'grammar Term) -> bool {
        self.insert_term(x, epsilon())
    }

    // Insert dollar to Follow(x)
    pub(crate) fn insert_dollar(&mut self, x: &'grammar Term) -> bool {
        self.insert_term(x, dollar())
    }

    /// First(x)
    pub(crate) fn first(&self, x: &Term) -> HashSet<&'grammar Term> {
        self.inner
            .get(x)
            .map_or_else(|| HashSet::new(), |set| set.clone())
    }

    /// Follow(x)
    pub(crate) fn follow(&self, x: &Term) -> HashSet<&'grammar Term> {
        self.inner
            .get(x)
            .map_or_else(|| HashSet::new(), |set| set.clone())
    }

    pub(crate) fn insert_set(&mut self, x: &'grammar Term, set: HashSet<&'grammar Term>) -> bool {
        // First(x)
        let first_x = self.inner.get_mut(x).unwrap();

        // Insert set into First(x)
        let before = first_x.len();
        first_x.extend(set);
        let after = first_x.len();

        // check if set changes
        return before != after;
    }

    /// Insert First(y) \ { ε } into First(x)
    ///
    /// return true if the First(x) changes
    /// otherwise return false
    pub(crate) fn insert_first_no_epsilon(&mut self, x: &'grammar Term, y: &'grammar Term) -> bool {
        // First(y)
        let mut first_y = self.first(y);
        // First(y) \ { ε }
        first_y.remove(epsilon());
        // Insert First(y) \ { ε } into First(x)
        self.insert_set(x, first_y)
    }

    /// Insert Follow(y) into Follow(x)
    pub(crate) fn insert_follow(&mut self, x: &'grammar Term, y: &'grammar Term) -> bool {
        // Follow(y)
        let follow_y = self.follow(y);
        println!("Insert {:?} to Follow({})", follow_y, x);
        // Insert Follow(y) into Follow(x)
        self.insert_set(x, follow_y)
    }

    pub(crate) fn build(self) -> HashMap<&'grammar Term, HashSet<&'grammar Term>> {
        self.inner
    }
}
