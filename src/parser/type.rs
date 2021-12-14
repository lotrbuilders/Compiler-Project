use std::fmt::Display;

use crate::token::{Token, TokenType};

// Type contains a component C Type used by something
// The Name if any should be the highes
// This is followed in order of dereferencing/calling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeNode {
    Int,
    Name(String),
    Function(Vec<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    nodes: Vec<TypeNode>,
}

impl Type {
    pub fn empty() -> Type {
        Type { nodes: Vec::new() }
    }
    pub fn is_function(&self) -> bool {
        Type::is_function2(&self.nodes)
    }

    fn is_function2(input: &[TypeNode]) -> bool {
        match input.get(0) {
            Some(TypeNode::Name(_)) => Type::is_function2(&input[1..]),
            Some(TypeNode::Function(_)) => true,
            _ => false,
        }
    }

    pub fn get_function_arguments<'a>(&'a self) -> Option<&'a Vec<Type>> {
        Type::get_function_arguments2(&self.nodes)
    }

    fn get_function_arguments2<'a>(input: &'a [TypeNode]) -> Option<&'a Vec<Type>> {
        match input.get(0) {
            Some(TypeNode::Name(_)) => Type::get_function_arguments2(&input[1..]),
            Some(TypeNode::Function(arguments)) => Some(arguments),
            _ => None,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        match self.nodes.get(0) {
            Some(TypeNode::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }

    // Function works under current definition of the types, but might need to be processed further when more types are introduced
    pub fn combine(mut base_type: Type, mut declarator: Type) -> Type {
        declarator.nodes.append(&mut base_type.nodes);
        declarator
    }
}

impl From<Token> for TypeNode {
    fn from(token: Token) -> TypeNode {
        use TokenType::*;
        match token.token() {
            Int => TypeNode::Int,
            Ident(name) => TypeNode::Name(name),
            _ => {
                log::error!("From<Token> for Type called on unqualified type");
                std::process::exit(1);
            }
        }
    }
}

impl From<Vec<TypeNode>> for Type {
    fn from(nodes: Vec<TypeNode>) -> Type {
        Type { nodes }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_type(&self.nodes, f)
    }
}

fn format_type(typ: &[TypeNode], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if typ.is_empty() {
        return Ok(());
    }
    for i in (0..=(typ.len() - 1)).rev() {
        use TypeNode::*;
        match &typ[i] {
            Int => write!(f, "int ")?,
            Name(name) => write!(f, "{}", name)?,
            Function(arguments) => {
                //Extend later when functions are fully implemented
                format_type(&typ[0..i], f)?;
                write!(f, "(")?;
                if let Some(arg) = arguments.get(0) {
                    write!(f, "{}", arg)?;
                }
                for arg in arguments.iter().skip(1) {
                    write!(f, ", {}", arg)?;
                }
                write!(f, ")")?;
                break;
            }
        };
    }
    Ok(())
}
