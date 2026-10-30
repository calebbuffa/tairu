//! Tileset parsing and validation from bytes.

use crate::{Error, Tileset, validate_tileset};
use std::io::Read;

/// Parses and validates a tileset from raw JSON bytes, naming `uri` in errors.
pub fn parse_tileset(data: &[u8], uri: &str) -> Result<Tileset, Error> {
    parse_tileset_with_diagnostics(data, uri).map(|(tileset, _)| tileset)
}

/// Parses and validates a tileset, returning warnings without reporting them.
///
/// Use the returned diagnostics to apply an application-specific reporting
/// policy. The convenience parser discards non-fatal findings.
pub fn parse_tileset_with_diagnostics(
    data: &[u8],
    uri: &str,
) -> Result<(Tileset, Vec<crate::Diagnostic>), Error> {
    let tileset = serde_json::from_slice::<Tileset>(data)
        .map_err(|source| Error::parse(uri, source.to_string()))?;
    let diagnostics = validate_tileset(&tileset)?;
    Ok((tileset, diagnostics))
}

/// Parses and validates a tileset from raw JSON bytes.
pub fn from_slice(data: &[u8]) -> Result<Tileset, Error> {
    parse_tileset(data, "<slice>")
}

/// Parses and validates a tileset, returning warnings without reporting them.
pub fn from_slice_with_diagnostics(
    data: &[u8],
) -> Result<(Tileset, Vec<crate::Diagnostic>), Error> {
    parse_tileset_with_diagnostics(data, "<slice>")
}

/// Parses and validates a tileset from a JSON string.
pub fn from_str(value: &str) -> Result<Tileset, Error> {
    from_slice(value.as_bytes())
}

/// Parses and validates a tileset string, returning warnings without reporting them.
pub fn from_str_with_diagnostics(value: &str) -> Result<(Tileset, Vec<crate::Diagnostic>), Error> {
    from_slice_with_diagnostics(value.as_bytes())
}

/// Parses and validates a tileset from any reader.
pub fn from_reader(reader: impl Read) -> Result<Tileset, Error> {
    from_reader_with_diagnostics(reader).map(|(tileset, _)| tileset)
}

/// Parses and validates a tileset reader, returning warnings without reporting them.
pub fn from_reader_with_diagnostics(
    reader: impl Read,
) -> Result<(Tileset, Vec<crate::Diagnostic>), Error> {
    let tileset = serde_json::from_reader::<_, Tileset>(reader)
        .map_err(|source| Error::parse("<reader>", source.to_string()))?;
    let diagnostics = validate_tileset(&tileset)?;
    Ok((tileset, diagnostics))
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
    fn negative_geometric_error_is_returned_as_diagnostic() {
        let json = br#"{
            "asset": { "version": "1.1" },
            "geometricError": -1.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.0
            }
        }"#;
        assert!(from_slice(json).is_ok());
        let (_, diagnostics) = from_slice_with_diagnostics(json).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "tileset.negative_geometric_error");
        assert_eq!(diagnostics[0].path, "geometricError");
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

    #[test]
    fn generic_parser_does_not_validate_extension_payloads() {
        let json = br#"{
            "asset": { "version": "1.1" },
            "extensionsRequired": ["EXT_mesh_features"],
            "geometricError": 0.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.0,
                "content": {
                    "uri": "tile.glb",
                    "extensions": {
                        "EXT_mesh_features": {
                            "featureIds": [{ "featureCount": 0 }]
                        }
                    }
                }
            }
        }"#;
        let (_, diagnostics) = from_slice_with_diagnostics(json).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "tileset.required_extension_not_used");
    }
}
