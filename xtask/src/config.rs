//! The decisions about the 3D Tiles schemas that no general rule predicts.

use schemagen::Config;

/// Builds tairu's generation config.
///
/// Titles are spelled exactly as the schemas spell them; several schemas
/// differ only in case and are different types.
pub fn config() -> schemagen::Result<Config> {
    Config::builder()
        .extensible(true)
        .additional_schema("Subtree/subtree.schema.json")
        .additional_schema("Subtree/availability.schema.json")
        .additional_schema("Subtree/buffer.schema.json")
        .additional_schema("Subtree/bufferView.schema.json")
        // Counts, offsets and lengths index buffers in memory.
        .field_type("*.bitstream", "Option<usize>")
        .field_type("*.constant", "Option<u64>")
        .field_type("BufferView.buffer", "usize")
        .field_type("BufferView.byteOffset", "usize")
        .field_type("BufferView.byteLength", "usize")
        .field_type("Buffer.byteLength", "usize")
        .field_type("Subtrees.uri", "String")
        // Metadata types are validated by tairu, not by the type system.
        .field_type("Enum.valueType", "Option<serde_json::Value>")
        .field_type(
            "PropertyTableProperty.arrayOffsetType",
            "Option<serde_json::Value>",
        )
        .field_type(
            "PropertyTableProperty.stringOffsetType",
            "Option<serde_json::Value>",
        )
        .field_type("ClassProperty.componentType", "Option<serde_json::Value>")
        .field_type("ClassProperty.type", "Option<serde_json::Value>")
        .field_type(
            "PropertyStatistics.occurrences",
            "std::collections::HashMap<String, serde_json::Value>",
        )
        // Base schemas whose members are inlined, or that custom types replace.
        .schema("Root Property", |schema| schema.skip())
        .schema("Any Value", |schema| schema.replace_with("AnyValue"))
        .schema("Numeric Value", |schema| {
            schema.replace_with("NumericValue")
        })
        .schema("No Data Value", |schema| schema.replace_with("NoDataValue"))
        .schema("Tile", |schema| {
            schema
                .override_name("Tile")
                .property_type("refine", "Option<Refine>")
                .property_type("transform", "[f64; 16]")
                .property_default(
                    "transform",
                    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
                )
        })
        .schema("Implicit tiling", |schema| {
            schema
                .override_name("ImplicitTiling")
                .property_type("subdivisionScheme", "SubdivisionScheme")
        })
        .schema("Bounding Volume", |schema| {
            schema
                .override_name("BoundingVolume")
                .property_type("box", "Option<[f64; 12]>")
                .property_type("region", "Option<[f64; 6]>")
                .property_type("sphere", "Option<[f64; 4]>")
        })
        .schema("Group Metadata", |schema| schema.override_name("Group"))
        .schema("Buffer View", |schema| schema.override_name("BufferView"))
        .schema("Buffer", |schema| {
            schema
                .override_name("Buffer")
                .extra_field("data", "bytes::Bytes", |field| {
                    field
                        .doc("The buffer's decoded bytes; never part of the JSON.")
                        .skip_serde()
                })
        })
        .custom_enum("Refine", |custom| custom.variants(["ADD", "REPLACE"]))
        .custom_enum("SubdivisionScheme", |custom| {
            custom.variants(["Quadtree", "Octree", "S2"])
        })
        .custom_alias(
            "AnyValue",
            "serde_json::Value",
            "An arbitrary JSON value described by the 3D Tiles metadata specification.",
        )
        .custom_alias(
            "NumericValue",
            "serde_json::Value",
            "A numeric value described by the 3D Tiles metadata specification.",
        )
        .custom_alias(
            "NoDataValue",
            "serde_json::Value",
            "A no-data value described by the 3D Tiles metadata specification.",
        )
        .custom_alias("Value", "serde_json::Value", "An arbitrary JSON value.")
        .build()
}
