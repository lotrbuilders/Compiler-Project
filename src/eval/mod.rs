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
                for statement in statements {
                    statement.eval(&mut instructions, &mut vreg);
                }
                Some(IRFunction {
                    name: self.name.clone(),
                    return_size: IRSize::I32,
                    instructions,
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
    fn eval(&self, result: &mut Vec<IRInstruction>, vreg_counter: &mut u32) -> u32;
}

impl Evaluate for Statement {
    fn eval(&self, result: &mut Vec<IRInstruction>, vreg_counter: &mut u32) -> u32 {
        use Statement::*;
        match self {
            Return {
                span: _,
                expression,
            } => {
                let vreg = expression.eval(result, vreg_counter);
                result.push(IRInstruction::Ret(IRSize::I32, vreg))
            }
        }
        0
    }
}

impl Evaluate for Expression {
    fn eval(&self, result: &mut Vec<IRInstruction>, vreg_counter: &mut u32) -> u32 {
        use ExpressionVariant::*;
        match &self.variant {
            &ConstI(value) => {
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                result.push(IRInstruction::Imm(IRSize::I32, vreg, value));
                vreg
            }
            Add(left, right)
            | Subtract(left, right)
            | Multiply(left, right)
            | Divide(left, right) => {
                let _left = left.eval(result, vreg_counter);
                let _right = right.eval(result, vreg_counter);
                let vreg = *vreg_counter;
                *vreg_counter += 1;
                log::info!("Unimplemented evaluate");
                result.push(match self.variant {
                    Add(..) => IRInstruction::Imm(IRSize::I32, vreg, 0),
                    Subtract(..) => IRInstruction::Imm(IRSize::I32, vreg, 0),
                    Multiply(..) => IRInstruction::Imm(IRSize::I32, vreg, 0),
                    Divide(..) => IRInstruction::Imm(IRSize::I32, vreg, 0),
                    _ => unreachable!(),
                });
                vreg
            }
        }
    }
}
