//! abstracts logging initialization
//!
//! Additional configuration can be found in `config_dir/LOGGER_TOML`

use crate::config::Config;

use log::Record;
use std::path::PathBuf;

/// Logging information configuration file name
const LOGGER_TOML: &str= "logger.toml";

/// Adds colorized style based on ANSI escapes
pub fn style<T>(level: log::Level, item: T) -> yansi::Paint<T> {
    match level {
        log::Level::Error => yansi::Paint::fixed(196, item).bold(),
        log::Level::Warn => yansi::Paint::fixed(208, item).bold(),
        log::Level::Info => yansi::Paint::new(item),
        log::Level::Debug => yansi::Paint::fixed(7, item),
        log::Level::Trace => yansi::Paint::fixed(8, item),
    }
}

/// formatting in color
pub fn logdemo_format_color(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{} {} [{}] {}",
        // "{} {} {}",
        style(level, now.now().format("%T%.3f")),
        style(level, record.level()),
        record.module_path().unwrap_or("<module>"),
        style(level, &record.args())
    )
}

/// formatting without color
pub fn logdemo_format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{} {} [{}] {}",
        // "{} {} {}",
        now.now().format("%T%.3f"),
        record.level(),
        record.module_path().unwrap_or("<module>"),
        &record.args()
    )
}

/// Initialize logging
pub fn init(config_dir: &str, level: &str) {
    let mut logger_toml: PathBuf = config_dir.into();
    logger_toml.push(LOGGER_TOML);
    flexi_logger::Logger::with_str(level)
        .format(logdemo_format_color)
        .format_for_files(logdemo_format)
        .suppress_timestamp()
        .append()
        .log_to_file()
        .duplicate_to_stderr(flexi_logger::Duplicate::Info)
        .start_with_specfile(logger_toml.to_str()
                             .expect("connot create logger_toml path"))
        .expect("cannot initialize flexi_logger");
}

/// Demonstrates the various logging levels
pub fn act(config: &Config) -> i32 {
    println!("Welcome to {}!", config.app);
    println!("example error:");
    error!("example error");
    println!("example warn:");
    warn!("example warn");
    println!("example info:");
    info!("example info");
    println!("example debug:");
    debug!("example debug");
    println!("example trace:");
    trace!("example trace");
    0
}
