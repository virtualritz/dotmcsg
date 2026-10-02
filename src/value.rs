//! The generic `.mcsg` value tree and the attribute map built on it.
//!
//! The format is a small, schemaless tree: a document is an ordered list of
//! `"key" : value` sections, and every section is a value. Modelling it as a
//! tree rather than as structs with a fixed field list is what lets an unknown
//! key from a newer MagicaCSG survive a round trip unchanged.

use crate::error::Error;

/// One `.mcsg` value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// A quoted string, without escapes.
    String(String),
    /// `[ value ... ]`.
    Array(Vec<Value>),
    /// `{ "key" : value ... }`, order-preserving.
    Object(Vec<(String, Value)>),
    /// `[--[ raw text ]--]`.
    Literal(String),
}

impl Value {
    /// The string inside a [`Value::String`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(text.as_str()),
            _ => None,
        }
    }

    /// The elements inside a [`Value::Array`].
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(values) => Some(values.as_slice()),
            _ => None,
        }
    }

    /// The members inside a [`Value::Object`].
    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(members) => Some(members.as_slice()),
            _ => None,
        }
    }

    /// The member named `key`, when this is an object.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// The string member named `key`.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    /// Whether this is a [`Value::Literal`].
    pub fn is_literal(&self) -> bool {
        matches!(self, Value::Literal(_))
    }
}

/// An ordered, duplicate-preserving attribute list.
///
/// Every `.mcsg` object is an [`Attrs`]. Typed accessors parse on demand, so a
/// key from a newer build is carried through untouched and a malformed value is
/// an error rather than a silent default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Attrs(Vec<(String, Value)>);

impl Attrs {
    /// An empty attribute list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Build from an object's members.
    pub fn from_members(members: Vec<(String, Value)>) -> Self {
        Self(members)
    }

    /// The raw members, in order.
    pub fn members(&self) -> &[(String, Value)] {
        self.0.as_slice()
    }

    /// Consume the list, returning the raw members.
    pub fn into_members(self) -> Vec<(String, Value)> {
        self.0
    }

    /// The number of attributes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no attributes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every attribute key, in order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(key, _)| key.as_str())
    }

    /// The value of `key`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// The string value of `key`.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    /// Set `key`, replacing the first occurrence or appending.
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        let key = key.into();
        match self.0.iter_mut().find(|(name, _)| *name == key) {
            Some(slot) => slot.1 = value,
            None => self.0.push((key, value)),
        }
    }

    /// Remove `key`, returning its value.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        let index = self.0.iter().position(|(name, _)| name == key)?;
        Some(self.0.remove(index).1)
    }

    /// A flag attribute: absent is `false`, `"1"` is `true`.
    pub fn get_bool(&self, key: &str) -> Result<bool, Error> {
        match self.get_str(key) {
            None => Ok(false),
            Some("1") | Some("true") => Ok(true),
            Some("0") | Some("false") => Ok(false),
            Some(other) => Err(self.value_error(key, other, "expected a flag")),
        }
    }

    /// Parse a `%g`/`%d` element of a whitespace-separated number list.
    fn numbers<'a>(&'a self, key: &str) -> Option<std::str::SplitWhitespace<'a>> {
        self.get_str(key).map(str::split_whitespace)
    }

    /// A single float attribute.
    pub fn get_f32(&self, key: &str) -> Result<Option<f32>, Error> {
        match self.get_str(key) {
            None => Ok(None),
            Some(text) => match text.parse::<f32>() {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(self.value_error(key, text, "expected a number")),
            },
        }
    }

    /// A single float attribute, defaulted when absent.
    pub fn get_f32_or(&self, key: &str, default: f32) -> Result<f32, Error> {
        Ok(self.get_f32(key)?.unwrap_or(default))
    }

    /// A single integer attribute.
    pub fn get_i64(&self, key: &str) -> Result<Option<i64>, Error> {
        match self.get_str(key) {
            None => Ok(None),
            Some(text) => match text.parse::<i64>() {
                Ok(value) => Ok(Some(value)),
                Err(_) => Err(self.value_error(key, text, "expected an integer")),
            },
        }
    }

    /// A single integer attribute, defaulted when absent.
    pub fn get_i64_or(&self, key: &str, default: i64) -> Result<i64, Error> {
        Ok(self.get_i64(key)?.unwrap_or(default))
    }

    /// A single unsigned integer attribute.
    pub fn get_u32(&self, key: &str) -> Result<Option<u32>, Error> {
        match self.get_i64(key)? {
            None => Ok(None),
            Some(value) => u32::try_from(value).map(Some).map_err(|_| {
                self.value_error(
                    key,
                    self.get_str(key).unwrap_or(""),
                    "expected a non-negative integer",
                )
            }),
        }
    }

    /// A whitespace-separated list of floats.
    pub fn get_floats(&self, key: &str) -> Result<Vec<f32>, Error> {
        self.numbers(key)
            .map(|texts| {
                texts
                    .map(|text| {
                        text.parse::<f32>()
                            .map_err(|_| self.value_error(key, text, "expected a number"))
                    })
                    .collect()
            })
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// A whitespace-separated list of integers.
    pub fn get_ints(&self, key: &str) -> Result<Vec<i64>, Error> {
        self.numbers(key)
            .map(|texts| {
                texts
                    .map(|text| {
                        text.parse::<i64>()
                            .map_err(|_| self.value_error(key, text, "expected an integer"))
                    })
                    .collect()
            })
            .unwrap_or_else(|| Ok(Vec::new()))
    }

    /// A colour triple. Accepts a three- or four-channel value and takes the
    /// first three channels.
    pub fn get_rgb(&self, key: &str) -> Result<Option<[u8; 3]>, Error> {
        let channels = self.channels(key)?;
        if channels.is_empty() {
            return Ok(None);
        }
        match channels.len() {
            3 | 4 => Ok(Some([channels[0], channels[1], channels[2]])),
            count => Err(self.value_error(
                key,
                self.get_str(key).unwrap_or(""),
                &format!("expected three or four channels, found {count}"),
            )),
        }
    }

    /// A colour quadruple. Accepts a three- or four-channel value; a missing
    /// alpha reads as fully opaque.
    pub fn get_rgba(&self, key: &str) -> Result<Option<[u8; 4]>, Error> {
        let channels = self.channels(key)?;
        if channels.is_empty() {
            return Ok(None);
        }
        match channels.as_slice() {
            [r, g, b] => Ok(Some([*r, *g, *b, 255])),
            [r, g, b, a] => Ok(Some([*r, *g, *b, *a])),
            other => Err(self.value_error(
                key,
                self.get_str(key).unwrap_or(""),
                &format!("expected three or four channels, found {}", other.len()),
            )),
        }
    }

    /// Byte channels from a whitespace-separated attribute.
    fn channels(&self, key: &str) -> Result<Vec<u8>, Error> {
        self.get_ints(key)?
            .into_iter()
            .map(|value| {
                u8::try_from(value).map_err(|_| {
                    self.value_error(key, self.get_str(key).unwrap_or(""), "expected bytes")
                })
            })
            .collect()
    }

    /// Build a value error for `key`.
    pub(crate) fn value_error(&self, key: &str, value: &str, message: &str) -> Error {
        Error::Value {
            key: key.to_string(),
            value: value.to_string(),
            message: message.to_string(),
        }
    }
}

/// Parse a bare `"R G B"` string, as a palette entry.
pub(crate) fn parse_rgb(text: &str) -> Result<[u8; 3], Error> {
    let error = || Error::Value {
        key: "palette".to_string(),
        value: text.to_string(),
        message: "expected three bytes".to_string(),
    };
    let channels = text
        .split_whitespace()
        .map(|token| {
            let value: i64 = token.parse().map_err(|_| error())?;
            u8::try_from(value).map_err(|_| error())
        })
        .collect::<Result<Vec<u8>, Error>>()?;
    channels.try_into().map_err(|_| error())
}
