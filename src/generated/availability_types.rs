#[allow(unused_imports)]
use super::*;
///An object describing the availability of a set of elements.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    ///A number indicating how many 1 bits exist in the availability bitstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_count: Option<u64>,
    ///Index of a buffer view that indicates whether each element is available. The bitstream conforms to the boolean array encoding described in the 3D Metadata specification. If an element is available, its bit is 1, and if it is unavailable, its bit is 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitstream: Option<usize>,
    ///Integer indicating whether all of the elements are available (1) or all are unavailable (0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constant: Option<u64>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Availability {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
