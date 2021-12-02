use std::fmt::Display;

use crate::span::Span;
// Stores the specific type of a token and any associated values
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

    //Operators symbols
    Plus,
    Minus,
    Asterisk,
    Divide,
    Tilde,
    Exclamation,

    //Types with a value
    ConstI(u64),
    Ident(String),
}

// Stores the location and type of a lexed token
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Token {
    span: Span,
    token: TokenType,
}

// Convert a punctuation character or string into the assocated tokentype for readability
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

// Conversion of a characacter into the tokentype for punctuation characters
impl From<char> for TokenType {
    fn from(c: char) -> TokenType {
        use TokenType::*;
        match c {
            '{' => LBrace,
            '}' => RBrace,
            '(' => LParenthesis,
            ')' => RParenthesis,
            ';' => Semicolon,
            '+' => Plus,
            '-' => Minus,
            '*' => Asterisk,
            '/' => Divide,
            '~' => Tilde,
            '!' => Exclamation,
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

// Conversion of a string into the tokentype for multicharacter punctuation characters
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

// Display a token using std::fmt
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        match &self.token {
            Int => write!(f, "'int'"),

            Return => write!(f, "'return'"),

            LBrace => write!(f, "'{{'"),
            RBrace => write!(f, "'}}'"),
            LParenthesis => write!(f, "'('"),
            RParenthesis => write!(f, "')'"),
            Semicolon => write!(f, "';'"),

            Plus => write!(f, "'+'"),
            Minus => write!(f, "'-'"),
            Asterisk => write!(f, "'*'"),
            Divide => write!(f, "'/'"),
            Tilde => write!(f, "'~'"),
            Exclamation => write!(f, "'!'"),

            Ident(val) => write!(f, "'{}'", val),
            ConstI(val) => write!(f, "'{}'", val),
        }
    }
}
