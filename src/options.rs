use clap::{clap_derive::Parser, ArgGroup, Args, StructOpt};
// use clap::{App,Arg}
// use colored::Colorize;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
pub struct Options {
    /// Input files
    #[clap(required = true)]
    pub input: Vec<String>,

    /// Output files
    #[clap(short, long, default_value_t = String::from("./a.out"))]
    pub output: String,

    #[clap(flatten)]
    pub last_stage: OptionStage,

    #[clap(flatten)]
    pub optimization_settings: OptimizationSettings,

    /// Register allocator to use. Normally use briggs
    #[clap(long="reg-alloc", default_value_t = String::from("briggs"), possible_values(&["simple", "briggs"]))]
    pub register_allocator: String,
}

#[derive(Clone, Debug, Args)]
#[clap(group(
    ArgGroup::new("vers")
        .required(false)
        .args(& ["obj", "asm", "ppc"])
))]
pub struct OptionStage {
    /// Compiles and assembles code, but does link
    #[clap(short = 'c')]
    pub obj: bool,

    /// Compiles code, but does not assemble
    #[clap(short = 'S')]
    pub asm: bool,

    /// Only preprocess code
    #[clap(short = 'E')]
    pub ppc: bool,
}
#[derive(Clone, Debug, Args)]
pub struct OptimizationSettings {
    /// Optimization level to use
    #[clap(short = 'O', default_value_t = 0,possible_values(&["-1", "0", "1", "2", "3"]))]
    pub optimization_level: i32,

    /// Explicit enable or disable of optimizations
    #[clap(long = "opt")]
    pub optimizations: Vec<String>,
}

/// Gets command line options and input using clap.
/// Checks for illegal combinations.
/// Returns an Options struct representing the fully parsed options
pub fn get() -> Options {
    /*let matches = App::new("UTCC - A C compiler")
        .version("0.1")
        .author("Daan O.")
        .about("Compiles a subset A of C where union(A,C)<=>epsilon")
        .arg(
            Arg::new("until-assembled")
                .short('c')
                .help("Compiles and assembles code, but does link"),
        )
        .arg(
            Arg::new("until-compiled")
                .short('S')
                .help("Compiles code, but does not assemble"),
        )
        .arg(
            Arg::new("until-preprocessed")
                .short('E')
                .help("Preprocesses code, but does not compile"),
        )
        .group(ArgGroup::new("until-stage").args(&[
            "until-assembled",
            "until-compiled",
            "until-preprocessed",
        ]))
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .value_name("file")
                .help("The output file to output. Defaults to './a.out'"),
        )
        .arg(
            Arg::new("input")
                .required(true)
                .multiple_occurrences(true)
                .help("Sets the input file(s) to use"),
        )
        .arg(
            Arg::new("register-allocation")
                .long("reg-alloc")
                .possible_values(&["simple", "briggs"])
                .default_value("simple")
                .value_name("allocator")
                .help("Selects the register allocator algorithm used"),
        )
        .arg(
            Arg::new("optimization-level")
                .short('O')
                .takes_value(true)
                .possible_values(&["-1", "0", "1", "2", "3"])
                .default_value("0")
                .help("Optimization level used"),
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

    let optimization_level = matches
        .value_of("optimization-level")
        .unwrap()
        .parse()
        .unwrap();

    let register_allocator = matches.value_of("register-allocation").unwrap().to_string();

    Options {
        input,
        output,
        last_stage,
        optimization_level,
        register_allocator,
    }*/
    Options::parse()
}
