mod analysis;
mod expression_analysis;
mod type_checking;
use self::analysis::Analysis;
use crate::parser::ast::*;

#[derive(Clone, Debug)]
struct SemanticAnalyzer {
    file_table: Vec<String>,
    errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new(file_table: Vec<String>) -> SemanticAnalyzer {
        SemanticAnalyzer {
            file_table,
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, translation_unit: &mut TranslationUnit) -> Result<(), Vec<String>> {
        translation_unit.analyze(self);
        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(())
        }
    }
}
