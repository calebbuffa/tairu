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
//! - [`walk`] traverses a tileset (including external tilesets, with cycle
//!   detection), handing each tile to your visitor with resolved transforms.
//! - [`TilesetWriter`], [`SubtreeWriter`], and [`SchemaWriter`] serialize
//!   tilesets, implicit-tiling subtrees, and metadata schemas.
//! - [`SubtreeAvailability::from_bytes`] decodes binary/JSON subtree payloads, and the
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
//! use kiba::{FetchRequest, FetchResponse};
//! use tairu::{TilesetLoader, TraversalControl, walk};
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
//!     Ok::<_, kiba::FetchError>(FetchResponse {
//!         bytes: tileset_json.as_bytes().to_vec(),
//!         content_type: Some("application/json".into()),
//!     })
//! };
//! let loader = TilesetLoader::open("tileset.json", fetch);
//! let mut count = 0;
//! walk(&loader, |visit| {
//!     count += 1;
//!     Ok::<_, std::io::Error>(TraversalControl::Continue)
//! })
//! .await
//! .unwrap();
//! assert_eq!(count, 1);
//! # }
//! ```

#![warn(missing_docs)]

mod error;
mod extension;
mod generated;
mod implicit_tiling;
mod impls;
mod io;
mod tile;
mod traversal;
mod uri;

pub use error::Error;
pub use extension::{
    EsriCrs, EsriCrsTransform, ExtMeshFeatures, ExtMeshFeaturesFeatureId, Extension, HasExtensions,
};
pub use generated::*;
pub use implicit_tiling::{
    AvailabilityNode, AvailabilityView, OctreeAvailability, OctreeAvailabilityNode, OctreeTileId,
    QuadtreeAvailability, QuadtreeTileId, SubtreeAvailability, SubtreeTileId,
    TileAvailabilityFlags,
};
pub use io::{
    ContentRef, LoadedContent, SchemaWriter, SubtreeWriter, TileRef, TilesetLoader, TilesetWriter,
    WriteOptions, from_reader, from_slice, from_str,
};
pub use tile::TileFormat;
pub use traversal::{TileVisit, TraversalControl, TraversalPolicy, walk, walk_with_policy};
pub use uri::{Uri, is_external_tileset_uri, resolve_uri};
