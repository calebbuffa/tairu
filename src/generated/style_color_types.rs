#[allow(unused_imports)]
use super::*;
///A `color expression` or `conditions` property which determines the color blended with the feature's intrinsic color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StyleColor {
    ColorExpression(ColorExpression),
    Conditions(Conditions),
}
impl Default for StyleColor {
    fn default() -> Self {
        Self::ColorExpression(Default::default())
    }
}
///A `boolean expression` or `conditions` property which determines if a feature should be shown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StyleShow {
    BooleanExpression(BooleanExpression),
    Conditions(Conditions),
}
impl Default for StyleShow {
    fn default() -> Self {
        Self::BooleanExpression(Default::default())
    }
}
