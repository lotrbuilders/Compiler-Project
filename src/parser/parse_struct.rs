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
                /*if !self.struct_table.contains(&name) {
                    let _ = self.struct_table.try_insert(Some(&name));
                }*/
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

        let ast_struct = Box::new(ASTStruct {
            name,
            members: struct_definition,
        });

        /*let index = match (name, struct_definition) {
            // struct {...}a;
            (None, Some(definition)) => {
                let index = self.struct_table.try_insert(None).unwrap();
                self.struct_table.qualify(self.backend, index, definition);
                index
            }

            // struct b or struct b a;
            (Some(key), None) => self.struct_table.get_index(&key).unwrap(),

            // struct b{...} a;
            (Some(key), Some(mut definition)) => {
                let mut index = self.struct_table.get_index(&key).unwrap();
                definition.name = Some(key.clone());
                let def = self.struct_table.get(&key).unwrap();

                // Check if struct has already been defined
                if def.is_qualified() {
                    if let Ok(i) = self.struct_table.try_insert(Some(&key)) {
                        index = i;
                    } else {
                        let span = begin.to(&self.peek_span());
                        self.errors.push(error!(span, "Struct {} redefined", key));
                    }
                }
                self.struct_table.qualify(self.backend, index, definition);

                index
            }

            _ => todo!(),
        };*/

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
            /*if let Ok(decl) = decl {
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
            }*/
        }
        Ok(result)
    }
}
