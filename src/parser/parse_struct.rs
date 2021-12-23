use super::r#type::StructType;
use super::Type;
use super::{recovery::RecoveryStrategy, Parser, TypeNode};
use crate::error;
use crate::token::TokenType;

impl<'a> Parser<'a> {
    pub fn parse_struct(&mut self) -> Result<TypeNode, ()> {
        let begin = self.peek_span();
        self.next();

        let name = match self.peek_type() {
            Some(TokenType::Ident(name)) => {
                self.next();
                let _ = self.struct_table.try_insert(Some(&name));
                Some(name)
            }
            Some(TokenType::LBrace) => None,
            _ => {
                self.errors
                    .push(error!(begin, "Expected identifier or '{{'"));
                self.recover(&RecoveryStrategy::UpTo(';'));
                return Err(());
            }
        };

        //Add new struct
        let struct_definition = if let Some(TokenType::LBrace) = self.peek_type() {
            Some(self.parse_braced('{', Parser::parse_struct_declaration)?)
        } else {
            None
        };

        let index = match (name, struct_definition) {
            // struct {...}a;
            (None, Some(definition)) => {
                let index = self.struct_table.try_insert(None).unwrap();
                self.struct_table.qualify(index, definition);
                index
            }

            // struct b or struct b a;
            (Some(key), None) => self.struct_table.get_index(&key).unwrap(),

            // struct b{...} a;
            (Some(key), Some(definition)) => {
                let index = self.struct_table.get_index(&key).unwrap();
                let def = self.struct_table.get(&key).unwrap();

                // Check if struct has already been defined
                if def.is_qualified() {
                    let span = begin.to(&self.peek_span());
                    self.errors.push(error!(span, "Struct {} redefined", key));
                } else {
                    self.struct_table.qualify(index, definition);
                }

                index
            }

            _ => todo!(),
        };

        Ok(TypeNode::Struct(index))
    }

    fn parse_struct_declaration(&mut self) -> Result<StructType, ()> {
        let mut result = Vec::<(String, Type)>::new();
        let begin = self.peek_span();
        loop {
            if let Some(TokenType::RBrace) = self.peek_type() {
                break;
            }
            let decl = self.parse_declaration();
            self.expect_semicolon();
            if let Ok(decl) = decl {
                let span = begin.to(&self.peek_span());
                if let Some(name) = decl.get_name() {
                    let typ: Type = decl.remove_name().into();
                    if !typ.is_qualified(&self.struct_table) {
                        self.errors.push(error!(span,"Expected a qualified type within struct definition, but {} is not qualified",typ))
                    }
                    result.push((name, typ));
                } else {
                    self.errors.push(error!(
                        span,
                        "Expected name in member declaration of struct"
                    ))
                }
            }
        }
        Ok(StructType {
            members: Some(result),
        })
    }
}