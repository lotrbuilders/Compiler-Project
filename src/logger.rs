use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

// Uses 'log' to allow for logging
// Used for debugging the compiler more quickly
// The error level for specific modules and custom codes can be specified
// Logging is done by logging the level, target and the given arguments
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level()
            <= match metadata.target() {
                "lexer" => Level::Info,
                "parser" => Level::Info,
                "utcc::semantic_analysis" => Level::Trace,
                _ => Level::Trace,
            }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let output = format!(
                "{} - {} - {}",
                record.level(),
                record.target(),
                record.args()
            );

            match record.level() {
                Level::Error => println!("{}", output.red()),
                Level::Warn => println!("{}", output.purple()),
                Level::Info => println!("{}", output.blue()),
                _ => println!("{}", output),
            }
        }
    }

    fn flush(&self) {}
}

// Initializes 'log' with the custom logger
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}
static LOGGER: SimpleLogger = SimpleLogger;
