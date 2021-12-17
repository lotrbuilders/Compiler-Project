use crate::parser::{Type, TypeNode};

pub enum TypeClass {
    StandardSignedInteger,
    StandardInteger,
    Pointer,
    Integer,
    Scalar,
    Arithmetic,
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
            Pointer => self.nodes[0] == TypeNode::Pointer,
            StandardInteger => self.is_in2(StandardSignedInteger),
            Integer => self.is_in2(StandardInteger),
            Arithmetic => self.is_in2(Integer),
            Scalar => self.is_in2(Arithmetic) | self.is_in2(Pointer),
        }
    }
}
