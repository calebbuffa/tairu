#[allow(unused_imports)]
use super::*;
///A set of Instanced 3D Model semantics that contains values defining the position and appearance properties for instanced models in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instanced3dModelFeatureTable {
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "BATCH_ID", default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<FeatureTableBinaryBodyReference>,
    ///A `GlobalPropertyBoolean` object defining a boolean property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "EAST_NORTH_UP",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub east_north_up: Option<GlobalPropertyBoolean>,
    ///A `GlobalPropertyInteger` object defining an integer property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "INSTANCES_LENGTH")]
    pub instances_length: GlobalPropertyInteger,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "NORMAL_RIGHT",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normal_right: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "NORMAL_RIGHT_OCT32P",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normal_right_oct32p: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "NORMAL_UP", default, skip_serializing_if = "Option::is_none")]
    pub normal_up: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "NORMAL_UP_OCT32P",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normal_up_oct32p: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "POSITION", default, skip_serializing_if = "Option::is_none")]
    pub position: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "POSITION_QUANTIZED",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub position_quantized: Option<FeatureTableBinaryBodyReference>,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "QUANTIZED_VOLUME_OFFSET",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_offset: Option<GlobalPropertyCartesian3>,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "QUANTIZED_VOLUME_SCALE",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_scale: Option<GlobalPropertyCartesian3>,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "RTC_CENTER",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "SCALE", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "SCALE_NON_UNIFORM",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scale_non_uniform: Option<FeatureTableBinaryBodyReference>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Instanced3dModelFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
