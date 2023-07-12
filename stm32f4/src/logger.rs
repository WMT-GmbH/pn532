use crate::println;

pub fn init_logger(level: log::LevelFilter) {
    unsafe {
        log::set_logger_racy(&UsbLogger).unwrap();
    }

    log::set_max_level(level);
}

struct UsbLogger;

impl log::Log for UsbLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        const RESET: &str = "\u{001B}[0m";
        const RED: &str = "\u{001B}[31m";
        const GREEN: &str = "\u{001B}[32m";
        const YELLOW: &str = "\u{001B}[33m";
        const BLUE: &str = "\u{001B}[34m";
        const CYAN: &str = "\u{001B}[35m";

        let color = match record.level() {
            log::Level::Error => RED,
            log::Level::Warn => YELLOW,
            log::Level::Info => GREEN,
            log::Level::Debug => BLUE,
            log::Level::Trace => CYAN,
        };
        let reset = RESET;

        println!("{}{} - {}{}", color, record.level(), record.args(), reset);
    }

    fn flush(&self) {}
}
