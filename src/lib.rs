pub mod backend;
pub mod compiler;
pub mod driver;
mod error;
mod eval;
pub mod file_table;
pub mod ir;
pub mod lexer;
pub mod logger;
mod optimization;
pub mod options;
pub mod parser;
pub mod semantic_analysis;
pub mod table;
pub mod utility;

mod span;
mod token;
