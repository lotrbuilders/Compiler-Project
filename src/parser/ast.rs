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
    pub ast_type: Vec<Type>,
    pub name: String,
    pub function_body: Option<Vec<Statement>>,
}

// Represents all possible statements
#[derive(Debug, Clone)]
pub enum Statement {
    Return { span: Span, expression: Expression },
}

// Expression has a seperate expression variant
// This is used to seperate the shared components
#[derive(Debug, Clone)]
pub struct Expression {
    pub span: Span,
    pub ast_type: Vec<Type>,
    pub variant: ExpressionVariant,
}

#[derive(Debug, Clone)]
pub enum ExpressionVariant {
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    ConstI(i128),
}
