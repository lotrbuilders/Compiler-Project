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

impl Analysis for Statement {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use Statement::*;
        match self {
            Return {
                span: _,
                expression,
            } => expression.analyze(analyzer),

            If {
                span,
                expression,
                statement,
                else_statement,
            } => {
                expression.analyze(analyzer);
                statement.analyze(analyzer);
                if let Declaration { .. } = **statement {
                    analyzer.errors.push(warning!(
                        span,
                        "A declaration can not be used as the body of a control flow statement"
                    ))
                }
                if let Some(statement) = else_statement {
                    statement.analyze(analyzer);
                    if let Declaration { .. } = **statement {
                        analyzer.errors.push(warning!(
                            span,
                            "A declaration can not be used as the body of a control flow statement"
                        ))
                    }
                }
            }

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

            Expression {
                span: _,
                expression,
            } => {
                expression.analyze(analyzer);
            }
        }
    }
}
