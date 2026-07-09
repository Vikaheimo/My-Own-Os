use log::LevelFilter;

use crate::serial_println;

pub struct KernelLogger;

pub const MAX_LOG_LEVEL: LevelFilter = {
    #[cfg(kernel_log_level = "off")]
    {
        LevelFilter::Off
    }

    #[cfg(kernel_log_level = "error")]
    {
        LevelFilter::Error
    }

    #[cfg(kernel_log_level = "warn")]
    {
        LevelFilter::Warn
    }

    #[cfg(kernel_log_level = "info")]
    {
        LevelFilter::Info
    }

    #[cfg(kernel_log_level = "debug")]
    {
        LevelFilter::Debug
    }

    #[cfg(kernel_log_level = "trace")]
    {
        LevelFilter::Trace
    }

    #[cfg(not(any(
        kernel_log_level = "off",
        kernel_log_level = "error",
        kernel_log_level = "warn",
        kernel_log_level = "info",
        kernel_log_level = "debug",
        kernel_log_level = "trace",
    )))]
    {
        LevelFilter::Debug
    }
};

static LOGGER: KernelLogger = KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= MAX_LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        let uptime = crate::time::uptime();
        serial_println!(
            "{} {:<5} {}:{}  {}",
            LogDurationFormat::from(uptime),
            record.level(),
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0),
            record.args()
        );
    }

    fn flush(&self) {
        // This is a no-op as we send to serial
    }
}

pub fn init() {
    #[allow(clippy::expect_used)]
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(MAX_LOG_LEVEL))
        .expect("Failed to initialize logger!");
}

#[derive(Debug)]
struct LogDurationFormat {
    hours: u64,
    minutes: u64,
    seconds: u64,
    milliseconds: u32,
}

impl core::fmt::Display for LogDurationFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[{:02}:{:02}:{:02}.{:03}]",
            self.hours, self.minutes, self.seconds, self.milliseconds
        )
    }
}

const SECONDS_IN_HOUR: u64 = 3600;
const SECONDS_IN_MINUTE: u64 = 60;

impl From<core::time::Duration> for LogDurationFormat {
    fn from(value: core::time::Duration) -> Self {
        let full_seconds = value.as_secs();
        let hours = full_seconds / SECONDS_IN_HOUR;
        let minutes = (full_seconds % SECONDS_IN_HOUR) / SECONDS_IN_MINUTE;
        let seconds = full_seconds % SECONDS_IN_MINUTE;
        let milliseconds = value.subsec_millis();

        Self {
            milliseconds,
            seconds,
            minutes,
            hours,
        }
    }
}
