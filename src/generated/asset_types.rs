#[allow(unused_imports)]
use super::*;
///Metadata about the entire tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    ///Application-specific version of this tileset, e.g., for when an existing tileset is updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tileset_version: Option<Box<str>>,
    ///The 3D Tiles version. The version defines the JSON schema for the tileset JSON and the base set of tile formats.
    pub version: Box<str>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Asset {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
