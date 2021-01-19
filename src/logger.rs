use crate::println;
use log::LevelFilter;
use log::{Metadata, Record};

static LOGGER: SimpleLogger = SimpleLogger;

/// Initializes the global logger.
pub fn init_logger(max_level: LevelFilter) {
    unsafe {
        log::set_logger_racy(&LOGGER)
            .map(|()| log::set_max_level(max_level))
            .unwrap()
    };
}

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true // metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(" {} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
