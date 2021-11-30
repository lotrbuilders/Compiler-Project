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
