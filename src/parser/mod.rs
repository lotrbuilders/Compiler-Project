mod ast;
mod parse_declaration;
mod parse_expression;
mod parse_global;
mod parse_statement;
mod r#type;

pub use self::ast::*;
pub use self::parse_declaration::*;
pub use self::parse_expression::*;
pub use self::parse_global::*;
pub use self::parse_statement::*;
pub use self::r#type::Type;

pub struct Parser {
    file_table: Vec<String>,
    errors: Vec<String>,
}
