use crate::print;
use crate::println;
use arrayvec::ArrayString;
use core::fmt::Write;

use log::{Level, Metadata, Record};

use log::LevelFilter;
static LOGGER: SimpleLogger = SimpleLogger;

/// Initializes the global logger.
pub fn init_logger() {
    unsafe {
        log::set_logger_racy(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
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
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
