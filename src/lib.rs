//! Data model for [3D Tiles 1.0/1.1][spec].
//!
//! [spec]: https://github.com/CesiumGS/3d-tiles/tree/main/specification
//!
//! # Overview
//!
//! - [`Tileset`] (and the other generated types, re-exported at the crate
//!   root) is the in-memory data model, generated from the official 3D Tiles
//!   JSON schemas. Extension objects retain their JSON representation.
//! - [`from_slice`] / [`from_str`] / [`from_reader`] / [`parse_tileset`] parse
//!   and validate a `tileset.json`.
//! - [`write_tileset`], [`SubtreeWriter`], and [`write_schema`] serialize
//!   tilesets, implicit-tiling subtrees, and metadata schemas.
//! - [`SubtreeAvailability::from_bytes`] decodes binary/JSON subtree payloads,
//!   and the availability types ([`QuadtreeAvailability`],
//!   [`OctreeAvailability`]) provide random access into implicit tiling.
//!
//! # Scope
//!
//! The crate is the 3D Tiles specification only. It parses and serializes
//! caller-supplied data without acquiring resources or owning threads.
//! Resource reading, hierarchy traversal, and navigation are provided by
//! the `hiera` crate's `tairu` feature.
//!
//! ```
//! use tairu::from_str;
//!
//! let json = r#"{
//!     "asset": { "version": "1.1" },
//!     "geometricError": 100.0,
//!     "root": {
//!         "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 10.0] },
//!         "geometricError": 10.0,
//!         "refine": "REPLACE",
//!         "content": { "uri": "root.b3dm" }
//!     }
//! }"#;
//!
//! let tileset = from_str(json).unwrap();
//! assert_eq!(&*tileset.asset.version, "1.1");
//!
//! let mut uris = Vec::new();
//! tileset.root.for_each_content(|content| uris.push(content.uri.to_string()));
//! assert_eq!(uris, ["root.b3dm"]);
//! ```

#![warn(missing_docs)]

mod error;
mod extension;
mod generated;
mod implicit_tiling;
mod impls;
mod io;
mod tile;
mod uri;
mod validation;

pub use error::Error;
pub use extension::{ExtMeshFeatures, ExtMeshFeaturesFeatureId, Extension, HasExtensions};
pub use generated::*;
pub use implicit_tiling::{
    AvailabilityView, OctreeAvailability, OctreeTileId, QuadtreeAvailability, QuadtreeTileId,
    SubtreeAvailability, SubtreeTileId, TileAvailabilityFlags,
};
pub use io::{
    SubtreeWriter, WriteOptions, from_reader, from_reader_with_diagnostics, from_slice,
    from_slice_with_diagnostics, from_str, from_str_with_diagnostics, parse_tileset,
    parse_tileset_with_diagnostics, write_schema, write_tileset,
};
pub use tile::TileFormat;
pub use uri::is_external_tileset_uri;
pub use validation::{Diagnostic, validate_tileset};
