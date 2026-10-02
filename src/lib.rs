//! A Rust library for loading and writing MagicaCSG `.mcsg` files.
//!
//! `.mcsg` is a text format: quoted keys, `{ }` objects, `[ ]` arrays, no
//! commas between members, `//` comments, and `[--[ ... ]--]` for free text.
//! `dotmcsg` parses it into a [`Document`] and writes it back.
//!
//! # Example
//!
//! ```
//! let source = r#"
//! "meta" :
//! {
//!     "version": "730"
//! }
//! "#;
//! let document = dotmcsg::parse(source).expect("valid mcsg");
//! assert_eq!(document.meta().expect("meta").version().expect("integer"), Some(730));
//! let written = document.to_mcsg_string();
//! assert_eq!(dotmcsg::parse(&written).expect("round trip"), document);
//! ```
//!
//! The tree is schemaless: an attribute this release does not name is retained
//! and written back, so a file from a newer MagicaCSG survives a round trip.

#![forbid(unsafe_code)]

mod error;
mod model;
mod parse;
mod value;
mod write;

pub use error::Error;
pub use model::{
    Asset, Boolean, Document, Instance, Material, Meta, ObjectKind, Profile, Render, Shape, Stroke,
    StrokeGroup, Transform,
};
pub use value::{Attrs, Value};

/// Parse `.mcsg` source into a [`Document`].
pub fn parse(source: &str) -> Result<Document, Error> {
    Document::parse(source)
}

#[cfg(test)]
mod tests;
