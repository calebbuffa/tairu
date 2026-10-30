#[allow(unused_imports)]
use super::*;
///A set of Point Cloud semantics that contains values defining the position and appearance properties for points in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointCloudFeatureTable {
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "BATCH_ID", default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<FeatureTableBinaryBodyReference>,
    ///A `GlobalPropertyInteger` object defining an integer property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "BATCH_LENGTH",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub batch_length: Option<GlobalPropertyInteger>,
    ///A `GlobalPropertyCartesian4` object defining a 4-component numeric property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "CONSTANT_RGBA",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub constant_rgba: Option<GlobalPropertyCartesian4>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "NORMAL", default, skip_serializing_if = "Option::is_none")]
    pub normal: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "NORMAL_OCT16P",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub normal_oct16p: Option<FeatureTableBinaryBodyReference>,
    ///A `GlobalPropertyInteger` object defining an integer property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "POINTS_LENGTH")]
    pub points_length: GlobalPropertyInteger,
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
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "QUANTIZED_VOLUME_OFFSET",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_offset: Option<GlobalPropertyCartesian3>,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "QUANTIZED_VOLUME_SCALE",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_scale: Option<GlobalPropertyCartesian3>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "RGB", default, skip_serializing_if = "Option::is_none")]
    pub rgb: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "RGB565", default, skip_serializing_if = "Option::is_none")]
    pub rgb565: Option<FeatureTableBinaryBodyReference>,
    ///A `BinaryBodyReference` object defining the reference to a section of the binary body where the property values are stored. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "RGBA", default, skip_serializing_if = "Option::is_none")]
    pub rgba: Option<FeatureTableBinaryBodyReference>,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all points. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "RTC_CENTER",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for PointCloudFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///A 3D Tiles style with additional properties for Point Clouds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointCloudStyle {
    ///A `number expression` or `conditions` property which determines the size of the points in pixels.
    #[serde(default = "default_point_cloud_style_point_size")]
    pub point_size: PointCloudStylePointSize,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl Default for PointCloudStyle {
    fn default() -> Self {
        Self {
            point_size: default_point_cloud_style_point_size(),
            extensions: Default::default(),
            extras: Default::default(),
        }
    }
}
impl crate::HasExtensions for PointCloudStyle {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
