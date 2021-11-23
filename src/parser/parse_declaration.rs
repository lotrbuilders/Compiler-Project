//use super::ast::*;
//use super::RecoveryStrategy;
use super::{Parser, Type};
use crate::expect;
//use crate::token;
use crate::token::TokenType;

impl Parser {
    pub(super) fn parse_declaration(&mut self) -> Result<Vec<Type>, ()> {
        let base_type = self.parse_declaration_specifiers()?;
        let declarator = self.parse_declarator()?;
        Ok(Type::combine(base_type, declarator))
    }

    fn parse_declaration_specifiers(&mut self) -> Result<Vec<Type>, ()> {
        if self.peek().filter(Parser::is_type_qualifier) == None {
            todo!();
            //return Err(());
        }
        let mut result = Vec::<Type>::new();
        while let Some(token) = self.peek().filter(Parser::is_type_qualifier) {
            self.next();
            result.push(token.into());
        }
        Ok(result)
    }

    fn parse_declarator(&mut self) -> Result<Vec<Type>, ()> {
        let mut result = Vec::<Type>::new();
        let name = expect!(
            self,
            TokenType::Ident(_),
            RecoveryStrategy::or(
                RecoveryStrategy::Until(';'),
                RecoveryStrategy::UntilBraced('{')
            )
        )?;
        result.push(name.into());
        if let Some(TokenType::LParenthesis) = self.peek().map(|token| token.token()) {
            self.next();
            expect!(
                self,
                TokenType::RParenthesis,
                RecoveryStrategy::or(
                    RecoveryStrategy::Until(')'), // Might be better to be UntilBraced later, but should not be a problem in logical cases
                    RecoveryStrategy::or(
                        RecoveryStrategy::Until(';'),
                        RecoveryStrategy::UntilBraced('{')
                    )
                )
            )?;
            result.push(Type::Function(Vec::new()))
        }
        Ok(result)
    }
}
