# The `.mcsg` format

This documents the MagicaCSG `.mcsg` text format as reverse engineered from
MagicaCSG Demo 0.7.3 (`MagicaCSG.exe`, strings dated 2026-04-26) and its five
shipped `csg/*.mcsg` files.

## Grammar

```text
document := (STRING ':' value)*
value    := STRING | object | array | bracketed
object   := '{' (STRING ':' value)* '}'
array    := '[' value* ']'
bracketed:= '[--[' <raw text> ']--]'
STRING   := '"' <characters up to the next '"'> '"'
```

- Members are separated by whitespace. There are no commas.
- `//` starts a line comment anywhere outside a string and outside a bracketed
  literal. The demo files use comments as array element indices (`//0`).
- Line endings may be CRLF or LF.
- Strings have no escape sequences. A `"` always ends a string. Free text that
  must contain a `"` uses a bracketed literal instead.
- A document is an ordered list of sections. Section order is not significant;
  `robot.mcsg` orders `object` before `mtl`.
- Keys may repeat, and order is preserved. `dotmcsg` models objects as ordered
  member lists for this reason.

## Sections

### `meta`

| Key | Type | Meaning |
| --- | --- | --- |
| `version` | integer | Format version. Demo 0.7.3 writes `730`; `robot.mcsg` writes `2`. |
| `hq` | flag | High-quality flag. |

### `render`

| Key | Type | Meaning |
| --- | --- | --- |
| `media_g` | float | Media gain. |

### `mtl`

An array of materials.

| Key | Type | Meaning |
| --- | --- | --- |
| `trans` | float | Transmission. |
| `ior` | float | Index of refraction. |
| `metal` | float | Metalness. |
| `rough` | float | Roughness. |
| `emit` | float | Emission. |
| `d` | float | Scatter density. |
| `color` | 3 or 4 bytes | Base colour, `R G B` or `R G B A`. |
| `media_color` | 3 bytes | Media colour. |

### `asset`

An array of `{"type": "..." "path": "..."}` entries. Empty in the demo files.

### `csg`

An array of stroke groups. Each group is an array of strokes, combined in order.
See "Strokes" below.

### `object`

An array of instances.

| Key | Type | Meaning |
| --- | --- | --- |
| `type` | enum | `csg` or `bg`. |
| `name` | string | Object name. |
| `res` | integer | SDF resolution. |
| `cid` | integer | Index into `csg` for a `csg` object. |
| `mid` | integer | Primary material index. |
| `mid2` | integer | Secondary material index. |
| `indent` | integer | Hierarchy indent. |
| `invis` | integer | Invisible flag. |
| `r` | 9 floats | Row-major 3x3 rotation. |
| `t` | 3 floats | Translation. |
| `s` | 3 floats | Scale. |
| `pivot_r` | 9 floats | Pivot rotation. |
| `pivot_t` | 3 floats | Pivot translation. |
| `pivot_k` | integer | Pivot flag. |
| `m` | integer | Mirror mask. |
| `lm` | integer | Symmetry mask. |
| `hidden` | integer | Hidden flag. |

### `palette`

An array of `"R G B"` strings.

## Strokes

A stroke is an object. The keys below are the ones the shipping writer emits.

### Shapes

`type`: `cube`, `sphere`, `cylinder`, `triangle`, `circle`, `polygon`, `bpoly`,
`bezier`, `chr`, `svg`, `csg`, `group`, `joint`, `line`, `line3`, `bezier3`,
`tetra`, `rhombus`. The demo files also write `jt`, which `dotmcsg` reads as
`joint`.

`subtype`: `elli_l2s`, `elli_linf`, `elli_l2` (sphere), `tri_iso`, `tri_para`,
`tri_tetra` (triangle), `cir_arc`, `cir_pie`, `cir_seg` (circle), `jt_line`,
`jt_egg` (joint).

### Profile operations

`op`: `extrude` (the default when absent), `revolve`, `torus`, `helix`,
`sweep`. The profile is the shape's cross-section, swept by the operation.

### Boolean operation

`mode`: `add` (the default when absent), `sub`, `inter`, `rep`. A group folds
its strokes in order; each stroke's `mode` combines it with the accumulated
field.

### Common attributes

| Key | Type | Meaning |
| --- | --- | --- |
| `name` | string | Stroke name. |
| `rgb` | 3 or 4 bytes | Stroke colour. |
| `mtl` | integer | Material slot override. |
| `blend` | float | Soft-min blend radius. |
| `m` | integer | Mirror mask (bit 0 X, bit 1 Y, bit 2 Z). |
| `lm` | integer | Symmetry mask. |
| `half` | integer | Half cut. |
| `hidden` | integer | Hidden flag. |
| `r`, `t`, `s` | 9, 3, 3 floats | Local placement. |

### Shape modifiers

| Key | Type | Applies to |
| --- | --- | --- |
| `n` | integer | Polygon vertex count. |
| `star%` | float | Star polygon inset. |
| `bevel%`, `bevel`, `bevel_v` | float | Bevel. |
| `round%`, `round`, `round_v` | float | Round. |
| `hole%`, `hole`, `hole_v` | float | Hole. |
| `shell%`, `shell`, `shell_v` | float | Shell. |
| `cone%`, `cone`, `cone_v`, `cone_iso`, `cone_ell` | float / flag | Cone taper. |
| `power` | float | Superellipsoid exponent. |
| `top_v%`, `top_w%` | float | Triangle apex controls. |
| `edge_chamfer%` | float | Box edge chamfer. |
| `curved` | integer | Curved joints. |
| `line_t` | float | Joint position along its line. |
| `poly` | integer | Polygon-from-points mode. |
| `sid` | integer | Sub-stroke index. |
| `sub_n` | integer | Sub-stroke count. |
| `closed`, `d2`, `pass`, `overridden` | flag | Path and boolean flags. |
| `rotx90` | integer | Rotate 90 degrees. |

### Blend modifiers

`blend`, `blend_mirror`, `blend_array`, `blend_chamfer`, `blend_groove`,
`blend_offset`, `blend_soft`, `groove_h`. These tune the soft-min blend.

### Array and radial modifiers

`mod` (`array` or `radial`), `array` (2 integers), `array_sp` (2 floats),
`radial` (integer), `radial_r`, `radial_span`.

### Profile paths

Paths carry `v_n` (vertex count) and a parallel value list:

| Key | Meaning |
| --- | --- |
| `v` | Vertex positions, `v_n * 3` floats. |
| `v_s` | Per-vertex size. |
| `v_q` | Per-vertex quaternion, 4 floats each. |
| `v_color` | Per-vertex colour, 4 bytes each. |
| `v_r` | Per-vertex round. |
| `v_c` | Per-vertex integer channels. |
| `v_3d` | Whether the path is 3D. |
| `path_size_con` | Size continuity. |
| `path_auto`, `path_loop`, `path_d`, `path_h`, `path_w`, `path_c`, `path_dim` | Path flags and parameters. |

### Helix

`helix_n`, `helix_s` (2 floats), `helix_z`.

### Revolve

`rev_r`, `rev_span`, `rev_rot`.

### Text

`text`: a bracketed literal. `chr` and `svg` strokes use it.

## Unknowns

- Whether a stroke of `type: csg` or `group`, with `sid`/`sub_n`, nests another
  group. No demo file uses it.
- The world-unit meaning of `blend`. It is written as an absolute `%.2f`, from
  0.36 against shapes tens of units across (`primitive-type.mcsg`) to 25.0
  (`robot.mcsg`).
- `mtl` slots: `mid` is the surface material and `mid2` the secondary, but
  which shape feature uses `mid2` is not established.
