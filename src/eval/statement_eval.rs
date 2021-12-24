use super::{Evaluate, EvaluationContext};
use crate::backend::ir::*;
use crate::parser::ast::*;

impl Evaluate for Statement {
    fn eval(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32 {
        use Statement::*;
        match self {
            Break { .. } => {
                let (index, _) = context.insert_place_holder_jump(result);
                context.add_break(index);
            }

            Continue { .. } => {
                let (index, _) = context.insert_place_holder_jump(result);
                context.add_continue(index);
            }

            Compound {
                span: _,
                statements,
            } => {
                for stmt in statements {
                    stmt.eval(result, context);
                }
            }

            Declaration {
                span: _,
                ident: _,
                decl_type,
                ast_type: _,
                init,
            } => {
                let index = context.variables.len();
                let (array_type, array_count) = decl_type.deconstruct();
                let size = context
                    .backend
                    .get_size_struct(&array_type, &context.struct_size_table);
                let variable = IRVariable {
                    size,
                    count: array_count,
                };
                context.variables.push(variable);
                if let Some(exp) = init {
                    let vreg = exp.eval(result, context);
                    let addr = context.next_vreg();
                    result.push(IRInstruction::AddrL(IRSize::P, addr, index));
                    result.push(IRInstruction::Store(size, vreg, addr));
                }
            }

            Empty(_) => (),

            Expression {
                span: _,
                expression,
            } => {
                expression.eval(result, context);
            }

            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                let cond = expression.eval(result, context);
                let size = context.get_size(&expression.ast_type);

                let previous_label = context.get_current_label();
                let (index, if_label) = context.insert_place_holder_jump(result);
                statement.eval(result, context);

                if let Some(statement) = else_statement {
                    let (else_index, else_label) = context.insert_place_holder_jump(result);

                    statement.eval(result, context);

                    let (last_index, label) = context.insert_place_holder_jump_phi(
                        result,
                        IRPhi::empty(vec![if_label, else_label]),
                    );
                    result[index] = IRInstruction::Jnc(size, cond, else_label);
                    result[else_index] = IRInstruction::Jmp(label);
                    result[last_index] = IRInstruction::Jmp(label);
                } else {
                    let (last_index, label) = context.insert_place_holder_jump_phi(
                        result,
                        IRPhi::empty(vec![previous_label, if_label]),
                    );
                    result[index] = IRInstruction::Jnc(size, cond, label);
                    result[last_index] = IRInstruction::Jmp(label);
                }
            }

            For {
                span: _,
                init,
                condition,
                expression,
                statement,
            } => {
                let size = condition
                    .as_ref()
                    .map(|exp| context.get_size(&exp.ast_type))
                    .unwrap_or(IRSize::S32);

                context.enter_loop();
                if let Some(init) = init {
                    init.eval(result, context);
                }

                let (jmp_index, loop_label) = context.insert_place_holder_jump(result);

                statement.eval(result, context);
                let continue_label = context.insert_fall_through(result);

                expression.as_ref().map(|exp| exp.eval(result, context));

                let check_label = context.insert_fall_through(result);
                let comparison = condition
                    .as_ref()
                    .map(|cond| cond.eval(result, context))
                    .unwrap_or(0);

                let (last_index, label_after) = context.insert_place_holder_jump(result);

                context.fix_jumps(result, label_after, continue_label);
                result[jmp_index] = IRInstruction::Jmp(check_label);
                result[last_index] = match condition {
                    Some(_) => IRInstruction::Jcc(size, comparison, loop_label),
                    None => IRInstruction::Jmp(loop_label),
                };
            }

            Return {
                span: _,
                ast_type,
                expression,
            } => {
                if let Some(exp) = expression {
                    let size = context.get_size(&ast_type);
                    let exp_size = context.get_size(&exp.ast_type);
                    let vreg = exp.eval(result, context);
                    let vreg = context.promote(result, size, exp_size, vreg);
                    result.push(IRInstruction::Ret(size, vreg))
                } else {
                    use crate::parser::Type;
                    // Creates a temporary that isn't used.
                    let vreg = context.next_vreg();
                    result.push(IRInstruction::Imm(context.get_size(&Type::int()), vreg, 0));
                    result.push(IRInstruction::Ret(IRSize::V, vreg))
                }
            }

            // The check is done last, therefore an extra jump is inserted at the front
            // In most cases this should lead to a speedup as most loops are entered
            While {
                span: _,
                expression,
                statement,
                do_while,
            } => {
                let size = context.get_size(&expression.ast_type);
                context.enter_loop();
                let (jmp_index, loop_label) = context.insert_place_holder_jump(result);

                statement.eval(result, context);
                let check_label = context.insert_fall_through(result);

                let expression = expression.eval(result, context);
                let (last_index, label_after) = context.insert_place_holder_jump(result);

                context.fix_jumps(result, label_after, check_label);
                result[last_index] = IRInstruction::Jcc(size, expression, loop_label);
                result[jmp_index] = IRInstruction::Jmp(match do_while {
                    true => loop_label,
                    false => check_label,
                });
            }
        }
        0
    }
}
