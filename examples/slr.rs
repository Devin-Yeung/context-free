use bnf::Production;
use context_free::slr::builder::SLRTableBuilder;
use std::str::FromStr;

fn main() {
    let grammar = r#"
        <S'> ::= <P>
        <P> ::= <Q> 'id' <R>
        <Q> ::= '∃' | '∀'
        <R> ::= <E> '=' <E>
	    <E> ::= <E> '+' <T> | <T>
        <T> ::= '(' <E> ')' | 'id'
        "#
    .parse()
    .unwrap();
    let augmentation = Production::from_str("<S'> ::= <P>").unwrap();

    let builder = SLRTableBuilder::new(&grammar, &augmentation);
    let slr = builder.build();

    println!("{}", slr);
}
