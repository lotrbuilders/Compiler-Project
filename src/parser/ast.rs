use super::Type;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct TranslationUnit {
    pub global_declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Clone)]
pub struct ExternalDeclaration {
    pub span: Span,
    pub ast_type: Vec<Type>,
    pub function_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return { span: Span, expression: Expression },
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub span: Span,
    pub ast_type: Vec<Type>,
    pub variant: ExpressionVariant,
}

#[derive(Debug, Clone)]
pub enum ExpressionVariant {
    ConstI(i128),
}
