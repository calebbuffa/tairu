#[allow(unused_imports)]
use super::*;
///A 3D Tiles style.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    ///A `color expression` or `conditions` property which determines the color blended with the feature's intrinsic color.
    #[serde(default = "default_style_color")]
    pub color: StyleColor,
    ///A dictionary object of `expression` strings mapped to a variable name key that may be referenced throughout the style. If an expression references a defined variable, it is replaced with the evaluated result of the corresponding expression.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub defines: SortedMap<String, Expression>,
    ///A `meta` object which determines the values of non-visual properties of the feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    ///A `boolean expression` or `conditions` property which determines if a feature should be shown.
    #[serde(default = "default_style_show")]
    pub show: StyleShow,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: default_style_color(),
            defines: Default::default(),
            meta: Default::default(),
            show: default_style_show(),
            extensions: Default::default(),
            extras: Default::default(),
        }
    }
}
impl crate::HasExtensions for Style {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
