#[macro_export]
macro_rules! error {
    ($span:expr,$( $exp:expr ),*) => {
        {
            use colored::Colorize;
            format!("{:?}: error: {}",$span,format!($($exp,)*).red())
        }
    };
}

#[macro_export]
macro_rules! warning {
    ($span:expr,$( $exp:expr ),*) => {
        {
            use colored::Colorize;
            format!("{:?}: warning: {}",$span,format!($($exp,)*).purple())
        }
    };
}
