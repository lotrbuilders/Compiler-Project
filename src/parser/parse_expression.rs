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
        let mut left = self.parse_unary()?;
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

    fn parse_unary(&mut self) -> Result<Expression, ()> {
        use TokenType::*;
        let token = self.peek();
        let exp = match self.peek_type() {
            Some(Plus) | Some(Minus) | Some(Tilde) | Some(Exclamation) => {
                self.next();
                let exp = self.parse_unary()?;
                new_unary_expression(&token.unwrap(), exp)
            }
            _ => self.parse_primary()?,
        };
        Ok(exp)
    }

    fn parse_primary(&mut self) -> Result<Expression, ()> {
        let begin = self.peek_span();
        match self.peek_type() {
            Some(TokenType::LParenthesis) => {
                self.next();
                let expr = self.parse_expression();
                let _ = expect!(
                    self,
                    TokenType::RParenthesis,
                    RecoveryStrategy::or(RecoveryStrategy::Until(')'), RecoveryStrategy::UpTo(';'))
                );
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
            Some(TokenType::Ident(name)) => {
                self.next();
                Ok(Expression {
                    span: begin,
                    ast_type: Vec::new(),
                    variant: ExpressionVariant::Ident(name, 0),
                })
            }
            Some(_) => {
                self.errors.push(crate::error!(
                    begin,
                    "Expected expression. Found {}",
                    self.peek().unwrap()
                ));
                self.recover(&RecoveryStrategy::or(
                    RecoveryStrategy::UpTo(')'),
                    RecoveryStrategy::UpTo(';'),
                ));
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

fn right_associative(bp: u8) -> (u8, u8) {
    let bp = bp * 2 + 1;
    (bp + 1, bp)
}

fn binding_power(token: &Token) -> (u8, u8) {
    use TokenType::*;
    match token.token() {
        Assign => right_associative(1),
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
            Plus | Minus | Asterisk | Divide | Assign => true,
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
        TokenType::Assign => Assign(left, right),
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

fn new_unary_expression(token: &Token, exp: Expression) -> Expression {
    let span = token.span().clone();
    let exp = Box::new(exp);
    use ExpressionVariant::*;
    let variant = match token.token() {
        TokenType::Plus => Identity(exp),
        TokenType::Minus => Negate(exp),
        TokenType::Tilde => BinNot(exp),
        TokenType::Exclamation => LogNot(exp),
        _ => unreachable!(),
    };
    Expression {
        span,
        ast_type: Vec::new(),
        variant,
    }
}
