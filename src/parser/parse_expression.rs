use super::ast::*;
//use super::RecoveryStrategy;
use super::{Parser, Type};
use crate::expect;
use crate::token::TokenType;

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, ()> {
        let begin = self.peek_span();
        let constant = expect!(self, TokenType::ConstI(_), RecoveryStrategy::UpTo(';'))?;
        let value = match constant.token() {
            TokenType::ConstI(value) => value,
            _ => {
                log::warn!("parse_expression should only accept ConstI for now");
                0
            }
        };
        let span = begin.to(&constant.span());
        Ok(Expression {
            span,
            ast_type: vec![Type::Int],
            variant: ExpressionVariant::ConstI(value as i128),
        })
    }
}
