use crate::parser::{Type, TypeNode};
use std::cmp;

pub trait TypePromotion {
    fn promote(self) -> Type;
}

impl TypePromotion for &Type {
    fn promote(self) -> Type {
        use TypeNode::*;
        match self.nodes[0] {
            Char => Type::int(),
            Short => Type::int(),
            _ => self.array_promotion(),
        }
    }
}

impl TypePromotion for (Type, Type) {
    fn promote(self) -> Type {
        use TypeNode::*;
        let (lhs, rhs) = self;
        cmp::max_by_key(lhs, rhs, |typ| match typ.nodes.get(0) {
            Some(Char) => 0,
            Some(Short) => 10,
            Some(Int) => 20,
            Some(Long) => 30,
            _ => i32::MAX,
        })
    }
}

impl Type {
    pub fn promote2(self, rhs: Type) -> Type {
        (self, rhs).promote()
    }
    pub fn array_promotion(&self) -> Type {
        match self.nodes[0] {
            TypeNode::Array(..) => {
                let mut result = self.clone();
                result.nodes[0] = TypeNode::Pointer;
                result
            }
            _ => self.clone(),
        }
    }
}
