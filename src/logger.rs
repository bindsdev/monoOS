use log::{
    Level::{self, *},
    LevelFilter, Log, Metadata, Record,
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
                .unwrap();
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
                Debug => log_inner!("\x1b[35;1m[DEBUG] "),
                Info => log_inner!("\x1b[36;1m[INFO] "),
                Warn => log_inner!("\x1b[33;1m[WARNING] "),
                Error => log_inner!("\x1b[32;1m[ERROR] "),
                Trace => log_inner!("\x1b[34;1m[TRACE] "),
            }

            log_inner!("\x1b[0m({file}:{line}) {}\n", record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SystemLogger = SystemLogger;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap()
}
