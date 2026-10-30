#[allow(unused_imports)]
use super::*;
///A 3D Tiles tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tileset {
    ///Metadata about the entire tileset.
    pub asset: Asset,
    ///Names of 3D Tiles extensions required to properly load this tileset. Each element of this array shall also be contained in `extensionsUsed`.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub extensions_required: Vec<Box<str>>,
    ///Names of 3D Tiles extensions used somewhere in this tileset.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub extensions_used: Vec<Box<str>>,
    ///The error, in meters, introduced if this tileset is not rendered. At runtime, the geometric error is used to compute screen space error (SSE), i.e., the error measured in pixels.
    pub geometric_error: f64,
    ///An array of groups that tile content may belong to. Each element of this array is a metadata entity that describes the group. The tile content `group` property is an index into this array.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub groups: Vec<Group>,
    ///A metadata entity that is associated with this tileset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///A dictionary object of metadata about per-feature properties.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub properties: SortedMap<String, Properties>,
    ///The root tile.
    pub root: Tile,
    ///An object defining the structure of metadata classes and enums. When this is defined, then `schemaUri` shall be undefined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    ///The URI (or IRI) of the external schema file. When this is defined, then `schema` shall be undefined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_uri: Option<Box<str>>,
    ///An object containing statistics about metadata entities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Tileset {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
