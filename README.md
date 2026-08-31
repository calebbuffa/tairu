# tairu

*tairu* ("tile") is a Rust crate for working with [3D Tiles 1.0/1.1](https://github.com/CesiumGS/3d-tiles/tree/main/specification) assets in memory.

## Features

- **Generated types** — auto-generated Rust structs covering the full 3D Tiles JSON schema.
- **Parsing** — parse and validate `tileset.json` from slices, strings, or readers (`from_slice`, `from_str`, `from_reader`), plus a bounded-memory streaming fold (`fold_from_reader`).
- **Traversal** — walk a tileset, including external tilesets with cycle detection, with resolved world transforms (`walk`, `TileVisit`).
- **Implicit tiling** — quadtree/octree availability with Morton-index random access (`QuadtreeAvailability`, `OctreeAvailability`, `SubtreeAvailability`), and binary/JSON subtree parsing (`parse_subtree`).
- **Writing** — serialize tilesets, subtrees, and metadata schemas (`TilesetWriter`, `SubtreeWriter`, `SchemaWriter`).
- **Typed extensions** — read/write vendor extensions as typed values (`Extension`, `HasExtensions`).

## Quick start

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
assert_eq!(tileset.asset.version, "1.1");

let mut uris = Vec::new();
tileset.for_each_content(|content| uris.push(content.uri.clone()));
assert_eq!(uris, ["root.b3dm"]);
```

## License

Apache License 2.0, see [LICENSE](LICENSE) for details.

