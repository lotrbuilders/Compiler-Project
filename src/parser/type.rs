use crate::{
    table::StructTable,
    token::{Token, TokenType},
};
use std::fmt::Display;

use super::ast_print::ASTDisplay;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum DeclarationType {
    Declaration,
    Prototype,
    Definition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType {
    pub name: Option<String>,
    pub members: Option<Vec<(String, Type)>>,
}

impl StructType {
    pub fn is_qualified(&self) -> bool {
        matches!(self.members, Some(..))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeNode {
    Char,
    Int,
    Long,
    Short,
    Pointer,
    Struct(usize),
    Array(usize),
    Name(String),
    Function(Vec<Type>),
}

// Type contains  C Type used by something
// The Name if any should be the highest
// This is followed in order of dereferencing/calling
#[derive(Debug, Clone, Eq)]
pub struct Type {
    pub nodes: Vec<TypeNode>,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        let mut lhs = self.nodes.iter().peekable();
        let mut rhs = other.nodes.iter().peekable();
        use TypeNode::*;
        while lhs.peek().is_some() && rhs.peek().is_some() {
            if let Some(Name(_)) = lhs.peek() {
                lhs.next();
                continue;
            } else if let Some(Name(_)) = rhs.peek() {
                rhs.next();
                continue;
            } else {
                if lhs.next() != rhs.next() {
                    return false;
                }
            }
        }
        lhs.next().is_none() && rhs.next().is_none()
    }
}

impl Type {
    pub fn empty() -> Type {
        Type { nodes: Vec::new() }
    }
    pub fn int() -> Type {
        Type {
            nodes: vec![TypeNode::Int],
        }
    }
    pub fn pointer() -> Type {
        Type {
            nodes: vec![TypeNode::Pointer],
        }
    }
    pub fn error() -> Type {
        Type::int()
    }

    pub fn is_qualified(&self, struct_table: &StructTable) -> bool {
        Type::is_qualified2(&self.nodes, struct_table)
    }
    fn is_qualified2(nodes: &[TypeNode], struct_table: &StructTable) -> bool {
        match nodes.get(0) {
            Some(TypeNode::Struct(index)) => struct_table[*index].is_qualified(),
            Some(TypeNode::Array(..)) => Type::is_qualified2(&nodes[1..], struct_table),
            _ => true,
        }
    }

    pub fn is_char(&self) -> bool {
        match self.nodes.get(0) {
            Some(TypeNode::Char) => true,
            _ => false,
        }
    }

    pub fn is_declaration(&self) -> bool {
        self.get_function_arguments()
            .map(|args| args.len() == 0)
            .unwrap_or(false)
    }

    pub fn is_pointer(&self) -> bool {
        match self.nodes.get(0) {
            Some(TypeNode::Pointer) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self.nodes.get(0) {
            Some(TypeNode::Array(..)) => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match self.nodes.get(0) {
            Some(TypeNode::Struct(..)) => true,
            _ => false,
        }
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

    pub fn has_name(&self) -> bool {
        matches!(self.nodes.get(0), Some(TypeNode::Name(..)))
    }
    pub fn get_name(&self) -> Option<String> {
        match self.nodes.get(0) {
            Some(TypeNode::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }
    pub fn remove_name(self) -> Type {
        match self.nodes.get(0) {
            Some(TypeNode::Name(_)) => self.nodes[1..].into(),
            _ => self.clone(),
        }
    }

    pub fn get_return_type<'a>(&'a self) -> Option<&'a [TypeNode]> {
        Type::get_return_type2(&self.nodes)
    }

    fn get_return_type2<'a>(input: &'a [TypeNode]) -> Option<&'a [TypeNode]> {
        match input.get(0) {
            Some(TypeNode::Name(_)) => Type::get_return_type2(&input[1..]),
            Some(TypeNode::Function(_)) => Some(&input[1..]),
            Some(_) => Some(&input[0..]),
            _ => None,
        }
    }

    pub fn get_struct_index(&self) -> usize {
        match self.nodes.get(0) {
            Some(TypeNode::Struct(index)) => *index,
            _ => unreachable!(),
        }
    }

    // Function works under current definition of the types, but might need to be processed further when more types are introduced
    pub fn combine(mut base_type: Type, mut declarator: Type) -> Type {
        declarator.nodes.append(&mut base_type.nodes);
        declarator
    }

    pub fn append(mut self, other: &Type) -> Type {
        self.nodes.extend(other.nodes.clone());
        self
    }

    pub fn deref(self) -> Type {
        if let Some(TypeNode::Pointer | TypeNode::Array(..)) = self.nodes.get(0) {
            self.nodes[1..].into()
        } else {
            self
        }
    }

    pub fn deconstruct(&self) -> (Type, usize) {
        Type::deconstruct2(&self.nodes)
    }

    fn deconstruct2(typ: &[TypeNode]) -> (Type, usize) {
        match typ.get(0) {
            Some(TypeNode::Array(size)) => {
                let (t, s) = Type::deconstruct2(&typ[1..]);
                (t, s * size)
            }
            Some(..) => (typ.into(), 1),
            None => unreachable!(),
        }
    }
}

impl From<Token> for TypeNode {
    fn from(token: Token) -> TypeNode {
        use TokenType::*;
        match token.token() {
            Char => TypeNode::Char,
            Int => TypeNode::Int,
            Long => TypeNode::Long,
            Short => TypeNode::Short,
            Asterisk => TypeNode::Pointer,
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

impl From<&[TypeNode]> for Type {
    fn from(nodes: &[TypeNode]) -> Self {
        Type {
            nodes: nodes.iter().map(|t| t.clone()).collect(),
        }
    }
}

impl ASTDisplay for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter, table: &StructTable) -> std::fmt::Result {
        format_type(&self.nodes, f, Some(table))
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_type(&self.nodes, f, None)
    }
}

fn format_type(
    typ: &[TypeNode],
    f: &mut std::fmt::Formatter<'_>,
    table: Option<&StructTable>,
) -> std::fmt::Result {
    if typ.is_empty() {
        return Ok(());
    }
    for i in (0..=(typ.len() - 1)).rev() {
        use TypeNode::*;
        match &typ[i] {
            Char => write!(f, "char ")?,
            Int => write!(f, "int ")?,
            Long => write!(f, "long ")?,
            Short => write!(f, "short ")?,
            Pointer => write!(f, "* ")?,
            Name(name) => write!(f, "{}", name)?,
            Function(arguments) => {
                //Extend later when functions are fully implemented
                format_type(&typ[0..i], f, table)?;
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
            Array(size) => {
                //Extend later when functions are fully implemented
                format_type(&typ[0..i], f, table)?;
                write!(f, "[{}]", size)?;
                break;
            }

            Struct(index) => {
                if let Some(table) = table {
                    if let Some(name) = &table[*index].name {
                        write!(f, "struct {}__{}", name, index)?;
                    } else {
                        write!(f, "__anonymous_struct__{}", index)?;
                    }
                } else {
                    write!(f, "struct")?
                }
            }
        };
    }
    Ok(())
}
