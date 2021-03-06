use super::{Evaluate, EvaluationContext};
use crate::eval::evaluation_context::EvaluateSize;
use crate::eval::jump_eval::JumpType;
use crate::ir::*;
use crate::parser::{ast::*, Type};
use crate::semantic_analysis::type_promotion::TypePromotion;

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

            CString(value) => {
                let addr = context.next_vreg();
                let number = context.add_string(value);
                result.push(IRInstruction::AddrG(
                    IRSize::P,
                    addr,
                    format!(".__string{}", number),
                ));
                addr
            }

            Ident(..) => {
                let addr = self.eval_lvalue(result, context);
                self.optional_load(result, context, addr)
            }

            Sizeof(typ) => {
                let size = context.eval_sizeof(typ) as i128;
                let size_t = context.get_size(&context.size_t());
                let vreg = context.next_vreg();
                result.push(IRInstruction::Imm(size_t, vreg, size));
                vreg
            }

            Function(func, arguments) => {
                let size = context.get_size(&self.ast_type);
                let count = arguments.len();
                let empty = vec![Type::empty(); count];
                let sizes = func.ast_type.get_function_arguments().unwrap_or(&empty);
                let argument_sizes: Vec<IRSize> = arguments
                    .iter()
                    .map(|e| context.get_size(&e.ast_type.array_promotion()))
                    .collect();

                let sizes = if !sizes.is_empty() {
                    sizes
                        .iter()
                        .filter(|&t| !t.is_void())
                        .map(|t| context.get_size(&t.clone().remove_name().array_promotion()))
                        .collect()
                } else {
                    argument_sizes.clone()
                };

                log::debug!(
                    "Evaluating call {} with arguments {:?}",
                    func.ast_type,
                    sizes
                );
                let in_registers = context.backend.get_arguments_in_registers(&sizes);
                let count = arguments.len();
                let mut first = true;
                let mut arg_index = None;
                use crate::backend::Direction;
                match context.backend.argument_evaluation_direction_stack() {
                    Direction::Left2Right => {
                        for arg in 0..arguments.len() {
                            if !in_registers[arg] {
                                let arg_size = argument_sizes[arg];
                                let vreg = arguments[arg].eval(result, context);
                                let vreg = context.promote(result, sizes[arg], arg_size, vreg);

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
                                let arg_size = argument_sizes[arg];
                                let vreg = arguments[arg].eval(result, context);
                                let vreg = context.promote(result, sizes[arg], arg_size, vreg);
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
                        .map(|arg| {
                            Some({
                                let arg_size = argument_sizes[arg];
                                let vreg = arguments[arg].eval(result, context);
                                context.promote(result, sizes[arg], arg_size, vreg)
                            })
                        })
                        .collect(),
                    Direction::Right2Left => (0..arguments.len())
                        .rev()
                        .filter(|&arg| in_registers[arg])
                        .map(|arg| {
                            Some({
                                let arg_size = argument_sizes[arg];
                                let vreg = arguments[arg].eval(result, context);
                                context.promote(result, sizes[arg], arg_size, vreg)
                            })
                        })
                        .collect(),
                };

                let arguments = Box::new(IRArguments {
                    variables: Vec::new(), //Not used in expressions, only in globals
                    arguments,
                    sizes,
                    count,
                });

                if let (Ident(name, ..), true) =
                    (&func.variant, !func.ast_type.is_function_pointer())
                {
                    let vreg = context.next_vreg();
                    let index = result.len();
                    if let Some(arg_index) = arg_index {
                        if let IRInstruction::Arg(_, _, Some(fix)) = &mut result[arg_index] {
                            *fix = index;
                        }
                    }
                    result.push(IRInstruction::Call(size, vreg, name.clone(), arguments));
                    vreg
                } else {
                    // Function pointers are not yet supported
                    let addr = func.eval(result, context);
                    let vreg = context.next_vreg();
                    let index = result.len();
                    if let Some(arg_index) = arg_index {
                        if let IRInstruction::Arg(_, _, Some(fix)) = &mut result[arg_index] {
                            *fix = index;
                        }
                    }
                    result.push(IRInstruction::CallV(size, vreg, addr, arguments));
                    vreg
                }
            }

            Member(..) => {
                let addr = self.eval_lvalue(result, context);
                self.optional_load(result, context, addr)
            }

            Assign(left, right) => {
                let size = context.get_size(&self.ast_type);
                let right_size = context.get_size(&right.ast_type.array_promotion());
                let vreg = right.eval(result, context);
                let vreg = context.promote(result, size, right_size, vreg);
                let addr = left.eval_lvalue(result, context);

                result.push(IRInstruction::Store(size, vreg, addr));
                vreg
            }

            #[allow(unused_variables)]
            Ternary(cond, left, right) => {
                //let cond_size = context.get_size(&cond.ast_type);
                let size = context.get_size(&self.ast_type);
                let left_size = context.get_size(&left.ast_type);
                let right_size = context.get_size(&right.ast_type);

                //let cond = cond.eval(result, context);
                let cond = cond.condition_eval(result, context, JumpType::Jnc);

                //let (if_index, _) = context.insert_place_holder_jump(result);
                let left = left.eval(result, context);
                let left = context.promote(result, size, left_size, left);

                let if_label = context.get_current_label();
                let (else_index, _) = context.insert_place_holder_jump(result);

                let right = right.eval(result, context);
                let right = context.promote(result, size, right_size, right);

                let vreg = context.next_vreg();
                let else_label = context.get_current_label();
                let (last_index, label) = context.insert_place_holder_jump_phi(
                    result,
                    IRPhi::ternary(size, (if_label, else_label), vreg, (left, right)),
                );

                //result[if_index] = IRInstruction::Jnc(cond_size, cond, else_label);
                context.fix_jumps(result, &cond, else_label);
                result[else_index] = IRInstruction::Jmp(label);
                result[last_index] = IRInstruction::Jmp(label);
                vreg
            }

            // Could benefit from constants in phi nodes
            Binary(op @ (LogOr | LogAnd), left, right) => {
                let left_size = context.get_size(&left.ast_type);
                let right_size = context.get_size(&right.ast_type);
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
                    result.push(IRInstruction::Imm(right_size, temp, 0));
                    result.push(IRInstruction::Ne(right_size, vreg, right, temp));
                    vreg
                };
                let vreg = context.next_vreg();
                let left_label = context.get_current_label();
                let (right_jmp, right_label) = context.insert_place_holder_jump_phi(
                    result,
                    IRPhi::ternary(
                        IRSize::S32,
                        (start_label, left_label),
                        vreg,
                        (first_operand, second_operand),
                    ),
                );

                result[right_jmp] = IRInstruction::Jmp(right_label);
                result[left_jmp] = match op {
                    LogOr => IRInstruction::Jcc(left_size, left, right_label),
                    LogAnd => IRInstruction::Jnc(left_size, left, right_label),
                    _ => unreachable!(),
                };
                vreg
            }

            Binary(Subtract, left, right)
                if (left.ast_type.is_pointer() || left.ast_type.is_array())
                    && (right.ast_type.is_pointer() || left.ast_type.is_array()) =>
            {
                let int_ptr_size = context.int_ptr(true);
                let left_vreg = left.eval(result, context);
                let right = right.eval(result, context);

                let vreg = context.next_vreg();
                result.push(IRInstruction::Sub(IRSize::P, vreg, left_vreg, right));

                // IF we are subtracting two pointers we need to devide their distance by the sizeof the pointed-to object
                let size = context.sizeof(&left.ast_type.clone().deref());
                if size != 1 {
                    let constant = context.next_vreg();
                    let right = vreg;
                    let vreg = context.next_vreg();
                    result.push(IRInstruction::Imm(int_ptr_size, constant, size as i128));
                    result.push(IRInstruction::Div(int_ptr_size, vreg, right, constant));
                    vreg
                } else {
                    vreg
                }
            }

            Binary(op @ (Subtract | Add | Index), left, right)
                if left.ast_type.is_pointer()
                    || right.ast_type.is_pointer()
                    || left.ast_type.is_array()
                    || right.ast_type.is_array() =>
            {
                let (left_vreg, right) = self.eval_pointer_addition(result, context);
                let vreg = context.next_vreg();
                match op {
                    Add => {
                        result.push(IRInstruction::Add(IRSize::P, vreg, left_vreg, right));
                        vreg
                    }
                    Subtract => {
                        result.push(IRInstruction::Sub(IRSize::P, vreg, left_vreg, right));
                        vreg
                    }
                    Index => {
                        let addr = vreg;
                        result.push(IRInstruction::Add(IRSize::P, addr, left_vreg, right));
                        self.optional_load(result, context, addr)
                    }
                    _ => unreachable!(),
                }
            }

            Binary(Comma, left, right) => {
                let _left = left.eval(result, context);
                let right = right.eval(result, context);
                right
            }

            Binary(
                op @ (Equal | Inequal | Less | LessEqual | Greater | GreaterEqual),
                left,
                right,
            ) => {
                let size = context
                    .get_size(&(left.ast_type.promote(), right.ast_type.promote()).promote());
                let left_size = context.get_size(&left.ast_type);
                let right_size = context.get_size(&right.ast_type);

                let left = left.eval(result, context);
                let right = right.eval(result, context);
                let left = context.promote(result, size, left_size, left);
                let right = context.promote(result, size, right_size, right);
                let vreg = context.next_vreg();

                result.push(match op {
                    Equal => IRInstruction::Eq(size, vreg, left, right),
                    Inequal => IRInstruction::Ne(size, vreg, left, right),
                    Less => IRInstruction::Lt(size, vreg, left, right),
                    LessEqual => IRInstruction::Le(size, vreg, left, right),
                    Greater => IRInstruction::Gt(size, vreg, left, right),
                    GreaterEqual => IRInstruction::Ge(size, vreg, left, right),
                    _ => unreachable!(),
                });
                vreg
            }

            // TODO add IRInstruction for adding number to pointer?
            // Or add conversion from integer in int* +/- int
            // Adding/subtracting pointer does not currently lead to correct behaviour
            Binary(op, left, right) => {
                let left_type = &left.ast_type;
                let right_type = &right.ast_type;
                let size = context.get_size(&self.ast_type);

                let left = left.eval(result, context);
                let right = right.eval(result, context);

                let left_size = context.get_size(&left_type); //
                let right_size = context.get_size(&right_type);

                let left = context.promote(result, size, left_size, left);
                let right = context.promote(result, size, right_size, right);
                let vreg = context.next_vreg();

                result.push(match op {
                    Add => IRInstruction::Add(size, vreg, left, right),
                    Subtract => IRInstruction::Sub(size, vreg, left, right),
                    Multiply => IRInstruction::Mul(size, vreg, left, right),
                    Divide => IRInstruction::Div(size, vreg, left, right),

                    BinOr => IRInstruction::Or(size, vreg, left, right),
                    BinAnd => IRInstruction::And(size, vreg, left, right),

                    Comma | LogOr | LogAnd | Equal | Inequal | Less | LessEqual | Greater
                    | GreaterEqual | Index => unreachable!(),
                });
                vreg
            }

            Cast(exp, _) => {
                let exp_size = context.get_size(&exp.ast_type);
                let size = context.get_size(&self.ast_type);
                let vreg = exp.eval(result, context);
                context.promote(result, size, exp_size, vreg)
            }

            Unary(Identity, exp) => exp.eval(result, context),
            Unary(Address, exp) => exp.eval_lvalue(result, context),
            Unary(Deref, _exp) => {
                let addr = self.eval_lvalue(result, context);
                self.optional_load(result, context, addr)
            }

            Unary(LogNot, exp) => {
                let size = context.get_size(&exp.ast_type);
                let left = exp.eval(result, context);
                let right = context.next_vreg();
                let vreg = context.next_vreg();
                result.push(IRInstruction::Imm(size, right, 0));
                result.push(IRInstruction::Eq(size, vreg, left, right));
                vreg
            }

            Unary(op, exp) => {
                let size = context.get_size(&self.ast_type);
                let exp_size = context.get_size(&exp.ast_type);

                let left = exp.eval(result, context);
                let left = context.promote(result, size, exp_size, left);
                let right = context.next_vreg();
                let vreg = context.next_vreg();

                match op {
                    Negate => result.push(IRInstruction::Imm(size, right, 0)),
                    BinNot => result.push(IRInstruction::Imm(size, right, -1)),
                    _ => (),
                }
                result.push(match op {
                    Negate => IRInstruction::Sub(size, vreg, right, left),
                    BinNot => IRInstruction::Xor(size, vreg, left, right),
                    LogNot | Identity | Address | Deref => unreachable!(),
                });
                vreg
            }
        }
    }
}

impl Expression {
    fn eval_lvalue(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32 {
        use BinaryExpressionType::*;
        use ExpressionVariant::*;
        use UnaryExpressionType::*;
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
            Unary(Deref, exp) => exp.eval(result, context),
            Binary(Index, ..) => {
                let (left, right) = self.eval_pointer_addition(result, context);
                let addr = context.next_vreg();
                result.push(IRInstruction::Add(IRSize::P, addr, left, right));
                addr
            }
            Member(exp, _, indirect, index) => {
                let left = exp.eval_lvalue(result, context);
                let left = if *indirect {
                    let vreg = context.next_vreg();
                    result.push(IRInstruction::Load(IRSize::P, vreg, left));
                    vreg
                } else {
                    left
                };

                let struct_index = exp.ast_type.get_struct_index();
                let offset = context.struct_offset_table[struct_index][*index as usize];

                let right = context.next_vreg();
                let addr = context.next_vreg();
                result.push(IRInstruction::Imm(IRSize::P, right, offset as i128));
                result.push(IRInstruction::Add(IRSize::P, addr, left, right));
                addr
            }

            _ => {
                unreachable!()
            }
        }
    }
}

impl Expression {
    fn optional_load(
        &self,
        result: &mut Vec<IRInstruction>,
        context: &mut EvaluationContext,
        addr: u32,
    ) -> u32 {
        if !self.ast_type.is_array() && !self.ast_type.is_function() {
            let size = context.get_size(&self.ast_type);
            let vreg = context.next_vreg();
            result.push(IRInstruction::Load(size, vreg, addr));
            vreg
        } else {
            addr
        }
    }

    fn eval_pointer_addition(
        &self,
        result: &mut Vec<IRInstruction>,
        context: &mut EvaluationContext,
    ) -> (u32, u32) {
        let (_, left, right) = match &self.variant {
            ExpressionVariant::Binary(op, left, right) => (op, left, right),
            _ => unreachable!(),
        };
        // Swap pointers such that left is always a pointer
        // This also ensures that matching only needs to consider left side
        let (left, right) = if right.ast_type.is_pointer() {
            (right, left)
        } else {
            (left, right)
        };

        let right_size = context.get_size(&right.ast_type);
        let int_ptr_size = context.int_ptr(true);
        let left_vreg = left.eval(result, context);
        let mut right = right.eval(result, context);

        //Conversions are only inserted if sizeof(right) != sizeof(pointer)
        //This means that on ILP32 and IP16 environments convert is often not inserted
        if right_size != IRSize::P && right_size != int_ptr_size {
            let vreg = context.next_vreg();
            result.push(IRInstruction::Cvs(
                context.int_ptr(true),
                vreg,
                right_size,
                right,
            ));
            right = vreg
        }

        // We must multiply the right side with sizeof(*left)
        // The constant will always be added on the right side
        let size = context.sizeof(&left.ast_type.clone().deref());
        if size != 1 {
            let constant = context.next_vreg();
            let vreg = context.next_vreg();
            result.push(IRInstruction::Imm(int_ptr_size, constant, size as i128));
            result.push(IRInstruction::Mul(int_ptr_size, vreg, right, constant));
            right = vreg
        }
        (left_vreg, right)
    }
}
