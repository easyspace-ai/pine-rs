use logos::Logos;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Source span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// Token type for Pine Script v6
#[derive(Logos, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[logos(skip r"[ \t\r]+")] // Skip whitespace (but not newlines - handled separately)
#[logos(error = LexError)]
pub enum Token {
    // ===== Virtual tokens for indentation handling =====
    /// Virtual token for increased indentation level
    #[token("→INDENT")]
    Indent,

    /// Virtual token for decreased indentation level
    #[token("→DEDENT")]
    Dedent,

    /// Newline (significant for statement separation)
    #[token("\n", priority = 10)]
    Newline,

    // ===== Keywords =====
    /// Variable declaration
    #[token("var", priority = 20)]
    Var,

    /// Variable declaration with intra-bar persistence
    #[token("varip", priority = 20)]
    Varip,

    /// Type definition (v6)
    #[token("type", priority = 20)]
    Type,

    /// Method definition (v6)
    #[token("method", priority = 20)]
    Method,

    /// Import statement (v6)
    #[token("import", priority = 20)]
    Import,

    /// Export statement (v6)
    #[token("export", priority = 20)]
    Export,

    /// Library declaration (v6)
    #[token("library", priority = 20)]
    Library,

    /// If statement
    #[token("if", priority = 20)]
    If,

    /// Else statement
    #[token("else", priority = 20)]
    Else,

    /// Elif statement
    #[token("elif", priority = 20)]
    Elif,

    /// For loop
    #[token("for", priority = 20)]
    For,

    /// To (in for loop range)
    #[token("to", priority = 20)]
    To,

    /// By (in for loop step)
    #[token("by", priority = 20)]
    By,

    /// While loop
    #[token("while", priority = 20)]
    While,

    /// Break statement
    #[token("break", priority = 20)]
    Break,

    /// Continue statement
    #[token("continue", priority = 20)]
    Continue,

    /// Switch statement (v6)
    #[token("switch", priority = 20)]
    Switch,

    /// Case in switch (v6)
    #[token("case", priority = 20)]
    Case,

    /// Default in switch (v6)
    #[token("default", priority = 20)]
    Default,

    /// Function definition
    #[token("fn", priority = 20)]
    Fn,

    /// Return statement
    #[token("return", priority = 20)]
    Return,

    /// True literal
    #[token("true", priority = 20)]
    True,

    /// False literal
    #[token("false", priority = 20)]
    False,

    /// Na (not available) literal
    #[token("na", priority = 20)]
    Na,

    // ===== Operators =====
    /// +
    #[token("+")]
    Plus,

    /// -
    #[token("-")]
    Minus,

    /// *
    #[token("*")]
    Star,

    /// /
    #[token("/")]
    Slash,

    /// %
    #[token("%")]
    Percent,

    /// ^
    #[token("^")]
    Hat,

    /// ==
    #[token("==")]
    Eq,

    /// !=
    #[token("!=")]
    Neq,

    /// <
    #[token("<")]
    Lt,

    /// <=
    #[token("<=")]
    Le,

    /// >
    #[token(">")]
    Gt,

    /// >=
    #[token(">=")]
    Ge,

    /// and
    #[token("and", priority = 15)]
    And,

    /// or
    #[token("or", priority = 15)]
    Or,

    /// not
    #[token("not", priority = 15)]
    Not,

    /// =
    #[token("=")]
    Assign,

    /// :=
    #[token(":=")]
    ColonEq,

    /// +=
    #[token("+=")]
    PlusEq,

    /// -=
    #[token("-=")]
    MinusEq,

    /// *=
    #[token("*=")]
    StarEq,

    /// /=
    #[token("/=")]
    SlashEq,

    /// ?
    #[token("?")]
    Question,

    /// ?? (na coalesce)
    #[token("??")]
    Question2,

    /// => (arrow for methods/lambdas)
    #[token("=>")]
    Arrow,

    // ===== Delimiters =====
    /// (
    #[token("(")]
    LParen,

    /// )
    #[token(")")]
    RParen,

    /// [
    #[token("[")]
    LBracket,

    /// ]
    #[token("]")]
    RBracket,

    /// {
    #[token("{")]
    LBrace,

    /// }
    #[token("}")]
    RBrace,

    /// ,
    #[token(",")]
    Comma,

    /// .
    #[token(".")]
    Dot,

    /// :
    #[token(":")]
    Colon,

    /// ;
    #[token(";")]
    Semi,

    // ===== Literals =====
    /// Integer literal
    #[regex(r"-?\d[\d_]*", |lex| parse_int(lex.slice()))]
    #[regex(r"-?0x[0-9a-fA-F_]+", |lex| parse_hex(lex.slice()))]
    Int(i64),

    /// Float literal
    #[regex(r"-?\d[\d_]*\.\d[\d_]*([eE][+-]?\d+)?", |lex| parse_float(lex.slice()))]
    #[regex(r"-?\d[\d_]*[eE][+-]?\d+", |lex| parse_float(lex.slice()))]
    Float(f64),

    /// String literal (single or double quoted)
    #[regex(r#""([^"\\]|\\.)*""#, |lex| parse_string(lex.slice()))]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| parse_string(lex.slice()))]
    String(String),

    /// Color literal (#RGB, #RGBA, #RRGGBB, #RRGGBBAA)
    #[regex(r"#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})", |lex| parse_color(lex.slice()))]
    Color(u32),

    /// Identifier
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // ===== Comments =====
    /// Single-line comment
    #[regex(r"//[^\n]*", logos::skip)]
    Comment,

    /// Multi-line comment
    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip)]
    BlockComment,

    /// Error token
    Error,
}

/// Lexer error type
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LexError;

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexical error")
    }
}

impl std::error::Error for LexError {}

// ===== Parser functions for literals =====

fn parse_int(s: &str) -> Result<i64, LexError> {
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
    cleaned.parse().map_err(|_| LexError)
}

fn parse_hex(s: &str) -> Result<i64, LexError> {
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
    i64::from_str_radix(&cleaned[2..], 16).map_err(|_| LexError)
}

fn parse_float(s: &str) -> Result<f64, LexError> {
    let cleaned: String = s.chars().filter(|&c| c != '_').collect();
    cleaned.parse().map_err(|_| LexError)
}

fn parse_string(s: &str) -> Result<String, LexError> {
    // Remove surrounding quotes and handle escape sequences
    let inner = &s[1..s.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => return Err(LexError),
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn parse_color(s: &str) -> Result<u32, LexError> {
    let hex = &s[1..]; // Remove #

    let rgba = match hex.len() {
        3 => {
            // #RGB -> #RRGGBBFF
            let mut chars = hex.chars();
            let r = chars.next().unwrap().to_digit(16).unwrap() * 17;
            let g = chars.next().unwrap().to_digit(16).unwrap() * 17;
            let b = chars.next().unwrap().to_digit(16).unwrap() * 17;
            (r << 24) | (g << 16) | (b << 8) | 0xFF
        }
        4 => {
            // #RGBA -> #RRGGBBAA
            let mut chars = hex.chars();
            let r = chars.next().unwrap().to_digit(16).unwrap() * 17;
            let g = chars.next().unwrap().to_digit(16).unwrap() * 17;
            let b = chars.next().unwrap().to_digit(16).unwrap() * 17;
            let a = chars.next().unwrap().to_digit(16).unwrap() * 17;
            (r << 24) | (g << 16) | (b << 8) | a
        }
        6 => {
            // #RRGGBB -> #RRGGBBFF
            let rgb = u32::from_str_radix(hex, 16).map_err(|_| LexError)?;
            (rgb << 8) | 0xFF
        }
        8 => u32::from_str_radix(hex, 16).map_err(|_| LexError)?,
        _ => return Err(LexError),
    };

    Ok(rgba)
}

impl Token {
    /// Get the human-readable name of this token type
    pub fn name(&self) -> &'static str {
        match self {
            Token::Indent => "indent",
            Token::Dedent => "dedent",
            Token::Newline => "newline",
            Token::Var => "'var'",
            Token::Varip => "'varip'",
            Token::Type => "'type'",
            Token::Method => "'method'",
            Token::Import => "'import'",
            Token::Export => "'export'",
            Token::Library => "'library'",
            Token::If => "'if'",
            Token::Else => "'else'",
            Token::Elif => "'elif'",
            Token::For => "'for'",
            Token::To => "'to'",
            Token::By => "'by'",
            Token::While => "'while'",
            Token::Break => "'break'",
            Token::Continue => "'continue'",
            Token::Switch => "'switch'",
            Token::Case => "'case'",
            Token::Default => "'default'",
            Token::Fn => "'fn'",
            Token::Return => "'return'",
            Token::True => "'true'",
            Token::False => "'false'",
            Token::Na => "'na'",
            Token::Plus => "'+'",
            Token::Minus => "'-'",
            Token::Star => "'*'",
            Token::Slash => "'/'",
            Token::Percent => "'%'",
            Token::Hat => "'^'",
            Token::Eq => "'=='",
            Token::Neq => "'!='",
            Token::Lt => "'<'",
            Token::Le => "'<='",
            Token::Gt => "'>'",
            Token::Ge => "'>='",
            Token::And => "'and'",
            Token::Or => "'or'",
            Token::Not => "'not'",
            Token::Assign => "'='",
            Token::ColonEq => "':='",
            Token::PlusEq => "'+='",
            Token::MinusEq => "'-='",
            Token::StarEq => "'*='",
            Token::SlashEq => "'/='",
            Token::Question => "'?'",
            Token::Question2 => "'??'",
            Token::Arrow => "'=>'",
            Token::LParen => "'('",
            Token::RParen => "')'",
            Token::LBracket => "'['",
            Token::RBracket => "']'",
            Token::LBrace => "'{'",
            Token::RBrace => "'}'",
            Token::Comma => "','",
            Token::Dot => "'.'",
            Token::Colon => "':'",
            Token::Semi => "';'",
            Token::Int(_) => "integer",
            Token::Float(_) => "float",
            Token::String(_) => "string",
            Token::Color(_) => "color",
            Token::Ident(_) => "identifier",
            Token::Comment => "comment",
            Token::BlockComment => "block comment",
            Token::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lex = Token::lexer("var x = 42");
        assert_eq!(lex.next(), Some(Ok(Token::Var)));
        assert_eq!(lex.next(), Some(Ok(Token::Ident("x".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Assign)));
        assert_eq!(lex.next(), Some(Ok(Token::Int(42))));
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_operators() {
        let mut lex = Token::lexer("+ - * / == != <= >= ?? =>");
        assert_eq!(lex.next(), Some(Ok(Token::Plus)));
        assert_eq!(lex.next(), Some(Ok(Token::Minus)));
        assert_eq!(lex.next(), Some(Ok(Token::Star)));
        assert_eq!(lex.next(), Some(Ok(Token::Slash)));
        assert_eq!(lex.next(), Some(Ok(Token::Eq)));
        assert_eq!(lex.next(), Some(Ok(Token::Neq)));
        assert_eq!(lex.next(), Some(Ok(Token::Le)));
        assert_eq!(lex.next(), Some(Ok(Token::Ge)));
        assert_eq!(lex.next(), Some(Ok(Token::Question2)));
        assert_eq!(lex.next(), Some(Ok(Token::Arrow)));
    }

    #[test]
    fn test_color_literals() {
        let _lex = Token::lexer("#RGB #RGBA #RRGGBB #RRGGBBAA");
        // These won't match as colors since they contain letters
        // But actual hex colors will work
        let mut lex2 = Token::lexer("#ff5733 #ff5733cc");
        assert!(matches!(lex2.next(), Some(Ok(Token::Color(_)))));
        assert!(matches!(lex2.next(), Some(Ok(Token::Color(_)))));
    }

    #[test]
    fn test_string_literals() {
        let mut lex = Token::lexer(r#""hello" 'world' "escaped\n\t\\""#);
        assert_eq!(lex.next(), Some(Ok(Token::String("hello".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::String("world".to_string()))));
        assert_eq!(
            lex.next(),
            Some(Ok(Token::String("escaped\n\t\\".to_string())))
        );
    }

    #[test]
    fn test_numbers() {
        let mut lex = Token::lexer("42 3.14 1_000_000 0xFF 1e10");
        assert_eq!(lex.next(), Some(Ok(Token::Int(42))));
        assert_eq!(lex.next(), Some(Ok(Token::Float(3.14))));
        assert_eq!(lex.next(), Some(Ok(Token::Int(1000000))));
        assert_eq!(lex.next(), Some(Ok(Token::Int(255))));
        assert_eq!(lex.next(), Some(Ok(Token::Float(1e10))));
    }
}
