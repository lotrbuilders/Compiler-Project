use super::r#type::TypeNode;
use super::{recovery::RecoveryStrategy, Parser, Type};
use crate::token::TokenType;
use crate::{error, expect};

impl Parser {
    pub(super) fn parse_declaration(&mut self) -> Result<Type, ()> {
        let base_type = self.parse_declaration_specifiers()?;
        self.check_declaration_specifiers(&base_type);
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
        let name = expect!(
            self,
            TokenType::Ident(_),
            RecoveryStrategy::or(
                RecoveryStrategy::Until(';'),
                RecoveryStrategy::UntilBraced('{')
            )
        )?;
        result.push(name.into());
        if let Some(TokenType::LParenthesis) = self.peek_type() {
            let arguments = self.parse_braced('(', Parser::parse_parameter_type_list)?;
            result.push(TypeNode::Function(arguments))
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

    fn check_declaration_specifiers(&mut self, typ: &Type) {
        let mut type_specifier_count = 0;
        for node in &typ.nodes {
            match node {
                TypeNode::Char | TypeNode::Int | TypeNode::Long | TypeNode::Short => {
                    type_specifier_count += 1
                }
                TypeNode::Function(..) | TypeNode::Name(..) | TypeNode::Pointer => unreachable!(),
            }
        }
        let span = self.peek_span();
        if type_specifier_count != 1 {
            self.errors
                .push(error!(span, "Multiple type specifiers provided in {}", typ));
        }
    }
}
