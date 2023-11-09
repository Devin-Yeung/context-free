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

    fn symbols(&self) -> HashSet<&str> {
        let mut symbols = HashSet::new();
        self.grammar.productions_iter().for_each(|production| {
            production.rhs_iter().for_each(|expr| {
                expr.terms_iter().for_each(|term| {
                    match term {
                        Term::Terminal(ref s) => {
                            if !s.is_empty() {
                                symbols.insert(s.as_str());
                            }
                        }
                        _ => { /* skip */ }
                    };
                })
            });
        });
        symbols
    }
}

#[cfg(test)]
mod tests {
    use crate::first::First;
    use bnf::Grammar;

    pub fn grammar() -> Grammar {
        let input = r#"
        <E> ::= <T> <E'>
        <E'> ::= '+' <T> <E'> | <𝜀>
        <T> ::= <F> <T'>
        <T'> ::= '*' <F> <T'> | <𝜀>
        <F> ::= '(' <E> ')' | 'id'
        <𝜀> ::= ''
        "#;
        let grammar: Grammar = input.parse().unwrap();
        grammar
    }

    #[test]
    fn symbols() {
        let grammar = grammar();
        let first = First::new(&grammar);
        assert_eq!(first.symbols(), ["+", "*", "(", ")", "id"].into());
    }
}
