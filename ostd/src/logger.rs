// SPDX-License-Identifier: MPL-2.0

//! Logging support.

use log::{Level, Metadata, Record};

use crate::{boot::kernel_cmdline, early_println};

const LOGGER: Logger = Logger {};

/// The log level. It will change during `logger::init`
static mut LOG_LEVEL: Level = Level::Error;

struct Logger {}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // SAFETY: Once changed in `logger::init`, LOG_LEVEL is guaranteed to be immutable during
        // the system lifetime.
        unsafe { metadata.level() <= LOG_LEVEL }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            early_println!("[{}]: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// Initialize the logger. Users should avoid using the log macros before this function is called.
pub(crate) fn init() {
    log::set_logger(&LOGGER).unwrap();
    let level = kernel_cmdline().get_log_level();
    if let Some(level) = level {
        // SAFETY: LOG_LEVEL only changed during initilization
        unsafe {
            LOG_LEVEL = level;
        }
    }
    // SAFETY: LOG_LEVEL is guaranteed to be immutable.
    unsafe {
        log::set_max_level(LOG_LEVEL.to_level_filter());
    }
}
