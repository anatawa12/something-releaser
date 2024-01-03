//! Properties file modification utility

use crate::utils::properties::parsing::ParseError;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::{fmt, io};

mod parsing;

#[cfg(test)]
mod tests;

type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug)]
pub(crate) struct PropertiesFile {
    elements: Vec<Element>,
}

impl PropertiesFile {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub async fn load_may_not_exist(path: impl AsRef<Path>) -> io::Result<Self> {
        match tokio::fs::read_to_string(path).await {
            Ok(string) => {
                Self::from_str(&string).map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(e),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.elements
            .iter()
            .filter_map(|x| x.as_kvp())
            .find(|x| x.key_parsed == key)
            .map(|x| parsing::parse_value(&x.value))
    }

    pub fn set(&mut self, key: &str, value: String) {
        if let Some(kvp) = self
            .elements
            .iter_mut()
            .filter_map(|x| x.as_kvp_mut())
            .find(|x| x.key_parsed == key)
        {
            kvp.value = Value::Provided { value };
        } else {
            self.elements.push(Element::KeyValuePair(KeyValuePair {
                blank_before_key: String::new(),
                key: None,
                key_parsed: key.to_owned(),
                separator: String::from("="),
                value: Value::Provided { value },
            }))
        }
    }
}

impl FromStr for PropertiesFile {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        parsing::parse(s.lines()).map(|elements| Self { elements })
    }
}

impl Display for PropertiesFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_to_source(f, &self.elements)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Element {
    KeyValuePair(KeyValuePair),
    BlankOrComment(String),
}

impl Element {
    fn as_kvp(&self) -> Option<&KeyValuePair> {
        match self {
            Element::KeyValuePair(kvp) => Some(kvp),
            _ => None,
        }
    }

    fn as_kvp_mut(&mut self) -> Option<&mut KeyValuePair> {
        match self {
            Element::KeyValuePair(kvp) => Some(kvp),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct KeyValuePair {
    blank_before_key: String,
    key: Option<String>,
    key_parsed: String,
    separator: String,
    value: Value,
}

#[derive(Debug, Eq, PartialEq)]
enum Value {
    Parsed { lines: Vec<ParsedValueLine> },
    Provided { value: String },
}

#[derive(Debug, Eq, PartialEq)]
struct ParsedValueLine {
    blank: String,
    line: String,
    parsed: String,
}

//////////// writing ////////////

fn write_to_source(f: &mut Formatter, elements: &[Element]) -> fmt::Result {
    for element in elements {
        match element {
            Element::KeyValuePair(kvp) => {
                write_key_value_pair(f, kvp)?;
            }
            Element::BlankOrComment(line) => {
                writeln!(f, "{}", line)?;
            }
        }
    }
    Ok(())
}

fn write_key_value_pair(f: &mut Formatter, pair: &KeyValuePair) -> fmt::Result {
    write!(f, "{}", pair.blank_before_key)?;
    if let Some(key) = &pair.key {
        write!(f, "{}", key)?;
    } else {
        write_escaped(f, &pair.key_parsed)?;
    }
    write!(f, "{}", pair.separator)?;
    match &pair.value {
        Value::Parsed { lines } => {
            let mut lines = lines.iter();
            write!(f, "{}", lines.next().unwrap().line)?;
            for ParsedValueLine { blank, line, parsed: _ }in lines {
                write!(f, "\\\n{}{}", blank, line)?;
            }
            writeln!(f)?;
        }
        Value::Provided { value } => {
            write_escaped(f, value)?;
            writeln!(f)?;
        }
    }
    Ok(())
}

fn is_special_char(c: char) -> bool {
    matches!(
        c,
        '\\' | '\t' | '\n' | '\r' | '\x0c' | '=' | ':' | '#' | '!'
    ) || c.is_control()
}

fn write_escaped(f: &mut Formatter, s: &str) -> fmt::Result {
    if !s.starts_with(' ') && s.contains(is_special_char) {
        return write!(f, "{}", s);
    }

    let mut first = true;

    for char in s.chars() {
        match char {
            ' ' if first => write!(f, "\\ ")?,
            '=' | ':' | '#' | '!' if first => write!(f, "\\{}", char)?,
            '\t' => write!(f, "\\t")?,
            '\n' => write!(f, "\\n")?,
            '\r' => write!(f, "\\r")?,
            '\x0c' => write!(f, "\\f")?,
            '\\' => write!(f, "\\\\")?,
            _ => write!(f, "{}", char)?,
        }
        first = false;
    }

    Ok(())
}
