//! Expression parser using Pratt parsing
//!
//! Implements Pratt parsing (top-down operator precedence) for Pine Script v6
//! with 17 levels of operator precedence.

use crate::ast::*;
use crate::error::ParseError;
use pine_lexer::{Span, Token};

/// Token wrapper with span for parsing
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span,
}

/// Expression parser using Pratt parsing
pub struct ExprParser {
    tokens: Vec<TokenInfo>,
    pos: usize,
}

/// Operator precedence levels (higher = binds tighter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
    /// Minimum precedence
    Lowest = 0,
    /// Ternary conditional (?:)
    Ternary = 10,
    /// NA coalesce (??)
    Coalesce = 20,
    /// Logical or
    Or = 30,
    /// Logical and
    And = 40,
    /// Equality (==, !=)
    Equality = 50,
    /// Comparison (<, >, <=, >=)
    Comparison = 60,
    /// Addition/subtraction (+, -)
    Sum = 70,
    /// Multiplication/division (*, /, %)
    Product = 80,
    /// Power (^)
    Power = 90,
    /// Unary operators (not, -)
    Prefix = 100,
    /// Function calls, indexing, field access
    Call = 110,
    /// Highest precedence
    Highest = 120,
}

impl ExprParser {
    /// Create a new expression parser from tokens
    pub fn new(tokens: Vec<(Token, Span)>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|(t, _)| !matches!(t, Token::Newline | Token::Comment | Token::BlockComment))
            .map(|(token, span)| TokenInfo { token, span })
            .collect();

        Self { tokens, pos: 0 }
    }

    /// Get current position
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Parse an expression (entry point)
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expression(Precedence::Lowest)
    }

    /// Parse expression with minimum precedence
    fn parse_expression(&mut self, min_prec: Precedence) -> Result<Expr, ParseError> {
        // Parse the left-hand side (prefix)
        let mut lhs = self.parse_prefix()?;

        // Parse infix operators with higher precedence
        while let Some((op_prec, op_span)) = self.get_infix_precedence() {
            if op_prec < min_prec {
                break;
            }

            // Handle special cases for infix operators
            match self.peek_token() {
                Some(Token::Question) => {
                    // Ternary: cond ? then : else
                    lhs = self.parse_ternary(lhs)?;
                }
                Some(Token::Question2) => {
                    // NA coalesce: lhs ?? rhs
                    self.advance(); // consume ??
                    let rhs = self.parse_expression(Precedence::Coalesce)?;
                    let span = lhs.span().merge(rhs.span());
                    lhs = Expr::NaCoalesce {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        span,
                    };
                }
                Some(Token::LParen) if op_prec >= Precedence::Call => {
                    // Function call
                    lhs = self.parse_call(lhs)?;
                }
                Some(Token::LBracket) if op_prec >= Precedence::Call => {
                    // Index access
                    lhs = self.parse_index(lhs)?;
                }
                Some(Token::Dot) if op_prec >= Precedence::Call => {
                    // Field access or method call
                    lhs = self.parse_field_or_method(lhs)?;
                }
                _ => {
                    // Binary operator
                    lhs = self.parse_binary(lhs, op_prec, op_span)?;
                }
            }
        }

        Ok(lhs)
    }

    /// Parse prefix expression
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let token_info = self
            .peek_info()
            .ok_or(ParseError::unexpected_eof(Span::default()))?;
        let span = token_info.span;

        match &token_info.token {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Literal(Lit::Int(*n), span))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expr::Literal(Lit::Float(*f), span))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Lit::String(s), span))
            }
            Token::Color(c) => {
                self.advance();
                Ok(Expr::Literal(Lit::Color(*c), span))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Lit::Bool(true), span))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(Lit::Bool(false), span))
            }
            Token::Na => {
                // Special case: if "na" is followed by "(", it's a function call, not a literal
                if self
                    .tokens
                    .get(self.pos + 1)
                    .map(|t| t.token == Token::LParen)
                    .unwrap_or(false)
                {
                    self.advance();
                    Ok(Expr::Ident(Ident::new("na".to_string(), span)))
                } else {
                    self.advance();
                    Ok(Expr::Literal(Lit::Na, span))
                }
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(Ident::new(name, span)))
            }
            Token::Minus => {
                self.advance();
                let operand = self.parse_expression(Precedence::Prefix)?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span,
                })
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_expression(Precedence::Prefix)?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                })
            }
            Token::LParen => {
                self.advance(); // consume (

                // Check for lambda: (params) => body
                if self.is_lambda_params() {
                    self.parse_lambda(span)
                } else {
                    // Grouped expression
                    let expr = self.parse_expression(Precedence::Lowest)?;
                    self.expect_token(Token::RParen, "expected rparen")?;
                    Ok(expr)
                }
            }
            Token::LBracket => self.parse_array_literal(),
            _ => Err(ParseError::unexpected_token(
                format!("{:?}", token_info.token),
                "expression",
                span,
            )),
        }
    }

    /// Parse binary operation
    fn parse_binary(
        &mut self,
        lhs: Expr,
        op_prec: Precedence,
        op_span: Span,
    ) -> Result<Expr, ParseError> {
        let token = self
            .peek_token()
            .ok_or(ParseError::unexpected_eof(Span::default()))?;

        let op = match token {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Sub,
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            Token::Percent => BinOp::Mod,
            Token::Hat => BinOp::Pow,
            Token::Eq => BinOp::Eq,
            Token::Neq => BinOp::Neq,
            Token::Lt => BinOp::Lt,
            Token::Le => BinOp::Le,
            Token::Gt => BinOp::Gt,
            Token::Ge => BinOp::Ge,
            Token::And => BinOp::And,
            Token::Or => BinOp::Or,
            _ => {
                return Err(ParseError::unexpected_token(
                    format!("{:?}", token),
                    "binary operator",
                    op_span,
                ))
            }
        };

        self.advance(); // consume operator

        // For right-associative operators (^), use same precedence
        // For left-associative, use one higher
        let next_prec = if op == BinOp::Pow {
            op_prec
        } else {
            Precedence::from_u8(op_prec as u8 + 1)
        };

        let rhs = self.parse_expression(next_prec)?;
        let span = lhs.span().merge(rhs.span());

        Ok(Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    }

    /// Parse ternary conditional: cond ? then : else
    fn parse_ternary(&mut self, cond: Expr) -> Result<Expr, ParseError> {
        self.expect_token(Token::Question, "expected question")?;
        let then_branch = self.parse_expression(Precedence::Ternary)?;
        self.expect_token(Token::Colon, "expected colon")?;
        let else_branch = self.parse_expression(Precedence::Ternary)?;
        let span = cond.span().merge(else_branch.span());

        Ok(Expr::Ternary {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            span,
        })
    }

    /// Parse function call: func(args)
    fn parse_call(&mut self, func: Expr) -> Result<Expr, ParseError> {
        self.expect_token(Token::LParen, "expected lparen")?;

        let mut args = Vec::new();

        // Check for empty args
        if self.peek_token() == Some(Token::RParen) {
            self.advance();
            // Span must cover the whole call so call-site interning distinguishes f(1) vs f(2).
            let span = func.span().merge(self.prev_span());
            return Ok(Expr::FnCall {
                func: Box::new(func),
                args,
                span,
            });
        }

        loop {
            // Check for named argument: name = value
            if self.is_named_arg() {
                let name = self.expect_ident("expected identifier")?;
                self.expect_token(Token::Assign, "expected equals")?;
                let value = self.parse_expression(Precedence::Lowest)?;
                args.push(Arg {
                    name: Some(name),
                    value,
                });
            } else {
                let value = self.parse_expression(Precedence::Lowest)?;
                args.push(Arg { name: None, value });
            }

            match self.peek_token() {
                Some(Token::Comma) => {
                    self.advance();
                    continue;
                }
                Some(Token::RParen) => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        format!("{:?}", self.peek_token()),
                        ", or rparen",
                        self.peek_span().unwrap_or_default(),
                    ))
                }
            }
        }

        let span = func.span().merge(self.prev_span());
        Ok(Expr::FnCall {
            func: Box::new(func),
            args,
            span,
        })
    }

    /// Parse index access: base[offset]
    fn parse_index(&mut self, base: Expr) -> Result<Expr, ParseError> {
        self.expect_token(Token::LBracket, "expected lbracket")?;
        let offset = self.parse_expression(Precedence::Lowest)?;
        self.expect_token(Token::RBracket, "expected rbracket")?;
        let span = base.span().merge(self.prev_span());

        Ok(Expr::Index {
            base: Box::new(base),
            offset: Box::new(offset),
            span,
        })
    }

    /// Parse field access or method call
    fn parse_field_or_method(&mut self, base: Expr) -> Result<Expr, ParseError> {
        self.expect_token(Token::Dot, "expected dot")?;
        let field = self.expect_ident("expected field name")?;

        // Check for method call
        if self.peek_token() == Some(Token::LParen) {
            // Method call: obj.method(args)
            self.advance(); // consume (
            let mut args = Vec::new();

            if self.peek_token() != Some(Token::RParen) {
                loop {
                    // Check for named argument: name = value
                    if self.is_named_arg() {
                        let name = self.expect_ident("expected identifier")?;
                        self.expect_token(Token::Assign, "expected equals")?;
                        let value = self.parse_expression(Precedence::Lowest)?;
                        args.push(Arg {
                            name: Some(name),
                            value,
                        });
                    } else {
                        let value = self.parse_expression(Precedence::Lowest)?;
                        args.push(Arg { name: None, value });
                    }

                    match self.peek_token() {
                        Some(Token::Comma) => {
                            self.advance();
                            continue;
                        }
                        Some(Token::RParen) => break,
                        _ => {
                            return Err(ParseError::unexpected_token(
                                format!("{:?}", self.peek_token()),
                                ", or rparen",
                                self.peek_span().unwrap_or_default(),
                            ))
                        }
                    }
                }
            }

            self.expect_token(Token::RParen, "expected rparen")?;
            let span = base.span();

            Ok(Expr::MethodCall {
                base: Box::new(base),
                method: field,
                args,
                span,
            })
        } else {
            // Field access: obj.field
            let span = base.span().merge(field.span);
            Ok(Expr::FieldAccess {
                base: Box::new(base),
                field,
                span,
            })
        }
    }

    /// Parse array literal: [expr, ...]
    fn parse_array_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::LBracket, "expected lbracket")?;

        let mut elements = Vec::new();

        if self.peek_token() == Some(Token::RBracket) {
            self.advance();
            let span = start_span.merge(self.prev_span());
            return Ok(Expr::ArrayLit(elements, span));
        }

        loop {
            // Check for map literal syntax: [key: value, ...]
            if self.is_map_entry() {
                return self.parse_map_literal(start_span);
            }

            let expr = self.parse_expression(Precedence::Lowest)?;
            elements.push(expr);

            match self.peek_token() {
                Some(Token::Comma) => {
                    self.advance();
                    if self.peek_token() == Some(Token::RBracket) {
                        self.advance();
                        break;
                    }
                    continue;
                }
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        format!("{:?}", self.peek_token()),
                        "comma or rbracket",
                        self.peek_span().unwrap_or_default(),
                    ))
                }
            }
        }

        let span = start_span.merge(self.prev_span());
        Ok(Expr::ArrayLit(elements, span))
    }

    /// Parse map literal: [key1: value1, key2: value2]
    fn parse_map_literal(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        let mut entries = Vec::new();

        loop {
            let key = self.parse_expression(Precedence::Lowest)?;
            self.expect_token(Token::Colon, "expected colon")?;
            let value = self.parse_expression(Precedence::Lowest)?;
            entries.push((key, value));

            match self.peek_token() {
                Some(Token::Comma) => {
                    self.advance();
                    if self.peek_token() == Some(Token::RBracket) {
                        self.advance();
                        break;
                    }
                    continue;
                }
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        format!("{:?}", self.peek_token()),
                        "comma or rbracket",
                        self.peek_span().unwrap_or_default(),
                    ))
                }
            }
        }

        let span = start_span.merge(self.prev_span());
        Ok(Expr::MapLit(entries, span))
    }

    /// Parse lambda expression: (params) => body
    fn parse_lambda(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        // Parse parameters
        let params = self.parse_lambda_params()?;
        self.expect_token(Token::Arrow, "expected arrow")?;

        // Parse body (lambda body is a single expression for now)
        let body = Box::new(self.parse_expression(Precedence::Lowest)?);

        let span = start_span.merge(self.prev_span());
        Ok(Expr::Lambda { params, body, span })
    }

    /// Parse lambda parameters
    fn parse_lambda_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        while self.peek_token() != Some(Token::RParen) {
            let name = self.expect_ident("expected param")?;
            let type_ann = if self.peek_token() == Some(Token::Colon) {
                self.advance();
                Some(self.parse_type_ann()?)
            } else {
                None
            };

            params.push(Param {
                name,
                type_ann,
                default: None,
            });

            if self.peek_token() == Some(Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_token(Token::RParen, "expected rparen")?;
        Ok(params)
    }

    /// Parse type annotation
    fn parse_type_ann(&mut self) -> Result<TypeAnn, ParseError> {
        // Simplified type annotation parsing
        let ident = self.expect_ident("expected typename")?;
        Ok(TypeAnn::Simple(ident.name))
    }

    /// Check if current position looks like lambda parameters
    fn is_lambda_params(&mut self) -> bool {
        // Caller already consumed `(`; scan to matching `)` then require `=>`.
        let saved_pos = self.pos;
        let mut depth = 1usize;

        while self.pos < self.tokens.len() && depth > 0 {
            match self.peek_token() {
                Some(Token::LParen) => {
                    depth += 1;
                    self.advance();
                }
                Some(Token::RParen) => {
                    depth -= 1;
                    self.advance();
                }
                _ => self.advance(),
            }
        }

        let found_arrow = depth == 0 && self.peek_token() == Some(Token::Arrow);
        self.pos = saved_pos;
        found_arrow
    }

    /// Check if current position is a named argument
    fn is_named_arg(&self) -> bool {
        if let Some(Token::Ident(_)) = self.peek_token() {
            let saved_pos = self.pos;
            // Look ahead for '='
            if self.pos + 1 < self.tokens.len() {
                if let Token::Assign = self.tokens[saved_pos + 1].token {
                    return true;
                }
            }
        }
        false
    }

    /// Check if current position is a map entry (key: value)
    fn is_map_entry(&self) -> bool {
        // Look ahead for ':' after an expression
        // Simplified check: if we see an identifier followed by ':'
        if let Some(Token::Ident(_)) = self.peek_token() {
            let saved_pos = self.pos;
            if saved_pos + 1 < self.tokens.len() {
                if let Token::Colon = self.tokens[saved_pos + 1].token {
                    return true;
                }
            }
        }
        false
    }

    /// Get precedence of infix operator at current position
    fn get_infix_precedence(&self) -> Option<(Precedence, Span)> {
        let token_info = self.peek_info()?;
        let prec = match &token_info.token {
            Token::Question => Precedence::Ternary,
            Token::Question2 => Precedence::Coalesce,
            Token::Or => Precedence::Or,
            Token::And => Precedence::And,
            Token::Eq | Token::Neq => Precedence::Equality,
            Token::Lt | Token::Le | Token::Gt | Token::Ge => Precedence::Comparison,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Star | Token::Slash | Token::Percent => Precedence::Product,
            Token::Hat => Precedence::Power,
            Token::LParen | Token::LBracket | Token::Dot => Precedence::Call,
            _ => return None,
        };
        Some((prec, token_info.span))
    }

    /// Peek at current token
    fn peek_token(&self) -> Option<Token> {
        self.tokens.get(self.pos).map(|t| t.token.clone())
    }

    /// Peek at current token info
    fn peek_info(&self) -> Option<TokenInfo> {
        self.tokens.get(self.pos).cloned()
    }

    /// Peek at current span
    fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.pos).map(|t| t.span)
    }

    /// Get span of previous token
    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::default()
        }
    }

    /// Advance to next token
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    /// Expect a specific token
    fn expect_token(
        &mut self,
        expected: Token,
        msg: impl Into<String>,
    ) -> Result<Span, ParseError> {
        let msg = msg.into();
        match self.peek_info() {
            Some(info) if info.token == expected => {
                let span = info.span;
                self.advance();
                Ok(span)
            }
            Some(info) => Err(ParseError::unexpected_token(
                format!("{:?}", info.token),
                msg,
                info.span,
            )),
            None => Err(ParseError::unexpected_eof(Span::default())),
        }
    }

    /// Expect an identifier
    fn expect_ident(&mut self, msg: impl Into<String>) -> Result<Ident, ParseError> {
        let msg = msg.into();
        match self.peek_info() {
            Some(info) => {
                if let Token::Ident(name) = &info.token {
                    let ident = Ident::new(name.clone(), info.span);
                    self.advance();
                    Ok(ident)
                } else {
                    Err(ParseError::unexpected_token(
                        format!("{:?}", info.token),
                        msg,
                        info.span,
                    ))
                }
            }
            None => Err(ParseError::unexpected_eof(Span::default())),
        }
    }
}

impl Precedence {
    fn from_u8(n: u8) -> Self {
        match n {
            0 => Precedence::Lowest,
            1..=10 => Precedence::Ternary,
            11..=20 => Precedence::Coalesce,
            21..=30 => Precedence::Or,
            31..=40 => Precedence::And,
            41..=50 => Precedence::Equality,
            51..=60 => Precedence::Comparison,
            61..=70 => Precedence::Sum,
            71..=80 => Precedence::Product,
            81..=90 => Precedence::Power,
            91..=100 => Precedence::Prefix,
            101..=110 => Precedence::Call,
            _ => Precedence::Highest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Lexer;

    fn parse_expr(input: &str) -> Result<Expr, ParseError> {
        let tokens = Lexer::lex(input).unwrap();
        let mut parser = ExprParser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_literal() {
        let expr = parse_expr("42").unwrap();
        assert!(matches!(expr, Expr::Literal(Lit::Int(42), _)));
    }

    #[test]
    fn test_binary_op() {
        let expr = parse_expr("1 + 2").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let expr = parse_expr("1 + 2 * 3").unwrap();
        match &expr {
            Expr::BinOp {
                op: BinOp::Add,
                lhs,
                rhs,
                ..
            } => {
                assert!(matches!(lhs.as_ref(), Expr::Literal(Lit::Int(1), _)));
                assert!(matches!(rhs.as_ref(), Expr::BinOp { op: BinOp::Mul, .. }));
            }
            _ => panic!("expected Add expression"),
        }
    }

    #[test]
    fn test_ternary() {
        let expr = parse_expr("a ? b : c").unwrap();
        assert!(matches!(expr, Expr::Ternary { .. }));
    }

    #[test]
    fn test_na_coalesce() {
        let expr = parse_expr("a ?? b").unwrap();
        assert!(matches!(expr, Expr::NaCoalesce { .. }));
    }

    #[test]
    fn test_function_call() {
        let expr = parse_expr("func(a, b)").unwrap();
        assert!(matches!(expr, Expr::FnCall { .. }));
    }

    #[test]
    fn test_fncall_span_covers_closing_paren() {
        let e = parse_expr("f(100)").unwrap();
        let Expr::FnCall { span, .. } = e else {
            panic!("expected FnCall");
        };
        assert!(span.end > span.start);
        // Line-local `lex()` offsets: span covers full `f(100)` in single-line snippet.
        assert_eq!(span.end - span.start, "f(100)".len());
    }

    #[test]
    fn test_field_access() {
        let expr = parse_expr("obj.field").unwrap();
        assert!(matches!(expr, Expr::FieldAccess { .. }));
    }

    #[test]
    fn test_index_access() {
        let expr = parse_expr("close[1]").unwrap();
        assert!(matches!(expr, Expr::Index { .. }));
    }

    #[test]
    fn test_unary() {
        let expr = parse_expr("-x").unwrap();
        assert!(matches!(
            expr,
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                ..
            }
        ));

        let expr = parse_expr("not x").unwrap();
        assert!(matches!(
            expr,
            Expr::UnaryOp {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn test_array_literal() {
        let expr = parse_expr("[1, 2, 3]").unwrap();
        assert!(matches!(expr, Expr::ArrayLit(elems, _) if elems.len() == 3));
    }

    #[test]
    fn test_comparison_chain() {
        // a < b and c > d
        let expr = parse_expr("a < b and c > d").unwrap();
        println!("Parsed: {:?}", expr);
        match &expr {
            Expr::BinOp {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => {
                // lhs should be "a < b"
                assert!(matches!(lhs.as_ref(), Expr::BinOp { op: BinOp::Lt, .. }));
                // rhs should be "c > d"
                assert!(matches!(rhs.as_ref(), Expr::BinOp { op: BinOp::Gt, .. }));
            }
            _ => panic!("expected And expression, got {:?}", expr),
        }
    }
}
