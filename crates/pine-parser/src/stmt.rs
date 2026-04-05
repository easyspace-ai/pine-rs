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
            Token::Enum => self.parse_enum_def(),
            Token::Method => self.parse_method_def(),
            Token::Import => self.parse_import(),
            Token::Export => self.parse_export(),
            Token::Library => self.parse_library(),
            Token::Break => self.parse_break(),
            Token::Continue => self.parse_continue(),
            Token::Return => self.parse_return(),
            Token::Ident(_) => {
                // TV official UDF syntax is the primary path: name(params) => expr/body
                if let Some(stmt) = self.try_parse_tv_arrow_function_stmt()? {
                    return Ok(stmt);
                }
                self.parse_assign_or_expr()
            }
            _ => self.parse_assign_or_expr(),
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

        let mut else_block = None;
        while self.peek_token() == Some(Token::Else) {
            self.advance();
            while self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            if self.peek_token() == Some(Token::If) {
                self.advance();
                let cond = self.parse_expr()?;
                let block = self.parse_block()?;
                elifs.push((cond, block));
                continue;
            }
            else_block = Some(self.parse_block()?);
            break;
        }

        let span = start_span.merge(self.prev_span());

        Ok(Stmt::If {
            cond,
            then_block,
            elifs,
            else_block,
            span,
        })
    }

    /// Parse for loop: numeric `for i = a to b` or `for x in arr` / `for [i,v] in arr`
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::For, "expected 'for'")?;

        if self.peek_token() == Some(Token::LBracket) {
            self.advance();
            let a = self.expect_ident("expected index name")?;
            self.expect_token(Token::Comma, "expected ','")?;
            let b = self.expect_ident("expected value name")?;
            self.expect_token(Token::RBracket, "expected ']'")?;
            self.expect_token(Token::In, "expected 'in'")?;
            let iterable = self.parse_expr()?;
            let body = self.parse_block()?;
            let span = start_span.merge(self.prev_span());
            return Ok(Stmt::ForIn {
                pattern: ForInPattern::Tuple(a, b),
                iterable,
                body,
                span,
            });
        }

        let var = self.expect_ident("expected loop variable")?;
        if self.peek_token() == Some(Token::In) {
            self.advance();
            let iterable = self.parse_expr()?;
            let body = self.parse_block()?;
            let span = start_span.merge(self.prev_span());
            return Ok(Stmt::ForIn {
                pattern: ForInPattern::Single(var),
                iterable,
                body,
                span,
            });
        }

        self.expect_token(Token::Assign, "expected '=' or 'in'")?;
        let from = self.parse_expr()?;
        self.expect_token(Token::To, "expected 'to'")?;
        let to = self.parse_expr()?;

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

    /// Parse TV v6 switch: optional scrutinee; arms are `expr => body` or `=> body`
    fn parse_switch_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Switch, "expected 'switch'")?;

        while self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        let scrutinee = if self.peek_token() == Some(Token::Indent) {
            None
        } else {
            let s = self.parse_expr()?;
            while self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            Some(s)
        };

        self.expect_token(Token::Indent, "expected indentation")?;

        let mut arms = Vec::new();

        while self.peek_token() != Some(Token::Dedent) && self.peek_token().is_some() {
            while self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            if self.peek_token() == Some(Token::Dedent) {
                break;
            }

            let arm_start = self.peek_span().unwrap_or_default();

            let pattern = if self.peek_token() == Some(Token::Arrow) {
                None
            } else {
                let p = self.parse_expr()?;
                Some(p)
            };

            self.expect_token(Token::Arrow, "expected '=>'")?;

            while self.peek_token() == Some(Token::Newline) {
                self.advance();
            }

            let body = if self.peek_token() == Some(Token::Indent) {
                SwitchArmBody::Block(self.parse_block()?)
            } else {
                let stmt = self.parse_stmt()?;
                let span = stmt.span();
                SwitchArmBody::Block(Block {
                    stmts: vec![stmt],
                    span,
                })
            };

            let span = arm_start.merge(self.prev_span());
            arms.push(SwitchArm {
                pattern,
                body,
                span,
            });
        }

        self.expect_token(Token::Dedent, "expected dedent")?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::Switch {
            scrutinee,
            arms,
            span,
        })
    }

    /// Parse function definition: fn name(params) [=> expr | block]
    fn parse_fn_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Fn, "expected 'fn'")?;

        let name = self.expect_ident("expected function name")?;

        self.expect_token(Token::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect_token(Token::RParen, "expected ')'")?;

        let (ret_type, body) = self.parse_fn_sig_body_after_params(None)?;
        let span = start_span.merge(self.prev_span());

        Ok(Stmt::FnDef {
            name,
            params,
            ret_type,
            body,
            span,
        })
    }

    /// After `)`: `=> expr`, `=>` + indented block, or indented / single-line block.
    fn parse_fn_sig_body_after_params(
        &mut self,
        ret_type: Option<TypeAnn>,
    ) -> Result<(Option<TypeAnn>, FnBody), ParseError> {
        while self.peek_token() == Some(Token::Newline) {
            self.advance();
        }
        if self.peek_token() == Some(Token::Arrow) {
            self.advance();
            // After =>, could be an expression or a block body
            while self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            if self.peek_token() == Some(Token::Indent) {
                return Ok((ret_type, FnBody::Block(self.parse_block()?)));
            }
            let e = self.parse_expr()?;
            return Ok((ret_type, FnBody::Expr(e)));
        }
        Ok((ret_type, FnBody::Block(self.parse_block()?)))
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

    /// Parse enum definition: enum Name newline indent fields...
    fn parse_enum_def(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Enum, "expected 'enum'")?;
        let name = self.expect_ident("expected enum name")?;
        self.expect_token(Token::Indent, "expected indentation")?;

        let mut variants = Vec::new();
        while self.peek_token() != Some(Token::Dedent) && self.peek_token().is_some() {
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
                continue;
            }
            let vname = self.expect_ident("expected enum variant name")?;
            let init = if self.peek_token() == Some(Token::Assign) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            variants.push(EnumVariant { name: vname, init });
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
        }

        self.expect_token(Token::Dedent, "expected dedent")?;
        let span = start_span.merge(self.prev_span());
        Ok(Stmt::EnumDef {
            name,
            variants,
            span,
        })
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

        let (ret_type, body) = self.parse_fn_sig_body_after_params(None)?;
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

    /// Parse import: import "path" | import a/b/c [as name]
    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Import, "expected 'import'")?;

        let path = match self.peek_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                ImportPath::String(s)
            }
            Some(Token::Ident(_)) => {
                let mut segments = Vec::new();
                loop {
                    let seg = match self.peek_token() {
                        Some(Token::Ident(s)) => {
                            let x = s.clone();
                            self.advance();
                            x
                        }
                        Some(Token::Int(n)) => {
                            self.advance();
                            n.to_string()
                        }
                        _ => {
                            return Err(ParseError::unexpected_eof(Span::default()));
                        }
                    };
                    segments.push(seg);
                    if self.peek_token() == Some(Token::Slash) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                ImportPath::Qualified(segments)
            }
            _ => return Err(ParseError::unexpected_eof(Span::default())),
        };

        let alias = if matches!(self.peek_token(), Some(Token::Ident(ref s)) if s == "as") {
            self.advance();
            Some(self.expect_ident("expected alias name")?)
        } else {
            None
        };

        let span = start_span.merge(self.prev_span());

        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::Import { path, alias, span })
    }

    /// Parse export: `export name(params) => expr | block` or `export name = expr`
    fn parse_export(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.peek_span().unwrap_or_default();
        self.expect_token(Token::Export, "expected 'export'")?;

        let name = self.expect_ident("expected export name")?;

        if self.peek_token() == Some(Token::Assign) {
            self.advance();
            let init = self.parse_expr()?;
            let span = start_span.merge(self.prev_span());
            if self.peek_token() == Some(Token::Newline) {
                self.advance();
            }
            return Ok(Stmt::ExportAssign { name, init, span });
        }

        self.expect_token(Token::LParen, "expected '('")?;
        let params = self.parse_params()?;
        self.expect_token(Token::RParen, "expected ')'")?;

        let (ret_type, body) = self.parse_fn_sig_body_after_params(None)?;
        let span = start_span.merge(self.prev_span());

        if self.peek_token() == Some(Token::Newline) {
            self.advance();
        }

        Ok(Stmt::ExportFn {
            name,
            params,
            ret_type,
            body,
            span,
        })
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
        let mut prev_token: Option<&Token> = None;

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
                | Token::SlashEq
                | Token::PercentEq => {
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
                // Arrow ends expressions for switch arms, but not for lambdas
                // (where it follows a closing paren).
                Token::Arrow => {
                    if paren_depth == 0
                        && bracket_depth == 0
                        && !matches!(prev_token, Some(Token::RParen))
                    {
                        break;
                    }
                }
                _ => {}
            }

            prev_token = Some(token);
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
            Some(Token::PercentEq) => Some(AssignOp::PercentEq),
            _ => None,
        }
    }

    /// `name(params) => expr` or `name(params) => block` at statement level (TV style, no `fn` keyword)
    fn try_parse_tv_arrow_function_stmt(&mut self) -> Result<Option<Stmt>, ParseError> {
        let start_pos = self.pos;
        let start_span = match self.peek_span() {
            Some(s) => s,
            None => return Ok(None),
        };
        let name = match self.peek_info() {
            Some(info) => {
                if let Token::Ident(n) = &info.token {
                    Ident::new(n.clone(), info.span)
                } else {
                    return Ok(None);
                }
            }
            None => return Ok(None),
        };

        self.advance();
        if self.peek_token() != Some(Token::LParen) {
            self.pos = start_pos;
            return Ok(None);
        }
        self.advance(); // past '('
                        // TV `name(params) => ...` only: parameter lists start with `)` or an identifier.
                        // Reject call-shaped opens like `indicator("title", ...)` or `plot(na)`.
        match self.peek_token() {
            Some(Token::RParen) | Some(Token::Ident(_)) => {}
            _ => {
                self.pos = start_pos;
                return Ok(None);
            }
        }
        let params = match self.parse_params() {
            Ok(p) => p,
            Err(_) => {
                self.pos = start_pos;
                return Ok(None);
            }
        };
        if self.expect_token(Token::RParen, "expected ')'").is_err() {
            self.pos = start_pos;
            return Ok(None);
        }
        // Must see `=>` to commit as a TV arrow function; otherwise backtrack.
        let mut lookahead = self.pos;
        while self.tokens.get(lookahead).map(|t| t.token.clone()) == Some(Token::Newline) {
            lookahead += 1;
        }
        if self.tokens.get(lookahead).map(|t| t.token.clone()) != Some(Token::Arrow) {
            self.pos = start_pos;
            return Ok(None);
        }
        let (ret_type, body) = self.parse_fn_sig_body_after_params(None)?;
        let span = start_span.merge(self.prev_span());
        Ok(Some(Stmt::FnDef {
            name,
            params,
            ret_type,
            body,
            span,
        }))
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
    fn test_else_if_without_elif() {
        // Test else if chain without using elif keyword
        let input = r#"if x < 3
    a := 1
else if x < 7
    a := 2
else
    a := 3"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        assert_eq!(script.stmts.len(), 1);
        if let Stmt::If {
            cond,
            elifs,
            else_block,
            ..
        } = &script.stmts[0]
        {
            assert!(!elifs.is_empty(), "should have elifs");
            assert!(else_block.is_some(), "should have else block");
        } else {
            panic!("expected If statement");
        }
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

    #[test]
    fn test_tv_arrow_fn_expr() {
        // TV official syntax: expression body
        let input = "myDiff(a, b) => math.abs(a - b)";
        let script = parse(input).unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(
            &script.stmts[0],
            Stmt::FnDef {
                name,
                params,
                body: FnBody::Expr(_),
                ..
            } if name.name == "myDiff" && params.len() == 2
        ));
    }

    #[test]
    fn test_tv_arrow_fn_block() {
        // TV official syntax: indented block body after =>
        let input = "myScale(src, factor) =>\n    src * factor + 1.0\n";
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(
            &script.stmts[0],
            Stmt::FnDef {
                name,
                params,
                body: FnBody::Block(_),
                ..
            } if name.name == "myScale" && params.len() == 2
        ));
    }

    #[test]
    fn test_indicator_call_not_arrow_fn() {
        let input = r#"indicator("Hello pine-rs", shorttitle="hello")"#;
        let script = parse(input).unwrap();
        assert_eq!(script.stmts.len(), 1);
        assert!(matches!(script.stmts[0], Stmt::Expr(..)));
    }

    #[test]
    fn test_export_assign_lambda() {
        let input = "export add = (a, b) => a + b";
        let script = parse(input).unwrap();
        assert!(matches!(
            &script.stmts[0],
            Stmt::ExportAssign {
                init: Expr::Lambda { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_export_assign_const() {
        let input = "export PI = 3.14159";
        let script = parse(input).unwrap();
        let Stmt::ExportAssign { init, .. } = &script.stmts[0] else {
            panic!("expected ExportAssign");
        };
        assert!(matches!(init, Expr::Literal(Lit::Float(_), _)));
    }

    #[test]
    fn test_two_udf_call_sites_have_distinct_fn_call_spans() {
        let input = r#"fn f(x)
    x
v1 = f(100)
v2 = f(200)
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        let Stmt::VarDecl { init: Some(v1), .. } = &script.stmts[1] else {
            panic!("expected v1 = ..., got {:?}", script.stmts[1]);
        };
        let Stmt::VarDecl { init: Some(v2), .. } = &script.stmts[2] else {
            panic!("expected v2 = ..., got {:?}", script.stmts[2]);
        };
        let (Expr::FnCall { span: s1, .. }, Expr::FnCall { span: s2, .. }) = (v1, v2) else {
            panic!("expected FnCall values");
        };
        assert_ne!(
            (s1.start, s1.end),
            (s2.start, s2.end),
            "full-script offsets must differ for call-site interning"
        );
    }

    #[test]
    fn test_parse_switch_simple() {
        let input = r#"switch x % 3
    0 => r := 10
    1 => r := 20
"#;
        let tokens = pine_lexer::Lexer::lex_with_indentation(input).unwrap();
        let mut parser = StmtParser::new(tokens);
        let script = parser.parse_script().unwrap();
        assert!(matches!(script.stmts[0], Stmt::Switch { .. }));
    }
}
