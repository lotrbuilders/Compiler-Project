use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{io, str};

fn find_dir(test: &str) -> io::Result<BTreeMap<String, String>> {
    let home_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = format!("{}/tests/performance/{}", home_dir, test);
    let dir = Path::new(&test_dir);
    if dir.is_dir() {
        return performance_test(dir, test);
    } else {
        panic!("Expected directory")
    }
}

fn performance_test(dir: &Path, test: &str) -> io::Result<BTreeMap<String, String>> {
    let dir = dir.to_str().unwrap();
    let main_file = format!("{}/main.c", dir);
    let main_output = format!("{}/main.o", dir);
    let test_cases = vec![
        ("gcc-O0", "gcc", vec!["-O0", "-c", "-o"]),
        ("gcc-O1", "gcc", vec!["-O1", "-c", "-o"]),
        ("gcc-O2", "gcc", vec!["-O2", "-c", "-o"]),
        ("utcc-O0", "cargo", vec!["run", "--", "-O0", "-c", "-o"]),
        ("utcc-O1", "cargo", vec!["run", "--", "-O1", "-c", "-o"]),
    ];

    let res = Command::new("gcc")
        .args(["-c", "-o", &main_output, &main_file])
        .output()
        .expect("gcc failed on test");

    println!(
        "res: {} : {}",
        str::from_utf8(&res.stderr).unwrap(),
        str::from_utf8(&res.stdout).unwrap(),
    );

    let mut result = BTreeMap::new();
    for (name, exe, options) in test_cases {
        println!("Running {} with {}", test, name);
        let res = performance_case(exe, options, dir, test);
        result.insert(String::from(name), res);
    }

    Ok(result)
}

fn performance_case(exe: &str, options: Vec<&str>, dir: &str, test: &str) -> String {
    let main_file = format!("{}/main.o", dir);
    let c_file = format!("{}/{}.c", dir, test);
    let obj_file = format!("{}/{}.o", dir, test);
    let out_file = format!("{}/a.out", dir);

    let files = vec![&obj_file as &str, &c_file as &str].into_iter();

    let options: Vec<_> = options.iter().cloned().chain(files).collect();
    Command::new(exe)
        .args(&options)
        .output()
        .expect("compiler failed on test");

    Command::new("gcc")
        .args(&["-o", &out_file, &main_file, &obj_file])
        .output()
        .expect("linking failed");

    let output = Command::new(out_file).output().expect("running failed");

    format!("{}", str::from_utf8(&output.stdout).unwrap())
}

fn perf_test2() -> io::Result<()> {
    let home_dir = env!("CARGO_MANIFEST_DIR");
    let test_file = format!("{}/tests/performance/result.txt", home_dir);
    let test_path = Path::new(&test_file);
    let mut test_file = File::create(test_path)?;

    let _tests = vec!["sudoku", "gcd", "primes", "fibonacci"];
    let tests = vec!["sudoku"];

    for test in tests {
        let map = find_dir(test)?;
        writeln!(test_file, "test: {}", test)?;

        for (test, result) in &map {
            write!(test_file, "{:<10}: {}", test, result)?;
        }
        writeln!(test_file, "")?;
    }
    Ok(())
}

#[test]
fn perf_test() {
    perf_test2().expect("performance test");
}
