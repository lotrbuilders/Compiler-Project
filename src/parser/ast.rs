use super::Type;

#[derive(Debug, Default, Clone)]
pub struct TranslationUnit {
    pub global_declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Default, Clone)]
pub struct ExternalDeclaration {
    pub ast_type: Vec<Type>,
    pub function_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub ast_type: Vec<Type>,
    pub variant: ExpressionVariant,
}

#[derive(Debug, Clone)]
pub enum ExpressionVariant {
    ConstI(i128),
}
