use crate::CommonOptions;
use log::*;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone)]
pub(crate) enum LogLevel {
    Warn,
    Info,
    Verbose,
    Debug,
}

impl LogLevel {
    fn to_filter(self) -> LevelFilter {
        match self {
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Verbose => LevelFilter::Debug,
            LogLevel::Debug => LevelFilter::Trace,
        }
    }
}

pub(crate) struct SimpleCommandLogger {
    log_level: LogLevel,
    module: String,
}

pub(crate) fn init_command_logger_with_options(options: &CommonOptions, module: impl Into<String>) {
    let log_level = if options.quiet {
        LogLevel::Warn
    } else if options.debug {
        LogLevel::Debug
    } else if options.verbose {
        LogLevel::Verbose
    } else {
        LogLevel::Info
    };

    let logger = SimpleCommandLogger {
        log_level,
        module: module.into(),
    };

    log::set_max_level(log_level.to_filter());
    log::set_boxed_logger(Box::new(logger)).unwrap();
}

impl log::Log for SimpleCommandLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if !metadata.target().starts_with(&self.module) {
            return false;
        }
        match self.log_level {
            LogLevel::Warn => metadata.level() >= Level::Warn,
            LogLevel::Info => metadata.level() >= Level::Info,
            LogLevel::Verbose => metadata.level() >= Level::Debug,
            LogLevel::Debug => metadata.level() >= Level::Trace,
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        match record.level() {
            Level::Error => eprintln!("e: {}", record.args()),
            Level::Warn => eprintln!("w: {}", record.args()),
            Level::Info => eprintln!("{}", record.args()),
            Level::Debug => eprintln!("VERBOSE {}: {}", record.target(), record.args()),
            Level::Trace => eprintln!("TRACE   {}: {}", record.target(), record.args()),
        }
    }

    fn flush(&self) {
        std::io::stderr().flush().unwrap()
    }
}

pub(crate) struct ActionsLogger {
    module: String,
    debug: bool,
}

impl ActionsLogger {
    fn max_level(&self) -> LevelFilter {
        if self.debug {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        }
    }
}

static ACTIONS_ENV: AtomicBool = AtomicBool::new(false);

pub(crate) fn init_actions(module: impl Into<String>) {
    ACTIONS_ENV.store(true, Ordering::SeqCst);
    assert!(is_actions_env());

    let logger = ActionsLogger {
        module: module.into(),
        debug: actions_core::is_debug(),
    };

    log::set_max_level(logger.max_level());
    log::set_boxed_logger(Box::new(logger)).unwrap();
}

pub(crate) fn is_actions_env() -> bool {
    ACTIONS_ENV.load(Ordering::SeqCst)
}

impl log::Log for ActionsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with(&self.module)
            && (self.debug || metadata.level() >= Level::Info)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        use ::actions_core as core;
        match record.level() {
            Level::Error => core::error(record.args()),
            Level::Warn => core::warning(record.args()),
            Level::Info => eprintln!("{}", record.args()),
            Level::Debug => core::debug(format_args!(
                "VERBOSE {}: {}",
                record.target(),
                record.args()
            )),
            Level::Trace => core::debug(format_args!(
                "TRACE   {}: {}",
                record.target(),
                record.args()
            )),
        }
    }

    fn flush(&self) {
        std::io::stderr().flush().unwrap()
    }
}
