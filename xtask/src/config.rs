use serde::Deserialize;
use std::collections::HashMap;

/// Generator configuration - equivalent to the Node.js `glTF.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Per-class overrides keyed by the schema `title`.
    #[serde(default)]
    pub classes: HashMap<String, ClassConfig>,

    /// Custom type definitions (enums, etc.) to generate get module level.
    #[serde(default, rename = "customTypes")]
    pub custom_types: HashMap<String, CustomTypeConfig>,

    /// Additional root schema files to process, relative to `--schema-dir`.
    /// These are seeded into the BFS queue alongside the main root schema.
    #[serde(default, rename = "additionalSchemas")]
    pub additional_schemas: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfig {
    /// Rename the schema title to this Rust type name.
    pub override_name: Option<String>,

    /// If true, this type will NOT be generated - it is either a base class
    /// whose properties are inlined, or a type alias (like `serde_json::Value`).
    #[serde(default)]
    pub skip: bool,

    /// The official extension name string.
    pub extension_name: Option<String>,

    /// Per-property type overrides: property name -> Rust type string.
    /// Example: { "componentType": "u32", "mode": "u32" }
    /// If the value starts with "Option<", the field is treated as optional
    /// (skip_serializing_if = "Option::is_none" is emitted automatically).
    #[serde(default)]
    pub property_overrides: HashMap<String, String>,

    /// Per-property JSON defaults used when the schema omits a semantic default.
    #[serde(default, rename = "propertyDefaults")]
    pub property_defaults: HashMap<String, serde_json::Value>,

    /// Per-property skip_serializing_if values: property name -> Rust comparison expression.
    /// Generates a `fn skip_if_{struct}_{field}(v: &Type) -> bool { *v == VALUE }` helper
    /// and wires it into the serde attribute.
    /// Example: { "translation": "[0.0_f64, 0.0, 0.0]" }
    #[serde(default, rename = "propertySkipIf")]
    pub property_skip_if: HashMap<String, String>,

    /// Extra fields to inject into the generated struct.
    /// These are emitted with `#[serde(skip)]` just before `extensions`/`extras`.
    #[serde(default)]
    pub extra_fields: Vec<ExtraFieldConfig>,

    /// Do not enforce schema minItems during serde decoding for this type.
    #[serde(default, rename = "allowEmptyArrays")]
    pub allow_empty_arrays: bool,
}

/// A single extra field to inject into a generated struct.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFieldConfig {
    /// Field name (Rust snake_case).
    pub name: String,
    /// Rust type expression, e.g. `"Vec<u8>"`.
    pub rust_type: String,
    /// Doc comment (single line).
    #[serde(default)]
    pub doc: Option<String>,
    /// If true (default), emit `#[serde(skip)]` so the field is excluded from
    /// JSON (de)serialization. Set to false for fields that exist in real JSON
    /// payloads but are absent from the JSON Schema.
    #[serde(default = "default_true")]
    pub skip_serde: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTypeConfig {
    /// Type kind: "enum" for now.
    pub kind: String,

    /// For enums: list of variant names.
    #[serde(default)]
    pub variants: Vec<String>,

    /// Doc comment for the type.
    #[serde(default)]
    pub doc: Option<String>,

    /// For numeric enums: deserialize from u32 indices instead of string names.
    #[serde(default)]
    pub numeric: bool,

    /// For numeric enums: maps variant names to actual numeric values.
    /// If not provided, uses 0-based indices.
    #[serde(default)]
    pub numeric_values: std::collections::HashMap<String, u32>,

    /// Default variant name (PascalCase). If not provided, defaults to the
    /// first variant in the `variants` list.
    #[serde(default)]
    pub default: Option<String>,

    /// Human-readable explanation of why this custom type exists (e.g. which
    /// schema field it overrides, or which extension requires it).
    /// This is emitted as a comment in the generated code and in MANIFEST.md.
    #[serde(default)]
    pub origin: Option<String>,
}
