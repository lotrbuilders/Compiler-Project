use std::fmt::Display;

use super::Parser;
use crate::token::{punct, Token, TokenType};

/* The RecoveryStrategy is used to specify how the parser might recover from an error
** UpTo         - Remove all tokens up to the first occurance of this character
** Until        - Remove all tokens until and including the first occurance of this character
** UntilBrace   - Remove all all tokens until the first occurance of this character and the entire braced block,
**                Takes into account possible sub-blocks.
**                Allows specifying the end token, which parses as if the block has already started
** Or           - Use either of the two recovery strategies(possibly recursive)
** Nothing      - Do not actively recover(For use by expect)
*/
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RecoveryStrategy {
    UpTo(char),
    Until(char),
    UntilBraced(char),
    Or(Box<RecoveryStrategy>, Box<RecoveryStrategy>),
    Nothing,
}

#[allow(dead_code)]
impl RecoveryStrategy {
    pub fn or(a: RecoveryStrategy, b: RecoveryStrategy) -> RecoveryStrategy {
        RecoveryStrategy::Or(Box::new(a), Box::new(b))
    }
}

impl Parser {
    pub(super) fn recover(&mut self, strategy: &RecoveryStrategy) {
        log::debug!("Recovering from parsing error");
        log::debug!("Strategy: {}", strategy);
        while let Some(token) = self.peek() {
            if self.try_recover(&token, strategy) {
                log::debug!("Succesfully recovered");
                break;
            }
            self.next();
        }
    }

    fn try_recover(&mut self, token: &Token, strategy: &RecoveryStrategy) -> bool {
        match strategy {
            &RecoveryStrategy::UpTo(c) => token.token() == punct(c),
            &RecoveryStrategy::Until(c) => {
                let result = token.token() == punct(c);
                if result {
                    self.next();
                }
                result
            }
            &RecoveryStrategy::UntilBraced(c) => {
                let result =
                    (token.token() == punct(c)) || (token.token() == punct(to_open_brace(c)));
                if result {
                    if !is_closed_brace(c) {
                        self.next();
                    }
                    self.recover_braced(c);
                }
                result
            }
            RecoveryStrategy::Or(left, right) => {
                self.try_recover(token, &**left) || self.try_recover(token, &**right)
            }
            RecoveryStrategy::Nothing => true,
        }
    }

    fn recover_braced(&mut self, c: char) {
        let (left, right) = get_braces(c);
        let mut counter = 1;
        while let Some(token) = self.next() {
            let token = token.token();
            match token {
                _ if left == token => counter += 1,
                _ if right == token => {
                    counter -= 1;
                    if counter == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn get_braces(c: char) -> (TokenType, TokenType) {
    use TokenType::*;
    match c {
        '{' | '}' => (LBrace, RBrace),
        '(' | ')' => (LParenthesis, RParenthesis),
        '[' | ']' => (LSquare, RSquare),
        _ => {
            log::error!("Brace recovery on non brace character {}", c);
            (LBrace, RBrace)
        }
    }
}

fn is_closed_brace(c: char) -> bool {
    match c {
        '}' | ']' | ')' => true,
        _ => false,
    }
}

fn to_open_brace(c: char) -> char {
    match c {
        '}' => '{',
        ']' => '[',
        ')' => '(',
        _ => c,
    }
}

impl Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryStrategy::UpTo(c) => write!(f, "up to {}", c),
            RecoveryStrategy::Until(c) => write!(f, "until {}", c),
            RecoveryStrategy::UntilBraced(c) => write!(f, "until braced {}", c),
            RecoveryStrategy::Or(left, right) => write!(f, "{} or {}", left, right),
            RecoveryStrategy::Nothing => write!(f, "nothing"),
        }
    }
}
