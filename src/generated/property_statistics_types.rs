#[allow(unused_imports)]
use super::*;
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
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub occurrences: std::collections::HashMap<String, serde_json::Value>,
    ///The standard deviation of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the standard deviation of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard_deviation: Option<NumericValue>,
    ///The sum of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the sum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum: Option<NumericValue>,
    ///The variance of property values occurring in the tileset. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the variance of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<NumericValue>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for PropertyStatistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///Properties conforming to a class, organized as property values stored in binary columnar arrays.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyTable {
    ///The class that property values conform to. The value shall be a class ID declared in the `classes` dictionary.
    pub class: Box<str>,
    ///The number of elements in each property array.
    pub count: u64,
    ///The name of the property table, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<str>>,
    ///A dictionary, where each key corresponds to a property ID in the class' `properties` dictionary and each value is an object describing where property values are stored. Required properties shall be included in this dictionary.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub properties: SortedMap<String, PropertyTableProperty>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for PropertyTable {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///An array of binary property values. This represents one column of a property table, and contains one value of a certain property for each metadata entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyTableProperty {
    ///The type of values in `arrayOffsets`.
    #[serde(default = "default_property_table_property_array_offset_type")]
    pub array_offset_type: Option<serde_json::Value>,
    ///The index of the buffer view containing offsets for variable-length arrays. The number of offsets is equal to the property table `count` plus one. The offsets represent the start positions of each array, with the last offset representing the position after the last array. The array length is computed using the difference between the subsequent offset and the current offset. If `type` is `STRING` the offsets index into the string offsets array (stored in `stringOffsets`), otherwise they index into the property array (stored in `values`). The data type of these offsets is determined by `arrayOffsetType`. The buffer view `byteOffset` shall be aligned to a multiple of the `arrayOffsetType` size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default = "default_property_table_property_string_offset_type")]
    pub string_offset_type: Option<serde_json::Value>,
    ///The index of the buffer view containing offsets for strings. The number of offsets is equal to the number of string elements plus one. The offsets represent the byte offsets of each string in the property array (stored in `values`), with the last offset representing the byte offset after the last string. The string byte length is computed using the difference between the subsequent offset and the current offset. The data type of these offsets is determined by `stringOffsetType`. The buffer view `byteOffset` shall be aligned to a multiple of the `stringOffsetType` size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string_offsets: Option<u64>,
    ///The index of the buffer view containing property values. The data type of property values is determined by the property definition: When `type` is `BOOLEAN` values are packed into a bitstream. When `type` is `STRING` values are stored as byte sequences and decoded as UTF-8 strings. When `type` is `SCALAR`, `VECN`, or `MATN` the values are stored as the provided `componentType` and the buffer view `byteOffset` shall be aligned to a multiple of the `componentType` size. When `type` is `ENUM` values are stored as the enum's `valueType` and the buffer view `byteOffset` shall be aligned to a multiple of the `valueType` size. Each enum value in the array shall match one of the allowed values in the enum definition. `arrayOffsets` is required for variable-length arrays and `stringOffsets` is required for strings (for variable-length arrays of strings, both are required).
    pub values: u64,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl Default for PropertyTableProperty {
    fn default() -> Self {
        Self {
            array_offset_type: default_property_table_property_array_offset_type(),
            array_offsets: Default::default(),
            max: Default::default(),
            min: Default::default(),
            offset: Default::default(),
            scale: Default::default(),
            string_offset_type: default_property_table_property_string_offset_type(),
            string_offsets: Default::default(),
            values: Default::default(),
            extensions: Default::default(),
            extras: Default::default(),
        }
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
