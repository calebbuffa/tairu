//! Classification of resource references.

/// Whether a reference names an external 3D Tiles document.
///
/// Queries and fragments are ignored. Ordinary glTF content is not mistaken
/// for a tileset: only JSON and `.tileset.gltf`/`.tileset.glb` are recognized.
pub fn is_external_tileset_uri(uri: &str) -> bool {
    let lower = uri
        .split(['?', '#'])
        .next()
        .unwrap_or(uri)
        .to_ascii_lowercase();
    lower.ends_with(".json") || lower.ends_with(".tileset.gltf") || lower.ends_with(".tileset.glb")
}

#[cfg(test)]
mod tests {
    use super::is_external_tileset_uri;

    #[test]
    fn recognizes_tilesets_without_classifying_ordinary_gltf() {
        for name in [
            "tileset.json",
            "TILESET.JSON?token=x#root",
            "a.tileset.gltf",
            "a.tileset.glb",
        ] {
            assert!(is_external_tileset_uri(name));
        }
        for name in ["tile.glb", "tile.gltf", "tile.b3dm", "tile.bin?name=.json"] {
            assert!(!is_external_tileset_uri(name));
        }
    }
}
