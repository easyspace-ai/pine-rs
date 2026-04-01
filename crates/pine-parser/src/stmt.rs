//! Statement parser for Pine Script v6
//!
//! Parses statements using recursive descent with indentation-aware parsing.

use crate::ast::*;
use crate::error::ParseError;
use crate::expr::ExprParser;
use pine_lexer::{Span, Token};

/// Token wrapper with span
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span,
}

/// Statement parser
pub struct StmtParser {
    tokens: Vec<TokenInfo>,
    pos: usize,
}

impl StmtParser {
    /// Create a new statement parser
    pub fn new(tokens: Vec<(Token, Span)>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|(t, _)| !matches!(t, Token::Comment | Token::BlockComment))
            .map(|(token, span)| TokenInfo { token, span })
            .collect();

        Self { tokens, pos: 0 }
    }

    /// Parse a complete script
    pub fn parse_script(&mut self) -> Result<Script, ParseError> {
        let mut stmts = Vec::new();
        let start_span = self.peek_span().unwrap_or_default();

        while self.peek_token().is_some() {
            // Skip newlines at statement level
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
                continue;
            }

            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        let end_span = self.prev_span();
        Ok(Script {
            stmts,
            span: start_span.merge(end_span),
        })
    }

    /// Parse a single statement
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let token_info = self
            .peek_info()
            .ok_or(ParseError::unexpected_eof(Span::default()))?;

        match &token_info.token {
            Token::Var | Token::Varip => self.parse_var_decl(),
            Token::If => self.parse_if_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::Switch => self.parse_switch_stmt(),
            Token::Fn => self.parse_fn_def(),
            Token::Type => self.parse_type_def(),
            Token::Method => self.parse_method_def(),
            Token::Import => self.parse_import(),
            Token::Export => self.parse_export(),
            Token::Library => self.parse_library(),
            Token::Break => self.parse_break(),
            Token::Continue => self.parse_continue(),
            Token::Return => self.parse_return(),
            _ => {
                // Could be assignment or expression statement
                self.parse_assign_or_expr()
            }
        }
    }

    /// Parse variable declaration: [var/varip] name[: type] [= init]
    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();

        // Parse var/varip
        let kind = if self.peek_token() == Some(Token::Var) {
            self.advance();
            VarKind::Var
        } else if self.peek_token() == Some(Token::Varip) {
            self.advance();
            VarKind::Varip
        } else {
            VarKind::Plain
        };

        // Parse identifier
        let name = self.expect_ident("expected variable name")?;

        // Parse optional type annotation
        let type_ann = if self.peek_token() == Some(Token::Colon) {
            self.advance();
            Some(self.parse_type_ann()?)
        } else {
            None
        };

        // Parse optional initializer
        let init = if self.peek_token() == Some(Token::Assign) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = start_span.merge(self.prev_span());

        // Skip trailing newline
        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::VarDecl {
            name,
            kind,
            type_ann,
            init,
            span,
        })
    }

    /// Parse if statement: if cond then_block [elif ...] [else else_block]
    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::If, "expected 'if'")?;

        let cond = self.parse_expr()?;

        // Parse then block
        let then_block = self.parse_block()?;

        // Parse elif blocks
        let mut elifs = Vec::new();
        while self.peek_token() == Some(Token::Elif) {
            self.advance();
            let elif_cond = self.parse_expr()?;
            let elif_block = self.parse_block()?;
            elifs.push((elif_cond, elif_block));
        }

        // Parse optional else block
        let else_block = if self.peek_token() == Some(Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.prev_span());

        Ok(Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            span,
        })
    }

    /// Parse for loop: for var = from to to [by step] body
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::For, "expected 'for'")?;

        let var = self.expect_ident("expected loop variable")?;
        self.expect_token(Token::Assign, "expected '='")?;

        let from = self.parse_expr()?;
        self.expect_token(Token::To, "expected 'to'")?;

        let to = self.parse_expr()?;

        // Optional by clause
        let by = if self.peek_token() == Some(Token::By) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::For {
            var,
            from,
            to,
            by,
            body,
            span,
        })
    }

    /// Parse while loop: while cond body
    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::While, "expected 'while'")?;

        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::While { cond, body, span })
    }

    /// Parse switch statement
    fn parse_switch_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Switch, "expected 'switch'")?;

        let expr = self.parse_expr()?;

        // Expect indent before cases
        self.expect_token(Token::Indent, "expected indentation")?;

        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match self.peek_token() {
                Some(Token::Case) => {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.expect_token(Token::Colon, "expected ':'")?;
                    let body = self.parse_case_body()?;
                    cases.push(SwitchCase { value, body });
                }
                Some(Token::Default) => {
                    self.advance();
                    self.expect_token(Token::Colon, "expected ':'")?;
                    default = Some(self.parse_case_body()?);
                }
                _ => break,
            }
        }

        self.expect_token(Token::Dedent, "expected dedent")?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::Switch {
            expr,
            cases,
            default,
            span,
        })
    }

    /// Parse function definition: fn name(params) [-> type] body
    fn parse_fn_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Fn, "expected 'fn'")?;

        let name = self.expect_ident("expected function name")?;

        // Parse parameters
        self.expect_token(Token::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect_token(Token::RParen, "expected ')'")?;

        // Optional return type
        let ret_type = if self.peek_token() == Some(Token::Arrow) {
            self.advance();
            Some(self.parse_type_ann()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::FnDef {
            name,
            params,
            ret_type,
            body,
            span,
        })
    }

    /// Parse type definition: type Name { fields... }
    fn parse_type_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Type, "expected 'type'")?;

        let name = self.expect_ident("expected type name")?;

        self.expect_token(Token::Indent, "expected indentation")?;

        let mut fields = Vec::new();
        while self.peek_token() != Some(Token::Dedent) && self.peek_token().is_some() {
            fields.push(self.parse_field()?);
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
        }

        self.expect_token(Token::Dedent, "expected dedent")?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::TypeDef { name, fields, span })
    }

    /// Parse method definition: method Type.name(params) body
    fn parse_method_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Method, "expected 'method'")?;

        let type_name = self.expect_ident("expected type name")?;
        self.expect_token(Token::Dot, "expected '.'")?;
        let name = self.expect_ident("expected method name")?;

        self.expect_token(Token::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect_token(Token::RParen, "expected ')'")?;

        let ret_type = if self.peek_token() == Some(Token::Arrow) {
            self.advance();
            Some(self.parse_type_ann()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::MethodDef {
            type_name,
            name,
            params,
            ret_type,
            body,
            span,
        })
    }

    /// Parse import statement: import "path" [as name]
    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Import, "expected 'import'")?;

        let path = match self.peek_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(ParseError::unexpected_eof(Span::default())),
        };

        let alias = if self.peek_token() == Some(Token::Ident("as".to_string())) {
            self.advance();
            Some(self.expect_ident("expected alias name")?)
        } else {
            None
        };

        let span = start_span.merge(self.prev_span());

        // Skip trailing newline
        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Import { path, alias, span })
    }

    /// Parse export statement: export name
    fn parse_export(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Export, "expected 'export'")?;

        let name = self.expect_ident("expected name to export")?;
        let span = start_span.merge(self.prev_span());

        // Skip trailing newline
        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Export { name, span })
    }

    /// Parse library declaration: library(name [, ...])
    fn parse_library(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Library, "expected 'library'")?;

        self.expect_token(Token::LParen, "expected '('")?;

        // Parse library name
        let name = match self.peek_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err(ParseError::unexpected_eof(Span::default())),
        };

        // Parse optional properties
        let mut props = Vec::new();
        while self.peek_token() == Some(Token::Comma) {
            self.advance();
            let prop_name = self.expect_ident("expected property name")?;
            self.expect_token(Token::Assign, "expected '='")?;
            let value = self.parse_expr()?;
            props.push((prop_name, value));
        }

        self.expect_token(Token::RParen, "expected ')'")?;
        let span = start_span.merge(self.prev_span());

        // Skip trailing newline
        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Library { name, props, span })
    }

    /// Parse break statement
    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        let span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Break, "expected 'break'")?;

        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Break { span })
    }

    /// Parse continue statement
    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        let span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Continue, "expected 'continue'")?;

        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Continue { span })
    }

    /// Parse return statement
    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Return, "expected 'return'")?;

        let value = if self.peek_token() != Some(Token::Newline)
            && self.peek_token() != Some(Token::Dedent)
            && self.peek_token().is_some()
        {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = start_span.merge(self.prev_span());

        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Return { value, span })
    }

    /// Parse assignment or expression statement
    fn parse_assign_or_expr(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;

        // Check if this is an assignment or declaration
        if self.peek_token() == Some(Token::Assign) {
            // Plain variable declaration: x = value
            // Or tuple assignment: [a, b, c] = value
            self.advance();
            let value = self.parse_expr()?;
            let span = expr.span().merge(value.span());

            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }

            match expr {
                Expr::Ident(ident) => Ok(Stmt::VarDecl {
                    name: ident,
                    kind: VarKind::Plain,
                    type_ann: None,
                    init: Some(value),
                    span,
                }),
                other => {
                    let target = expr_to_target(other)?;
                    Ok(Stmt::Assign {
                        target,
                        op: AssignOp::Assign,
                        value,
                        span,
                    })
                }
            }
        } else if let Some(op) = self.peek_compound_assign_op() {
            // Compound assignment: x := value, x += value, etc.
            self.advance();
            let value = self.parse_expr()?;

            // Convert expression to assignment target
            let target = expr_to_target(expr)?;
            let span = target.span().merge(value.span());

            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }

            Ok(Stmt::Assign {
                target,
                op,
                value,
                span,
            })
        } else {
            // Expression statement
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            Ok(Stmt::Expr(expr))
        }
    }

    /// Parse a block of statements
    /// Supports both indented blocks and single-statement blocks
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        // Get start span before skipping newlines
        let start_span = self.peek_span().unwrap_or_default();

        // Skip newlines before checking for indentation
        while self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        // Check for indented block
        if self.peek_token() == Some(Token::Indent) {
            self.advance(); // consume indent

            let mut stmts = Vec::new();
            while self.peek_token() != Some(Token::Dedent) && self.peek_token().is_some() {
                if self.peek_token() == Some(Token::Newline) {
                    self.advance();
                    continue;
                }

                stmts.push(self.parse_stmt()?);
            }

            self.expect_token(Token::Dedent, "expected dedent")?;

            let end_span = self.prev_span();
            return Ok(Block {
                stmts,
                span: start_span.merge(end_span),
            });
        }

        // Single-statement block (e.g., `if x > 0 y = 1`)
        let stmt = self.parse_stmt()?;
        let span = stmt.span();

        Ok(Block {
            stmts: vec![stmt],
            span,
        })
    }

    /// Parse case body (simplified - just a single statement or block)
    fn parse_case_body(&mut self) -> Result<Block, ParseError> {
        // If we see indent, parse a block
        if self.peek_token() == Some(Token::Indent) {
            return self.parse_block();
        }

        // Otherwise, parse a single statement
        let stmt = self.parse_stmt()?;
        let span = stmt.span();

        Ok(Block {
            stmts: vec![stmt],
            span,
        })
    }

    /// Parse function parameters
    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.peek_token() == Some(Token::RParen) {
            return Ok(params);
        }

        loop {
            let name = self.expect_ident("expected parameter name")?;

            let type_ann = if self.peek_token() == Some(Token::Colon) {
                self.advance();
                Some(self.parse_type_ann()?)
            } else {
                None
            };

            let default = if self.peek_token() == Some(Token::Assign) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };

            params.push(Param {
                name,
                type_ann,
                default,
            });

            if self.peek_token() == Some(Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    /// Parse a field definition
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.expect_ident("expected field name")?;

        let type_ann = if self.peek_token() == Some(Token::Colon) {
            self.advance();
            Some(self.parse_type_ann()?)
        } else {
            None
        };

        let default = if self.peek_token() == Some(Token::Assign) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Field {
            name,
            type_ann,
            default,
        })
    }

    /// Parse type annotation
    fn parse_type_ann(&mut self) -> Result<TypeAnn, ParseError> {
        let ident = self.expect_ident("expected type name")?;

        // Check for generic types
        match ident.name.as_str() {
            "series" => {
                if self.peek_token() == Some(Token::Lt) {
                    self.advance();
                    let inner = self.parse_type_ann()?;
                    self.expect_token(Token::Gt, "expected '>'")?;
                    Ok(TypeAnn::Series(Box::new(inner)))
                } else {
                    Ok(TypeAnn::Simple("series".to_string()))
                }
            }
            "array" => {
                if self.peek_token() == Some(Token::Lt) {
                    self.advance();
                    let inner = self.parse_type_ann()?;
                    self.expect_token(Token::Gt, "expected '>'")?;
                    Ok(TypeAnn::Array(Box::new(inner)))
                } else {
                    Ok(TypeAnn::Simple("array".to_string()))
                }
            }
            "matrix" => {
                if self.peek_token() == Some(Token::Lt) {
                    self.advance();
                    let inner = self.parse_type_ann()?;
                    self.expect_token(Token::Gt, "expected '>'")?;
                    Ok(TypeAnn::Matrix(Box::new(inner)))
                } else {
                    Ok(TypeAnn::Simple("matrix".to_string()))
                }
            }
            "map" => {
                if self.peek_token() == Some(Token::Lt) {
                    self.advance();
                    let key = self.parse_type_ann()?;
                    self.expect_token(Token::Comma, "expected ','")?;
                    let value = self.parse_type_ann()?;
                    self.expect_token(Token::Gt, "expected '>'")?;
                    Ok(TypeAnn::Map(Box::new(key), Box::new(value)))
                } else {
                    Ok(TypeAnn::Simple("map".to_string()))
                }
            }
            _ => Ok(TypeAnn::Simple(ident.name)),
        }
    }

    /// Parse an expression using the expression parser
    /// Parse an expression using the expression parser
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Find how many tokens to consume for this expression
        // We need to look ahead to find where the expression ends
        let end_pos = self.find_expr_end()?;

        // Get tokens for expression (filter out newlines in the middle of expressions)
        let expr_tokens: Vec<(Token, Span)> = self.tokens[self.pos..end_pos]
            .iter()
            .filter(|t| !matches!(t.token, Token::Newline))
            .map(|t| (t.token.clone(), t.span))
            .collect();

        let mut expr_parser = ExprParser::new(expr_tokens);
        let expr = expr_parser.parse()?;

        // Advance past consumed tokens
        self.pos = end_pos;

        Ok(expr)
    }

    /// Find the end of the current expression
    /// Returns the position of the first token after the expression
    fn find_expr_end(&self) -> Result<usize, ParseError> {
        let mut pos = self.pos;
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        while pos < self.tokens.len() {
            let token = &self.tokens[pos].token;

            match token {
                // These tokens end an expression (when not inside parens/brackets)
                Token::Newline | Token::Indent | Token::Dedent => {
                    if paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                }
                Token::LParen => paren_depth += 1,
                Token::RParen => {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                }
                Token::LBracket => bracket_depth += 1,
                Token::RBracket => {
                    if bracket_depth == 0 {
                        break;
                    }
                    bracket_depth -= 1;
                }
                // Assignment operators also end expressions
                Token::Assign
                | Token::ColonEq
                | Token::PlusEq
                | Token::MinusEq
                | Token::StarEq
                | Token::SlashEq => {
                    if paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                }
                // For loop keywords end expressions
                Token::To | Token::By => {
                    if paren_depth == 0 && bracket_depth == 0 {
                        break;
                    }
                }
                _ => {}
            }

            pos += 1;
        }

        // Expression must have at least one token
        if pos == self.pos {
            return Err(ParseError::unexpected_eof(Span::default()));
        }

        Ok(pos)
    }

    /// Peek compound assignment operator (:=, +=, -=, etc.)
    fn peek_compound_assign_op(&self) -> Option<AssignOp> {
        match self.peek_token() {
            Some(Token::ColonEq) => Some(AssignOp::ColonEq),
            Some(Token::PlusEq) => Some(AssignOp::PlusEq),
            Some(Token::MinusEq) => Some(AssignOp::MinusEq),
            Some(Token::StarEq) => Some(AssignOp::StarEq),
            Some(Token::SlashEq) => Some(AssignOp::SlashEq),
            _ => None,
        }
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

/// Convert expression to assignment target
fn expr_to_target(expr: Expr) -> Result<AssignTarget, ParseError> {
    match expr {
        Expr::Ident(ident) => Ok(AssignTarget::Var(ident)),
        Expr::ArrayLit(elements, _) => {
            let mut idents = Vec::with_capacity(elements.len());
            for element in elements {
                match element {
                    Expr::Ident(ident) => idents.push(ident),
                    _ => return Err(ParseError::unexpected_eof(Span::default())),
                }
            }
            Ok(AssignTarget::Tuple(idents))
        }
        Expr::Index {
            base,
            offset,
            span: _,
        } => Ok(AssignTarget::Index { base, offset }),
        Expr::FieldAccess {
            base,
            field,
            span: _,
        } => Ok(AssignTarget::Field { base, field }),
        _ => Err(ParseError::unexpected_eof(Span::default())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pine_lexer::Lexer;

    fn parse(input: &str) -> Result<Script, ParseError> {
        let tokens = Lexer::lex(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        parser.parse_script()
    }

    #[test]
    fn test_empty_script() {
        let script = parse("").unwrap();
        assert!(script.stmts.is_empty());
    }

    #[test]
    fn test_var_decl() {
        let script = parse("x = 42").unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(script.stmts[0], Stmt::VarDecl { .. }));
    }

    #[test]
    fn test_if_stmt() {
        // Use indentation-based block instead of single-line for this test
        let input = "if x\n    y";
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(script.stmts[0], Stmt::If { .. }));
    }

    #[test]
    fn test_for_loop() {
        // For loop with indented body
        let input = "for i = 0 to 10\n    plot(i)";
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(script.stmts[0], Stmt::For { .. }));
    }

    #[test]
    fn test_fn_def() {
        // Single-line function
        let input = "fn add(a, b) a + b";
        let script = parse(input).unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(script.stmts[0], Stmt::FnDef { .. }));
    }
}
