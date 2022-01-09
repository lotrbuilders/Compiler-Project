use super::ast::{ASTType, ASTTypeNode};
use super::r#type::TypeNode;
use super::{recovery::RecoveryStrategy, Parser};
use crate::expect;
use crate::token::TokenType;

impl<'a> Parser<'a> {
    pub(super) fn parse_declaration(&mut self) -> Result<ASTType, ()> {
        let base_type = self.parse_declaration_specifiers()?;
        //let base_type = self.check_declaration_specifiers(base_type);
        let declarator = self.parse_declarator()?;
        Ok(ASTType::combine(base_type, declarator))
    }

    // Parses all type qualifiers (const, int, void)
    // <declaration-specifiers> ::= <type-qualifier>+
    fn parse_declaration_specifiers(&mut self) -> Result<ASTType, ()> {
        let begin = self.peek_span();
        if self.peek().filter(Parser::is_type_qualifier) == None {
            self.expect_some()?;
            let span = self.peek_span();
            let token = self.peek_type().unwrap();
            self.errors.push(crate::error!(
                span,
                "Expected type qualifier in declaration, found {}",
                token
            ));
            self.recover(&RecoveryStrategy::Next);
            return Err(());
        }
        let mut result = Vec::<ASTTypeNode>::new();
        while let Some(token) = self.peek().filter(Parser::is_type_qualifier) {
            if let TokenType::Struct = self.peek_type().unwrap() {
                result.push(self.parse_struct()?);
            } else {
                self.next();
                result.push(token.into());
            }
        }
        let span = begin.to(&self.peek_span());
        Ok(ASTType::from_slice(&result, span))
    }

    // Parse a declarator optionally containing pointers and function
    // <declarator> ::= ('*')* name ( '(' <parameter-type-list>? ')' )?
    fn parse_declarator(&mut self) -> Result<ASTType, ()> {
        let begin = self.peek_span();
        let mut pointers = Vec::new();
        while let Some(TokenType::Asterisk) = self.peek_type() {
            self.next();
            pointers.push(ASTTypeNode::Simple(TypeNode::Pointer));
        }

        let mut result = Vec::<ASTTypeNode>::new();
        match self.peek_type() {
            Some(TokenType::LParenthesis) => {
                let inner = self.parse_braced('(', Parser::parse_declarator)?;
                result = inner.list
            }
            Some(TokenType::Ident(name)) => {
                self.next();
                result.push(ASTTypeNode::Name(name));
            }
            _ => (),
        }

        loop {
            match self.peek_type() {
                Some(TokenType::LParenthesis) => {
                    let arguments = self.parse_braced('(', Parser::parse_parameter_type_list)?;
                    result.push(ASTTypeNode::Function(arguments))
                }
                Some(TokenType::LSquare) => {
                    let expression = self.parse_braced('[', Parser::parse_conditional)?;
                    result.push(ASTTypeNode::Array(Box::new(expression)));
                }
                _ => break,
            }
        }

        result.append(&mut pointers);
        let span = begin.to(&self.peek_span());
        Ok(ASTType::from_slice(&result, span))
    }

    // Parse a list of paremeter declarations seperated by comma's
    // <parameter-type-list> ::= <declaration> ( ,<declaration> )*
    fn parse_parameter_type_list(&mut self) -> Result<Vec<ASTType>, ()> {
        let mut arguments = Vec::new();
        while let Some(true) = self.peek().as_ref().map(Parser::is_type_qualifier) {
            arguments.push(self.parse_declaration()?);
            if let Some(TokenType::RParenthesis) = self.peek_type() {
            } else {
                let _ = expect!(self, TokenType::Comma, RecoveryStrategy::Nothing);
            }
        }
        Ok(arguments)
    }
}
