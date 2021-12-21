use super::ast::*;
use super::r#type::*;
use super::recovery::RecoveryStrategy;
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
    //               | ';'
    //               | <expression> ';'
    //               | <declaration> ('=' <expression>)? ';'
    //               | break ';'
    //               | continue ';'
    //               | if '(' <expression< ')' <statement> (else <statement)?
    //               | do <statement> while '(' <expression> ')' ';'
    //               | while '(' <expression> ')' <statement>
    //               | for '( <statement> <expression>? ';' <expression>? ')'
    //               | | '{' <statement>* '}'
    fn parse_statement(&mut self) -> Result<Statement, ()> {
        let begin = self.peek_span();
        use TokenType::*;
        match self.peek_type() {
            Some(_) if Parser::is_type_qualifier(&self.peek().unwrap()) => {
                self.parse_local_declaration()
            }

            Some(Break) => {
                self.next();
                self.expect_semicolon();
                Ok(Statement::Break { span: begin })
            }

            Some(Continue) => {
                self.next();
                self.expect_semicolon();
                Ok(Statement::Continue { span: begin })
            }

            Some(Do) => {
                self.next();
                let statement = Box::new(self.parse_statement()?);
                expect!(self, TokenType::While, RecoveryStrategy::Nothing)?;
                let expression = self.parse_braced('(', Parser::parse_expression)?;
                let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);

                let span = begin.to(&self.peek_span());
                Ok(Statement::While {
                    span,
                    expression,
                    statement,
                    do_while: true,
                })
            }

            Some(If) => {
                self.next();
                let expression = self.parse_braced('(', Parser::parse_expression)?;
                let statement = Box::new(self.parse_statement()?);
                let else_statement = if let Some(Else) = self.peek_type() {
                    self.next();
                    Some(Box::new(self.parse_statement()?))
                } else {
                    None
                };

                let span = begin.to(&self.peek_span());
                Ok(Statement::If {
                    span,
                    expression,
                    statement,
                    else_statement,
                })
            }

            Some(For) => {
                self.next();
                let (init, condition, expression) =
                    self.parse_braced('(', Parser::parse_for_clause)?;
                let statement = Box::new(self.parse_statement()?);

                let span = begin.to(&self.peek_span());
                Ok(Statement::For {
                    span,
                    init,
                    condition,
                    expression,
                    statement,
                })
            }

            Some(Return) => {
                self.next();
                let expression = self.parse_expression();
                let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);

                //Expression is unwrapped here to first parse semicolon first
                let expression = expression?;
                let span = begin.to(&self.peek_span());
                Ok(Statement::Return { span, expression })
            }

            Some(Semicolon) => {
                self.next();
                Ok(Statement::Empty(begin))
            }

            Some(While) => {
                self.next();
                let expression = self.parse_braced('(', Parser::parse_expression)?;
                let statement = Box::new(self.parse_statement()?);

                let span = begin.to(&self.peek_span());
                Ok(Statement::While {
                    span,
                    expression,
                    statement,
                    do_while: false,
                })
            }

            Some(LBrace) => {
                let statements = self.parse_compound_statement()?;
                let span = begin.to(&self.peek_span());
                Ok(Statement::Compound { span, statements })
            }

            Some(_) => {
                let expression = self.parse_expression();
                let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);

                //Expression is unwrapped here to first parse semicolon first
                let expression = expression?;
                let span = begin.to(&self.peek_span());
                Ok(Statement::Expression { span, expression })
            }

            None => Err(()),
        }
    }

    fn parse_local_declaration(&mut self) -> Result<Statement, ()> {
        let begin = self.peek_span();
        let decl_type = self.parse_declaration()?;
        let ident = Type::get_name(&decl_type).unwrap_or_else(|| {
            let span = begin.to(&self.peek_span());
            self.errors
                .push(error!(span, "Missing identifier in declaration"));
            "name".to_string()
        });
        let decl_type = decl_type.remove_name();

        let init = if let Some(TokenType::Assign) = self.peek_type() {
            self.next();
            match self.parse_expression() {
                Ok(init) => Some(init),
                Err(_) => None,
            }
        } else {
            None
        };

        let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);

        Ok(Statement::Declaration {
            span: begin.to(&self.peek_span()),
            ident,
            decl_type,
            init,
        })
    }

    fn parse_for_clause(
        &mut self,
    ) -> Result<
        (
            Option<Box<Statement>>,
            Option<Box<Expression>>,
            Option<Box<Expression>>,
        ),
        (),
    > {
        let init = match self.peek_type() {
            Some(TokenType::Semicolon) => {
                self.next();
                None
            }
            Some(_) => Some(Box::new(self.parse_statement()?)),

            None => {
                self.error_unexpected_eof();
                return Err(());
            }
        };

        let condition = match self.peek_type() {
            Some(TokenType::Semicolon) => {
                self.next();
                None
            }
            Some(_) => {
                let condition = self.parse_expression();
                let _ = expect!(self, TokenType::Semicolon, RecoveryStrategy::Nothing);
                Some(Box::new(condition?))
            }

            None => {
                self.error_unexpected_eof();
                return Err(());
            }
        };

        let expression = match self.peek_type() {
            Some(TokenType::RParenthesis) => None,
            Some(_) => Some(Box::new(self.parse_expression()?)),

            None => {
                self.error_unexpected_eof();
                return Err(());
            }
        };

        Ok((init, condition, expression))
    }
}
