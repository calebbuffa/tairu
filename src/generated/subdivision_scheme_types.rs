#[allow(unused_imports)]
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubdivisionScheme {
    #[serde(rename = "Quadtree")]
    Quadtree,
    #[serde(rename = "Octree")]
    Octree,
    #[serde(rename = "S2")]
    S2,
}
impl Default for SubdivisionScheme {
    fn default() -> Self {
        Self::Quadtree
    }
}
