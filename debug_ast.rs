// Simple debug script to print AST
use pine_lexer::Lexer;
use pine_parser::parser::parse;

fn main() {
    let script = r#"
//@version=6
indicator("Debug NA Call", shorttitle="DebugNACall", overlay=true)
cond = na(close)
plot(cond ? 30.0 : 40.0, title="var_na")
"#;

    let tokens = Lexer::lex(script).unwrap();
    println!("TOKENS: {:?}", tokens);
    let ast = parse(tokens).unwrap();
    println!("AST: {:#?}", ast);
}
