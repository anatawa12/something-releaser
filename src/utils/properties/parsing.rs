use crate::utils::properties::{Element, KeyValuePair, ParsedValueLine, Result, Value};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::Chars;

use ParseError::*;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedEOL(usize),
    InvalidChar(usize),
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnexpectedEOF => write!(f, "unexpected end of file"),
            UnexpectedEOL(usize) => write!(f, "unexpected end of line at line {}", usize),
            InvalidChar(usize) => write!(f, "invalid character at line {}", usize),
        }
    }
}

pub(super) struct EscapeParser<I: Iterator<Item = char>> {
    chars: I,
    line: usize,
    pub ends_with_backslash: bool,
}

impl<'a> EscapeParser<Chars<'a>> {
    pub fn new(chars: Chars<'a>, line: usize) -> Self {
        Self {
            chars,
            line,
            ends_with_backslash: false,
        }
    }

    pub fn next_char(&mut self) -> Result<Option<(bool, char, &'a str)>> {
        let str = self.chars.clone().as_str();
        Ok(match self.chars.next() {
            Some('\\') => {
                // escaped
                match self.chars.next() {
                    Some('n') => Some((true, '\n', "\\n")),
                    Some('r') => Some((true, '\r', "\\r")),
                    Some('t') => Some((true, '\t', "\\t")),
                    Some('f') => Some((true, '\x0C', "\\f")), // form feed
                    Some('u') => {
                        let mut code = 0;
                        for _ in 0..4 {
                            let c = self.chars.next().ok_or(UnexpectedEOL(self.line))?;
                            code = code * 16 + c.to_digit(16).ok_or(InvalidChar(self.line))?;
                        }
                        Some((true, std::char::from_u32(code).unwrap(), &str[..6]))
                        // TODO: support surrogate
                    }
                    Some(other) => Some((true, other, str[..1 + other.len_utf8()].into())),
                    None => {
                        self.ends_with_backslash = true;
                        None
                    }
                }
            }
            Some(other) => Some((false, other, &str[..other.len_utf8()])),
            None => None,
        })
    }
}

fn split_to_blank_and_rest(line: &str) -> (&str, &str) {
    line.find(|c: char| !c.is_ascii_whitespace())
        .map(|i| line.split_at(i))
        .unwrap_or((line, ""))
}

struct KvpLineParser<'a> {
    chars: EscapeParser<Chars<'a>>,
    key: String,
    key_parsed: String,
    sep: String,
    value: String,
    value_parsed: String,
}

impl<'a> KvpLineParser<'a> {
    pub fn new(line: &'a str, line_num: usize) -> Self {
        Self {
            chars: EscapeParser::new(line.chars(), line_num),
            key: String::new(),
            key_parsed: String::new(),
            sep: String::new(),
            value: String::new(),
            value_parsed: String::new(),
        }
    }

    pub fn do_parse(&mut self) -> Result<bool> {
        // loop while key and separator
        loop {
            match self.chars.next_char()? {
                None => return Ok(self.chars.ends_with_backslash),
                Some((false, c @ (':' | '='), _)) => {
                    // END OF KEY AND SEPARATOR
                    self.sep.push(c);

                    return self.parse_after_sep_char();
                }
                Some((false, c @ (' ' | '\t' | '\x0C'), _)) => {
                    self.sep.push(c);

                    loop {
                        match self.chars.next_char()? {
                            None => return Ok(self.chars.ends_with_backslash),
                            Some((false, c @ (' ' | '\t' | '\x0C'), _)) => self.sep.push(c),
                            Some((false, c @ (':' | '='), _)) => {
                                self.sep.push(c);
                                break;
                            }
                            Some((_, c, esc)) => return self.parse_value(c, esc),
                        }
                    }

                    return self.parse_after_sep_char();
                }
                Some((_, c, esc)) => {
                    self.key_parsed.push(c);
                    self.key.push_str(esc);
                }
            }
        }
    }

    fn parse_after_sep_char(&mut self) -> Result<bool> {
        // appends whitespaces to separator and parse value
        loop {
            match self.chars.next_char()? {
                None => return Ok(self.chars.ends_with_backslash),
                Some((false, c @ (' ' | '\t' | '\x0C'), _)) => self.sep.push(c),
                Some((_, c, esc)) => {
                    return self.parse_value(c, esc);
                }
            }
        }
    }

    fn parse_value(&mut self, first_char: char, first: &str) -> Result<bool> {
        // appends whitespaces to separator and parse value
        self.value.push_str(first);
        self.value_parsed.push(first_char);

        while let Some((_, c, esc)) = self.chars.next_char()? {
            self.value.push_str(esc);
            self.value_parsed.push(c);
        }

        Ok(self.chars.ends_with_backslash)
    }
}

pub(super) fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Vec<Element>> {
    let mut lines = lines.enumerate();
    let mut result = vec![];

    while let Some((line_num, line_in)) = lines.next() {
        let (blank_before_key, line) = split_to_blank_and_rest(line_in);
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            result.push(Element::BlankOrComment(line_in.to_owned()));
            continue;
        }

        let mut parser = KvpLineParser::new(line, line_num);
        let ends_with_backslash = parser.do_parse()?;

        let mut value_lines = vec![
            ParsedValueLine {
                blank: "".into(),
                line: parser.value,
                parsed: parser.value_parsed,
            }
        ];

        macro_rules! kvp {
            () => {
                Element::KeyValuePair(KeyValuePair {
                    blank_before_key: blank_before_key.to_owned(),
                    key: Some(parser.key),
                    key_parsed: parser.key_parsed,
                    separator: parser.sep,
                    value: Value::Parsed { lines: value_lines },
                })
            };
        }

        if ends_with_backslash {
            // parse multi line value
            loop {
                let (line_num, line) = lines.next().ok_or(UnexpectedEOF)?;
                let (blank, line) = split_to_blank_and_rest(line);
                if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                    // when we encounter blank or comment, we should not append it to value_lines
                    result.push(kvp!());

                    result.push(Element::BlankOrComment(line.to_owned()));
                    break;
                }

                let mut line_parse = EscapeParser::new(line.chars(), line_num);

                let mut line_parsed = String::new();
                while let Some((_, c, _)) = line_parse.next_char()? {
                    line_parsed.push(c);
                }

                if !line_parse.ends_with_backslash {
                    value_lines.push(ParsedValueLine {
                        blank: blank.to_owned(),
                        line: line.to_owned(),
                        parsed: line_parsed,
                    });
                    result.push(kvp!());
                    break;
                }

                let trimmed = line.strip_suffix('\\').unwrap().to_owned();

                value_lines.push(ParsedValueLine {
                    blank: blank.to_owned(),
                    line: trimmed.to_owned(),
                    parsed: line_parsed,
                });
            }
        } else {
            result.push(kvp!());
            continue;
        }
    }

    Ok(result)
}

pub(crate) fn parse_value(value: &Value) -> String {
    match value  {
        Value::Provided { value } => value.to_owned(),
        Value::Parsed { lines, .. } => {
            lines.iter().map(|x| x.parsed.as_str()).collect()
        }
    }
}
