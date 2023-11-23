use bnf::Production;
use context_free::slr::builder::SLRTableBuilder;
use log::LevelFilter;
use std::str::FromStr;

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(
            LevelFilter::from_str(&std::env::var("RUST_LOG").unwrap_or(String::from("info")))
                .unwrap(),
        )
        .init();

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
