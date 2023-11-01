use std::{
    fmt::{Arguments, Formatter},
    fs::OpenOptions,
    io::{Error, Write},
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex, Once},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

pub struct Logger {
    format: Box<dyn LoggerFormat>,
    log_level: LogLevel,
    log_file: Mutex<std::fs::File>,
}

static mut GLOBAL_LOGGER: Option<Arc<Logger>> = None;
static GLOBAL_LOGGER_INIT: Once = Once::new();

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[used]
#[cfg_attr(target_os = "linux", link_section = ".fini_array.65534")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XPTZ65534")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_term_func")]
pub static __global_logger_destructor: extern "C" fn() = {
    extern "C" fn __drop_global_logger() {
        if let Some(pool) = unsafe { GLOBAL_LOGGER.take() } {
            drop(pool)
        }
    }
    __drop_global_logger
};

pub fn init_global_logger<T: LoggerFormat + 'static>(file: PathBuf, format: Option<T>) {
    GLOBAL_LOGGER_INIT.call_once(|| unsafe {
        let _ = &*GLOBAL_LOGGER.get_or_insert(Arc::new(
            Logger::new(file, format).expect("failed to init logger"),
        ));
    })
}

#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {
        $crate::global_log(
            format_args!($($arg)+),
            $lvl,
        (
            $target,
            module_path!(),
            file!()
        ),
        line!()
        )
    };
    ($lvl:expr, $($arg:tt)+) => { $crate::log!(target: module_path!(), $lvl, $($arg)+) }
}

#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::log!( target: $target, $crate::LogLevel::Error, $($arg)+)
    };
    ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Error, $($arg)+) }
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::log!( target: $target, $crate::LogLevel::Warn, $($arg)+)
    };
    ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Warn, $($arg)+) }
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::log!( target: $target, $crate::LogLevel::Info, $($arg)+)
    };
    ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Info, $($arg)+) }
}

#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::log!( target: $target, $crate::LogLevel::Debug, $($arg)+)
    };
    ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Debug, $($arg)+) }
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::log!( target: $target, $crate::LogLevel::Trace, $($arg)+)
    };
    ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Trace, $($arg)+) }
}

pub fn global_log(
    args: Arguments,
    lvl: LogLevel,
    (target, module, file): (&str, &str, &str),
    line: u32,
) {
    if let Some(global_log) = unsafe { &GLOBAL_LOGGER } {
        global_log.log(args, lvl, (target, module, file), line)
    }
}

macro_rules! cyan {
    ($str:literal) => {
        concat!("\x1b[0;36m", $str, "\x1b[0m")
    };
}

macro_rules! red {
    ($str:literal) => {
        concat!("\x1b[0;31m", $str, "\x1b[0m")
    };
}

macro_rules! yellow {
    ($str:literal) => {
        concat!("\x1b[0;33m", $str, "\x1b[0m")
    };
}

macro_rules! green {
    ($str:literal) => {
        concat!("\x1b[0;32m", $str, "\x1b[0m")
    };
}

macro_rules! purple {
    ($str:literal) => {
        concat!("\x1b[0;35m", $str, "\x1b[0m")
    };
}

macro_rules! gray {
    ($str:literal) => {
        concat!("\x1b[0;90m", $str, "\x1b[0m")
    };
}

const SECONDS_IN_A_DAY: u64 = SECONDS_IN_AN_HOUR * 24;
const SECONDS_IN_AN_HOUR: u64 = SECONDS_IN_A_MINUTE * 60;
const SECONDS_IN_A_MINUTE: u64 = 60;
const DAYS_IN_A_YEAR: u64 = 365;

const UNIX_EPOCH_YEAR: u64 = 1970;

pub fn generate_utc_string() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before unix epoch");
    let mut seconds = timestamp.as_secs();

    let days = seconds / SECONDS_IN_A_DAY;
    seconds %= SECONDS_IN_A_DAY;

    let hours = seconds / SECONDS_IN_AN_HOUR;
    seconds %= SECONDS_IN_AN_HOUR;

    let minutes = seconds / SECONDS_IN_A_MINUTE;
    seconds %= SECONDS_IN_A_MINUTE;

    let mut days_from_epoch = days;
    let mut year = UNIX_EPOCH_YEAR;
    let mut month = 1;
    let mut day = 1;

    let is_leap_year = |year| (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    loop {
        let days_in_year = if is_leap_year(year) {
            DAYS_IN_A_YEAR + 1
        } else {
            DAYS_IN_A_YEAR
        };

        if days_from_epoch >= days_in_year {
            days_from_epoch -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let days_in_month = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    for (i, &days_in_current_month) in days_in_month.iter().enumerate() {
        month = i + 1;
        if days_from_epoch >= days_in_current_month {
            days_from_epoch -= days_in_current_month;
        } else {
            day += days_from_epoch as i32;
            break;
        }
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    #[default]
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "Error"),
            LogLevel::Warn => write!(f, "Warn "),
            LogLevel::Info => write!(f, "Info "),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Trace => write!(f, "Trace"),
        }
    }
}

impl LogLevel {
    pub fn to_pretty_string(&self) -> &'static str {
        match self {
            LogLevel::Error => red!("Error"),
            LogLevel::Warn => yellow!("Warn "),
            LogLevel::Info => green!("Info "),
            LogLevel::Debug => cyan!("Debug"),
            LogLevel::Trace => purple!("Trace"),
        }
    }
}

#[derive(Debug, Error)]
pub enum LogLevelParseError {
    #[error("invalid log level string")]
    InvalidLogLevel,
}

impl FromStr for LogLevel {
    type Err = LogLevelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(LogLevelParseError::InvalidLogLevel),
        }
    }
}

impl Logger {
    /// Format view:
    /// ```norust
    /// "%t %T %c %f:%l %s"
    /// "%r"
    /// ```
    pub fn new<T: LoggerFormat + 'static>(path: PathBuf, format: Option<T>) -> Result<Self, Error> {
        let log_file = OpenOptions::new().create(true).append(true).open(path)?;
        let log_level = std::env::var("RUST_LOG");
        let log_level = match log_level {
            Ok(s) => s.parse().unwrap_or(LogLevel::default()),
            Err(_) => LogLevel::default(),
        };
        Ok(Self {
            format: if let Some(formatter) = format {
                Box::new(formatter)
            } else {
                Box::new(DefaultLogger {})
            },
            log_level,
            log_file: log_file.into(),
        })
    }

    pub fn log(
        &self,
        args: Arguments,
        lvl: LogLevel,
        (target, module, file): (&str, &str, &str),
        line: u32,
    ) {
        if lvl > self.log_level {
            return;
        }
        self.log_file
            .lock()
            .unwrap()
            .write_all(
                self.format
                    .render(args, lvl, (target, module, file), line)
                    .as_bytes(),
            )
            .unwrap();
        println!(
            "{}",
            self.format
                .render_pretty(args, lvl, (target, module, file), line)
        );
    }
}

pub trait LoggerFormat {
    fn render_pretty(
        &self,
        args: Arguments,
        lvl: LogLevel,
        target_module_file_tuple: (&str, &str, &str),
        line: u32,
    ) -> String {
        self.render(args, lvl, target_module_file_tuple, line)
    }

    fn render(
        &self,
        args: Arguments,
        lvl: LogLevel,
        target_module_file_tuple: (&str, &str, &str),
        line: u32,
    ) -> String;
}

pub struct DefaultLogger {}

impl LoggerFormat for DefaultLogger {
    fn render(
        &self,
        args: Arguments,
        lvl: LogLevel,
        (target, module, file): (&str, &str, &str),
        line: u32,
    ) -> String {
        format!(
            "[{}] {lvl:5} [{module} {file}:{line}] {target}: {args}\n",
            generate_utc_string()
        )
    }

    fn render_pretty(
        &self,
        args: Arguments,
        lvl: LogLevel,
        (target, module, file): (&str, &str, &str),
        line: u32,
    ) -> String {
        format!(
            concat!(
                "[{}] {} ",
                light_gray!("[{} {}:{}] "),
                purple!("{}"),
                ": {}"
            ),
            generate_utc_string(),
            lvl.to_pretty_string(),
            module,
            file,
            line,
            target,
            args
        )
    }
}
