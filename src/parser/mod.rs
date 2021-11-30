pub mod ast;
pub mod ast_graph;
pub mod ast_print;
mod parse_declaration;
mod parse_expression;
mod parse_global;
mod parse_statement;
mod recovery;
pub mod r#type;

pub use self::r#type::Type;
use crate::span::Span;
use crate::token::{Token, TokenType};

#[allow(dead_code)]
pub struct Parser {
    errors: Vec<String>,
    tokens: Vec<Token>,
    token_index: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            errors: Vec::new(),
            tokens: Vec::new(),
            token_index: 0,
        }
    }

    fn peek(&self) -> Option<Token> {
        match self.tokens.get(self.token_index) {
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
            Int => true,
            _ => false,
        }
    }
}

// This macro looks at the incoming token
// Returns Ok(token) if it matches the $expected pattenn
// Allows specifying a recovery strategy too allow for further parsing
#[allow(unused_macros)]
#[macro_export]
macro_rules! expect {
    ($self:ident, $expected: pat, $recover: expr) => {
        match $self.peek() {
            None => {
                //Error code
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
                    //Error code
                    //Recovery
                    let loc = $self.peek_span();
                    $self
                        .errors
                        .push(crate::error!(loc, "Unexpected token {}", token));
                    Err(())
                }
            },
        }
    };
}
