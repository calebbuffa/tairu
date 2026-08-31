//! Parse `tileset.json` eagerly, or process it with one
//! bounded-memory streaming fold.

use crate::ext_mesh_features;
use crate::uri::{Uri, UriLoadError};
use crate::{Content, Tile, Tileset};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::io::Read;
use std::ops::ControlFlow;

/// Error returned when parsing a tileset.
#[derive(Debug, thiserror::Error)]
pub enum TileParseError {
    /// The input could not be read.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The JSON payload could not be parsed.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    /// The tileset failed schema validation.
    #[error("validation error: {0}")]
    Validation(String),
}

/// Capability for streaming processing of a tileset.
///
/// This is the single extension point for traversal, inspection, and mutation
/// logic in streaming mode.
pub trait TilesetFold {
    /// Called for each top-level tileset field except `root`.
    fn on_tileset_field(&mut self, _key: &str, _value: &JsonValue) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    /// Called for each tile with traversal depth (`0` for root).
    ///
    /// In streaming mode, each callback receives one tile at a time. The tile's
    /// `children` field is left empty to keep memory bounded.
    fn on_tile(&mut self, _tile: &mut Tile, _depth: usize) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

/// Fetches and parses a tileset through a caller-owned closure.
pub fn load<F, E>(uri: impl Into<Uri>, fetch: &mut F) -> Result<Tileset, UriLoadError<E>>
where
    F: FnMut(&str) -> Result<Vec<u8>, E>,
    E: std::error::Error + 'static,
{
    let uri = uri.into();
    let bytes = fetch(&uri.to_string()).map_err(|source| UriLoadError::Fetch {
        uri: uri.to_string(),
        source,
    })?;
    from_slice(&bytes).map_err(|source| UriLoadError::Parse {
        uri: uri.to_string(),
        source,
    })
}

/// Asynchronously fetches and parses a tileset through a caller-owned closure.
pub async fn load_async<F, Fut, E>(
    uri: impl Into<Uri>,
    fetch: &mut F,
) -> Result<Tileset, UriLoadError<E>>
where
    F: FnMut(&str) -> Fut,
    Fut: Future<Output = Result<Vec<u8>, E>>,
    E: std::error::Error + 'static,
{
    let uri = uri.into();
    let bytes = fetch(&uri.to_string())
        .await
        .map_err(|source| UriLoadError::Fetch {
            uri: uri.to_string(),
            source,
        })?;
    from_slice(&bytes).map_err(|source| UriLoadError::Parse {
        uri: uri.to_string(),
        source,
    })
}

/// Parses and validates a tileset from a raw JSON byte slice.
pub fn from_slice(data: &[u8]) -> Result<Tileset, TileParseError> {
    let tileset = serde_json::from_slice::<Tileset>(data)?;
    validate(&tileset)?;
    Ok(tileset)
}

/// Parses and validates a tileset from a JSON string slice.
pub fn from_str(s: &str) -> Result<Tileset, TileParseError> {
    from_slice(s.as_bytes())
}

/// Parses and validates a tileset from any reader.
pub fn from_reader(reader: impl Read) -> Result<Tileset, TileParseError> {
    let tileset = serde_json::from_reader::<_, Tileset>(reader)?;
    validate(&tileset)?;
    Ok(tileset)
}

/// Folds over a tileset stream with bounded memory.
pub fn fold_from_reader(
    reader: impl Read,
    fold: &mut impl TilesetFold,
) -> Result<usize, TileParseError> {
    let mut de = serde_json::Deserializer::from_reader(reader);
    let state = RefCell::new(FoldState {
        fold,
        tile_count: 0,
        stopped: false,
    });

    TilesetSeed { state: &state }.deserialize(&mut de)?;
    Ok(state.borrow().tile_count)
}

struct FoldState<'a, F>
where
    F: TilesetFold,
{
    fold: &'a mut F,
    tile_count: usize,
    stopped: bool,
}

impl<'a, F> FoldState<'a, F>
where
    F: TilesetFold,
{
    fn emit_tileset_field(&mut self, key: &str, value: &JsonValue) {
        if self.stopped {
            return;
        }
        if let ControlFlow::Break(()) = self.fold.on_tileset_field(key, value) {
            self.stopped = true;
        }
    }

    fn emit_tile(&mut self, tile: &mut Tile, depth: usize) {
        if self.stopped {
            return;
        }
        self.tile_count += 1;
        if let ControlFlow::Break(()) = self.fold.on_tile(tile, depth) {
            self.stopped = true;
        }
    }
}

struct TilesetSeed<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
}

impl<'de, 'a, F> DeserializeSeed<'de> for TilesetSeed<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TilesetVisitor { state: self.state })
    }
}

struct TilesetVisitor<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
}

impl<'de, 'a, F> Visitor<'de> for TilesetVisitor<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a 3D Tiles tileset object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut saw_asset = false;
        let mut saw_root = false;
        let mut saw_geometric_error = false;
        let mut extensions_required: Option<Vec<String>> = None;
        let mut extensions_used: Option<Vec<String>> = None;

        while let Some(key) = map.next_key::<String>()? {
            if self.state.borrow().stopped {
                map.next_value::<IgnoredAny>()?;
                continue;
            }

            match key.as_str() {
                "asset" => {
                    saw_asset = true;
                    let value = map.next_value::<JsonValue>()?;
                    self.state.borrow_mut().emit_tileset_field("asset", &value);

                    let asset: AssetVersionOnly =
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                    if asset.version.is_empty() {
                        return Err(serde::de::Error::custom(
                            "asset.version is required and must not be empty",
                        ));
                    }
                }
                "geometricError" => {
                    saw_geometric_error = true;
                    let value = map.next_value::<JsonValue>()?;
                    self.state
                        .borrow_mut()
                        .emit_tileset_field("geometricError", &value);

                    match value.as_f64() {
                        Some(v) => {
                            if v < 0.0 {
                                log::warn!("geometricError is negative ({}); expected >= 0", v);
                            }
                        }
                        None => {
                            return Err(serde::de::Error::custom(
                                "geometricError must be a number",
                            ));
                        }
                    }
                }
                "extensionsRequired" => {
                    let value = map.next_value::<JsonValue>()?;
                    self.state
                        .borrow_mut()
                        .emit_tileset_field("extensionsRequired", &value);
                    let required: Vec<String> =
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                    extensions_required = Some(required);
                }
                "extensionsUsed" => {
                    let value = map.next_value::<JsonValue>()?;
                    self.state
                        .borrow_mut()
                        .emit_tileset_field("extensionsUsed", &value);
                    let used: Vec<String> =
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                    extensions_used = Some(used);
                }
                "root" => {
                    saw_root = true;
                    map.next_value_seed(TileSeed {
                        state: self.state,
                        is_root: true,
                        depth: 0,
                    })?;
                }
                _ => {
                    let value = map.next_value::<JsonValue>()?;
                    self.state.borrow_mut().emit_tileset_field(&key, &value);
                }
            }
        }

        if !saw_asset {
            return Err(serde::de::Error::custom("asset is required"));
        }

        if !saw_root {
            return Err(serde::de::Error::custom("root is required"));
        }

        if !saw_geometric_error {
            return Err(serde::de::Error::custom("geometricError is required"));
        }

        if let (Some(required), Some(used)) = (extensions_required, extensions_used) {
            for ext in required {
                if !used.contains(&ext) {
                    log::warn!(
                        "extensionsRequired contains '{}' which is not listed in extensionsUsed",
                        ext
                    );
                }
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct AssetVersionOnly {
    version: String,
}

struct TileSeed<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
    is_root: bool,
    depth: usize,
}

impl<'de, 'a, F> DeserializeSeed<'de> for TileSeed<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TileVisitor {
            state: self.state,
            is_root: self.is_root,
            depth: self.depth,
        })
    }
}

struct TileVisitor<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
    is_root: bool,
    depth: usize,
}

impl<'de, 'a, F> Visitor<'de> for TileVisitor<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a tile object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut tile = Tile::default();
        let mut saw_bounding_volume = false;
        let mut saw_geometric_error = false;

        while let Some(key) = map.next_key::<String>()? {
            if self.state.borrow().stopped {
                map.next_value::<IgnoredAny>()?;
                continue;
            }

            match key.as_str() {
                "boundingVolume" => {
                    saw_bounding_volume = true;
                    tile.bounding_volume = map.next_value()?;
                }
                "children" => {
                    map.next_value_seed(ChildrenSeed {
                        state: self.state,
                        child_depth: self.depth + 1,
                    })?;
                }
                "content" => {
                    let raw = map.next_value::<JsonValue>()?;
                    tile.content =
                        Some(parse_content_compat_value(raw).map_err(serde::de::Error::custom)?);
                }
                "contents" => {
                    let raw_list = map.next_value::<Vec<JsonValue>>()?;
                    let mut contents = Vec::with_capacity(raw_list.len());
                    for raw in raw_list {
                        contents.push(
                            parse_content_compat_value(raw).map_err(serde::de::Error::custom)?,
                        );
                    }
                    tile.contents = contents;
                }
                "geometricError" => {
                    saw_geometric_error = true;
                    tile.geometric_error = map.next_value::<f64>()?;
                    if tile.geometric_error < 0.0 {
                        if self.is_root {
                            log::warn!(
                                "root.geometricError is negative ({}); expected >= 0",
                                tile.geometric_error
                            );
                        } else {
                            log::warn!(
                                "tile.geometricError is negative ({}); expected >= 0",
                                tile.geometric_error
                            );
                        }
                    }
                }
                "implicitTiling" => {
                    tile.implicit_tiling = Some(map.next_value()?);
                }
                "metadata" => {
                    tile.metadata = Some(map.next_value()?);
                }
                "refine" => {
                    tile.refine = Some(map.next_value()?);
                }
                "transform" => {
                    tile.transform = map.next_value()?;
                }
                "viewerRequestVolume" => {
                    tile.viewer_request_volume = Some(map.next_value()?);
                }
                "extensions" => {
                    tile.extensions = map.next_value::<HashMap<String, JsonValue>>()?;
                }
                "extras" => {
                    tile.extras = Some(map.next_value()?);
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        if self.is_root && !saw_bounding_volume {
            return Err(serde::de::Error::custom("root.boundingVolume is required"));
        }

        if self.is_root && !saw_geometric_error {
            return Err(serde::de::Error::custom("root.geometricError is required"));
        }

        self.state.borrow_mut().emit_tile(&mut tile, self.depth);
        Ok(())
    }
}

struct ChildrenSeed<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
    child_depth: usize,
}

impl<'de, 'a, F> DeserializeSeed<'de> for ChildrenSeed<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChildrenVisitor {
            state: self.state,
            child_depth: self.child_depth,
        })
    }
}

struct ChildrenVisitor<'a, F>
where
    F: TilesetFold,
{
    state: &'a RefCell<FoldState<'a, F>>,
    child_depth: usize,
}

impl<'de, 'a, F> Visitor<'de> for ChildrenVisitor<'a, F>
where
    F: TilesetFold,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a tile children array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while !self.state.borrow().stopped
            && (seq.next_element_seed(TileSeed {
                state: self.state,
                is_root: false,
                depth: self.child_depth,
            })?)
            .is_some()
        {}
        Ok(())
    }
}

fn parse_content_compat_value(value: JsonValue) -> Result<Content, String> {
    let mut object = match value {
        JsonValue::Object(map) => map,
        _ => return Err("content entry must be an object".to_string()),
    };

    if !object.contains_key("uri")
        && let Some(JsonValue::String(url)) = object.get("url")
    {
        object.insert("uri".to_string(), JsonValue::String(url.clone()));
    }

    serde_json::from_value::<Content>(JsonValue::Object(object)).map_err(|e| e.to_string())
}

/// Validate a successfully-parsed [`Tileset`], returning fatal errors and emitting
/// non-fatal issues as warnings.
fn validate(tileset: &Tileset) -> Result<(), TileParseError> {
    if tileset.asset.version.is_empty() {
        return Err(TileParseError::Validation(
            "asset.version is required and must not be empty".into(),
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

        if ext == ext_mesh_features::EXTENSION_NAME
            && let Some(content) = &tileset.root.content
            && let Some(value) = content.extensions.get(ext_mesh_features::EXTENSION_NAME)
            && let Some(emf) = ext_mesh_features::ExtMeshFeatures::from_json(value)
        {
            for fid in &emf.feature_ids {
                if fid.feature_count == 0 {
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
    fn uri_loading_fetches_the_requested_uri() {
        let mut requested = None;
        let mut fetch = |uri: &str| -> Result<Vec<u8>, std::io::Error> {
            requested = Some(uri.to_string());
            Ok(minimal_json().to_vec())
        };

        let tileset = load("memory://tileset.json", &mut fetch).unwrap();

        assert_eq!(requested.as_deref(), Some("memory://tileset.json"));
        assert_eq!(tileset.geometric_error, 100.0);
    }

    struct CaptureFold {
        fields: Vec<String>,
        depths: Vec<usize>,
        uris: Vec<String>,
    }

    impl TilesetFold for CaptureFold {
        fn on_tileset_field(&mut self, key: &str, _value: &JsonValue) -> ControlFlow<()> {
            self.fields.push(key.to_string());
            ControlFlow::Continue(())
        }

        fn on_tile(&mut self, tile: &mut Tile, depth: usize) -> ControlFlow<()> {
            self.depths.push(depth);
            if let Some(content) = &tile.content {
                self.uris.push(content.uri.clone());
            }
            ControlFlow::Continue(())
        }
    }

    #[test]
    fn parses_minimal_tileset() {
        let ts = from_slice(minimal_json()).expect("should parse");
        assert_eq!(ts.asset.version, "1.1");
        assert_eq!(ts.geometric_error, 100.0);
    }

    #[test]
    fn warns_on_negative_geometric_error() {
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
    fn errors_on_invalid_json() {
        let err = from_slice(b"not json");
        assert!(matches!(err, Err(TileParseError::Json(_))));
    }

    #[test]
    fn errors_on_empty_version() {
        let json = br#"{
            "asset": { "version": "" },
            "geometricError": 0.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.0
            }
        }"#;
        let err = from_slice(json);
        assert!(matches!(err, Err(TileParseError::Validation(_))));
    }

    #[test]
    fn fold_exposes_tileset_fields_and_tiles() {
        let json = br#"{
            "asset": { "version": "1.1" },
            "geometricError": 100.0,
            "metadata": { "name": "city" },
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1000] },
                "geometricError": 100.0,
                "children": [
                    {
                        "boundingVolume": { "sphere": [0, 0, 0, 100] },
                        "geometricError": 10.0,
                        "content": { "uri": "child_a.b3dm" }
                    }
                ]
            }
        }"#;

        let mut fold = CaptureFold {
            fields: Vec::new(),
            depths: Vec::new(),
            uris: Vec::new(),
        };

        let count = fold_from_reader(json.as_slice(), &mut fold).expect("fold should succeed");

        assert_eq!(count, 2);
        assert!(fold.fields.iter().any(|k| k == "asset"));
        assert!(fold.fields.iter().any(|k| k == "metadata"));
        assert!(fold.fields.iter().any(|k| k == "geometricError"));
        assert_eq!(fold.depths, vec![1, 0]);
        assert_eq!(fold.uris, vec!["child_a.b3dm".to_string()]);
    }

    #[test]
    fn fold_can_stop_early() {
        struct StopAfterTwo {
            seen: usize,
        }

        impl TilesetFold for StopAfterTwo {
            fn on_tile(&mut self, _tile: &mut Tile, _depth: usize) -> ControlFlow<()> {
                self.seen += 1;
                if self.seen >= 2 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        }

        let json = br#"{
            "asset": { "version": "1.1" },
            "geometricError": 100.0,
            "root": {
                "boundingVolume": { "sphere": [0, 0, 0, 1000] },
                "geometricError": 100.0,
                "children": [
                    {
                        "boundingVolume": { "sphere": [0, 0, 0, 100] },
                        "geometricError": 10.0
                    },
                    {
                        "boundingVolume": { "sphere": [0, 0, 0, 100] },
                        "geometricError": 10.0
                    }
                ]
            }
        }"#;

        let mut fold = StopAfterTwo { seen: 0 };
        let count = fold_from_reader(json.as_slice(), &mut fold).expect("fold should succeed");

        assert_eq!(count, 2);
        assert_eq!(fold.seen, 2);
    }
}
