#[allow(unused_imports)]
use super::*;
///A class containing a set of properties.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    ///The description of the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
    ///The name of the class, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<str>>,
    ///A dictionary, where each key is a property ID and each value is an object defining the property. Property IDs shall be alphanumeric identifiers matching the regular expression `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub properties: SortedMap<String, ClassProperty>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for Class {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
///A single property of a metadata class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassProperty {
    ///Whether the property is an array. When `count` is defined the property is a fixed-length array. Otherwise the property is a variable-length array.
    #[serde(default = "default_class_property_array")]
    pub array: bool,
    ///The datatype of the element's components. Required for `SCALAR`, `VECN`, and `MATN` types, and disallowed for other types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_type: Option<serde_json::Value>,
    ///The number of array elements. May only be defined when `array` is `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    ///A default value to use when encountering a `noData` value or an omitted property. The value is given in its final form, taking the effect of `normalized`, `offset`, and `scale` properties into account. Shall not be defined if `required` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<AnyValue>,
    ///The description of the property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
    ///Enum ID as declared in the `enums` dictionary. Required when `type` is `ENUM`. Disallowed when `type` is not `ENUM`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enum_type: Option<Box<str>>,
    ///Maximum allowed value for the property. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the maximum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<NumericValue>,
    ///Minimum allowed value for the property. Only applicable to `SCALAR`, `VECN`, and `MATN` types. This is the minimum of all property values, after the transforms based on the `normalized`, `offset`, and `scale` properties have been applied. Not applicable to variable-length arrays.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<NumericValue>,
    ///The name of the property, e.g. for display purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Box<str>>,
    ///A `noData` value represents missing data — also known as a sentinel value — wherever it appears. `BOOLEAN` properties may not specify `noData` values. This is given as the plain property value, without the transforms from the `normalized`, `offset`, and `scale` properties. Shall not be defined if `required` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    pub semantic: Option<Box<str>>,
    ///The element type.
    pub r#type: Option<serde_json::Value>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl Default for ClassProperty {
    fn default() -> Self {
        Self {
            array: default_class_property_array(),
            component_type: Default::default(),
            count: Default::default(),
            default: Default::default(),
            description: Default::default(),
            enum_type: Default::default(),
            max: Default::default(),
            min: Default::default(),
            name: Default::default(),
            no_data: Default::default(),
            normalized: default_class_property_normalized(),
            offset: Default::default(),
            required: default_class_property_required(),
            scale: Default::default(),
            semantic: Default::default(),
            r#type: Default::default(),
            extensions: Default::default(),
            extras: Default::default(),
        }
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
///Statistics about entities that conform to a class that was defined in a metadata schema.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStatistics {
    ///The number of entities that conform to the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    ///A dictionary, where each key corresponds to a property ID in the class' `properties` dictionary and each value is an object containing statistics about property values.
    #[serde(
        default,
        skip_serializing_if = "is_empty_collection",
        deserialize_with = "deserialize_null_default"
    )]
    pub properties: SortedMap<String, PropertyStatistics>,
    ///Extension-specific data.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
    ///Application-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}
impl crate::HasExtensions for ClassStatistics {
    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.extensions
    }
}
