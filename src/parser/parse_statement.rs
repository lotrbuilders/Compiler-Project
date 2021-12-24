use super::ast::*;
use super::r#type::*;
use super::recovery::RecoveryStrategy;
use super::Parser;
use crate::error;
use crate::expect;
use crate::token::TokenType;

impl<'a> Parser<'a> {
    // Compound statements can contain a lot of other statements.
    // All statements within the brace are parsed
    // <compound-statement> ::= '{' <statement>* '}'
    pub(super) fn parse_compound_statement(&mut self) -> Result<Vec<Statement>, ()> {
        let _ = expect!(
            self,
            TokenType::LBrace,
            RecoveryStrategy::or(RecoveryStrategy::Until(';'), RecoveryStrategy::Until('{'))
        );
        self.enter_scope();
        let mut result = Vec::<Statement>::new();
        loop {
            if let Some(TokenType::RBrace) = self.peek_type() {
                self.next();
                self.leave_scope();
                break;
            }
            if self.peek() == None {
                let loc = self.peek_span();
                self.errors
                    .push(error!(loc, "Expected }} before end of file"));
                return Err(());
            }
            let statement = self.parse_statement();
            if let Ok(statement) = statement {
                result.push(statement);
            }
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
                self.expect_semicolon();

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

                self.enter_scope();
                let statement = self.parse_statement();
                self.leave_scope();
                let statement = Box::new(statement?);

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
                let expression = if let Some(TokenType::Semicolon) = self.peek_type() {
                    self.next();
                    None
                } else {
                    let expression = self.parse_expression();
                    self.expect_semicolon();
                    //Expression is unwrapped here to first parse semicolon first
                    Some(expression?)
                };

                let span = begin.to(&self.peek_span());
                Ok(Statement::Return {
                    span,
                    expression,
                    ast_type: Type::empty(),
                })
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
                self.expect_semicolon();

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
        let ast_type = self.parse_declaration()?;
        let ident = ast_type.get_name();
        //let decl_type = decl_type.remove_name();

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
            decl_type: Type::empty(),
            ast_type,
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
                self.expect_semicolon();
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
