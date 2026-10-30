# Tairu

Rust implementation of [3D Tiles 1.0 and 1.1](https://github.com/CesiumGS/3d-tiles/tree/main/specification).

## Installation

```toml
[dependencies]
tairu = "0.2"
```

Requires Rust 1.85.

## Parse a tileset

```rust
use tairu::from_str;

let json = r#"{
    "asset": { "version": "1.1" },
    "geometricError": 100.0,
    "root": {
        "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 10.0] },
        "geometricError": 10.0,
        "refine": "REPLACE",
        "content": { "uri": "root.b3dm" }
    }
}"#;

let tileset = from_str(json).unwrap();
assert_eq!(tileset.asset.version.as_ref(), "1.1");
```

Parsing rejects invalid required data. `from_str_with_diagnostics` (and the
corresponding slice, reader, and named tileset functions) also returns
non-fatal findings with stable codes, JSON-style paths, and messages.
Use `write_tileset` with `WriteOptions` to serialize a tileset.

## What's included

- Generated schema types, including `Tileset`, `Tile`, and `BoundingVolume`,
  re-exported at the crate root.
- Parsing from strings, byte slices, and synchronous readers.
- Specification validation and structured diagnostics.
- Typed extension access through `Extension` and `HasExtensions`.
- Quadtree and octree availability, subtree decoding, and `SubtreeWriter`
  serialization.

See the [API documentation](https://docs.rs/tairu)
for parsing, validation, and subtree utilities.

## Development

Generated types are checked in. The development-only `xtask` toolchain
regenerates them from the schema submodules.

## License

Apache License 2.0. See [LICENSE](LICENSE).
