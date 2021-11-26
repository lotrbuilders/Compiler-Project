use std::fs::read_to_string;

use colored::Colorize;

use crate::backend;
use crate::eval::evaluate;
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
    let (mut ast, parse_errors) = parser.parse(tokens);
    log::debug!("Parser result:\n{}", ast);
    let _ = crate::parser::ast_graph::print_graph("graph.gv", &ast);

    log::info!("Analyzer started");
    let mut analyzer = SemanticAnalyzer::new(file_table);
    let analysis_errors = analyzer.analyze(&mut ast);

    if parse_errors.is_err() || analysis_errors.is_err() {
        log::info!("Exited due to errors");
        return Err("Error in parsing or analysis".to_string());
    }

    log::info!("Evaluation started");
    log::debug!("\n{}", ast);
    let ir_functions = evaluate(&ast);
    for ir in &ir_functions {
        log::debug!("Evaluation result:\n{}", ir);
    }

    log::info!("Started the backend");
    log::info!("Using backend amd64");
    backend::generate_code(ir_functions, "amd64".to_string()).expect("Unsupported backend");
    Ok(())
}
