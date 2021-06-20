use std::fmt::{Debug, Display};

pub(crate) trait ResultExt<T, R> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T;
}

impl<T, R: Debug> ResultExt<T, R> for Result<T, R> {
    fn expect_fn<S: Display, F: FnOnce() -> S>(self, func: F) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic!("{}: {:?}", func(), e),
        }
    }
}
