pub mod ast;
pub mod ast_graph;
pub mod ast_print;
pub mod parse_delimiters;
pub mod r#type;

mod parse_declaration;
mod parse_expression;
mod parse_global;
mod parse_statement;
mod parse_struct;
mod recovery;

pub use self::parse_delimiters::*;
pub use self::r#type::{Type, TypeNode};
use self::recovery::RecoveryStrategy;
use crate::backend::Backend;
use crate::span::Span;
use crate::table::StructTable;
use crate::token::{Token, TokenType};
use crate::{error, expect};

#[allow(dead_code)]
pub struct Parser<'a> {
    errors: Vec<String>,
    tokens: Vec<Token>,
    struct_table: StructTable,
    backend: &'a dyn Backend,
    token_index: usize,
}

impl Parser<'_> {
    pub fn new<'a>(backend: &'a dyn Backend) -> Parser<'a> {
        Parser {
            errors: Vec::new(),
            tokens: Vec::new(),
            struct_table: StructTable::new(),
            backend,
            token_index: 0,
        }
    }

    pub fn get_struct_table(&mut self) -> StructTable {
        std::mem::replace(&mut self.struct_table, StructTable::new())
    }

    fn peek(&self) -> Option<Token> {
        match self.tokens.get(self.token_index) {
            Some(token) => Some(token.clone()),
            None => None,
        }
    }

    fn peek2(&self) -> Option<Token> {
        match self.tokens.get(self.token_index + 1) {
            Some(token) => Some(token.clone()),
            None => None,
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.token_index += 1;
        self.peek()
    }

    fn empty(&self) -> bool {
        self.token_index >= self.tokens.len()
    }

    fn peek_span(&mut self) -> Span {
        match self.peek() {
            Some(token) => token.span().clone(),
            None => self
                .tokens
                .last()
                .map(|token| token.span().clone())
                .unwrap_or(Span::new(0, 1, 1, 0, 1)),
        }
    }

    fn peek_type(&mut self) -> Option<TokenType> {
        match self.peek() {
            Some(token) => Some(token.token()),
            None => None,
        }
    }

    fn is_type_qualifier(token: &Token) -> bool {
        use TokenType::*;
        match token.token() {
            Char | Int | Long | Short | Struct | Void => true,
            _ => false,
        }
    }

    fn expect_some(&mut self) -> Result<Token, ()> {
        let token = self.peek();
        match token {
            Some(token) => Ok(token),
            None => {
                self.error_unexpected_eof();
                Err(())
            }
        }
    }

    // Expect a semicolon, but do not do anything if it faisl
    fn expect_semicolon(&mut self) {
        let _ = crate::expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);
    }

    // Standard error for cases where end of file was unexpectably hit
    fn error_unexpected_eof(&mut self) {
        let span = self.peek_span();
        self.errors.push(error!(span, "Unexpected end of file"));
    }

    // Takes a type of brace and a function that should be parsed within said brace
    // Inspired by syn::parse_braced
    fn parse_braced<F, T>(&mut self, c: char, f: F) -> Result<T, ()>
    where
        F: Fn(&mut Self) -> Result<T, ()>,
    {
        let (left, right) = recovery::get_braces(c);
        expect!(self, left, RecoveryStrategy::Nothing)?;
        let result = f(self);
        expect!(self, right, RecoveryStrategy::Nothing)?;

        result
    }
}

impl<'a> Parser<'a> {
    pub fn enter_scope(&mut self) {
        self.struct_table.enter_scope();
    }
    pub fn leave_scope(&mut self) {
        self.struct_table.leave_scope();
    }
}

// This macro looks at the incoming token
// Returns Ok(token) if it matches the $expected pattern
//                              or the given identifier
// Allows specifying a recovery strategy too allow for further parsing
#[allow(unused_macros)]
#[macro_export]
macro_rules! expect {
    ($self:ident, $expected: ident, $recover: expr) => {
        expect!($self,$expected,$recover,"Expected {} but found {}",$expected)
    };
    ($self:ident, $expected: pat, $recover: expr) => {
        expect!($self,$expected,$recover,"Unexpected token {}")
    };

    ($self:ident, $expected: ident, $recover: expr, $( $exp:expr ),*) => {
        match $self.peek() {
            None => {
                let loc = $self.peek_span();
                $self
                    .errors
                    .push(crate::error!(loc, "Unexpected end of file"));
                Err(())
            }
            Some(token) =>
                if token.token()==$expected {
                    $self.next();
                    Ok(token)
                }
                else {
                    log::debug!("Error from line {}",line!());
                    let loc = $self.peek_span();
                    $self.recover(&$recover);
                    $self
                        .errors
                        .push(crate::error!(loc, $($exp,)* token));
                    Err(())
                }

        }
    };

    ($self:ident, $expected: pat, $recover: expr, $( $exp:expr ),*) => {
        match $self.peek() {
            None => {
                let loc = $self.peek_span();
                $self
                    .errors
                    .push(crate::error!(loc, "Unexpected end of file"));
                Err(())
            }
            Some(token) => match token.token() {
                $expected => {
                    $self.next();
                    Ok(token)
                }
                _ => {
                    log::debug!("Error from line {}",line!());
                    let loc = $self.peek_span();
                    $self.recover(&$recover);
                    $self
                        .errors
                        .push(crate::error!(loc, $($exp,)* token));
                    Err(())
                }
            },
        }
    };


}
