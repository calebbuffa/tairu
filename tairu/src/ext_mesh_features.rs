//! Typed structs for the EXT_mesh_features glTF extension.
//!
//! Reference: https://github.com/CesiumGS/glTF/tree/proposal-EXT_mesh_features

// GltfExtension registration lives in the cesna-gltf layer, not here.
pub const EXTENSION_NAME: &str = "EXT_mesh_features";

use crate::extensions::Extension;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtMeshFeatures {
    pub feature_ids: Vec<FeatureId>,
}

impl Extension for ExtMeshFeatures {
    const NAME: &'static str = EXTENSION_NAME;
}

impl ExtMeshFeatures {
    /// Parse an `EXT_mesh_features` extension object from a raw JSON value.
    ///
    /// Returns `None` if the value cannot be deserialized as `ExtMeshFeatures`.
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureId {
    pub feature_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_table: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
