#[allow(unused_imports)]
use super::*;
///A tile in a 3D Tiles tileset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    ///The bounding volume that encloses the tile.
    pub bounding_volume: BoundingVolume,
    ///An array of objects that define child tiles. Each child tile content is fully enclosed by its parent tile's bounding volume and, generally, has a geometricError less than its parent tile's geometricError. For leaf tiles, there are no children, and this property may not be defined.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub children: Vec<Tile>,
    ///Metadata about the tile's content and a link to the content. When this is omitted the tile is just used for culling. When this is defined, then `contents` shall be undefined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    ///An array of contents. When this is defined, then `content` shall be undefined.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub contents: Vec<Content>,
    ///The error, in meters, introduced if this tile is rendered and its children are not. At runtime, the geometric error is used to compute screen space error (SSE), i.e., the error measured in pixels.
    pub geometric_error: f64,
    ///An object that describes the implicit subdivision of this tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implicit_tiling: Option<ImplicitTiling>,
    ///A metadata entity that is associated with this tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///Specifies if additive or replacement refinement is used when traversing the tileset for rendering. This property is required for the root tile of a tileset; it is optional for all other tiles. The default is to inherit from the parent tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refine: Option<Refine>,
    ///A floating-point 4x4 affine transformation matrix, stored in column-major order, that transforms the tile's content--i.e., its features as well as content.boundingVolume, boundingVolume, and viewerRequestVolume--from the tile's local coordinate system to the parent tile's coordinate system, or, in the case of a root tile, from the tile's local coordinate system to the tileset's coordinate system. `transform` does not apply to any volume property when the volume is a region, defined in EPSG:4979 coordinates. `transform` scales the `geometricError` by the maximum scaling factor from the matrix.
    #[serde(default = "default_tile_transform")]
    pub transform: [f64; 16],
    ///Optional bounding volume that defines the volume the viewer shall be inside of before the tile's content will be requested and before the tile will be refined based on geometricError.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewer_request_volume: Option<BoundingVolume>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl Default for Tile {
    fn default() -> Self {
        Self {
            bounding_volume: Default::default(),
            children: Default::default(),
            content: Default::default(),
            contents: Default::default(),
            geometric_error: Default::default(),
            implicit_tiling: Default::default(),
            metadata: Default::default(),
            refine: Default::default(),
            transform: default_tile_transform(),
            viewer_request_volume: Default::default(),
            extensions: Default::default(),
            extras: Default::default(),
        }
    }
}
impl crate::HasExtensions for Tile {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
