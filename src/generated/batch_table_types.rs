#[allow(unused_imports)]
use super::*;
///A set of properties defining application-specific metadata for features in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTable {
    #[serde(default, flatten)]
    pub additional_properties: std::collections::HashMap<String, BatchTableProperty>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for BatchTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///An object defining the reference to a section of the binary body of the batch table where the property values are stored if not defined directly in the JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTableBinaryBodyReference {
    ///The offset into the buffer in bytes.
    pub byte_offset: u64,
    ///The datatype of components in the property.
    pub component_type: Box<str>,
    ///Specifies if the property is a scalar or vector.
    pub r#type: Box<str>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for BatchTableBinaryBodyReference {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
