use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::config::{Config, CustomTypeConfig};
use crate::schema::{JsonSchema, SchemaCache};

/// A resolved Rust property (field) for a generated struct.
#[derive(Debug)]
pub struct RustProperty {
    /// The original JSON property name (e.g. "meshes").
    pub json_name: String,
    /// The Rust field name in snake_case (e.g. "meshes").
    pub rust_name: String,
    /// The Rust type expression (e.g. "Vec<Mesh>").
    pub rust_type: String,
    /// The serde default expression, if any.
    pub default_expr: Option<String>,
    /// Documentation string.
    pub doc: Option<String>,
    /// Whether this needs `#[serde(rename = "...")]`.
    pub needs_rename: bool,
    /// Whether to include `skip_serializing_if`.
    pub skip_serializing_if: Option<String>,
    /// If Some, a `fn skip_if_...(v: &Type) -> bool { *v == VALUE }` function
    /// will be generated. The value here is the Rust comparison expression.
    pub skip_if_value: Option<String>,
    /// Whether this property is required in the source schema.
    pub is_required: bool,
    /// Whether this field must deserialize as a non-empty array.
    pub enforce_non_empty: bool,
    /// Additional types discovered (schemas to also generate).
    pub discovered_schemas: Vec<DiscoveredSchema>,
}

#[derive(Debug)]
pub struct DiscoveredSchema {
    pub title: String,
    pub schema: JsonSchema,
    pub source_path: Option<std::path::PathBuf>,
}

/// A complete generated Rust struct definition.
#[derive(Debug)]
pub struct GeneratedStruct {
    /// The Rust struct name.
    pub name: String,
    /// The original schema title.
    pub title: String,
    /// Doc comment.
    pub doc: Option<String>,
    /// Extension name constant, if applicable.
    pub extension_name: Option<String>,
    /// The glTF object this extension attaches to (e.g. "Material", "Gltf").
    /// None if not an extension or if the attachment target couldn't be determined.
    pub attach_target: Option<String>,
    /// Fields (including inherited ones, fully flattened).
    pub fields: Vec<RustProperty>,
    /// Extra fields injected via config (emitted with `#[serde(skip)]`).
    pub extra_fields: Vec<crate::config::ExtraFieldConfig>,
}

/// Resolve the Rust type name for a schema, applying config overrides.
pub fn rust_type_name(config: &Config, title: &str) -> String {
    if let Some(class_config) = config.classes.get(title) {
        if let Some(ref override_name) = class_config.override_name {
            return override_name.clone();
        }
    }

    let mut candidate = title.to_upper_camel_case();
    if candidate.is_empty() {
        candidate = "GeneratedType".to_string();
    }

    // Avoid invalid/colliding generated identifiers like `u32` from schema titles.
    if !candidate
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
        || is_builtin_type_name(&candidate)
    {
        candidate = format!("Schema{}", candidate.to_upper_camel_case());
    }

    candidate
}

fn is_builtin_type_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "bool"
            | "str"
            | "string"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}

fn minimum_is_non_negative(value: Option<&serde_json::Value>) -> bool {
    value.is_some_and(|minimum| {
        minimum.as_u64().is_some()
            || minimum.as_i64().is_some_and(|number| number >= 0)
            || minimum
                .as_f64()
                .is_some_and(|number| number >= 0.0 && number.fract() == 0.0)
    })
}

/// Map a single JSON Schema property to a Rust field.
pub fn resolve_property(
    cache: &mut SchemaCache,
    config: &Config,
    discovered_custom_types: &mut HashMap<String, CustomTypeConfig>,
    parent_schema: &JsonSchema,
    parent_name: &str,
    property_name: &str,
    property_json: &serde_json::Value,
    required: &[String],
) -> Option<RustProperty> {
    let details: JsonSchema = serde_json::from_value(property_json.clone()).ok()?;

    let is_required = required.contains(&property_name.to_string());
    let has_default = details.default.is_some();
    let make_optional = !is_required && !has_default;

    let snake_name = property_name.to_snake_case();
    let rust_name = make_safe_identifier(&snake_name);
    let needs_rename = rust_name != property_name;

    let doc = details
        .description
        .clone()
        .or_else(|| details.detailed_description.clone());
    let enum_type_hint = Some(format!(
        "{}{}",
        parent_name,
        property_name.to_upper_camel_case()
    ));
    let map_value_hint = Some(format!(
        "{}{}Value",
        parent_name,
        property_name.to_upper_camel_case()
    ));

    // Use the schema title (not the Rust name) to look up config.
    let config_key = parent_schema.title.as_deref().unwrap_or(parent_name);
    let class_config = config.classes.get(config_key);

    // Step 1: Resolve type, default, and base skip_if (via override or schema).
    let (rust_type, mut default_expr, skip_if, discovered) = if let Some(override_type) =
        class_config.and_then(|c| c.property_overrides.get(property_name))
    {
        // If the override type already wraps in Option<>, treat it as optional.
        let is_option_override = override_type.starts_with("Option<");
        let rust_type = if make_optional && !is_option_override {
            format!("Option<{}>", override_type)
        } else {
            override_type.clone()
        };
        let skip_if = if make_optional || is_option_override {
            Some("Option::is_none".into())
        } else {
            None
        };
        // Option types default to None; non-option overrides keep schema default.
        let default_expr = if is_option_override || make_optional {
            None
        } else {
            details
                .default
                .as_ref()
                .map(|d| format_default(&rust_type, d))
        };
        (rust_type, default_expr, skip_if, vec![])
    } else {
        // Normal resolution from schema.
        resolve_type(
            cache,
            config,
            discovered_custom_types,
            &details,
            enum_type_hint.as_deref(),
            map_value_hint.as_deref(),
            make_optional,
            parent_schema,
        )?
    };

    if let Some(default_value) =
        class_config.and_then(|class| class.property_defaults.get(property_name))
    {
        default_expr = Some(format_array_default(&rust_type, default_value));
    }

    // Fallback for enum-like/custom-type fields: if schema provides a default and
    // the resolved field is a non-optional custom type, emit serde default even
    // when the type resolver did not attach a concrete default expression.
    if default_expr.is_none() && details.default.is_some() {
        let is_optional_type = rust_type.starts_with("Option<");
        if !is_optional_type {
            let base_type = rust_type
                .strip_prefix("Vec<")
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(&rust_type);

            if config.custom_types.contains_key(base_type)
                || discovered_custom_types.contains_key(base_type)
                || matches!(base_type, "FilterMode" | "WrapMode" | "BufferViewTarget")
            {
                default_expr = Some("Default::default()".to_string());
            }
        }
    }

    // Step 2: Apply property_skip_if override (generates a value-comparison skip function).
    let (final_skip_if, skip_if_value) =
        if let Some(skip_val) = class_config.and_then(|c| c.property_skip_if.get(property_name)) {
            let fn_name = format!(
                "skip_if_{}_{}",
                parent_name.to_snake_case(),
                sanitize_identifier_fragment(&rust_name)
            );
            (Some(fn_name), Some(skip_val.clone()))
        } else {
            (skip_if, None)
        };

    let enforce_non_empty = !class_config.is_some_and(|class| class.allow_empty_arrays)
        && details.schema_type.as_deref() == Some("array")
        && details.min_items.unwrap_or(0) >= 1
        && rust_type.starts_with("Vec<");

    Some(RustProperty {
        json_name: property_name.to_string(),
        rust_name,
        rust_type,
        default_expr,
        doc,
        needs_rename,
        skip_serializing_if: final_skip_if,
        skip_if_value,
        is_required,
        enforce_non_empty,
        discovered_schemas: discovered,
    })
}

fn resolve_type(
    cache: &mut SchemaCache,
    config: &Config,
    discovered_custom_types: &mut HashMap<String, CustomTypeConfig>,
    details: &JsonSchema,
    type_hint: Option<&str>,
    map_value_hint: Option<&str>,
    make_optional: bool,
    parent_schema: &JsonSchema,
) -> Option<(
    String,
    Option<String>,
    Option<String>,
    Vec<DiscoveredSchema>,
)> {
    let mut discovered = Vec::new();

    // Enum-shaped schemas (`enum`, `const`, or `anyOf` finite literals) should
    // resolve to their schema primitive, not generated enum custom types.
    if details.schema_type.is_none() {
        if let Some(primitive_kind) = infer_enum_primitive_schema_type(details) {
            let mut primitive_details = details.clone();
            primitive_details.schema_type = Some(primitive_kind.to_string());
            return resolve_type(
                cache,
                config,
                discovered_custom_types,
                &primitive_details,
                type_hint,
                map_value_hint,
                make_optional,
                parent_schema,
            );
        }
    }

    // Primitive types.
    match details.schema_type.as_deref() {
        Some("integer") => {
            let base_ty = if minimum_is_non_negative(details.minimum.as_ref()) {
                "usize"
            } else {
                "i64"
            };
            let ty = if make_optional {
                format!("Option<{base_ty}>")
            } else {
                base_ty.to_string()
            };
            let default_expr = if !make_optional {
                details.default.as_ref().map(|d| format_default(base_ty, d))
            } else {
                None
            };
            let skip_if = if make_optional {
                Some("Option::is_none".into())
            } else {
                None
            };
            return Some((ty, default_expr, skip_if, discovered));
        }
        Some("number") => {
            let ty = if make_optional { "Option<f64>" } else { "f64" };
            let default_expr = if !make_optional {
                details.default.as_ref().map(|d| format_default("f64", d))
            } else {
                None
            };
            let skip_if = if make_optional {
                Some("Option::is_none".into())
            } else {
                None
            };
            return Some((ty.to_string(), default_expr, skip_if, discovered));
        }
        Some("boolean") => {
            let ty = if make_optional {
                "Option<bool>"
            } else {
                "bool"
            };
            let default_expr = if !make_optional {
                details.default.as_ref().map(|d| format_default("bool", d))
            } else {
                None
            };
            let skip_if = if make_optional {
                Some("Option::is_none".into())
            } else {
                None
            };
            return Some((ty.to_string(), default_expr, skip_if, discovered));
        }
        Some("string") => {
            let ty = if make_optional {
                "Option<String>"
            } else {
                "String"
            };
            let default_expr = if !make_optional {
                details
                    .default
                    .as_ref()
                    .map(|d| format_default("String", d))
            } else {
                None
            };
            let skip_if = if make_optional {
                Some("Option::is_none".into())
            } else {
                None
            };
            return Some((ty.to_string(), default_expr, skip_if, discovered));
        }
        Some("array") => {
            // Resolve item type.
            if let Some(items) = &details.items {
                let item_schema: JsonSchema = serde_json::from_value(items.as_ref().clone())
                    .unwrap_or(JsonSchema {
                        title: None,
                        description: None,
                        schema_type: Some("object".into()),
                        properties: HashMap::new(),
                        required: Vec::new(),
                        reference: None,
                        all_of: Vec::new(),
                        additional_properties: None,
                        items: None,
                        enum_values: None,
                        any_of: None,
                        const_value: None,
                        detailed_description: None,
                        default: None,
                        minimum: None,
                        maximum: None,
                        min_items: None,
                        max_items: None,
                    });

                let (inner_type, _, _, inner_discovered) = resolve_type(
                    cache,
                    config,
                    discovered_custom_types,
                    &item_schema,
                    None,
                    map_value_hint,
                    false,
                    parent_schema,
                )?;
                discovered.extend(inner_discovered);

                // Check if array length is constrained to a fixed size (minItems == maxItems)
                if let (Some(min), Some(max)) = (details.min_items, details.max_items) {
                    if min == max && min > 0 && (min as usize) <= 16 {
                        let ty = format!("[{}; {}]", inner_type, min);
                        // Fixed arrays with schema defaults: format as proper array literal
                        let default_expr = details
                            .default
                            .as_ref()
                            .map(|d| format_array_default(&ty, d));
                        return Some((ty, default_expr, None, discovered));
                    }
                }

                let ty = format!("Vec<{inner_type}>");
                let skip_if = Some("Vec::is_empty".into());
                let default_expr = details
                    .default
                    .as_ref()
                    .map(|d| format_array_default(&ty, d));
                return Some((ty, default_expr, skip_if, discovered));
            }
            return Some((
                "Vec<serde_json::Value>".to_string(),
                None,
                Some("Vec::is_empty".into()),
                discovered,
            ));
        }
        Some("object") if details.additional_properties.is_some() => {
            // Dictionary / map type.
            if let Some(ap) = &details.additional_properties {
                let ap_schema: JsonSchema =
                    serde_json::from_value(ap.clone()).unwrap_or(JsonSchema {
                        title: None,
                        description: None,
                        schema_type: Some("string".into()),
                        properties: HashMap::new(),
                        required: Vec::new(),
                        reference: None,
                        all_of: Vec::new(),
                        additional_properties: None,
                        items: None,
                        enum_values: None,
                        any_of: None,
                        const_value: None,
                        detailed_description: None,
                        default: None,
                        minimum: None,
                        maximum: None,
                        min_items: None,
                        max_items: None,
                    });
                let (val_type, _, _, inner_discovered) = resolve_type(
                    cache,
                    config,
                    discovered_custom_types,
                    &ap_schema,
                    map_value_hint,
                    None,
                    false,
                    parent_schema,
                )?;
                discovered.extend(inner_discovered);

                let key_type = infer_map_key_type(details, &ap_schema);
                let ty = format!("std::collections::HashMap<{key_type}, {val_type}>");
                let skip_if = Some("std::collections::HashMap::is_empty".into());
                return Some((ty, None, skip_if, discovered));
            }
        }
        _ => {}
    }

    // $ref - reference to another schema.
    if let Some(ref reference) = details.reference {
        if let Some((ref_schema, ref_path)) = cache.load_with_path(reference) {
            let title = ref_schema.title.as_deref().unwrap_or("Unknown");

            // glTF Id -> Option<i32> (absent means "not set").
            if title == "glTF Id" {
                if make_optional {
                    return Some((
                        "Option<usize>".to_string(),
                        None,
                        Some("Option::is_none".into()),
                        discovered,
                    ));
                } else {
                    // Required glTF ids are non-negative indices into root arrays.
                    return Some(("usize".to_string(), None, None, discovered));
                }
            }

            // If the referenced schema is a primitive, resolve it directly.
            if ref_schema
                .schema_type
                .as_deref()
                .is_some_and(|t| t != "object")
            {
                return resolve_type(
                    cache,
                    config,
                    discovered_custom_types,
                    &ref_schema,
                    type_hint,
                    map_value_hint,
                    make_optional,
                    parent_schema,
                );
            }

            let type_name = rust_type_name(config, title);

            // Skipped types (base classes whose properties are inlined).
            if config.classes.get(title).is_some_and(|c| c.skip) {
                // Don't emit as a field - properties are inlined by generate_struct.
                return None;
            }

            discovered.push(DiscoveredSchema {
                title: title.to_string(),
                schema: ref_schema.clone(),
                source_path: ref_path,
            });

            let ty = if make_optional {
                format!("Option<{type_name}>")
            } else {
                type_name
            };
            let skip_if = if make_optional {
                Some("Option::is_none".into())
            } else {
                None
            };
            return Some((ty, None, skip_if, discovered));
        }
    }

    // allOf with single entry - unwrap.
    if details.all_of.len() == 1 {
        if let Ok(inner) = serde_json::from_value::<JsonSchema>(details.all_of[0].clone()) {
            return resolve_type(
                cache,
                config,
                discovered_custom_types,
                &inner,
                type_hint,
                map_value_hint,
                make_optional,
                parent_schema,
            );
        }
    }

    // Fallback: serde_json::Value.
    if make_optional {
        Some((
            "Option<serde_json::Value>".to_string(),
            None,
            Some("Option::is_none".into()),
            discovered,
        ))
    } else {
        Some(("serde_json::Value".to_string(), None, None, discovered))
    }
}

fn format_default(rust_type: &str, value: &serde_json::Value) -> String {
    match rust_type {
        "i64" | "i32" | "u64" | "u32" | "usize" => {
            if let Some(n) = value.as_i64() {
                return n.to_string();
            }
            if let Some(n) = value.as_u64() {
                return n.to_string();
            }
            if let Some(n) = value.as_f64() {
                return format!("{}", n as i64);
            }
            "0".to_string()
        }
        "f64" => {
            if let Some(n) = value.as_f64() {
                // Ensure it has a decimal point.
                let s = format!("{n}");
                if s.contains('.') { s } else { format!("{s}.0") }
            } else {
                "0.0".to_string()
            }
        }
        "bool" => value.as_bool().unwrap_or(false).to_string(),
        "String" => {
            if let Some(s) = value.as_str() {
                format!("\"{s}\".to_string()")
            } else {
                "String::new()".to_string()
            }
        }
        _ => "Default::default()".to_string(),
    }
}

/// Format a default value for array types (both fixed-size and Vec).
fn format_array_default(rust_type: &str, value: &serde_json::Value) -> String {
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr
            .iter()
            .map(|v| {
                if let Some(n) = v.as_f64() {
                    let s = format!("{n}");
                    if s.contains('.') { s } else { format!("{s}.0") }
                } else if let Some(n) = v.as_i64() {
                    n.to_string()
                } else if let Some(b) = v.as_bool() {
                    b.to_string()
                } else if let Some(s) = v.as_str() {
                    format!("\"{s}\".to_string()")
                } else {
                    "Default::default()".to_string()
                }
            })
            .collect();

        // Determine if this is a fixed array or Vec
        if rust_type.contains('[') && rust_type.contains(']') {
            // Fixed array like [f64; 16]
            format!("[{}]", items.join(", "))
        } else {
            // Vec type
            format!("vec![{}]", items.join(", "))
        }
    } else {
        "Default::default()".to_string()
    }
}

/// Returns true if the given default expression is the same as `Default::default()`
/// for the given Rust type, meaning we can use plain `#[serde(default)]`.
fn is_type_default(default_expr: &str, rust_type: &str) -> bool {
    match rust_type {
        "i64" | "i32" | "u64" | "u32" | "usize" => default_expr == "0",
        "f64" => default_expr == "0" || default_expr == "0.0",
        "bool" => default_expr == "false",
        "String" => default_expr == "String::new()",
        _ => default_expr == "Default::default()",
    }
}

fn infer_map_key_type(
    object_schema: &JsonSchema,
    additional_properties_schema: &JsonSchema,
) -> &'static str {
    // Mesh primitive attribute maps are semantically keyed by vertex attribute names.
    let mentions_attribute_semantic = object_schema
        .description
        .as_deref()
        .or(object_schema.detailed_description.as_deref())
        .is_some_and(|text| {
            let normalized = text.to_ascii_lowercase();
            normalized.contains("attribute semantic")
                || normalized.contains("mesh attribute semantic")
        });

    let references_gltf_id = additional_properties_schema
        .reference
        .as_deref()
        .is_some_and(|r| r.ends_with("glTFid.schema.json"));

    if mentions_attribute_semantic && references_gltf_id {
        "VertexAttribute"
    } else {
        "String"
    }
}

fn infer_enum_primitive_schema_type(details: &JsonSchema) -> Option<&'static str> {
    let mut saw_enum_shape = false;
    let mut saw_string = false;
    let mut saw_integer = false;

    let mut mark_literal = |value: &serde_json::Value| -> Option<()> {
        saw_enum_shape = true;
        if value.as_str().is_some() {
            saw_string = true;
            return Some(());
        }
        if value.as_u64().is_some() {
            saw_integer = true;
            return Some(());
        }
        None
    };

    if let Some(const_value) = &details.const_value {
        mark_literal(const_value)?;
    }

    if let Some(values) = &details.enum_values {
        for value in values {
            mark_literal(value)?;
        }
    }

    if let Some(any_of) = &details.any_of {
        for branch in any_of {
            let Some(object) = branch.as_object() else {
                continue;
            };

            if let Some(const_value) = object.get("const") {
                mark_literal(const_value)?;
                continue;
            }

            if let Some(enum_values) = object.get("enum").and_then(|value| value.as_array()) {
                for enum_value in enum_values {
                    mark_literal(enum_value)?;
                }
            }
        }
    }

    if !saw_enum_shape || (saw_string && saw_integer) {
        return None;
    }

    if saw_string {
        return Some("string");
    }

    if saw_integer {
        return Some("integer");
    }

    None
}

fn make_safe_identifier(name: &str) -> String {
    let reserved = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
    ];
    if reserved.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

fn sanitize_identifier_fragment(name: &str) -> String {
    name.strip_prefix("r#").unwrap_or(name).to_string()
}

/// Process a single schema and generate a Rust struct definition.
/// Properties from base schemas (allOf inheritance) are inlined - no
/// `#[serde(flatten)]`, no separate base struct.
pub fn generate_struct(
    cache: &mut SchemaCache,
    config: &Config,
    discovered_custom_types: &mut HashMap<String, CustomTypeConfig>,
    schema: &JsonSchema,
    source_path: Option<&std::path::Path>,
) -> Option<GeneratedStruct> {
    let title = schema.title.as_ref()?;
    let class_config = config.classes.get(title.as_str());

    // Skip types marked as skip (base classes, type aliases).
    if class_config.is_some_and(|c| c.skip) {
        return None;
    }

    let name = rust_type_name(config, title);

    if let Some(sp) = source_path {
        cache.push_context(sp);
    }

    // Collect all properties: own + inherited from allOf chain.
    let mut all_properties: HashMap<String, serde_json::Value> = HashMap::new();
    let mut all_required: Vec<String> = schema.required.clone();

    // Walk the allOf chain to collect inherited properties.
    collect_inherited_properties(cache, schema, &mut all_properties, &mut all_required);

    // Own properties override inherited ones, but skip empty objects
    // (used in JSON Schema to signal "this property exists" without redefining it).
    // Also skip `extensions` and `extras` - they're added by render_struct.
    for (k, v) in &schema.properties {
        if k == "extensions" || k == "extras" {
            continue;
        }
        if v.as_object().is_some_and(|o| o.is_empty()) {
            continue;
        }
        all_properties.insert(k.clone(), v.clone());
    }

    // Resolve properties into Rust fields.
    let mut fields = Vec::new();
    let mut prop_names: Vec<_> = all_properties.keys().cloned().collect();
    prop_names.sort();

    for prop_name in &prop_names {
        if let Some(prop_json) = all_properties.get(prop_name) {
            if let Some(field) = resolve_property(
                cache,
                config,
                discovered_custom_types,
                schema,
                &name,
                prop_name,
                prop_json,
                &all_required,
            ) {
                fields.push(field);
            }
        }
    }

    if source_path.is_some() {
        cache.pop_context();
    }

    Some(GeneratedStruct {
        name,
        title: title.clone(),
        doc: schema.description.clone(),
        extension_name: class_config
            .and_then(|c| c.extension_name.clone())
            .filter(|name| !name.is_empty()),
        attach_target: None, // Set by main.rs after generation
        fields,
        extra_fields: class_config
            .map(|c| c.extra_fields.clone())
            .unwrap_or_default(),
    })
}

/// A generated custom type inferred directly from a schema.
#[derive(Debug)]
struct InferredCustomType {
    name: String,
    config: CustomTypeConfig,
}

fn canonical_inferred_type_name(
    discovered_custom_types: &HashMap<String, CustomTypeConfig>,
    inferred: &InferredCustomType,
) -> String {
    if let Some(runtime_name) = canonical_runtime_enum_name(&inferred.config) {
        return runtime_name.to_string();
    }

    let mut matching_existing: Vec<&String> = discovered_custom_types
        .iter()
        .filter_map(|(name, existing)| {
            enum_domain_contains(existing, &inferred.config).then_some(name)
        })
        .collect();
    matching_existing.sort();

    matching_existing
        .into_iter()
        .next()
        .cloned()
        .unwrap_or_else(|| inferred.name.clone())
}

fn enum_domain_contains(existing: &CustomTypeConfig, inferred: &CustomTypeConfig) -> bool {
    if existing.kind != "enum" || inferred.kind != "enum" || existing.numeric != inferred.numeric {
        return false;
    }

    if existing.numeric {
        if !existing.numeric_values.is_empty() && !inferred.numeric_values.is_empty() {
            return inferred.numeric_values.iter().all(|(variant, value)| {
                existing
                    .numeric_values
                    .get(variant)
                    .is_some_and(|existing_value| existing_value == value)
            });
        }

        return inferred
            .variants
            .iter()
            .all(|variant| existing.variants.contains(variant));
    }

    inferred
        .variants
        .iter()
        .all(|variant| existing.variants.contains(variant))
}

fn canonical_runtime_enum_name(inferred: &CustomTypeConfig) -> Option<&'static str> {
    if inferred.kind != "enum" || !inferred.numeric {
        return None;
    }

    let filter_mode = [
        ("Nearest", 9728_u32),
        ("Linear", 9729_u32),
        ("NearestMipmapNearest", 9984_u32),
        ("LinearMipmapNearest", 9985_u32),
        ("NearestMipmapLinear", 9986_u32),
        ("LinearMipmapLinear", 9987_u32),
    ];
    if inferred.numeric_values.iter().all(|(variant, value)| {
        filter_mode
            .iter()
            .any(|(expected_variant, expected_value)| {
                variant == expected_variant && value == expected_value
            })
    }) {
        return Some("FilterMode");
    }

    let wrap_mode = [
        ("ClampToEdge", 33071_u32),
        ("MirroredRepeat", 33648_u32),
        ("Repeat", 10497_u32),
    ];
    if inferred.numeric_values.iter().all(|(variant, value)| {
        wrap_mode.iter().any(|(expected_variant, expected_value)| {
            variant == expected_variant && value == expected_value
        })
    }) {
        return Some("WrapMode");
    }

    None
}

fn register_inferred_custom_type(
    discovered_custom_types: &mut HashMap<String, CustomTypeConfig>,
    inferred: InferredCustomType,
) -> String {
    let type_name = canonical_inferred_type_name(discovered_custom_types, &inferred);

    if matches!(type_name.as_str(), "FilterMode" | "WrapMode") {
        return type_name;
    }

    if let Some(existing) = discovered_custom_types.get_mut(&type_name) {
        if existing.kind == "enum" && inferred.config.kind == "enum" {
            if !existing.numeric && !inferred.config.numeric {
                existing.variants.extend(inferred.config.variants);
                existing.variants.sort();
                existing.variants.dedup();
            } else if existing.numeric && inferred.config.numeric {
                for (variant, value) in inferred.config.numeric_values {
                    existing.numeric_values.entry(variant).or_insert(value);
                }

                if existing.numeric_values.is_empty() {
                    existing.variants.extend(inferred.config.variants);
                    existing.variants.sort();
                    existing.variants.dedup();
                } else {
                    let mut pairs: Vec<(String, u32)> = existing
                        .numeric_values
                        .iter()
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();
                    pairs.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
                    existing.variants = pairs.into_iter().map(|(k, _)| k).collect();
                }
            }

            if existing.doc.is_none() {
                existing.doc = inferred.config.doc;
            }
            if existing.origin.is_none() {
                existing.origin = inferred.config.origin;
            }
        }
    } else {
        discovered_custom_types.insert(type_name.clone(), inferred.config);
    }

    type_name
}

fn is_structured_token_pair_domain(values: &[String]) -> bool {
    !values.is_empty() && values.iter().all(|value| is_structured_token_pair(value))
}

fn is_structured_token_pair(value: &str) -> bool {
    let Some((left, right)) = value.split_once('/') else {
        return false;
    };

    if left.is_empty() || right.is_empty() || right.contains('/') {
        return false;
    }

    is_token(left) && is_token(right)
}

fn is_token(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(is_token_char)
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '!' | '#' | '$' | '&' | '^' | '_' | '.' | '+' | '-')
}

fn is_numeric_like_type_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !matches!(first, 'U' | 'I' | 'F') {
        return false;
    }

    let mut saw_digit = false;
    for c in chars {
        if !c.is_ascii_digit() {
            return false;
        }
        saw_digit = true;
    }

    saw_digit
}

fn is_inferable_custom_type_name(name: &str) -> bool {
    let is_upper_camel = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());

    if !is_upper_camel {
        return false;
    }

    if is_builtin_type_name(name) && !is_numeric_like_type_name(name) {
        return false;
    }

    // Runtime-provided types that should not be regenerated by schema inference.
    !matches!(name, "FilterMode" | "WrapMode" | "VertexAttribute")
}

/// Infer a glTF enum type from schema `enum`, `const`, or `anyOf` branches.
/// Returns `None` when the schema is not a finite enum-like union.
fn infer_enum_custom_type(
    details: &JsonSchema,
    type_hint: Option<&str>,
) -> Option<InferredCustomType> {
    let mut string_variants: Vec<String> = Vec::new();
    let mut numeric_variants: Vec<(String, u32)> = Vec::new();
    let mut saw_enum_shape = false;

    if let Some(const_value) = &details.const_value {
        saw_enum_shape = true;
        if let Some(text) = const_value.as_str() {
            string_variants.push(text.to_string());
        } else if let Some(number) = const_value.as_u64() {
            numeric_variants.push((format!("V{number}"), number as u32));
        } else {
            return None;
        }
    }

    if let Some(values) = &details.enum_values {
        saw_enum_shape = true;
        for value in values {
            if let Some(text) = value.as_str() {
                string_variants.push(text.to_string());
            } else if let Some(number) = value.as_u64() {
                numeric_variants.push((format!("V{number}"), number as u32));
            } else {
                return None;
            }
        }
    }

    if let Some(any_of) = &details.any_of {
        for branch in any_of {
            let Some(object) = branch.as_object() else {
                continue;
            };

            if let Some(const_value) = object.get("const") {
                saw_enum_shape = true;
                if let Some(text) = const_value.as_str() {
                    string_variants.push(text.to_string());
                } else if let Some(number) = const_value.as_u64() {
                    let variant = object
                        .get("description")
                        .and_then(|value| value.as_str())
                        .map(rust_enum_variant_name)
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| format!("V{number}"));
                    numeric_variants.push((variant, number as u32));
                } else {
                    return None;
                }
            } else if let Some(enum_values) = object.get("enum").and_then(|value| value.as_array())
            {
                saw_enum_shape = true;
                for enum_value in enum_values {
                    if let Some(text) = enum_value.as_str() {
                        string_variants.push(text.to_string());
                    } else if let Some(number) = enum_value.as_u64() {
                        let variant = object
                            .get("description")
                            .and_then(|value| value.as_str())
                            .map(rust_enum_variant_name)
                            .filter(|value| !value.is_empty())
                            .unwrap_or_else(|| format!("V{number}"));
                        numeric_variants.push((variant, number as u32));
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    if !saw_enum_shape {
        return None;
    }

    if !string_variants.is_empty() && !numeric_variants.is_empty() {
        return None;
    }

    if !string_variants.is_empty() {
        string_variants.sort();
        string_variants.dedup();

        let type_name = {
            let inferred_name = details
                .title
                .as_deref()
                .or(type_hint)
                .map(rust_enum_type_name)?;
            if !is_inferable_custom_type_name(&inferred_name) {
                return None;
            }
            inferred_name
        };

        return Some(InferredCustomType {
            name: type_name,
            config: CustomTypeConfig {
                kind: "enum".to_string(),
                variants: string_variants,
                doc: details.description.clone(),
                numeric: false,
                numeric_values: HashMap::new(),
                default: None,
                origin: Some("schema enum inferred from finite literals".to_string()),
            },
        });
    }

    if !numeric_variants.is_empty() {
        let inferred_name = details
            .title
            .as_deref()
            .or(type_hint)
            .map(rust_enum_type_name)?;
        if !is_inferable_custom_type_name(&inferred_name) {
            return None;
        }

        numeric_variants.sort_by(|a, b| a.1.cmp(&b.1));
        numeric_variants.dedup_by(|a, b| a.1 == b.1);

        let mut variants = Vec::with_capacity(numeric_variants.len());
        let mut numeric_values = HashMap::new();
        for (variant, number) in numeric_variants {
            numeric_values.insert(variant.clone(), number);
            variants.push(variant);
        }

        return Some(InferredCustomType {
            name: inferred_name,
            config: CustomTypeConfig {
                kind: "enum".to_string(),
                variants,
                doc: details.description.clone(),
                numeric: true,
                numeric_values,
                default: None,
                origin: Some("schema enum inferred from finite literals".to_string()),
            },
        });
    }

    None
}

fn rust_enum_type_name(name: &str) -> String {
    name.to_upper_camel_case()
}

fn rust_enum_variant_name(name: &str) -> String {
    name.to_upper_camel_case()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_from_json(value: serde_json::Value) -> JsonSchema {
        serde_json::from_value(value).expect("schema should deserialize")
    }

    #[test]
    fn infers_string_enum_from_anyof_consts() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Animation Channel Target Path",
            "anyOf": [
                { "const": "translation" },
                { "const": "rotation" },
                { "const": "scale" },
                { "const": "weights" },
                { "type": "string" }
            ]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("AnimationChannelTargetPath"))
            .expect("expected enum inference");

        assert_eq!(inferred.name, "AnimationChannelTargetPath");
        assert!(!inferred.config.numeric);
        assert_eq!(
            inferred.config.variants,
            vec!["rotation", "scale", "translation", "weights"]
        );
    }

    #[test]
    fn infers_string_enum_from_anyof_single_value_enums() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Articulation Stage Type",
            "anyOf": [
                { "enum": ["xTranslate"] },
                { "enum": ["yTranslate"] },
                { "enum": ["zTranslate"] },
                { "enum": ["uniformScale"] },
                { "type": "string" }
            ]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("ArticulationStageType"))
            .expect("expected enum inference");

        assert_eq!(
            inferred.config.variants,
            vec!["uniformScale", "xTranslate", "yTranslate", "zTranslate"]
        );
        assert!(!inferred.config.numeric);
    }

    #[test]
    fn infers_numeric_enum_from_anyof_consts() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Accessor Component Type",
            "anyOf": [
                { "const": 5120, "description": "BYTE", "type": "integer" },
                { "const": 5121, "description": "UNSIGNED_BYTE", "type": "integer" },
                { "const": 5122, "description": "SHORT", "type": "integer" },
                { "const": 5123, "description": "UNSIGNED_SHORT", "type": "integer" },
                { "const": 5125, "description": "UNSIGNED_INT", "type": "integer" },
                { "const": 5126, "description": "FLOAT", "type": "integer" },
                { "type": "integer" }
            ]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("AccessorComponentType"))
            .expect("expected numeric enum inference");

        assert_eq!(inferred.name, "AccessorComponentType");
        assert!(inferred.config.numeric);
        assert_eq!(inferred.config.numeric_values.get("Byte"), Some(&5120));
        assert_eq!(
            inferred.config.numeric_values.get("UnsignedInt"),
            Some(&5125)
        );
    }

    #[test]
    fn infers_structured_token_pair_domain_with_type_hint() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Texture Mime Type",
            "anyOf": [
                { "const": "image/png" },
                { "const": "image/jpeg" },
                { "type": "string" }
            ]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("TextureMimeType"))
            .expect("expected enum inference");

        assert_eq!(inferred.name, "TextureMimeType");
        assert_eq!(inferred.config.variants, vec!["image/jpeg", "image/png"]);
    }

    #[test]
    fn merges_structured_token_pair_variants_when_names_match() {
        let mut discovered_custom_types: HashMap<String, CustomTypeConfig> = HashMap::new();

        let first = schema_from_json(serde_json::json!({
            "anyOf": [
                { "const": "image/png" },
                { "const": "image/jpeg" },
                { "type": "string" }
            ]
        }));
        let second = schema_from_json(serde_json::json!({
            "anyOf": [
                { "const": "model/gltf+json" },
                { "const": "image/jpeg" },
                { "type": "string" }
            ]
        }));

        let first_inferred =
            infer_enum_custom_type(&first, Some("SharedDomain")).expect("expected first inference");
        let second_inferred = infer_enum_custom_type(&second, Some("SharedDomain"))
            .expect("expected second inference");

        let first_type =
            register_inferred_custom_type(&mut discovered_custom_types, first_inferred);
        let second_type =
            register_inferred_custom_type(&mut discovered_custom_types, second_inferred);

        assert_eq!(first_type, "SharedDomain");
        assert_eq!(second_type, "SharedDomain");

        let merged = discovered_custom_types
            .get("SharedDomain")
            .expect("content type should exist");
        assert_eq!(
            merged.variants,
            vec!["image/jpeg", "image/png", "model/gltf+json"]
        );
    }

    #[test]
    fn does_not_canonicalize_non_token_pair_string_enums() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Animation Path",
            "anyOf": [
                { "const": "translation" },
                { "const": "rotation" },
                { "type": "string" }
            ]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("AnimationPath"))
            .expect("expected enum inference");

        assert_eq!(inferred.name, "AnimationPath");
        assert_eq!(inferred.config.variants, vec!["rotation", "translation"]);
    }

    #[test]
    fn skips_reserved_runtime_type_names() {
        let schema = schema_from_json(serde_json::json!({
            "title": "Filter Mode",
            "enum": [9728, 9729]
        }));

        let inferred = infer_enum_custom_type(&schema, Some("FilterMode"));
        assert!(inferred.is_none());
    }

    #[test]
    fn normalizes_numeric_like_type_names() {
        let schema = schema_from_json(serde_json::json!({
            "title": "u32",
            "enum": [34962, 34963]
        }));

        let inferred =
            infer_enum_custom_type(&schema, Some("u32")).expect("expected enum inference");
        assert_eq!(inferred.name, "U32");
    }

    #[test]
    fn canonicalizes_subset_to_existing_custom_type() {
        let mut discovered_custom_types: HashMap<String, CustomTypeConfig> = HashMap::new();
        discovered_custom_types.insert(
            "AlphaMode".to_string(),
            CustomTypeConfig {
                kind: "enum".to_string(),
                variants: vec!["OPAQUE".into(), "MASK".into(), "BLEND".into()],
                doc: None,
                numeric: false,
                numeric_values: HashMap::new(),
                default: None,
                origin: None,
            },
        );

        let inferred = InferredCustomType {
            name: "MaterialAlphaMode".to_string(),
            config: CustomTypeConfig {
                kind: "enum".to_string(),
                variants: vec!["BLEND".into(), "MASK".into(), "OPAQUE".into()],
                doc: None,
                numeric: false,
                numeric_values: HashMap::new(),
                default: None,
                origin: None,
            },
        };

        let type_name = register_inferred_custom_type(&mut discovered_custom_types, inferred);
        assert_eq!(type_name, "AlphaMode");
    }

    #[test]
    fn canonicalizes_subset_to_runtime_filter_mode() {
        let mut discovered_custom_types: HashMap<String, CustomTypeConfig> = HashMap::new();

        let inferred = InferredCustomType {
            name: "SamplerMagFilter".to_string(),
            config: CustomTypeConfig {
                kind: "enum".to_string(),
                variants: vec!["Nearest".into(), "Linear".into()],
                doc: None,
                numeric: true,
                numeric_values: HashMap::from([
                    ("Nearest".to_string(), 9728_u32),
                    ("Linear".to_string(), 9729_u32),
                ]),
                default: None,
                origin: None,
            },
        };

        let type_name = register_inferred_custom_type(&mut discovered_custom_types, inferred);
        assert_eq!(type_name, "FilterMode");
        assert!(!discovered_custom_types.contains_key("FilterMode"));
    }
}

fn collect_inherited_properties(
    cache: &mut SchemaCache,
    schema: &JsonSchema,
    properties: &mut HashMap<String, serde_json::Value>,
    required: &mut Vec<String>,
) {
    for entry in &schema.all_of {
        // Each allOf entry can be a $ref or an inline schema with properties.
        if let Some(obj) = entry.as_object() {
            if let Some(ref_str) = obj.get("$ref").and_then(|v| v.as_str()) {
                if let Some(base_schema) = cache.load(ref_str) {
                    // Check if this base is a skipped type - if so, still
                    // collect its properties (that's the point of inlining).
                    // Recurse first to get the base's own bases.
                    collect_inherited_properties(cache, &base_schema, properties, required);

                    // Then collect the base's own properties.
                    for (k, v) in &base_schema.properties {
                        // Skip extensions/extras - added by render_struct.
                        if k == "extensions" || k == "extras" {
                            continue;
                        }
                        properties.insert(k.clone(), v.clone());
                    }
                    required.extend(base_schema.required.iter().cloned());
                }
            } else {
                // Inline schema in allOf - collect its properties directly.
                if let Ok(inline) = serde_json::from_value::<JsonSchema>(entry.clone()) {
                    for (k, v) in &inline.properties {
                        properties.insert(k.clone(), v.clone());
                    }
                    required.extend(inline.required.iter().cloned());
                }
            }
        }
    }
}

/// Render a `GeneratedStruct` to a Rust source string.
pub fn render_struct(s: &GeneratedStruct) -> String {
    let mut out = String::new();
    let mut default_fns: Vec<(String, String, String)> = Vec::new();
    // (fn_name, rust_type, comparison_value)
    let mut skip_fns: Vec<(String, String, String)> = Vec::new();

    // Pre-pass: determine whether any field has a non-trivial serde default.
    // If so, we cannot use #[derive(Default)] — it calls <T>::default() for every
    // field, ignoring the serde default functions, producing wrong values (e.g.
    // Node::rotation becomes [0,0,0,0] instead of the required [0,0,0,1]).
    let has_non_trivial_default = s.fields.iter().any(|f| {
        f.default_expr
            .as_deref()
            .map(|d| !is_type_default(d, &f.rust_type))
            .unwrap_or(false)
    });

    // Doc comment.
    if let Some(ref doc) = s.doc {
        for line in doc.lines() {
            let _ = writeln!(out, "/// {line}");
        }
    }

    // Derive macros. Omit Default when there are non-trivial serde defaults;
    // a manual impl Default is emitted after the struct instead.
    if has_non_trivial_default {
        let _ = writeln!(
            out,
            "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"
        );
    } else {
        let _ = writeln!(
            out,
            "#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]"
        );
    }

    let _ = writeln!(out, "pub struct {} {{", s.name);

    // Fields.
    for field in &s.fields {
        let _ = writeln!(out);

        // Doc.
        if let Some(ref doc) = field.doc {
            for line in doc.lines() {
                let _ = writeln!(out, "    /// {line}");
            }
        }

        // Serde attributes.
        let mut attrs = Vec::new();

        if field.needs_rename {
            attrs.push(format!("rename = \"{}\"", field.json_name));
        }

        if let Some(ref skip_if) = field.skip_serializing_if
            && !field.is_required
        {
            attrs.push(format!("skip_serializing_if = \"{skip_if}\""));
        }

        if field.enforce_non_empty {
            attrs.push("deserialize_with = \"deserialize_non_empty_vec\"".to_string());
        }

        // Collect skip comparison functions for emission after the struct.
        if let Some(ref val) = field.skip_if_value {
            if let Some(ref fn_name) = field.skip_serializing_if {
                skip_fns.push((fn_name.clone(), field.rust_type.clone(), val.clone()));
            }
        }

        if let Some(ref default_expr) = field.default_expr {
            if is_type_default(default_expr, &field.rust_type) {
                attrs.push("default".to_string());
            } else {
                // Generate a named default function.
                let fn_name = format!(
                    "default_{}_{}",
                    s.name.to_snake_case(),
                    sanitize_identifier_fragment(&field.rust_name)
                );
                default_fns.push((
                    fn_name.clone(),
                    field.rust_type.clone(),
                    default_expr.clone(),
                ));
                attrs.push(format!("default = \"{fn_name}\""));
            }
        } else if field.skip_serializing_if.is_some() && !field.is_required {
            attrs.push("default".to_string());
        }

        if !attrs.is_empty() {
            let _ = writeln!(out, "    #[serde({})]", attrs.join(", "));
        }

        let _ = writeln!(out, "    pub {}: {},", field.rust_name, field.rust_type);
    }

    // Extra (non-schema) fields injected via config, e.g. runtime binary payloads.
    // Extra fields injected via config - optionally participating in serde.
    for ef in &s.extra_fields {
        let _ = writeln!(out);
        if let Some(ref doc) = ef.doc {
            let _ = writeln!(out, "    /// {doc}");
        }
        if ef.skip_serde {
            let _ = writeln!(out, "    #[serde(skip)]");
        } else {
            let _ = writeln!(
                out,
                "    #[serde(default, skip_serializing_if = \"Option::is_none\")]"
            );
        }
        let _ = writeln!(out, "    pub {}: {},", ef.name, ef.rust_type);
    }

    // Extensions + extras (all ExtensibleObject types get these).
    let _ = writeln!(out);
    let _ = writeln!(out, "    /// Extension-specific data.");
    let _ = writeln!(
        out,
        "    #[serde(default, skip_serializing_if = \"std::collections::HashMap::is_empty\")]"
    );
    let _ = writeln!(
        out,
        "    pub extensions: std::collections::HashMap<String, serde_json::Value>,"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "    /// Application-specific data.");
    let _ = writeln!(
        out,
        "    #[serde(default, skip_serializing_if = \"Option::is_none\")]"
    );
    let _ = writeln!(out, "    pub extras: Option<serde_json::Value>,");

    let _ = writeln!(out, "}}");

    // Auto-generate GltfExtension impl for extension structs.
    if let Some(ref ext_name) = s.extension_name {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "impl crate::extensions::GltfExtension for {} {{",
            s.name
        );
        let _ = writeln!(out, "    const NAME: &'static str = \"{ext_name}\";");
        let _ = writeln!(out, "}}");
    }

    // Default helper functions.
    for (fn_name, rust_type, default_expr) in &default_fns {
        let _ = writeln!(out);
        let _ = writeln!(out, "fn {fn_name}() -> {rust_type} {{ {default_expr} }}");
    }

    // Manual impl Default for structs with non-trivial defaults, so that
    // programmatic construction (Struct::default()) matches serde deserialization.
    if has_non_trivial_default {
        let _ = writeln!(out);
        let _ = writeln!(out, "impl Default for {} {{", s.name);
        let _ = writeln!(out, "    fn default() -> Self {{");
        let _ = writeln!(out, "        Self {{");
        for field in &s.fields {
            if let Some(ref default_expr) = field.default_expr {
                if !is_type_default(default_expr, &field.rust_type) {
                    // Use the same serde default function.
                    let fn_name = format!(
                        "default_{}_{}",
                        s.name.to_snake_case(),
                        sanitize_identifier_fragment(&field.rust_name)
                    );
                    let _ = writeln!(out, "            {}: {}(),", field.rust_name, fn_name);
                    continue;
                }
            }
            // Trivial or absent default: fall back to the field's own Default.
            let _ = writeln!(out, "            {}: Default::default(),", field.rust_name);
        }
        // Extra fields (e.g. runtime binary payloads) always use Default.
        for ef in &s.extra_fields {
            let _ = writeln!(out, "            {}: Default::default(),", ef.name);
        }
        // extensions and extras always have trivial defaults.
        let _ = writeln!(out, "            extensions: Default::default(),");
        let _ = writeln!(out, "            extras: Default::default(),");
        let _ = writeln!(out, "        }}");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");
    }

    // Skip helper functions (value-comparison based).
    for (fn_name, rust_type, value) in &skip_fns {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "fn {fn_name}(v: &{rust_type}) -> bool {{ *v == {value} }}"
        );
    }

    out
}

/// Generate the complete Rust source file for a set of structs.
pub fn render_module(
    module_doc: &str,
    structs: &[GeneratedStruct],
    extra_imports: &[&str],
    custom_types: &std::collections::HashMap<String, crate::config::CustomTypeConfig>,
) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "// This file was generated by schema-gen.");
    let _ = writeln!(out, "// DO NOT EDIT THIS FILE!");
    let _ = writeln!(out);
    let _ = writeln!(out, "//! {module_doc}");
    let _ = writeln!(out);
    let _ = writeln!(out, "#![allow(clippy::all, missing_docs)]");
    let _ = writeln!(out);
    let referenced_custom_types = referenced_custom_types(structs, custom_types);

    // Only import Deserializer if a numeric enum custom type is present (needs a manual impl).
    let needs_deserializer = referenced_custom_types
        .values()
        .any(|c| c.kind == "enum" && !c.numeric_values.is_empty());
    if needs_deserializer {
        let _ = writeln!(
            out,
            "use serde::{{Serialize, Deserialize, Serializer, Deserializer}};"
        );
    } else {
        let _ = writeln!(out, "use serde::{{Deserialize, Serialize}};");
    }

    let generated_struct_symbols: HashSet<&str> = structs.iter().map(|s| s.name.as_str()).collect();

    for imp in extra_imports {
        let symbol = imp.rsplit("::").next().unwrap_or(imp);
        if referenced_custom_types.contains_key(symbol) || generated_struct_symbols.contains(symbol)
        {
            continue;
        }
        if !structs_reference_symbol(structs, symbol) {
            continue;
        }
        let _ = writeln!(out, "use {imp};");
    }

    let _ = writeln!(out);

    let needs_non_empty_vec_deserializer = structs.iter().any(|s| {
        s.fields
            .iter()
            .any(|field| field.enforce_non_empty && field.rust_type.starts_with("Vec<"))
    });

    if needs_non_empty_vec_deserializer {
        let _ = writeln!(
            out,
            "fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>"
        );
        let _ = writeln!(out, "where");
        let _ = writeln!(out, "    D: serde::Deserializer<'de>,");
        let _ = writeln!(out, "    T: serde::Deserialize<'de>,");
        let _ = writeln!(out, "{{");
        let _ = writeln!(
            out,
            "    let values = Vec::<T>::deserialize(deserializer)?;"
        );
        let _ = writeln!(out, "    if values.is_empty() {{");
        let _ = writeln!(
            out,
            "        return Err(serde::de::Error::custom(\"array must contain at least one item\"));"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    Ok(values)");
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);
    }

    // Render custom types first, in stable order.
    let mut sorted_types: Vec<_> = referenced_custom_types.iter().collect();
    sorted_types.sort_by(|a, b| a.0.cmp(b.0));
    for (type_name, type_config) in sorted_types {
        let _ = writeln!(out, "{}", render_custom_type(type_name, type_config));
    }

    for s in structs {
        let _ = writeln!(out, "{}", render_struct(s));
    }

    out
}

fn referenced_custom_types<'a>(
    structs: &[GeneratedStruct],
    custom_types: &'a std::collections::HashMap<String, crate::config::CustomTypeConfig>,
) -> std::collections::HashMap<String, &'a crate::config::CustomTypeConfig> {
    let mut referenced = std::collections::HashMap::new();
    for struct_def in structs {
        for field in &struct_def.fields {
            collect_custom_types_from_rust_type(&field.rust_type, custom_types, &mut referenced);
        }
    }
    referenced
}

fn collect_custom_types_from_rust_type<'a>(
    rust_type: &str,
    custom_types: &'a std::collections::HashMap<String, crate::config::CustomTypeConfig>,
    referenced: &mut std::collections::HashMap<String, &'a crate::config::CustomTypeConfig>,
) {
    for token in rust_type
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
    {
        if let Some(custom_type) = custom_types.get(token) {
            referenced.entry(token.to_string()).or_insert(custom_type);
        }
    }
}

fn structs_reference_symbol(structs: &[GeneratedStruct], symbol: &str) -> bool {
    structs.iter().any(|struct_def| {
        struct_def
            .fields
            .iter()
            .any(|field| rust_type_mentions_symbol(&field.rust_type, symbol))
            || struct_def
                .extra_fields
                .iter()
                .any(|field| rust_type_mentions_symbol(&field.rust_type, symbol))
    })
}

fn rust_type_mentions_symbol(rust_type: &str, symbol: &str) -> bool {
    rust_type
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|token| token == symbol)
}

/// Render a custom enum type definition.
fn render_custom_type(name: &str, config: &crate::config::CustomTypeConfig) -> String {
    let mut out = String::new();

    if config.kind == "enum" {
        // Doc comment
        if let Some(ref doc) = config.doc {
            for line in doc.lines() {
                let _ = writeln!(out, "/// {line}");
            }
        }

        let is_numeric = config.numeric;

        // Derive macros for enum.
        if is_numeric {
            let _ = writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
        } else {
            let _ = writeln!(
                out,
                "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]"
            );
        }

        let _ = writeln!(out, "pub enum {} {{", name);

        if is_numeric {
            for variant in config.variants.iter() {
                let pascal_case = variant.to_upper_camel_case();
                let _ = writeln!(out, "    {},", pascal_case);
            }
        } else {
            for variant in &config.variants {
                let pascal_case = variant.to_upper_camel_case();
                // Use the original variant name (from JSON) with serde rename only if it differs from pascal case
                if &pascal_case != variant {
                    let _ = writeln!(out, "    #[serde(rename = \"{}\")]", variant);
                }
                let _ = writeln!(out, "    {},", pascal_case);
            }
        }

        let _ = writeln!(out, "}}");

        // Add custom Serialize and Deserialize impls for numeric enums
        if is_numeric {
            let _ = writeln!(out);
            let _ = writeln!(out, "impl Serialize for {} {{", name);
            let _ = writeln!(
                out,
                "    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>"
            );
            let _ = writeln!(out, "    where");
            let _ = writeln!(out, "        S: Serializer,");
            let _ = writeln!(out, "    {{");
            let _ = writeln!(out, "        let value: u32 = match self {{");

            for variant in config.variants.iter() {
                let pascal_case = variant.to_upper_camel_case();
                let numeric_value = if !config.numeric_values.is_empty() {
                    config.numeric_values.get(variant).copied().unwrap_or(0)
                } else {
                    config
                        .variants
                        .iter()
                        .position(|v| v == variant)
                        .unwrap_or(0) as u32
                };
                let _ = writeln!(
                    out,
                    "            Self::{} => {},",
                    pascal_case, numeric_value
                );
            }

            let _ = writeln!(out, "        }};");
            let _ = writeln!(out, "        serializer.serialize_u32(value)");
            let _ = writeln!(out, "    }}");
            let _ = writeln!(out, "}}");
            let _ = writeln!(out);
            let _ = writeln!(out, "impl<'de> Deserialize<'de> for {} {{", name);
            let _ = writeln!(
                out,
                "    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>"
            );
            let _ = writeln!(out, "    where");
            let _ = writeln!(out, "        D: Deserializer<'de>,");
            let _ = writeln!(out, "    {{");
            let _ = writeln!(out, "        let value = u32::deserialize(deserializer)?;");
            let _ = writeln!(out, "        match value {{");

            for variant in config.variants.iter() {
                let pascal_case = variant.to_upper_camel_case();
                // Use the numeric value from the config if provided, otherwise use the variant index
                let numeric_value = if !config.numeric_values.is_empty() {
                    config.numeric_values.get(variant).copied().unwrap_or(0)
                } else {
                    config
                        .variants
                        .iter()
                        .position(|v| v == variant)
                        .unwrap_or(0) as u32
                };
                let _ = writeln!(
                    out,
                    "            {} => Ok(Self::{}),",
                    numeric_value, pascal_case
                );
            }

            let _ = writeln!(
                out,
                "            _ => Err(serde::de::Error::custom(format!(\"invalid {} value: {{}}\", value))),",
                name
            );
            let _ = writeln!(out, "        }}");
            let _ = writeln!(out, "    }}");
            let _ = writeln!(out, "}}");
        }

        // Implement Default trait, using configured default or first variant
        let default_variant = config
            .default
            .as_deref()
            .or_else(|| config.variants.first().map(|s| s.as_str()));
        if let Some(default_variant) = default_variant {
            let pascal_case = default_variant.to_upper_camel_case();
            let _ = writeln!(out);
            let _ = writeln!(out, "impl Default for {} {{", name);
            let _ = writeln!(out, "    fn default() -> Self {{");
            let _ = writeln!(out, "        Self::{}", pascal_case);
            let _ = writeln!(out, "    }}");
            let _ = writeln!(out, "}}");
        }
    }

    let _ = writeln!(out);
    out
}

/// Render a MANIFEST.md file documenting the generated types.
///
/// The manifest includes:
/// - A summary of generated structs (name, schema title, extension info)
/// - A summary of custom types (name, kind, origin)
/// - A summary of discovered extensions
pub fn render_manifest(
    structs: &[GeneratedStruct],
    custom_types: &std::collections::HashMap<String, crate::config::CustomTypeConfig>,
    config: &Config,
) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "# Generated Type Manifest");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "This file is auto-generated by `xtask`. Do not edit manually."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Structs ({} total)", structs.len());
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Rust Type | Schema Title | Extension | Attach Target | Fields |"
    );
    let _ = writeln!(
        out,
        "|-----------|-------------|-----------|---------------|--------|"
    );

    for s in structs {
        let ext = s.extension_name.as_deref().unwrap_or("");
        let target = s.attach_target.as_deref().unwrap_or("");
        let field_count = s.fields.len() + s.extra_fields.len();
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} |",
            s.name, s.title, ext, target, field_count
        );
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "## Custom Types ({} total)", custom_types.len());
    let _ = writeln!(out);
    let _ = writeln!(out, "| Type Name | Kind | Numeric | Origin |");
    let _ = writeln!(out, "|-----------|------|---------|--------|");

    // Sort by name for stable output.
    let mut sorted_types: Vec<_> = custom_types.iter().collect();
    sorted_types.sort_by(|a, b| a.0.cmp(b.0));

    for (name, tc) in &sorted_types {
        let kind = &tc.kind;
        let numeric = if tc.numeric { "yes" } else { "no" };
        let origin = tc.origin.as_deref().unwrap_or("(not documented)");
        let _ = writeln!(out, "| `{}` | {} | {} | {} |", name, kind, numeric, origin);
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "## Config Overrides ({} total)", config.classes.len());
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Schema Title | Override Name | Skipped | Property Overrides | Extra Fields |"
    );
    let _ = writeln!(
        out,
        "|--------------|---------------|---------|-------------------|--------------|"
    );

    let mut sorted_classes: Vec<_> = config.classes.iter().collect();
    sorted_classes.sort_by(|a, b| a.0.cmp(b.0));

    for (title, cc) in &sorted_classes {
        let override_name = cc.override_name.as_deref().unwrap_or("");
        let skipped = if cc.skip { "yes" } else { "no" };
        let prop_count = cc.property_overrides.len();
        let extra_count = cc.extra_fields.len();
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {} | {} |",
            title, override_name, skipped, prop_count, extra_count
        );
    }

    out
}
