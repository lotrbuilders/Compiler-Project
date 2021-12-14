use super::analysis::Analysis;
use super::SemanticAnalyzer;
use crate::error;
use crate::parser::ast::*;
use crate::parser::r#type::Type;
use crate::semantic_analysis::type_checking::check_arguments_function;

// The analysis for expressions
impl Analysis for Expression {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use ExpressionVariant::*;
        match &mut self.variant {
            ConstI(_) => {}

            Ident(name, symbol_number) => {
                if let Some(symbol) = analyzer.symbol_table.get(name) {
                    if Type::is_function(&symbol.symbol_type) {
                        log::error!(
                            "{} Currently unsupported function declaration in function",
                            self.span
                        );
                    }
                    self.ast_type = symbol.symbol_type.clone();
                    *symbol_number = symbol.number;
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
                left.analyze(analyzer);
                right.analyze(analyzer);
                left.analyze_lvalue(analyzer);
            }
        }
    }
}

impl Expression {
    fn analyze_lvalue(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use ExpressionVariant::*;
        match &mut self.variant {
            Ident(..) => (),
            _ => {
                analyzer
                    .errors
                    .push(error!(self.span, "Expected lvalue not '{}'", self));
            }
        }
    }
}
