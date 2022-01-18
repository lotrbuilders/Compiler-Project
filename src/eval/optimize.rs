use super::evaluation_context::EvaluateSize;
use crate::backend::{Backend, TypeInfo, TypeInfoTable};
use crate::options::OptimizationSettings;
use crate::parser::ast::*;
use crate::span::Span;
use crate::table::StructTable;
use std::mem;

struct Optimizer<'a> {
    type_info: TypeInfoTable,
    struct_size_table: &'a Vec<TypeInfo>,
    fold_if: bool,
}

impl EvaluateSize for Optimizer<'_> {
    fn type_info<'a>(&'a self) -> &'a TypeInfoTable {
        &self.type_info
    }

    fn struct_size_table<'a>(&'a self) -> &'a Vec<TypeInfo> {
        &self.struct_size_table
    }
}

pub fn optimize(
    ast: &mut TranslationUnit,
    backend: &dyn Backend,
    struct_table: &StructTable,
    optimization_settings: &OptimizationSettings,
) {
    log::trace!("optimization_settings: {:?}", optimization_settings);
    if optimization_settings.optimization_level >= 0
        || optimization_settings
            .optimizations
            .contains(&String::from("const-eval"))
    {
        let fold_if = optimization_settings.optimization_level >= 1
            || optimization_settings
                .optimizations
                .contains(&String::from("const-eval-if"));

        let optimizer = Optimizer {
            type_info: backend.get_type_info_table(),
            struct_size_table: &struct_table.info,
            fold_if,
        };
        ast.constant_eval(&optimizer)
    }
}

trait Optimize {
    fn constant_eval(&mut self, optimizer: &Optimizer);
}

impl Optimize for TranslationUnit {
    fn constant_eval(&mut self, optimizer: &Optimizer) {
        for global in &mut self.global_declarations {
            global.constant_eval(optimizer);
        }
    }
}

impl Optimize for ExternalDeclaration {
    fn constant_eval(&mut self, optimizer: &Optimizer) {
        if let Some(statements) = &mut self.function_body {
            for statement in statements {
                statement.constant_eval(optimizer);
            }
        }
    }
}

impl Optimize for Statement {
    fn constant_eval(&mut self, optimizer: &Optimizer) {
        match self {
            Statement::Compound {
                span: _,
                statements,
            } => {
                for statement in statements {
                    statement.constant_eval(optimizer);
                }
            }

            Statement::If {
                span,
                expression,
                statement,
                else_statement,
            } => {
                expression.constant_eval(optimizer);
                if optimizer.fold_if && expression.is_constant() {
                    if expression.get_const_value() != 0 {
                        let statement =
                            mem::replace(statement, Box::new(Statement::Empty(span.clone())));
                        *self = *statement;
                    } else if let Some(statement) = else_statement {
                        *self = mem::replace(statement, Statement::Empty(span.clone()));
                    } else {
                        *self = Statement::Empty(span.clone());
                    }
                }
            }

            Statement::For {
                span: _,
                init,
                condition,
                expression,
                statement: _,
            } => {
                if let Some(init) = init {
                    init.constant_eval(optimizer);
                }
                if let Some(expression) = expression {
                    expression.constant_eval(optimizer);
                }
                if let Some(condition) = condition {
                    condition.constant_eval(optimizer);
                }
            }

            Statement::While { expression, .. }
            | Statement::Declaration {
                init: Some(expression),
                ..
            }
            | Statement::Return {
                expression: Some(expression),
                ..
            }
            | Statement::Expression {
                span: _,
                expression,
            } => expression.constant_eval(optimizer),

            Statement::Declaration { .. }
            | Statement::Return { .. }
            | Statement::Continue { span: _ }
            | Statement::Break { span: _ }
            | Statement::Empty(_) => (),
        }
    }
}

impl Optimize for Expression {
    fn constant_eval(&mut self, optimizer: &Optimizer) {
        let span = Span::empty();
        let exp = mem::replace(self, Expression::default(&span));
        *self = exp.const_eval(optimizer);
    }
}
