use crate::{Color, Style, CURRENT_PROGRESS_BAR};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

struct StdoutLogger;

impl log::Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

#[derive(Clone, Copy)]
pub enum InnerLogger {
    /// A main logger will always be used.
    ///
    /// When a progress bar is active, the provided logger will be used. The progress bar will always be properly updated after each log.
    /// When there is no progress bar, the provided logger will be used.
    Main(&'static dyn Log),

    /// A fallback logger will be used when no progress bar is active.
    ///
    /// When a progress bar is active, logs will be displayed using [`ProgressBar::print_info`].
    /// When there is no progress bar, the provided logger will be used.
    Fallback(&'static dyn Log),

    /// No inner logger is set.
    ///
    /// When a progress bar is active, logs will be displayed using [`ProgressBar::print_info`].
    /// When there is no progress bar, logs will NOT be printed. If you would like them to be printed to stdout, use [`InnerLogger::stdout_fallback`].
    None,
}

impl InnerLogger {
    /// No inner logger is set but logs will still always be printed.
    ///
    /// When a progress bar is active, logs will be displayed using [`ProgressBar::print_info`].
    /// When there is no progress bar, logs will be printed to stdout.
    fn stdout_fallback() -> Self {
        InnerLogger::Fallback(&StdoutLogger)
    }
}

impl Default for InnerLogger {
    fn default() -> Self {
        Self::stdout_fallback()
    }
}

impl<L: Log + 'static> From<L> for InnerLogger {
    fn from(logger: L) -> Self {
        InnerLogger::Main(Box::leak(Box::new(logger)))
    }
}

struct ProgressBarLogger(InnerLogger);

impl log::Log for ProgressBarLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match self.0 {
            InnerLogger::Main(inner) => inner.enabled(metadata),
            InnerLogger::Fallback(inner) => match CURRENT_PROGRESS_BAR.lock().as_deref() {
                Ok(Some(_)) => true,
                Ok(None) | Err(_) => inner.enabled(metadata),
            },
            InnerLogger::None => matches!(CURRENT_PROGRESS_BAR.lock().as_deref(), Ok(Some(_))),
        }
    }

    fn log(&self, record: &Record) {
        match self.0 {
            InnerLogger::Main(inner) => {
                print!("\r\x1B[K\r");
                eprint!("\r\x1B[K\r");
                inner.log(record);
                if let Ok(Some(progress_bar)) = CURRENT_PROGRESS_BAR.lock().as_deref() {
                    progress_bar.display();
                }
            }
            inner => match CURRENT_PROGRESS_BAR.lock().as_deref_mut() {
                Ok(Some(progress_bar)) => match record.level() {
                    Level::Error => progress_bar.print_info(
                        "Error",
                        &record.args().to_string(),
                        Color::Red,
                        Style::Bold,
                    ),
                    Level::Warn => progress_bar.print_info(
                        "Warn",
                        &record.args().to_string(),
                        Color::Yellow,
                        Style::Bold,
                    ),
                    Level::Info => progress_bar.print_info(
                        "Info",
                        &record.args().to_string(),
                        Color::LightGreen,
                        Style::Bold,
                    ),
                    Level::Debug => progress_bar.print_info(
                        "Debug",
                        &record.args().to_string(),
                        Color::Blue,
                        Style::Normal,
                    ),
                    Level::Trace => progress_bar.print_info(
                        "Trace",
                        &record.args().to_string(),
                        Color::LightGray,
                        Style::Normal,
                    ),
                },
                Ok(None) | Err(_) => match inner {
                    InnerLogger::Main(_) => unreachable!(),
                    InnerLogger::Fallback(inner) => {
                        print!("\r\x1B[K\r");
                        eprint!("\r\x1B[K\r");
                        inner.log(record)
                    }
                    InnerLogger::None => (),
                },
            },
        }
    }

    fn flush(&self) {}
}

/// Initializes the logger with a simple stdout inner logger.
///
/// See [`init_logger_with_inner`] for more options.
pub fn init_logger() -> Result<(), SetLoggerError> {
    init_logger_with_inner(InnerLogger::default())
}

/// Like [`init_logger`], but allows you to set a specific log level.
pub fn init_logger_with_level(level: LevelFilter) -> Result<(), SetLoggerError> {
    init_logger_with_inner_and_level(InnerLogger::default(), level)
}

/// Initializes the logger just with a custom inner logger.
///
/// See [`InnerLogger`] for more information on the different inner loggers.
pub fn init_logger_with_inner(inner: impl Into<InnerLogger>) -> Result<(), SetLoggerError> {
    init_logger_with_inner_and_level(inner, LevelFilter::Trace)
}

/// Like [`init_logger_with_inner`], but allows you to set a specific log level.
pub fn init_logger_with_inner_and_level(
    inner: impl Into<InnerLogger>,
    level: LevelFilter,
) -> Result<(), SetLoggerError> {
    let logger = Box::leak(Box::new(ProgressBarLogger(inner.into())));
    log::set_logger(logger).map(|()| log::set_max_level(level))
}
