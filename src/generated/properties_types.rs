#[allow(unused_imports)]
use super::*;
///A dictionary object of metadata about per-feature properties.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    ///The maximum value of this property of all the features in the tileset. The maximum value shall not be smaller than the minimum value.
    pub maximum: f64,
    ///The minimum value of this property of all the features in the tileset. The maximum value shall not be smaller than the minimum value.
    pub minimum: f64,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Properties {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
