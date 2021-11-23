use std::fs::read_to_string;

use colored::Colorize;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic_analysis::SemanticAnalyzer;

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

    log::info!("Lexer started");
    let tokens = lexer.lex(&mut file.chars()).unwrap();
    let file_table = lexer.file_table();
    log::trace!(target: "lexer","Lexed tokens: {:?}", tokens);

    log::info!("Parser started");
    let mut parser = Parser::new(file_table.clone());
    let (mut ast, _parse_errors) = parser.parse(tokens);
    log::debug!("Parser result:\n{}", ast);

    log::info!("Analyzer started");
    let mut analyzer = SemanticAnalyzer::new(file_table);
    let _analysis_errors = analyzer.analyze(&mut ast);
    Ok(())
}
