use super::SemanticAnalyzer;
use crate::parser::ast::*;

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
                for statement in statements {
                    statement.analyze(analyzer);
                }
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
            Declaration {
                span: _,
                ident: _,
                decl_type: _,
                init,
            } => {
                if let Some(init) = init {
                    init.analyze(analyzer);
                }
                todo!()
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
