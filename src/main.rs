use utcc_lib;
use utcc_lib as utcc;

fn main() {
    utcc::logger::init().expect("Logger initialization failed");
    log::info!("hello logger");
    let options = utcc::options::get();

    if let Err(()) = utcc::driver::drive(options) {
        std::process::exit(1);
    }
}
