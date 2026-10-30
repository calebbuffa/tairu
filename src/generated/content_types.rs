#[allow(unused_imports)]
use super::*;
///Metadata about the tile's content and a link to the content.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    ///An optional bounding volume that tightly encloses tile content. tile.boundingVolume provides spatial coherence and tile.content.boundingVolume enables tight view frustum culling. When this is omitted, tile.boundingVolume is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounding_volume: Option<BoundingVolume>,
    ///The group this content belongs to. The value is an index into the array of `groups` that is defined for the containing tileset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<u64>,
    ///Metadata that is associated with this content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///A uri that points to tile content. When the uri is relative, it is relative to the referring tileset JSON file.
    pub uri: Box<str>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Content {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
