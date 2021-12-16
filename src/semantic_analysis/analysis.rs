use super::SemanticAnalyzer;
use crate::parser::ast::TranslationUnit;

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
