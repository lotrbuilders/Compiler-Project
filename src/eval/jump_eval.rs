use std::ops::Not;

use smallvec::SmallVec;

use crate::backend::ir::ir::IRSize;
use crate::eval::evaluation_context::EvaluateSize;
use crate::eval::Evaluate;
use crate::parser::ast::{BinaryExpressionType, UnaryExpressionType};
use crate::semantic_analysis::type_promotion::TypePromotion;
use crate::{backend::ir::ir::IRInstruction, parser::ast::Expression};

use super::evaluation_context::EvaluationContext;
use super::ExpressionVariant;

#[derive(Debug, Clone)]
pub struct JumpRecord {
    pub jump_if: SmallVec<[(usize, u32, JumpType); 1]>,
    pub jump_else: SmallVec<[(usize, u32, JumpType); 1]>,
}

#[derive(Debug, Clone, Copy)]
pub enum JumpDestination {
    If,
    Else,
}

impl Not for JumpDestination {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            JumpDestination::Else => JumpDestination::If,
            JumpDestination::If => JumpDestination::Else,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JumpType {
    Jnc,
    Jcc,
}

impl Not for JumpType {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            JumpType::Jnc => JumpType::Jcc,
            JumpType::Jcc => JumpType::Jnc,
        }
    }
}

impl Expression {
    pub fn condition_eval(
        &self,
        result: &mut Vec<IRInstruction>,
        context: &mut EvaluationContext,
        jump_type: JumpType,
    ) -> SmallVec<[(usize, u32, IRSize, JumpType); 1]> {
        let mut list = SmallVec::new();
        self.cond_eval(result, context, &mut list, jump_type);
        list
    }

    fn cond_eval(
        &self,
        result: &mut Vec<IRInstruction>,
        context: &mut EvaluationContext,
        list: &mut SmallVec<[(usize, u32, IRSize, JumpType); 1]>,
        jump_type: JumpType,
    ) {
        use BinaryExpressionType::*;
        use ExpressionVariant::*;
        match (&self.variant, jump_type) {
            (Unary(UnaryExpressionType::LogNot, exp), _) => {
                exp.cond_eval(result, context, list, !jump_type);
            }

            (Binary(LogAnd, left, right), JumpType::Jnc)
            | (Binary(LogOr, left, right), JumpType::Jcc) => {
                left.cond_eval(result, context, list, jump_type);
                right.cond_eval(result, context, list, jump_type);
            }

            (Binary(LogOr, left, right), JumpType::Jnc)
            | (Binary(LogAnd, left, right), JumpType::Jcc) => {
                let mut left_list = SmallVec::new();
                left.cond_eval(result, context, &mut left_list, !jump_type);
                right.cond_eval(result, context, list, jump_type);

                let label = context.get_current_label();
                context.fix_jumps(result, &left_list, label);
            }

            (
                Binary(
                    op @ (Equal | Inequal | Less | LessEqual | Greater | GreaterEqual),
                    left,
                    right,
                ),
                _,
            ) => {
                // Duplication of normal code, might not be wanted
                let size = context
                    .get_size(&(left.ast_type.promote(), right.ast_type.promote()).promote());
                let left_size = context.get_size(&left.ast_type);
                let right_size = context.get_size(&right.ast_type);

                let left = left.eval(result, context);
                let right = right.eval(result, context);
                let left = context.promote(result, size, left_size, left);
                let right = context.promote(result, size, right_size, right);
                let vreg = context.next_vreg();

                result.push(match (op, jump_type) {
                    (Equal, JumpType::Jcc) | (Inequal, JumpType::Jnc) => {
                        IRInstruction::Eq(size, vreg, left, right)
                    }
                    (Inequal, JumpType::Jcc) | (Equal, JumpType::Jnc) => {
                        IRInstruction::Ne(size, vreg, left, right)
                    }
                    (Less, JumpType::Jcc) | (GreaterEqual, JumpType::Jnc) => {
                        IRInstruction::Lt(size, vreg, left, right)
                    }
                    (LessEqual, JumpType::Jcc) | (Greater, JumpType::Jnc) => {
                        IRInstruction::Le(size, vreg, left, right)
                    }
                    (Greater, JumpType::Jcc) | (LessEqual, JumpType::Jnc) => {
                        IRInstruction::Gt(size, vreg, left, right)
                    }
                    (GreaterEqual, JumpType::Jcc) | (Less, JumpType::Jnc) => {
                        IRInstruction::Ge(size, vreg, left, right)
                    }
                    _ => unreachable!(),
                });

                let (index, _) = context.insert_place_holder_jump(result);
                list.push((index, vreg, context.type_info.int.irsize, JumpType::Jcc));
            }

            _ => {
                let vreg = self.eval(result, context);
                let from = context.get_size(&self.ast_type);
                let size = context.get_size(&self.ast_type.promote());
                let cond = context.promote(result, size, from, vreg);
                let (index, _label) = context.insert_place_holder_jump(result);
                list.push((index, cond, size, jump_type));
            }
        }
    }
}
