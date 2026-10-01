//! Data model and toolkit for [3D Tiles 1.0/1.1][spec].
//!
//! [spec]: https://github.com/CesiumGS/3d-tiles/tree/main/specification
//!
//! # Overview
//!
//! - [`Tileset`] (and the other generated types, re-exported at the crate
//!   root) is the in-memory data model, generated from the official 3D Tiles
//!   JSON schemas so every property and vendor extension round-trips.
//! - [`from_slice`] / [`from_str`] / [`from_reader`] parse and validate a
//!   `tileset.json`.
//! - [`TilesetNavigator`] exposes retainable cursors for caller-controlled
//!   BFS, DFS, conversion, analytics, and spatial queries. A
//!   [`TileResolver`] derives application-defined behavior from raw tile data.
//! - [`TilesetReader`] decorates a shared [`TilesetLoader`] with typed tileset
//!   and tile access while preserving every Hiera loader operation. It returns
//!   tile-content bytes without decoding GLB/glTF or choosing an executor.
//! - [`TilesetWriter`], [`SubtreeWriter`], and [`SchemaWriter`] serialize
//!   tilesets, implicit-tiling subtrees, and metadata schemas.
//! - [`SubtreeAvailability::from_bytes`] decodes binary/JSON subtree payloads, and the
//!   availability types ([`QuadtreeAvailability`], [`OctreeAvailability`])
//!   provide random access into implicit tiling.
//!
//! # Scope
//!
//! The crate stops at the 3D Tiles specification's boundary. Transport,
//! thread placement, geometry decoding, geodesy, and vendor extensions the
//! specification does not define are application decisions, so they are not
//! provided here: a loader is handed a fetch callback, and traversal is
//! driven by the caller.
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
//! assert_eq!(&*tileset.asset.version, "1.1");
//!
//! let mut uris = Vec::new();
//! tileset.for_each_content(|content| uris.push(content.uri.to_string()));
//! assert_eq!(uris, ["root.b3dm"]);
//! ```
//!
//! # Navigate a tileset
//!
//! ```
//! use hiera::{FetchRequest, FetchResponse};
//! use tairu::{TilesetLoader, TilesetNavigator};
//! # async fn example() {
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
//! let fetch = move |_: FetchRequest| async move {
//!     Ok::<_, hiera::FetchError>(FetchResponse {
//!         bytes: tileset_json.as_bytes().to_vec().into(),
//!         content_type: Some("application/json".into()),
//!     })
//! };
//! let loader = TilesetLoader::open("tileset.json", fetch);
//! let navigator = TilesetNavigator::new(loader);
//! let root = navigator.root().await.unwrap();
//! assert_eq!(root.depth(), 0);
//! # }
//! ```

#![warn(missing_docs)]

mod error;
mod extension;
mod generated;
mod implicit_tiling;
mod impls;
mod io;
mod navigation;
mod tile;
mod uri;

pub use error::Error;
pub use extension::{ExtMeshFeatures, ExtMeshFeaturesFeatureId, Extension, HasExtensions};
pub use generated::*;
pub use implicit_tiling::{
    AvailabilityNode, AvailabilityView, OctreeAvailability, OctreeAvailabilityNode, OctreeTileId,
    QuadtreeAvailability, QuadtreeTileId, SubtreeAvailability, SubtreeTileId,
    TileAvailabilityFlags,
};
pub use io::{
    ContentRef, LoadedContent, SchemaWriter, SubtreeWriter, TileBounds, TileInfo, TileRef,
    TilesetLoader, TilesetReader, TilesetWriter, WriteOptions, from_reader, from_slice, from_str,
};
pub use navigation::{TileCursor, TileExpansion, TileResolveInput, TileResolver, TilesetNavigator};
pub use tile::TileFormat;
pub use uri::{Uri, is_external_tileset_uri, resolve_uri};
