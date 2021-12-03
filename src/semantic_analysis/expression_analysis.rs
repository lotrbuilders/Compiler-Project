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
            Ident(_) => {
                todo!()
            }

            Identity(exp) | Negate(exp) | BinNot(exp) | LogNot(exp) => {
                exp.analyze(analyzer);
            }

            Add(left, right)
            | Subtract(left, right)
            | Multiply(left, right)
            | Divide(left, right) => {
                left.analyze(analyzer);
                right.analyze(analyzer);
            }

            Assign(left, right) => {
                left.analyze(analyzer);
                right.analyze(analyzer);
            }
        }
    }
}
