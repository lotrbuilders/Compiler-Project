// Returns a string representing an error and prints it using the standardized format
#[macro_export]
macro_rules! error {
    ($span:expr,$( $exp:expr ),*) => {
        {
            use colored::Colorize;
            let string = format!("{}: {} {}", $span, "error:".bright_red(),format!($($exp,)*));
            eprintln!("{}", string);
            string
        }
    };
}

// Returns a string representing an error and prints it using the standardized format
#[macro_export]
macro_rules! warning {
    ($span:expr,$( $exp:expr ),*) => {
        {
            use colored::Colorize;
            let string = format!("{}: {} {}", $span, "warning:".purple(),format!($($exp,)*));
            eprintln!("{}", string);
            string
        }
    };
}
