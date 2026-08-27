//! URI resolution and caller-provided resource loading.

use crate::reader::TileParseError;

/// A resource-loading failure while loading a tileset by URI.
#[derive(Debug, thiserror::Error)]
pub enum UriLoadError<E: std::error::Error + 'static> {
    #[error("failed to fetch {uri}: {source}")]
    Fetch {
        uri: String,
        #[source]
        source: E,
    },
    #[error("failed to parse {uri}: {source}")]
    Parse {
        uri: String,
        #[source]
        source: TileParseError,
    },
}

/// Returns `true` if the given URI refers to an external tileset.
///
/// This checks if the URI (after stripping query parameters and fragments)
/// ends with `.json` when lowercased. This is a heuristic used to distinguish
/// tileset JSON files from content files (GLB, B3DM, etc.).
pub fn is_external_tileset_uri(uri: &str) -> bool {
    uri.split(['?', '#'])
        .next()
        .unwrap_or(uri)
        .to_ascii_lowercase()
        .ends_with(".json")
}

/// Resolves a resource reference against a URI or filesystem path.
pub fn resolve_uri(base: &str, reference: &str) -> String {
    if url::Url::parse(reference).is_ok() || reference.starts_with("//") {
        return reference.to_string();
    }
    if let Ok(base_url) = url::Url::parse(base) {
        return base_url
            .join(reference)
            .map(|uri| uri.to_string())
            .unwrap_or_else(|_| reference.to_string());
    }
    let path = std::path::Path::new(base)
        .parent()
        .map(|parent| parent.join(reference))
        .unwrap_or_else(|| std::path::PathBuf::from(reference));
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::resolve_uri;

    #[test]
    fn resolves_http_references_with_url_semantics() {
        assert_eq!(
            resolve_uri("https://example.com/a/tileset.json", "../b/tile.json"),
            "https://example.com/b/tile.json"
        );
    }

    #[test]
    fn preserves_unc_path_structure() {
        assert_eq!(
            resolve_uri(r"\\server\share\a\tileset.json", r"..\b\tile.json"),
            r"\\server\share\a\..\b\tile.json"
        );
    }
}
