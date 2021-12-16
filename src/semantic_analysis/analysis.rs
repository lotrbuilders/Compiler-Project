use super::SemanticAnalyzer;
use crate::parser::ast::*;
use crate::parser::r#type::DeclarationType;
use crate::semantic_analysis::type_checking::{compare_arguments, compare_return_types};
use crate::{error, warning};

pub(super) trait Analysis {
    fn analyze(&mut self, _analyzer: &mut SemanticAnalyzer) -> () {
        log::error!("analyze called on unanalyzable structure");
    }
}

impl Analysis for TranslationUnit {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        for declaration in &mut self.global_declarations {
            declaration.analyze(analyzer);
        }
    }
}

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

        if let Some(statements) = &mut self.function_body {
            analyzer.symbol_table.enter_scope();
            if let Some(arguments) = self.ast_type.get_function_arguments() {
                for arg in arguments {
                    if let Some(name) = arg.get_name() {
                        if let Err(()) = analyzer.symbol_table.try_insert(
                            &name,
                            arg,
                            DeclarationType::Definition,
                        ) {
                            analyzer.errors.push(error!(
                                self.span,
                                "Argument {} with type {} already defined as type {}",
                                &name,
                                arg,
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

impl Statement {
    fn check_for_declaration(&self, analyzer: &mut SemanticAnalyzer) -> () {
        if let Statement::Declaration { span, .. } = self {
            analyzer.errors.push(warning!(
                span,
                "A declaration can not be used as the body of a control flow statement"
            ));
        }
    }
}

impl Analysis for Statement {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use Statement::*;
        match self {
            While {
                span: _,
                expression,
                statement,
                do_while: _,
            } => {
                analyzer.enter_loop();
                expression.analyze(analyzer);
                statement.analyze(analyzer);
                statement.check_for_declaration(analyzer);
                analyzer.leave_loop();
            }

            Return {
                span: _,
                expression,
            } => expression.analyze(analyzer),

            For {
                span: _,
                init,
                condition,
                expression,
                statement,
            } => {
                analyzer.symbol_table.enter_scope();
                analyzer.enter_loop();

                init.as_mut().map(|init| init.analyze(analyzer));
                condition
                    .as_mut()
                    .map(|condition| condition.analyze(analyzer));
                expression
                    .as_mut()
                    .map(|expression| expression.analyze(analyzer));

                statement.analyze(analyzer);
                statement.check_for_declaration(analyzer);

                analyzer.leave_loop();
                analyzer.symbol_table.leave_scope();
            }

            If {
                span: _,
                expression,
                statement,
                else_statement,
            } => {
                expression.analyze(analyzer);
                statement.analyze(analyzer);
                statement.check_for_declaration(analyzer);
                if let Some(statement) = else_statement {
                    statement.analyze(analyzer);
                    statement.check_for_declaration(analyzer);
                }
            }

            Expression {
                span: _,
                expression,
            } => {
                expression.analyze(analyzer);
            }

            Empty(_) => (),

            Declaration {
                span,
                ident,
                decl_type: symbol_type,
                init,
            } => {
                log::trace!("Declaration of {} with type {}", ident, symbol_type);
                if let Some(init) = init {
                    init.analyze(analyzer);
                }
                if let Err(()) = analyzer.symbol_table.try_insert(
                    ident,
                    symbol_type,
                    DeclarationType::Definition,
                ) {
                    analyzer.errors.push(error!(
                        span,
                        "Identifier {} with type {} already defined as type {}",
                        ident,
                        symbol_type,
                        &analyzer.symbol_table.get(ident).unwrap().symbol_type
                    ));
                }
            }

            Compound {
                span: _,
                statements,
            } => {
                analyzer.symbol_table.enter_scope();
                for stmt in statements {
                    stmt.analyze(analyzer);
                }
                analyzer.symbol_table.leave_scope();
            }

            Continue { span } => {
                if !analyzer.in_loop() {
                    analyzer
                        .errors
                        .push(error!(span, "'continue' must be in a loop"))
                }
            }

            Break { span } => {
                if !analyzer.in_loop() {
                    analyzer
                        .errors
                        .push(error!(span, "'break' must be in a loop")); //Or switch statement later
                }
            }
        }
    }
}
