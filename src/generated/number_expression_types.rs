#[allow(unused_imports)]
use super::*;
///3D Tiles style expression that evaluates to a number. Details are described in the 3D Tiles Styling specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NumberExpression {
    Number(f64),
    String(Box<str>),
}
impl Default for NumberExpression {
    fn default() -> Self {
        Self::Number(Default::default())
    }
}
