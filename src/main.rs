mod logger;
mod span;

use crate::span::Span;

fn main() {
    println!("Hello, world!");
    crate::logger::init().expect("Logger initialization filled");
    log::info!("hello logger");
    let a = Span::new(1, 1, 0, 1);
    let b = Span::new(1, 2, 1, 1);
    let c = a.to(&b);
    println!("{:?}", c);
}
