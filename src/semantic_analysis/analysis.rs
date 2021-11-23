use super::SemanticAnalyzer;
use crate::parser::ast::*;

pub(super) trait Analysis {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        analyzer.errors.push("Unimplemented".to_string()); //Use official error reporting
    }
}

impl Analysis for TranslationUnit {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        for declaration in &mut self.global_declarations {
            let _ = declaration.analyze(analyzer);
        }
    }
}

impl Analysis for ExternalDeclaration {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        match &mut self.function_body {
            Some(statements) => {
                for statement in statements {
                    let _ = statement.analyze(analyzer);
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
            Return { span, expression } => expression.analyze(analyzer),
            _ => {
                //Error
            }
        }
    }
}
