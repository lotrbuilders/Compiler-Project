use super::analysis::Analysis;
use super::SemanticAnalyzer;
use crate::parser::ast::ExternalDeclaration;
use crate::parser::Type;
use crate::semantic_analysis::type_checking::{compare_arguments, compare_return_types};
use crate::{error, parser::r#type::DeclarationType};

impl ExternalDeclaration {
    fn insert_or_update(
        &mut self,
        analyzer: &mut SemanticAnalyzer,
        declaration_type: DeclarationType,
    ) {
        if let Err(()) =
            analyzer
                .symbol_table
                .try_insert(&self.name, &self.ast_type, declaration_type)
        {
            let old_definition = analyzer.symbol_table.get(&self.name).unwrap().clone();
            compare_return_types(
                analyzer,
                &self.span,
                &self.name,
                &old_definition.symbol_type,
                &self.ast_type,
            );
            use DeclarationType::*;
            match (old_definition.declaration_type, declaration_type) {
                (Declaration, Declaration) => (),
                (Definition | Prototype, Declaration) => (),
                (Declaration, Prototype | Definition) => {
                    let symbol = analyzer.symbol_table.get_mut(&self.name).unwrap();
                    symbol.symbol_type = self.ast_type.clone();
                    symbol.declaration_type = declaration_type;
                }

                (Prototype, Prototype | Definition) => {
                    compare_arguments(
                        analyzer,
                        &self.span,
                        &self.name,
                        &old_definition.symbol_type,
                        &self.ast_type,
                    );
                    let symbol = analyzer.symbol_table.get_mut(&self.name).unwrap();
                    symbol.symbol_type = self.ast_type.clone();
                    symbol.declaration_type = declaration_type;
                }
                (Definition, Prototype) => {
                    compare_arguments(
                        analyzer,
                        &self.span,
                        &self.name,
                        &old_definition.symbol_type,
                        &self.ast_type,
                    );
                }

                (Definition, Definition) => analyzer.errors.push(error!(
                    self.span,
                    "Global
                     {} redefined",
                    self.name
                )),
            }
        }
    }
}

impl Analysis for ExternalDeclaration {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        if let Some(_) = self.function_body {
            if !self.ast_type.is_function() {
                analyzer.errors.push(error!(
                    self.span,
                    "Function body defined for global variable"
                ))
            } else {
                self.insert_or_update(analyzer, DeclarationType::Definition)
            }
        } else {
            if self.ast_type.is_function() {
                if self.ast_type.is_declaration() {
                    self.insert_or_update(analyzer, DeclarationType::Declaration)
                } else {
                    self.insert_or_update(analyzer, DeclarationType::Prototype)
                }
            } else if let Some(_) = self.expression {
                self.insert_or_update(analyzer, DeclarationType::Definition)
            } else {
                self.insert_or_update(analyzer, DeclarationType::Declaration)
            }
        }

        if self.ast_type.is_function() {
            let return_type: Type = self.ast_type.get_return_type().unwrap().into();
            if return_type.is_array() {
                analyzer
                    .errors
                    .push(error!(self.span, "Cannot return an array"));
            }
        }

        if let Some(statements) = &mut self.function_body {
            analyzer.function_return_type = self.ast_type.get_return_type().unwrap().into();
            analyzer.symbol_table.enter_scope();
            if let Some(arguments) = self.ast_type.get_function_arguments() {
                for arg in arguments {
                    if let Some(name) = arg.get_name() {
                        let symbol_type = arg.clone().remove_name().array_promotion();
                        if let Err(()) = analyzer.symbol_table.try_insert(
                            &name,
                            &symbol_type,
                            DeclarationType::Definition,
                        ) {
                            analyzer.errors.push(error!(
                                self.span,
                                "Argument {} with type {} already defined as type {}",
                                &name,
                                &symbol_type,
                                &analyzer.symbol_table.get(&name).unwrap().symbol_type
                            ));
                        }
                    } else {
                        analyzer.errors.push(error!(
                            self.span,
                            "Function argument without name in {}", self.name
                        ));
                    }
                }
            }

            for statement in statements {
                statement.analyze(analyzer);
            }
            analyzer.symbol_table.leave_scope();
        }

        if let Some(expression) = &mut self.expression {
            expression.analyze(analyzer);
            *expression = expression.clone().const_eval();
            if !expression.is_constant() {
                analyzer.errors.push(error!(
                    self.span,
                    "Initialization of global variable must be constant"
                ));
            }
        }
    }
}
