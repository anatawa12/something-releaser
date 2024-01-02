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

macro_rules! escapes {
    ($value: expr, $( $pattern: pat => $replacement: expr ),+ $(,)?) => {
        {
            use std::fmt::Write;
            let value = $value;
            let mut builder = String::with_capacity(value.len());

            for c in value.chars() {
                match c {
                    $($pattern => write!(builder, "{}", $replacement).unwrap(),)*
                    _ => builder.push(c),
                }
            }

            builder
        }
    };
}
