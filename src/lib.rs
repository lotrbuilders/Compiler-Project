pub mod backend;
pub mod compiler;
pub mod driver;
mod error;
mod eval;
pub mod file_table;
pub mod lexer;
pub mod logger;
pub mod options;
pub mod parser;
pub mod semantic_analysis;
pub mod table;

mod span;
mod token;
