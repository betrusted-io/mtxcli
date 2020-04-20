//! abstracts logging initialization
//!
//! Additional configuration can be found in `config_dir/LOGGER_TOML`

use log::Record;

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
        now.now().format("%T%.3f"),
        record.level(),
        record.module_path().unwrap_or("<module>"),
        &record.args()
    )
}

/// Initialize logging
pub fn init(config_dir: &str, level: &str) {
    let mut logger_toml = String::from(config_dir);
    logger_toml.push('/');
    logger_toml.push_str(LOGGER_TOML);
    flexi_logger::Logger::with_str(level)
        .format(logdemo_format_color)
        .format_for_files(logdemo_format)
        .suppress_timestamp()
        .append()
        .log_to_file()
        .duplicate_to_stderr(flexi_logger::Duplicate::Warn)
        .start_with_specfile(logger_toml)
        .unwrap();
}
