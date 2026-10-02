//! Unit tests for parsing, modelling and writing.

use super::*;

const SAMPLE: &str = r#""meta" :
{
    "version": "730"
    "hq": "1"
}
"render" :
{
    "media_g": "0.5"
}
"mtl" :
[
  //0
  {
    "ior": "1.44"
    "color": "10 20 30 255"
  }
  {
  }
]
"asset" :
[
  {"type": "ibl" "path": "bright.hdr"}
]
"csg" :
[
  [
  {
    "type": "cube"
    "mode": "sub"
    "blend": "0.36"
    "rgb": "247 126 92"
    "r": "1 0 0 0 1 0 0 0 1"
    "t": "0 0 0"
    "s": "1 1 1"
    "m": "1"
    "lm": "6"
    "unknown_key": "keep me"
  }
  {
    "type": "polygon"
    "subtype": "tri_iso"
    "op": "revolve"
    "v_n": "3"
    "v": "0 0 0 1 2 3 4 5 6"
    "v_s": "1 1 1 1 1 1 1 1 1"
  }
  ]
]
"object" :
[
  {
    "type": "csg"
    "res": "64"
    "cid": "0"
    "mid": "0"
    "mid2": "1"
    "name": "thing"
    "pivot_r": "0 1 0 0 0 1 1 0 0"
    "pivot_t": "1 2 3"
  }
]
"palette" :
[
  "0 0 0"
  "255 255 255"
]
"#;

fn sample() -> Document {
    Document::parse(SAMPLE).expect("the sample is valid")
}

#[test]
fn parses_all_sections() {
    let document = sample();
    assert_eq!(
        document.meta().expect("meta").version().expect("a number"),
        Some(730),
    );
    assert!(document.meta().expect("meta").hq().expect("a flag"));
    assert_eq!(document.materials().expect("mtl").len(), 2);
    assert_eq!(document.assets().expect("asset").len(), 1);
    assert_eq!(document.groups().expect("csg").len(), 1);
    assert_eq!(document.groups().expect("csg")[0].len(), 2);
    assert_eq!(document.objects().expect("object").len(), 1);
    assert_eq!(document.palette().expect("palette").len(), 2);

    let color = document.materials().expect("mtl")[0]
        .color()
        .expect("rgba")
        .expect("present");
    assert_eq!(color, [10, 20, 30, 255]);

    let asset = &document.assets().expect("asset")[0];
    assert_eq!(asset.kind(), Some("ibl"));
    assert_eq!(asset.path(), Some("bright.hdr"));
}

#[test]
fn stroke_accessors_read_typed_values() {
    let document = sample();
    let groups = document.groups().expect("csg");
    let cube = &groups[0][0];
    assert_eq!(cube.shape(), Some(Shape::Cube));
    assert_eq!(cube.boolean().expect("mode"), Boolean::Subtract);
    assert_eq!(cube.blend().expect("blend"), Some(0.36));
    assert_eq!(cube.rgb().expect("rgb"), Some([247, 126, 92]));
    assert_eq!(cube.mirror().expect("mirror"), 1);
    assert_eq!(cube.symmetry().expect("symmetry"), 6);
    let transform = cube.transform().expect("transform");
    assert_eq!(
        transform.rotation,
        Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    );

    let polygon = &groups[0][1];
    assert_eq!(polygon.shape(), Some(Shape::Polygon));
    assert_eq!(polygon.subtype(), Some("tri_iso"));
    assert_eq!(polygon.profile().expect("op"), Profile::Revolve);
}

#[test]
fn instance_accessors_read_typed_values() {
    let document = sample();
    let objects = document.objects().expect("object");
    let object = &objects[0];
    assert_eq!(object.name(), Some("thing"));
    assert_eq!(object.kind(), Some(ObjectKind::Csg));
    assert_eq!(object.group().expect("cid"), Some(0));
    assert_eq!(object.resolution().expect("res"), Some(64));
    assert_eq!(object.material().expect("mid"), Some(0));
    assert_eq!(object.material_secondary().expect("mid2"), Some(1));
    assert_eq!(
        object.pivot_translation().expect("pivot_t"),
        Some([1.0, 2.0, 3.0]),
    );
}

#[test]
fn round_trips() {
    let document = sample();
    let written = document.to_mcsg_string();
    let reparsed = Document::parse(&written).expect("the writer's output parses");
    assert_eq!(reparsed, document);
}

#[test]
fn unknown_keys_survive() {
    let document = sample();
    let cube = &document.groups().expect("csg")[0][0];
    assert_eq!(cube.attrs().get_str("unknown_key"), Some("keep me"));
    let written = document.to_mcsg_string();
    assert!(written.contains("unknown_key"));
    assert_eq!(
        Document::parse(&written)
            .expect("round trip")
            .groups()
            .expect("csg")[0][0]
            .attrs()
            .get_str("unknown_key"),
        Some("keep me"),
    );
}

#[test]
fn sections_in_any_order() {
    let reordered = r#""object" : [ { "type": "csg" "cid": "0" } ]
"mtl" : [ { "ior": "1.44" } ]
"meta" : { "version": "730" }
"#;
    let document = Document::parse(reordered).expect("any order parses");
    assert_eq!(document.objects().expect("object").len(), 1);
    assert_eq!(document.materials().expect("mtl").len(), 1);
    assert_eq!(
        document.meta().expect("meta").version().expect("a number"),
        Some(730),
    );
}

#[test]
fn blank_lines_and_comments_are_ignored() {
    let source = "// leading comment\n\n\"meta\" :\n{\n  // an inner comment\n  \"version\": \"1\" // trailing\n}\n";
    let document = Document::parse(source).expect("comments parse");
    assert_eq!(
        document.meta().expect("meta").version().expect("a number"),
        Some(1),
    );
}

#[test]
fn malformed_reports_line_and_column() {
    let error = Document::parse("\"meta\" :\n{\n  @\n}").expect_err("bad syntax");
    match error {
        Error::Parse { line, column, .. } => {
            assert_eq!(line, 3);
            assert_eq!(column, 3);
        }
        other => panic!("expected a parse error, got {other:?}"),
    }
}

#[test]
fn unterminated_string_is_an_error() {
    let error = Document::parse("\"meta\" :\n{ \"version\" : \"730 ").expect_err("unterminated");
    assert!(matches!(error, Error::Parse { .. }));
}

#[test]
fn bad_number_is_an_error() {
    let document = Document::parse("\"csg\" : [ [ { \"type\": \"cube\" \"t\": \"x y z\" } ] ]")
        .expect("text parses");
    let error = document.groups().expect("csg")[0][0]
        .transform()
        .expect_err("bad number");
    match error {
        Error::Value { key, .. } => assert_eq!(key, "t"),
        other => panic!("expected a value error, got {other:?}"),
    }
}

#[test]
fn wrong_number_count_is_an_error() {
    let document = Document::parse("\"csg\" : [ [ { \"type\": \"cube\" \"r\": \"1 2 3\" } ] ]")
        .expect("text parses");
    let error = document.groups().expect("csg")[0][0]
        .transform()
        .expect_err("wrong count");
    match error {
        Error::Value { key, message, .. } => {
            assert_eq!(key, "r");
            assert!(message.contains("expected 9"));
        }
        other => panic!("expected a value error, got {other:?}"),
    }
}

#[test]
fn unknown_boolean_is_an_error() {
    let document = Document::parse("\"csg\" : [ [ { \"type\": \"cube\" \"mode\": \"xor\" } ] ]")
        .expect("text parses");
    let error = document.groups().expect("csg")[0][0]
        .boolean()
        .expect_err("unknown mode");
    assert!(matches!(error, Error::Value { .. }));
}

#[test]
fn literal_values_are_kept() {
    let document =
        Document::parse("\"csg\" : [ [ { \"type\": \"chr\" \"text\": [--[hi there]--] } ] ]")
            .expect("literal parses");
    let stroke = &document.groups().expect("csg")[0][0];
    assert!(stroke.attrs().get("text").expect("text").is_literal());
    assert!(document.to_mcsg_string().contains("[--[hi there]--]"),);
}

/// The reader must never panic. This walks a deterministic set of malformed
/// strings, including truncations of a valid document and strings drawn from a
/// hostile alphabet.
#[test]
fn never_panics_on_malformed_input() {
    let mut inputs: Vec<String> = Vec::new();
    let mut alphabet = Vec::new();
    for byte in SAMPLE.bytes().chain(b"{}[]\":/@-\\\n\t".iter().copied()) {
        alphabet.push(byte as char);
    }
    let mut state = 0x1234_5678_u32;
    (0..2_000).for_each(|_| {
        let mut text = String::new();
        let length = (state % 24) as usize;
        (0..length).for_each(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            text.push(alphabet[(state as usize) % alphabet.len()]);
        });
        inputs.push(text);
    });
    for cut in 0..SAMPLE.len() {
        inputs.push(SAMPLE[..cut].to_string());
    }
    inputs.into_iter().for_each(|text| {
        let _ = Document::parse(&text);
    });
}
