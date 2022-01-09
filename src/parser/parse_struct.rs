use super::ast::{ASTStruct, ASTType, ASTTypeNode};
use super::{recovery::RecoveryStrategy, Parser};
use crate::error;
use crate::token::TokenType;

impl<'a> Parser<'a> {
    pub fn parse_struct(&mut self) -> Result<ASTTypeNode, ()> {
        let begin = self.peek_span();
        self.next();

        let name = match self.peek_type() {
            Some(TokenType::Ident(name)) => {
                self.next();
                Some(name)
            }
            Some(TokenType::LBrace) => None,
            _ => {
                self.errors
                    .push(error!(begin, "Expected identifier or '{{'"));
                self.recover(&RecoveryStrategy::or(
                    RecoveryStrategy::UpTo(';'),
                    RecoveryStrategy::Until('{'),
                ));
                return Err(());
            }
        };

        //Add new struct
        let struct_definition = if let Some(TokenType::LBrace) = self.peek_type() {
            Some(self.parse_braced('{', Parser::parse_struct_declaration)?)
        } else {
            None
        };

        let ast_struct = Box::new(ASTStruct {
            name,
            members: struct_definition,
        });

        Ok(ASTTypeNode::Struct(ast_struct))
    }

    fn parse_struct_declaration(&mut self) -> Result<Vec<ASTType>, ()> {
        let mut result = Vec::<ASTType>::new();
        loop {
            if let Some(TokenType::RBrace) = self.peek_type() {
                break;
            }
            let decl = self.parse_declaration();
            if decl.is_ok() {
                result.push(decl.unwrap());
            }
            if let Some(TokenType::RBrace) = self.peek_type() {
                break;
            }
            self.expect_semicolon();
        }
        Ok(result)
    }
}
