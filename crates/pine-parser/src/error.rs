//! Error handling and recovery for the parser
//!
//! This module provides error recovery mechanisms to allow the parser
//! to continue after encountering errors and report multiple issues.

use pine_lexer::{Span, Token};

/// A single parse error with location information
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Location in source code
    pub span: Span,
    /// Error kind
    pub kind: ErrorKind,
}

/// Types of parse errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Unexpected token encountered
    UnexpectedToken { found: String, expected: String },
    /// Unexpected end of file
    UnexpectedEOF,
    /// Invalid indentation
    InvalidIndentation,
    /// Missing closing delimiter (paren, bracket, brace)
    UnclosedDelimiter { open: char, close: char },
    /// Invalid syntax
    InvalidSyntax,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedToken { found, expected } => {
                write!(f, "unexpected '{}', expected {}", found, expected)
            }
            ErrorKind::UnexpectedEOF => write!(f, "unexpected end of file"),
            ErrorKind::InvalidIndentation => write!(f, "invalid indentation"),
            ErrorKind::UnclosedDelimiter { open, close } => {
                write!(f, "unclosed delimiter: '{}' without '{}'", open, close)
            }
            ErrorKind::InvalidSyntax => write!(f, "invalid syntax"),
        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Create a new unexpected token error
    pub fn unexpected_token(
        found: impl Into<String>,
        expected: impl Into<String>,
        span: Span,
    ) -> Self {
        let found = found.into();
        let expected = expected.into();
        Self {
            message: format!("unexpected '{}', expected {}", found, expected),
            span,
            kind: ErrorKind::UnexpectedToken { found, expected },
        }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(span: Span) -> Self {
        Self {
            message: "unexpected end of file".to_string(),
            span,
            kind: ErrorKind::UnexpectedEOF,
        }
    }

    /// Create a new unclosed delimiter error
    pub fn unclosed_delimiter(open: char, close: char, span: Span) -> Self {
        Self {
            message: format!("unclosed '{}' without '{}'", open, close),
            span,
            kind: ErrorKind::UnclosedDelimiter { open, close },
        }
    }

    /// Create a new invalid syntax error
    pub fn invalid_syntax(message: impl Into<String>, span: Span) -> Self {
        let message = message.into();
        Self {
            message: message.clone(),
            span,
            kind: ErrorKind::InvalidSyntax,
        }
    }
}

/// Result of parsing with error recovery
#[derive(Debug)]
pub struct ParseResult<T> {
    /// Parsed result (may be partial if errors occurred)
    pub ast: T,
    /// Collected errors during parsing
    pub errors: Vec<ParseError>,
}

impl<T> ParseResult<T> {
    /// Create a new successful parse result
    pub fn new(ast: T) -> Self {
        Self {
            ast,
            errors: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    /// Check if there were any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Convert to a Result, returning Err if there were errors
    pub fn into_result(self) -> Result<T, Vec<ParseError>> {
        if self.errors.is_empty() {
            Ok(self.ast)
        } else {
            Err(self.errors)
        }
    }
}

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStrategy {
    /// Stop at first error
    Stop,
    /// Skip to next statement boundary
    SkipToStatement,
    /// Skip to synchronization token
    SkipToSync,
}

/// Synchronization points for error recovery
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncPoint {
    /// Newline (statement boundary)
    Newline,
    /// Indent/Dedent (block boundary)
    Indentation,
    /// Block keyword (if, for, while, fn, etc.)
    BlockStart,
    /// Expression delimiter
    Delimiter,
}

impl SyncPoint {
    /// Check if token matches this sync point
    pub fn matches(&self, token: &Token) -> bool {
        match self {
            SyncPoint::Newline => matches!(token, Token::Newline),
            SyncPoint::Indentation => matches!(token, Token::Indent | Token::Dedent),
            SyncPoint::BlockStart => matches!(
                token,
                Token::If
                    | Token::For
                    | Token::While
                    | Token::Fn
                    | Token::Type
                    | Token::Method
                    | Token::Switch
                    | Token::Var
                    | Token::Varip
                    | Token::Import
                    | Token::Export
                    | Token::Library
            ),
            SyncPoint::Delimiter => matches!(token, Token::RParen | Token::RBracket | Token::Comma),
        }
    }
}

/// Error recovery context
#[derive(Debug)]
pub struct RecoveryContext {
    /// Recovery strategy
    pub strategy: RecoveryStrategy,
    /// Synchronization points
    pub sync_points: Vec<SyncPoint>,
    /// Maximum number of errors to collect
    pub max_errors: usize,
}

impl Default for RecoveryContext {
    fn default() -> Self {
        Self {
            strategy: RecoveryStrategy::SkipToStatement,
            sync_points: vec![SyncPoint::Newline, SyncPoint::BlockStart],
            max_errors: 10,
        }
    }
}

impl RecoveryContext {
    /// Create a new recovery context with stop strategy
    pub fn stop() -> Self {
        Self {
            strategy: RecoveryStrategy::Stop,
            sync_points: Vec::new(),
            max_errors: 1,
        }
    }

    /// Create a new recovery context with skip-to-statement strategy
    pub fn skip_to_statement() -> Self {
        Self {
            strategy: RecoveryStrategy::SkipToStatement,
            sync_points: vec![
                SyncPoint::Newline,
                SyncPoint::BlockStart,
                SyncPoint::Indentation,
            ],
            max_errors: 10,
        }
    }

    /// Check if we should stop parsing
    pub fn should_stop(&self, error_count: usize) -> bool {
        self.strategy == RecoveryStrategy::Stop || error_count >= self.max_errors
    }

    /// Check if token is a synchronization point
    pub fn is_sync_token(&self, token: &Token) -> bool {
        self.sync_points.iter().any(|sp| sp.matches(token))
    }
}

/// Trait for types that can be recovered from errors
pub trait Recoverable {
    /// Merge partial results from recovery
    fn merge(&mut self, other: Self);
    /// Check if this is a valid (complete) result
    fn is_valid(&self) -> bool;
}

impl Recoverable for crate::ast::Script {
    fn merge(&mut self, other: Self) {
        self.stmts.extend(other.stmts);
        self.span = self.span.merge(other.span);
    }

    fn is_valid(&self) -> bool {
        !self.stmts.is_empty()
    }
}
