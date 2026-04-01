use crate::ast::Script;
use crate::stmt::StmtParser;
use pine_lexer::{Span, Token};

/// Parse a complete Pine Script
pub fn parse(tokens: Vec<(Token, Span)>) -> Result<Script, Vec<String>> {
    let mut parser = StmtParser::new(tokens);
    parser.parse_script().map_err(|e| vec![e.to_string()])
}
