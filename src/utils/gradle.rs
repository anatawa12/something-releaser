use std::env;
use std::path::PathBuf;

pub fn gradle_home_dir() -> PathBuf {
    env::var("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut home = home::home_dir().expect("failed to get home directory");
            home.push(".gradle");
            home
        })
}

pub fn escape_groovy_string(s: &str) -> String {
    escapes!(
        s,
        '\n' => "\\n",
        '\r' => "\\r",
        '\t' => "\\t",
        '\'' => "\\'",
        '\\' => "\\\\",
        '"' => "\\\"",
        '$' => "\\$",
        c@'\x00'..='\x1F' => format_args!("\\u{:04x}", c as u32),
    )
}
