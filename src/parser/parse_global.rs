use super::ast::*;
use super::{Parser, Type};
use crate::expect;
use crate::token::{Token, TokenType};

impl Parser {
    pub fn parse(&mut self, tokens: Vec<Token>) -> (TranslationUnit, Result<(), Vec<String>>) {
        self.tokens = tokens;
        let mut global_declarations = Vec::<ExternalDeclaration>::new();
        while !self.empty() {
            match self.parse_external_declaration() {
                Ok(declaration) => global_declarations.push(declaration),
                Err(_) => (),
            }
        }
        (
            TranslationUnit {
                global_declarations,
            },
            Ok(()),
        )
    }

    pub fn parse_external_declaration(&mut self) -> Result<ExternalDeclaration, ()> {
        let declaration = self.parse_declaration()?;
        if Type::is_function(&declaration) {
            if let Some(TokenType::LBrace) = self.peek_type() {
                let _compound_statement = self.parse_compound_statement()?;
            } else {
                expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing)?;
            }
        }
        Err(())
    }
}
