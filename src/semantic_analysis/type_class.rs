use std::fmt::Display;

use crate::{
    error,
    parser::{ast::ASTType, Type, TypeNode},
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
    pub fn assert_function_arguments(&mut self, span: &Span, types: &[Type]) {
        if types.len() > 1 && types[0].is_void() {
            self.errors
                .push(error!(span, "Unexpected arguments after a void argumnet"))
        }
        for (i, typ) in types.iter().enumerate() {
            if typ.is_void() && i != 0 {
                self.errors.push(error!(
                    span,
                    "Void argument, which is not in the first place"
                ))
            }
        }
    }

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

    pub fn assert_no_name(&mut self, span: &Span, ast_type: &ASTType) {
        if ast_type.has_name() {
            self.errors
                .push(error!(span, "Variable name not allowed in sizeof/cast"));
        }
    }
}

impl Type {
    pub fn is_in(&self, class: TypeClass) -> bool {
        //use TypeNode::*;
        match self.nodes.get(0) {
            Some(_) => Type::is_in2(&self.nodes, class),
            None => {
                log::error!("Type was improperly passed");
                true
            }
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
            match (lhs.get(1), rhs.get(1)) {
                (Some(Void), Some(_)) | (Some(_), Some(Void)) => true,
                _ => lhs[1..] == rhs[1..],
            }
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
