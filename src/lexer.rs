use crate::error;
use crate::file_table;
use crate::span::Span;
use crate::token;
use crate::token::Token;
use crate::token::TokenType;

// The Lexer is a mutuable structure keeping track of the current location in the source
pub struct Lexer {
    file_index: u32,
    line: u32,
    column: u32,
    offset: u32,
    last_char: Option<char>,
}

impl Lexer {
    pub fn new(filename: &String) -> Lexer {
        unsafe {
            file_table::reset();
        }
        file_table::add_sourcefile(filename);
        Lexer {
            file_index: 0,
            line: 1,
            column: 1,
            offset: 0,
            last_char: None,
        }
    }
}

impl Lexer {
    // Returns the current location of the lexer
    fn here(&mut self) -> Span {
        Span::new(self.file_index, self.line, self.column, self.offset, 1)
    }

    pub fn peek<T: Iterator<Item = char>>(&mut self, it: &mut T) -> Option<char> {
        if self.last_char.is_none() {
            self.last_char = it.next();
            // Keeping track of current character location and offset
            self.offset += 1;
            self.column += 1;
            if let Some('\n') = self.last_char {
                self.line += 1;
                self.column = 0;
            }
        }
        self.last_char
    }
    pub fn next<T: Iterator<Item = char>>(&mut self, it: &mut T) -> Option<char> {
        let result = self.peek(it);
        self.last_char = None;
        result
    }

    // General lexing function
    // More complex analysis like identifiers and numbers are split out
    // The lexer can work on any arbitrary iterator that returns characters
    pub fn lex<T: Iterator<Item = char>>(
        &mut self,
        input: &mut T,
    ) -> (Vec<Token>, Result<(), Vec<String>>) {
        let mut output = Vec::<Token>::new();
        let mut errors = Vec::<String>::new();
        while let Some(c) = self.peek(input).clone() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => output.push(self.lex_identifier(input)),
                '1'..='9' | '0' => match self.lex_number(input) {
                    (token, Ok(_)) => output.push(token),
                    (token, Err(err)) => {
                        output.push(token);
                        errors.push(err);
                    }
                },
                '\'' => match self.lex_char(input) {
                    (token, Ok(_)) => output.push(token),
                    (token, Err(err)) => {
                        output.push(token);
                        errors.push(err);
                    }
                },
                '"' => match self.lex_string(input) {
                    (token, Ok(_)) => output.push(token),
                    (token, Err(err)) => {
                        output.push(token);
                        errors.push(err);
                    }
                },
                ';' | '{' | '}' | '(' | ')' | '+' | '-' | '*' | '/' | '~' | '?' | ':' | ',' => {
                    self.next(input);
                    output.push(Token::new(token::punct(c), self.here()));
                }
                '=' | '!' | '<' | '>' => {
                    let begin = self.here();
                    self.next(input);
                    if let Some('=') = self.peek(input) {
                        self.next(input);
                        output.push(Token::new(
                            match c {
                                '=' => TokenType::Equal,
                                '!' => TokenType::Inequal,
                                '<' => TokenType::LessEqual,
                                '>' => TokenType::GreaterEqual,
                                _ => unreachable!(),
                            },
                            begin.to(&self.here()),
                        ));
                    } else {
                        output.push(Token::new(token::punct(c), begin));
                    }
                }
                '|' | '&' => {
                    let first_char = self.peek(input).unwrap();
                    let begin = self.here();
                    self.next(input);
                    let c = self.peek(input);
                    let span = begin.to(&self.here());
                    match (c, first_char) {
                        (Some('|'), '|') => {
                            output.push(Token::new(TokenType::LogicalOr, span));
                            self.next(input);
                        }
                        (Some('&'), '&') => {
                            output.push(Token::new(TokenType::LogicalAnd, span));
                            self.next(input);
                        }
                        _ => output.push(Token::new(token::punct(first_char), begin)),
                    }
                }

                ' ' | '\t' | '\n' | '\r' => {
                    self.next(input);
                }
                _ => {
                    self.next(input);
                    errors.push(crate::error!(self.here(), "Unknown character {}", c));
                }
            }
        }
        match errors.is_empty() {
            true => (output, Ok(())),
            false => (output, Err(errors)),
        }
    }

    // Lex an identifier or keyword
    pub fn lex_identifier<T: Iterator<Item = char>>(&mut self, input: &mut T) -> Token {
        let start = self.here();
        let mut identifier = String::new();
        while let Some(c) = self.peek(input) {
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0' | '1'..='9' => {
                    self.next(input);
                    identifier.push(c);
                }
                _ => {
                    break;
                }
            }
        }
        let span = start.to(&self.here());
        use TokenType::*;
        match identifier.as_str() {
            "char" => Token::new(Char, span),
            "int" => Token::new(Int, span),
            "if" => Token::new(If, span),
            "else" => Token::new(Else, span),
            "while" => Token::new(While, span),
            "for" => Token::new(For, span),
            "do" => Token::new(Do, span),
            "break" => Token::new(Break, span),
            "continue" => Token::new(Continue, span),
            "return" => Token::new(Return, span),
            _ => Token::new(Ident(identifier), span),
        }
    }

    // Lex a number
    // Currently only decimal numbers are implemented
    pub fn lex_number<T: Iterator<Item = char>>(
        &mut self,
        input: &mut T,
    ) -> (Token, Result<(), String>) {
        let start = self.here();
        let mut number = String::new();
        while let Some(c) = self.peek(input) {
            match c {
                '1'..='9' | '0' => {
                    self.next(input);
                    number.push(c);
                }
                _ => {
                    break;
                }
            }
        }
        let end = self.here();
        let span: Span = start.to(&end);
        match number.parse::<u64>() {
            Ok(number) => (Token::new(TokenType::ConstI(number), span), Ok(())),
            Err(_) => (
                Token::new(TokenType::ConstI(0), span.clone()),
                Err(error!(
                    span,
                    "number {} to big too fit in any integer", number
                )),
            ),
        }
    }

    fn lex_string<T: Iterator<Item = char>>(
        &mut self,
        input: &mut T,
    ) -> (Token, Result<(), String>) {
        let start = self.here();
        let mut errors = String::new();
        let mut string = String::new();
        self.next(input);

        while let Some(c) = self.peek(input) {
            if c == '"' {
                self.next(input);
                break;
            }
            let (c, err) = self.lex_single_char(input);
            string.push(c);
            if let Err(err) = err {
                errors.push_str(&err)
            }
        }

        let err = if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        };

        let span = start.to(&self.here());
        let token = Token::new(TokenType::CString(string), span.clone());
        match self.peek(input) {
            Some(_) => (token, err),
            None => (token, Err(error!(span, "Unexpected end of file"))),
        }
    }

    fn lex_char<T: Iterator<Item = char>>(&mut self, input: &mut T) -> (Token, Result<(), String>) {
        let start = self.here();
        self.next(input);
        let (c, err) = self.lex_single_char(input);
        let span = start.to(&self.here());
        let token = Token::new(TokenType::ConstI(c as u64), span.clone());
        match self.peek(input) {
            Some('\'') => {
                self.next(input);
                (token, err)
            }
            Some(c) => (
                token,
                Err(error!(
                    span,
                    "Expected ' after character constant, but found {}", c
                )),
            ),
            None => (
                token,
                Err(error!(
                    span,
                    "Expected ' after character constant, but found end of file"
                )),
            ),
        }
    }

    fn lex_single_char<T: Iterator<Item = char>>(
        &mut self,
        input: &mut T,
    ) -> (char, Result<(), String>) {
        let start = self.here();
        match self.next(input) {
            Some('\\') => match self.next(input) {
                Some('\'') => ('\'', Ok(())),
                Some('"') => ('"', Ok(())),
                Some('a') => ('\x07', Ok(())),
                Some('b') => ('\x08', Ok(())),
                Some('f') => ('\x0e', Ok(())),
                Some('n') => ('\n', Ok(())),
                Some('r') => ('\r', Ok(())),
                Some('t') => ('\t', Ok(())),
                Some('v') => ('\x0b', Ok(())),
                Some(c) if c != '\n' => (
                    '_',
                    Err(error!(
                        start,
                        "Expected an escape sequence in string/character, but found '\\{}'", c
                    )),
                ),
                _ => (
                    '_',
                    Err(error!(
                        start,
                        "Expected an escape sequence in string/character, but found end of file"
                    )),
                ),
            },
            //   \' \" \? \\ \a \b \f \n \r \t \v
            Some(c) if c.is_ascii() && c != '\n' => (c, Ok(())),
            Some(c) => (
                '_',
                Err(error!(
                    start,
                    "Expected an ascii character in string/character, but found {:?}", c
                )),
            ),
            None => (
                '_',
                Err(error!(
                    start,
                    "Expected an ascii character in string/character, but found end of file"
                )),
            ),
        }
    }
}
