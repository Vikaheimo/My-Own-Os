use crate::serial_println;

pub struct KernelLogger;

static MAX_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Trace;

static LOGGER: KernelLogger = KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= MAX_LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        let ms = crate::time::uptime_ms();
        serial_println!(
            "[{:>8}.{:03} ms] {:<5} {}:{}  {}",
            ms / 1000,
            ms % 1000,
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
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(MAX_LOG_LEVEL))
        .expect("Failed to initialize logger");
}
