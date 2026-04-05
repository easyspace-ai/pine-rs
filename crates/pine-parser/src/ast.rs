use pine_lexer::Span;
use serde::{Deserialize, Serialize};

/// Identifier with span
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// Variable kind (var, varip, or plain)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VarKind {
    /// Plain variable (re-initialized each bar)
    Plain,
    /// var - persists across bars
    Var,
    /// varip - persists across bars and intra-bar updates
    Varip,
}

/// Assignment operator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssignOp {
    /// =
    Assign,
    /// :=
    ColonEq,
    /// +=
    PlusEq,
    /// -=
    MinusEq,
    /// *=
    StarEq,
    /// /=
    SlashEq,
    /// %=
    PercentEq,
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// ^
    Pow,
    /// ==
    Eq,
    /// !=
    Neq,
    /// <
    Lt,
    /// <=
    Le,
    /// >
    Gt,
    /// >=
    Ge,
    /// and
    And,
    /// or
    Or,
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// -
    Neg,
    /// not
    Not,
}

/// Literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
    Na,
}

/// Type annotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnn {
    /// Simple type (int, float, bool, string, color)
    Simple(String),
    /// Series type (series int, series float, etc.)
    Series(Box<TypeAnn>),
    /// Array type (array<int>, array<float>, etc.)
    Array(Box<TypeAnn>),
    /// Matrix type (matrix<int>, matrix<float>, etc.)
    Matrix(Box<TypeAnn>),
    /// Map type (map<string, int>, etc.)
    Map(Box<TypeAnn>, Box<TypeAnn>),
    /// User-defined type
    User(String),
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: Ident,
    pub type_ann: Option<TypeAnn>,
    pub default: Option<Expr>,
}

/// Field definition (for type definitions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: Ident,
    pub type_ann: Option<TypeAnn>,
    pub default: Option<Expr>,
}

/// Argument in function call
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arg {
    pub name: Option<Ident>,
    pub value: Expr,
}

/// TV v6 switch arm: [pattern] => body (pattern omitted = default arm)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchArm {
    pub pattern: Option<Expr>,
    pub body: SwitchArmBody,
    pub span: Span,
}

/// Body after `=>` in a switch arm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SwitchArmBody {
    Expr(Expr),
    Block(Block),
}

/// Function / export / method body: block or `=> expr`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FnBody {
    Block(Block),
    Expr(Expr),
}

/// `for ... in` loop variable pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForInPattern {
    Single(Ident),
    Tuple(Ident, Ident),
}

/// Enum field (Pine v6)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: Ident,
    pub init: Option<Expr>,
}

/// Library import path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportPath {
    String(String),
    /// e.g. `username/lib_name/1`
    Qualified(Vec<String>),
}

/// Block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// Assignment target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    /// Simple variable: x
    Var(Ident),
    /// Tuple destructuring assignment: [a, b, c]
    Tuple(Vec<Ident>),
    /// Array/series index: arr[0], close[1]
    Index { base: Box<Expr>, offset: Box<Expr> },
    /// Field access: obj.field
    Field { base: Box<Expr>, field: Ident },
}

impl AssignTarget {
    /// Get the span of this assignment target
    pub fn span(&self) -> Span {
        match self {
            AssignTarget::Var(ident) => ident.span,
            AssignTarget::Tuple(idents) => idents
                .iter()
                .fold(Span::default(), |acc, ident| acc.merge(ident.span)),
            AssignTarget::Index { base, offset } => base.span().merge(offset.span()),
            AssignTarget::Field { base, field } => base.span().merge(field.span),
        }
    }
}

/// Statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Variable declaration: [var/varip] name[: type] [= init]
    VarDecl {
        name: Ident,
        kind: VarKind,
        type_ann: Option<TypeAnn>,
        init: Option<Expr>,
        span: Span,
    },

    /// Assignment: target op value
    Assign {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
        span: Span,
    },

    /// If statement: if cond then_block [elif ...] [else else_block]
    If {
        cond: Expr,
        then_block: Block,
        elifs: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },

    /// For loop: for var = from to to [by step] body
    For {
        var: Ident,
        from: Expr,
        to: Expr,
        by: Option<Expr>,
        body: Block,
        span: Span,
    },

    /// For-in loop: for x in arr / for [i, v] in arr
    ForIn {
        pattern: ForInPattern,
        iterable: Expr,
        body: Block,
        span: Span,
    },

    /// While loop: while cond body
    While { cond: Expr, body: Block, span: Span },

    /// Switch (TV v6): optional scrutinee + `pattern => body` arms
    Switch {
        scrutinee: Option<Expr>,
        arms: Vec<SwitchArm>,
        span: Span,
    },

    /// Break statement
    Break { span: Span },

    /// Continue statement
    Continue { span: Span },

    /// Return statement: return [value]
    Return { value: Option<Expr>, span: Span },

    /// Function definition: fn name(params) [-> ret_type] body  OR  body via =>
    FnDef {
        name: Ident,
        params: Vec<Param>,
        ret_type: Option<TypeAnn>,
        body: FnBody,
        span: Span,
    },

    /// Type definition (v6): type Name { fields... }
    TypeDef {
        name: Ident,
        fields: Vec<Field>,
        span: Span,
    },

    /// Enum definition (v6): enum Name ...
    EnumDef {
        name: Ident,
        variants: Vec<EnumVariant>,
        span: Span,
    },

    /// Method definition (v6): method Type.name(params) body
    MethodDef {
        type_name: Ident,
        name: Ident,
        params: Vec<Param>,
        ret_type: Option<TypeAnn>,
        body: FnBody,
        span: Span,
    },

    /// Import: import "s" | import a/b/c [as name]
    Import {
        path: ImportPath,
        alias: Option<Ident>,
        span: Span,
    },

    /// Export function (v6): export name(params) [=> expr | block]
    ExportFn {
        name: Ident,
        params: Vec<Param>,
        ret_type: Option<TypeAnn>,
        body: FnBody,
        span: Span,
    },

    /// Export binding (v6): `export name = expr` (e.g. lambda `(a,b) => a + b` or constant)
    ExportAssign { name: Ident, init: Expr, span: Span },

    /// Library declaration (v6): library(name [, overlay = true])
    Library {
        name: String,
        props: Vec<(Ident, Expr)>,
        span: Span,
    },

    /// Expression statement
    Expr(Expr),
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl { span, .. } => *span,
            Stmt::Assign { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::ForIn { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::Switch { span, .. } => *span,
            Stmt::Break { span } => *span,
            Stmt::Continue { span } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::FnDef { span, .. } => *span,
            Stmt::TypeDef { span, .. } => *span,
            Stmt::EnumDef { span, .. } => *span,
            Stmt::MethodDef { span, .. } => *span,
            Stmt::Import { span, .. } => *span,
            Stmt::ExportFn { span, .. } => *span,
            Stmt::ExportAssign { span, .. } => *span,
            Stmt::Library { span, .. } => *span,
            Stmt::Expr(expr) => expr.span(),
        }
    }
}

/// Expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value
    Literal(Lit, Span),

    /// Variable reference
    Ident(Ident),

    /// Binary operation: lhs op rhs
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },

    /// Unary operation: op operand
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },

    /// Ternary: cond ? then : else
    Ternary {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        span: Span,
    },

    /// Na coalesce: lhs ?? rhs
    NaCoalesce {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },

    /// Index access: base[offset]
    Index {
        base: Box<Expr>,
        offset: Box<Expr>,
        span: Span,
    },

    /// Field access: base.field
    FieldAccess {
        base: Box<Expr>,
        field: Ident,
        span: Span,
    },

    /// Function call: func(args)
    FnCall {
        func: Box<Expr>,
        args: Vec<Arg>,
        span: Span,
    },

    /// Method call: base.method(args)
    MethodCall {
        base: Box<Expr>,
        method: Ident,
        args: Vec<Arg>,
        span: Span,
    },

    /// Array literal: [expr, ...]
    ArrayLit(Vec<Expr>, Span),

    /// Map literal: [key: value, ...]
    MapLit(Vec<(Expr, Expr)>, Span),

    /// Anonymous function/lambda (v6): (params) => expr
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        span: Span,
    },

    /// Switch expression: switch [scrutinee] { pattern => expr, ... }
    /// Returns the value of the first matching arm
    SwitchExpr {
        scrutinee: Option<Box<Expr>>,
        arms: Vec<SwitchArm>,
        span: Span,
    },
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Ident(ident) => ident.span,
            Expr::BinOp { span, .. } => *span,
            Expr::UnaryOp { span, .. } => *span,
            Expr::Ternary { span, .. } => *span,
            Expr::NaCoalesce { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::FnCall { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::ArrayLit(_, span) => *span,
            Expr::MapLit(_, span) => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::SwitchExpr { span, .. } => *span,
        }
    }
}

/// The complete AST for a Pine Script
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}
