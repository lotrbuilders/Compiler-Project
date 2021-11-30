use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;

use colored::Colorize;

use crate::backend;
use crate::eval::evaluate;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic_analysis::SemanticAnalyzer;

// Helper function which opens and reads a file
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

// Helper function which opens and writes to a file
fn write(filename: String, content: String) -> Result<(), String> {
    let mut file = match File::create(&filename) {
        Err(reason) => {
            return Err(format!(
                "{}",
                format!("File error: {}", reason).bright_red()
            ))
        }
        Ok(file) => file,
    };

    match file.write_all(content.as_bytes()) {
        Err(reason) => {
            return Err(format!(
                "{}",
                format!("File error: {}", reason).bright_red()
            ))
        }
        Ok(file) => file,
    }
    Ok(())
}

// The main function used in actual compilation
// Opens one file and process it in its entirety by
// - Lexing
// - Parsing
// - Semantic Analysis
// -- (Quits if there are errors here)
// - Generating an intermediate representation
// - (Unimplemented optimizations)
// - Generating assembly from the IR
// - Writing the result back to another file
pub fn compile(filename: String, output: String) -> Result<(), String> {
    let mut lexer = Lexer::new(&filename);
    let file = open(filename)?;

    log::info!("Lexer started");
    let (tokens, lexer_errors) = lexer.lex(&mut file.chars());
    log::trace!(target: "lexer","Lexed tokens: {:?}", tokens);

    log::info!("Parser started");
    let mut parser = Parser::new();
    let (mut ast, parse_errors) = parser.parse(tokens);
    log::debug!("Parser result:\n{}", ast);
    let _ = crate::parser::ast_graph::print_graph("graph.gv", &ast);

    log::info!("Analyzer started");
    let mut analyzer = SemanticAnalyzer::new();
    let analysis_errors = analyzer.analyze(&mut ast);

    if lexer_errors.is_err() || parse_errors.is_err() || analysis_errors.is_err() {
        log::info!("Exited due to errors");
        return Err("Error in lexing parsing or analysis".to_string());
    }

    log::info!("Evaluation started");
    log::debug!("\n{}", ast);
    let ir_functions = evaluate(&ast);
    for ir in &ir_functions {
        log::debug!("Evaluation result:\n{}", ir);
    }

    log::info!("Started the backend");
    log::info!("Using backend amd64");
    let assembly = backend::generate_code(ir_functions, "amd64".to_string())?;

    write(output, assembly)?;
    Ok(())
}
