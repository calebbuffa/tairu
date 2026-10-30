#[allow(unused_imports)]
use super::*;
///An object defining a global 3-component numeric property values for all features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian3 {
    BinaryBodyOffset(BinaryBodyOffset),
    Array3([f64; 3]),
}
impl Default for GlobalPropertyCartesian3 {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
///An object defining a global 4-component numeric property values for all features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian4 {
    BinaryBodyOffset(BinaryBodyOffset),
    Array4([f64; 4]),
}
impl Default for GlobalPropertyCartesian4 {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
///An object defining a global integer property value for all features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyInteger {
    BinaryBodyOffset(BinaryBodyOffset),
    Integer(u64),
}
impl Default for GlobalPropertyInteger {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
///An object defining a global numeric property value for all features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyNumber {
    BinaryBodyOffset(BinaryBodyOffset),
    Number(f64),
}
impl Default for GlobalPropertyNumber {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
