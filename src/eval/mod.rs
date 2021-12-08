use crate::backend::ir::*;
use crate::parser::ast::*;

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
                let mut variables = Vec::<IRSize>::new();
                for statement in statements {
                    statement.eval(&mut instructions, &mut vreg, &mut variables);
                }
                Some(IRFunction {
                    name: self.name.clone(),
                    return_size: IRSize::I32,
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
        variables: &mut Vec<IRSize>,
    ) -> u32;
}

impl Evaluate for Statement {
    fn eval(
        &self,
        result: &mut Vec<IRInstruction>,
        vreg_counter: &mut u32,
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
                variables.push(IRSize::I32); //Should be determined by type of declaration later
                if let Some(exp) = init {
                    let vreg = exp.eval(result, vreg_counter, variables);
                    let addr = *vreg_counter;
                    *vreg_counter += 1;
                    result.push(IRInstruction::AddrL(IRSize::P, addr, index));
                    result.push(IRInstruction::Store(IRSize::I32, vreg, addr));
                }
            }

            Expression {
                span: _,
                expression,
            } => {
                expression.eval(result, vreg_counter, variables);
            }

            Return {
                span: _,
                expression,
            } => {
                let vreg = expression.eval(result, vreg_counter, variables);
                result.push(IRInstruction::Ret(IRSize::I32, vreg))
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
        variables: &mut Vec<IRSize>,
    ) -> u32 {
        use BinaryExpressionType::*;
        use ExpressionVariant::*;
        use UnaryExpressionType::*;
        match &self.variant {
            &ConstI(value) => {
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                result.push(IRInstruction::Imm(IRSize::I32, vreg, value));
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
                result.push(IRInstruction::Load(IRSize::I32, vreg, addr));
                vreg
            }

            Assign(left, right) => {
                let vreg = right.eval(result, vreg_counter, variables);
                let addr = left.eval_lvalue(result, vreg_counter, variables);

                result.push(IRInstruction::Store(IRSize::I32, vreg, addr));
                vreg
            }

            Binary(op, left, right) => {
                let left = left.eval(result, vreg_counter, variables);
                let right = right.eval(result, vreg_counter, variables);
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                result.push(match op {
                    Add => IRInstruction::Add(IRSize::I32, vreg, left, right),
                    Subtract => IRInstruction::Sub(IRSize::I32, vreg, left, right),
                    Multiply => IRInstruction::Mul(IRSize::I32, vreg, left, right),
                    Divide => IRInstruction::Div(IRSize::I32, vreg, left, right),
                    _ => unimplemented!(),
                });
                vreg
            }

            Unary(Identity, exp) => {
                let exp = exp.eval(result, vreg_counter, variables);
                exp
            }

            Unary(op, exp) => {
                let left = exp.eval(result, vreg_counter, variables);
                let right = *vreg_counter;
                let vreg = *vreg_counter + 1;
                *vreg_counter += 2;

                result.push(match op {
                    Negate | LogNot => IRInstruction::Imm(IRSize::I32, right, 0),
                    BinNot => IRInstruction::Imm(IRSize::I32, right, -1),
                    _ => unreachable!(),
                });
                result.push(match op {
                    Negate => IRInstruction::Sub(IRSize::I32, vreg, right, left),
                    BinNot => IRInstruction::Xor(IRSize::I32, vreg, left, right),
                    LogNot => IRInstruction::Eq(IRSize::I32, vreg, left, right),
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
