use std::error::Error;
use std::fmt::{Display, Formatter};

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
pub struct PropertiesFile<'s> {
    body: Vec<FileElement<'s>>,
}

#[allow(dead_code)]
impl<'s> PropertiesFile<'s> {
    pub fn parse(source: &'s str) -> Result<PropertiesFile<'s>, ParsingError> {
        parsing::parse_file(source.lines())
    }

    fn find_ref(&self, key: &str) -> Option<&KeyValuePair> {
        self.body
            .iter()
            .filter_map(|elem| {
                if let FileElement::KeyValuePair(pair) = elem {
                    Some(pair)
                } else {
                    None
                }
            })
            .find(|pair| pair.key_parsed.as_str() == key)
    }

    fn find_ref_mut(&mut self, key: &str) -> Option<&mut KeyValuePair<'s>> {
        self.body
            .iter_mut()
            .filter_map(|elem| {
                if let FileElement::KeyValuePair(pair) = elem {
                    Some(pair)
                } else {
                    None
                }
            })
            .find(|pair| pair.key_parsed.as_str() == key)
    }

    pub fn find_value(&self, key: &str) -> Option<String> {
        self.find_ref(key).map(|x| parsing::parse_value(x))
    }

    pub fn set_value(&mut self, key: String, value: String) {
        use PropertyValue::ActualValue;

        if let Some(pair) = self.find_ref_mut(&key) {
            pair.value = ActualValue(value)
        } else {
            let pair = KeyValuePair {
                key_parsed: key,
                blank: "",
                key: None,
                separator: ":",
                value: ActualValue(value),
            };
            self.body.push(FileElement::KeyValuePair(pair))
        }
    }

    pub fn write<W: std::io::Write>(&self, out: &mut W) -> std::io::Result<()> {
        writing::write(self, out)
    }
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
enum FileElement<'s> {
    KeyValuePair(KeyValuePair<'s>),
    SkipLine(&'s str),
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
struct KeyValuePair<'s> {
    key_parsed: String,

    blank: &'s str,
    key: Option<&'s str>,
    separator: &'s str,
    value: PropertyValue<'s>,
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
enum PropertyValue<'s> {
    Parsed(Vec<BlankValuePair<'s>>),
    ActualValue(String),
}

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Debug)]
struct BlankValuePair<'s> {
    blank: &'s str,
    value: &'s str,
}

#[derive(Debug)]
pub struct ParsingError {
    inner: ParsingErrorInner,
}

#[derive(Debug)]
pub enum ParsingErrorInner {
    InvalidCharAt { line: usize, col: usize },
    UnexpectedEOL { line: usize },
    UnexpectedEOF,
}

impl From<ParsingErrorInner> for ParsingError {
    fn from(inner: ParsingErrorInner) -> Self {
        Self { inner }
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ParsingErrorInner::InvalidCharAt { line, col } => {
                write!(f, "invalid character at line {} col {}", line, col)
            }
            ParsingErrorInner::UnexpectedEOL { line } => {
                write!(f, "unexpected end of line at line {}", line)
            }
            ParsingErrorInner::UnexpectedEOF => write!(f, "unexpected eof"),
        }
    }
}

impl Error for ParsingError {}

impl From<ParsingError> for std::io::Error {
    fn from(e: ParsingError) -> Self {
        Self::new(std::io::ErrorKind::InvalidData, e)
    }
}

mod parsing {
    use super::*;

    enum ParseLineResult<T> {
        CommentOrBlankLine,
        ContinueLine(T),
        EndValueLine(T),
    }

    impl<T> ParseLineResult<T> {
        fn map_value<R, F: FnOnce(T) -> R>(self, func: F) -> ParseLineResult<R> {
            use ParseLineResult::*;
            match self {
                CommentOrBlankLine => CommentOrBlankLine,
                ContinueLine(v) => ContinueLine(func(v)),
                EndValueLine(v) => EndValueLine(func(v)),
            }
        }

        fn expect_continue(self, msg: &str) -> T {
            use ParseLineResult::*;
            match self {
                CommentOrBlankLine => panic!("expects continue but was blank: {}", msg),
                EndValueLine(_) => panic!("expects continue but was end of value: {}", msg),
                ContinueLine(v) => v,
            }
        }

        fn expect_end_value(self, msg: &str) -> T {
            use ParseLineResult::*;
            match self {
                CommentOrBlankLine => panic!("expects end of value but was blank: {}", msg),
                ContinueLine(_) => panic!("expects end of value but was continue: {}", msg),
                EndValueLine(v) => v,
            }
        }
    }

    struct ParseLineSuccess {
        // end of blank before key (exclusive) or start of key(inclusive)
        blank_end: usize,
        // the parsed key value
        key: String,
        // end of key (exclusive) or separator blank or char(inclusive)
        key_end: usize,
        // end of separator blank or char(exclusive) or start of value
        sep_end: usize,
    }

    #[derive(Debug)]
    enum ParseLineErr {
        InvalidCharAt(usize),
        UnexpectedEOL,
    }

    impl ParseLineErr {
        pub(super) fn parsing(self, line: usize) -> ParsingError {
            match self {
                ParseLineErr::InvalidCharAt(col) => {
                    ParsingErrorInner::InvalidCharAt { line, col }.into()
                }
                ParseLineErr::UnexpectedEOL => ParsingErrorInner::UnexpectedEOL { line }.into(),
            }
        }
    }

    fn key_value_pair_from(line: &str, start: ParseLineSuccess) -> KeyValuePair {
        KeyValuePair {
            blank: &line[0..start.blank_end],
            key: (&line[start.blank_end..start.key_end]).into(),
            key_parsed: start.key,
            separator: &line[start.key_end..start.sep_end],
            value: PropertyValue::Parsed(vec![BlankValuePair {
                blank: "",
                value: &line[start.sep_end..],
            }]),
        }
    }

    pub(super) fn parse_file<'a, I: IntoIterator<Item = &'a str>>(
        lines: I,
    ) -> Result<PropertiesFile<'a>, ParsingError> {
        use ParsingState::*;
        enum ParsingState<'s> {
            Start,
            InContinue(KeyValuePair<'s>),
        }

        let mut elements = Vec::<FileElement<'a>>::new();

        let mut stat: ParsingState<'a> = Start;

        fn add_trailing<'a>(pair: &mut KeyValuePair<'a>, blank: BlankValuePair<'a>) {
            if let PropertyValue::Parsed(trailing) = &mut pair.value {
                trailing.push(blank);
            } else {
                panic!("pair.value must be parsed")
            }
        }

        for (l_num, line) in lines.into_iter().enumerate() {
            let line: &'a str = line;
            stat = match stat {
                Start => match parse_key_value_line(line).map_err(|x| x.parsing(l_num))? {
                    ParseLineResult::CommentOrBlankLine => {
                        elements.push(FileElement::SkipLine(line));
                        Start
                    }
                    ParseLineResult::EndValueLine(data) => {
                        elements.push(FileElement::KeyValuePair(key_value_pair_from(line, data)));
                        Start
                    }
                    ParseLineResult::ContinueLine(data) => {
                        InContinue(key_value_pair_from(&line[..line.len() - 1], data))
                    }
                },
                InContinue(mut pair) => {
                    match parse_continuous_line(line).map_err(|x| x.parsing(l_num))? {
                        ParseLineResult::CommentOrBlankLine => {
                            elements.push(FileElement::KeyValuePair(pair));
                            elements.push(FileElement::SkipLine(line));
                            Start
                        }
                        ParseLineResult::EndValueLine(data) => {
                            add_trailing(&mut pair, data);
                            elements.push(FileElement::KeyValuePair(pair));
                            Start
                        }
                        ParseLineResult::ContinueLine(data) => {
                            add_trailing(&mut pair, data);
                            InContinue(pair)
                        }
                    }
                }
            }
        }

        match stat {
            Start => Ok(PropertiesFile { body: elements }),
            InContinue(_) => Err(ParsingErrorInner::UnexpectedEOF.into()),
        }
    }

    fn parse_key_value_line(line: &str) -> Result<ParseLineResult<ParseLineSuccess>, ParseLineErr> {
        enum KeyState {
            Building(String),
            BuiltBeforeSepBlank(ParseLineSuccess),
            BuiltAfterSepBlank(ParseLineSuccess),
            Built(ParseLineSuccess),
        }

        let mut key_state = KeyState::Building(String::with_capacity(line.len() / 2));

        // utils
        fn continue_building(mut key: String, c: char) -> KeyState {
            key += c.encode_utf8(&mut [0; 4]);
            KeyState::Building(key)
        }
        fn mk_success(key: String, at: usize) -> ParseLineSuccess {
            ParseLineSuccess {
                blank_end: 0,
                key,
                key_end: at,
                sep_end: 0,
            }
        }
        fn end_sep(suc: ParseLineSuccess, at: usize) -> ParseLineSuccess {
            ParseLineSuccess { sep_end: at, ..suc }
        }

        let result = parse_line(line, |escaped, c, i| {
            use KeyState::*;
            let mut data = unsafe { std::ptr::read(&mut key_state) };
            data = match data {
                Building(key) => {
                    if !escaped {
                        match c {
                            ':' | '=' => BuiltAfterSepBlank(mk_success(key, i)),
                            ' ' | '\t' | '\x0c' => BuiltBeforeSepBlank(mk_success(key, i)),
                            _ => continue_building(key, c),
                        }
                    } else {
                        continue_building(key, c)
                    }
                }
                BuiltBeforeSepBlank(suc) => {
                    if !escaped {
                        match c {
                            ':' | '=' => BuiltAfterSepBlank(suc),
                            ' ' | '\t' | '\x0c' => BuiltBeforeSepBlank(suc),
                            _ => Built(end_sep(suc, i)),
                        }
                    } else {
                        Built(end_sep(suc, i))
                    }
                }
                BuiltAfterSepBlank(suc) => {
                    if !escaped {
                        match c {
                            ' ' | '\t' | '\x0c' => BuiltAfterSepBlank(suc),
                            _ => Built(end_sep(suc, i)),
                        }
                    } else {
                        Built(end_sep(suc, i))
                    }
                }
                Built(suc) => Built(suc),
            };
            unsafe { std::ptr::write(&mut key_state, data) };
        })?;

        let line_len = match result {
            ParseLineResult::CommentOrBlankLine => line.len(),
            ParseLineResult::EndValueLine(_) => line.len(),
            ParseLineResult::ContinueLine(_) => line.len() - 1,
        };

        let suc = match key_state {
            KeyState::Building(key) => end_sep(mk_success(key, line_len), line_len),
            KeyState::BuiltBeforeSepBlank(suc) => end_sep(suc, line_len),
            KeyState::BuiltAfterSepBlank(suc) => end_sep(suc, line_len),
            KeyState::Built(suc) => suc,
        };
        Ok(result.map_value(|blank_end| ParseLineSuccess { blank_end, ..suc }))
    }

    fn parse_continuous_line(
        line_in: &str,
    ) -> Result<ParseLineResult<BlankValuePair<'_>>, ParseLineErr> {
        let res = parse_line(line_in, |_, _, _| ())?;
        Ok(match res {
            ParseLineResult::ContinueLine(blank_end) => {
                ParseLineResult::ContinueLine(BlankValuePair {
                    blank: &line_in[..blank_end],
                    value: &line_in[blank_end..line_in.len() - 1],
                })
            }
            ParseLineResult::EndValueLine(blank_end) => {
                ParseLineResult::EndValueLine(BlankValuePair {
                    blank: &line_in[..blank_end],
                    value: &line_in[blank_end..],
                })
            }
            ParseLineResult::CommentOrBlankLine => ParseLineResult::CommentOrBlankLine,
        })
    }

    // returns: blank_end
    fn parse_line<F: FnMut(bool, char, usize)>(
        line_in: &str,
        f: F,
    ) -> Result<ParseLineResult<usize>, ParseLineErr> {
        let line = line_in.as_bytes();
        let blank_end = match line.iter().position(|c| !b" \t\x0c\r\n".contains(c)) {
            None => return Ok(ParseLineResult::CommentOrBlankLine),
            Some(x) => x,
        };
        if b"#!".contains(&line[blank_end]) {
            return Ok(ParseLineResult::CommentOrBlankLine);
        };

        parse_escape_sequence(&line_in[blank_end..], f).map(|x| x.map_value(|_| blank_end))
    }

    fn parse_escape_sequence<F: FnMut(bool, char, usize)>(
        line_in: &str,
        mut f: F,
    ) -> Result<ParseLineResult<()>, ParseLineErr> {
        use ParseLineErr::*;

        #[derive(Eq, PartialEq)]
        enum ParsingState {
            Start,
            AfterBackSlash,
            AfterBU0,
            AfterBU1(u16),
            AfterBU2(u16),
            AfterBU3(u16),
        }
        use ParsingState::*;
        let mut stat = Start;
        let mut upper_surrogate: Option<u16> = None;

        for (i, ch) in line_in.char_indices() {
            let is_escape = stat != Start;
            let c = match stat {
                Start => {
                    if ch == '\\' {
                        stat = AfterBackSlash;
                        None
                    } else {
                        Some(ch)
                    }
                }
                AfterBackSlash => match ch {
                    'u' => {
                        stat = AfterBU0;
                        None
                    }
                    't' => {
                        stat = Start;
                        Some('\t')
                    }
                    'r' => {
                        stat = Start;
                        Some('\r')
                    }
                    'n' => {
                        stat = Start;
                        Some('\n')
                    }
                    'f' => {
                        stat = Start;
                        Some('\x0c')
                    }
                    c => {
                        stat = Start;
                        Some(c)
                    }
                },
                AfterBU0 => {
                    stat = AfterBU1(parse_hex(ch).ok_or_else(|| InvalidCharAt(i))?);
                    None
                }
                AfterBU1(cur) => {
                    stat = AfterBU2(cur << 4 + parse_hex(ch).ok_or_else(|| InvalidCharAt(i))?);
                    None
                }
                AfterBU2(cur) => {
                    stat = AfterBU3(cur << 4 + parse_hex(ch).ok_or_else(|| InvalidCharAt(i))?);
                    None
                }
                AfterBU3(cur) => {
                    let c = cur << 4 + parse_hex(ch).ok_or_else(|| InvalidCharAt(i))?;
                    if (0xD800..0xDC00).contains(&c) {
                        if upper_surrogate.is_some() {
                            return Err(InvalidCharAt(i));
                        }
                        upper_surrogate = Some(c);
                        None
                    } else if (0xDC00..0xE000).contains(&c) {
                        if let Some(upper) = upper_surrogate {
                            let c = 0x10000 + (upper - 0xD800) as u32 * 0x400 + (c - 0xDC00) as u32;
                            Some(unsafe { std::char::from_u32_unchecked(c) })
                        } else {
                            return Err(InvalidCharAt(i));
                        }
                    } else {
                        // out of surrogate, u16 so safe to cast
                        Some(unsafe { std::char::from_u32_unchecked(c as u32) })
                    }
                }
            };
            if let Some(c) = c {
                if upper_surrogate.is_some() {
                    return Err(InvalidCharAt(i));
                }
                f(is_escape, c, i);
            }
        }

        match stat {
            Start => Ok(ParseLineResult::EndValueLine(())),
            AfterBackSlash => Ok(ParseLineResult::ContinueLine(())),
            AfterBU0 => Err(UnexpectedEOL),
            AfterBU1(_) => Err(UnexpectedEOL),
            AfterBU2(_) => Err(UnexpectedEOL),
            AfterBU3(_) => Err(UnexpectedEOL),
        }
    }

    fn parse_hex(c: char) -> Option<u16> {
        match c {
            '0'..='9' => Some((c as u8 - b'0') as u16),
            'a'..='f' => Some((c as u8 - b'a' + 10) as u16),
            'A'..='F' => Some((c as u8 - b'A' + 10) as u16),
            _ => None,
        }
    }

    pub(super) fn parse_value(pair: &KeyValuePair<'_>) -> String {
        let trailing = match &pair.value {
            PropertyValue::Parsed(lines) => lines,
            PropertyValue::ActualValue(x) => return x.clone(),
        };

        let mut string_builder =
            String::with_capacity(trailing.iter().map(|x| x.value.len()).sum::<usize>());

        fn parse_escape_append(to: &mut String, value: &str) -> ParseLineResult<()> {
            parse_escape_sequence(&value, |_, x, _| *to += x.encode_utf8(&mut [0; 4]))
                .expect("parsed values")
        }

        let mut iter = trailing.iter();
        let mut cur = iter.next();
        while let Some(next) = iter.next() {
            parse_escape_append(&mut string_builder, &cur.unwrap().value)
                .expect_continue("parsed values");
            cur = Some(next);
        }

        if let Some(last) = cur {
            parse_escape_append(&mut string_builder, &last.value).expect_end_value("parsed values");
        }

        string_builder
    }
}

mod writing {
    use std::io::Result;
    use std::io::Write;

    use super::*;

    pub(super) fn write<W: Write>(file: &PropertiesFile<'_>, out: &mut W) -> Result<()> {
        for elem in &file.body {
            match elem {
                FileElement::KeyValuePair(pair) => write_key_value_pair(pair, out)?,
                FileElement::SkipLine(line) => writeln!(out, "{}", line)?,
            }
        }
        Ok(())
    }

    fn write_key_value_pair<W: Write>(pair: &KeyValuePair<'_>, out: &mut W) -> Result<()> {
        write!(out, "{}", pair.blank)?;
        if let Some(key) = pair.key {
            write!(out, "{}", key)?;
        } else if let Some(escaped) = escape(&pair.key_parsed) {
            write!(out, "{}", escaped)?;
        } else {
            write!(out, "{}", &pair.key_parsed)?;
        };
        write!(out, "{}", pair.separator)?;
        match &pair.value {
            PropertyValue::Parsed(lines) => {
                let mut iter = lines.iter();
                let mut cur = iter.next();
                while let Some(next) = iter.next() {
                    writeln!(out, "{}{}\\", cur.unwrap().blank, cur.unwrap().value)?;
                    cur = Some(next);
                }

                if let Some(last) = cur {
                    writeln!(out, "{}{}", last.blank, last.value)?;
                }
            }
            PropertyValue::ActualValue(v) => {
                writeln!(out, "{}", escape(v).as_ref().unwrap_or(v))?;
            }
        }
        Ok(())
    }

    fn escape(str: &str) -> Option<String> {
        if !str.contains(|x: char| " \t\n\r\x0c=:#!".contains(x) || x.is_control()) {
            return None;
        }
        let mut builder = String::with_capacity(str.len());
        for (i, c) in str.char_indices() {
            if c.is_control() {
                for x in c.encode_utf16(&mut [0; 2]) {
                    builder += &format!("\\u{:#04x}", *x);
                }
            } else {
                match c {
                    ' ' => {
                        if i == 0 {
                            builder += "\\ "
                        } else {
                            builder += " "
                        }
                    }
                    '=' => builder += "\\=",
                    ':' => builder += "\\:",
                    '#' => builder += "\\#",
                    '!' => builder += "\\!",
                    '\t' => builder += "\\t",
                    '\n' => builder += "\\n",
                    '\r' => builder += "\\r",
                    '\x0c' => builder += "\\f",
                    _ => builder += c.encode_utf8(&mut [0; 4]),
                }
            }
        }
        Some(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::FileElement::KeyValuePair as ElemKeyValuePair;
    use super::FileElement::*;
    use super::KeyValuePair;
    use super::PropertyValue::*;
    use super::*;

    include!("tests/properties.rs");
}
