//! Specification-level tileset validation and reporting-neutral diagnostics.

use crate::{Error, Tileset};

/// A non-fatal validation finding.
///
/// Diagnostics are warnings; invalid required data remains an [`Error`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Stable identifier for the validation rule.
    pub code: String,
    /// JSON-style location of the finding.
    pub path: String,
    /// Human-readable explanation.
    pub message: String,
}

impl Diagnostic {
    /// Creates a warning diagnostic.
    pub fn warning(
        code: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Validates tileset invariants and returns non-fatal findings.
///
/// An empty `asset.version` is invalid and returns an error. Negative
/// geometric errors and required extensions not listed as used are retained
/// as warnings.
pub fn validate_tileset(tileset: &Tileset) -> Result<Vec<Diagnostic>, Error> {
    if tileset.asset.version.is_empty() {
        return Err(Error::parse(
            "<tileset>",
            "asset.version is required and must not be empty",
        ));
    }

    let mut diagnostics = Vec::new();
    if tileset.geometric_error < 0.0 {
        diagnostics.push(Diagnostic::warning(
            "tileset.negative_geometric_error",
            "geometricError",
            format!(
                "geometricError is negative ({}); expected >= 0",
                tileset.geometric_error
            ),
        ));
    }
    if tileset.root.geometric_error < 0.0 {
        diagnostics.push(Diagnostic::warning(
            "tile.negative_geometric_error",
            "root.geometricError",
            format!(
                "root.geometricError is negative ({}); expected >= 0",
                tileset.root.geometric_error
            ),
        ));
    }

    for (index, extension) in tileset.extensions_required.iter().enumerate() {
        if !tileset.extensions_used.contains(extension) {
            diagnostics.push(Diagnostic::warning(
                "tileset.required_extension_not_used",
                format!("extensionsRequired[{index}]"),
                format!(
                    "extensionsRequired contains '{}' which is not listed in extensionsUsed",
                    extension
                ),
            ));
        }
    }

    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, BoundingVolume, Tile};

    fn minimal_tileset() -> Tileset {
        Tileset {
            asset: Asset {
                version: "1.1".into(),
                ..Default::default()
            },
            root: Tile {
                bounding_volume: BoundingVolume {
                    sphere: Some([0.0, 0.0, 0.0, 1.0]),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn reports_all_spec_warnings_with_locations() {
        let mut tileset = minimal_tileset();
        tileset.geometric_error = -1.0;
        tileset.root.geometric_error = -2.0;
        tileset.extensions_required = vec!["EXT_a".into(), "EXT_b".into()];
        tileset.extensions_used = vec!["EXT_b".into()];

        let diagnostics = validate_tileset(&tileset).unwrap();
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.path.as_str()))
                .collect::<Vec<_>>(),
            [
                ("tileset.negative_geometric_error", "geometricError"),
                ("tile.negative_geometric_error", "root.geometricError"),
                (
                    "tileset.required_extension_not_used",
                    "extensionsRequired[0]"
                ),
            ]
        );
    }

    #[test]
    fn empty_version_remains_an_error() {
        let mut tileset = minimal_tileset();
        tileset.asset.version = "".into();
        assert!(matches!(
            validate_tileset(&tileset),
            Err(Error::Parse { .. })
        ));
    }
}
