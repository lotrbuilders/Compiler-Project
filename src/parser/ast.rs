use super::Type;
use crate::span::Span;

// This module declares all the AST members that are used

// Represents the entire TranslationUnit
#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub global_declarations: Vec<ExternalDeclaration>,
}

// Represent a function or global variable declaration
#[derive(Debug, Clone)]
pub struct ExternalDeclaration {
    pub span: Span,
    pub ast_type: Type,
    pub name: String,
    pub function_body: Option<Vec<Statement>>,
    pub expression: Option<Expression>,
}

// Represents all possible statements
#[derive(Debug, Clone)]
pub enum Statement {
    Return {
        span: Span,
        expression: Expression,
    },

    If {
        span: Span,
        expression: Expression,
        statement: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },

    While {
        span: Span,
        expression: Expression,
        statement: Box<Statement>,
        do_while: bool,
    },

    For {
        span: Span,
        init: Option<Box<Statement>>,
        condition: Option<Box<Expression>>,
        expression: Option<Box<Expression>>,
        statement: Box<Statement>,
    },

    Break {
        span: Span,
    },

    Continue {
        span: Span,
    },

    Expression {
        span: Span,
        expression: Expression,
    },

    Empty(Span),

    Declaration {
        span: Span,
        ident: String,
        decl_type: Type,
        init: Option<Expression>,
    },

    Compound {
        span: Span,
        statements: Vec<Statement>,
    },
}

// Expression has a seperate expression variant
// This is used to seperate the shared components
#[derive(Debug, Clone)]
pub struct Expression {
    pub span: Span,
    pub ast_type: Type,
    pub variant: ExpressionVariant,
}

#[derive(Debug, Clone)]
pub enum ExpressionVariant {
    Assign(Box<Expression>, Box<Expression>),

    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
    Binary(BinaryExpressionType, Box<Expression>, Box<Expression>),
    Unary(UnaryExpressionType, Box<Expression>),

    Function(Box<Expression>, Vec<Expression>),

    ConstI(i128),
    CString(String),
    Ident(String, u32, bool),
}

#[derive(Debug, Clone)]
pub enum BinaryExpressionType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Inequal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BinOr,
    BinAnd,
    LogOr,
    LogAnd,
    Comma,
}

#[derive(Debug, Clone)]
pub enum UnaryExpressionType {
    Identity,
    Negate,
    BinNot,
    LogNot,
    Deref,
    Address,
}
