#[allow(unused_imports)]
use super::*;
///A set of Batched 3D Model semantics that contain additional information about features in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batched3dModelFeatureTable {
    ///A `GlobalPropertyInteger` object defining an integer property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "BATCH_LENGTH")]
    pub batch_length: GlobalPropertyInteger,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "RTC_CENTER",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Batched3dModelFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
