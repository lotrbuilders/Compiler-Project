use super::ast::*;
use super::{recovery::RecoveryStrategy, Parser, Type};
use crate::expect;
use crate::token::{Token, TokenType};

// Expression parsing is done using Pratt parsing(unimplemented)
// Everything that is hard to parse, starting from cast expression is handwritten
impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, ()> {
        self.pratt_parse(0)
    }

    // <expression> ::= <primary-expression> | <expression> <bin-op> <expression>
    // <bin-op> ::= '+' | '-' | '*' | '/'
    fn pratt_parse(&mut self, min_bp: u8) -> Result<Expression, ()> {
        let mut left = self.parse_primary()?;
        while let Some(token) = is_binary_operator(self.peek()) {
            let (l_bp, r_bp) = binding_power(&token);
            if l_bp < min_bp {
                break;
            }
            self.next();

            let right = self.pratt_parse(r_bp)?;
            left = new_binary_expression(&token, left, right);
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, ()> {
        let begin = self.peek_span();
        match self.peek().map(|t| t.token()) {
            Some(TokenType::LParenthesis) => {
                self.next();
                let expr = self.parse_expression();
                let _ = expect!(
                    self,
                    TokenType::RParenthesis,
                    RecoveryStrategy::or(RecoveryStrategy::Until(')'), RecoveryStrategy::UpTo(';'))
                );
                //let span=begin.to(self.peek_span());
                expr
            }
            Some(TokenType::ConstI(value)) => {
                self.next();
                Ok(Expression {
                    span: begin,
                    ast_type: vec![Type::Int],
                    variant: ExpressionVariant::ConstI(value as i128),
                })
            }
            Some(t) => {
                log::info!("Unexpected token {:?}", t);
                Err(())
            }
            None => {
                self.errors
                    .push(crate::error!(begin, "Unexpected end of file"));
                Err(())
            }
        }
    }
}

fn left_associative(bp: u8) -> (u8, u8) {
    let bp = bp * 2 + 1;
    (bp, bp + 1)
}

fn binding_power(token: &Token) -> (u8, u8) {
    use TokenType::*;
    match token.token() {
        Plus | Minus => left_associative(9),
        Asterisk | Divide => left_associative(10),
        _ => {
            log::error!("Binding power called on unsupported token {}", token);
            left_associative(0)
        }
    }
}

fn is_binary_operator(token: Option<Token>) -> Option<Token> {
    token.filter(|t| {
        use TokenType::*;
        match t.token() {
            Plus | Minus | Asterisk | Divide => true,
            _ => false,
        }
    })
}

fn new_binary_expression(token: &Token, left: Expression, right: Expression) -> Expression {
    let span = token.span().clone();
    let left = Box::new(left);
    let right = Box::new(right);
    use ExpressionVariant::*;
    let variant = match token.token() {
        TokenType::Plus => Add(left, right),
        TokenType::Minus => Subtract(left, right),
        TokenType::Asterisk => Multiply(left, right),
        TokenType::Divide => Divide(left, right),
        _ => unreachable!(),
    };
    Expression {
        span,
        ast_type: Vec::new(),
        variant,
    }
}
