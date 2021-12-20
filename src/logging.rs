use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming,
};
use log::Record;
use once_cell::sync::OnceCell;
use std::{
    error::Error,
    io::{Result as IoResult, Write},
};
use time::{format_description::FormatItem, macros::format_description};

lazy_static! {
    static ref LOG_DATE_FORMAT: &'static [FormatItem<'static>] =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
}

static LOGGER: OnceCell<LoggerHandle> = OnceCell::new();

pub fn initialize() -> Result<(), Box<dyn Error>> {
    let file_spec = FileSpec::default().directory("logs");

    let logger_handle = Logger::try_with_str("danser_thing_rust=debug")?
        .log_to_file(file_spec)
        .format(log_format)
        .format_for_files(log_format_files)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogAndCompressedFiles(5, 20),
        )
        .duplicate_to_stdout(Duplicate::Info)
        .start()?;

    let _ = LOGGER.set(logger_handle);

    Ok(())
}

pub fn log_format(w: &mut dyn Write, now: &mut DeferredNow, record: &Record<'_>) -> IoResult<()> {
    write!(
        w,
        "[{}] {} {}",
        now.format(&LOG_DATE_FORMAT),
        record.level(),
        &record.args()
    )
}

pub fn log_format_files(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> IoResult<()> {
    write!(
        w,
        "[{}] {:^5} [{}:{}] {}",
        now.format(&LOG_DATE_FORMAT),
        record.level(),
        record.file_static().unwrap_or_else(|| record.target()),
        record.line().unwrap_or(0),
        &record.args()
    )
}
