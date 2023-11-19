use bnf::{Expression, Grammar, Term};

type LR0Production<'grammar> = (&'grammar Term, &'grammar Expression);

#[derive(Debug)]
pub struct Lookup<'grammar> {
    grammar: &'grammar Grammar,
    // lookup: HashMap<&'grammar Term, &'grammar Production>,
}

impl<'grammar> Lookup<'grammar> {
    pub fn new(grammar: &'grammar Grammar) -> Lookup<'grammar> {
        Lookup { grammar }
    }

    pub fn get(&self, term: &Term) -> impl IntoIterator<Item = LR0Production<'grammar>> {
        let prod = self
            .grammar
            .productions_iter()
            .filter(|production| production.lhs == *term)
            .flat_map(|production| production.rhs_iter().map(|expr| (&production.lhs, expr)))
            .collect::<Vec<_>>();
        prod
    }

    pub fn productions(&self) -> impl IntoIterator<Item = LR0Production<'grammar>> {
        let prod = self
            .grammar
            .productions_iter()
            .flat_map(|production| production.rhs_iter().map(|expr| (&production.lhs, expr)))
            .collect::<Vec<_>>();
        prod
    }
}

#[cfg(test)]
mod tests {
    use crate::lr0::lookup::{LR0Production, Lookup};
    use bnf::{Grammar, Term};

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
    fn productions() {
        let grammar = grammar();
        let lookup = Lookup::new(&grammar);
        let production: Vec<LR0Production> = lookup.productions().into_iter().collect::<Vec<_>>();
        assert_eq!(production.len(), 7);
    }

    #[test]
    fn term_productions() {
        let grammar = grammar();
        let lookup = Lookup::new(&grammar);
        let production: Vec<LR0Production> = lookup
            .get(&Term::Nonterminal("E".to_string()))
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(production.len(), 2);
    }
}
