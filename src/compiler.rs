use std::fs::read_to_string;

use colored::Colorize;

use crate::lexer::Lexer;
use crate::parser::Parser;

fn open(filename: String) -> Result<String, String> {
    let contents = read_to_string(filename.clone());
    match contents {
        Ok(contents) => Ok(contents),
        Err(_) => Err(format!(
            "{} {} ",
            "Error: Failed to open file".bright_red(),
            filename
        )),
    }
}

pub fn compile(filename: String, _output: String) -> Result<(), String> {
    let mut lexer = Lexer::new(&filename);
    let file = open(filename)?;
    let tokens = lexer.lex(&mut file.chars()).unwrap();
    let file_table = lexer.file_table();
    let mut parser = Parser::new(file_table);
    let (_ast, _parse_errors) = parser.parse(tokens);
    Ok(())
}
