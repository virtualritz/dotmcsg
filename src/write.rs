//! The `.mcsg` writer.
//!
//! The writer emits the parsed tree in a canonical, indented layout. It does
//! not reproduce the demo files byte for byte -- the format has no required
//! layout -- but the output is valid `.mcsg` that MagicaCSG and this crate both
//! read back to an equal [`Document`].

use crate::model::Document;
use crate::value::Value;

/// Indentation of one nesting level.
const INDENT: usize = 2;

impl Document {
    /// Write the document as `.mcsg` text.
    ///
    /// The format has no escape sequences, so a string containing a `"` cannot
    /// be represented. Such a value is written as-is and the result is not
    /// valid `.mcsg`; no such value occurs in practice.
    pub fn to_mcsg_string(&self) -> String {
        let mut out = String::new();
        for (key, value) in self.sections() {
            out.push('"');
            out.push_str(key);
            out.push_str("\" :\n");
            write_value(value, 0, &mut out);
            out.push('\n');
        }
        out
    }
}

fn write_value(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::String(text) => {
            out.push('"');
            out.push_str(text);
            out.push('"');
        }
        Value::Literal(text) => {
            out.push_str("[--[");
            out.push_str(text);
            out.push_str("]--]");
        }
        Value::Object(members) => {
            out.push_str("{\n");
            for (key, member) in members {
                push_indent(indent + INDENT, out);
                out.push('"');
                out.push_str(key);
                out.push_str("\": ");
                write_value(member, indent + INDENT, out);
                out.push('\n');
            }
            push_indent(indent, out);
            out.push('}');
        }
        Value::Array(values) => {
            out.push_str("[\n");
            for element in values {
                push_indent(indent + INDENT, out);
                write_value(element, indent + INDENT, out);
                out.push('\n');
            }
            push_indent(indent, out);
            out.push(']');
        }
    }
}

fn push_indent(count: usize, out: &mut String) {
    (0..count).for_each(|_| out.push(' '));
}
