use crate::{
    backend::{Backend, TypeInfo},
    parser::{ast::*, Type},
};

impl Expression {
    pub fn is_constant(&self) -> bool {
        match self.variant {
            ExpressionVariant::ConstI(_) => true,
            _ => false,
        }
    }
    pub fn const_eval(self, backend: &dyn Backend, struct_info: &Vec<TypeInfo>) -> Expression {
        use ExpressionVariant::*;
        match self.variant {
            ConstI(_) => self,
            Assign(..)
            | Function(..)
            | Ident(..)
            | CString(..)
            | Member(..)
            | Binary(BinaryExpressionType::Index, ..)
            | Unary(UnaryExpressionType::Deref | UnaryExpressionType::Address, ..) => self,

            Sizeof(typ) => {
                let size = backend.eval_sizeof(&typ, &struct_info);
                Expression {
                    span: self.span,
                    ast_type: self.ast_type,
                    variant: ConstI(size as i128),
                }
            }

            Ternary(cond, left, right) => {
                let cond = cond.const_eval(backend, struct_info);
                let left = left.const_eval(backend, struct_info);
                let right = right.const_eval(backend, struct_info);

                match (&cond.variant, &left.variant, &right.variant) {
                    (ConstI(cond), ConstI(left), ConstI(right)) => Expression {
                        span: self.span,
                        ast_type: self.ast_type,

                        variant: ConstI(if *cond != 0 { *left } else { *right }),
                    },
                    _ => Expression {
                        span: self.span,
                        ast_type: self.ast_type,

                        variant: ExpressionVariant::Ternary(
                            Box::new(cond),
                            Box::new(left),
                            Box::new(right),
                        ),
                    },
                }
            }
            Binary(op, left, right) => {
                let left = left.const_eval(backend, struct_info);
                let right = right.const_eval(backend, struct_info);

                match (&left.variant, &right.variant) {
                    (ConstI(left), ConstI(right)) => Expression {
                        span: self.span,
                        ast_type: self.ast_type,

                        variant: ConstI(op.const_eval(left, right)),
                    },
                    _ => Expression {
                        span: self.span,
                        ast_type: self.ast_type,
                        variant: ExpressionVariant::Binary(op, Box::new(left), Box::new(right)),
                    },
                }
            }
            Unary(op, exp) => {
                let exp = exp.const_eval(backend, struct_info);

                match &exp.variant {
                    ConstI(exp) => Expression {
                        span: self.span,
                        ast_type: self.ast_type.clone(),
                        variant: ConstI(op.const_eval(backend, struct_info, exp, &self.ast_type)),
                    },
                    _ => Expression {
                        span: self.span,
                        ast_type: self.ast_type,
                        variant: ExpressionVariant::Unary(op, Box::new(exp)),
                    },
                }
            }
        }
    }
}

impl BinaryExpressionType {
    fn const_eval(&self, &left: &i128, &right: &i128) -> i128 {
        use BinaryExpressionType::*;
        match self {
            Add => left + right,
            Subtract => left - right,
            Multiply => left * right,
            Equal => (left == right) as i128,
            Inequal => (left != right) as i128,
            Less => (left < right) as i128,
            LessEqual => (left <= right) as i128,
            Greater => (left > right) as i128,
            GreaterEqual => (left >= right) as i128,
            BinOr => left | right,
            BinAnd => left & right,
            LogOr => (left != 0 || right != 0) as i128,
            LogAnd => (left != 0 && right != 0) as i128,
            Comma => right,
            Divide => {
                if right != 0 {
                    left / right
                } else {
                    0
                }
            }
            Index => unreachable!(),
        }
    }
}

impl UnaryExpressionType {
    fn const_eval(
        &self,
        backend: &dyn Backend,
        struct_info: &Vec<TypeInfo>,
        &exp: &i128,
        ast_type: &Type,
    ) -> i128 {
        use UnaryExpressionType::*;
        match self {
            Identity => exp,
            Negate => -exp,
            BinNot => !exp,
            LogNot => (exp == 0) as i128,
            Cast => {
                let size = (backend.sizeof(ast_type, struct_info) * 8) as i128;
                exp % (1 << size)
            }
            Deref | Address => unreachable!(),
        }
    }
}
