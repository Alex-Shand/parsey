use parsey::grammar;

fn main() {
    let grammar = grammar! {
        Sum -> Sum ["+-"] Product;
        Sum -> Product;
        Product -> Product ["*/"] Factor;
        Product -> Factor;
        Factor -> "(" Sum ")";
        Factor -> Number;
        Number -> ["0123456789"] Number;
        Number -> ["0123456789"];
    };

    println!(
        "{:#?}",
        parsey::parse(&grammar, "(1+2)*3/500").map(|i| i.collect::<Vec<_>>())
    )
}
