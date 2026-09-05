//! A typed base location for 3D Tiles resources: a remote URL, a Unix path,
//! or a Windows path.
//!
//! `std::path::Path` parses a string differently depending on the platform a
//! binary was *compiled* for (backslashes and drive letters are meaningless
//! on a Unix build, and vice versa), and a generic [`url::Url`] cannot
//! reliably tell a Windows drive letter (`C:\...`) apart from a URL scheme.
//! This type removes that ambiguity by deciding, once, which kind of
//! location a string represents, then resolving relative references using
//! that kind's own rules rather than re-guessing on every call.

use typed_path::{Utf8Encoding, Utf8PathBuf, Utf8TypedPath, Utf8UnixPathBuf, Utf8WindowsPathBuf};
use url::Url;

/// Returns `true` if the given URI refers to an external tileset.
///
/// This checks if the URI (after stripping query parameters and fragments)
/// ends with one of:
///
/// - `.json` (3D Tiles 1.x tileset JSON),
/// - `.tileset.gltf` (3D Tiles Next tileset-as-glTF),
/// - `.tileset.glb` (3D Tiles Next binary tileset-as-glTF).
///
/// It deliberately does **not** classify every `.gltf`/`.glb` as a tileset,
/// because those are still common tile-content payloads.
pub fn is_external_tileset_uri(uri: &str) -> bool {
    let lower = uri
        .split(['?', '#'])
        .next()
        .unwrap_or(uri)
        .to_ascii_lowercase();
    lower.ends_with(".json") || lower.ends_with(".tileset.gltf") || lower.ends_with(".tileset.glb")
}

/// Resolves a resource reference against a URI or filesystem path.
///
/// Handles remote URLs, Windows paths (drive letters and UNC shares), and
/// Unix paths uniformly, regardless of which platform this is compiled for.
/// See [`Uri`] for the underlying resolution rules.
pub fn resolve_uri(base: &str, reference: &str) -> String {
    Uri::parse(base).resolve(reference).to_string()
}

/// A resolved tileset/content location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Uri {
    /// A remote or `file://` URL.
    Url(Url),
    /// A Unix-style filesystem path (`/` separators).
    UnixPath(Utf8UnixPathBuf),
    /// A Windows-style filesystem path (`\` separators, optional drive
    /// letter or UNC prefix).
    WindowsPath(Utf8WindowsPathBuf),
}

impl Uri {
    /// Parses `s`, detecting a remote URL, a Windows path, or a Unix path.
    ///
    /// Bare relative references (no scheme, no drive letter, no leading
    /// `\\`) are classified by [`Utf8TypedPath::derive`], which treats a
    /// leading `\` or a `X:` drive prefix as Windows and everything else as
    /// Unix.
    pub fn parse(s: &str) -> Uri {
        if let Some(url) = Self::parse_as_url(s) {
            return Uri::Url(url);
        }
        // A literal backslash never appears as a Unix separator, so its
        // presence is a stronger signal than the general heuristic
        // `Utf8TypedPath::derive` uses (e.g. for an unrecognized scheme like
        // "notadriveletter:\file").
        if s.contains('\\') {
            return Uri::WindowsPath(Utf8WindowsPathBuf::from(s));
        }
        match Utf8TypedPath::derive(s) {
            Utf8TypedPath::Windows(path) => Uri::WindowsPath(path.to_path_buf()),
            Utf8TypedPath::Unix(path) => Uri::UnixPath(path.to_path_buf()),
        }
    }

    /// Parses `s` as a URL only if it is a genuine, hierarchical URL (i.e.
    /// not a Windows drive letter mistaken for a one-letter scheme).
    fn parse_as_url(s: &str) -> Option<Url> {
        if looks_like_windows_drive_path(s) {
            // e.g. "C:/data/tileset.json": a forward-slash drive path parses
            // as a syntactically valid (non-opaque) URL with scheme "c", so
            // it must be excluded explicitly rather than relying on
            // `cannot_be_a_base`.
            return None;
        }
        if let Some(rest) = s.strip_prefix("//") {
            // Protocol-relative URL; assume https like a browser would.
            return Url::parse(&format!("https://{rest}")).ok();
        }
        Url::parse(s).ok().filter(|url| !url.cannot_be_a_base())
    }

    /// Resolves `reference` against `self` as a base location.
    ///
    /// An absolute reference (a real URL, or an absolute path in the base's
    /// own flavor) replaces `self` entirely. Otherwise the reference is
    /// joined onto the base's parent directory using the base's own path
    /// rules.
    pub fn resolve(&self, reference: &str) -> Uri {
        if reference.is_empty() {
            return self.clone();
        }
        if let Some(url) = Self::parse_as_url(reference) {
            return Uri::Url(url);
        }
        match self {
            Uri::Url(base) => base
                .join(reference)
                .map(Uri::Url)
                .unwrap_or_else(|_| Uri::parse(reference)),
            Uri::WindowsPath(base) => {
                Uri::WindowsPath(resolve_path(base, Utf8WindowsPathBuf::from(reference)))
            }
            Uri::UnixPath(base) => {
                Uri::UnixPath(resolve_path(base, Utf8UnixPathBuf::from(reference)))
            }
        }
    }

    /// Returns the final path segment (or URL path segment), if any.
    /// The final component of this location, including any extension.
    ///
    /// Returns `None` for URLs without a path, paths ending in `..`, etc.
    pub fn file_name(&self) -> Option<&str> {
        match self {
            Uri::Url(url) => url.path_segments()?.next_back().filter(|s| !s.is_empty()),
            Uri::UnixPath(path) => path.file_name(),
            Uri::WindowsPath(path) => path.file_name(),
        }
    }

    /// Returns the file name without its extension, if any.
    ///
    /// A leading dot (e.g. `.hidden`) is not treated as an extension
    /// separator, so the stem of `.hidden` is `.hidden`, not empty.
    /// The final component of this location without its extension.
    pub fn stem(&self) -> Option<&str> {
        match self {
            Uri::Url(_) => {
                let name = self.file_name()?;
                match name.rfind('.') {
                    None | Some(0) => Some(name),
                    Some(dot) => Some(&name[..dot]),
                }
            }
            Uri::UnixPath(path) => path.file_stem(),
            Uri::WindowsPath(path) => path.file_stem(),
        }
    }

    /// The extension of the final component, *including* the leading dot.
    ///
    /// A trailing dot (e.g. `"test."`) yields `Some(".")`; dotfiles have no
    /// extension. Returns `None` when there is no extension.
    pub fn extension(&self) -> Option<&str> {
        match self {
            Uri::Url(_) => {
                let name = self.file_name()?;
                let dot = name.rfind('.')?;
                (dot > 0).then(|| &name[dot..])
            }
            Uri::UnixPath(path) => extension_with_dot(path.file_name()?, path.extension()),
            Uri::WindowsPath(path) => extension_with_dot(path.file_name()?, path.extension()),
        }
    }
}

/// `typed_path`'s `extension()` omits the leading `.` (like `std::path`);
/// re-attach it so all three [`Uri`] variants agree on the convention.
fn extension_with_dot<'a>(file_name: &'a str, extension: Option<&'a str>) -> Option<&'a str> {
    let ext = extension?;
    let dot = file_name.len() - ext.len() - 1;
    (dot > 0).then(|| &file_name[dot..])
}

/// Returns `true` if `s` starts with an ASCII drive letter followed by `:`
/// (e.g. `C:\...` or `C:/...`), which `url::Url` would otherwise mistake for
/// a one-letter URL scheme.
fn looks_like_windows_drive_path(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes.len() == 2 || bytes[2] == b'\\' || bytes[2] == b'/')
}

fn resolve_path<T: Utf8Encoding>(
    base: &Utf8PathBuf<T>,
    reference: Utf8PathBuf<T>,
) -> Utf8PathBuf<T> {
    if reference.is_absolute() {
        return reference;
    }
    // Normalize explicitly so ".." resolution is identical for both path
    // flavors and matches the dot-segment removal url::Url::join performs.
    match base.parent() {
        Some(parent) => parent.join(&reference).normalize(),
        None => reference,
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Uri::Url(url) => write!(f, "{url}"),
            Uri::UnixPath(path) => write!(f, "{path}"),
            Uri::WindowsPath(path) => write!(f, "{path}"),
        }
    }
}

// Parsing is infallible (an unrecognized string always falls back to a
// path), so `From` rather than `TryFrom` is the right conversion here.
impl From<&str> for Uri {
    fn from(s: &str) -> Self {
        Uri::parse(s)
    }
}

impl From<String> for Uri {
    fn from(s: String) -> Self {
        Uri::parse(&s)
    }
}

impl From<Uri> for String {
    fn from(uri: Uri) -> Self {
        uri.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Uri, is_external_tileset_uri, resolve_uri};

    mod classification {
        use super::*;

        #[test]
        fn remote_schemes_are_urls() {
            assert!(matches!(
                Uri::parse("https://example.com/a.json"),
                Uri::Url(_)
            ));
            assert!(matches!(
                Uri::parse("http://example.com/a.json"),
                Uri::Url(_)
            ));
        }

        #[test]
        fn protocol_relative_is_treated_as_https() {
            assert_eq!(
                Uri::parse("//example.com/a.json").to_string(),
                "https://example.com/a.json"
            );
        }

        #[test]
        fn custom_hierarchical_schemes_are_urls() {
            assert!(matches!(
                Uri::parse("geopackage:/home/courtyard_imagery.gpkg"),
                Uri::Url(_)
            ));
        }

        #[test]
        fn windows_drive_letters_are_never_urls() {
            // The bug this module exists to fix: a drive letter followed by
            // ':' is syntactically a valid one-letter URL scheme, and must
            // be excluded explicitly.
            assert!(matches!(
                Uri::parse(r"C:\data\tileset.json"),
                Uri::WindowsPath(_)
            ));
            assert!(matches!(
                Uri::parse("C:/data/tileset.json"),
                Uri::WindowsPath(_)
            ));
            assert!(matches!(
                Uri::parse("c:\\data\\tileset.json"),
                Uri::WindowsPath(_)
            ));
        }

        #[test]
        fn multi_letter_prefix_with_backslash_is_not_a_url() {
            // "notadriveletter:\file" is a multi-letter, opaque (cannot
            // be a base) scheme once a literal backslash follows the
            // colon, so it is rejected as a URL and treated as a path.
            assert!(matches!(
                Uri::parse(r"notadriveletter:\file"),
                Uri::WindowsPath(_)
            ));
        }

        #[test]
        fn unc_paths_are_windows_paths() {
            assert!(matches!(
                Uri::parse(r"\\qalab_server\vsdata\v36\tileset.json"),
                Uri::WindowsPath(_)
            ));
        }

        #[test]
        fn unix_paths_absolute_and_relative() {
            assert!(matches!(Uri::parse("/data/tileset.json"), Uri::UnixPath(_)));
            assert!(matches!(Uri::parse("data/tileset.json"), Uri::UnixPath(_)));
        }
    }

    mod resolve_url_base {
        use super::*;

        #[test]
        fn resolves_root_relative_reference() {
            let uri = Uri::parse("https://www.example.com/");
            assert_eq!(
                uri.resolve("/page/test").to_string(),
                "https://www.example.com/page/test"
            );
        }

        #[test]
        fn resolves_dot_segments_like_a_url() {
            let uri = Uri::parse("https://example.com/a/tileset.json");
            assert_eq!(
                uri.resolve("../b/tile.json").to_string(),
                "https://example.com/b/tile.json"
            );
        }

        #[test]
        fn resolves_simple_relative_reference() {
            let uri = Uri::parse("https://example.com/a/tileset.json");
            assert_eq!(
                uri.resolve("content/1.glb").to_string(),
                "https://example.com/a/content/1.glb"
            );
        }

        #[test]
        fn absolute_url_reference_replaces_base() {
            let uri = Uri::parse("https://example.com/a/tileset.json");
            assert_eq!(
                uri.resolve("https://cdn.example.com/x.glb").to_string(),
                "https://cdn.example.com/x.glb"
            );
        }

        #[test]
        fn protocol_relative_reference_resolves_against_https_base() {
            let uri = Uri::parse("//www.example.com");
            assert_eq!(
                uri.resolve("/page/test").to_string(),
                "https://www.example.com/page/test"
            );
        }

        #[test]
        fn unicode_reference_is_percent_encoded() {
            let uri = Uri::parse("https://www.example.com/");
            assert_eq!(
                uri.resolve("/Ῥόδος").to_string(),
                "https://www.example.com/%E1%BF%AC%CF%8C%CE%B4%CE%BF%CF%82"
            );
        }
    }

    // ==== Uri::resolve — Windows paths ===================================

    mod resolve_windows_path_base {
        use super::*;

        #[test]
        fn resolves_relative_reference() {
            let uri = Uri::parse(r"C:\data\tileset.json");
            assert_eq!(
                uri.resolve(r"content\1.glb").to_string(),
                r"C:\data\content\1.glb"
            );
        }

        #[test]
        fn accepts_forward_slash_references() {
            let uri = Uri::parse(r"C:\data\tileset.json");
            assert_eq!(
                uri.resolve("content/1.glb").to_string(),
                r"C:\data\content\1.glb"
            );
        }

        #[test]
        fn resolves_parent_directory_reference() {
            let uri = Uri::parse(r"C:\data\a\tileset.json");
            assert_eq!(
                uri.resolve(r"..\b\tile.json").to_string(),
                r"C:\data\b\tile.json"
            );
        }

        #[test]
        fn absolute_drive_reference_replaces_base() {
            let uri = Uri::parse(r"C:\data\tileset.json");
            assert_eq!(
                uri.resolve(r"D:\other\tile.json").to_string(),
                r"D:\other\tile.json"
            );
        }

        #[test]
        fn resolves_relative_reference_against_unc_share() {
            let uri = Uri::parse(
                r"\\qalab_server\vsdata\v36\Desktop\SceneLayers\data\GS\Cambridge\3d_tiles\tileset.json",
            );
            assert_eq!(
                uri.resolve(r"content\0\0.glb").to_string(),
                r"\\qalab_server\vsdata\v36\Desktop\SceneLayers\data\GS\Cambridge\3d_tiles\content\0\0.glb"
            );
        }

        #[test]
        fn resolves_parent_reference_against_unc_share() {
            let uri = Uri::parse(r"\\server\share\a\tileset.json");
            assert_eq!(
                uri.resolve(r"..\b\tile.json").to_string(),
                r"\\server\share\b\tile.json"
            );
        }
    }

    // ==== Uri::resolve — Unix paths ======================================

    mod resolve_unix_path_base {
        use super::*;

        #[test]
        fn resolves_relative_reference() {
            let uri = Uri::parse("/data/tileset.json");
            assert_eq!(
                uri.resolve("content/1.glb").to_string(),
                "/data/content/1.glb"
            );
        }

        #[test]
        fn resolves_parent_directory_reference() {
            let uri = Uri::parse("/data/a/tileset.json");
            assert_eq!(
                uri.resolve("../b/tile.json").to_string(),
                "/data/b/tile.json"
            );
        }

        #[test]
        fn absolute_reference_replaces_base() {
            let uri = Uri::parse("/data/tileset.json");
            assert_eq!(
                uri.resolve("/other/tile.json").to_string(),
                "/other/tile.json"
            );
        }

        #[test]
        fn resolves_relative_base_and_reference() {
            let uri = Uri::parse("data/tileset.json");
            assert_eq!(
                uri.resolve("content/1.glb").to_string(),
                "data/content/1.glb"
            );
        }

        #[test]
        fn empty_reference_returns_base_unchanged() {
            let uri = Uri::parse("/data/tileset.json");
            assert_eq!(uri.resolve("").to_string(), "/data/tileset.json");
        }
    }

    mod file_name_stem_extension {
        use super::*;

        #[test]
        fn url_typical_file() {
            let uri = Uri::parse("http://example.com/a/b/c/test.txt");
            assert_eq!(uri.file_name(), Some("test.txt"));
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), Some(".txt"));
        }

        #[test]
        fn url_root_has_no_file_name() {
            assert_eq!(Uri::parse("http://example.com/").file_name(), None);
            assert_eq!(Uri::parse("http://example.com/a/b/c/").file_name(), None);
        }

        #[test]
        fn url_trailing_dot_is_an_extension_of_just_dot() {
            let uri = Uri::parse("http://example.com/a/b/c/test.");
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), Some("."));
        }

        #[test]
        fn url_no_extension() {
            let uri = Uri::parse("http://example.com/a/b/c/test");
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), None);
        }

        #[test]
        fn url_last_segment_without_dot_is_whole_stem() {
            let uri = Uri::parse("http://example.com/a/b/c");
            assert_eq!(uri.file_name(), Some("c"));
            assert_eq!(uri.stem(), Some("c"));
            assert_eq!(uri.extension(), None);
        }

        #[test]
        fn url_dotfile_has_no_extension() {
            let uri = Uri::parse("http://example.com/a/b/c/.hidden");
            assert_eq!(uri.extension(), None);
        }

        #[test]
        fn url_dot_and_dotdot_segments_are_normalized_away() {
            // "." and ".." path segments are removed during URL parsing,
            // leaving a trailing slash (empty last segment): no file name.
            assert_eq!(Uri::parse("http://example.com/a/b/c/.").file_name(), None);
            assert_eq!(Uri::parse("http://example.com/a/b/c/..").file_name(), None);
        }

        #[test]
        fn windows_typical_file() {
            let uri = Uri::parse(r"C:\data\a\tile.b3dm");
            assert_eq!(uri.file_name(), Some("tile.b3dm"));
            assert_eq!(uri.stem(), Some("tile"));
            assert_eq!(uri.extension(), Some(".b3dm"));
        }

        #[test]
        fn windows_trailing_dot_is_an_extension_of_just_dot() {
            let uri = Uri::parse(r"C:\data\a\test.");
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), Some("."));
        }

        #[test]
        fn windows_no_extension() {
            let uri = Uri::parse(r"C:\data\a\test");
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), None);
        }

        #[test]
        fn windows_dotfile_has_no_extension() {
            let uri = Uri::parse(r"C:\data\.hidden");
            assert_eq!(uri.extension(), None);
            assert_eq!(uri.stem(), Some(".hidden"));
        }

        #[test]
        fn unix_typical_file() {
            let uri = Uri::parse("/data/a/tile.b3dm");
            assert_eq!(uri.file_name(), Some("tile.b3dm"));
            assert_eq!(uri.stem(), Some("tile"));
            assert_eq!(uri.extension(), Some(".b3dm"));
        }

        #[test]
        fn unix_trailing_dot_is_an_extension_of_just_dot() {
            let uri = Uri::parse("/data/a/test.");
            assert_eq!(uri.stem(), Some("test"));
            assert_eq!(uri.extension(), Some("."));
        }

        #[test]
        fn unix_no_extension() {
            let uri = Uri::parse("/data/README");
            assert_eq!(uri.stem(), Some("README"));
            assert_eq!(uri.extension(), None);
        }

        #[test]
        fn unix_dotfile_has_no_extension() {
            let uri = Uri::parse("/data/.hidden");
            assert_eq!(uri.extension(), None);
            assert_eq!(uri.stem(), Some(".hidden"));
        }
    }

    // ==== is_external_tileset_uri =======================================

    mod external_tileset_detection {
        use super::*;

        #[test]
        fn json_uri_is_external_tileset() {
            assert!(is_external_tileset_uri("tileset.json"));
            assert!(is_external_tileset_uri("https://example.com/a/b.JSON"));
            assert!(is_external_tileset_uri("tileset.json?query=1#frag"));
            assert!(is_external_tileset_uri("a/layer.tileset.gltf"));
            assert!(is_external_tileset_uri("a/layer.tileset.glb"));
        }

        #[test]
        fn non_json_uri_is_not_external_tileset() {
            assert!(!is_external_tileset_uri("content.glb"));
            assert!(!is_external_tileset_uri("content.b3dm"));
            assert!(!is_external_tileset_uri("content.gltf"));
            assert!(!is_external_tileset_uri("layer.gltf"));
        }
    }

    // ==== resolve_uri (public compatibility wrapper) =====================

    mod resolve_uri_wrapper {
        use super::*;

        #[test]
        fn resolves_http_references_with_url_semantics() {
            assert_eq!(
                resolve_uri("https://example.com/a/tileset.json", "../b/tile.json"),
                "https://example.com/b/tile.json"
            );
        }

        #[test]
        fn resolves_relative_reference_against_unc_path() {
            assert_eq!(
                resolve_uri(r"\\server\share\a\tileset.json", r"..\b\tile.json"),
                r"\\server\share\b\tile.json"
            );
        }

        #[test]
        fn resolves_relative_reference_against_windows_drive_path() {
            assert_eq!(
                resolve_uri(r"C:\data\tileset.json", "content/1.glb"),
                r"C:\data\content\1.glb"
            );
        }
    }
}
