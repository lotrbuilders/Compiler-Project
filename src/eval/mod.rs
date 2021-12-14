use std::collections::{HashMap, HashSet};

use crate::backend::ir::*;
use crate::parser::ast::*;
use crate::parser::r#type::DeclarationType;
use crate::semantic_analysis::symbol_table::Symbol;

struct EvaluationContext {
    vreg_counter: u32,
    label_counter: u32,
    variables: Vec<IRSize>,
    unfixed_continue: Vec<(usize, u32)>,
    unfixed_break: Vec<(usize, u32)>,
    loop_depth: u32,
}

impl EvaluationContext {
    fn next_vreg(&mut self) -> u32 {
        let vreg = self.vreg_counter;
        self.vreg_counter += 1;
        vreg
    }
    fn next_label(&mut self) -> u32 {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

impl EvaluationContext {
    fn insert_place_holder_jump(&mut self, result: &mut Vec<IRInstruction>) -> (usize, u32) {
        let index = result.len();
        result.push(IRInstruction::Jmp(0));

        let label = self.insert_label(result);

        (index, label)
    }

    fn insert_place_holder_jump_phi(
        &mut self,
        result: &mut Vec<IRInstruction>,
        phi: Box<IRPhi>,
    ) -> (usize, u32) {
        let index = result.len();
        result.push(IRInstruction::Jmp(0));

        let label = self.insert_phi_label(result, phi);

        (index, label)
    }

    fn insert_fall_through(&mut self, result: &mut Vec<IRInstruction>) -> u32 {
        let label = self.label_counter;
        result.push(IRInstruction::Jmp(label));
        self.insert_label(result)
    }

    fn insert_label(&mut self, result: &mut Vec<IRInstruction>) -> u32 {
        let label = self.next_label();
        result.push(IRInstruction::Label(None, label));

        label
    }

    fn insert_phi_label(&mut self, result: &mut Vec<IRInstruction>, phi: Box<IRPhi>) -> u32 {
        let label = self.next_label();
        result.push(IRInstruction::Label(Some(phi), label));

        label
    }

    fn get_current_label(&self) -> u32 {
        let label = self.label_counter - 1;
        label
    }
}

impl EvaluationContext {
    fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    fn add_break(&mut self, index: usize) {
        self.unfixed_break.push((index, self.loop_depth))
    }

    fn add_continue(&mut self, index: usize) {
        self.unfixed_continue.push((index, self.loop_depth))
    }

    fn fix_jumps(&mut self, result: &mut Vec<IRInstruction>, break_label: u32, coninue_label: u32) {
        self.unfixed_break = self
            .unfixed_break
            .iter()
            .filter_map(|(i, depth)| {
                if *depth == self.loop_depth {
                    result[*i] = IRInstruction::Jmp(break_label);
                    None
                } else {
                    Some((*i, *depth))
                }
            })
            .collect();

        self.unfixed_continue = self
            .unfixed_continue
            .iter()
            .filter_map(|(i, depth)| {
                if *depth == self.loop_depth {
                    result[*i] = IRInstruction::Jmp(coninue_label);
                    None
                } else {
                    Some((*i, *depth))
                }
            })
            .collect();

        self.loop_depth -= 1;
    }
}

// This module is used to evaluate the AST into an IR

// The public function used to evaluate the ast
pub fn evaluate(
    ast: &TranslationUnit,
    map: &HashMap<String, Symbol>,
) -> (Vec<IRFunction>, Vec<IRGlobal>) {
    let mut functions = Vec::<IRFunction>::new();
    for global in &ast.global_declarations {
        if let Some(declaration) = global.eval() {
            functions.push(declaration);
        }
    }

    let mut globals = Vec::new();
    let mut defined = HashSet::new();
    for global in &ast.global_declarations {
        log::trace!("Evaluating individual global");
        if let Some(declaration) = global.eval_global(map, &mut defined) {
            globals.push(declaration);
        }
    }
    (functions, Vec::new())
}

impl ExternalDeclaration {
    fn eval(&self) -> Option<IRFunction> {
        match &self.function_body {
            Some(statements) => {
                let mut instructions = Vec::<IRInstruction>::new();
                let mut context = EvaluationContext {
                    vreg_counter: 0,
                    label_counter: 1,
                    variables: Vec::new(),
                    unfixed_break: Vec::new(),
                    unfixed_continue: Vec::new(),
                    loop_depth: 0,
                };

                instructions.push(IRInstruction::Label(None, 0));
                for statement in statements {
                    statement.eval(&mut instructions, &mut context);
                }
                Some(IRFunction {
                    name: self.name.clone(),
                    return_size: IRSize::S32,
                    instructions,
                    variables: context.variables,
                })
            }
            None => {
                log::info!("Empty function body");
                None
            }
        }
    }
    fn eval_global(
        &self,
        map: &HashMap<String, Symbol>,
        defined: &mut HashSet<String>,
    ) -> Option<IRGlobal> {
        if defined.contains(&self.name) {
            None
        } else if let Some(_) = self.function_body {
            defined.insert(self.name.clone());
            None
        } else if self.ast_type.is_function() {
            defined.insert(self.name.clone());
            if map[&self.name].declaration_type == DeclarationType::Definition {
                None
            } else {
                Some(IRGlobal {
                    name: self.name.clone(),
                    size: IRSize::S32,
                    value: None,
                    function: true,
                })
            }
        } else {
            defined.insert(self.name.clone());
            if map[&self.name].declaration_type == DeclarationType::Definition {
                None
            } else {
                Some(IRGlobal {
                    name: self.name.clone(),
                    size: IRSize::S32,
                    value: None,
                    function: false,
                })
            }
        }
    }
}

// The trait Evaluate is used by statements and expressions
// The vreg counter should be updated every use
// The function returns the virtual register representing its result
trait Evaluate {
    fn eval(&self, result: &mut Vec<IRInstruction>, context: &mut EvaluationContext) -> u32;
}

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
                decl_type: _,
                init,
            } => {
                let index = context.variables.len();
                context.variables.push(IRSize::S32); //Should be determined by type of declaration later
                if let Some(exp) = init {
                    let vreg = exp.eval(result, context);
                    let addr = context.next_vreg();
                    result.push(IRInstruction::AddrL(IRSize::P, addr, index));
                    result.push(IRInstruction::Store(IRSize::S32, vreg, addr));
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

                let previous_label = context.get_current_label();
                let (index, if_label) = context.insert_place_holder_jump(result);
                statement.eval(result, context);

                if let Some(statement) = else_statement {
                    let (else_index, else_label) = context.insert_place_holder_jump(result);
                    result[index] = IRInstruction::Jnc(IRSize::S32, cond, else_label);

                    statement.eval(result, context);

                    let (last_index, label) = context.insert_place_holder_jump_phi(
                        result,
                        IRPhi::empty(vec![if_label, else_label]),
                    );
                    result[else_index] = IRInstruction::Jmp(label);
                    result[last_index] = IRInstruction::Jmp(label);
                } else {
                    let (last_index, label) = context.insert_place_holder_jump_phi(
                        result,
                        IRPhi::empty(vec![previous_label, if_label]),
                    );
                    result[index] = IRInstruction::Jnc(IRSize::S32, cond, label);
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
                    Some(_) => IRInstruction::Jcc(IRSize::S32, comparison, loop_label),
                    None => IRInstruction::Jmp(loop_label),
                };
            }

            Return {
                span: _,
                expression,
            } => {
                let vreg = expression.eval(result, context);
                result.push(IRInstruction::Ret(IRSize::S32, vreg))
            }

            // The check is done last, therefore an extra jump is inserted at the front
            // In most cases this should lead to a speedup as most loops are entered
            While {
                span: _,
                expression,
                statement,
                do_while,
            } => {
                context.enter_loop();
                let (jmp_index, loop_label) = context.insert_place_holder_jump(result);

                statement.eval(result, context);
                let check_label = context.insert_fall_through(result);

                let expression = expression.eval(result, context);
                let (last_index, label_after) = context.insert_place_holder_jump(result);

                context.fix_jumps(result, label_after, check_label);
                result[last_index] = IRInstruction::Jcc(IRSize::S32, expression, loop_label);
                result[jmp_index] = IRInstruction::Jmp(match do_while {
                    true => loop_label,
                    false => check_label,
                });
            }
        }
        0
    }
}

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

            Ident(_name, symbol_number, global) => {
                let addr = context.next_vreg();
                let vreg = context.next_vreg();

                if *global {
                    todo!();
                } else {
                    result.push(IRInstruction::AddrL(
                        IRSize::P,
                        addr,
                        *symbol_number as usize,
                    ));
                }

                result.push(IRInstruction::Load(IRSize::S32, vreg, addr));
                vreg
            }

            Function(func, arguments) => {
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.eval(result, context))
                    .collect::<Vec<u32>>();
                let sizes = vec![IRSize::S32; arguments.len()];
                let arguments = Box::new(IRArguments { arguments, sizes });

                if let Ident(name, ..) = &func.variant {
                    let vreg = context.next_vreg();
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
            Ident(_name, symbol_number, true) => {
                let addr = context.next_vreg();

                result.push(IRInstruction::AddrL(
                    IRSize::P,
                    addr,
                    *symbol_number as usize,
                ));
                addr
            }
            _ => {
                unreachable!()
            }
        }
    }
}
