//! Error type for `dotmcsg`.

use std::fmt;

/// Everything that can go wrong reading or writing a `.mcsg` document.
///
/// Syntax errors name the location so a reader can point at the file. Value
/// errors name the attribute key, because the value tree does not carry spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The text is not valid `.mcsg` syntax.
    Parse {
        /// One-based line of the offending byte.
        line: usize,
        /// One-based column of the offending byte.
        column: usize,
        /// What the reader expected and what it found.
        message: String,
    },
    /// An attribute's value does not parse as the type its reader wants.
    Value {
        /// The attribute key.
        key: String,
        /// The raw value as written.
        value: String,
        /// Why it does not parse.
        message: String,
    },
    /// A required section is absent or its top-level value is the wrong kind.
    Section {
        /// The section key.
        name: String,
        /// Why the section is unusable.
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse {
                line,
                column,
                message,
            } => write!(f, "line {line}, column {column}: {message}"),
            Error::Value {
                key,
                value,
                message,
            } => {
                write!(f, "attribute \"{key}\": {message} (value {value:?})")
            }
            Error::Section { name, message } => {
                write!(f, "section \"{name}\": {message}")
            }
        }
    }
}

impl std::error::Error for Error {}
