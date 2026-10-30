#[allow(unused_imports)]
use super::*;
///Statistics about entities.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    ///A dictionary, where each key corresponds to a class ID in the `classes` dictionary of the metatata schema that was defined for the tileset that contains these statistics. Each value is an object containing statistics about entities that conform to the class.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub classes: SortedMap<String, ClassStatistics>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Statistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
