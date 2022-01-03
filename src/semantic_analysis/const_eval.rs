use crate::{
    error,
    eval::evaluation_context::EvaluateSize,
    parser::{ast::*, Type},
};

use super::SemanticAnalyzer;

impl Expression {
    pub fn is_constant(&self) -> bool {
        match self.variant {
            ExpressionVariant::ConstI(_) => true,
            _ => false,
        }
    }

    pub fn get_const_value(&self) -> i128 {
        match self.variant {
            ExpressionVariant::ConstI(val) => val,
            _ => 0,
        }
    }

    pub fn force_const_eval(&mut self, analyzer: &mut SemanticAnalyzer) {
        let expression = std::mem::replace(self, Expression::default(&self.span));
        let expression = expression.const_eval(analyzer);
        if !expression.is_constant() {
            analyzer.errors.push(error!(
                self.span,
                "Initialization of global variable must be constant"
            ));
        }
        *self = expression;
    }

    pub fn const_eval(self, evaluation: &dyn EvaluateSize) -> Expression {
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
                let size = evaluation.eval_sizeof(&typ);
                Expression {
                    span: self.span,
                    ast_type: self.ast_type,
                    variant: ConstI(size as i128),
                }
            }

            Ternary(cond, left, right) => {
                let cond = cond.const_eval(evaluation);
                let left = left.const_eval(evaluation);
                let right = right.const_eval(evaluation);

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
                let left = left.const_eval(evaluation);
                let right = right.const_eval(evaluation);

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

            Cast(exp, typ) => {
                let exp = exp.const_eval(evaluation);

                match &exp.variant {
                    ConstI(exp) => {
                        let size = (evaluation.sizeof(&self.ast_type) * 8) as i128;
                        let value = exp % (1 << size);
                        Expression {
                            span: self.span,
                            ast_type: self.ast_type.clone(),
                            variant: ConstI(value),
                        }
                    }
                    _ => Expression {
                        span: self.span,
                        ast_type: self.ast_type,
                        variant: ExpressionVariant::Cast(Box::new(exp), typ),
                    },
                }
            }

            Unary(op, exp) => {
                let exp = exp.const_eval(evaluation);

                match &exp.variant {
                    ConstI(exp) => Expression {
                        span: self.span,
                        ast_type: self.ast_type.clone(),
                        variant: ConstI(op.const_eval(exp, &self.ast_type)),
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
    fn const_eval(&self, &exp: &i128, ast_type: &Type) -> i128 {
        use UnaryExpressionType::*;
        let _ = ast_type;
        match self {
            Identity => exp,
            Negate => -exp,
            BinNot => !exp,
            LogNot => (exp == 0) as i128,
            Deref | Address => unreachable!(),
        }
    }
}
