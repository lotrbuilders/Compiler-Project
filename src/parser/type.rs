use smallvec::{smallvec, SmallVec};

use crate::{
    table::StructTable,
    token::{Token, TokenType},
};
use std::fmt::Display;

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
    Void,
    Pointer,
    Struct(usize),
    Array(usize),
    Function(Vec<Type>),
}

impl TypeNode {
    pub fn error() -> TypeNode {
        TypeNode::Int
    }
}

// Type contains  C Type used by something
// The Name if any should be the highest
// This is followed in order of dereferencing/calling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub nodes: SmallVec<[TypeNode; 2]>,
}

impl Type {
    pub fn empty() -> Type {
        Type {
            nodes: SmallVec::new(),
        }
    }
    pub fn int() -> Type {
        Type {
            nodes: smallvec![TypeNode::Int],
        }
    }
    pub fn pointer() -> Type {
        Type {
            nodes: smallvec![TypeNode::Pointer],
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
            Some(TypeNode::Void) => false,
            _ => true,
        }
    }

    pub fn is_declaration(&self) -> bool {
        self.get_function_arguments()
            .map(|args| args.len() == 0)
            .unwrap_or(false)
    }

    pub fn is_char(&self) -> bool {
        match self.nodes.get(0) {
            Some(TypeNode::Char) => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        matches!(self.nodes.get(0), Some(TypeNode::Void))
    }

    pub fn is_void_pointer(&self) -> bool {
        matches!(
            (self.nodes.get(0), self.nodes.get(1)),
            (Some(TypeNode::Pointer), Some(TypeNode::Void))
        )
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

    pub fn is_callable(&self) -> bool {
        self.is_function() || self.is_function_pointer()
    }

    pub fn is_function_pointer(&self) -> bool {
        matches!(self.nodes.get(0), Some(TypeNode::Pointer)) && Type::is_function2(&self.nodes[1..])
    }

    pub fn is_function(&self) -> bool {
        Type::is_function2(&self.nodes)
    }

    fn is_function2(input: &[TypeNode]) -> bool {
        match input.get(0) {
            Some(TypeNode::Function(_)) => true,
            _ => false,
        }
    }

    pub fn get_function_arguments<'a>(&'a self) -> Option<&'a Vec<Type>> {
        Type::get_function_arguments2(&self.nodes)
            .or_else(|| Type::get_function_arguments2(&self.nodes[1..]))
    }

    fn get_function_arguments2<'a>(input: &'a [TypeNode]) -> Option<&'a Vec<Type>> {
        match input.get(0) {
            Some(TypeNode::Function(arguments)) => Some(arguments),
            _ => None,
        }
    }

    pub fn get_element(&self) -> TypeNode {
        Type::get_element2(&self.nodes)
    }
    fn get_element2(nodes: &[TypeNode]) -> TypeNode {
        match nodes.get(0) {
            Some(TypeNode::Array(..)) => Type::get_element2(&nodes[1..]),
            Some(t) => t.clone(),
            None => unreachable!(),
        }
    }

    pub fn remove_name(self) -> Type {
        self
    }

    pub fn get_return_type<'a>(&'a self) -> Option<&'a [TypeNode]> {
        if self.is_function_pointer() {
            Type::get_return_type2(&self.nodes[1..])
        } else {
            Type::get_return_type2(&self.nodes)
        }
    }

    fn get_return_type2<'a>(input: &'a [TypeNode]) -> Option<&'a [TypeNode]> {
        match input.get(0) {
            Some(TypeNode::Function(_)) => Some(&input[1..]),
            Some(_) => Some(&input[0..]),
            _ => None,
        }
    }

    pub fn get_struct_index(&self) -> usize {
        Type::get_struct_index2(&self.nodes, true)
    }

    pub fn get_struct_index2(nodes: &[TypeNode], first: bool) -> usize {
        match (nodes.get(0), first) {
            (Some(TypeNode::Struct(index)), _) => *index,
            (Some(TypeNode::Pointer), true) => Type::get_struct_index2(&nodes[1..], false),
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

    pub fn deconstruct(&self) -> (TypeNode, usize) {
        Type::deconstruct2(&self.nodes)
    }

    fn deconstruct2(typ: &[TypeNode]) -> (TypeNode, usize) {
        match typ.get(0) {
            Some(TypeNode::Array(size)) => {
                let (t, s) = Type::deconstruct2(&typ[1..]);
                (t, s * size)
            }
            Some(t) => (t.clone(), 1),
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
            Void => TypeNode::Void,
            _ => {
                log::error!("From<Token> for Type called on unqualified type");
                std::process::exit(1);
            }
        }
    }
}

impl From<Vec<TypeNode>> for Type {
    fn from(nodes: Vec<TypeNode>) -> Type {
        Type {
            nodes: nodes.into_iter().collect(),
        }
    }
}

impl From<&[TypeNode]> for Type {
    fn from(nodes: &[TypeNode]) -> Self {
        Type {
            nodes: nodes.iter().map(|t| t.clone()).collect(),
        }
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
            Void => write!(f, "void ")?,
            Pointer => write!(f, "* ")?,
            Function(arguments) => {
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
                format_type(&typ[0..i], f, table)?;
                write!(f, "[{}]", size)?;
                break;
            }

            Struct(index) => {
                if let Some(table) = table {
                    if let Some(name) = &table[*index].name {
                        write!(f, "struct {}__{} ", name, index)?;
                    } else {
                        write!(f, "__anonymous_struct__{} ", index)?;
                    }
                } else {
                    write!(f, "struct ")?
                }
            }
        };
    }
    Ok(())
}
