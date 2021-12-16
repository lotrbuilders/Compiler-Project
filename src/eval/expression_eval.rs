use super::{Evaluate, EvaluationContext};
use crate::backend::ir::*;
use crate::parser::ast::*;

impl Evaluate for Expression {
    fn eval(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32 {
        use BinaryExpressionType::*;
        use ExpressionVariant::*;
        use UnaryExpressionType::*;
        match &self.variant {
            &ConstI(value) => {
                let vreg = context.next_vreg();
                result.push(IRInstruction::Imm(IRSize::S32, vreg, value));
                vreg
            }

            Ident(..) => {
                let addr = self.eval_lvalue(result, context);
                let vreg = context.next_vreg();
                result.push(IRInstruction::Load(IRSize::S32, vreg, addr));
                vreg
            }

            Function(func, arguments) => {
                let sizes = vec![IRSize::S32; arguments.len()];
                let in_registers = context.backend.get_arguments_in_registers(&sizes);
                let count = arguments.len();
                use crate::backend::Direction;
                let mut first = true;
                let mut arg_index = None;
                match context.backend.argument_evaluation_direction_stack() {
                    Direction::Left2Right => {
                        for arg in 0..arguments.len() {
                            if !in_registers[arg] {
                                let vreg = arguments[arg].eval(result, context);
                                if first {
                                    arg_index = Some(result.len());
                                    result.push(IRInstruction::Arg(
                                        sizes[arg].clone(),
                                        vreg,
                                        Some(0),
                                    ));
                                    first = false;
                                } else {
                                    result.push(IRInstruction::Arg(sizes[arg].clone(), vreg, None));
                                }
                            }
                        }
                    }
                    Direction::Right2Left => {
                        for arg in (0..arguments.len()).rev() {
                            if !in_registers[arg] {
                                let vreg = arguments[arg].eval(result, context);
                                if first {
                                    arg_index = Some(result.len());
                                    result.push(IRInstruction::Arg(
                                        sizes[arg].clone(),
                                        vreg,
                                        Some(0),
                                    ));
                                    first = false;
                                } else {
                                    result.push(IRInstruction::Arg(sizes[arg].clone(), vreg, None));
                                }
                            }
                        }
                    }
                }

                let arguments = match context.backend.argument_evaluation_direction_registers() {
                    Direction::Left2Right => (0..arguments.len())
                        .filter(|&arg| in_registers[arg])
                        .map(|arg| Some(arguments[arg].eval(result, context)))
                        .collect(),
                    Direction::Right2Left => (0..arguments.len())
                        .rev()
                        .filter(|&arg| in_registers[arg])
                        .map(|arg| Some(arguments[arg].eval(result, context)))
                        .collect(),
                };

                let arguments = Box::new(IRArguments {
                    arguments,
                    sizes,
                    count,
                });

                if let Ident(name, ..) = &func.variant {
                    let vreg = context.next_vreg();
                    let index = result.len();
                    if let Some(arg_index) = arg_index {
                        if let IRInstruction::Arg(_, _, Some(fix)) = &mut result[arg_index] {
                            *fix = index;
                        }
                    }
                    result.push(IRInstruction::Call(
                        IRSize::S32,
                        vreg,
                        name.clone(),
                        arguments,
                    ));
                    vreg
                } else {
                    todo!();
                }
            }

            Assign(left, right) => {
                let vreg = right.eval(result, context);
                let addr = left.eval_lvalue(result, context);

                result.push(IRInstruction::Store(IRSize::S32, vreg, addr));
                vreg
            }

            #[allow(unused_variables)]
            Ternary(cond, left, right) => {
                let cond = cond.eval(result, context);

                let (if_index, _) = context.insert_place_holder_jump(result);
                let left = left.eval(result, context);

                let if_label = context.get_current_label();
                let (else_index, _) = context.insert_place_holder_jump(result);

                let right = right.eval(result, context);

                let vreg = context.next_vreg();
                let else_label = context.get_current_label();
                let (last_index, label) = context.insert_place_holder_jump_phi(
                    result,
                    IRPhi::ternary((if_label, else_label), vreg, (left, right)),
                );

                result[if_index] = IRInstruction::Jnc(IRSize::S32, cond, else_label);
                result[else_index] = IRInstruction::Jmp(label);
                result[last_index] = IRInstruction::Jmp(label);
                vreg
            }

            // Could benefit from constants in phi nodes
            Binary(op @ (LogOr | LogAnd), left, right) => {
                let left = left.eval(result, context);
                let first_operand = {
                    let vreg = context.next_vreg();
                    result.push(IRInstruction::Imm(
                        IRSize::S32,
                        vreg,
                        match op {
                            LogOr => 1,
                            LogAnd => 0,
                            _ => unreachable!(),
                        },
                    ));
                    vreg
                };
                let start_label = context.get_current_label();
                let (left_jmp, _) = context.insert_place_holder_jump(result);

                let right = right.eval(result, context);
                let second_operand = {
                    let temp = context.next_vreg();
                    let vreg = context.next_vreg();
                    result.push(IRInstruction::Imm(IRSize::S32, temp, 0));
                    result.push(IRInstruction::Ne(IRSize::S32, vreg, right, temp));
                    vreg
                };
                let vreg = context.next_vreg();
                let left_label = context.get_current_label();
                let (right_jmp, right_label) = context.insert_place_holder_jump_phi(
                    result,
                    IRPhi::ternary(
                        (start_label, left_label),
                        vreg,
                        (first_operand, second_operand),
                    ),
                );

                result[right_jmp] = IRInstruction::Jmp(right_label);
                result[left_jmp] = match op {
                    LogOr => IRInstruction::Jcc(IRSize::S32, left, right_label),
                    LogAnd => IRInstruction::Jnc(IRSize::S32, left, right_label),
                    _ => unreachable!(),
                };

                vreg
            }

            Binary(Comma, left, right) => {
                let _left = left.eval(result, context);
                let right = right.eval(result, context);
                right
            }

            Binary(op, left, right) => {
                let left = left.eval(result, context);
                let right = right.eval(result, context);
                let vreg = context.next_vreg();
                result.push(match op {
                    Add => IRInstruction::Add(IRSize::S32, vreg, left, right),
                    Subtract => IRInstruction::Sub(IRSize::S32, vreg, left, right),
                    Multiply => IRInstruction::Mul(IRSize::S32, vreg, left, right),
                    Divide => IRInstruction::Div(IRSize::S32, vreg, left, right),

                    BinOr => IRInstruction::Or(IRSize::S32, vreg, left, right),
                    BinAnd => IRInstruction::And(IRSize::S32, vreg, left, right),

                    Equal => IRInstruction::Eq(IRSize::S32, vreg, left, right),
                    Inequal => IRInstruction::Ne(IRSize::S32, vreg, left, right),
                    Less => IRInstruction::Lt(IRSize::S32, vreg, left, right),
                    LessEqual => IRInstruction::Le(IRSize::S32, vreg, left, right),
                    Greater => IRInstruction::Gt(IRSize::S32, vreg, left, right),
                    GreaterEqual => IRInstruction::Ge(IRSize::S32, vreg, left, right),
                    _ => unreachable!(),
                });
                vreg
            }

            Unary(Identity, exp) => {
                let exp = exp.eval(result, context);
                exp
            }

            Unary(op, exp) => {
                let left = exp.eval(result, context);
                let right = context.next_vreg();
                let vreg = context.next_vreg();

                result.push(match op {
                    Negate | LogNot => IRInstruction::Imm(IRSize::S32, right, 0),
                    BinNot => IRInstruction::Imm(IRSize::S32, right, -1),
                    _ => unreachable!(),
                });
                result.push(match op {
                    Negate => IRInstruction::Sub(IRSize::S32, vreg, right, left),
                    BinNot => IRInstruction::Xor(IRSize::S32, vreg, left, right),
                    LogNot => IRInstruction::Eq(IRSize::S32, vreg, left, right),
                    _ => unreachable!(),
                });
                vreg
            }
        }
    }
}

impl Expression {
    fn eval_lvalue(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32 {
        use ExpressionVariant::*;
        match &self.variant {
            Ident(_name, symbol_number, false) => {
                let addr = context.next_vreg();
                result.push(IRInstruction::AddrL(
                    IRSize::P,
                    addr,
                    *symbol_number as usize,
                ));
                addr
            }
            Ident(name, _symbol_number, true) => {
                let addr = context.next_vreg();
                result.push(IRInstruction::AddrG(IRSize::P, addr, name.clone()));
                addr
            }
            _ => {
                unreachable!()
            }
        }
    }
}
