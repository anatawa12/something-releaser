macro_rules! ok {
    () => {{
        return ::core::result::Result::Ok(());
    }};
}

macro_rules! err {
    ($($tt:tt)*) => {
        {
            eprintln!($($tt)*);
            return ::core::result::Result::Err(::core::num::NonZeroI32::new(1).unwrap());
        }
    };
}
