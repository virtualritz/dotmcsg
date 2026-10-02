//! Typed views over a parsed `.mcsg` document.
//!
//! A [`Document`] stores the raw sections, so a key this release does not name
//! survives a round trip. The typed views below parse on demand; they are views,
//! not storage.

use crate::error::Error;
use crate::value::{Attrs, Value};

/// A parsed `.mcsg` document: its ordered top-level sections.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Document {
    sections: Vec<(String, Value)>,
}

impl Document {
    /// Parse `.mcsg` source.
    pub fn parse(source: &str) -> Result<Self, Error> {
        Ok(Self {
            sections: crate::parse::parse(source)?,
        })
    }

    /// Build from already-parsed sections.
    pub fn from_sections(sections: Vec<(String, Value)>) -> Self {
        Self { sections }
    }

    /// The raw sections, in order.
    pub fn sections(&self) -> &[(String, Value)] {
        self.sections.as_slice()
    }

    /// The raw value of a section.
    pub fn section(&self, name: &str) -> Option<&Value> {
        self.sections
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    /// Set a section, replacing the first occurrence or appending.
    pub fn set_section(&mut self, name: impl Into<String>, value: Value) {
        let name = name.into();
        match self.sections.iter_mut().find(|(key, _)| *key == name) {
            Some(slot) => slot.1 = value,
            None => self.sections.push((name, value)),
        }
    }

    /// The `meta` section.
    pub fn meta(&self) -> Option<Meta> {
        self.section("meta")
            .and_then(Value::as_object)
            .map(|members| Meta {
                attrs: Attrs::from_members(members.to_vec()),
            })
    }

    /// The `render` section.
    pub fn render(&self) -> Option<Render> {
        self.section("render")
            .and_then(Value::as_object)
            .map(|members| Render {
                attrs: Attrs::from_members(members.to_vec()),
            })
    }

    /// The `mtl` array.
    pub fn materials(&self) -> Result<Vec<Material>, Error> {
        array_objects(self, "mtl").map(|members| members.into_iter().map(Material::new).collect())
    }

    /// The `asset` array.
    pub fn assets(&self) -> Result<Vec<Asset>, Error> {
        array_objects(self, "asset").map(|members| members.into_iter().map(Asset::new).collect())
    }

    /// The `csg` array of stroke groups.
    pub fn groups(&self) -> Result<Vec<StrokeGroup>, Error> {
        let Some(value) = self.section("csg") else {
            return Ok(Vec::new());
        };
        let groups = value
            .as_array()
            .ok_or_else(|| section_error("csg", "expected an array"))?;
        groups
            .iter()
            .map(|group| {
                let strokes = group
                    .as_array()
                    .ok_or_else(|| section_error("csg", "expected an array of arrays"))?;
                strokes
                    .iter()
                    .map(|stroke| {
                        stroke
                            .as_object()
                            .map(|members| Stroke::new(Attrs::from_members(members.to_vec())))
                            .ok_or_else(|| section_error("csg", "expected a stroke object"))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect()
    }

    /// The `object` array.
    pub fn objects(&self) -> Result<Vec<Instance>, Error> {
        array_objects(self, "object")
            .map(|members| members.into_iter().map(Instance::new).collect())
    }

    /// The `palette` array of `"R G B"` strings.
    pub fn palette(&self) -> Result<Vec<[u8; 3]>, Error> {
        let Some(value) = self.section("palette") else {
            return Ok(Vec::new());
        };
        let entries = value
            .as_array()
            .ok_or_else(|| section_error("palette", "expected an array"))?;
        entries
            .iter()
            .map(|entry| {
                let text = entry
                    .as_str()
                    .ok_or_else(|| section_error("palette", "expected a string"))?;
                crate::value::parse_rgb(text)
            })
            .collect()
    }
}

/// The `meta` section.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Meta {
    attrs: Attrs,
}

impl Meta {
    /// The file's format version.
    pub fn version(&self) -> Result<Option<i64>, Error> {
        self.attrs.get_i64("version")
    }

    /// The `hq` flag.
    pub fn hq(&self) -> Result<bool, Error> {
        self.attrs.get_bool("hq")
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }
}

/// The `render` section.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Render {
    attrs: Attrs,
}

impl Render {
    /// The `media_g` value.
    pub fn media_g(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("media_g")
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }
}

/// One `mtl` entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Material {
    attrs: Attrs,
}

impl Material {
    fn new(attrs: Attrs) -> Self {
        Self { attrs }
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }

    /// `trans`.
    pub fn transmission(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("trans")
    }

    /// `ior`.
    pub fn index_of_refraction(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("ior")
    }

    /// `metal`.
    pub fn metallic(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("metal")
    }

    /// `rough`.
    pub fn roughness(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("rough")
    }

    /// `emit`.
    pub fn emission(&self) -> Result<Option<f32>, Error> {
        self.attrs.get_f32("emit")
    }

    /// `color`, as RGBA.
    pub fn color(&self) -> Result<Option<[u8; 4]>, Error> {
        self.attrs.get_rgba("color")
    }

    /// `media_color`, as RGB.
    pub fn media_color(&self) -> Result<Option<[u8; 3]>, Error> {
        self.attrs.get_rgb("media_color")
    }
}

/// One `asset` entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Asset {
    attrs: Attrs,
}

impl Asset {
    fn new(attrs: Attrs) -> Self {
        Self { attrs }
    }

    /// The asset's kind.
    pub fn kind(&self) -> Option<&str> {
        self.attrs.get_str("type")
    }

    /// The asset's path.
    pub fn path(&self) -> Option<&str> {
        self.attrs.get_str("path")
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }
}

/// One `csg` group: an ordered list of strokes.
pub type StrokeGroup = Vec<Stroke>;

/// One stroke.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stroke {
    attrs: Attrs,
}

impl Stroke {
    fn new(attrs: Attrs) -> Self {
        Self { attrs }
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }

    /// The stroke's optional name.
    pub fn name(&self) -> Option<&str> {
        self.attrs.get_str("name")
    }

    /// The stroke's shape.
    pub fn shape(&self) -> Option<Shape> {
        self.attrs.get_str("type").map(Shape::from_token)
    }

    /// The stroke's subtype token, as written.
    pub fn subtype(&self) -> Option<&str> {
        self.attrs.get_str("subtype")
    }

    /// The stroke's boolean operation.
    pub fn boolean(&self) -> Result<Boolean, Error> {
        match self.attrs.get_str("mode") {
            None => Ok(Boolean::Add),
            Some(token) => Boolean::from_token(token).ok_or_else(|| {
                self.attrs
                    .value_error("mode", token, "unknown boolean operation")
            }),
        }
    }

    /// The stroke's profile operation.
    pub fn profile(&self) -> Result<Profile, Error> {
        match self.attrs.get_str("op") {
            None => Ok(Profile::Extrude),
            Some(token) => Profile::from_token(token).ok_or_else(|| {
                self.attrs
                    .value_error("op", token, "unknown profile operation")
            }),
        }
    }

    /// The soft-min blend radius, when set and non-zero.
    pub fn blend(&self) -> Result<Option<f32>, Error> {
        Ok(self.attrs.get_f32("blend")?.filter(|blend| *blend != 0.0))
    }

    /// The stroke's material slot override.
    pub fn material(&self) -> Result<Option<u32>, Error> {
        self.attrs.get_u32("mtl")
    }

    /// The stroke's colour.
    pub fn rgb(&self) -> Result<Option<[u8; 3]>, Error> {
        self.attrs.get_rgb("rgb")
    }

    /// The stroke's local `r`/`t`/`s`.
    pub fn transform(&self) -> Result<Transform, Error> {
        transform_of(&self.attrs)
    }

    /// The mirror mask `m`.
    pub fn mirror(&self) -> Result<u32, Error> {
        Ok(self.attrs.get_u32("m")?.unwrap_or(0))
    }

    /// The symmetry mask `lm`.
    pub fn symmetry(&self) -> Result<u32, Error> {
        Ok(self.attrs.get_u32("lm")?.unwrap_or(0))
    }
}

/// One `object` instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instance {
    attrs: Attrs,
}

impl Instance {
    fn new(attrs: Attrs) -> Self {
        Self { attrs }
    }

    /// The raw attributes.
    pub fn attrs(&self) -> &Attrs {
        &self.attrs
    }

    /// The instance's optional name.
    pub fn name(&self) -> Option<&str> {
        self.attrs.get_str("name")
    }

    /// What the instance refers to.
    pub fn kind(&self) -> Option<ObjectKind> {
        self.attrs.get_str("type").map(ObjectKind::from_token)
    }

    /// The referenced `csg` group index.
    pub fn group(&self) -> Result<Option<u32>, Error> {
        self.attrs.get_u32("cid")
    }

    /// The SDF resolution.
    pub fn resolution(&self) -> Result<Option<u32>, Error> {
        self.attrs.get_u32("res")
    }

    /// The primary material slot.
    pub fn material(&self) -> Result<Option<u32>, Error> {
        self.attrs.get_u32("mid")
    }

    /// The secondary material slot.
    pub fn material_secondary(&self) -> Result<Option<u32>, Error> {
        self.attrs.get_u32("mid2")
    }

    /// The world `r`/`t`/`s`.
    pub fn transform(&self) -> Result<Transform, Error> {
        transform_of(&self.attrs)
    }

    /// The pivot rotation.
    pub fn pivot_rotation(&self) -> Result<Option<[f32; 9]>, Error> {
        floats::<9>(&self.attrs, "pivot_r")
    }

    /// The pivot translation.
    pub fn pivot_translation(&self) -> Result<Option<[f32; 3]>, Error> {
        floats::<3>(&self.attrs, "pivot_t")
    }
}

/// A `r`/`t`/`s` placement, each part optional as the file leaves it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// A row-major 3x3 rotation.
    pub rotation: Option<[f32; 9]>,
    /// A translation.
    pub translation: Option<[f32; 3]>,
    /// A per-axis scale.
    pub scale: Option<[f32; 3]>,
}

fn transform_of(attrs: &Attrs) -> Result<Transform, Error> {
    Ok(Transform {
        rotation: floats::<9>(attrs, "r")?,
        translation: floats::<3>(attrs, "t")?,
        scale: floats::<3>(attrs, "s")?,
    })
}

/// Exactly `N` floats from a whitespace-separated attribute.
fn floats<const N: usize>(attrs: &Attrs, key: &str) -> Result<Option<[f32; N]>, Error> {
    if attrs.get(key).is_none() {
        return Ok(None);
    }
    let values = attrs.get_floats(key)?;
    values.try_into().map(Some).map_err(|values: Vec<f32>| {
        attrs.value_error(
            key,
            attrs.get_str(key).unwrap_or(""),
            &format!("expected {N} numbers, found {}", values.len()),
        )
    })
}

fn array_objects(document: &Document, name: &str) -> Result<Vec<Attrs>, Error> {
    let Some(value) = document.section(name) else {
        return Ok(Vec::new());
    };
    let entries = value
        .as_array()
        .ok_or_else(|| section_error(name, "expected an array"))?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_object()
                .map(|members| Attrs::from_members(members.to_vec()))
                .ok_or_else(|| section_error(name, "expected an object"))
        })
        .collect()
}

fn section_error(name: &str, message: &str) -> Error {
    Error::Section {
        name: name.to_string(),
        message: message.to_string(),
    }
}

/// A stroke's shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shape {
    /// An axis-aligned box.
    Cube,
    /// A sphere or superellipsoid.
    Sphere,
    /// A cylinder or prism.
    Cylinder,
    /// A triangle profile.
    Triangle,
    /// A circular profile.
    Circle,
    /// A regular polygon profile.
    Polygon,
    /// A polygon from explicit points.
    Bpoly,
    /// A bezier polyline profile.
    Bezier,
    /// A glyph outline.
    Chr,
    /// An SVG outline.
    Svg,
    /// A nested CSG reference.
    Csg,
    /// A grouped stroke.
    Group,
    /// A joint.
    Joint,
    /// A line.
    Line,
    /// A 3D line.
    Line3,
    /// A 3D bezier.
    Bezier3,
    /// A tetrahedron.
    Tetra,
    /// A rhombus.
    Rhombus,
    /// Any token this release does not name.
    Other(String),
}

impl Shape {
    /// Parse a `type` token.
    pub fn from_token(token: &str) -> Self {
        match token {
            "cube" => Shape::Cube,
            "sphere" => Shape::Sphere,
            "cylinder" => Shape::Cylinder,
            "triangle" => Shape::Triangle,
            "circle" => Shape::Circle,
            "polygon" => Shape::Polygon,
            "bpoly" => Shape::Bpoly,
            "bezier" => Shape::Bezier,
            "chr" => Shape::Chr,
            "svg" => Shape::Svg,
            "csg" => Shape::Csg,
            "group" => Shape::Group,
            "joint" | "jt" => Shape::Joint,
            "line" => Shape::Line,
            "line3" => Shape::Line3,
            "bezier3" => Shape::Bezier3,
            "tetra" => Shape::Tetra,
            "rhombus" => Shape::Rhombus,
            other => Shape::Other(other.to_string()),
        }
    }

    /// The token this shape is written as.
    pub fn as_token(&self) -> &str {
        match self {
            Shape::Cube => "cube",
            Shape::Sphere => "sphere",
            Shape::Cylinder => "cylinder",
            Shape::Triangle => "triangle",
            Shape::Circle => "circle",
            Shape::Polygon => "polygon",
            Shape::Bpoly => "bpoly",
            Shape::Bezier => "bezier",
            Shape::Chr => "chr",
            Shape::Svg => "svg",
            Shape::Csg => "csg",
            Shape::Group => "group",
            Shape::Joint => "jt",
            Shape::Line => "line",
            Shape::Line3 => "line3",
            Shape::Bezier3 => "bezier3",
            Shape::Tetra => "tetra",
            Shape::Rhombus => "rhombus",
            Shape::Other(token) => token.as_str(),
        }
    }
}

/// A stroke's boolean operation against the accumulated field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boolean {
    /// `add`.
    Add,
    /// `sub`.
    Subtract,
    /// `inter`.
    Intersect,
    /// `rep`.
    Replace,
}

impl Boolean {
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "add" => Some(Boolean::Add),
            "sub" => Some(Boolean::Subtract),
            "inter" => Some(Boolean::Intersect),
            "rep" => Some(Boolean::Replace),
            _ => None,
        }
    }

    /// The token this operation is written as.
    pub fn as_token(self) -> &'static str {
        match self {
            Boolean::Add => "add",
            Boolean::Subtract => "sub",
            Boolean::Intersect => "inter",
            Boolean::Replace => "rep",
        }
    }
}

/// A stroke's profile operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    /// `extrude`, the default when `op` is absent.
    Extrude,
    /// `revolve`.
    Revolve,
    /// `torus`.
    Torus,
    /// `helix`.
    Helix,
    /// `sweep`.
    Sweep,
}

impl Profile {
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "extrude" => Some(Profile::Extrude),
            "revolve" => Some(Profile::Revolve),
            "torus" => Some(Profile::Torus),
            "helix" => Some(Profile::Helix),
            "sweep" => Some(Profile::Sweep),
            _ => None,
        }
    }

    /// The token this profile is written as.
    pub fn as_token(self) -> &'static str {
        match self {
            Profile::Extrude => "extrude",
            Profile::Revolve => "revolve",
            Profile::Torus => "torus",
            Profile::Helix => "helix",
            Profile::Sweep => "sweep",
        }
    }
}

/// What an `object` instance refers to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    /// A reference to a `csg` group.
    Csg,
    /// A background object.
    Background,
    /// Any token this release does not name.
    Other(String),
}

impl ObjectKind {
    /// Parse an `object.type` token.
    pub fn from_token(token: &str) -> Self {
        match token {
            "csg" => ObjectKind::Csg,
            "bg" => ObjectKind::Background,
            other => ObjectKind::Other(other.to_string()),
        }
    }

    /// The token this kind is written as.
    pub fn as_token(&self) -> &str {
        match self {
            ObjectKind::Csg => "csg",
            ObjectKind::Background => "bg",
            ObjectKind::Other(token) => token.as_str(),
        }
    }
}
