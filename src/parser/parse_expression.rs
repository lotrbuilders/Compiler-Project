use super::ast::*;
use super::r#type::TypeNode;
use super::{recovery::RecoveryStrategy, Parser, Type};
use crate::expect;
use crate::token::{Token, TokenType};

// Expression parsing is done using Pratt parsing
// Everything that is hard to parse, starting from cast expression is handwritten
impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, ()> {
        self.pratt_parse(0)
    }
    pub(super) fn parse_assignment(&mut self) -> Result<Expression, ()> {
        self.pratt_parse(2)
    }
    pub(super) fn parse_conditional(&mut self) -> Result<Expression, ()> {
        self.pratt_parse(4)
    }

    // <expression> ::= <unary-expression> | <expression> <bin-op> <expression>
    // <bin-op> ::= '+' | '-' | '*' | '/' | '==' | '!=' | '<' | '<=' | '>' | '>=' | '||' | '&&'
    fn pratt_parse(&mut self, min_bp: u8) -> Result<Expression, ()> {
        let mut left = self.parse_unary()?;
        while let Some(token) = is_binary_operator(self.peek()) {
            let (l_bp, r_bp) = binding_power(&token);
            if l_bp < min_bp {
                break;
            }
            self.next();

            if token.token() == TokenType::Question {
                let middle = self.parse_expression()?;
                expect!(self, TokenType::Colon, RecoveryStrategy::UpTo(';'))?;
                let right = self.pratt_parse(r_bp)?;
                left = new_ternary_expression(&token, left, middle, right);
            } else {
                let right = self.pratt_parse(r_bp)?;
                left = new_binary_expression(&token, left, right);
            }
        }
        Ok(left)
    }

    // <unary-expression> ::= (<unary-op>)* <postfix-expression>
    // <unary-op> ::= '+' | '-' | '~' | '!'
    fn parse_unary(&mut self) -> Result<Expression, ()> {
        use TokenType::*;
        let token = self.peek();
        let exp = match self.peek_type() {
            Some(Plus) | Some(Minus) | Some(Tilde) | Some(Exclamation) | Some(Asterisk)
            | Some(And) => {
                self.next();
                let exp = self.parse_unary()?;
                new_unary_expression(&token.unwrap(), exp)
            }
            _ => self.parse_postfix()?,
        };
        Ok(exp)
    }

    // <postfix-expression> ::= <primary-expression> ( <postfix-op> )*
    // <postfix-op> ::= '(' <argument-list> ')'
    fn parse_postfix(&mut self) -> Result<Expression, ()> {
        use TokenType::*;
        let begin = self.peek_span();
        let mut exp = self.parse_primary()?;
        loop {
            match self.peek_type() {
                Some(LSquare) => {
                    let token = self.peek().unwrap();
                    let right = self.parse_braced('[', Parser::parse_expression)?;
                    exp = new_binary_expression(&token, exp, right);
                }
                Some(LParenthesis) => {
                    let arguments = self.parse_braced('(', Parser::parse_argument_list)?;
                    let span = begin.to(&self.peek_span());
                    exp = Expression {
                        span,
                        ast_type: Type::empty(),
                        variant: ExpressionVariant::Function(Box::new(exp), arguments),
                    };
                }
                _ => break,
            }
        }
        Ok(exp)
    }

    // <primary-expression> ::= '(' <expression> ')'
    //                        | <integer-constant>
    //                        | <identifier>
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
                    ast_type: vec![TypeNode::Int].into(),
                    variant: ExpressionVariant::ConstI(value as i128),
                })
            }
            Some(TokenType::CString(string)) => {
                self.next();
                Ok(Expression {
                    span: begin,
                    ast_type: vec![TypeNode::Pointer, TypeNode::Char].into(),
                    variant: ExpressionVariant::CString(string),
                })
            }
            Some(TokenType::Ident(name)) => {
                self.next();
                Ok(Expression {
                    span: begin,
                    ast_type: Type::empty(),
                    variant: ExpressionVariant::Ident(name, 0, false),
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
                self.error_unexpected_eof();
                Err(())
            }
        }
    }

    // <argument-list> ::=  <assignment-expression>? (',' <assignment-expression>)*
    fn parse_argument_list(&mut self) -> Result<Vec<Expression>, ()> {
        let mut result = Vec::new();
        use TokenType::*;
        loop {
            match self.peek_type() {
                Some(RParenthesis) => break,
                _ => {
                    result.push(self.parse_assignment()?);
                    if let Some(RParenthesis) = self.peek_type() {
                        break;
                    } else {
                        expect!(self, Comma, RecoveryStrategy::Nothing)?;
                    }
                }
            }
        }
        Ok(result)
    }
}

// Generates the pratt binding power for left-to-right operands
fn left_associative(bp: u8) -> (u8, u8) {
    let bp = bp * 2 + 1;
    (bp, bp + 1)
}

// Generates the pratt binding power for right-to-left operands
fn right_associative(bp: u8) -> (u8, u8) {
    let bp = bp * 2 + 1;
    (bp + 1, bp)
}

// Gets the binding power of a binary or ternary expression for pratt parsing
fn binding_power(token: &Token) -> (u8, u8) {
    use TokenType::*;
    match token.token() {
        Comma => left_associative(0),
        Assign => right_associative(1),
        Question => left_associative(2),
        LogicalOr => left_associative(3),
        LogicalAnd => left_associative(4),
        Or => left_associative(5),
        And => left_associative(6),
        Equal | Inequal => left_associative(7),
        Less | LessEqual | Greater | GreaterEqual => left_associative(8),
        Plus | Minus => left_associative(10),
        Asterisk | Divide => left_associative(11),
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
            Plus | Minus | Asterisk | Divide | Less | LessEqual | Greater | GreaterEqual
            | Equal | Inequal | Assign | Question | LogicalOr | LogicalAnd | Or | And | Comma => {
                true
            }
            _ => false,
        }
    })
}

fn new_ternary_expression(
    token: &Token,
    left: Expression,
    middle: Expression,
    right: Expression,
) -> Expression {
    let span = token.span().clone();
    let cond = Box::new(left);
    let left = Box::new(middle);
    let right = Box::new(right);

    Expression {
        span,
        ast_type: Type::empty(),
        variant: ExpressionVariant::Ternary(cond, left, right),
    }
}

fn new_binary_expression(token: &Token, left: Expression, right: Expression) -> Expression {
    let span = token.span().clone();
    let left = Box::new(left);
    let right = Box::new(right);
    use ExpressionVariant::*;
    if let TokenType::Assign = token.token() {
        return Expression {
            span,
            ast_type: Type::empty(),
            variant: Assign(left, right),
        };
    }

    use BinaryExpressionType::*;
    let op = match token.token() {
        TokenType::Plus => Add,
        TokenType::Minus => Subtract,
        TokenType::Asterisk => Multiply,
        TokenType::Divide => Divide,
        TokenType::Equal => Equal,
        TokenType::Inequal => Inequal,
        TokenType::Less => Less,
        TokenType::LessEqual => LessEqual,
        TokenType::Greater => Greater,
        TokenType::GreaterEqual => GreaterEqual,
        TokenType::LogicalOr => LogOr,
        TokenType::LogicalAnd => LogAnd,
        TokenType::Or => BinOr,
        TokenType::And => BinAnd,
        TokenType::Comma => Comma,
        TokenType::LSquare => Index,
        _ => unreachable!(),
    };
    Expression {
        span,
        ast_type: Type::empty(),
        variant: Binary(op, left, right),
    }
}

fn new_unary_expression(token: &Token, exp: Expression) -> Expression {
    let span = token.span().clone();
    let exp = Box::new(exp);
    use UnaryExpressionType::*;
    let variant = match token.token() {
        TokenType::Plus => Identity,
        TokenType::Minus => Negate,
        TokenType::Tilde => BinNot,
        TokenType::Exclamation => LogNot,
        TokenType::Asterisk => Deref,
        TokenType::And => Address,
        _ => unreachable!(),
    };
    Expression {
        span,
        ast_type: Type::empty(),
        variant: ExpressionVariant::Unary(variant, exp),
    }
}
