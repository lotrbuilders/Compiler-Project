mod analysis;
mod expression_analysis;
mod type_checking;
use self::analysis::Analysis;
use crate::parser::ast::*;

// The semantic analyzer checks the entire syntax tree for problems
// The semantic analyzer is passed as a member and modified using traits

#[derive(Clone, Debug)]
pub struct SemanticAnalyzer {
    errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        SemanticAnalyzer { errors: Vec::new() }
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
