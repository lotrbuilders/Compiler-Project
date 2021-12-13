use crate::backend::ir::*;
use crate::parser::ast::*;

fn insert_place_holder_jump(
    result: &mut Vec<IRInstruction>,
    label_counter: &mut u32,
) -> (usize, u32) {
    let index = result.len();
    result.push(IRInstruction::Jmp(0));

    let label = insert_label(result, label_counter);

    (index, label)
}

fn insert_place_holder_jump_phi(
    result: &mut Vec<IRInstruction>,
    label_counter: &mut u32,
    phi: Box<IRPhi>,
) -> (usize, u32) {
    let index = result.len();
    result.push(IRInstruction::Jmp(0));

    let label = insert_phi_label(result, label_counter, phi);

    (index, label)
}

fn insert_label(result: &mut Vec<IRInstruction>, label_counter: &mut u32) -> u32 {
    let label = *label_counter;
    *label_counter += 1;
    result.push(IRInstruction::Label(None, label));

    label
}

fn insert_phi_label(
    result: &mut Vec<IRInstruction>,
    label_counter: &mut u32,
    phi: Box<IRPhi>,
) -> u32 {
    let label = *label_counter;
    *label_counter += 1;
    result.push(IRInstruction::Label(Some(phi), label));

    label
}

fn insert_phi_src(_result: &mut Vec<IRInstruction>, label_counter: &u32) -> u32 {
    let label = *label_counter - 1;
    //result.push(IRInstruction::PhiSrc(label));
    label
}

// This module is used to evaluate the AST into an IR

// The public function used to evaluate the ast
pub fn evaluate(ast: &TranslationUnit) -> Vec<IRFunction> {
    let mut result = Vec::<IRFunction>::new();
    for global in &ast.global_declarations {
        log::trace!("Evaluating individual global");
        if let Some(declaration) = global.eval() {
            result.push(declaration);
        }
    }
    result
}

impl ExternalDeclaration {
    fn eval(&self) -> Option<IRFunction> {
        match &self.function_body {
            Some(statements) => {
                let mut instructions = Vec::<IRInstruction>::new();
                let mut vreg = 0;
                let mut label = 1;
                let mut variables = Vec::<IRSize>::new();
                instructions.push(IRInstruction::Label(None, 0));
                for statement in statements {
                    statement.eval(&mut instructions, &mut vreg, &mut label, &mut variables);
                }
                Some(IRFunction {
                    name: self.name.clone(),
                    return_size: IRSize::S32,
                    instructions,
                    variables,
                })
            }
            None => {
                log::info!("Empty function body");
                None
            }
        }
    }
}

// The trait Evaluate is used by statements and expressions
// The vreg counter should be updated every use
// The function returns the virtual register representing its result
pub trait Evaluate {
    fn eval(
        &self,
        result: &mut Vec<IRInstruction>,
        vreg_counter: &mut u32,
        label_counter: &mut u32,
        variables: &mut Vec<IRSize>,
    ) -> u32;
}

impl Evaluate for Statement {
    fn eval(
        &self,
        result: &mut Vec<IRInstruction>,
        vreg_counter: &mut u32,
        label_counter: &mut u32,
        variables: &mut Vec<IRSize>,
    ) -> u32 {
        use Statement::*;
        match self {
            Declaration {
                span: _,
                ident: _,
                decl_type: _,
                init,
            } => {
                let index = variables.len();
                variables.push(IRSize::S32); //Should be determined by type of declaration later
                if let Some(exp) = init {
                    let vreg = exp.eval(result, vreg_counter, label_counter, variables);
                    let addr = *vreg_counter;
                    *vreg_counter += 1;
                    result.push(IRInstruction::AddrL(IRSize::P, addr, index));
                    result.push(IRInstruction::Store(IRSize::S32, vreg, addr));
                }
            }

            Empty(_) => (),

            Expression {
                span: _,
                expression,
            } => {
                expression.eval(result, vreg_counter, label_counter, variables);
            }

            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                let cond = expression.eval(result, vreg_counter, label_counter, variables);

                let phi1 = insert_phi_src(result, label_counter);
                let (index, _) = insert_place_holder_jump(result, label_counter);
                statement.eval(result, vreg_counter, label_counter, variables);

                if let Some(statement) = else_statement {
                    let phi1 = insert_phi_src(result, label_counter);

                    let (else_index, label) = insert_place_holder_jump(result, label_counter);
                    result[index] = IRInstruction::Jnc(IRSize::S32, cond, label);

                    statement.eval(result, vreg_counter, label_counter, variables);

                    let phi2 = insert_phi_src(result, label_counter);
                    let (last_index, label) = insert_place_holder_jump_phi(
                        result,
                        label_counter,
                        IRPhi::empty(vec![phi1, phi2]),
                    );
                    result[else_index] = IRInstruction::Jmp(label);
                    result[last_index] = IRInstruction::Jmp(label);
                } else {
                    let phi2 = insert_phi_src(result, label_counter);

                    let (last_index, label) = insert_place_holder_jump_phi(
                        result,
                        label_counter,
                        IRPhi::empty(vec![phi1, phi2]),
                    );
                    result[index] = IRInstruction::Jnc(IRSize::S32, cond, label);
                    result[last_index] = IRInstruction::Jmp(label);
                }
            }

            Return {
                span: _,
                expression,
            } => {
                let vreg = expression.eval(result, vreg_counter, label_counter, variables);
                result.push(IRInstruction::Ret(IRSize::S32, vreg))
            }
        }
        0
    }
}

impl Evaluate for Expression {
    fn eval(
        &self,
        result: &mut Vec<IRInstruction>,
        vreg_counter: &mut u32,
        label_counter: &mut u32,
        variables: &mut Vec<IRSize>,
    ) -> u32 {
        use BinaryExpressionType::*;
        use ExpressionVariant::*;
        use UnaryExpressionType::*;
        match &self.variant {
            &ConstI(value) => {
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                result.push(IRInstruction::Imm(IRSize::S32, vreg, value));
                vreg
            }

            Ident(_name, symbol_number) => {
                let addr = *vreg_counter;
                let vreg = *vreg_counter + 1;
                *vreg_counter += 2;

                result.push(IRInstruction::AddrL(
                    IRSize::P,
                    addr,
                    *symbol_number as usize,
                ));
                result.push(IRInstruction::Load(IRSize::S32, vreg, addr));
                vreg
            }

            Assign(left, right) => {
                let vreg = right.eval(result, vreg_counter, label_counter, variables);
                let addr = left.eval_lvalue(result, vreg_counter, variables);

                result.push(IRInstruction::Store(IRSize::S32, vreg, addr));
                vreg
            }

            #[allow(unused_variables)]
            Ternary(cond, left, right) => {
                let cond = cond.eval(result, vreg_counter, label_counter, variables);

                let (index, _) = insert_place_holder_jump(result, label_counter);
                let left = left.eval(result, vreg_counter, label_counter, variables);

                let phi1 = insert_phi_src(result, label_counter);

                let (else_index, label) = insert_place_holder_jump(result, label_counter);
                result[index] = IRInstruction::Jnc(IRSize::S32, cond, label);

                let right = right.eval(result, vreg_counter, label_counter, variables);

                let phi2 = insert_phi_src(result, label_counter);

                //let (last_index, label) = insert_place_holder_jump(result, label_counter);

                let vreg = *vreg_counter;
                *vreg_counter += 1;

                let (last_index, label) = insert_place_holder_jump_phi(
                    result,
                    label_counter,
                    IRPhi::ternary((phi1, phi2), vreg, (left, right)),
                );
                result[else_index] = IRInstruction::Jmp(label);
                result[last_index] = IRInstruction::Jmp(label);
                vreg
            }

            // Could benefit from constants in phi nodes
            Binary(op @ (LogOr | LogAnd), left, right) => {
                let start_label = insert_phi_src(result, label_counter);
                let left = left.eval(result, vreg_counter, label_counter, variables);
                let first_operand = {
                    let vreg = *vreg_counter;
                    *vreg_counter += 1;
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
                let (left_jmp, left_label) = insert_place_holder_jump(result, label_counter);

                let right = right.eval(result, vreg_counter, label_counter, variables);
                let second_operand = {
                    let vreg = *vreg_counter;
                    *vreg_counter += 2;
                    result.push(IRInstruction::Imm(IRSize::S32, vreg, 0));
                    result.push(IRInstruction::Ne(IRSize::S32, vreg + 1, right, vreg));
                    vreg + 1
                };
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                let (right_jmp, right_label) = insert_place_holder_jump_phi(
                    result,
                    label_counter,
                    IRPhi::ternary(
                        (start_label, left_label),
                        vreg,
                        (first_operand, second_operand),
                    ),
                );

                result[right_jmp] = IRInstruction::Jmp(right_label);
                result[left_jmp] = match op {
                    LogOr => IRInstruction::Jnc(IRSize::S32, left, right_label),
                    LogAnd => IRInstruction::Jcc(IRSize::S32, left, right_label),
                    _ => unreachable!(),
                };

                vreg
            }

            Binary(op, left, right) => {
                let left = left.eval(result, vreg_counter, label_counter, variables);
                let right = right.eval(result, vreg_counter, label_counter, variables);
                let vreg = *vreg_counter;
                *vreg_counter += 1;
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
                let exp = exp.eval(result, vreg_counter, label_counter, variables);
                exp
            }

            Unary(op, exp) => {
                let left = exp.eval(result, vreg_counter, label_counter, variables);
                let right = *vreg_counter;
                let vreg = *vreg_counter + 1;
                *vreg_counter += 2;

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
    fn eval_lvalue(
        &self,
        result: &mut Vec<IRInstruction>,
        vreg_counter: &mut u32,
        _variables: &mut Vec<IRSize>,
    ) -> u32 {
        use ExpressionVariant::*;
        match &self.variant {
            Ident(_name, symbol_number) => {
                let addr = *vreg_counter;
                *vreg_counter += 1;

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
