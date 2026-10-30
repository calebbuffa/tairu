#[allow(unused_imports)]
use super::*;
///A user-defined property which specifies per-feature application-specific metadata in a tile. Values either can be defined directly in the JSON as an array, or can refer to sections in the binary body with a `BinaryBodyReference` object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BatchTableProperty {
    BatchTableBinaryBodyReference(BatchTableBinaryBodyReference),
    Array(Vec<serde_json::Value>),
}
impl Default for BatchTableProperty {
    fn default() -> Self {
        Self::BatchTableBinaryBodyReference(Default::default())
    }
}
