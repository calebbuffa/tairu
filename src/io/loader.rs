//! kiba loader implementation for 3D Tiles.
//!
//! # Composition model
//!
//! [`TilesetLoader`] composes three concrete, internal loader states as
//! plain structs/enums rather than routing "child loading" through a
//! generic kiba trait:
//!
//! - [`DocumentState`] retains a fully parsed [`crate::Tileset`] (asset,
//!   schema, and the whole tile tree) for as long as any [`TileRef`]
//!   references it, instead of extracting the root [`crate::Tile`] and
//!   discarding the rest of the document.
//! - [`ExternalTilesetLoader`] fetches and caches the [`DocumentState`] for
//!   the loader's root and for every external tileset referenced by a
//!   content URI, so a manifest is fetched and parsed at most once no
//!   matter how many content references point at it.
//! - [`ImplicitQuadtreeLoaderState`] / [`ImplicitOctreeLoaderState`] retain
//!   the availability tree accumulated while descending an implicitly
//!   tiled subtree, so a subtree file already fetched is never re-fetched
//!   or re-parsed by a later expansion.
//!
//! [`TileRef`] is a lightweight, cheaply [`Clone`]able handle into this
//! retained state (an `Arc` plus either a small tile-tree path or an
//! implicit tile id) - expanding it dispatches through the retained state
//! rather than re-parsing a document or rebuilding implicit loader state.

use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use ::kiba::{
    ExpandFuture, Expansion, Fetch, FetchRequest, LoadFuture, LoadOutcome, Loader, RootFuture,
};

use crate::Error;

// ---------------------------------------------------------------------
// Document state: retained parsed tileset documents.
// ---------------------------------------------------------------------

/// A fully parsed tileset document, retained for as long as any
/// [`TileRef`] still references it.
///
/// Earlier revisions parsed a [`crate::Tileset`] only to immediately
/// discard everything but its root [`crate::Tile`]. Retaining the whole
/// document behind an `Arc` keeps asset/schema/metadata available and lets
/// [`TileRef`] address any tile in the tree by path instead of owning a
/// clone of it.
struct DocumentState {
    /// The document's resolved source URI (its tileset.json address).
    source_uri: crate::Uri,
    /// The fully parsed tileset document.
    tileset: crate::Tileset,
    /// Implicit-tiling loader state, cached by the path of the tile that
    /// declared `implicitTiling`, so it is built once and reused by every
    /// expansion of that subtree rather than being recreated.
    implicit_loaders: Mutex<HashMap<Vec<usize>, ImplicitLoaderState>>,
}

impl std::fmt::Debug for DocumentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentState")
            .field("source_uri", &self.source_uri)
            .finish_non_exhaustive()
    }
}

impl DocumentState {
    /// Navigates to the tile at `path` (the root tile's path is empty).
    fn tile_at(&self, path: &[usize]) -> &crate::Tile {
        let mut node = &self.tileset.root;
        for &index in path {
            node = &node.children[index];
        }
        node
    }

    /// Returns the retained implicit-tiling loader state for the tile at
    /// `path`, or `None` if that tile does not declare implicit tiling (or
    /// declares an unsupported subdivision scheme).
    ///
    /// The state is created once per path and cached; subsequent calls -
    /// whether from re-expanding the same explicit tile or from expanding
    /// its implicit descendants - reuse the same `Arc`, so accumulated
    /// availability data and already-fetched subtree files are never
    /// rebuilt or re-fetched.
    fn implicit_loader(&self, path: &[usize], tile: &crate::Tile) -> Option<ImplicitLoaderState> {
        let implicit = tile.implicit_tiling.as_ref()?;
        let mut loaders = self.implicit_loaders.lock().unwrap();
        if let Some(state) = loaders.get(path) {
            return Some(state.clone());
        }
        let state = match implicit.subdivision_scheme {
            crate::SubdivisionScheme::Quadtree => {
                ImplicitQuadtreeLoaderState::new(&self.source_uri, tile, implicit)
                    .map(ImplicitLoaderState::Quadtree)
            }
            crate::SubdivisionScheme::Octree => {
                ImplicitOctreeLoaderState::new(&self.source_uri, tile, implicit)
                    .map(ImplicitLoaderState::Octree)
            }
            crate::SubdivisionScheme::S2 => None,
        }?;
        loaders.insert(path.to_vec(), state.clone());
        Some(state)
    }
}

/// Fetches raw resources and caches the parsed [`DocumentState`] for
/// tileset manifests (the loader's root, and any external tileset
/// referenced by a content URI).
///
/// A manifest is fetched and parsed at most once per resolved URI: the
/// resulting [`DocumentState`] is retained in `cache` and handed out via
/// `Arc` to every subsequent request for the same URI.
struct ExternalTilesetLoader {
    fetch: Fetch,
    cache: Mutex<HashMap<String, Arc<DocumentState>>>,
}

impl ExternalTilesetLoader {
    fn new(fetch: Fetch) -> Self {
        Self {
            fetch,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Fetches the raw bytes at `uri`, performing no caching or parsing.
    async fn fetch_bytes(&self, uri: impl Into<String>) -> Result<Vec<u8>, Error> {
        let uri = uri.into();
        let response = (self.fetch)(FetchRequest {
            uri: uri.clone(),
            range: None,
        })
        .await
        .map_err(|source| Error::fetch(uri.clone(), source))?;
        Ok(response.bytes)
    }

    /// Returns the retained [`DocumentState`] for the tileset manifest at
    /// `uri`, fetching and parsing it only on the first request.
    async fn load_document(&self, uri: crate::Uri) -> Result<Arc<DocumentState>, Error> {
        let key = uri.to_string();
        if let Some(document) = self.cache.lock().unwrap().get(&key).cloned() {
            return Ok(document);
        }
        let bytes = self.fetch_bytes(key.clone()).await?;
        let tileset = crate::from_slice(&bytes)
            .map_err(|source| Error::parse(key.clone(), source.to_string()))?;
        let document = Arc::new(DocumentState {
            source_uri: uri,
            tileset,
            implicit_loaders: Mutex::new(HashMap::new()),
        });
        self.cache.lock().unwrap().insert(key, document.clone());
        Ok(document)
    }
}

/// Which concrete implicit-tiling loader state governs a subtree.
#[derive(Clone)]
enum ImplicitLoaderState {
    /// State for a `QUADTREE` subdivision scheme.
    Quadtree(Arc<ImplicitQuadtreeLoaderState>),
    /// State for an `OCTREE` subdivision scheme.
    Octree(Arc<ImplicitOctreeLoaderState>),
}

/// Retained state for one implicitly-tiled quadtree subtree.
///
/// Created once for the explicit tile that declares `implicitTiling` and
/// shared (via `Arc`) by every descendant node produced while expanding
/// it, so the accumulated [`crate::QuadtreeAvailability`] tree - and the
/// subtree files already fetched into it - persist across expansions
/// instead of being rebuilt on every call.
struct ImplicitQuadtreeLoaderState {
    source_uri: crate::Uri,
    root_bounding_volume: crate::BoundingVolume,
    root_geometric_error: f64,
    content_uri_template: String,
    subtree_uri_template: String,
    subtree_levels: u32,
    available_levels: u32,
    availability: Mutex<crate::QuadtreeAvailability>,
}

impl std::fmt::Debug for ImplicitQuadtreeLoaderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImplicitQuadtreeLoaderState")
            .field("subtree_levels", &self.subtree_levels)
            .field("available_levels", &self.available_levels)
            .finish_non_exhaustive()
    }
}

impl ImplicitQuadtreeLoaderState {
    fn new(
        source_uri: &crate::Uri,
        tile: &crate::Tile,
        implicit: &crate::ImplicitTiling,
    ) -> Option<Arc<Self>> {
        let content_uri_template = tile
            .content
            .as_ref()
            .map(|content| content.uri.clone())
            .or_else(|| tile.contents.first().map(|content| content.uri.clone()))?;
        let subtree_levels = implicit.subtree_levels as u32;
        let available_levels = implicit.available_levels as u32;
        Some(Arc::new(Self {
            source_uri: source_uri.clone(),
            root_bounding_volume: tile.bounding_volume.clone(),
            root_geometric_error: tile.geometric_error,
            content_uri_template,
            subtree_uri_template: implicit.subtrees.uri.clone(),
            subtree_levels,
            available_levels,
            availability: Mutex::new(crate::QuadtreeAvailability::new(
                subtree_levels,
                available_levels,
            )),
        }))
    }

    fn flags(&self, id: crate::QuadtreeTileId) -> crate::TileAvailabilityFlags {
        self.availability.lock().unwrap().compute_availability(id)
    }

    fn subtree_uri(&self, subtree_root: crate::QuadtreeTileId) -> crate::Uri {
        subtree_root.resolve_url(&self.source_uri, &self.subtree_uri_template)
    }

    fn content_uri(&self, id: crate::QuadtreeTileId) -> crate::Uri {
        id.resolve_url(&self.source_uri, &self.content_uri_template)
    }

    fn geometric_error(&self, id: crate::QuadtreeTileId) -> f64 {
        self.root_geometric_error / 2.0_f64.powi(id.level as i32)
    }

    fn register_subtree(
        &self,
        subtree_root: crate::QuadtreeTileId,
        data: &[u8],
        uri: &crate::Uri,
    ) -> Result<(), Error> {
        let availability = crate::SubtreeAvailability::from_bytes(
            data,
            crate::SubdivisionScheme::Quadtree,
            self.subtree_levels,
        )
        .map_err(|source| Error::parse(uri.to_string(), source.to_string()))?;
        self.availability
            .lock()
            .unwrap()
            .add_subtree(subtree_root, availability);
        Ok(())
    }
}

/// Retained state for one implicitly-tiled octree subtree. See
/// [`ImplicitQuadtreeLoaderState`] for the retention rationale.
struct ImplicitOctreeLoaderState {
    source_uri: crate::Uri,
    root_bounding_volume: crate::BoundingVolume,
    root_geometric_error: f64,
    content_uri_template: String,
    subtree_uri_template: String,
    subtree_levels: u32,
    available_levels: u32,
    availability: Mutex<crate::OctreeAvailability>,
}

impl std::fmt::Debug for ImplicitOctreeLoaderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImplicitOctreeLoaderState")
            .field("subtree_levels", &self.subtree_levels)
            .field("available_levels", &self.available_levels)
            .finish_non_exhaustive()
    }
}

impl ImplicitOctreeLoaderState {
    fn new(
        source_uri: &crate::Uri,
        tile: &crate::Tile,
        implicit: &crate::ImplicitTiling,
    ) -> Option<Arc<Self>> {
        let content_uri_template = tile
            .content
            .as_ref()
            .map(|content| content.uri.clone())
            .or_else(|| tile.contents.first().map(|content| content.uri.clone()))?;
        let subtree_levels = implicit.subtree_levels as u32;
        let available_levels = implicit.available_levels as u32;
        Some(Arc::new(Self {
            source_uri: source_uri.clone(),
            root_bounding_volume: tile.bounding_volume.clone(),
            root_geometric_error: tile.geometric_error,
            content_uri_template,
            subtree_uri_template: implicit.subtrees.uri.clone(),
            subtree_levels,
            available_levels,
            availability: Mutex::new(crate::OctreeAvailability::new(
                subtree_levels,
                available_levels,
            )),
        }))
    }

    fn flags(&self, id: crate::OctreeTileId) -> crate::TileAvailabilityFlags {
        self.availability.lock().unwrap().compute_availability(id)
    }

    fn subtree_uri(&self, subtree_root: crate::OctreeTileId) -> crate::Uri {
        subtree_root.resolve_url(&self.source_uri, &self.subtree_uri_template)
    }

    fn content_uri(&self, id: crate::OctreeTileId) -> crate::Uri {
        id.resolve_url(&self.source_uri, &self.content_uri_template)
    }

    fn geometric_error(&self, id: crate::OctreeTileId) -> f64 {
        self.root_geometric_error / 2.0_f64.powi(id.level as i32)
    }

    fn register_subtree(
        &self,
        subtree_root: crate::OctreeTileId,
        data: &[u8],
        uri: &crate::Uri,
    ) -> Result<(), Error> {
        let availability = crate::SubtreeAvailability::from_bytes(
            data,
            crate::SubdivisionScheme::Octree,
            self.subtree_levels,
        )
        .map_err(|source| Error::parse(uri.to_string(), source.to_string()))?;
        self.availability
            .lock()
            .unwrap()
            .add_subtree(subtree_root, availability);
        Ok(())
    }
}

/// Which retained state a [`TileRef`] addresses, and how.
#[derive(Clone, Debug)]
enum Node {
    /// A tile taken directly from a parsed tileset document's explicit
    /// tree, addressed by its path of child indices from the document
    /// root (empty for the root tile itself).
    Document {
        document: Arc<DocumentState>,
        path: Vec<usize>,
    },
    /// A tile synthesized from quadtree availability data.
    ImplicitQuadtree {
        state: Arc<ImplicitQuadtreeLoaderState>,
        id: crate::QuadtreeTileId,
    },
    /// A tile synthesized from octree availability data.
    ImplicitOctree {
        state: Arc<ImplicitOctreeLoaderState>,
        id: crate::OctreeTileId,
    },
}

#[derive(Clone)]
/// A lightweight, stable handle to a demand-loaded Tairu tile.
///
/// `TileRef` never owns a copy of tile data; it addresses state retained
/// by [`TilesetLoader`] (a parsed document, or accumulated implicit-tiling
/// availability) via a cheap `Arc` clone plus a small path or tile id.
pub struct TileRef {
    node: Node,
    depth: usize,
    external_root: bool,
}

impl std::fmt::Debug for TileRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("TileRef");
        match &self.node {
            Node::Document { document, path } => s
                .field("source_uri", &document.source_uri)
                .field("path", path),
            Node::ImplicitQuadtree { id, .. } => s.field("quadtree_id", id),
            Node::ImplicitOctree { id, .. } => s.field("octree_id", id),
        };
        s.field("depth", &self.depth).finish()
    }
}

#[derive(Debug, Clone)]
/// Raw content loaded for a Tairu content reference.
pub struct LoadedContent {
    /// Resolved content URI.
    pub uri: String,
    /// Content bytes.
    pub bytes: Vec<u8>,
    /// Detected tile content format.
    pub format: crate::TileFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A resolved content reference belonging to a Tairu tile.
pub struct ContentRef {
    /// Resolved content URI.
    pub uri: String,
    /// Detected tile content format.
    pub format: crate::TileFormat,
}

/// kiba [`Loader`] implementation for 3D Tiles.
pub struct TilesetLoader {
    root_uri: crate::Uri,
    documents: Arc<ExternalTilesetLoader>,
}

impl TilesetLoader {
    /// Creates a loader without fetching the root resource.
    pub fn open<F, Fut>(root_uri: impl Into<crate::Uri>, fetch: F) -> Self
    where
        F: Fn(FetchRequest) -> Fut + Send + Sync + 'static,
        Fut:
            Future<Output = Result<::kiba::FetchResponse, ::kiba::FetchError>> + Send + 'static,
    {
        let fetch: Fetch = Arc::new(move |request| Box::pin(fetch(request)));
        Self {
            root_uri: root_uri.into(),
            documents: Arc::new(ExternalTilesetLoader::new(fetch)),
        }
    }

    pub(crate) fn visit_data(&self, item: &TileRef) -> (crate::Tile, crate::Uri, usize, bool) {
        match &item.node {
            Node::Document { document, path } => (
                document.tile_at(path).clone(),
                document.source_uri.clone(),
                item.depth,
                item.external_root,
            ),
            Node::ImplicitQuadtree { state, id } => (
                crate::Tile {
                    bounding_volume: id.subdivide_bounding_volume(&state.root_bounding_volume),
                    geometric_error: state.geometric_error(*id),
                    content: Some(crate::Content {
                        uri: state.content_uri(*id).to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                state.source_uri.clone(),
                item.depth,
                false,
            ),
            Node::ImplicitOctree { state, id } => (
                crate::Tile {
                    bounding_volume: id.subdivide_bounding_volume(&state.root_bounding_volume),
                    geometric_error: state.geometric_error(*id),
                    content: Some(crate::Content {
                        uri: state.content_uri(*id).to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                state.source_uri.clone(),
                item.depth,
                false,
            ),
        }
    }
}

impl Loader for TilesetLoader {
    type Item = TileRef;
    type ContentRef = ContentRef;
    type Content = LoadedContent;
    type Error = Error;

    fn root(&self) -> RootFuture<Self::Item, Self::Error> {
        let root_uri = self.root_uri.clone();
        let documents = self.documents.clone();
        Box::pin(async move {
            let document = documents.load_document(root_uri).await?;
            Ok(TileRef {
                node: Node::Document {
                    document,
                    path: Vec::new(),
                },
                depth: 0,
                external_root: false,
            })
        })
    }

    fn expand(&self, item: Self::Item) -> ExpandFuture<Self::Item, Self::ContentRef, Self::Error> {
        let documents = self.documents.clone();
        Box::pin(async move {
            match item.node {
                Node::Document { document, path } => {
                    expand_document_tile(documents, document, path, item.depth).await
                }
                Node::ImplicitQuadtree { state, id } => {
                    expand_implicit_quadtree(&documents, &state, id, item.depth).await
                }
                Node::ImplicitOctree { state, id } => {
                    expand_implicit_octree(&documents, &state, id, item.depth).await
                }
            }
        })
    }

    fn load(&self, content: Self::ContentRef) -> LoadFuture<Self::Content, Self::Error> {
        let documents = self.documents.clone();
        Box::pin(async move {
            let uri = content.uri;
            let bytes = documents.fetch_bytes(uri.clone()).await?;
            Ok(LoadOutcome::Ready(LoadedContent {
                uri,
                bytes,
                format: content.format,
            }))
        })
    }
}

/// Expands a tile taken directly from a parsed tileset document.
///
/// Dispatches to the retained implicit-tiling loader state when the tile
/// declares `implicitTiling`; otherwise enumerates its explicit children
/// and content, fetching/parsing external tilesets through the shared,
/// caching [`ExternalTilesetLoader`].
async fn expand_document_tile(
    documents: Arc<ExternalTilesetLoader>,
    document: Arc<DocumentState>,
    path: Vec<usize>,
    depth: usize,
) -> Result<Expansion<TileRef, ContentRef>, Error> {
    let tile = document.tile_at(&path);

    if let Some(state) = document.implicit_loader(&path, tile) {
        return match state {
            ImplicitLoaderState::Quadtree(state) => {
                let id = crate::QuadtreeTileId::new(0, 0, 0);
                expand_implicit_quadtree(&documents, &state, id, depth).await
            }
            ImplicitLoaderState::Octree(state) => {
                let id = crate::OctreeTileId::new(0, 0, 0, 0);
                expand_implicit_octree(&documents, &state, id, depth).await
            }
        };
    }

    let mut children = Vec::with_capacity(tile.children.len());
    for index in 0..tile.children.len() {
        let mut child_path = path.clone();
        child_path.push(index);
        children.push(TileRef {
            node: Node::Document {
                document: document.clone(),
                path: child_path,
            },
            depth: depth + 1,
            external_root: false,
        });
    }

    let mut contents = Vec::new();
    for content in tile.content.iter().chain(tile.contents.iter()) {
        let uri = document.source_uri.resolve(&content.uri);
        if crate::is_external_tileset_uri(&content.uri) {
            let external_document = documents.load_document(uri).await?;
            children.push(TileRef {
                node: Node::Document {
                    document: external_document,
                    path: Vec::new(),
                },
                depth: depth + 1,
                external_root: true,
            });
        } else {
            contents.push(ContentRef {
                format: crate::TileFormat::detect(&uri.to_string(), &[]),
                uri: uri.to_string(),
            });
        }
    }

    Ok(Expansion { children, contents })
}

/// Expands one node of an implicitly-tiled quadtree, fetching and
/// registering its subtree file with the retained `state` only if it has
/// not already been loaded.
async fn expand_implicit_quadtree(
    documents: &ExternalTilesetLoader,
    state: &Arc<ImplicitQuadtreeLoaderState>,
    id: crate::QuadtreeTileId,
    depth: usize,
) -> Result<Expansion<TileRef, ContentRef>, Error> {
    let flags = loop {
        let flags = state.flags(id);
        if flags.contains(crate::TileAvailabilityFlags::SUBTREE_AVAILABLE)
            && !flags.contains(crate::TileAvailabilityFlags::SUBTREE_LOADED)
        {
            let subtree_root = id.subtree_root(state.subtree_levels);
            let uri = state.subtree_uri(subtree_root);
            let bytes = documents.fetch_bytes(uri.to_string()).await?;
            state.register_subtree(subtree_root, &bytes, &uri)?;
            continue;
        }
        break flags;
    };

    if !flags.contains(crate::TileAvailabilityFlags::TILE_AVAILABLE)
        || id.level + 1 >= state.available_levels
    {
        return Ok(Expansion {
            children: Vec::new(),
            contents: Vec::new(),
        });
    }

    let mut children = Vec::new();
    let mut contents = Vec::new();
    for child_id in id.children() {
        let child_flags = state.flags(child_id);
        if !child_flags.contains(crate::TileAvailabilityFlags::TILE_AVAILABLE) {
            continue;
        }
        if child_flags.contains(crate::TileAvailabilityFlags::CONTENT_AVAILABLE) {
            let uri = state.content_uri(child_id);
            contents.push(ContentRef {
                format: crate::TileFormat::detect(&uri.to_string(), &[]),
                uri: uri.to_string(),
            });
        }
        children.push(TileRef {
            node: Node::ImplicitQuadtree {
                state: state.clone(),
                id: child_id,
            },
            depth: depth + 1,
            external_root: false,
        });
    }
    Ok(Expansion { children, contents })
}

/// Expands one node of an implicitly-tiled octree. See
/// [`expand_implicit_quadtree`] for the retention/fetch behavior.
async fn expand_implicit_octree(
    documents: &ExternalTilesetLoader,
    state: &Arc<ImplicitOctreeLoaderState>,
    id: crate::OctreeTileId,
    depth: usize,
) -> Result<Expansion<TileRef, ContentRef>, Error> {
    let flags = loop {
        let flags = state.flags(id);
        if flags.contains(crate::TileAvailabilityFlags::SUBTREE_AVAILABLE)
            && !flags.contains(crate::TileAvailabilityFlags::SUBTREE_LOADED)
        {
            let subtree_root = id.subtree_root(state.subtree_levels);
            let uri = state.subtree_uri(subtree_root);
            let bytes = documents.fetch_bytes(uri.to_string()).await?;
            state.register_subtree(subtree_root, &bytes, &uri)?;
            continue;
        }
        break flags;
    };

    if !flags.contains(crate::TileAvailabilityFlags::TILE_AVAILABLE)
        || id.level + 1 >= state.available_levels
    {
        return Ok(Expansion {
            children: Vec::new(),
            contents: Vec::new(),
        });
    }

    let mut children = Vec::new();
    let mut contents = Vec::new();
    for child_id in id.children() {
        let child_flags = state.flags(child_id);
        if !child_flags.contains(crate::TileAvailabilityFlags::TILE_AVAILABLE) {
            continue;
        }
        if child_flags.contains(crate::TileAvailabilityFlags::CONTENT_AVAILABLE) {
            let uri = state.content_uri(child_id);
            contents.push(ContentRef {
                format: crate::TileFormat::detect(&uri.to_string(), &[]),
                uri: uri.to_string(),
            });
        }
        children.push(TileRef {
            node: Node::ImplicitOctree {
                state: state.clone(),
                id: child_id,
            },
            depth: depth + 1,
            external_root: false,
        });
    }
    Ok(Expansion { children, contents })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn loads_and_expands_tileset_items() {
        let requests = Arc::new(AtomicUsize::new(0));
        let tileset = br#"{"asset":{"version":"1.1"},"geometricError":10.0,
          "root":{"boundingVolume":{"sphere":[0.0,0.0,0.0,1.0]},
          "geometricError":1.0,"content":{"uri":"root.glb"},
          "children":[{"boundingVolume":{"sphere":[0.0,0.0,0.0,0.5]},
          "geometricError":0.0,"content":{"uri":"child.glb"}}]}}"#
            .to_vec();
        let fetch_requests = Arc::clone(&requests);
        let fetch = move |request: FetchRequest| {
            fetch_requests.fetch_add(1, Ordering::SeqCst);
            let result = match request.uri.as_str() {
                "memory://root/tileset.json" => Ok(kiba::FetchResponse {
                    bytes: tileset.clone(),
                    content_type: Some("application/json".into()),
                }),
                "memory://root/root.glb" => Ok(kiba::FetchResponse {
                    bytes: b"glTFroot".to_vec(),
                    content_type: Some("model/gltf-binary".into()),
                }),
                other => Err(Box::new(std::io::Error::other(other)) as _),
            };
            Box::pin(async move { result })
        };
        let loader = TilesetLoader::open("memory://root/tileset.json", fetch);
        assert_eq!(requests.load(Ordering::SeqCst), 0);
        let root = block_on(loader.root()).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        let expansion = block_on(loader.expand(root)).unwrap();
        assert_eq!(expansion.children.len(), 1);
        assert_eq!(expansion.contents.len(), 1);
        let content = block_on(loader.load(expansion.contents[0].clone())).unwrap();
        assert!(matches!(content, LoadOutcome::Ready(LoadedContent { .. })));
    }

    #[test]
    fn open_performs_no_io() {
        let requests = Arc::new(AtomicUsize::new(0));
        let fetch_requests = Arc::clone(&requests);
        let fetch = move |_: FetchRequest| {
            fetch_requests.fetch_add(1, Ordering::SeqCst);
            let result: Result<kiba::FetchResponse, kiba::FetchError> =
                Err(Box::new(std::io::Error::other("unused")));
            Box::pin(async move { result })
        };
        // Constructing a loader must not touch the fetch callback at all.
        let _loader = TilesetLoader::open("memory://root/tileset.json", fetch);
        assert_eq!(requests.load(Ordering::SeqCst), 0);
    }

    /// Two content references pointing at the same external tileset URI
    /// must fetch and parse that manifest only once; both resulting items
    /// must address the very same retained [`DocumentState`].
    #[test]
    fn external_manifest_document_is_retained_and_reused() {
        let requests = Arc::new(AtomicUsize::new(0));
        let root_tileset = br#"{"asset":{"version":"1.1"},"geometricError":10.0,
          "root":{"boundingVolume":{"sphere":[0.0,0.0,0.0,10.0]},"geometricError":5.0,
          "children":[
            {"boundingVolume":{"sphere":[0.0,0.0,0.0,5.0]},"geometricError":0.0,"content":{"uri":"external.json"}},
            {"boundingVolume":{"sphere":[0.0,0.0,0.0,5.0]},"geometricError":0.0,"content":{"uri":"external.json"}}
          ]}}"#
            .to_vec();
        let external_tileset = br#"{"asset":{"version":"1.1"},"geometricError":1.0,
          "root":{"boundingVolume":{"sphere":[0.0,0.0,0.0,1.0]},"geometricError":0.0,
          "content":{"uri":"ext_root.glb"}}}"#
            .to_vec();
        let fetch_requests = Arc::clone(&requests);
        let fetch = move |request: FetchRequest| {
            fetch_requests.fetch_add(1, Ordering::SeqCst);
            let result = match request.uri.as_str() {
                "memory://root/tileset.json" => Ok(kiba::FetchResponse {
                    bytes: root_tileset.clone(),
                    content_type: Some("application/json".into()),
                }),
                "memory://root/external.json" => Ok(kiba::FetchResponse {
                    bytes: external_tileset.clone(),
                    content_type: Some("application/json".into()),
                }),
                other => Err(Box::new(std::io::Error::other(other)) as _),
            };
            Box::pin(async move { result })
        };
        let loader = TilesetLoader::open("memory://root/tileset.json", fetch);
        let root = block_on(loader.root()).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        let root_expansion = block_on(loader.expand(root)).unwrap();
        assert_eq!(root_expansion.children.len(), 2);
        // Explicit children are not fetched; only the shared external
        // manifest is.
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        let mut external_documents = Vec::new();
        for child in root_expansion.children {
            let expansion = block_on(loader.expand(child)).unwrap();
            assert_eq!(expansion.children.len(), 1);
            match &expansion.children[0].node {
                Node::Document { document, .. } => external_documents.push(document.clone()),
                other => panic!("expected a Document node, got {other:?}"),
            }
        }
        // The external manifest is fetched exactly once...
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        // ...and both content references resolve to the very same retained
        // document, not two separately parsed copies.
        assert_eq!(external_documents.len(), 2);
        assert!(Arc::ptr_eq(&external_documents[0], &external_documents[1]));
    }

    /// Expanding an implicitly-tiled node fetches its subtree file at most
    /// once: the retained availability state persists across repeated
    /// expansions of the same node instead of being rebuilt from scratch.
    #[test]
    fn implicit_loader_state_is_retained_and_reused() {
        let requests = Arc::new(AtomicUsize::new(0));
        let root_tileset = br#"{"asset":{"version":"1.1"},"geometricError":100.0,
          "root":{"boundingVolume":{"sphere":[0.0,0.0,0.0,100.0]},"geometricError":50.0,
          "content":{"uri":"content/{level}/{x}/{y}.glb"},
          "implicitTiling":{"subdivisionScheme":"Quadtree","subtreeLevels":2,"availableLevels":2,
          "subtrees":{"uri":"subtrees/{level}/{x}/{y}.subtree"}}}}"#
            .to_vec();
        let subtree = br#"{"tileAvailability":{"constant":1},
          "contentAvailability":[{"constant":1}],
          "childSubtreeAvailability":{"constant":0}}"#
            .to_vec();
        let fetch_requests = Arc::clone(&requests);
        let fetch = move |request: FetchRequest| {
            fetch_requests.fetch_add(1, Ordering::SeqCst);
            let result = match request.uri.as_str() {
                "memory://root/tileset.json" => Ok(kiba::FetchResponse {
                    bytes: root_tileset.clone(),
                    content_type: Some("application/json".into()),
                }),
                "memory://root/subtrees/0/0/0.subtree" => Ok(kiba::FetchResponse {
                    bytes: subtree.clone(),
                    content_type: Some("application/octet-stream".into()),
                }),
                other => Err(Box::new(std::io::Error::other(other)) as _),
            };
            Box::pin(async move { result })
        };
        let loader = TilesetLoader::open("memory://root/tileset.json", fetch);
        let root = block_on(loader.root()).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        let first = block_on(loader.expand(root.clone())).unwrap();
        // Fetching the tile's implicit subtree adds exactly one request.
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(first.children.len(), 4);
        assert_eq!(first.contents.len(), 4);
        let first_state = match &first.children[0].node {
            Node::ImplicitQuadtree { state, .. } => state.clone(),
            other => panic!("expected an ImplicitQuadtree node, got {other:?}"),
        };

        // Re-expanding the very same explicit tile must reuse the cached
        // availability tree rather than rebuilding it and re-fetching the
        // already-loaded subtree.
        let second = block_on(loader.expand(root)).unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_eq!(second.children.len(), 4);
        let second_state = match &second.children[0].node {
            Node::ImplicitQuadtree { state, .. } => state.clone(),
            other => panic!("expected an ImplicitQuadtree node, got {other:?}"),
        };
        assert!(Arc::ptr_eq(&first_state, &second_state));
    }

    fn block_on<F: std::future::Future>(mut future: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut context = Context::from_waker(&waker);
        loop {
            match unsafe { std::pin::Pin::new_unchecked(&mut future) }.poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }
}
