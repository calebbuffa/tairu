//! Parse and validate 3D Tiles JSON documents.

use crate::{Error, ExtMeshFeatures, Extension, Tileset};
use std::io::Read;

/// Parses and validates a tileset from raw JSON bytes.
pub fn from_slice(data: &[u8]) -> Result<Tileset, Error> {
    let tileset = serde_json::from_slice::<Tileset>(data)
        .map_err(|source| Error::parse("<slice>", source.to_string()))?;
    validate(&tileset)?;
    Ok(tileset)
}

/// Parses and validates a tileset from a JSON string.
pub fn from_str(value: &str) -> Result<Tileset, Error> {
    from_slice(value.as_bytes())
}

/// Parses and validates a tileset from any reader.
pub fn from_reader(reader: impl Read) -> Result<Tileset, Error> {
    let tileset = serde_json::from_reader::<_, Tileset>(reader)
        .map_err(|source| Error::parse("<reader>", source.to_string()))?;
    validate(&tileset)?;
    Ok(tileset)
}

/// Validate a successfully parsed tileset, emitting non-fatal issues as warnings.
fn validate(tileset: &Tileset) -> Result<(), Error> {
    if tileset.asset.version.is_empty() {
        return Err(Error::parse(
            "<tileset>",
            "asset.version is required and must not be empty",
        ));
    }

    if tileset.geometric_error < 0.0 {
        log::warn!(
            "geometricError is negative ({}); expected >= 0",
            tileset.geometric_error
        );
    }
    if tileset.root.geometric_error < 0.0 {
        log::warn!(
            "root.geometricError is negative ({}); expected >= 0",
            tileset.root.geometric_error
        );
    }

    for ext in &tileset.extensions_required {
        if !tileset.extensions_used.contains(ext) {
            log::warn!(
                "extensionsRequired contains '{}' which is not listed in extensionsUsed",
                ext
            );
        }
        if &**ext == ExtMeshFeatures::NAME
            && let Some(content) = &tileset.root.content
            && let Some(value) = content.extensions.get(ExtMeshFeatures::NAME)
            && let Some(emf) = ExtMeshFeatures::from_json(value)
        {
            for feature_id in &emf.feature_ids {
                if feature_id.feature_count == 0 {
                    log::warn!(
                        "EXT_mesh_features: featureCount is 0 (feature ID set has no features)"
                    );
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_json() -> &'static [u8] {
        br#"{
            "asset": { "version": "1.1" },
            "geometricError": 100.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1000] },
                "geometricError": 100.0,
                "refine": "ADD"
            }
        }"#
    }

    #[test]
    fn parses_minimal_tileset() {
        let tileset = from_slice(minimal_json()).unwrap();
        assert_eq!(&*tileset.asset.version, "1.1");
        assert_eq!(tileset.geometric_error, 100.0);
    }

    #[test]
    fn accepts_negative_geometric_error_with_warning() {
        let json = br#"{
            "asset": { "version": "1.1" },
            "geometricError": -1.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.0
            }
        }"#;
        assert!(from_slice(json).is_ok());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(from_slice(b"not json"), Err(Error::Parse { .. })));
    }

    #[test]
    fn rejects_empty_version() {
        let json = br#"{
            "asset": { "version": "" },
            "geometricError": 0.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.0
            }
        }"#;
        assert!(matches!(from_slice(json), Err(Error::Parse { .. })));
    }
}
