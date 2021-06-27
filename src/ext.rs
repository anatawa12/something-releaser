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

pub(crate) trait StrExt {
    fn escape_groovy(&self) -> String;
}

impl StrExt for str {
    fn escape_groovy(&self) -> String {
        let mut builder = Vec::<u8>::with_capacity(self.len());
        for x in self.bytes() {
            if x == b'\r' {
                builder.extend_from_slice(b"\\r")
            } else if x == b'\n' {
                builder.extend_from_slice(b"\\n")
            } else if x == b'\\' {
                builder.extend_from_slice(b"\\\\")
            } else if x == b'$' {
                builder.extend_from_slice(b"\\$")
            } else if x == b'\'' {
                builder.extend_from_slice(b"\\\'")
            } else if x == b'\"' {
                builder.extend_from_slice(b"\\\"")
            } else {
                builder.push(x)
            }
        }
        unsafe { String::from_utf8_unchecked(builder) }
    }
}

impl StrExt for String {
    fn escape_groovy(&self) -> String {
        self.as_str().escape_groovy()
    }
}
