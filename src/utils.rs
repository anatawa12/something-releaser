pub(crate) mod json;

use std::env::Args;
use std::str::FromStr;

pub trait ArgsExt {
    fn next_parsed_or<T: FromStr>(&mut self, default: T) -> T;
}

impl ArgsExt for Args {
    fn next_parsed_or<T: FromStr>(&mut self, default: T) -> T {
        self.next().and_then(|s| s.parse().ok()).unwrap_or(default)
    }
}
