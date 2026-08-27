//! [spec]: https://github.com/CesiumGS/3d-tiles/tree/main/specification
mod availability;
mod error;
mod esri_crs;
mod ext_mesh_features;
mod extensions;
mod generated;
mod implicit_tiling;
mod impls;
mod metadata_query;
mod reader;
mod resource;
mod subtree;
mod tile;
mod traversal;
mod writer;

pub use generated::*;

// Primary unified error type
pub use error::Error;

pub use availability::{
    AvailabilityNode, AvailabilityView, OctreeAvailability, OctreeAvailabilityNode, OctreeTileId,
    QuadtreeAvailability, QuadtreeTileId, SubtreeAvailability, TileAvailabilityFlags,
};
pub use esri_crs::{EsriCrs, EsriCrsTransform};
pub use ext_mesh_features::{EXTENSION_NAME as EXT_MESH_FEATURES_NAME, ExtMeshFeatures, FeatureId};
pub use extensions::{Extension, HasExtensions};
pub use implicit_tiling::*;
pub use metadata_query::{FoundMetadataProperty, MetadataQuery};
pub use reader::{
    TileParseError, fold_from_reader, from_reader, from_slice, from_str, load, load_async,
};
pub use resource::{is_external_tileset_uri, resolve_uri};
pub use subtree::{SubtreeParseError, parse_subtree, parse_subtree_with_buffers};
pub use tile::TileFormat;
pub use traversal::{TileVisit, TraversalControl, walk};
pub use writer::{
    SchemaWriter, SchemaWriterResult, SubtreeWriter, SubtreeWriterResult, TilesetWriter,
    TilesetWriterResult, WriteOptions,
};

// Legacy error types for backward compatibility during transition
pub use resource::UriLoadError;
pub use traversal::WalkError;
