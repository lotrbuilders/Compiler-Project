use super::ast::*;
use super::Parser;
use crate::error;
use crate::expect;
use crate::token::TokenType;

impl Parser {
    // Compound statements can contain a lot of other statements.
    // All statements within the brace are parsed
    // <compound-statement> ::= '{' <statement>* '}'
    pub(super) fn parse_compound_statement(&mut self) -> Result<Vec<Statement>, ()> {
        let _ = expect!(
            self,
            TokenType::LBrace,
            RecoveryStrategy::or(RecoveryStrategy::Until(';'), RecoveryStrategy::Until('{'))
        );
        let mut result = Vec::<Statement>::new();
        loop {
            if let Some(TokenType::RBrace) = self.peek_type() {
                self.next();
                break;
            }
            if self.peek() == None {
                let loc = self.peek_span();
                self.errors
                    .push(error!(loc, "Expected }} before end of file"));
                return Err(());
            }
            result.push(self.parse_statement()?);
        }
        Ok(result)
    }

    // Parsing statements
    // <statement> ::= return <expression> ';'
    fn parse_statement(&mut self) -> Result<Statement, ()> {
        let begin = self.peek_span();
        expect!(self, TokenType::Return, RecoveryStrategy::Until(';'))?;
        let expression = self.parse_expression();
        let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);

        //Expression is unwrapped here to first parse semicolon first
        let expression = expression?;
        let span = begin.to(&self.peek_span());
        Ok(Statement::Return { span, expression })
    }
}
