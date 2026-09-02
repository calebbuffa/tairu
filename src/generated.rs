//!Generated 3D Tiles data model. Do not edit.
#![allow(missing_docs)]
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
///An arbitrary JSON value described by the 3D Tiles metadata specification.
pub type AnyValue = serde_json::Value;
///A numeric value described by the 3D Tiles metadata specification.
pub type NumericValue = serde_json::Value;
///A no-data value described by the 3D Tiles metadata specification.
pub type NoDataValue = serde_json::Value;
///An arbitrary JSON value.
pub type Value = serde_json::Value;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BatchTableProperty {
    BatchTableBinaryBodyReference(BatchTableBinaryBodyReference),
    Array(Vec<serde_json::Value>),
}
impl Default for BatchTableProperty {
    fn default() -> Self {
        Self::BatchTableBinaryBodyReference(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BooleanExpression {
    Boolean(bool),
    String(String),
}
impl Default for BooleanExpression {
    fn default() -> Self {
        Self::Boolean(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureTableProperty {
    FeatureTableBinaryBodyReference(FeatureTableBinaryBodyReference),
    GlobalPropertyBoolean(GlobalPropertyBoolean),
    GlobalPropertyInteger(GlobalPropertyInteger),
    GlobalPropertyNumber(GlobalPropertyNumber),
    GlobalPropertyCartesian3(GlobalPropertyCartesian3),
    GlobalPropertyCartesian4(GlobalPropertyCartesian4),
}
impl Default for FeatureTableProperty {
    fn default() -> Self {
        Self::FeatureTableBinaryBodyReference(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian3 {
    BinaryBodyOffset(BinaryBodyOffset),
    Array3([f64; 3]),
}
impl Default for GlobalPropertyCartesian3 {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian4 {
    BinaryBodyOffset(BinaryBodyOffset),
    Array4([f64; 4]),
}
impl Default for GlobalPropertyCartesian4 {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyInteger {
    BinaryBodyOffset(BinaryBodyOffset),
    Integer(u64),
}
impl Default for GlobalPropertyInteger {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyNumber {
    BinaryBodyOffset(BinaryBodyOffset),
    Number(f64),
}
impl Default for GlobalPropertyNumber {
    fn default() -> Self {
        Self::BinaryBodyOffset(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NumberExpression {
    Number(f64),
    String(String),
}
impl Default for NumberExpression {
    fn default() -> Self {
        Self::Number(Default::default())
    }
}
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StyleColor {
    ColorExpression(ColorExpression),
    Conditions(Conditions),
}
impl Default for StyleColor {
    fn default() -> Self {
        Self::ColorExpression(Default::default())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StyleShow {
    BooleanExpression(BooleanExpression),
    Conditions(Conditions),
}
impl Default for StyleShow {
    fn default() -> Self {
        Self::BooleanExpression(Default::default())
    }
}
///Metadata about the entire tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    ///Application-specific version of this tileset, e.g., for when an existing tileset is updated.
    #[serde(
        rename = "tilesetVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tileset_version: Option<String>,
    ///The 3D Tiles version. The version defines the JSON schema for the tileset JSON and the base set of tile formats.
    pub version: String,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object describing the availability of a set of elements.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    ///A number indicating how many 1 bits exist in the availability bitstream.
    #[serde(
        rename = "availableCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub available_count: Option<u64>,
    ///Index of a buffer view that indicates whether each element is available. The bitstream conforms to the boolean array encoding described in the 3D Metadata specification. If an element is available, its bit is 1, and if it is unavailable, its bit is 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitstream: Option<usize>,
    ///Integer indicating whether all of the elements are available (1) or all are unavailable (0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constant: Option<u64>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A set of properties defining application-specific metadata for features in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTable {
    #[serde(default, flatten)]
    pub additional_properties: std::collections::HashMap<String, BatchTableProperty>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object defining the reference to a section of the binary body of the batch table where the property values are stored if not defined directly in the JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTableBinaryBodyReference {
    ///The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: u64,
    ///The datatype of components in the property.
    #[serde(rename = "componentType")]
    pub component_type: String,
    ///Specifies if the property is a scalar or vector.
    #[serde(rename = "type")]
    pub r#type: String,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A set of Batched 3D Model semantics that contain additional information about features in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batched3dModelFeatureTable {
    ///A `GlobalPropertyInteger` object defining an integer property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(rename = "BATCH_LENGTH")]
    pub batch_length: GlobalPropertyInteger,
    ///A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all features. Details about this property are described in the 3D Tiles specification.
    #[serde(
        rename = "RTC_CENTER",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object defining the offset into a section of the binary body of the features table where the property values are stored if not defined directly in the JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryBodyOffset {
    ///The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: u64,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A bounding volume that encloses a tile or its content. At least one bounding volume property is required. Bounding volumes include `box`, `region`, or `sphere`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingVolume {
    ///An array of 12 numbers that define an oriented bounding box. The first three elements define the x, y, and z values for the center of the box. The next three elements (with indices 3, 4, and 5) define the x axis direction and half-length. The next three elements (indices 6, 7, and 8) define the y axis direction and half-length. The last three elements (indices 9, 10, and 11) define the z axis direction and half-length.
    #[serde(rename = "box", default, skip_serializing_if = "Option::is_none")]
    pub r#box: Option<[f64; 12]>,
    ///An array of six numbers that define a bounding geographic region in EPSG:4979 coordinates with the order [west, south, east, north, minimum height, maximum height]. Longitudes and latitudes are in radians. The range for latitudes is [-PI/2,PI/2]. The range for longitudes is [-PI,PI]. The value that is given as the 'south' of the region shall not be larger than the value for the 'north' of the region. The heights are in meters above (or below) the WGS84 ellipsoid. The 'minimum height' shall not be larger than the 'maximum height'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<[f64; 6]>,
    ///An array of four numbers that define a bounding sphere. The first three elements define the x, y, and z values for the center of the sphere. The last element (with index 3) defines the radius in meters. The radius shall not be negative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sphere: Option<[f64; 4]>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A buffer is a binary blob. It is either the binary chunk of the subtree file, or an external buffer referenced by a URI.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buffer {
    ///The length of the buffer in bytes.
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
    ///The name of the buffer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///The URI (or IRI) of the file that contains the binary buffer data. Relative paths are relative to the file containing the buffer JSON. `uri` is required when using the JSON subtree format and not required when using the binary subtree format - when omitted the buffer refers to the binary chunk of the subtree file. Data URIs are not allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip)]
    pub data: Vec<u8>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A contiguous subset of a buffer
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferView {
    ///The index of the buffer.
    pub buffer: usize,
    ///The total byte length of the buffer view.
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
    ///The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
    ///The name of the `bufferView`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A class containing a set of properties.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    ///The description of the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///The name of the class, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///A dictionary, where each key is a property ID and each value is an object defining the property. Property IDs shall be alphanumeric identifiers matching the regular expression `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, ClassProperty>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A single property of a metadata class.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassProperty {
    ///Whether the property is an array. When `count` is defined the property is a fixed-length array. Otherwise the property is a variable-length array.
    #[serde(default = "default_class_property_array")]
    pub array: bool,
    ///The datatype of the element's components. Required for `SCALAR`, `VECN`, and `MATN` types, and disallowed for other types.
    #[serde(
        rename = "componentType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub component_type: Option<serde_json::Value>,
    ///The number of array elements. May only be defined when `array` is `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    ///A default value to use when encountering a `noData` value or an omitted property. The value is given in its final form, taking the effect of `normalized`, `offset`, and `scale` properties into account. Shall not be defined if `required` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<AnyValue>,
    ///The description of the property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///Enum ID as declared in the `enums` dictionary. Required when `type` is `ENUM`. Disallowed when `type` is not `ENUM`
    #[serde(rename = "enumType", default, skip_serializing_if = "Option::is_none")]
    pub enum_type: Option<String>,
    ///Maximum allowed value for the property. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the maximum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<NumericValue>,
    ///Minimum allowed value for the property. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the minimum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<NumericValue>,
    ///The name of the property, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///A `noData` value represents missing data — also known as a sentinel value — wherever it appears. `BOOLEAN` properties may not specify `noData` values. This is given as the plain property value, without the transforms from the `normalized`, `offset`, and `scale` properties. Shall not be defined if `required` is true.
    #[serde(rename = "noData", default, skip_serializing_if = "Option::is_none")]
    pub no_data: Option<NoDataValue>,
    ///Specifies whether integer values are normalized. Only applicable to `SCALAR`, `VECN`, and `MATN` types with integer component types. For unsigned integer component types, values are normalized between `[0.0, 1.0]`. For signed integer component types, values are normalized between `[-1.0, 1.0]`. For all other component types, this property shall be false.
    #[serde(default = "default_class_property_normalized")]
    pub normalized: bool,
    ///An offset to apply to property values. Only applicable to `SCALAR`, `VECN`, and `MATN` types when the component type is `FLOAT32` or `FLOAT64`, or when the property is `normalized`. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<NumericValue>,
    ///If required, the property shall be present in every entity conforming to the class. If not required, individual entities may include `noData` values, or the entire property may be omitted. As a result, `noData` has no effect on a required property. Client implementations may use required properties to make performance optimizations.
    #[serde(default = "default_class_property_required")]
    pub required: bool,
    ///A scale to apply to property values. Only applicable to `SCALAR`, `VECN`, and `MATN` types when the component type is `FLOAT32` or `FLOAT64`, or when the property is `normalized`. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<NumericValue>,
    ///An identifier that describes how this property should be interpreted. The semantic cannot be used by other properties in the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<String>,
    ///The element type.
    #[serde(rename = "type")]
    pub r#type: Option<serde_json::Value>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_class_property_array() -> bool {
    false
}
fn default_class_property_normalized() -> bool {
    false
}
fn default_class_property_required() -> bool {
    false
}
///Statistics about entities that conform to a class that was defined in a metadata schema.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStatistics {
    ///The number of entities that conform to the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    ///A dictionary, where each key corresponds to a property ID in the class' `properties` dictionary and each value is an object containing statistics about property values.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, PropertyStatistics>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///3D Tiles style `expression` that evaluates to a Color. Details are described in the 3D Tiles Styling specification.
pub type ColorExpression = String;
///An `expression` evaluated as the result of a condition being true. An array of two expressions. If the first expression is evaluated and the result is `true`, then the second expression is evaluated and returned as the result of the condition.
pub type Condition = [Expression; 2];
///A series of conditions evaluated in order, like a series of if...else statements that result in an expression being evaluated.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conditions {
    ///A series of boolean conditions evaluated in order. For the first one that evaluates to true, its value, the 'result' (which is also an expression), is evaluated and returned. Result expressions shall all be the same type. If no condition evaluates to true, the result is `undefined`. When conditions is `undefined`, `null`, or an empty object, the result is `undefined`.
    #[serde(default)]
    pub conditions: Vec<Condition>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Metadata about the tile's content and a link to the content.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    ///An optional bounding volume that tightly encloses tile content. tile.boundingVolume provides spatial coherence and tile.content.boundingVolume enables tight view frustum culling. When this is omitted, tile.boundingVolume is used.
    #[serde(
        rename = "boundingVolume",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bounding_volume: Option<BoundingVolume>,
    ///The group this content belongs to. The value is an index into the array of `groups` that is defined for the containing tileset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<u64>,
    ///Metadata that is associated with this content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///A uri that points to tile content. When the uri is relative, it is relative to the referring tileset JSON file.
    pub uri: String,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Common definitions used in schema files.
pub type Definitions = serde_json::Value;
///An object defining the values of an enum.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    ///The description of the enum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///The name of the enum, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///The type of the integer enum value.
    #[serde(rename = "valueType", default = "default_enum_value_type")]
    pub value_type: Option<serde_json::Value>,
    ///An array of enum values. Duplicate names or duplicate integer values are not allowed.
    pub values: Vec<EnumValue>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_enum_value_type() -> Option<serde_json::Value> {
    Some(serde_json::Value::String("UINT16".to_owned()))
}
///An enum value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    ///The description of the enum value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///The name of the enum value.
    pub name: String,
    ///The integer enum value.
    pub value: i64,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A valid 3D Tiles style expression. Details are described in the 3D Tiles Styling specification.
pub type Expression = String;
///Dictionary object with extension-specific objects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    #[serde(default, flatten)]
    pub additional_properties: std::collections::HashMap<String, serde_json::Value>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Application-specific data.
pub type Extras = serde_json::Value;
///A set of semantics containing per-tile and per-feature values defining the position and appearance properties for features in a tile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTable {
    #[serde(default, flatten)]
    pub additional_properties: std::collections::HashMap<String, FeatureTableProperty>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object defining the reference to a section of the binary body of the features table where the property values are stored if not defined directly in the JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTableBinaryBodyReference {
    ///The datatype of components in the property. This is defined only if the semantic allows for overriding the implicit component type. These cases are specified in each tile format.
    #[serde(
        rename = "componentType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub component_type: Option<String>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object defining a global boolean property value for all features.
pub type GlobalPropertyBoolean = bool;
///An object containing metadata about a group.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///This object allows a tile to be implicitly subdivided. Tile and content availability and metadata is stored in subtrees which are referenced externally.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplicitTiling {
    ///The numbers of the levels in the tree with available tiles.
    #[serde(rename = "availableLevels")]
    pub available_levels: u64,
    ///A string describing the subdivision scheme used within the tileset.
    #[serde(rename = "subdivisionScheme")]
    pub subdivision_scheme: SubdivisionScheme,
    ///The number of distinct levels in each subtree. For example, a quadtree with `subtreeLevels = 2` will have subtrees with 5 nodes (one root and 4 children).
    #[serde(rename = "subtreeLevels")]
    pub subtree_levels: u64,
    ///An object describing the location of subtree files.
    pub subtrees: Subtrees,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
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
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A series of property names and the `expression` to evaluate for the value of that property.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(default, flatten)]
    pub additional_properties: std::collections::HashMap<String, Expression>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object containing a reference to a class from a metadata schema, and property values that conform to the properties of that class.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataEntity {
    ///The class that property values conform to. The value shall be a class ID declared in the `classes` dictionary of the metadata schema.
    pub class: String,
    ///A dictionary, where each key corresponds to a property ID in the class' `properties` dictionary and each value contains the property values. The type of the value shall match the property definition: For `BOOLEAN` use `true` or `false`. For `STRING` use a JSON string. For numeric types use a JSON number. For `ENUM` use a valid enum `name`, not an integer value. For `ARRAY`, `VECN`, and `MATN` types use a JSON array containing values matching the `componentType`. Required properties shall be included in this dictionary.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, AnyValue>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
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
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A 3D Tiles style with additional properties for Point Clouds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointCloudStyle {
    ///A `number expression` or `conditions` property which determines the size of the points in pixels.
    #[serde(rename = "pointSize", default = "default_point_cloud_style_point_size")]
    pub point_size: PointCloudStylePointSize,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_point_cloud_style_point_size() -> PointCloudStylePointSize {
    serde_json::from_str("1.0").expect("valid generated JSON default")
}
///A dictionary object of metadata about per-feature properties.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    ///The maximum value of this property of all the features in the tileset. The maximum value shall not be smaller than the minimum value.
    pub maximum: f64,
    ///The minimum value of this property of all the features in the tileset. The maximum value shall not be smaller than the minimum value.
    pub minimum: f64,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Statistics about property values.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyStatistics {
    ///The maximum property value occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the maximum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<NumericValue>,
    ///The arithmetic mean of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the mean of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean: Option<NumericValue>,
    ///The median of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the median of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median: Option<NumericValue>,
    ///The minimum property value occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the minimum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<NumericValue>,
    ///A dictionary, where each key corresponds to an enum `name` and each value is the number of occurrences of that enum. Only applicable when `type` is `ENUM`. For fixed-length arrays, this is an array of component-wise occurrences.
    #[serde(default)]
    pub occurrences: std::collections::HashMap<String, serde_json::Value>,
    ///The standard deviation of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the standard deviation of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(
        rename = "standardDeviation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub standard_deviation: Option<NumericValue>,
    ///The sum of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the sum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum: Option<NumericValue>,
    ///The variance of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the variance of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<NumericValue>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Properties conforming to a class, organized as property values stored in binary columnar arrays.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyTable {
    ///The class that property values conform to. The value shall be a class ID declared in the `classes` dictionary.
    pub class: String,
    ///The number of elements in each property array.
    pub count: u64,
    ///The name of the property table, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///A dictionary, where each key corresponds to a property ID in the class' `properties` dictionary and each value is an object describing where property values are stored. Required properties shall be included in this dictionary.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, PropertyTableProperty>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An array of binary property values. This represents one column of a property table, and contains one value of a certain property for each metadata entity.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyTableProperty {
    ///The type of values in `arrayOffsets`.
    #[serde(
        rename = "arrayOffsetType",
        default = "default_property_table_property_array_offset_type"
    )]
    pub array_offset_type: Option<serde_json::Value>,
    ///The index of the buffer view containing offsets for variable-length arrays. The number of offsets is equal to the property table `count` plus one. The offsets represent the start positions of each array, with the last offset representing the position after the last array. The array length is computed using the difference between the subsequent offset and the current offset. If `type` is `STRING` the offsets index into the string offsets array (stored in `stringOffsets`), otherwise they index into the property array (stored in `values`). The data type of these offsets is determined by `arrayOffsetType`. The buffer view `byteOffset` shall be aligned to a multiple of the `arrayOffsetType` size.
    #[serde(
        rename = "arrayOffsets",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub array_offsets: Option<u64>,
    ///Maximum value present in the property values. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the maximum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<NumericValue>,
    ///Minimum value present in the property values. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the minimum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<NumericValue>,
    ///An offset to apply to property values. Only applicable when the component type is `FLOAT32` or `FLOAT64`, or when the property is `normalized`. Overrides the class property's `offset` if both are defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<NumericValue>,
    ///A scale to apply to property values. Only applicable when the component type is `FLOAT32` or `FLOAT64`, or when the property is `normalized`. Overrides the class property's `scale` if both are defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<NumericValue>,
    ///The type of values in `stringOffsets`.
    #[serde(
        rename = "stringOffsetType",
        default = "default_property_table_property_string_offset_type"
    )]
    pub string_offset_type: Option<serde_json::Value>,
    ///The index of the buffer view containing offsets for strings. The number of offsets is equal to the number of string elements plus one. The offsets represent the byte offsets of each string in the property array (stored in `values`), with the last offset representing the byte offset after the last string. The string byte length is computed using the difference between the subsequent offset and the current offset. The data type of these offsets is determined by `stringOffsetType`. The buffer view `byteOffset` shall be aligned to a multiple of the `stringOffsetType` size.
    #[serde(
        rename = "stringOffsets",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub string_offsets: Option<u64>,
    ///The index of the buffer view containing property values. The data type of property values is determined by the property definition: When `type` is `BOOLEAN` values are packed into a bitstream. When `type` is `STRING` values are stored as byte sequences and decoded as UTF-8 strings. When `type` is `SCALAR`, `VECN`, or `MATN` the values are stored as the provided `componentType` and the buffer view `byteOffset` shall be aligned to a multiple of the `componentType` size. When `type` is `ENUM` values are stored as the enum's `valueType` and the buffer view `byteOffset` shall be aligned to a multiple of the `valueType` size. Each enum value in the array shall match one of the allowed values in the enum definition. `arrayOffsets` is required for variable-length arrays and `stringOffsets` is required for strings (for variable-length arrays of strings, both are required).
    pub values: u64,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_property_table_property_array_offset_type() -> Option<serde_json::Value> {
    Some(serde_json::Value::String("UINT32".to_owned()))
}
fn default_property_table_property_string_offset_type() -> Option<serde_json::Value> {
    Some(serde_json::Value::String("UINT32".to_owned()))
}
///An object defining classes and enums.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    ///A dictionary, where each key is a class ID and each value is an object defining the class. Class IDs shall be alphanumeric identifiers matching the regular expression `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    #[serde(default)]
    pub classes: std::collections::HashMap<String, Class>,
    ///The description of the schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///A dictionary, where each key is an enum ID and each value is an object defining the values for the enum. Enum IDs shall be alphanumeric identifiers matching the regular expression `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    #[serde(default)]
    pub enums: std::collections::HashMap<String, Enum>,
    ///Unique identifier for the schema. Schema IDs shall be alphanumeric identifiers matching the regular expression `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    pub id: String,
    ///The name of the schema, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///Application-specific version of the schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///Statistics about entities.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    ///A dictionary, where each key corresponds to a class ID in the `classes` dictionary of the metatata schema that was defined for the tileset that contains these statistics. Each value is an object containing statistics about entities that conform to the class.
    #[serde(default)]
    pub classes: std::collections::HashMap<String, ClassStatistics>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A 3D Tiles style.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    ///A `color expression` or `conditions` property which determines the color blended with the feature's intrinsic color.
    #[serde(default = "default_style_color")]
    pub color: StyleColor,
    ///A dictionary object of `expression` strings mapped to a variable name key that may be referenced throughout the style. If an expression references a defined variable, it is replaced with the evaluated result of the corresponding expression.
    #[serde(default)]
    pub defines: std::collections::HashMap<String, Expression>,
    ///A `meta` object which determines the values of non-visual properties of the feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    ///A `boolean expression` or `conditions` property which determines if a feature should be shown.
    #[serde(default = "default_style_show")]
    pub show: StyleShow,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_style_color() -> StyleColor {
    serde_json::from_str("\"color('#FFFFFF')\"").expect("valid generated JSON default")
}
fn default_style_show() -> StyleShow {
    serde_json::from_str("\"true\"").expect("valid generated JSON default")
}
///An object describing the availability of tiles and content in a subtree, as well as availability of children subtrees. May also store metadata for available tiles and content.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtree {
    ///An array of buffer views.
    #[serde(rename = "bufferViews", default)]
    pub buffer_views: Vec<BufferView>,
    ///An array of buffers.
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    ///The availability of children subtrees. The availability bitstream is a 1D boolean array where subtrees are ordered by their Morton index in the level of the tree immediately below the bottom row of the subtree. A child subtree's availability is determined by a single bit, 1 meaning a subtree exists at that spatial index, and 0 meaning it does not. The number of elements in the array is `N^subtreeLevels` where N is 4 for subdivision scheme `QUADTREE` and 8 for `OCTREE`. Availability may be stored in a buffer view or as a constant value that applies to all child subtrees. If availability is 0 for all child subtrees, then the tileset does not subdivide further.
    #[serde(rename = "childSubtreeAvailability")]
    pub child_subtree_availability: Availability,
    ///An array of content availability objects. If the tile has a single content this array will have one element; if the tile has multiple contents - as supported by 3DTILES_multiple_contents and 3D Tiles 1.1 - this array will have multiple elements.
    #[serde(rename = "contentAvailability", default)]
    pub content_availability: Vec<Availability>,
    ///An array of indexes to property tables containing content metadata. If the tile has a single content this array will have one element; if the tile has multiple contents - as supported by 3DTILES_multiple_contents and 3D Tiles 1.1 - this array will have multiple elements. Content metadata only exists for available contents and is tightly packed by increasing tile index. To access individual content metadata, implementations may create a mapping from tile indices to content metadata indices.
    #[serde(rename = "contentMetadata", default)]
    pub content_metadata: Vec<u64>,
    ///An array of property tables.
    #[serde(rename = "propertyTables", default)]
    pub property_tables: Vec<PropertyTable>,
    ///Subtree metadata encoded in JSON.
    #[serde(
        rename = "subtreeMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subtree_metadata: Option<MetadataEntity>,
    ///The availability of tiles in the subtree. The availability bitstream is a 1D boolean array where tiles are ordered by their level in the subtree and Morton index within that level. A tile's availability is determined by a single bit, 1 meaning a tile exists at that spatial index, and 0 meaning it does not. The number of elements in the array is `(N^subtreeLevels - 1)/(N - 1)` where N is 4 for subdivision scheme `QUADTREE` and 8 for `OCTREE`. Availability may be stored in a buffer view or as a constant value that applies to all tiles. If a non-root tile's availability is 1 its parent tile's availability shall also be 1. `tileAvailability.constant: 0` is disallowed, as subtrees shall have at least one tile.
    #[serde(rename = "tileAvailability")]
    pub tile_availability: Availability,
    ///Index of the property table containing tile metadata. Tile metadata only exists for available tiles and is tightly packed by increasing tile index. To access individual tile metadata, implementations may create a mapping from tile indices to tile metadata indices.
    #[serde(
        rename = "tileMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tile_metadata: Option<u64>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///An object describing the location of subtree files.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtrees {
    ///A template URI pointing to subtree files. A subtree is a fixed-depth (defined by `subtreeLevels`) portion of the tree to keep memory use bounded. The URI of each file is substituted with the subtree root's global level, x, and y. For subdivision scheme `OCTREE`, z shall also be given. Relative paths are relative to the tileset JSON.
    pub uri: String,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
///A URI with embedded expressions that describes the resource that is associated with an implicit tile in an implicit tileset. Allowed expressions are `{level}`, `{x}`, `{y}`, and `{z}`. `{level}` is substituted with the level of the node, `{x}` is substituted with the x index of the node within the level, and `{y}` is substituted with the y index of the node within the level. `{z}` may only be given when the subdivision scheme is `OCTREE`, and it is substituted with the z index of the node within the level.
pub type TemplateUri = String;
///A tile in a 3D Tiles tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    ///The bounding volume that encloses the tile.
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: BoundingVolume,
    ///An array of objects that define child tiles. Each child tile content is fully enclosed by its parent tile's bounding volume and, generally, has a geometricError less than its parent tile's geometricError. For leaf tiles, there are no children, and this property may not be defined.
    #[serde(default)]
    pub children: Vec<Tile>,
    ///Metadata about the tile's content and a link to the content. When this is omitted the tile is just used for culling. When this is defined, then `contents` shall be undefined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    ///An array of contents. When this is defined, then `content` shall be undefined.
    #[serde(default)]
    pub contents: Vec<Content>,
    ///The error, in meters, introduced if this tile is rendered and its children are not. At runtime, the geometric error is used to compute screen space error (SSE), i.e., the error measured in pixels.
    #[serde(rename = "geometricError")]
    pub geometric_error: f64,
    ///An object that describes the implicit subdivision of this tile.
    #[serde(
        rename = "implicitTiling",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub implicit_tiling: Option<ImplicitTiling>,
    ///A metadata entity that is associated with this tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///Specifies if additive or replacement refinement is used when traversing the tileset for rendering. This property is required for the root tile of a tileset; it is optional for all other tiles. The default is to inherit from the parent tile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refine: Option<Refine>,
    ///A floating-point 4x4 affine transformation matrix, stored in column-major order, that transforms the tile's content--i.e., its features as well as content.boundingVolume, boundingVolume, and viewerRequestVolume--from the tile's local coordinate system to the parent tile's coordinate system, or, in the case of a root tile, from the tile's local coordinate system to the tileset's coordinate system. `transform` does not apply to any volume property when the volume is a region, defined in EPSG:4979 coordinates. `transform` scales the `geometricError` by the maximum scaling factor from the matrix.
    #[serde(default = "default_tile_transform")]
    pub transform: [f64; 16],
    ///Optional bounding volume that defines the volume the viewer shall be inside of before the tile's content will be requested and before the tile will be refined based on geometricError.
    #[serde(
        rename = "viewerRequestVolume",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub viewer_request_volume: Option<BoundingVolume>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
fn default_tile_transform() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
///A 3D Tiles tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tileset {
    ///Metadata about the entire tileset.
    pub asset: Asset,
    ///Names of 3D Tiles extensions required to properly load this tileset. Each element of this array shall also be contained in `extensionsUsed`.
    #[serde(rename = "extensionsRequired", default)]
    pub extensions_required: Vec<String>,
    ///Names of 3D Tiles extensions used somewhere in this tileset.
    #[serde(rename = "extensionsUsed", default)]
    pub extensions_used: Vec<String>,
    ///The error, in meters, introduced if this tileset is not rendered. At runtime, the geometric error is used to compute screen space error (SSE), i.e., the error measured in pixels.
    #[serde(rename = "geometricError")]
    pub geometric_error: f64,
    ///An array of groups that tile content may belong to. Each element of this array is a metadata entity that describes the group. The tile content `group` property is an index into this array.
    #[serde(default)]
    pub groups: Vec<Group>,
    ///A metadata entity that is associated with this tileset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataEntity>,
    ///A dictionary object of metadata about per-feature properties.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, Properties>,
    ///The root tile.
    pub root: Tile,
    ///An object defining the structure of metadata classes and enums. When this is defined, then `schemaUri` shall be undefined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    ///The URI (or IRI) of the external schema file. When this is defined, then `schema` shall be undefined.
    #[serde(rename = "schemaUri", default, skip_serializing_if = "Option::is_none")]
    pub schema_uri: Option<String>,
    ///An object containing statistics about metadata entities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,
    ///Extension-specific data.
    #[serde(default)]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Asset {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Availability {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for BatchTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for BatchTableBinaryBodyReference {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Batched3dModelFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for BinaryBodyOffset {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for BoundingVolume {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Buffer {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for BufferView {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Class {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for ClassProperty {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for ClassStatistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Conditions {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Content {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Enum {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for EnumValue {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Extension {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for FeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for FeatureTableBinaryBodyReference {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Group {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for ImplicitTiling {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Instanced3dModelFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Meta {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for MetadataEntity {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for PointCloudFeatureTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
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
impl crate::HasExtensions for Properties {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for PropertyStatistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for PropertyTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for PropertyTableProperty {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Schema {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Statistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Style {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Subtree {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Subtrees {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Tile {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
impl crate::HasExtensions for Tileset {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
