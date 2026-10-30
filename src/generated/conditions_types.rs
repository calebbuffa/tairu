#[allow(unused_imports)]
use super::*;
///A series of conditions evaluated in order, like a series of if...else statements that result in an expression being evaluated.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conditions {
    ///A series of boolean conditions evaluated in order. For the first one that evaluates to true, its value, the 'result' (which is also an expression), is evaluated and returned. Result expressions shall all be the same type. If no condition evaluates to true, the result is `undefined`. When conditions is `undefined`, `null`, or an empty object, the result is `undefined`.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub conditions: Vec<Condition>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Conditions {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
