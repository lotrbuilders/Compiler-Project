use crate::{
    error,
    token::{Token, TokenType},
};
use TokenType::*;

fn is_open_delimiter(token: TokenType) -> bool {
    matches!(token, LBrace | LParenthesis | LSquare)
}
fn to_closed_delimiter(open: TokenType) -> TokenType {
    match open {
        LBrace => RBrace,
        LParenthesis => RParenthesis,
        LSquare => RSquare,
        _ => unreachable!(),
    }
}

fn maybe_swapped(
    delim_stack: &mut Vec<TokenType>,
    delimiters: &[&Token],
    index: usize,
) -> Option<TokenType> {
    let current = delimiters[index].token();
    let current_stack = delim_stack.last().unwrap().clone();
    if delim_stack.len() <= 1 {
        return None;
    }

    let next = delimiters.get(index + 1);
    let next_stack = delim_stack.get(delim_stack.len() - 2).cloned();

    if let (Some(&next), Some(next_stack)) = (next, next_stack) {
        let next = next.token();
        if next == current_stack && current == next_stack {
            delim_stack.pop();
            delim_stack.pop();
            delim_stack.push(current_stack.clone());
            Some(next)
        } else {
            None
        }
    } else {
        None
    }
}

// Possible error combinations:
// { x
// x }
// { ( } )
// error

pub fn parse_delimiters(tokens: &[Token]) -> Result<(), Vec<String>> {
    let delimiters: Vec<_> = tokens
        .iter()
        .filter(|&t| {
            matches!(
                t.token(),
                LBrace | RBrace | LParenthesis | RParenthesis | LSquare | RSquare
            )
        })
        .collect();

    let mut errors = Vec::new();
    let mut delim_stack = Vec::new();

    for (i, &delimiter) in delimiters.iter().enumerate() {
        if is_open_delimiter(delimiter.token()) {
            delim_stack.push(to_closed_delimiter(delimiter.token()));
        } else {
            let typ = delimiter.token();
            if delim_stack.is_empty() {
                errors.push(error!(
                    delimiter.span(),
                    "Found {}, whilst all braces were already close", typ
                ));
            } else if *delim_stack.last().unwrap() == typ {
                delim_stack.pop();
            } else if let Some(next) = maybe_swapped(&mut delim_stack, &delimiters, i) {
                errors.push(error!(
                    delimiter.span(),
                    "{} was swapped with {}", typ, next
                ));
            } else {
                errors.push(error!(
                    delimiter.span(),
                    "Found unexpecting closing {}", typ
                ));
            }
        }
    }

    if !delim_stack.is_empty() {
        let string = delim_stack
            .into_iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>();
        let string: String = string.iter().flat_map(|s| s.chars()).collect();
        errors.push(error!(
            delimiters.last().unwrap().span(),
            "Missing closing delimiters: {}", string
        ));
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}
