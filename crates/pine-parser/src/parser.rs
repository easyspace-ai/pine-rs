use crate::ast::*;
use pine_lexer::{Span, Token};

/// Parse a complete Pine Script (stub implementation)
pub fn parse(_tokens: Vec<(Token, Span)>) -> Result<Script, Vec<String>> {
    // Stub implementation - just return an empty script for now
    Ok(Script {
        span: Span::default(),
        stmts: vec![],
    })
}
