use super::{Evaluate, EvaluationContext};
use crate::backend::{ir::*, Backend};
use crate::parser::ast::*;
use crate::parser::r#type::DeclarationType;
use crate::semantic_analysis::symbol_table::Symbol;
use std::collections::{HashMap, HashSet};

impl ExternalDeclaration {
    pub fn eval(&self, backend: &mut dyn Backend) -> Option<IRFunction> {
        match &self.function_body {
            Some(statements) => {
                let mut instructions = Vec::<IRInstruction>::new();
                let mut context = EvaluationContext {
                    vreg_counter: 0,
                    label_counter: 1,
                    variables: Vec::new(),
                    strings: Vec::new(),
                    unfixed_break: Vec::new(),
                    unfixed_continue: Vec::new(),
                    loop_depth: 0,
                    backend: backend,
                };
                instructions.push(IRInstruction::Label(None, 0));

                let arguments = self.eval_function_arguments(&mut instructions, &mut context);
                for statement in statements {
                    statement.eval(&mut instructions, &mut context);
                }
                Some(IRFunction {
                    name: self.name.clone(),
                    return_size: IRSize::S32,
                    instructions,
                    arguments,
                    variables: context.variables,
                    strings: context.strings,
                })
            }
            None => {
                log::info!("Empty function body");
                None
            }
        }
    }

    fn eval_function_arguments(
        &self,
        result: &mut Vec<IRInstruction>,
        context: &mut EvaluationContext,
    ) -> IRArguments {
        let arguments = self.ast_type.get_function_arguments().unwrap();
        let count = arguments.len();
        let ir_arguments = arguments
            .iter()
            .map(|arg| context.get_size(&arg.clone().remove_name().array_promotion()))
            .collect();
        let in_register = context.backend.get_arguments_in_registers(&ir_arguments);
        let vreg_count = in_register.iter().filter(|&&in_reg| in_reg).count() as u32;
        context.vreg_counter += vreg_count * 2;
        let mut vregs = Vec::new();
        for arg in 0..arguments.len() {
            let variable = IRVariable {
                size: ir_arguments[arg].clone(),
                count: 1,
            };
            context.variables.push(variable);
            if in_register[arg] {
                let argument = arg as u32;
                let addr = vreg_count + argument;
                result.push(IRInstruction::AddrL(IRSize::P, addr, arg));
                result.push(IRInstruction::Store(
                    ir_arguments[arg].clone(),
                    arg as u32,
                    addr,
                ));
                vregs.push(Some(argument));
            } else {
                vregs.push(None);
            }
        }
        IRArguments {
            sizes: ir_arguments,
            arguments: vregs,
            count,
        }
    }

    pub fn eval_global(
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
            log::trace!(
                "Found non defined function {} with type {:?}",
                self.name,
                map[&self.name].declaration_type
            );
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
            if let Some(expression) = &self.expression {
                defined.insert(self.name.clone());
                if let ExpressionVariant::ConstI(value) = expression.variant {
                    Some(IRGlobal {
                        name: self.name.clone(),
                        size: IRSize::S32,
                        value: Some(value),
                        function: false,
                    })
                } else {
                    unreachable!();
                }
            } else if map[&self.name].declaration_type == DeclarationType::Declaration {
                defined.insert(self.name.clone());
                Some(IRGlobal {
                    name: self.name.clone(),
                    size: IRSize::S32,
                    value: None,
                    function: false,
                })
            } else {
                None
            }
        }
    }
}
