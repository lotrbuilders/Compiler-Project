use std::fmt::Display;

use crate::span::Span;
// Stores the specific type of a token and any associated values
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TokenType {
    //Type keywords
    Char,
    Int,
    Long,
    Short,
    Struct,
    Void,

    //Control flow keywords
    If,
    Else,
    While,
    For,
    Do,
    Break,
    Continue,
    Return,

    //Keywords
    Sizeof,

    //Punctuation symbols
    LBrace,
    RBrace,
    LParenthesis,
    RParenthesis,
    Semicolon,
    LSquare,
    RSquare,

    //Operators symbols
    Assign,
    Plus,
    Minus,
    Asterisk,
    Divide,
    Tilde,
    Exclamation,
    Equal,
    Inequal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Question,
    Colon,
    LogicalOr,
    LogicalAnd,
    Or,
    And,
    Comma,
    Period,
    Arrow,

    //Types with a value
    ConstI(u64),
    Ident(String),
    CString(String),
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
            '[' => LSquare,
            ']' => RSquare,
            ';' => Semicolon,
            '=' => Assign,
            '+' => Plus,
            '-' => Minus,
            '*' => Asterisk,
            '/' => Divide,
            '~' => Tilde,
            '!' => Exclamation,
            '<' => Less,
            '>' => Greater,
            '?' => Question,
            ':' => Colon,
            '|' => Or,
            '&' => And,
            '.' => Period,
            ',' => Comma,
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
            "==" => Equal,
            "!=" => Inequal,
            "<=" => LessEqual,
            ">=" => GreaterEqual,
            "||" => LogicalOr,
            "&&" => LogicalAnd,
            "->" => Arrow,
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

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenType::*;
        match self {
            Char => write!(f, "'char'"),
            Int => write!(f, "'int'"),
            Long => write!(f, "'long'"),
            Short => write!(f, "'short'"),
            Struct => write!(f, "'struct'"),
            Void => write!(f, "'void'"),

            If => write!(f, "'if'"),
            Else => write!(f, "'else'"),
            While => write!(f, "'while'"),
            For => write!(f, "'for'"),
            Do => write!(f, "'do'"),
            Break => write!(f, "'break'"),
            Continue => write!(f, "'continue'"),
            Return => write!(f, "'return'"),

            Sizeof => write!(f, "'sizeof'"),

            LBrace => write!(f, "'{{'"),
            RBrace => write!(f, "'}}'"),
            LParenthesis => write!(f, "'('"),
            RParenthesis => write!(f, "')'"),
            LSquare => write!(f, "'['"),
            RSquare => write!(f, "']'"),
            Semicolon => write!(f, "';'"),

            Assign => write!(f, "'='"),
            Plus => write!(f, "'+'"),
            Minus => write!(f, "'-'"),
            Asterisk => write!(f, "'*'"),
            Divide => write!(f, "'/'"),
            Tilde => write!(f, "'~'"),
            Exclamation => write!(f, "'!'"),
            Equal => write!(f, "'=='"),
            Inequal => write!(f, "'!='"),
            Less => write!(f, "'<'"),
            LessEqual => write!(f, "'<='"),
            Greater => write!(f, "'>'"),
            GreaterEqual => write!(f, "'>='"),
            Question => write!(f, "'?'"),
            Colon => write!(f, "':'"),
            LogicalOr => write!(f, "'||'"),
            LogicalAnd => write!(f, "'&&'"),
            Or => write!(f, "'|'"),
            And => write!(f, "'&'"),
            Comma => write!(f, "','"),
            Period => write!(f, "'.'"),
            Arrow => write!(f, "'->'"),

            Ident(val) => write!(f, "'{}'", val),
            ConstI(val) => write!(f, "'{}'", val),
            CString(val) => write!(f, "\"{}\"", val),
        }
    }
}

// Display a token using std::fmt
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token())
    }
}
