//! Typed structs for the EXT_mesh_features glTF extension.
//!
//! Reference: <https://github.com/CesiumGS/glTF/tree/proposal-EXT_mesh_features>

/// The `EXT_mesh_features` extension name.
const EXTENSION_NAME: &str = "EXT_mesh_features";

use super::Extension;
use serde::{Deserialize, Serialize};

/// The `EXT_mesh_features` extension data on a mesh primitive.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtMeshFeatures {
    /// The feature-ID descriptors carried by the primitive.
    pub feature_ids: Vec<ExtMeshFeaturesFeatureId>,
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

/// One feature-ID definition within `EXT_mesh_features`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtMeshFeaturesFeatureId {
    /// The number of features described by this definition.
    pub feature_count: u32,
    /// The vertex-attribute set index carrying per-vertex feature IDs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute: Option<u32>,
    /// The property-table index carrying per-feature properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_table: Option<u32>,
    /// A human-readable label for the feature set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
