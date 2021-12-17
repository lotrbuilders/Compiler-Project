use super::analysis::Analysis;
use super::SemanticAnalyzer;
use crate::error;
use crate::parser::ast::*;
use crate::semantic_analysis::type_checking::check_arguments_function;

// The analysis for expressions
impl Analysis for Expression {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use ExpressionVariant::*;
        match &mut self.variant {
            ConstI(_) => {}

            Ident(name, symbol_number, global) => {
                if let Some(symbol) = analyzer.symbol_table.get(name) {
                    self.ast_type = symbol.symbol_type.clone();
                    *symbol_number = symbol.number;
                    *global = symbol.global;
                } else {
                    analyzer
                        .errors
                        .push(error!(self.span, "Identifier {} is not defined", name))
                }
            }

            Function(func, arguments) => {
                func.analyze_lvalue(analyzer);
                for arg in arguments.iter_mut() {
                    arg.analyze(analyzer);
                }
                check_arguments_function(analyzer, &self.span, &func.ast_type, arguments);
            }

            Unary(UnaryExpressionType::Address, exp) => {
                exp.analyze_lvalue(analyzer);
            }

            Unary(_op, exp) => {
                exp.analyze(analyzer);
            }

            Binary(_op, left, right) => {
                left.analyze(analyzer);
                right.analyze(analyzer);
            }

            Ternary(cond, left, right) => {
                cond.analyze(analyzer);
                left.analyze(analyzer);
                right.analyze(analyzer);
            }

            Assign(left, right) => {
                left.analyze_lvalue(analyzer);
                right.analyze(analyzer);
            }
        }
    }
}

impl Expression {
    fn analyze_lvalue(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use ExpressionVariant::*;
        match &mut self.variant {
            Ident(..) => self.analyze(analyzer),
            Unary(UnaryExpressionType::Deref, _) => self.analyze(analyzer),
            _ => {
                analyzer
                    .errors
                    .push(error!(self.span, "Expected lvalue not '{}'", self));
            }
        }
    }
}
