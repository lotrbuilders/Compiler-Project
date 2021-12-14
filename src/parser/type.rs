use crate::token::{Token, TokenType};

// Type contains a component C Type used by something
// The Name if any should be the highes
// This is followed in order of dereferencing/calling
#[derive(Debug, Clone, PartialEq, Eq)]
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

pub fn type2string(typ: &[Type]) -> String {
    let mut result = String::new();
    if typ.is_empty() {
        return result;
    }
    for i in (0..=(typ.len() - 1)).rev() {
        use Type::*;
        match &typ[i] {
            Int => result.push_str("int "),
            Name(name) => result.push_str(&name),
            Function(arguments) => {
                //Extend later when functions are fully implemented
                result.push_str(&format!("{}(", type2string(&typ[0..i])));
                if let Some(arg) = arguments.get(0) {
                    result.push_str(&type2string(arg));
                }
                for arg in arguments.iter().skip(1) {
                    result.push_str(", ");
                    result.push_str(&type2string(arg));
                }
                result.push_str(")");
                break;
            }
        };
    }
    result
}
