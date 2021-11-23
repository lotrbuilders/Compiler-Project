use crate::span::Span;

#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TokenType {
    //Type keywords
    Int,

    //Statement keywords
    Return,

    //Punctuation symbols
    LBrace,
    RBrace,
    LParenthesis,
    RParenthesis,
    Semicolon,

    //Types with a value
    ConstI(u64),
    Ident(String),
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Token {
    span: Span,
    token: TokenType,
}

#[allow(dead_code)]
pub fn punct<T: Into<TokenType>>(input: T) -> TokenType {
    input.into()
}

#[allow(dead_code)]
impl Token {
    pub fn new(token: TokenType, span: Span) -> Token {
        Token { span, token }
    }
    pub fn span(&self) -> &Span {
        &self.span
    }
    pub fn token(&self) -> TokenType {
        self.token.clone()
    }
}

impl From<char> for TokenType {
    fn from(c: char) -> TokenType {
        use TokenType::*;
        match c {
            '{' => LBrace,
            '}' => RBrace,
            '(' => LParenthesis,
            ')' => RParenthesis,
            ';' => Semicolon,
            _ => {
                log::warn!(
                    "char to TokenType conversion with unimplemented character {}",
                    c
                );
                Semicolon
            }
        }
    }
}

impl From<&str> for TokenType {
    fn from(s: &str) -> TokenType {
        use TokenType::*;
        match s {
            _ => {
                log::warn!(
                    "&str to TokenType conversion with unimplemented string {}",
                    s
                );
                Semicolon
            }
        }
    }
}
