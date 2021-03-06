use super::ast::*;
use super::{Parser, Type};

use crate::token::{Token, TokenType};

impl<'a> Parser<'a> {
    // The public parser function used by the compiler
    // Parses global declarations until the end of the file
    // <translation-unit> ::= <external-declaration>*
    pub fn parse(&mut self, tokens: Vec<Token>) -> (TranslationUnit, Result<(), Vec<String>>) {
        self.tokens = tokens;
        let mut global_declarations = Vec::<ExternalDeclaration>::new();
        while !self.empty() {
            match self.parse_external_declaration() {
                Ok(declaration) => global_declarations.push(declaration),
                Err(_) => (),
            }
        }
        let result = match self.errors.is_empty() {
            true => Ok(()),
            false => Err(self.errors.clone()),
        };
        (
            TranslationUnit {
                global_declarations,
            },
            result,
        )
    }

    // Parses a single extarnal declaration, which is either a function or a global
    // <external-declaration> ::= <declaration> (';'|<compound-statement>|'=' <const-expression> ';')
    pub fn parse_external_declaration(&mut self) -> Result<ExternalDeclaration, ()> {
        let begin = self.peek_span();
        let declaration = self.parse_declaration()?;
        let name = declaration.get_name();
        //let decl_type = declaration.remove_name();
        let function_body = if let Some(TokenType::LBrace) = self.peek_type() {
            let compound_statement = self.parse_compound_statement()?;
            Some(compound_statement)
        } else {
            None
        };
        let expression = if function_body.is_none() {
            if let Some(TokenType::Assign) = self.peek_type() {
                self.next();
                let expression = self.parse_conditional().unwrap_or_else(|_| Expression {
                    span: begin.clone(),
                    ast_type: Type::int(),
                    variant: ExpressionVariant::ConstI(0),
                });
                self.expect_semicolon();
                Some(expression)
            } else {
                self.expect_semicolon();
                None
            }
        } else {
            None
        };

        Ok(ExternalDeclaration {
            span: begin.to(&self.peek_span()),
            ast_type: declaration,
            decl_type: Type::int(),
            name,
            function_body,
            expression,
        })
    }
}
