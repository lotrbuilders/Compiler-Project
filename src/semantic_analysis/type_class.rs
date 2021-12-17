use std::fmt::Display;

use crate::{
    error,
    parser::{Type, TypeNode},
    span::Span,
};

use super::SemanticAnalyzer;

#[derive(Clone, Copy)]
pub enum TypeClass {
    Pointer,
    Function,
    StandardSignedInteger,
    StandardInteger,
    Integer,
    Scalar,
    Arithmetic,
}

impl SemanticAnalyzer {
    pub fn assert_in(&mut self, span: &Span, typ: &Type, class: TypeClass) {
        if !typ.is_in(class) {
            self.errors
                .push(error!(span, "Expected {} to be {}", typ, class))
        }
    }

    pub fn assert_both_in(&mut self, span: &Span, left: &Type, right: &Type, class: TypeClass) {
        self.assert_in(span, left, class);
        self.assert_in(span, right, class);
    }

    pub fn assert_compatible(&mut self, span: &Span, left: &Type, right: &Type) {
        if !left.is_compatible(right) {
            self.errors.push(error!(
                span,
                "Types {} and {} are not compatible", left, right
            ))
        }
    }
}

impl Type {
    pub fn is_in(&self, class: TypeClass) -> bool {
        use TypeNode::*;
        match self.nodes.get(0) {
            Some(Name(_)) => {
                log::error!("Name should not be passed to is_in");
                true
            }
            None => {
                log::error!("Type was improperly passed");
                true
            }
            Some(_) => self.is_in2(class),
        }
    }

    fn is_in2(&self, class: TypeClass) -> bool {
        use TypeClass::*;
        match class {
            StandardSignedInteger => self.nodes[0] == TypeNode::Int,
            Function => matches!(self.nodes[0], TypeNode::Function(_)),
            Pointer => self.nodes[0] == TypeNode::Pointer,
            StandardInteger => self.is_in2(StandardSignedInteger),
            Integer => self.is_in2(StandardInteger),
            Arithmetic => self.is_in2(Integer),
            Scalar => self.is_in2(Arithmetic) | self.is_in2(Pointer),
        }
    }

    pub fn is_compatible(&self, other: &Type) -> bool {
        Type::is_compatible2(&self.nodes, &other.nodes)
    }

    fn is_compatible2(lhs: &[TypeNode], rhs: &[TypeNode]) -> bool {
        use TypeNode::*;
        if let (Some(Pointer), Some(Pointer)) = (lhs.get(0), rhs.get(0)) {
            Type::is_compatible2(&lhs[1..], &rhs[1..])
        } else {
            lhs == rhs
        }
    }
}

impl Display for TypeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeClass::Pointer => "a pointer",
                TypeClass::Function => "a function",
                TypeClass::StandardSignedInteger => "a signed integer",
                TypeClass::StandardInteger => "an integer",
                TypeClass::Integer => "an integer",
                TypeClass::Scalar => "a scalar value",
                TypeClass::Arithmetic => "an arithmetic value",
            }
        )
    }
}
