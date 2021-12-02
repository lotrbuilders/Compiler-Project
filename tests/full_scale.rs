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

fn test_directories(dir: &Path, failures: &mut Vec<String>) -> io::Result<i32> {
    let mut fail_count = 0;
    if dir.is_dir() {
        for directory in fs::read_dir(dir)? {
            let directory = directory?;
            let path = directory.path();
            if path.is_dir() {
                fail_count += test_stage(path.clone(), failures)?;
            }
        }
    } else {
        panic!("Expected directory")
    }
    Ok(fail_count)
}

fn test_stage(dir: PathBuf, failures: &mut Vec<String>) -> io::Result<i32> {
    let mut valid_dir = dir.clone();
    let mut invalid_dir = dir.clone();
    valid_dir.push("valid/");
    invalid_dir.push("invalid/");
    let fail_count =
        test_valid(valid_dir.as_path(), failures)? + test_invalid(invalid_dir.as_path(), failures)?;
    Ok(fail_count)
}

fn test_valid(dir: &Path, failures: &mut Vec<String>) -> io::Result<i32> {
    let mut fail_count = 0;
    for file in fs::read_dir(dir)? {
        let file = file?;
        let path = file.path();
        if is_c_file(&path) {
            eprintln!("Testing {}", path.to_str().unwrap());
            let options = get_options(&path);
            match utcc::driver::drive(options.clone()) {
                Err(()) => {
                    failures.push(format!("{}: driver failed", options.input[0].clone()));
                    fail_count += 1;
                }
                Ok(()) => (),
            }

            let output = Command::new(&format!("{}.exe", options.output)).output();
            let status = match output {
                Err(error) => {
                    failures.push(format!(
                        "{}: Running file unsuccesfull {}",
                        options.input[0].clone(),
                        error
                    ));
                    fail_count += 1;
                    continue;
                }
                Ok(output) => output,
            };

            Command::new("gcc")
                .args(["-o", &options.output, &options.input[0]])
                .output()
                .expect("gcc failed on test");
            let output = Command::new(&format!("{}.exe", options.output)).output();
            match output {
                Err(_) => {
                    eprintln!("Error when running gcc version");
                    continue;
                }
                Ok(output) => {
                    if output != status {
                        failures.push(format!(
                            "{}: output {:?} does not match status {:?}",
                            options.input[0].clone(),
                            output,
                            status
                        ));
                        fail_count += 1;
                        continue;
                    }
                }
            };
        }
    }
    Ok(fail_count)
}

fn test_invalid(dir: &Path, failures: &mut Vec<String>) -> io::Result<i32> {
    let mut fail_count = 0;
    for file in fs::read_dir(dir)? {
        let file = file?;
        let path = file.path();
        if is_c_file(&path) {
            eprintln!("Testing {}", path.to_str().unwrap());
            let options = get_options(&path);
            match utcc::driver::drive(options.clone()) {
                Err(()) => (),
                Ok(()) => {
                    failures.push(format!(
                        "Invalid example did not produce error: {}",
                        options.input[0].clone()
                    ));
                    fail_count += 1;
                }
            }
        }
    }
    Ok(fail_count)
}

#[test]
fn full_scale_test() {
    let mut failures = Vec::<String>::new();
    let home_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = format!("{}/tests/src", home_dir);
    let test_path = Path::new(&test_dir);
    let fail_count = test_directories(&test_path, &mut failures).expect("File error");
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
