#[allow(unused_imports)]
use super::*;
///A bounding volume that encloses a tile or its content. At least one bounding volume property is required. Bounding volumes include `box`, `region`, or `sphere`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingVolume {
    ///An array of 12 numbers that define an oriented bounding box. The first three elements define the x, y, and z values for the center of the box. The next three elements (with indices 3, 4, and 5) define the x axis direction and half-length. The next three elements (indices 6, 7, and 8) define the y axis direction and half-length. The last three elements (indices 9, 10, and 11) define the z axis direction and half-length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#box: Option<[f64; 12]>,
    ///An array of six numbers that define a bounding geographic region in EPSG:4979 coordinates with the order \[west, south, east, north, minimum height, maximum height\]. Longitudes and latitudes are in radians. The range for latitudes is \[-PI/2,PI/2\]. The range for longitudes is \[-PI,PI\]. The value that is given as the 'south' of the region shall not be larger than the value for the 'north' of the region. The heights are in meters above (or below) the WGS84 ellipsoid. The 'minimum height' shall not be larger than the 'maximum height'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<[f64; 6]>,
    ///An array of four numbers that define a bounding sphere. The first three elements define the x, y, and z values for the center of the sphere. The last element (with index 3) defines the radius in meters. The radius shall not be negative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sphere: Option<[f64; 4]>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for BoundingVolume {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
