use utcc_lib;
use utcc_lib as utcc;

fn main() {
    utcc::logger::init().expect("Logger initialization failed");
    if cfg!(debug_assertions) {
        log::info!("hello logger");
    } else {
        log::set_max_level(log::LevelFilter::Warn);
    }
    let options = utcc::options::get();

    if let Err(()) = utcc::driver::drive(options) {
        std::process::exit(1);
    }
}
