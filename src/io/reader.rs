//! Typed 3D Tiles reads composed over [`TilesetLoader`].

use std::sync::Arc;

use hiera::{DescribeFuture, ExpandFuture, LoadFuture, Loader, RootFuture};

use crate::{ContentRef, Error, LoadedContent, Tile, TileInfo, TileRef, Tileset, TilesetLoader};

/// Typed 3D Tiles access over a shared hierarchy loader.
///
/// This reader is a zero-policy decoration: it retains every [`Loader`]
/// operation and uses the loader's caller-provided transport/cache. It never
/// creates an executor or decodes GLB, glTF, images, or other tile payloads.
#[derive(Clone)]
pub struct TilesetReader {
    loader: Arc<TilesetLoader>,
}

impl TilesetReader {
    /// Creates a typed reader over a shared tileset loader.
    pub fn new(loader: Arc<TilesetLoader>) -> Self {
        Self { loader }
    }

    /// Returns the hierarchy loader this reader composes.
    pub fn loader(&self) -> &TilesetLoader {
        &self.loader
    }

    /// Returns a shared handle to the hierarchy loader this reader composes.
    pub fn shared_loader(&self) -> Arc<TilesetLoader> {
        Arc::clone(&self.loader)
    }

    /// Returns the typed source tile represented by `item`.
    ///
    /// Implicit tiles are synthesized from their retained availability state.
    pub fn tile(&self, item: &TileRef) -> Tile {
        self.loader.tile(item)
    }

    /// Returns the parsed tileset document that declared an explicit tile.
    ///
    /// Returns `None` for synthesized implicit tiles.
    pub fn tileset(&self, item: &TileRef) -> Option<Tileset> {
        self.loader.tileset(item)
    }

    /// Loads raw tile-content bytes through the caller-provided transport.
    ///
    /// The returned value retains its detected [`crate::TileFormat`]. Callers
    /// schedule any GLB/glTF or other payload decoding on their own workers.
    pub async fn content(
        &self,
        reference: ContentRef,
    ) -> Result<hiera::LoadOutcome<LoadedContent>, Error> {
        self.load(reference).await
    }
}

impl Loader for TilesetReader {
    type Item = TileRef;
    type ItemId = String;
    type ContentRef = ContentRef;
    type ItemInfo = TileInfo;
    type Content = LoadedContent;
    type Error = Error;

    fn root(&self) -> RootFuture<Self::Item, Self::Error> {
        self.loader.root()
    }

    fn item_id(&self, item: &Self::Item) -> Self::ItemId {
        self.loader.item_id(item)
    }

    fn describe(&self, item: Self::Item) -> DescribeFuture<Self::ItemInfo, Self::Error> {
        self.loader.describe(item)
    }

    fn expand(&self, item: Self::Item) -> ExpandFuture<Self::Item, Self::ContentRef, Self::Error> {
        self.loader.expand(item)
    }

    fn load(&self, content: Self::ContentRef) -> LoadFuture<Self::Content, Self::Error> {
        self.loader.load(content)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::executor::block_on;
    use hiera::{FetchResponse, LoadOutcome};

    use super::*;

    #[test]
    fn reader_delegates_hierarchy_and_returns_undecoded_content() {
        let tileset = br#"{
            "asset": { "version": "1.1" },
            "geometricError": 1.0,
            "root": {
                "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                "geometricError": 0.0,
                "content": { "uri": "tile.glb" }
            }
        }"#
        .to_vec();
        let loader = TilesetLoader::open("memory://tiles/tileset.json", move |request| {
            let result = match request.uri.as_str() {
                "memory://tiles/tileset.json" => Ok(FetchResponse {
                    bytes: tileset.clone().into(),
                    content_type: Some("application/json".into()),
                }),
                "memory://tiles/tile.glb" => Ok(FetchResponse {
                    bytes: b"glTFraw-content".to_vec().into(),
                    content_type: Some("model/gltf-binary".into()),
                }),
                other => Err(Box::new(std::io::Error::other(other)) as _),
            };
            async move { result }
        });
        let reader = TilesetReader::new(Arc::new(loader));

        let root = block_on(reader.root()).expect("reader root");
        assert_eq!(
            reader
                .tileset(&root)
                .expect("explicit root has a declaring document")
                .asset
                .version
                .as_ref(),
            "1.1"
        );
        assert_eq!(reader.tile(&root).geometric_error, 0.0);

        let navigator = crate::TilesetNavigator::from_reader(&reader);
        let cursor = block_on(navigator.root()).expect("navigator root");
        assert_eq!(cursor.tile().geometric_error, 0.0);

        let expansion = block_on(reader.expand(root)).expect("reader expansion");
        assert_eq!(expansion.contents.len(), 1);
        let loaded =
            block_on(reader.content(expansion.contents[0].clone())).expect("reader content");
        let LoadOutcome::Ready(loaded) = loaded else {
            panic!("content must be present");
        };
        assert_eq!(loaded.format, crate::TileFormat::Glb);
        assert_eq!(&loaded.bytes[..], b"glTFraw-content");
    }
}
