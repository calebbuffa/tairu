//! Retainable, extension-defined navigation over a [`TilesetLoader`].

use std::sync::Arc;

use ::hiera::{Cursor, Navigator};

use crate::{ContentRef, Error, Tile, TileInfo, TileRef, TilesetLoader, TilesetReader, Uri};

/// Source-native tile data supplied while a cursor is being resolved.
///
/// Tairu deliberately does not interpret this data beyond the 3D Tiles data
/// model. A resolver decides which facts are effective for its workflow and
/// records them in its own [`TileResolver::State`].
pub struct TileResolveInput<'a> {
    /// The complete source tile.
    pub tile: &'a Tile,
    /// Source-native facts retained by the loader.
    pub info: &'a TileInfo,
}

/// Derives application- or extension-defined state for a tile cursor.
///
/// A resolver owns all behavior beyond raw 3D Tiles loading: transforms,
/// coordinate systems, metadata inheritance, visibility, content selection,
/// and vendor extensions are represented by its [`Self::State`], not by
/// Tairu's cursor implementation.
pub trait TileResolver: Send + Sync + 'static {
    /// Immutable state retained by every cursor.
    type State: Send + Sync + 'static;

    /// Resolves state for the top-level root tile.
    fn root(&self, input: TileResolveInput<'_>) -> Result<Self::State, Error>;

    /// Resolves state for a child using its parent's resolved state.
    fn child(
        &self,
        input: TileResolveInput<'_>,
        parent: &Self::State,
    ) -> Result<Self::State, Error>;
}

impl TileResolver for () {
    type State = ();

    fn root(&self, _input: TileResolveInput<'_>) -> Result<Self::State, Error> {
        Ok(())
    }

    fn child(
        &self,
        _input: TileResolveInput<'_>,
        _parent: &Self::State,
    ) -> Result<Self::State, Error> {
        Ok(())
    }
}

struct TileCursorData<S> {
    tile: Arc<Tile>,
    source_uri: Uri,
    depth: usize,
    is_external_root: bool,
    resolved: S,
}

/// A retainable position in a loaded 3D Tiles hierarchy.
///
/// The cursor always retains raw source data and the resolver-defined state
/// selected by the caller. Tairu does not prescribe any effective behavior.
#[derive(Clone)]
pub struct TileCursor<S>
where
    S: Send + Sync + 'static,
{
    inner: Cursor<TilesetLoader, TileCursorData<S>>,
}

/// The shallow expansion of a [`TileCursor`].
pub struct TileExpansion<S>
where
    S: Send + Sync + 'static,
{
    /// Child positions with their resolver-defined state.
    pub children: Vec<TileCursor<S>>,
    /// Content references belonging to the expanded tile.
    pub contents: Vec<ContentRef>,
}

/// Creates [`TileCursor`] values over a [`TilesetLoader`].
#[derive(Clone)]
pub struct TilesetNavigator<R>
where
    R: TileResolver,
{
    inner: Navigator<TilesetLoader, TileCursorData<R::State>>,
}

impl TilesetNavigator<()> {
    /// Creates a navigator that retains raw 3D Tiles data only.
    pub fn new(loader: TilesetLoader) -> Self {
        Self::with_resolver(loader, ())
    }

    /// Creates a raw-data navigator over a typed reader's shared loader.
    pub fn from_reader(reader: &TilesetReader) -> Self {
        Self::from_shared(reader.shared_loader(), Arc::new(()))
    }
}

impl<R> TilesetNavigator<R>
where
    R: TileResolver,
{
    /// Creates a navigator that owns `loader` and derives cursor behavior
    /// with `resolver`.
    pub fn with_resolver(loader: TilesetLoader, resolver: R) -> Self {
        Self::from_shared(Arc::new(loader), Arc::new(resolver))
    }

    /// Creates a navigator over a typed reader's shared loader and `resolver`.
    ///
    /// The reader and navigator retain the same caller-owned transport and
    /// document cache; neither creates a runtime or changes loading policy.
    pub fn with_reader(reader: &TilesetReader, resolver: R) -> Self {
        Self::from_shared(reader.shared_loader(), Arc::new(resolver))
    }

    /// Creates a navigator over a shared loader and resolver.
    pub fn from_shared(loader: Arc<TilesetLoader>, resolver: Arc<R>) -> Self {
        let root_resolver = Arc::clone(&resolver);
        let child_resolver = resolver;
        let inner = Navigator::from_shared_sync_state(
            loader,
            move |loader: &TilesetLoader, item: &TileRef, info: &TileInfo| {
                let tile = Arc::new(loader.tile(item));
                root_resolver
                    .root(TileResolveInput { tile: &tile, info })
                    .map(|resolved| TileCursorData {
                        tile,
                        source_uri: info.source_uri.clone(),
                        depth: info.depth,
                        is_external_root: info.external_root,
                        resolved,
                    })
            },
            move |loader: &TilesetLoader,
                  parent: &Cursor<TilesetLoader, TileCursorData<R::State>>,
                  item: &TileRef,
                  info: &TileInfo| {
                let tile = Arc::new(loader.tile(item));
                child_resolver
                    .child(
                        TileResolveInput { tile: &tile, info },
                        &parent.state().resolved,
                    )
                    .map(|resolved| TileCursorData {
                        tile,
                        source_uri: info.source_uri.clone(),
                        depth: info.depth,
                        is_external_root: info.external_root,
                        resolved,
                    })
            },
        );
        Self { inner }
    }

    /// Returns the loader serving this navigator.
    pub fn loader(&self) -> &TilesetLoader {
        self.inner.loader()
    }

    /// Returns the resolved root cursor.
    pub async fn root(&self) -> Result<TileCursor<R::State>, Error> {
        Ok(TileCursor {
            inner: self.inner.root().await?,
        })
    }
}

impl<S> TileCursor<S>
where
    S: Send + Sync + 'static,
{
    /// Returns the complete source tile.
    pub fn tile(&self) -> &Tile {
        &self.inner.state().tile
    }

    /// Returns the tileset document URI that declared this tile.
    pub fn source_uri(&self) -> &Uri {
        &self.inner.state().source_uri
    }

    /// Returns the traversal depth of this tile.
    pub fn depth(&self) -> usize {
        self.inner.state().depth
    }

    /// Returns whether this is the root of an external tileset.
    pub fn is_external_root(&self) -> bool {
        self.inner.state().is_external_root
    }

    /// Returns the complete resolver-defined state for this cursor.
    pub fn state(&self) -> &S {
        &self.inner.state().resolved
    }

    /// Returns a stable identity for this source tile.
    ///
    /// Callers can use this to de-duplicate a graph-shaped traversal frontier.
    pub fn id(&self) -> String {
        self.inner.item_id()
    }

    /// Returns the parent cursor, or `None` for the root.
    pub fn parent(&self) -> Option<Self> {
        self.inner.parent().map(|inner| Self { inner })
    }

    /// Returns the immediate children and content references.
    pub async fn expand(&self) -> Result<TileExpansion<S>, Error> {
        let expansion = self.inner.expand().await?;
        Ok(TileExpansion {
            children: expansion
                .children
                .into_iter()
                .map(|inner| Self { inner })
                .collect(),
            contents: expansion.contents,
        })
    }

    /// Returns immediate child cursors.
    pub async fn children(&self) -> Result<Vec<Self>, Error> {
        Ok(self.expand().await?.children)
    }

    /// Returns every child of this cursor's parent except this cursor.
    pub async fn siblings(&self) -> Result<Vec<Self>, Error> {
        Ok(self
            .inner
            .siblings()
            .await?
            .into_iter()
            .map(|inner| Self { inner })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use ::hiera::FetchResponse;
    use futures::executor::block_on;

    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TransformState {
        world_x: f64,
    }

    struct TransformResolver;

    impl TileResolver for TransformResolver {
        type State = TransformState;

        fn root(&self, input: TileResolveInput<'_>) -> Result<Self::State, Error> {
            Ok(TransformState {
                world_x: input.info.transform[12],
            })
        }

        fn child(
            &self,
            input: TileResolveInput<'_>,
            parent: &Self::State,
        ) -> Result<Self::State, Error> {
            Ok(TransformState {
                world_x: parent.world_x + input.info.transform[12],
            })
        }
    }

    #[test]
    fn cursors_retain_raw_data_and_resolver_defined_state() {
        let tileset = br#"{
            "asset": { "version": "1.1" },
            "geometricError": 10.0,
            "root": {
                "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                "geometricError": 1.0,
                "transform": [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 1.0],
                "children": [
                    {
                        "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 0.5] },
                        "geometricError": 0.0,
                        "transform": [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 3.0, 0.0, 0.0, 1.0]
                    },
                    {
                        "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 0.25] },
                        "geometricError": 0.0
                    }
                ]
            }
        }"#
        .to_vec();
        let loader = TilesetLoader::open("memory://root/tileset.json", move |_request| {
            let response = FetchResponse {
                bytes: tileset.clone().into(),
                content_type: Some("application/json".into()),
            };
            async move { Ok::<_, ::hiera::FetchError>(response) }
        });
        let navigator = TilesetNavigator::with_resolver(loader, TransformResolver);

        let root = block_on(navigator.root()).unwrap();
        assert_eq!(root.state().world_x, 2.0);
        assert_eq!(root.source_uri().to_string(), "memory://root/tileset.json");

        let children = block_on(root.children()).unwrap();
        let first = &children[0];
        assert_eq!(first.state().world_x, 5.0);
        assert_eq!(first.parent().unwrap().state().world_x, 2.0);
        assert_eq!(block_on(first.siblings()).unwrap().len(), 1);
    }
}
