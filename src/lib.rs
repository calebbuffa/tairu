//! Data model and toolkit for [3D Tiles 1.0/1.1][spec].
//!
//! [spec]: https://github.com/CesiumGS/3d-tiles/tree/main/specification
//!
//! # Overview
//!
//! - [`Tileset`] (and the other [`generated`] types, re-exported at the crate
//!   root) is the in-memory data model, generated from the official 3D Tiles
//!   JSON schemas so every property and vendor extension round-trips.
//! - [`from_slice`] / [`from_str`] / [`from_reader`] parse and validate a
//!   `tileset.json`; [`fold_from_reader`] streams a tileset with bounded memory.
//! - [`walk`] traverses a tileset (including external tilesets, with cycle
//!   detection), handing each tile to your visitor with resolved transforms.
//! - [`TilesetWriter`], [`SubtreeWriter`], and [`SchemaWriter`] serialize
//!   tilesets, implicit-tiling subtrees, and metadata schemas.
//! - [`parse_subtree`] decodes binary/JSON subtree payloads, and the
//!   availability types ([`QuadtreeAvailability`], [`OctreeAvailability`])
//!   provide random access into implicit tiling.
//!
//! # Parse and inspect a tileset
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
//! assert_eq!(tileset.asset.version, "1.1");
//!
//! let mut uris = Vec::new();
//! tileset.for_each_content(|content| uris.push(content.uri.clone()));
//! assert_eq!(uris, ["root.b3dm"]);
//! ```
//!
//! # Walk a tileset
//!
//! ```
//! use tairu::{TraversalControl, walk};
//!
//! let tileset_json = r#"{
//!     "asset": { "version": "1.1" },
//!     "geometricError": 100.0,
//!     "root": {
//!         "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 10.0] },
//!         "geometricError": 10.0
//!     }
//! }"#;
//!
//! // The fetch closure resolves resource URIs (in-memory here; typically HTTP
//! // or file system). External tilesets are followed automatically.
//! let mut fetch = |_: &str| -> Result<Vec<u8>, std::io::Error> {
//!     Ok(tileset_json.as_bytes().to_vec())
//! };
//! let mut count = 0;
//! walk("tileset.json", &mut fetch, |visit| {
//!     count += 1;
//!     Ok(TraversalControl::Continue)
//! })
//! .unwrap();
//! assert_eq!(count, 1);
//! ```
//!
//! # Write a tileset
//!
//! ```
//! use tairu::{Asset, Tileset, TilesetWriter, WriteOptions};
//!
//! let tileset = Tileset {
//!     asset: Asset { version: "1.1".into(), ..Default::default() },
//!     geometric_error: 1000.0,
//!     ..Default::default()
//! };
//!
//! let result = TilesetWriter::write_tileset(&tileset, WriteOptions::default());
//! assert!(result.errors.is_empty());
//! assert!(result.bytes.starts_with(b"{"));
//! ```

#![warn(missing_docs)]

mod availability;
mod error;
mod extension;
mod generated;
mod implicit_tiling;
mod impls;
mod metadata_query;
mod reader;
mod subtree;
mod tile;
mod traversal;
mod uri;
mod writer;

pub use generated::*;

// Primary unified error type
pub use error::Error;

pub use availability::{
    AvailabilityNode, AvailabilityView, OctreeAvailability, OctreeAvailabilityNode, OctreeTileId,
    QuadtreeAvailability, QuadtreeTileId, SubtreeAvailability, TileAvailabilityFlags,
};
pub use extension::{
    EsriCrs, EsriCrsTransform, ExtMeshFeatures, ExtMeshFeaturesFeatureId, Extension, HasExtensions,
};
pub use metadata_query::{FoundMetadataProperty, MetadataQuery};
pub use reader::{
    TileParseError, fold_from_reader, from_reader, from_slice, from_str, load, load_async,
};
pub use subtree::{SubtreeParseError, parse_subtree, parse_subtree_with_buffers};
pub use tile::TileFormat;
pub use traversal::{TileVisit, TraversalControl, walk};
pub use uri::{Uri, is_external_tileset_uri, resolve_uri};
pub use writer::{
    SchemaWriter, SchemaWriterResult, SubtreeWriter, SubtreeWriterResult, TilesetWriter,
    TilesetWriterResult, WriteOptions,
};

// Legacy error types for backward compatibility during transition
pub use traversal::WalkError;
pub use uri::UriLoadError;
