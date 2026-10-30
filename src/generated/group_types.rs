#[allow(unused_imports)]
use super::*;
///An object containing metadata about a group.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Group {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
