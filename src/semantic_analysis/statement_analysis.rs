use super::analysis::Analysis;
use super::SemanticAnalyzer;
use crate::parser::ast::Statement;
use crate::parser::r#type::DeclarationType;
use crate::{error, warning};

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
            Break { .. } | Continue { .. } | For { .. } | If { .. } | While { .. } => {
                self.analyze_control_flow(analyzer)
            }

            Return {
                span,
                expression,
                ast_type,
            } => {
                *ast_type = analyzer.function_return_type.clone();
                if expression.is_none() {
                    log::debug!("Void return");
                    if !ast_type.is_void() {
                        analyzer
                            .errors
                            .push(error!(span, "Return without value in none void function"));
                    }
                    return;
                } else {
                    log::debug!("Normal return");
                }
                let expression = expression.as_mut().unwrap();
                expression.analyze(analyzer);
                if !expression
                    .ast_type
                    .is_compatible(&analyzer.function_return_type)
                {
                    analyzer.errors.push(error!(
                        span,
                        "Incompatible return types: function returns {}, but {} is returned",
                        analyzer.function_return_type,
                        expression.ast_type
                    ));
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
                ast_type,
                decl_type: symbol_type,
                init,
            } => {
                *symbol_type = ast_type.to_type(analyzer);
                if ident.is_none() && !ast_type.is_type_declaration() {
                    analyzer
                        .errors
                        .push(error!(span, "Declaration with a name"));
                }
                if ident.is_none() {
                    return;
                }
                let ident = ident.as_ref().unwrap();
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
                analyzer.enter_scope();
                for stmt in statements {
                    stmt.analyze(analyzer);
                }
                analyzer.leave_scope();
            }
        }
    }
}

impl Statement {
    fn analyze_control_flow(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use Statement::*;
        match self {
            Break { span } => {
                if !analyzer.in_loop() {
                    analyzer
                        .errors
                        .push(error!(span, "'break' must be in a loop")); //Or switch statement later
                }
            }

            Continue { span } => {
                if !analyzer.in_loop() {
                    analyzer
                        .errors
                        .push(error!(span, "'continue' must be in a loop"))
                }
            }

            For {
                span: _,
                init,
                condition,
                expression,
                statement,
            } => {
                analyzer.enter_scope();
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
                analyzer.leave_scope();
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

            _ => unreachable!(),
        }
    }
}
