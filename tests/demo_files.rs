//! The reader and writer against the shipped MagicaCSG demo files.
//!
//! These are the acceptance fixtures for the format. Each one must parse, model
//! every section, write, and re-parse to an equal document.

use std::path::{Path, PathBuf};

const DEMOS: &[&str] = &["car", "primitive-type", "robot", "tachikoma", "tire"];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("{name}.mcsg"))
}

#[test]
fn demo_files_parse() {
    for name in DEMOS {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let document =
            dotmcsg::parse(&source).unwrap_or_else(|error| panic!("parse {name}: {error}"));

        let meta = document.meta().unwrap_or_else(|| panic!("{name}: meta"));
        assert!(
            meta.version().expect("a version").is_some_and(|v| v > 0),
            "{name}: version",
        );
        assert!(
            !document.objects().expect("objects").is_empty(),
            "{name}: objects"
        );
        assert!(
            !document.groups().expect("groups").is_empty(),
            "{name}: groups"
        );
    }
}

#[test]
fn demo_files_round_trip() {
    for name in DEMOS {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).expect("read");
        let document = dotmcsg::parse(&source).expect("parse");
        let written = document.to_mcsg_string();
        let reparsed =
            dotmcsg::parse(&written).unwrap_or_else(|error| panic!("re-parse {name}: {error}"));
        assert_eq!(
            reparsed, document,
            "{name}: round trip changed the document"
        );
    }
}

#[test]
fn demo_strokes_have_shapes() {
    for name in DEMOS {
        let path = fixture(name);
        let source = std::fs::read_to_string(&path).expect("read");
        let document = dotmcsg::parse(&source).expect("parse");
        for group in document.groups().expect("groups") {
            for stroke in group {
                assert!(stroke.shape().is_some(), "{name}: a stroke has no type");
            }
        }
    }
}
