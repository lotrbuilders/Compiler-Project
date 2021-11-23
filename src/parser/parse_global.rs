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
        let begin = self.peek_span();
        let declaration = self.parse_declaration()?;
        let function_body = if Type::is_function(&declaration) {
            if let Some(TokenType::LBrace) = self.peek_type() {
                let compound_statement = self.parse_compound_statement()?;
                Some(compound_statement)
            } else {
                expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing)?;
                None
            }
        } else {
            None
        };
        let name = Type::get_name(&declaration).unwrap_or("name".to_string());
        Ok(ExternalDeclaration {
            span: begin.to(&self.peek_span()),
            ast_type: declaration,
            name,
            function_body,
        })
    }
}
