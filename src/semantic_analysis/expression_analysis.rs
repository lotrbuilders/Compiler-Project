use super::analysis::Analysis;
use super::SemanticAnalyzer;
use crate::parser::ast::*;

// The analysis for expressions
impl Analysis for Expression {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        let _ = analyzer;
        use ExpressionVariant::*;
        match &mut self.variant {
            ConstI(_) => {}
        }
    }
}
