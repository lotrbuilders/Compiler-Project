mod compiler;
mod driver;
mod lexer;
mod logger;
mod options;
mod parser;
mod span;
mod token;

use crate::span::Span;

fn main() {
    let options = options::get();
    println!("Hello, world!");
    crate::logger::init().expect("Logger initialization filled");
    log::info!("hello logger");
    let a = Span::new(0, 1, 1, 0, 1);
    let b = Span::new(0, 1, 2, 1, 1);
    let c = a.to(&b);
    println!("{:?}", c);

    if let Err(()) = driver::drive(options) {
        std::process::exit(1);
    }
}
