use utcc_lib;
use utcc_lib as utcc;

fn main() {
    let options = utcc::options::get();
    utcc::logger::init().expect("Logger initialization failed");
    log::info!("hello logger");

    if let Err(()) = utcc::driver::drive(options) {
        std::process::exit(1);
    }
}
