//! This module contains the Json Parser with preserving original token representation

pub(crate) enum JsonError {
    UnexpectedEof,
    LeadingZero(usize),
    InvalidEscape(usize),
    InvalidChar(usize),
}

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
}

fn is_json_ws(c: char) -> bool {
    matches!(c, '\x20' | '\x0A' | '\x0D' | '\x09')
}

impl<'src> Tokenizer<'src> {
    fn new(src: &'src str) -> Self {
        Self { src, pos: 0 }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn next(&mut self) -> Result<Option<Token<'src>>, JsonError> {
        let start = self.pos;
        loop {
            return match self.peek_char() {
                Some(c) if is_json_ws(c) => {
                    // it's whitespace; skip it
                    self.pos += 1;
                    continue
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
                Some('t') => return Ok(self.parse_literal("true", Token::BooleanLiteral(true))?.into()),
                Some('f') => return Ok(self.parse_literal("false", Token::BooleanLiteral(false))?.into()),
                Some('n') => return Ok(self.parse_literal("null", Token::NullLiteral)?.into()),

                // others are error
                Some(_) => return Err(JsonError::InvalidChar(self.pos)),
            }
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
                            if !matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F') {
                                return Err(JsonError::InvalidEscape(this.pos - c.len_utf8()));
                            }
                            return Ok(c);
                        }
                    }
                    Some(c) => return Err(JsonError::InvalidEscape(self.pos - c.len_utf8())),
                    None => return Err(JsonError::UnexpectedEof),
                },
                Some(c) => self.pos += c.len_utf8(),
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
                    if !matches!(c, '0'..='9') {
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
                if !matches!(c, '0'..='9') {
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
                if !matches!(c, '0'..='9') {
                    break;
                }
                self.pos += 1;
            }
        }

        Ok(Token::NumberLiteral(&self.src[start..self.pos])
    }

    fn parse_literal(&mut self, str: &str, token: Token<'static>) -> Result<Token<'static>, JsonError> {
        if self.src[self.pos..].starts_with(str) {
            self.pos += str.len();
            Ok(token)
        } else {
            Err(JsonError::InvalidChar(self.pos))
        }
    }
}
