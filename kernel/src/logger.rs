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
        let total_secs = uptime.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = uptime.subsec_millis();

        serial_println!(
            "[{:02}:{:02}:{:02}.{:03}] {:<5} {}:{}  {}",
            hours,
            minutes,
            seconds,
            millis,
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
