//!Generated 3D Tiles data model. Do not edit.
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::approx_constant)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::empty_docs)]
use serde::{Deserialize, Serialize};
/// Whether a collection-typed field should be omitted from output.
///
/// A schema property that is absent deserializes to an empty collection, so
/// writing it back out as `[]` or `{}` would not round-trip.
fn is_empty_collection<T: EmptyCollection>(value: &T) -> bool {
    value.is_empty_collection()
}
/// Implemented by the collection types that generated fields use.
trait EmptyCollection {
    /// Whether this collection has no entries.
    fn is_empty_collection(&self) -> bool;
}
impl<T> EmptyCollection for Vec<T> {
    fn is_empty_collection(&self) -> bool {
        self.is_empty()
    }
}
impl<T> EmptyCollection for Box<[T]> {
    fn is_empty_collection(&self) -> bool {
        self.is_empty()
    }
}
impl<K, V> EmptyCollection for std::collections::HashMap<K, V> {
    fn is_empty_collection(&self) -> bool {
        self.is_empty()
    }
}
/// Deserializes a collection, treating `null` as absent.
///
/// A schema that declares an array or object and does not require it is
/// routinely satisfied by producers writing an explicit `null` rather than
/// omitting the key. `#[serde(default)]` only covers the omitted case, so
/// without this a conforming document is rejected.
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
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
fn default_enum_value_type() -> Option<serde_json::Value> {
    Some(serde_json::from_str("\"UINT16\"").expect("a schema default must match its type"))
}
fn default_point_cloud_style_point_size() -> PointCloudStylePointSize {
    serde_json::from_str("1.0").expect("a schema default must match its type")
}
fn default_property_table_property_array_offset_type() -> Option<serde_json::Value> {
    Some(serde_json::from_str("\"UINT32\"").expect("a schema default must match its type"))
}
fn default_property_table_property_string_offset_type() -> Option<serde_json::Value> {
    Some(serde_json::from_str("\"UINT32\"").expect("a schema default must match its type"))
}
fn default_style_color() -> StyleColor {
    serde_json::from_str("\"color('#FFFFFF')\"").expect("a schema default must match its type")
}
fn default_style_show() -> StyleShow {
    serde_json::from_str("\"true\"").expect("a schema default must match its type")
}
fn default_tile_transform() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
#[path = "generated/sorted_map_types.rs"]
mod sorted_map_types;
#[allow(unused_imports)]
pub use sorted_map_types::*;
#[path = "generated/any_value_types.rs"]
mod any_value_types;
#[allow(unused_imports)]
pub use any_value_types::*;
#[path = "generated/no_data_value_types.rs"]
mod no_data_value_types;
#[allow(unused_imports)]
pub use no_data_value_types::*;
#[path = "generated/numeric_value_types.rs"]
mod numeric_value_types;
#[allow(unused_imports)]
pub use numeric_value_types::*;
#[path = "generated/refine_types.rs"]
mod refine_types;
#[allow(unused_imports)]
pub use refine_types::*;
#[path = "generated/subdivision_scheme_types.rs"]
mod subdivision_scheme_types;
#[allow(unused_imports)]
pub use subdivision_scheme_types::*;
#[path = "generated/value_types.rs"]
mod value_types;
#[allow(unused_imports)]
pub use value_types::*;
#[path = "generated/batch_table_property_types.rs"]
mod batch_table_property_types;
#[allow(unused_imports)]
pub use batch_table_property_types::*;
#[path = "generated/boolean_expression_types.rs"]
mod boolean_expression_types;
#[allow(unused_imports)]
pub use boolean_expression_types::*;
#[path = "generated/feature_table_property_types.rs"]
mod feature_table_property_types;
#[allow(unused_imports)]
pub use feature_table_property_types::*;
#[path = "generated/global_property_cartesian3_types.rs"]
mod global_property_cartesian3_types;
#[allow(unused_imports)]
pub use global_property_cartesian3_types::*;
#[path = "generated/number_expression_types.rs"]
mod number_expression_types;
#[allow(unused_imports)]
pub use number_expression_types::*;
#[path = "generated/point_cloud_style_point_size_types.rs"]
mod point_cloud_style_point_size_types;
#[allow(unused_imports)]
pub use point_cloud_style_point_size_types::*;
#[path = "generated/style_color_types.rs"]
mod style_color_types;
#[allow(unused_imports)]
pub use style_color_types::*;
#[path = "generated/asset_types.rs"]
mod asset_types;
#[allow(unused_imports)]
pub use asset_types::*;
#[path = "generated/availability_types.rs"]
mod availability_types;
#[allow(unused_imports)]
pub use availability_types::*;
#[path = "generated/batch_table_types.rs"]
mod batch_table_types;
#[allow(unused_imports)]
pub use batch_table_types::*;
#[path = "generated/batched3d_model_feature_table_types.rs"]
mod batched3d_model_feature_table_types;
#[allow(unused_imports)]
pub use batched3d_model_feature_table_types::*;
#[path = "generated/binary_body_offset_types.rs"]
mod binary_body_offset_types;
#[allow(unused_imports)]
pub use binary_body_offset_types::*;
#[path = "generated/bounding_volume_types.rs"]
mod bounding_volume_types;
#[allow(unused_imports)]
pub use bounding_volume_types::*;
#[path = "generated/buffer_types.rs"]
mod buffer_types;
#[allow(unused_imports)]
pub use buffer_types::*;
#[path = "generated/class_types.rs"]
mod class_types;
#[allow(unused_imports)]
pub use class_types::*;
#[path = "generated/color_expression_types.rs"]
mod color_expression_types;
#[allow(unused_imports)]
pub use color_expression_types::*;
#[path = "generated/condition_types.rs"]
mod condition_types;
#[allow(unused_imports)]
pub use condition_types::*;
#[path = "generated/conditions_types.rs"]
mod conditions_types;
#[allow(unused_imports)]
pub use conditions_types::*;
#[path = "generated/content_types.rs"]
mod content_types;
#[allow(unused_imports)]
pub use content_types::*;
#[path = "generated/definitions_types.rs"]
mod definitions_types;
#[allow(unused_imports)]
pub use definitions_types::*;
#[path = "generated/enum_types.rs"]
mod enum_types;
#[allow(unused_imports)]
pub use enum_types::*;
#[path = "generated/expression_types.rs"]
mod expression_types;
#[allow(unused_imports)]
pub use expression_types::*;
#[path = "generated/extension_types.rs"]
mod extension_types;
#[allow(unused_imports)]
pub use extension_types::*;
#[path = "generated/extras_types.rs"]
mod extras_types;
#[allow(unused_imports)]
pub use extras_types::*;
#[path = "generated/feature_table_types.rs"]
mod feature_table_types;
#[allow(unused_imports)]
pub use feature_table_types::*;
#[path = "generated/global_property_boolean_types.rs"]
mod global_property_boolean_types;
#[allow(unused_imports)]
pub use global_property_boolean_types::*;
#[path = "generated/group_types.rs"]
mod group_types;
#[allow(unused_imports)]
pub use group_types::*;
#[path = "generated/implicit_tiling_types.rs"]
mod implicit_tiling_types;
#[allow(unused_imports)]
pub use implicit_tiling_types::*;
#[path = "generated/instanced3d_model_feature_table_types.rs"]
mod instanced3d_model_feature_table_types;
#[allow(unused_imports)]
pub use instanced3d_model_feature_table_types::*;
#[path = "generated/meta_types.rs"]
mod meta_types;
#[allow(unused_imports)]
pub use meta_types::*;
#[path = "generated/metadata_entity_types.rs"]
mod metadata_entity_types;
#[allow(unused_imports)]
pub use metadata_entity_types::*;
#[path = "generated/point_cloud_feature_table_types.rs"]
mod point_cloud_feature_table_types;
#[allow(unused_imports)]
pub use point_cloud_feature_table_types::*;
#[path = "generated/properties_types.rs"]
mod properties_types;
#[allow(unused_imports)]
pub use properties_types::*;
#[path = "generated/property_statistics_types.rs"]
mod property_statistics_types;
#[allow(unused_imports)]
pub use property_statistics_types::*;
#[path = "generated/schema_types.rs"]
mod schema_types;
#[allow(unused_imports)]
pub use schema_types::*;
#[path = "generated/statistics_types.rs"]
mod statistics_types;
#[allow(unused_imports)]
pub use statistics_types::*;
#[path = "generated/style_types.rs"]
mod style_types;
#[allow(unused_imports)]
pub use style_types::*;
#[path = "generated/subtree_types.rs"]
mod subtree_types;
#[allow(unused_imports)]
pub use subtree_types::*;
#[path = "generated/subtrees_types.rs"]
mod subtrees_types;
#[allow(unused_imports)]
pub use subtrees_types::*;
#[path = "generated/template_uri_types.rs"]
mod template_uri_types;
#[allow(unused_imports)]
pub use template_uri_types::*;
#[path = "generated/tile_types.rs"]
mod tile_types;
#[allow(unused_imports)]
pub use tile_types::*;
#[path = "generated/tileset_types.rs"]
mod tileset_types;
#[allow(unused_imports)]
pub use tileset_types::*;
