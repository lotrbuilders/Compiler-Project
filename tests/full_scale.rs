use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
use utcc_lib as utcc;

fn get_options(path: &PathBuf) -> utcc::options::Options {
    let mut output = path.parent().unwrap().to_str().unwrap().to_string();
    output.push_str("/");
    output.push_str(path.file_stem().unwrap().to_str().unwrap());
    let output = output
        .chars()
        .map(|c| match c {
            '\\' => '/',
            _ => c,
        })
        .collect();
    utcc::options::Options {
        input: vec![path.to_str().unwrap().to_string()],
        output,
        last_stage: utcc::driver::Stage::Exe,
    }
}

fn is_c_file(path: &PathBuf) -> bool {
    if let Some("c") = &path.extension().iter().filter_map(|s| s.to_str()).next() {
        true
    } else {
        false
    }
}

fn empty_test(_path: PathBuf, _failures: &mut Vec<String>, _error_count: &mut i32) {}

fn test_directories<F, G>(
    dir: &Path,
    failures: &mut Vec<String>,
    valid: F,
    invalid: G,
) -> io::Result<i32>
where
    F: Fn(PathBuf, &mut Vec<String>, &mut i32),
    G: Fn(PathBuf, &mut Vec<String>, &mut i32),
{
    let mut fail_count = 0;
    if dir.is_dir() {
        for directory in fs::read_dir(dir)? {
            let directory = directory?;
            let path = directory.path();
            if path.is_dir() {
                fail_count += test_stage(path.clone(), failures, &valid, &invalid)?;
            }
        }
    } else {
        panic!("Expected directory")
    }
    Ok(fail_count)
}

fn test_stage<F, G>(
    dir: PathBuf,
    failures: &mut Vec<String>,
    valid: &F,
    invalid: &G,
) -> io::Result<i32>
where
    F: Fn(PathBuf, &mut Vec<String>, &mut i32),
    G: Fn(PathBuf, &mut Vec<String>, &mut i32),
{
    let mut valid_dir = dir.clone();
    let mut invalid_dir = dir.clone();
    valid_dir.push("valid/");
    invalid_dir.push("invalid/");
    let fail_count = test_valid(valid_dir.as_path(), failures, valid)?
        + test_invalid(invalid_dir.as_path(), failures, invalid)?;
    Ok(fail_count)
}

fn test_valid<F>(dir: &Path, failures: &mut Vec<String>, test: &F) -> io::Result<i32>
where
    F: Fn(PathBuf, &mut Vec<String>, &mut i32),
{
    let mut fail_count = 0;
    if !dir.is_dir() {
        return Ok(0);
    }
    for file in fs::read_dir(dir)? {
        let file = file?;
        let path = file.path();
        if is_c_file(&path) {
            eprintln!("Testing {}", path.to_str().unwrap());
            test(path, failures, &mut fail_count);
        }
    }
    Ok(fail_count)
}

fn test_invalid<F>(dir: &Path, failures: &mut Vec<String>, test: &F) -> io::Result<i32>
where
    F: Fn(PathBuf, &mut Vec<String>, &mut i32),
{
    let mut fail_count = 0;
    if !dir.is_dir() {
        return Ok(0);
    }
    for file in fs::read_dir(dir)? {
        let file = file?;
        let path = file.path();
        if is_c_file(&path) {
            eprintln!("Testing {}", path.to_str().unwrap());
            test(path, failures, &mut fail_count);
        }
    }
    Ok(fail_count)
}

fn test_valid_full_scale(path: PathBuf, failures: &mut Vec<String>, fail_count: &mut i32) {
    let options = get_options(&path);
    match utcc::driver::drive(options.clone()) {
        Err(()) => {
            failures.push(format!("{}: driver failed", options.input[0].clone()));
            *fail_count += 1;
        }
        Ok(()) => (),
    }

    let output = Command::new(&format!("{}", options.output)).output();
    let status = match output {
        Err(error) => {
            failures.push(format!(
                "{}: Running file unsuccesfull {}\n{}",
                options.input[0].clone(),
                error,
                options.output
            ));
            *fail_count += 1;
            return;
        }
        Ok(output) => output,
    };

    Command::new("gcc")
        .args(["-o", &options.output, &options.input[0]])
        .output()
        .expect("gcc failed on test");
    let output = Command::new(&format!("{}", options.output)).output();
    match output {
        Err(_) => {
            eprintln!("Error when running gcc version");
            return;
        }
        Ok(output) => {
            if output != status {
                failures.push(format!(
                    "{}: output {:?} does not match expected output {:?} {}",
                    options.input[0].clone(),
                    status,
                    output,
                    options.output
                ));
                *fail_count += 1;
                return;
            }
        }
    };
}

fn test_invalid_full_scale(path: PathBuf, failures: &mut Vec<String>, fail_count: &mut i32) {
    let options = get_options(&path);
    match utcc::driver::drive(options.clone()) {
        Err(()) => (),
        Ok(()) => {
            failures.push(format!(
                "Invalid example did not produce error: {}",
                options.input[0].clone()
            ));
            *fail_count += 1;
        }
    }
}

macro_rules! tests {
    ($($name:ident: ($file:literal, $valid:ident, $invalid:ident))*) => {
        $(
            #[test]
            fn $name() {
                let mut failures = Vec::<String>::new();
                let home_dir = env!("CARGO_MANIFEST_DIR");
                let test_dir = format!("{}/tests/{}", home_dir,$file);
                let test_path = Path::new(&test_dir);
                let fail_count = test_stage(
                    test_path.to_path_buf(),
                    &mut failures,
                    &$valid,
                    &$invalid,
                )
                .expect("File error");
                let mut string = String::new();
                for failure in failures {
                    string.push_str(&format!("{}\n", failure));
                }
                assert_eq!(
                    fail_count, 0,
                    "Failures occured during testing\n {}",
                    string
                );
            }
        )*
    };
}

tests! {
    full_scale_stage_1: ("src/stage_1",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_2: ("src/stage_2",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_3: ("src/stage_3",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_4: ("src/stage_4",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_5: ("src/stage_5",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_6: ("src/stage_6",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_7: ("src/stage_7",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_8: ("src/stage_8",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_9: ("src/stage_9",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_10: ("src/stage_10",test_valid_full_scale,test_invalid_full_scale)

    full_scale_stage_11: ("src/stage_11",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_12: ("src/stage_12",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_13: ("src/stage_13",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_14: ("src/stage_14",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_15: ("src/stage_15",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_16: ("src/stage_16",test_valid_full_scale,test_invalid_full_scale)

    full_scale_stage_19: ("src/stage_19",test_valid_full_scale,test_invalid_full_scale)
    full_scale_stage_21: ("src/stage_21",test_valid_full_scale,test_invalid_full_scale)
}

fn test_valid_parser(path: PathBuf, failures: &mut Vec<String>, fail_count: &mut i32) {
    use utcc_lib::backend;
    use utcc_lib::compiler::open;
    use utcc_lib::lexer::Lexer;
    use utcc_lib::parser::Parser;
    let filename = path.to_str().unwrap().to_string();
    let mut lexer = Lexer::new(&filename);
    let file = open(filename.clone()).expect("opening file failed");

    log::info!("Getting backend");
    let backend = backend::get_backend("amd64".to_string()).expect("getting backend");

    let (tokens, lexer_errors) = lexer.lex(&mut file.chars());

    let mut parser = Parser::new(&*backend);
    let (ast1, parse_errors) = parser.parse(tokens);
    let ast1_string = format!("{}", ast1);
    if lexer_errors.is_err() || parse_errors.is_err() {
        if let Err(err) = lexer_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for lexer {:?}",
                filename, err
            ));
        }
        if let Err(err) = parse_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for parse {:?}",
                filename, err
            ));
        }
        return;
    }
    lexer = Lexer::new(&filename);
    let (tokens, lexer_errors) = lexer.lex(&mut ast1_string.chars());
    parser = Parser::new(&*backend);
    let (ast2, parse_errors) = parser.parse(tokens);
    if lexer_errors.is_err() || parse_errors.is_err() {
        if let Err(err) = lexer_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for lexer 2{:?}",
                filename, err
            ));
        }
        if let Err(err) = parse_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for parse 2{:?}",
                filename, err
            ));
        }
        return;
    }
    let ast2_string = format!("{}", ast2);

    if ast1_string != ast2_string {
        *fail_count += 1;
        failures.push(format!(
            "{}: Difference in ast\nFirst ast:\n{}\nSecond ast:\n{}\n",
            filename, ast1_string, ast2_string
        ));
    }
}

#[test]
fn parser_test() {
    let mut failures = Vec::<String>::new();
    let home_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = format!("{}/tests/src", home_dir);
    let test_path = Path::new(&test_dir);
    let fail_count = test_directories(&test_path, &mut failures, test_valid_parser, empty_test)
        .expect("File error");
    let mut string = String::new();
    for failure in failures {
        string.push_str(&format!("{}\n", failure));
    }
    assert_eq!(
        fail_count, 0,
        "Failures occured during testing\n {}",
        string
    );
}

/*
fn test_valid_symbol_table(path: PathBuf, failures: &mut Vec<String>, fail_count: &mut i32) {
    use utcc_lib::compiler::open;
    use utcc_lib::lexer::Lexer;
    use utcc_lib::parser::Parser;
    use utcc_lib::semantic_analysis::SemanticAnalyzer;
    let filename = path.to_str().unwrap().to_string();
    let mut lexer = Lexer::new(&filename);
    let file = open(filename.clone()).expect("opening file failed");

    let (tokens, lexer_errors) = lexer.lex(&mut file.chars());

    let mut parser = Parser::new();
    let (mut ast, parse_errors) = parser.parse(tokens);

    let mut analyzer = SemanticAnalyzer::new();
    let analysis_errors = analyzer.analyze(&mut ast);
    if lexer_errors.is_err() || parse_errors.is_err() || analysis_errors.is_err() {
        if let Err(err) = lexer_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for lexer {:?}",
                filename, err
            ));
        }
        if let Err(err) = parse_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for parse {:?}",
                filename, err
            ));
        }
        if let Err(err) = analysis_errors {
            *fail_count += 1;
            failures.push(format!(
                "{}: Running file unsuccesfull for analysis{:?}",
                filename, err
            ));
        }
        return;
    }
}

#[test]
fn symbol_table_test() {
    let mut failures = Vec::<String>::new();
    let home_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = format!("{}/tests/src", home_dir);
    let test_path = Path::new(&test_dir);
    let fail_count = test_directories(
        &test_path,
        &mut failures,
        test_valid_symbol_table,
        empty_test,
    )
    .expect("File error");
    let mut string = String::new();
    for failure in failures {
        string.push_str(&format!("{}\n", failure));
    }
    assert_eq!(
        fail_count, 0,
        "Failures occured during testing\n {}",
        string
    );
}*/
