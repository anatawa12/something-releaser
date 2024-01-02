//! This module contains the Json Parser with preserving original token representation

use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum JsonError {
    UnexpectedEof,
    LeadingZero(usize),
    InvalidEscape(usize),
    InvalidChar(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Token<'src> {
    /// punctuation includes leading / trailing whitespace in their token
    Punctuation(Punctuation, &'src str),
    StringLiteral(&'src str),
    /// number literals may include negative sign
    NumberLiteral(&'src str),
    BooleanLiteral(bool),
    NullLiteral,

    /// If a json file starts with or ends with whitespace, it will be a floating whitespace
    FloatingWhitespace(&'src str),
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Punctuation(_, s) => write!(f, "{}", s),
            Token::StringLiteral(s) => write!(f, "{}", s),
            Token::NumberLiteral(s) => write!(f, "{}", s),
            Token::BooleanLiteral(b) => write!(f, "{}", b),
            Token::NullLiteral => write!(f, "null"),
            Token::FloatingWhitespace(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Punctuation {
    ObjectStart = '{' as isize,
    ObjectEnd = '}' as isize,
    ArrayStart = '[' as isize,
    ArrayEnd = ']' as isize,
    Comma = ',' as isize,
    Colon = ':' as isize,
}

impl Punctuation {
    fn as_char(&self) -> char {
        match self {
            Self::ObjectStart => '{',
            Self::ObjectEnd => '}',
            Self::ArrayStart => '[',
            Self::ArrayEnd => ']',
            Self::Comma => ',',
            Self::Colon => ':',
        }
    }
}

struct Tokenizer<'src> {
    src: &'src str,
    pos: usize,
    peeked: Option<Token<'src>>,
}

fn is_json_ws(c: char) -> bool {
    matches!(c, '\x20' | '\x0A' | '\x0D' | '\x09')
}

impl<'src> Tokenizer<'src> {
    fn new(src: &'src str) -> Self {
        Self {
            src,
            pos: 0,
            peeked: None,
        }
    }

    fn next(&mut self) -> Result<Option<Token<'src>>, JsonError> {
        if let Some(token) = self.peeked.take() {
            Ok(Some(token))
        } else {
            self.compute_next()
        }
    }

    fn peek(&mut self) -> Result<Option<&Token<'src>>, JsonError> {
        if self.peeked.is_none() {
            self.peeked = self.compute_next()?;
        }
        Ok(self.peeked.as_ref())
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn compute_next(&mut self) -> Result<Option<Token<'src>>, JsonError> {
        let start = self.pos;
        loop {
            return match self.peek_char() {
                Some(c) if is_json_ws(c) => {
                    // it's whitespace; skip it
                    self.pos += 1;
                    continue;
                }
                // punctuations
                Some('{') => Ok(self.punctuation(start, Punctuation::ObjectStart).into()),
                Some('}') => Ok(self.punctuation(start, Punctuation::ObjectEnd).into()),
                Some('[') => Ok(self.punctuation(start, Punctuation::ArrayStart).into()),
                Some(']') => Ok(self.punctuation(start, Punctuation::ArrayEnd).into()),
                Some(',') => Ok(self.punctuation(start, Punctuation::Comma).into()),
                Some(':') => Ok(self.punctuation(start, Punctuation::Colon).into()),

                // process floating whitespaces
                Some(_) | None if self.pos != start => {
                    // it's a floating whitespace
                    Ok(Token::FloatingWhitespace(&self.src[start..self.pos]).into())
                }
                None => Ok(None),

                // string literal
                Some('"') => Ok(self.parse_string()?.into()),

                // numeric literal
                Some('-' | '0'..='9') => Ok(self.parse_number()?.into()),

                // other literals
                Some('t') => {
                    return Ok(self
                        .parse_literal("true", Token::BooleanLiteral(true))?
                        .into())
                }
                Some('f') => {
                    return Ok(self
                        .parse_literal("false", Token::BooleanLiteral(false))?
                        .into())
                }
                Some('n') => return Ok(self.parse_literal("null", Token::NullLiteral)?.into()),

                // others are error
                Some(_) => return Err(JsonError::InvalidChar(self.pos)),
            };
        }
    }

    fn punctuation(&mut self, start: usize, p0: Punctuation) -> Token<'src> {
        debug_assert!(self.src[self.pos..].starts_with(p0.as_char()));
        self.pos += 1;

        // skip whitespaces
        while matches!(self.src[self.pos..].chars().next(), Some(c) if is_json_ws(c)) {
            self.pos += 1;
        }

        Token::Punctuation(p0, &self.src[start..self.pos])
    }

    fn parse_string(&mut self) -> Result<Token<'src>, JsonError> {
        let start = self.pos;
        self.pos += 1;
        loop {
            match self.next_char() {
                Some('"') => {
                    return Ok(Token::StringLiteral(&self.src[start..self.pos]));
                }
                // it's an escape sequence
                Some('\\') => match self.next_char() {
                    Some('"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => {}
                    Some('u') => {
                        // \uXXXX
                        let _ = read_hex(self)?;
                        let _ = read_hex(self)?;
                        let _ = read_hex(self)?;
                        let _ = read_hex(self)?;

                        fn read_hex(this: &mut Tokenizer) -> Result<char, JsonError> {
                            let c = this.next_char().ok_or(JsonError::UnexpectedEof)?;
                            if !c.is_ascii_hexdigit() {
                                return Err(JsonError::InvalidEscape(this.pos - c.len_utf8()));
                            }
                            Ok(c)
                        }
                    }
                    Some(c) => return Err(JsonError::InvalidEscape(self.pos - c.len_utf8())),
                    None => return Err(JsonError::UnexpectedEof),
                },
                Some(c) => {}
                None => return Err(JsonError::UnexpectedEof),
            }
        }
    }

    fn parse_number(&mut self) -> Result<Token<'src>, JsonError> {
        let start = self.pos;

        if self.peek_char() == Some('-') {
            self.pos += 1;
        }
        // integer part
        match self.peek_char() {
            Some('0') => {
                // leading zero is not allowed so if '0' is followed by a digit, it's an error
                self.pos += 1;
                if matches!(self.peek_char(), Some('0'..='9')) {
                    return Err(JsonError::LeadingZero(self.pos - 1));
                }
            }
            Some('1'..='9') => {
                self.pos += 1;
                while let Some(c) = self.peek_char() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    self.pos += 1;
                }
            }
            Some(_) => {
                return Err(JsonError::InvalidChar(self.pos));
            }
            None => {
                return Err(JsonError::UnexpectedEof);
            }
        }

        // fraction part
        if self.peek_char() == Some('.') {
            self.pos += 1;
            while let Some(c) = self.peek_char() {
                if !c.is_ascii_digit() {
                    break;
                }
                self.pos += 1;
            }
        }

        // exponent part
        if matches!(self.peek_char(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.pos += 1;
            }
            if matches!(self.peek_char(), Some('0'..='9')) {
                self.pos += 1;
            } else {
                return Err(JsonError::InvalidChar(self.pos));
            }
            while let Some(c) = self.peek_char() {
                if !c.is_ascii_digit() {
                    break;
                }
                self.pos += 1;
            }
        }

        Ok(Token::NumberLiteral(&self.src[start..self.pos]))
    }

    fn parse_literal(
        &mut self,
        str: &str,
        token: Token<'static>,
    ) -> Result<Token<'static>, JsonError> {
        if self.src[self.pos..].starts_with(str) {
            self.pos += str.len();
            Ok(token)
        } else {
            Err(JsonError::InvalidChar(self.pos))
        }
    }
}

pub(crate) struct JsonFile<'src> {
    pub heading_space: &'src str,
    pub value: JsonValue<'src>,
    pub trailing_space: &'src str,
}

impl Display for JsonFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.heading_space, self.value, self.trailing_space
        )
    }
}

pub(crate) enum JsonValue<'src> {
    Object(JsonObject<'src>),
    Array(JsonArray<'src>),
    Literal(Token<'src>),
}

impl Display for JsonValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Object(obj) => write!(f, "{}", obj),
            JsonValue::Array(arr) => write!(f, "{}", arr),
            JsonValue::Literal(token) => write!(f, "{}", token),
        }
    }
}

impl<'src> JsonValue<'src> {
    pub(crate) fn as_object_mut(&mut self) -> Option<&mut JsonObject<'src>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

pub(crate) struct JsonObject<'src> {
    pub open_brace: Token<'src>,
    pub members: Vec<JsonMember<'src>>,
    pub close_brace: Token<'src>,
}

impl Display for JsonObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.open_brace)?;
        for member in &self.members {
            if let Some(comma) = &member.comma {
                write!(f, "{}{}{}{}", member.key, member.colon, member.value, comma)?;
            } else {
                write!(f, "{}{}{}", member.key, member.colon, member.value)?;
            }
        }
        write!(f, "{}", self.close_brace)
    }
}

impl<'src> JsonObject<'src> {
    pub(crate) fn set(&mut self, key_quoted: &'src str, value: Token<'src>) {
        for member in &mut self.members {
            if member.key == Token::StringLiteral(key_quoted) {
                member.value = JsonValue::Literal(value);
                return;
            }
        }
        panic!("there are no key {} in json object", key_quoted);
    }
}

pub(crate) struct JsonMember<'src> {
    pub key: Token<'src>,
    pub colon: Token<'src>,
    pub value: JsonValue<'src>,
    pub comma: Option<Token<'src>>,
}

pub(crate) struct JsonArray<'src> {
    pub open_bracket: Token<'src>,
    pub elements: Vec<(JsonValue<'src>, Option<Token<'src>>)>,
    pub close_bracket: Token<'src>,
}

impl Display for JsonArray<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.open_bracket)?;
        for (element, comma) in &self.elements {
            if let Some(comma) = comma {
                write!(f, "{}{}", element, comma)?;
            } else {
                write!(f, "{}", element)?;
            }
        }
        write!(f, "{}", self.close_bracket)
    }
}

pub(crate) fn parse_json(src: &str) -> Result<JsonFile<'_>, JsonError> {
    let mut tokenizer = Tokenizer::new(src);
    let heading_space = match tokenizer.peek()? {
        Some(Token::FloatingWhitespace(ws)) => {
            let ws = *ws;
            tokenizer.next()?; // consume the token
            ws
        }
        _ => "", // no floating whitespace, might be no ws OR starts with '{'
    };

    let value = parse_value(&mut tokenizer)?;

    let trailing_space = match tokenizer.next()? {
        Some(Token::FloatingWhitespace(ws)) => {
            if (tokenizer.next()?).is_some() {
                return Err(JsonError::InvalidChar(tokenizer.pos));
            }
            ws
        }
        None => "",
        Some(_) => return Err(JsonError::InvalidChar(tokenizer.pos)),
    };

    Ok(JsonFile {
        heading_space,
        value,
        trailing_space,
    })
}

fn parse_value<'src>(tokenizer: &mut Tokenizer<'src>) -> Result<JsonValue<'src>, JsonError> {
    match tokenizer.next()? {
        None => Err(JsonError::UnexpectedEof),
        Some(Token::FloatingWhitespace(_)) => Err(JsonError::InvalidChar(tokenizer.pos)),
        Some(open_brace @ Token::Punctuation(Punctuation::ObjectStart, _)) => {
            if matches!(
                tokenizer.peek()?,
                Some(Token::Punctuation(Punctuation::ObjectEnd, _))
            ) {
                return Ok(JsonValue::Object(JsonObject {
                    open_brace,
                    members: Vec::new(),
                    close_brace: tokenizer.next().unwrap().unwrap(),
                }));
            }

            let mut members = Vec::new();
            loop {
                let key = match tokenizer.next()? {
                    Some(Token::StringLiteral(key)) => key,
                    Some(_) => return Err(JsonError::InvalidChar(tokenizer.pos)),
                    None => return Err(JsonError::UnexpectedEof),
                };

                let colon = match tokenizer.next()? {
                    Some(Token::Punctuation(Punctuation::Colon, colon)) => colon,
                    Some(_) => return Err(JsonError::InvalidChar(tokenizer.pos)),
                    None => return Err(JsonError::UnexpectedEof),
                };

                let value = parse_value(tokenizer)?;

                match tokenizer.next()? {
                    Some(comma @ Token::Punctuation(Punctuation::Comma, _)) => {
                        members.push(JsonMember {
                            key: Token::StringLiteral(key),
                            colon: Token::Punctuation(Punctuation::Colon, colon),
                            value,
                            comma: Some(comma),
                        });
                    }
                    Some(close_brace @ Token::Punctuation(Punctuation::ObjectEnd, _)) => {
                        members.push(JsonMember {
                            key: Token::StringLiteral(key),
                            colon: Token::Punctuation(Punctuation::Colon, colon),
                            value,
                            comma: None,
                        });
                        break Ok(JsonValue::Object(JsonObject {
                            open_brace,
                            members,
                            close_brace,
                        }));
                    }
                    Some(_) => return Err(JsonError::InvalidChar(tokenizer.pos)),
                    None => return Err(JsonError::UnexpectedEof),
                }
            }
        }
        Some(open_bracket @ Token::Punctuation(Punctuation::ArrayStart, _)) => {
            if matches!(
                tokenizer.peek()?,
                Some(Token::Punctuation(Punctuation::ArrayEnd, _))
            ) {
                return Ok(JsonValue::Array(JsonArray {
                    open_bracket,
                    elements: Vec::new(),
                    close_bracket: tokenizer.next().unwrap().unwrap(),
                }));
            }

            let mut elements = Vec::new();
            loop {
                let value = parse_value(tokenizer)?;

                match tokenizer.next()? {
                    Some(comma @ Token::Punctuation(Punctuation::Comma, _)) => {
                        elements.push((value, Some(comma)));
                    }
                    Some(close_bracket @ Token::Punctuation(Punctuation::ArrayEnd, _)) => {
                        elements.push((value, None));
                        break Ok(JsonValue::Array(JsonArray {
                            open_bracket,
                            elements,
                            close_bracket,
                        }));
                    }
                    Some(_) => return Err(JsonError::InvalidChar(tokenizer.pos)),
                    None => return Err(JsonError::UnexpectedEof),
                }
            }
        }
        Some(
            literal @ (Token::BooleanLiteral(_)
            | Token::NullLiteral
            | Token::NumberLiteral(_)
            | Token::StringLiteral(_)),
        ) => Ok(JsonValue::Literal(literal)),

        Some(Token::Punctuation(_, _)) => Err(JsonError::InvalidChar(tokenizer.pos)),
    }
}

pub(crate) fn quote_string(value: &str) -> String {
    let mut length = 0;
    for c in value.chars() {
        match c {
            '\x00'..='\x1F' => {
                length += 6; // '\uXXXX'
            }
            '"' | '\\' => length += 2, // '\"' | '\\'
            c => length += c.len_utf8(),
        }
    }

    if length == value.len() {
        return format!("\"{}\"", value);
    }

    let mut builder = String::with_capacity(length + 2);
    builder.push('"');

    for c in value.chars() {
        match c {
            '\x00'..='\x1F' => {
                builder.push('\\');
                builder.push('u');
                builder.push('0');
                builder.push('0');
                builder.push(b"0123456789abcdef"[(c as u32 >> 4) as usize] as char);
                builder.push(b"0123456789abcdef"[(c as u32 & 0xF) as usize] as char);
            }
            '"' | '\\' => {
                builder.push('\\');
                builder.push(c);
            }
            c @ '\x20'..='\u{10FFFF}' => builder.push(c),
        }
    }
    builder.push('"');
    builder
}
