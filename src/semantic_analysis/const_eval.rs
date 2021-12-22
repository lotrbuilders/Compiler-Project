use crate::{backend::Backend, parser::ast::*};

impl Expression {
    pub fn is_constant(&self) -> bool {
        match self.variant {
            ExpressionVariant::ConstI(_) => true,
            _ => false,
        }
    }
    pub fn const_eval(self, backend: &dyn Backend) -> Expression {
        use ExpressionVariant::ConstI;
        match self.variant {
            ExpressionVariant::ConstI(_) => self,
            ExpressionVariant::Assign(..)
            | ExpressionVariant::Function(..)
            | ExpressionVariant::Ident(..)
            | ExpressionVariant::CString(..)
            | ExpressionVariant::Binary(BinaryExpressionType::Index, ..)
            | ExpressionVariant::Unary(
                UnaryExpressionType::Deref | UnaryExpressionType::Address,
                ..,
            ) => self,

            ExpressionVariant::Sizeof(typ) => {
                let ast_type = match typ {
                    SizeofType::Type(typ) => typ,
                    SizeofType::Expression(exp) => exp.ast_type,
                };
                let size = backend.sizeof(ast_type);

                Expression {
                    span: self.span,
                    ast_type: self.ast_type,
                    variant: ConstI(size as i128),
                }
            }

            ExpressionVariant::Ternary(cond, left, right) => {
                let cond = cond.const_eval(backend);
                let left = left.const_eval(backend);
                let right = right.const_eval(backend);

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
            ExpressionVariant::Binary(op, left, right) => {
                let left = left.const_eval(backend);
                let right = right.const_eval(backend);

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
            ExpressionVariant::Unary(op, exp) => {
                let exp = exp.const_eval(backend);

                match &exp.variant {
                    ConstI(exp) => Expression {
                        span: self.span,
                        ast_type: self.ast_type,

                        variant: ConstI(op.const_eval(exp)),
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
            Divide => {
                if right != 0 {
                    left / right
                } else {
                    0
                }
            }
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
            Index => unreachable!(),
        }
    }
}

impl UnaryExpressionType {
    fn const_eval(&self, &exp: &i128) -> i128 {
        use UnaryExpressionType::*;
        match self {
            Identity => exp,
            Negate => -exp,
            BinNot => !exp,
            LogNot => (exp == 0) as i128,
            Cast => todo!(),
            Deref | Address => unreachable!(),
        }
    }
}
