use parsey::grammar;

fn main() {
    let _grammar = grammar! {
        Sum -> Sum ["+-"] Product;
        Sum -> Product;
        Product -> Product ["*/"] Factor;
        Product -> Factor;
        Factor -> "(" Sum ")";
        Factor -> Number;
        Number -> ["0123456789"] Number;
        Number -> ["0123456789"];
    };
}
