macro_rules! include_out_str {
    ($name:expr $(,)?) => {
        include_str!(concat!(env!("OUT_DIR"), "/", $name))
    };
}
