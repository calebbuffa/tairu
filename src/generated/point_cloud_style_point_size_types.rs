#[allow(unused_imports)]
use super::*;
///A `number expression` or `conditions` property which determines the size of the points in pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointCloudStylePointSize {
    NumberExpression(NumberExpression),
    Conditions(Conditions),
}
impl Default for PointCloudStylePointSize {
    fn default() -> Self {
        Self::NumberExpression(Default::default())
    }
}
