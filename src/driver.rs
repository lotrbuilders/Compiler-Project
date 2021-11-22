use std::ffi::OsStr;
use std::path::Path;

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

pub fn drive(options: Options) -> Result<(), ()> {
    log::info!("driver started");
    for filename in options.input {
        let file_stem = Path::new(&filename)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(&filename)
            .to_string();
        let begin_stage = filename2stage(&filename);
        let last_stage = options.last_stage;
        log::debug!("Going from {:?} to {:?}", begin_stage, last_stage);
        let /*mut*/ _next_filename = filename.clone();

        if begin_stage == Stage::C {
            // Invoke preprocessor
            /*Currently ignored*/
            log::info!("Preprocessor started");
        }
        if begin_stage >= Stage::Ppc && last_stage < Stage::Ppc {
            // Invoke compiler
            log::info!("Compiler started");
            let compiler_filename = filename.clone();
            let assembler_filename = "./".to_string() + &file_stem + ".s";
            compiler::compile(compiler_filename, assembler_filename).unwrap();
            log::info!("Compiler finished");
        }
        if begin_stage <= Stage::Asm && last_stage > Stage::Asm {
            // Invoke assembler
            log::info!("Assembler started");
            /*TODO*/
        }
        if begin_stage <= Stage::Obj && last_stage > Stage::Obj {
            // Invoke linker
            log::info!("Linker started");
            /*TODO*/
        }
    }
    Ok(())
}
