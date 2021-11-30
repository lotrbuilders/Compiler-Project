use crate::token::{Token, TokenType};

// Type contains a component C Type used by something
// The Name if any should be the highes
// This is followed in order of dereferencing/calling
#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Name(String),
    Function(Vec<Vec<Type>>),
}

impl Type {
    pub fn is_function(input: &Vec<Type>) -> bool {
        match input.get(1) {
            Some(Type::Function(_)) => true,
            _ => false,
        }
    }
    pub fn get_name(input: &Vec<Type>) -> Option<String> {
        match input.get(0) {
            Some(Type::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }
    // Function works under current definition of the types, but might need to be processed further when more types are introduced
    pub fn combine(mut base_type: Vec<Type>, mut declarator: Vec<Type>) -> Vec<Type> {
        declarator.append(&mut base_type);
        declarator
    }
}

impl From<Token> for Type {
    fn from(token: Token) -> Type {
        use TokenType::*;
        match token.token() {
            Int => Type::Int,
            Ident(name) => Type::Name(name),
            _ => {
                log::error!("From<Token> for Type called on unqualified type");
                std::process::exit(1);
            }
        }
    }
}
