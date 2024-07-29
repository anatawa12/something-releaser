macro_rules! ok {
    () => {{
        return ::core::result::Result::Ok(());
    }};
}

macro_rules! err {
    () => {
        {
            return ::core::result::Result::Err(::core::num::NonZeroI32::new(1).unwrap());
        }
    };
    ($($tt:tt)*) => {
        {
            eprintln!($($tt)*);
            return ::core::result::Result::Err(::core::num::NonZeroI32::new(1).unwrap());
        }
    };
}

macro_rules! escapes {
    (
        $value: expr,
        $( @prefix = $prefix: expr, )?
        $( @suffix = $suffix: expr, )?
        $( $pattern: pat => $replacement: expr ),+ $(,)?
    ) => {
        {
            use std::fmt::Write;
            let value = $value;
            $(let prefix = $prefix;)?
            $(let suffix = $suffix;)?
            let mut builder = ::std::string::String::with_capacity(
                value.len()
                + escapes!(@zero [$($prefix)?] [prefix.len()] [0])
                + escapes!(@zero [$($suffix)?] [suffix.len()] [0]) 
            );

            escapes!(@zero [$($prefix)?] [builder.push_str(prefix)] []);

            for c in value.chars() {
                match c {
                    $($pattern => write!(builder, "{}", $replacement).unwrap(),)*
                    _ => builder.push(c),
                }
            }

            escapes!(@zero [$($suffix)?] [builder.push_str(suffix)] []);

            builder
        }
    };

    (@zero [] [$($_t:tt)*] [$($tt:tt)*]) => {
        $($tt)*
    };
    (@zero [$($_t:tt)*] [$($tt:tt)*] [$($_2:tt)*]) => {
        $($tt)*
    };
}
