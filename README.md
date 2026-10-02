# dotmcsg

A Rust library for loading and writing MagicaCSG `.mcsg` files.

`.mcsg` is the text format MagicaCSG, a non-destructive signed-distance-field
editor, saves its scenes in. `dotmcsg` parses it into a tree and writes it back,
preserving attributes it does not know.

The crate has no dependencies and does no file I/O: it parses from and writes to
a `&str`.

## Usage

```rust
let source = std::fs::read_to_string("scene.mcsg")?;
let document = dotmcsg::parse(&source)?;

for object in document.objects()? {
    println!("object {:?} uses csg group {:?}", object.name(), object.group()?);
}

let written = document.to_mcsg_string();
std::fs::write("copy.mcsg", written)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The typed accessors (`Document::objects`, `Document::groups`, `Stroke::shape`,
and so on) parse on demand and return `Result` where a value can be malformed.
Unknown keys stay in the document and are written back, so a file from a newer
MagicaCSG survives a round trip.

## The format

[`FORMAT.md`](FORMAT.md) documents the grammar, the sections and the attributes.

## License

Licensed under any of `Apache-2.0`, `BSD-3-Clause`, `MIT` or `Zlib`, at your
option.
