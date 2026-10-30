#[allow(unused_imports)]
use super::*;
///A boolean or string with a 3D Tiles style expression that evaluates to a boolean. Details are described in the 3D Tiles Styling specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BooleanExpression {
    Boolean(bool),
    String(Box<str>),
}
impl Default for BooleanExpression {
    fn default() -> Self {
        Self::Boolean(Default::default())
    }
}
