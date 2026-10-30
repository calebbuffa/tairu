#[allow(unused_imports)]
use super::*;
///A user-defined property which specifies application-specific metadata in a tile. Values can refer to sections in the binary body with a `BinaryBodyReference` object. Global values can be also be defined directly in the JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureTableProperty {
    FeatureTableBinaryBodyReference(FeatureTableBinaryBodyReference),
    GlobalPropertyBoolean(GlobalPropertyBoolean),
    GlobalPropertyInteger(GlobalPropertyInteger),
    GlobalPropertyNumber(GlobalPropertyNumber),
    GlobalPropertyCartesian3(GlobalPropertyCartesian3),
    GlobalPropertyCartesian4(GlobalPropertyCartesian4),
}
impl Default for FeatureTableProperty {
    fn default() -> Self {
        Self::FeatureTableBinaryBodyReference(Default::default())
    }
}
