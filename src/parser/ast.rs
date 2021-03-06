use super::{Type, TypeNode};
use crate::{span::Span, token::Token};

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
    pub decl_type: Type,
    pub ast_type: ASTType,
    pub name: Option<String>,
    pub function_body: Option<Vec<Statement>>,
    pub expression: Option<Expression>,
}

// Represents all possible statements
#[derive(Debug, Clone)]
pub enum Statement {
    Return {
        span: Span,
        ast_type: Type,
        expression: Option<Expression>,
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
        ident: Option<String>,
        decl_type: Type,
        ast_type: ASTType,
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

impl Expression {
    pub fn default(span: &Span) -> Expression {
        Expression {
            span: span.clone(),
            ast_type: Type::empty(),
            variant: ExpressionVariant::ConstI(0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionVariant {
    Assign(Box<Expression>, Box<Expression>),

    Ternary(Box<Expression>, Box<Expression>, Box<Expression>),
    Binary(BinaryExpressionType, Box<Expression>, Box<Expression>),
    Unary(UnaryExpressionType, Box<Expression>),
    Member(Box<Expression>, String, bool, u16),
    Cast(Box<Expression>, ASTType),

    Function(Box<Expression>, Vec<Expression>),

    Sizeof(SizeofType),
    ConstI(i128),
    CString(String),
    Ident(String, u32, bool),
}

#[derive(Debug, Clone)]
pub enum SizeofType {
    Type(ASTType, Type),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum BinaryExpressionType {
    Index,
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

#[derive(Debug, Clone)]
pub struct ASTType {
    pub span: Span,
    pub list: Vec<ASTTypeNode>,
}
#[derive(Debug, Clone)]
pub enum ASTTypeNode {
    Simple(TypeNode),
    Array(Box<Expression>),
    Struct(Box<ASTStruct>),
    Name(String),
    Function(Vec<ASTType>),
}

#[derive(Debug, Clone)]
pub struct ASTStruct {
    pub name: Option<String>,
    pub members: Option<Vec<ASTType>>,
}

impl ASTType {
    pub fn combine(mut self, mut rhs: ASTType) -> ASTType {
        let span = self.span.to(&rhs.span);
        rhs.list.append(&mut self.list);
        ASTType {
            span,
            list: rhs.list,
        }
    }
    pub fn from_slice(slice: &[ASTTypeNode], span: Span) -> ASTType {
        ASTType {
            span,
            list: slice.into(),
        }
    }
    pub fn get_name(&self) -> Option<String> {
        match self.list.get(0) {
            Some(ASTTypeNode::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }
    pub fn has_name(&self) -> bool {
        self.get_name().is_some()
    }
    pub fn is_type_declaration(&self) -> bool {
        use ASTTypeNode::*;
        for entry in &self.list {
            match entry {
                Struct(s) => return s.members.is_some() && s.name.is_some(),
                Name(_) => continue,
                _ => break,
            }
        }
        false
    }
}

impl From<Token> for ASTTypeNode {
    fn from(t: Token) -> Self {
        ASTTypeNode::Simple(t.into())
    }
}
