use super::SemanticAnalyzer;
use crate::parser::ast::*;
use crate::parser::r#type::type2string;
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

impl Analysis for ExternalDeclaration {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        match &mut self.function_body {
            Some(statements) => {
                analyzer.symbol_table.enter_scope();
                for statement in statements {
                    statement.analyze(analyzer);
                }
                analyzer.symbol_table.leave_scope();
            }
            None => (),
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
                expression.analyze(analyzer);
                statement.analyze(analyzer);
                statement.check_for_declaration(analyzer);
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
                init.as_mut().map(|init| init.analyze(analyzer));
                analyzer.symbol_table.enter_scope();
                condition
                    .as_mut()
                    .map(|condition| condition.analyze(analyzer));
                expression
                    .as_mut()
                    .map(|expression| expression.analyze(analyzer));
                statement.analyze(analyzer);
                statement.check_for_declaration(analyzer);
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
                if let Some(init) = init {
                    init.analyze(analyzer);
                }
                if let Err(()) = analyzer.symbol_table.try_insert(ident, symbol_type) {
                    analyzer.errors.push(error!(
                        span,
                        "Identifier {} with type {} already defined as type {}",
                        ident,
                        type2string(symbol_type),
                        type2string(&analyzer.symbol_table.get(ident).unwrap().symbol_type)
                    ));
                }
            }

            Compound {
                span: _,
                statements,
            } => todo!(),

            Break { span } | Continue { span } => {
                todo!();
            }
        }
    }
}
