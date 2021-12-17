use super::analysis::Analysis;
use super::type_class::TypeClass;
use super::SemanticAnalyzer;
use crate::error;
use crate::parser::{ast::*, Type};
use crate::semantic_analysis::type_checking::check_arguments_function;

// The analysis for expressions
impl Analysis for Expression {
    fn analyze(&mut self, analyzer: &mut SemanticAnalyzer) -> () {
        use ExpressionVariant::*;
        match &mut self.variant {
            ConstI(_) => {
                return;
            }

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
                return;
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
        self.ast_type = self.get_type(analyzer);
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

impl Expression {
    fn get_type(&mut self, analyzer: &mut SemanticAnalyzer) -> Type {
        use ExpressionVariant::*;

        match &mut self.variant {
            Ident(..) | ConstI(_) => unreachable!(),

            Function(func, _) => func
                .ast_type
                .get_return_type()
                .map(|t| t.into())
                .unwrap_or_else(|| {
                    analyzer
                        .errors
                        .push(error!(self.span, "Function call on non function"));
                    Type::int()
                }),

            Unary(UnaryExpressionType::Address, exp) => Type::pointer().append(&exp.ast_type),

            Unary(op, exp) => op.get_type(analyzer, exp),

            Binary(op, left, right) => op.get_type(analyzer, left, right),

            Ternary(cond, left, right) => {
                use TypeClass::*;
                analyzer.assert_in(&self.span, &cond.ast_type, Arithmetic);
                if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Pointer) {
                    analyzer.assert_compatible(&self.span, &left.ast_type, &right.ast_type);
                } else {
                    analyzer.assert_both_in(&self.span, &left.ast_type, &right.ast_type, Arithmetic)
                }
                left.ast_type.clone()
            }

            Assign(left, right) => {
                use TypeClass::*;
                if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Pointer) {
                    analyzer.assert_compatible(&self.span, &left.ast_type, &right.ast_type);
                } else {
                    analyzer.assert_both_in(
                        &self.span,
                        &left.ast_type,
                        &right.ast_type,
                        Arithmetic,
                    );
                }
                left.ast_type.clone()
            }
        }
    }
}

impl UnaryExpressionType {
    fn get_type(&self, analyzer: &mut SemanticAnalyzer, exp: &Expression) -> Type {
        use TypeClass::*;
        use UnaryExpressionType::*;
        {
            let span = &exp.span;
            let typ = &exp.ast_type;
            match self {
                Identity | Negate | BinNot | LogNot => {
                    analyzer.assert_in(span, typ, self.get_type_class());
                    exp.ast_type.clone()
                }

                Deref => {
                    analyzer.assert_in(span, typ, Pointer);
                    exp.ast_type.clone().deref()
                }
                Address => unreachable!(),
            }
        }
    }

    fn get_type_class(&self) -> TypeClass {
        use TypeClass::*;
        use UnaryExpressionType::*;
        match self {
            Identity | Negate => Arithmetic,

            BinNot => Integer,
            LogNot => Scalar,
            Deref | Address => unreachable!(),
        }
    }
}

impl BinaryExpressionType {
    fn get_type(
        &self,
        analyzer: &mut SemanticAnalyzer,
        left: &Expression,
        right: &Expression,
    ) -> Type {
        use BinaryExpressionType::*;
        use TypeClass::*;
        let span = left.span.to(&right.span);
        let span = &span;
        {
            match self {
                Add => {
                    if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Integer) {
                        left.ast_type.clone()
                    } else if left.ast_type.is_in(Integer) && right.ast_type.is_in(Pointer) {
                        right.ast_type.clone()
                    } else {
                        analyzer.assert_both_in(
                            span,
                            &left.ast_type,
                            &right.ast_type,
                            self.get_type_class(),
                        );
                        left.ast_type.clone()
                    }
                }
                Subtract => {
                    if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Integer) {
                        left.ast_type.clone()
                    } else if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Pointer) {
                        analyzer.assert_compatible(span, &left.ast_type, &right.ast_type);
                        right.ast_type.clone()
                    } else {
                        analyzer.assert_both_in(
                            span,
                            &left.ast_type,
                            &right.ast_type,
                            self.get_type_class(),
                        );
                        left.ast_type.clone()
                    }
                }

                Equal | Inequal | Less | LessEqual | Greater | GreaterEqual => {
                    if left.ast_type.is_in(Pointer) && right.ast_type.is_in(Pointer) {
                        analyzer.assert_compatible(span, &left.ast_type, &right.ast_type);
                        Type::int()
                    } else {
                        analyzer.assert_both_in(
                            span,
                            &left.ast_type,
                            &right.ast_type,
                            self.get_type_class(),
                        );
                        left.ast_type.clone()
                    }
                }
                Multiply | Divide | BinOr | BinAnd | LogOr | LogAnd => {
                    analyzer.assert_both_in(
                        span,
                        &left.ast_type,
                        &right.ast_type,
                        self.get_type_class(),
                    );
                    left.ast_type.clone()
                }
                Comma => right.ast_type.clone(),
            }
        }
    }

    pub fn get_type_class(&self) -> TypeClass {
        use BinaryExpressionType::*;
        use TypeClass::*;
        {
            match self {
                Add | Subtract | Multiply | Divide | Equal | Inequal | Less | LessEqual
                | Greater | GreaterEqual => Arithmetic,
                BinOr | BinAnd => Integer,
                LogOr | LogAnd => Scalar,
                Comma => unreachable!(),
            }
        }
    }
}
