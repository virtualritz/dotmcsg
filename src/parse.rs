//! The hand-rolled `.mcsg` reader.
//!
//! The grammar has six productions and no ambiguity, so this is a recursive
//! descent over a byte cursor rather than a parser combinator. The reader is
//! total: every input either parses or returns an [`Error::Parse`] with a
//! line and column.

use crate::error::Error;
use crate::value::Value;

/// Parse a `.mcsg` document into its ordered top-level sections.
pub fn parse(source: &str) -> Result<Vec<(String, Value)>, Error> {
    let mut cursor = Cursor::new(source);
    let mut sections = Vec::new();
    loop {
        cursor.skip_trivia();
        if cursor.at_end() {
            return Ok(sections);
        }
        let key = cursor.parse_string()?;
        cursor.skip_trivia();
        cursor.expect(b':', "expected ':' after a section key")?;
        let value = cursor.parse_value()?;
        sections.push((key, value));
    }
}

struct Cursor<'a> {
    source: &'a str,
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            position: 0,
        }
    }

    fn at_end(&self) -> bool {
        self.position >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    /// The one-based line and column of the current position.
    fn location(&self) -> (usize, usize) {
        let consumed = &self.source[..self.position.min(self.source.len())];
        let line = consumed.bytes().filter(|&b| b == b'\n').count() + 1;
        let column = consumed
            .rsplit_once('\n')
            .map_or(consumed.chars().count(), |(_, tail)| tail.chars().count())
            + 1;
        (line, column)
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, Error> {
        let (line, column) = self.location();
        Err(Error::Parse {
            line,
            column,
            message: message.into(),
        })
    }

    /// Skip whitespace and `//` line comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => self.position += 1,
                Some(b'/') if self.bytes.get(self.position + 1) == Some(&b'/') => {
                    match self.bytes[self.position..].iter().position(|&b| b == b'\n') {
                        Some(offset) => self.position += offset + 1,
                        None => self.position = self.bytes.len(),
                    }
                }
                _ => return,
            }
        }
    }

    fn expect(&mut self, byte: u8, message: &str) -> Result<(), Error> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            self.error(message)
        }
    }

    fn expect_exact(&mut self, text: &str, message: &str) -> Result<(), Error> {
        if self.source[self.position..].starts_with(text) {
            self.position += text.len();
            Ok(())
        } else {
            self.error(message)
        }
    }

    /// A quoted string. There are no escape sequences in the format, so a `"`
    /// always ends the string.
    fn parse_string(&mut self) -> Result<String, Error> {
        self.expect(b'"', "expected '\"'")?;
        let start = self.position;
        match self.bytes[start..].iter().position(|&b| b == b'"') {
            Some(offset) => {
                self.position = start + offset + 1;
                Ok(self.source[start..start + offset].to_string())
            }
            None => self.error("unterminated string"),
        }
    }

    fn parse_literal(&mut self) -> Result<String, Error> {
        self.expect_exact("[--[", "expected '[--['")?;
        let start = self.position;
        match self.source[start..].find("]--]") {
            Some(offset) => {
                self.position = start + offset + 4;
                Ok(self.source[start..start + offset].to_string())
            }
            None => self.error("unterminated literal"),
        }
    }

    fn parse_value(&mut self) -> Result<Value, Error> {
        self.skip_trivia();
        match self.peek() {
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'{') => self.parse_object(),
            Some(b'[') if self.source[self.position..].starts_with("[--[") => {
                self.parse_literal().map(Value::Literal)
            }
            Some(b'[') => self.parse_array(),
            Some(other) => self.error(format!("expected a value, found {:?}", other as char)),
            None => self.error("expected a value, found end of input"),
        }
    }

    fn parse_object(&mut self) -> Result<Value, Error> {
        self.expect(b'{', "expected '{'")?;
        let mut members = Vec::new();
        loop {
            self.skip_trivia();
            match self.peek() {
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Value::Object(members));
                }
                Some(b'"') => {
                    let key = self.parse_string()?;
                    self.skip_trivia();
                    self.expect(b':', "expected ':' after an object key")?;
                    let value = self.parse_value()?;
                    members.push((key, value));
                }
                Some(other) => {
                    return self
                        .error(format!("expected a key or '}}', found {:?}", other as char));
                }
                None => return self.error("unterminated object"),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, Error> {
        self.expect(b'[', "expected '['")?;
        let mut values = Vec::new();
        loop {
            self.skip_trivia();
            match self.peek() {
                Some(b']') => {
                    self.position += 1;
                    return Ok(Value::Array(values));
                }
                Some(_) => values.push(self.parse_value()?),
                None => return self.error("unterminated array"),
            }
        }
    }
}
