use super::SemanticAnalyzer;
use crate::parser::ast::*;

pub(super) trait Analysis {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> Result<(), ()> {
        analyzer.errors.push("Unimplemented".to_string()); //Use official error reporting
        Err(())
    }
}

impl Analysis for TranslationUnit {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> Result<(), ()> {
        for declaration in &mut self.global_declarations {
            let _ = declaration.analyze(analyzer);
        }
        Ok(())
    }
}

impl Analysis for ExternalDeclaration {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> Result<(), ()> {
        match &mut self.function_body {
            Some(statements) => {
                for statement in statements {
                    let _ = statement.analyze(analyzer);
                }
            }
            None => (),
        }
        Ok(())
    }
}

impl Analysis for Statement {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> Result<(), ()> {
        use Statement::*;
        match self {
            Return { span, expression } => expression.analyze(analyzer)?,
            _ => {
                //Error
                return Err(());
            }
        }
        Ok(())
    }
}
