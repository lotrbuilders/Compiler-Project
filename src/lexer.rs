use crate::span::Span;
use crate::token;
use crate::token::Token;
use crate::token::TokenType;

pub struct Lexer {
    filenames: Vec<String>,
    line: u32,
    column: u32,
    offset: u32,
    last_char: Option<char>,
}

impl Lexer {
    pub fn new(filename: &String) -> Lexer {
        Lexer {
            filenames: vec![filename.clone()],
            line: 1,
            column: 1,
            offset: 0,
            last_char: None,
        }
    }
}

impl Lexer {
    fn here(&mut self) -> Span {
        Span::new(
            (self.filenames.len() - 1) as u32,
            self.line,
            self.column,
            self.offset,
            1,
        )
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

    pub fn lex<T: Iterator<Item = char>>(
        &mut self,
        input: &mut T,
    ) -> Result<Vec<Token>, Vec<String>> {
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
                ';' | '{' | '}' | '(' | ')' => {
                    self.next(input);
                    output.push(Token::new(token::punct(c), self.here()));
                }
                ' ' | '\t' | '\n' => {
                    self.next(input);
                }
                _ => {
                    self.next(input);
                    errors.push(format!("Todo"));
                }
            }
        }
        Err(Vec::new())
    }

    pub fn lex_identifier<T: Iterator<Item = char>>(&mut self, input: &mut T) -> Token {
        let start = self.here();
        let mut identifier = String::new();
        while let Some(c) = self.peek(input) {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => {
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
            "int" => Token::new(Int, span),
            "return" => Token::new(Return, span),
            _ => Token::new(Ident(identifier), span),
        }
    }

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
                Token::new(TokenType::ConstI(0), start.to(&self.here())),
                Err(format!("Todo")),
            ),
        }
    }
}
