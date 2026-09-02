use proc_macro2::TokenStream;
use quote::quote;
use schemagen::ir::SchemaNode;
use schemagen::types::{ExtraFieldDef, parse_type};
use schemagen::{GenerationPolicy, RustType, StructDef};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(default)]
    pub extensible: bool,
    #[serde(rename = "extensionTrait")]
    pub extension_trait: Option<String>,
    #[serde(rename = "refTypes", default)]
    pub ref_types: HashMap<String, String>,
    #[serde(default)]
    pub classes: HashMap<String, PolicyClass>,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PolicyClass {
    #[serde(default)]
    pub extra_fields: Vec<ExtraField>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraField {
    pub name: String,
    pub rust_type: String,
    #[serde(default = "default_true")]
    pub skip_serde: bool,
}
fn default_true() -> bool {
    true
}

pub struct TairuPolicy {
    pub config: PolicyConfig,
}
impl GenerationPolicy for TairuPolicy {
    fn skip_field(&self, _owner: &SchemaNode, field: &str, _schema: &SchemaNode) -> bool {
        self.config.extensible && matches!(field, "extensions" | "extras")
    }

    fn field_type(
        &self,
        owner: &SchemaNode,
        field: &str,
        _schema: &SchemaNode,
    ) -> Option<RustType> {
        if owner.metadata.title.as_deref() == Some("Bounding Volume") {
            return match field {
                "box" => Some(RustType::Option(Box::new(RustType::Array(
                    Box::new(RustType::F64),
                    12,
                )))),
                "region" => Some(RustType::Option(Box::new(RustType::Array(
                    Box::new(RustType::F64),
                    6,
                )))),
                "sphere" => Some(RustType::Option(Box::new(RustType::Array(
                    Box::new(RustType::F64),
                    4,
                )))),
                _ => None,
            };
        }
        match field {
            "bitstream" => return Some(RustType::Option(Box::new(RustType::Usize))),
            "constant" => return Some(RustType::Option(Box::new(RustType::U64))),
            _ => {}
        }
        match (owner.metadata.title.as_deref(), field) {
            (Some("Enum"), "valueType")
            | (Some("Property Table Property"), "arrayOffsetType")
            | (Some("Property Table Property"), "stringOffsetType")
            | (Some("Class Property"), "componentType")
            | (Some("Class Property"), "type") => {
                return Some(RustType::Option(Box::new(RustType::Value)));
            }
            (Some("Property Statistics"), "occurrences") => {
                return Some(RustType::Map(
                    Box::new(RustType::String),
                    Box::new(RustType::Value),
                ));
            }
            (Some("Buffer View"), "buffer") => return Some(RustType::Usize),
            (Some("Buffer View"), "byteOffset") => return Some(RustType::Usize),
            (Some("Buffer View"), "byteLength") => return Some(RustType::Usize),
            (Some("Buffer"), "byteLength") => return Some(RustType::Usize),
            (Some("Subtrees"), "uri") => return Some(RustType::String),
            _ => {}
        }
        None
    }
    fn reference_type(&self, title: &str, _schema: &SchemaNode) -> Option<RustType> {
        self.config
            .ref_types
            .get(title)
            .and_then(|value| parse_type(value))
    }
    fn augment_struct(&self, definition: &mut StructDef) {
        if self.config.extensible {
            definition.extensions = true;
        }
        if let Some(class) = self.config.classes.get(&definition.title) {
            definition
                .extra_fields
                .extend(class.extra_fields.iter().filter_map(|field| {
                    Some(ExtraFieldDef {
                        name: field.name.clone(),
                        rust_type: field.rust_type.clone(),
                        skip_serde: field.skip_serde,
                        description: None,
                    })
                }));
        }
    }

    fn additional_definitions(&self) -> Vec<TokenStream> {
        vec![
            quote!(
                #[doc = "An arbitrary JSON value described by the 3D Tiles metadata specification."]
                pub type AnyValue = serde_json::Value;
            ),
            quote!(
                #[doc = "A numeric value described by the 3D Tiles metadata specification."]
                pub type NumericValue = serde_json::Value;
            ),
            quote!(
                #[doc = "A no-data value described by the 3D Tiles metadata specification."]
                pub type NoDataValue = serde_json::Value;
            ),
            quote!(
                #[doc = "An arbitrary JSON value."]
                pub type Value = serde_json::Value;
            ),
        ]
    }
}
