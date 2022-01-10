use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use crate::compiler;
use crate::options::Options;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Stage {
    Exe,
    Obj,
    Asm,
    Ppc,
    C,
}

/// Derive the starting stage from the filename
fn filename2stage(filename: &str) -> Stage {
    let extension = Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");
    match extension {
        "c" => Stage::C,
        "ppc" => Stage::Ppc,
        "s" | "asm" => Stage::Asm,
        _ => Stage::Obj,
    }
}

// Return either the current filename or the last_filename
// Depending if we're in the last stage
fn new_or_final<'a>(
    filename: &'a String,
    last_filename: &'a String,
    last_stage: Stage,
    stage: Stage,
) -> &'a String {
    if stage == last_stage {
        last_filename
    } else {
        filename
    }
}

// This function calls all seperate sub-components with the correct parameters being
// - Preprocessor
// - Compiler
// - Assembler
// - Linking
pub fn drive(options: Options) -> Result<(), ()> {
    log::info!("driver started");
    for filename in options.input.clone() {
        let file_stem = Path::new(&filename)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(&filename)
            .chars()
            .map(|c| match c {
                '\\' => '/',
                _ => c,
            })
            .collect::<String>();

        let begin_stage = filename2stage(&filename);
        let last_stage = options.last_stage.clone();
        let last_filename = options.output.clone();
        let parent = Path::new(&filename)
            .parent()
            .expect("")
            .to_str()
            .expect("msg");

        let parent = parent
            .chars()
            .map(|c| match c {
                '\\' => '/',
                _ => c,
            })
            .collect::<String>();

        log::debug!("Going from {:?} to {:?}", begin_stage, last_stage);
        log::debug!("file_stem {}, parent {}", file_stem, parent);
        let mut next_filename = filename.clone();

        if begin_stage == Stage::C {
            // Invoke preprocessor
            /*Currently ignored*/
            log::info!("Preprocessor started");
        }
        if begin_stage >= Stage::Ppc && last_stage < Stage::Ppc {
            // Invoke compiler
            let compiler_filename = next_filename;
            let assembler_filename = parent.clone() + "/" + &file_stem + ".s";
            let assembler_filename =
                new_or_final(&assembler_filename, &last_filename, last_stage, Stage::Ppc);
            next_filename = assembler_filename.clone();

            log::info!(
                "Compiler started -o {} {}",
                assembler_filename,
                compiler_filename
            );

            compiler::compile(compiler_filename, assembler_filename.clone(), &options)
                .map_err(|_| ())?;
            log::info!("Compiler finished");
        }
        if begin_stage >= Stage::Asm && last_stage < Stage::Asm {
            // Invoke assembler
            let assembler_filename = next_filename;
            if parent.contains(":/") {}
            let linker_filename = parent.clone() + "/" + &file_stem + ".o";
            let linker_filename =
                new_or_final(&linker_filename, &last_filename, last_stage, Stage::Asm);
            next_filename = linker_filename.clone();

            log::info!(
                "Assembler started -o {} {}",
                linker_filename,
                assembler_filename
            );

            let output = Command::new("nasm")
                .args(["-felf64", "-o", &linker_filename, &assembler_filename])
                .output()
                .expect("failed to run assembler");

            log::info!(
                "status {}\nstdout: {}\nstderr: {}",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
            if !output.status.success() {
                return Err(());
            }
        }
        if begin_stage >= Stage::Obj && last_stage < Stage::Obj {
            // Invoke linker
            let linker_filename = next_filename;
            let result = last_filename.clone();

            /*let result = if result.contains(":\\") {
                "/mnt/".to_string()
                    + &result
                        .chars()
                        .next()
                        .unwrap()
                        .to_ascii_lowercase()
                        .to_string()
                    + &result.chars().skip_while(|c| *c != '/').collect::<String>()
            } else {
                result
            };*/

            log::info!("Linker started -o {} {}", result, linker_filename);

            let output = Command::new("gcc")
                .args(["-m64", "-fPIC", "-o", &result, &linker_filename])
                .output()
                .expect("failed to run linker");

            log::info!(
                "status {}\nstdout: {}\nstderr: {}",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
            if !output.status.success() {
                return Err(());
            }
        }
    }
    Ok(())
}
