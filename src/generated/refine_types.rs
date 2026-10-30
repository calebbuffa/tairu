#[allow(unused_imports)]
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Refine {
    #[serde(rename = "ADD")]
    Add,
    #[serde(rename = "REPLACE")]
    Replace,
}
impl Default for Refine {
    fn default() -> Self {
        Self::Add
    }
}
