
// Simple example to debug AST parsing
use pine_lexer::Lexer;
use pine_parser::ast;
use pine_parser::parser::parse;

fn main() {
    let script = r#"
//@version=6
indicator("Debug NA Call", shorttitle="DebugNACall", overlay=true)

cond = na(close)
plot(cond ? 30.0 : 40.0, title="var_na")
"#;

    match Lexer::lex(script) {
        Ok(tokens) => {
            println!("Tokens:");
            for (i, token) in tokens.iter().enumerate() {
                println!("  {}: {:?}", i, token);
            }
            match parse(tokens) {
                Ok(ast) => {
                    println!("\nAST:");
                    println!("{:#?}", ast);
                }
                Err(e) => eprintln!("Parse error: {:?}", e),
            }
        }
        Err(e) => eprintln!("Lex error: {:?}", e),
    }
}
