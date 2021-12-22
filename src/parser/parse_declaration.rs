use super::ast::ExpressionVariant;
use super::r#type::TypeNode;
use super::{recovery::RecoveryStrategy, Parser, Type};
use crate::token::TokenType;
use crate::{error, expect};

impl<'a> Parser<'a> {
    pub(super) fn parse_declaration(&mut self) -> Result<Type, ()> {
        let base_type = self.parse_declaration_specifiers()?;
        let base_type = self.check_declaration_specifiers(base_type);
        let declarator = self.parse_declarator()?;
        Ok(Type::combine(base_type, declarator))
    }

    // Parses all type qualifiers (const, int, void)
    // <declaration-specifiers> ::= <type-qualifier>+
    fn parse_declaration_specifiers(&mut self) -> Result<Type, ()> {
        if self.peek().filter(Parser::is_type_qualifier) == None {
            self.expect_some()?;
            let span = self.peek_span();
            let token = self.peek_type().unwrap();
            self.errors.push(crate::error!(
                span,
                "Expected type qualifier in declaration, found {}",
                token
            ));
            self.next();
            return Err(());
        }
        let mut result = Vec::<TypeNode>::new();
        while let Some(token) = self.peek().filter(Parser::is_type_qualifier) {
            self.next();
            result.push(token.into());
        }
        Ok(result.into())
    }

    // Parse a declarator optionally containing pointers and function
    // <declarator> ::= ('*')* name ( '(' <parameter-type-list>? ')' )?
    fn parse_declarator(&mut self) -> Result<Type, ()> {
        let mut pointers = Vec::new();
        while let Some(TokenType::Asterisk) = self.peek_type() {
            self.next();
            pointers.push(TypeNode::Pointer);
        }

        let mut result = Vec::<TypeNode>::new();
        if let Some(TokenType::Ident(_)) = self.peek_type() {
            let name = self.peek().unwrap();
            self.next();
            result.push(name.into());
        }

        loop {
            match self.peek_type() {
                Some(TokenType::LParenthesis) => {
                    let arguments = self.parse_braced('(', Parser::parse_parameter_type_list)?;
                    result.push(TypeNode::Function(arguments))
                }
                Some(TokenType::LSquare) => {
                    let expression = self.parse_braced('[', Parser::parse_conditional)?;
                    let expression = expression.const_eval(self.backend);
                    let number = if let ExpressionVariant::ConstI(value) = &expression.variant {
                        *value
                    } else {
                        self.errors.push(error!(
                            expression.span,
                            "Expected a constant expression in array declaration"
                        ));
                        1
                    };
                    result.push(TypeNode::Array(number as usize));
                }
                _ => break,
            }
        }

        result.append(&mut pointers);
        Ok(result.into())
    }

    // Parse a list of paremeter declarations seperated by comma's
    // <parameter-type-list> ::= <declaration> ( ,<declaration> )*
    fn parse_parameter_type_list(&mut self) -> Result<Vec<Type>, ()> {
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

    fn check_declaration_specifiers(&mut self, typ: Type) -> Type {
        use TypeNode::*;
        let mut type_specifier = None;
        let mut int_seen = false;
        for node in &typ.nodes {
            match node {
                TypeNode::Int => {
                    if let Some(TypeNode::Char | TypeNode::Struct(..)) = type_specifier {
                        self.invalid_type(&typ);
                    } else if int_seen {
                        self.invalid_type(&typ);
                    } else {
                        if let None = type_specifier {
                            type_specifier = Some(Int)
                        }
                        int_seen = true;
                    }
                }
                t @ (TypeNode::Long | TypeNode::Short) => {
                    if let Some(TypeNode::Int) | None = type_specifier {
                        type_specifier = Some(t.clone());
                    } else {
                        self.invalid_type(&typ);
                    }
                }
                TypeNode::Struct(..) | TypeNode::Char => {
                    if let Some(_) = type_specifier {
                        self.invalid_type(&typ);
                    }
                    type_specifier = Some(node.clone());
                }

                TypeNode::Function(..)
                | TypeNode::Name(..)
                | TypeNode::Array(..)
                | TypeNode::Pointer => unreachable!(),
            }
        }
        vec![type_specifier.expect("failure to check for type specifer")].into()
    }
    fn invalid_type(&mut self, typ: &Type) {
        let span = self.peek_span();
        self.errors
            .push(error!(span, "Invalid type specifiers provided: {}", typ));
    }
}
