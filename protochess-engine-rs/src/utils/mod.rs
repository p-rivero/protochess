mod board;
pub mod perft;
pub mod debug;

pub use board::*;

// Use this to return a formatted error from a function
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => (return Err(format!($($arg)*)))
}

#[macro_export]
macro_rules! err_assert {
    ($cond:expr, $($arg:tt)*) => {
        if !($cond) {
            $crate::err!($($arg)*);
        }
    }
}

// Use this to wrap a type in a Result (with String as the error type)
#[macro_export]
macro_rules! wrap_res {
    ($arg:ty) => (Result<$arg, String>);
    ($($arg:tt)*) => (Result<($($arg)*), String>);
    () => (Result<(), String>)
}
