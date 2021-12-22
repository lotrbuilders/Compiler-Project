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

impl<'a> SemanticAnalyzer<'a> {
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
            Some(_) => Type::is_in2(&self.nodes, class),
        }
    }

    fn is_in2(typ: &[TypeNode], class: TypeClass) -> bool {
        use TypeClass::*;
        match class {
            StandardSignedInteger => {
                matches!(typ[0], TypeNode::Int | TypeNode::Long | TypeNode::Short)
            }
            Function => matches!(typ[0], TypeNode::Function(_)),
            Pointer => matches!(typ[0], TypeNode::Pointer),
            StandardInteger => Type::is_in2(typ, StandardSignedInteger),
            Integer => Type::is_in2(typ, StandardInteger) | matches!(typ[0], TypeNode::Char),
            Arithmetic => Type::is_in2(typ, Integer),
            Scalar => Type::is_in2(typ, Arithmetic) | Type::is_in2(typ, Pointer),
        }
    }

    pub fn is_compatible(&self, other: &Type) -> bool {
        Type::is_compatible2(&self.nodes, &other.nodes)
    }

    fn is_compatible2(lhs: &[TypeNode], rhs: &[TypeNode]) -> bool {
        use TypeNode::*;
        if lhs.get(0).is_none() || rhs.get(0).is_none() {
            true
        } else if let (Some(Pointer), Some(Pointer)) = (lhs.get(0), rhs.get(0)) {
            lhs[1..] == rhs[1..]
        } else if Type::is_in2(lhs, TypeClass::Arithmetic)
            && Type::is_in2(rhs, TypeClass::Arithmetic)
        {
            true
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
