# Tairu

The [3D Tiles 1.0/1.1 specification](https://github.com/CesiumGS/3d-tiles/tree/main/specification)
as Rust data and common wire-format utilities.

The crate provides generated schema types, synchronous parsing and
serialization over caller-supplied bytes/readers/writers, validation,
typed extensions, and implicit-tiling availability and subtree layouts.
It has no feature flags, resource acquisition, filesystem or archive backend,
runtime, or worker pool.

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
The crate does not select a reporting policy.

## Application workflows

[Hiera](https://docs.rs/hiera) supplies resource acquisition, hierarchy
navigation, glTF integration, and package publication under application-owned
execution and byte admission. Applications can depend on Hiera alone:

```toml
[dependencies]
hiera = { version = "0.2", features = ["tairu"] }
```

`hiera::tairu` re-exports this crate's types and utilities alongside
`TilesetHierarchy`, `TilesetNavigator`, and the other adapters.
Its `SpecError` is `tairu::Error`; its `Error` describes adapter failures.
Hiera's `tairu-scene` feature adds standard transform/refinement resolution,
`moderu` enables glTF model integration, and `tairu-package` adds 3TZ storage.
No archive, runtime, or worker pool is selected by enabling `tairu`.

## Development

Generated modules are checked in. Building the crate does not require the
schema submodules or development-only `xtask` generator.

## License

Apache License 2.0. See [LICENSE](LICENSE).
