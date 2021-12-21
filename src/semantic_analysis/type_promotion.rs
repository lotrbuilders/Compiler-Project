use crate::parser::{Type, TypeNode};

pub trait TypePromotion {
    fn promote(self) -> Type;
}

impl TypePromotion for &Type {
    fn promote(self) -> Type {
        use TypeNode::*;
        match self.nodes[0] {
            Char => Type::int(),
            _ => self.clone(),
        }
    }
}

impl TypePromotion for (Type, Type) {
    fn promote(self) -> Type {
        let (lhs, _rhs) = self;
        lhs
    }
}

impl Type {
    pub fn promote2(self, rhs: Type) -> Type {
        (self, rhs).promote()
    }
}
