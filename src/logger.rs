use core::panic::PanicInfo;
use log::{
    Level::{self, *},
    LevelFilter, Log, Metadata, MetadataBuilder, Record, RecordBuilder,
};

struct SystemLogger;

impl Log for SystemLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let file = record
                .file()
                .map(|s| s.strip_prefix("src/"))
                .flatten()
                .unwrap_or("[external]");
            let line = record.line().unwrap();

            macro log_inner($($arg:tt)*) {
                {
                    if $crate::drivers::graphics::is_initialized() {
                        $crate::drivers::graphics::print!("{}", format_args!($($arg)*));
                    }

                    $crate::drivers::uart::serial_print!("{}", format_args!($($arg)*));
                }
            }

            match record.level() {
                Debug => log_inner!("\x1b[1;30m[DEBUG] "),
                Info => log_inner!("\x1b[1;37m[INFO] "),
                Warn => log_inner!("\x1b[1;94m[WARNING] "),
                Error => log_inner!("\x1b[1;33m[ERROR] "),
                Trace => log_inner!("\x1b[1;36m[TRACE] "),
            }

            if file == "[external]" {
                log_inner!("\x1b[0m({file}) {}\n", record.args());
            } else {
                log_inner!("\x1b[0m({file}:{line}) {}\n", record.args());
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SystemLogger = SystemLogger;

/// Log panics.
pub fn log_panic(info: &PanicInfo<'_>) {
    let location = info.location().unwrap();
    let file = location.file();
    let line = location.line();
    let args = info.message().unwrap();

    let record = RecordBuilder::new()
        .file(Some(file))
        .line(Some(line))
        .args(*args)
        .metadata(MetadataBuilder::new().level(Level::Error).build())
        .build();

    LOGGER.log(&record);
}

/// Initialize the system logger.
pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap()
}
