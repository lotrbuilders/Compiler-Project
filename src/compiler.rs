use std::fs::read_to_string;

use colored::Colorize;

use crate::lexer::Lexer;

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
    lexer.lex(&mut file.chars()).unwrap();
    Ok(())
}
