use core::cell::Cell;

use critical_section::{CriticalSection, Mutex};

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            writeln!($crate::println::Printer, $($arg)*).ok();
        }
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            write!($crate::println::Printer, $($arg)*).ok();
        }
    }};
}

// implementation adapted from `std::dbg`
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", ::core::file!(), ::core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    ::core::file!(), ::core::line!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|cs| {
            let f = PRINT_FUNC.borrow(cs).get();
            f(cs, s)
        })
    }
}

pub type PrintFunc = fn(cs: CriticalSection, s: &str) -> core::fmt::Result;

static PRINT_FUNC: Mutex<Cell<PrintFunc>> = Mutex::new(Cell::new(|_, _| Err(core::fmt::Error)));

pub fn set_print_func(cs: CriticalSection, f: PrintFunc) {
    PRINT_FUNC.borrow(cs).set(f);
}
