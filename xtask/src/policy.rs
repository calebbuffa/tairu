use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use schemagen::{GenerationPolicy, StructDef, TypeSettings};

/// Tairu's rules for the generated 3D Tiles model.
///
/// Everything that is data about particular schemas lives in `config.json`;
/// only the representation choices and the `HasExtensions` impls are code.
pub struct TairuPolicy;

impl GenerationPolicy for TairuPolicy {
    fn settings(&self) -> TypeSettings {
        // Tileset JSON is parsed once per document and then read repeatedly
        // during traversal, so boxed strings and sorted-slice maps trade
        // mutation capacity nobody uses for a smaller footprint. Numeric
        // widths stay at JSON's own: 3D Tiles carries geodetic coordinates and
        // byte offsets into potentially large buffers, so narrowing by default
        // would be lossy.
        TypeSettings::compact()
    }

    fn struct_items(&self, definition: &StructDef) -> Vec<TokenStream> {
        if !definition.extensions {
            return Vec::new();
        }
        let name = format_ident!("{}", definition.name);
        vec![quote! {
            impl crate::HasExtensions for #name {
                fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {
                    &self.extensions
                }
                fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
                    &mut self.extensions
                }
            }
        }]
    }
}
