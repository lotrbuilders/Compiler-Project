use crate::driver::Stage;
use clap::{App, Arg, ArgGroup};
use colored::Colorize;

pub struct Options {
    pub input: Vec<String>,
    pub output: String,
    pub last_stage: Stage,
}

/// Gets command line options and input using clap.
/// Checks for illegal combinations.
/// Returns an Options struct representing the fully parsed options
pub fn get() -> Options {
    let matches = App::new("UTCC - A C compiler")
        .version("0.1")
        .author("Daan O.")
        .about("Compiles the subset A of C where union(A,C)<=>epsilon")
        .arg(
            Arg::with_name("until-assembled")
                .short("c")
                .help("Compiles and assembles code, but does link"),
        )
        .arg(
            Arg::with_name("until-compiled")
                .short("S")
                .help("Compiles code, but does not assemble"),
        )
        .arg(
            Arg::with_name("until-preprocessed")
                .short("E")
                .help("Preprocesses code, but does not compile"),
        )
        .group(ArgGroup::with_name("until-stage").args(&[
            "until-assembled",
            "until-compiled",
            "until-preprocessed",
        ]))
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .value_name("file")
                .help("The output file to output. Defaults to './a.out'"),
        )
        .arg(
            Arg::with_name("input")
                .help("Sets the input file(s) to use")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    //Object files, assembly files and preprocessed C files cannot be compiled to a single file
    //Exits if this does oocur
    if matches.is_present("until-stage") && matches.occurrences_of("input") > 1 {
        eprintln!(
            "{}",
            format!("It is not allowed to have multiple inputs if -c, -S ,or -E is specified")
                .bright_red()
        );
        std::process::exit(1);
    }

    //Gets the files and transforms them to a vector of strings
    let input = matches
        .values_of("input")
        .unwrap()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    //Gets the output file as String
    let output = matches.value_of("output").unwrap_or("a.out").to_string();

    // Converts the String to the correct enum
    let last_stage = if matches.is_present("until-preprocessed") {
        Stage::Ppc
    } else if matches.is_present("until-compiled") {
        Stage::Asm
    } else if matches.is_present("until-assembled") {
        Stage::Obj
    } else {
        Stage::Exe
    };

    Options {
        input,
        output,
        last_stage,
    }
}
