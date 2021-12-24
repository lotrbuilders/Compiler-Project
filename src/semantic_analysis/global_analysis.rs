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
        let name = self.name.as_ref().unwrap();

        if let Err(()) = analyzer
            .symbol_table
            .try_insert(name, &self.decl_type, declaration_type)
        {
            let old_definition = analyzer.symbol_table.get(name).unwrap().clone();
            compare_return_types(
                analyzer,
                &self.span,
                name,
                &old_definition.symbol_type,
                &self.decl_type,
            );
            use DeclarationType::*;
            match (old_definition.declaration_type, declaration_type) {
                (Declaration, Declaration) => (),
                (Definition | Prototype, Declaration) => (),
                (Declaration, Prototype | Definition) => {
                    let symbol = analyzer.symbol_table.get_mut(name).unwrap();
                    symbol.symbol_type = self.decl_type.clone();
                    symbol.declaration_type = declaration_type;
                }

                (Prototype, Prototype | Definition) => {
                    compare_arguments(
                        analyzer,
                        &self.span,
                        name,
                        &old_definition.symbol_type,
                        &self.decl_type,
                    );
                    let symbol = analyzer.symbol_table.get_mut(name).unwrap();
                    symbol.symbol_type = self.decl_type.clone();
                    symbol.declaration_type = declaration_type;
                }
                (Definition, Prototype) => {
                    compare_arguments(
                        analyzer,
                        &self.span,
                        name,
                        &old_definition.symbol_type,
                        &self.decl_type,
                    );
                }

                (Definition, Definition) => analyzer.errors.push(error!(
                    self.span,
                    "Global
                     {} redefined",
                    name
                )),
            }
        }
    }
}

impl Analysis for ExternalDeclaration {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        self.decl_type = self.ast_type.to_type(analyzer);
        log::debug!("function name: {:?}", self.name);
        if self.name.is_none() {
            if !self.ast_type.is_type_declaration() {
                analyzer
                    .errors
                    .push(error!(self.span, "Global definition without name found"));
            }
            return;
        }
        log::debug!("function return type: {}", self.decl_type);
        //let name = self.name.as_ref().unwrap();

        if let Some(_) = self.function_body {
            if !self.decl_type.is_function() {
                analyzer.errors.push(error!(
                    self.span,
                    "Function body defined for global variable"
                ))
            } else {
                self.insert_or_update(analyzer, DeclarationType::Definition)
            }
        } else {
            if self.decl_type.is_function() {
                if self.decl_type.is_declaration() {
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

        if self.decl_type.is_function() {
            let return_type: Type = self.decl_type.get_return_type().unwrap().into();
            if return_type.is_array() || return_type.is_struct() {
                analyzer
                    .errors
                    .push(error!(self.span, "Cannot return an array or struct"));
            }
        }

        let function_body = self.function_body.is_some();
        if let Some(statements) = &mut self.function_body {
            analyzer.function_return_type = self.decl_type.get_return_type().unwrap().into();

            analyzer.enter_scope();
            let arguments = self.ast_type.get_function_arguments(analyzer);
            for (typ, name) in arguments {
                if let Some(name) = name {
                    let symbol_type = typ.array_promotion();
                    if symbol_type.is_struct() {
                        analyzer.errors.push(error!(
                            self.span,
                            "Structs as arguments currently not yet supported\n"
                        ))
                    }

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
                } else if function_body {
                    analyzer.errors.push(error!(
                        self.span,
                        "Function argument without name in {}",
                        self.name.clone().unwrap_or_default()
                    ));
                }
            }

            for statement in statements {
                statement.analyze(analyzer);
            }
            analyzer.leave_scope();
        }

        if let Some(expression) = &mut self.expression {
            expression.analyze(analyzer);
            expression.force_const_eval(analyzer);
        }
    }
}
