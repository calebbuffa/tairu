//! Exhaustive traversal over a retained [`TilesetLoader`] graph.

use std::{collections::HashSet, future::Future, pin::Pin};

use crate::Error;
use crate::generated::Tile;
use crate::io::TilesetLoader;
use crate::uri::Uri;
use ::kiba::Loader;

const MAX_TRAVERSAL_DEPTH: usize = 1024;
const MAX_TRAVERSAL_NODES: usize = 1_000_000;

/// Safety and resource limits for a traversal.
#[derive(Debug, Clone, Copy)]
pub struct TraversalPolicy {
    /// Maximum tile depth, inclusive.
    pub max_depth: usize,
    /// Maximum number of visited tiles.
    pub max_nodes: usize,
}

impl Default for TraversalPolicy {
    fn default() -> Self {
        Self {
            max_depth: MAX_TRAVERSAL_DEPTH,
            max_nodes: MAX_TRAVERSAL_NODES,
        }
    }
}

/// Column-major 4x4 matrix in f64.
pub type Mat4d = [f64; 16];

/// Control flow for node visitors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalControl {
    /// Visit this node's descendants and continue the traversal.
    Continue,
    /// Do not expand this node's children; continue with siblings.
    SkipChildren,
    /// Terminate the entire traversal immediately.
    Stop,
}

fn mul_col_major(a: &Mat4d, b: &Mat4d) -> Mat4d {
    let mut out = [0.0; 16];
    for col in 0..4 {
        for row in 0..4 {
            out[col * 4 + row] = a[row] * b[col * 4]
                + a[4 + row] * b[col * 4 + 1]
                + a[8 + row] * b[col * 4 + 2]
                + a[12 + row] * b[col * 4 + 3];
        }
    }
    out
}

fn identity_transform() -> Mat4d {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// A tile and the resolved transform state at the point it is visited.
#[non_exhaustive]
pub struct TileVisit<'a> {
    /// The complete source tile being visited.
    tile: &'a Tile,
    /// The tileset URI this tile came from.
    source_uri: &'a Uri,
    /// Traversal depth (0 for the root of the top-level tileset).
    depth: usize,
    /// Whether this tile is the root of an external tileset.
    is_external_root: bool,
    /// The accumulated world transform of the parent tile.
    parent_transform: Mat4d,
    /// The tile's own core transform.
    core_local_transform: Mat4d,
    local_transform: Mat4d,
}

impl TileVisit<'_> {
    /// Returns the source tile.
    pub fn tile(&self) -> &Tile {
        self.tile
    }

    /// Returns the tileset URI this tile came from.
    pub fn source_uri(&self) -> &Uri {
        self.source_uri
    }

    /// Returns the traversal depth of this tile.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns whether this tile is the root of an external tileset.
    pub fn is_external_root(&self) -> bool {
        self.is_external_root
    }

    /// Returns the accumulated world transform of the parent tile.
    pub fn parent_transform(&self) -> Mat4d {
        self.parent_transform
    }

    /// Returns the tile's own core transform.
    pub fn core_local_transform(&self) -> Mat4d {
        self.core_local_transform
    }

    /// Returns the effective local transform.
    pub fn local_transform(&self) -> Mat4d {
        self.local_transform
    }

    /// Replaces the local transform and preserves the world-transform law.
    pub fn replace_local_transform(&mut self, transform: Mat4d) {
        self.local_transform = transform;
    }

    /// Returns `parent_transform * local_transform`.
    pub fn world_transform(&self) -> Mat4d {
        mul_col_major(&self.parent_transform, &self.local_transform)
    }
}

/// Exhaustively walks a retained [`TilesetLoader`] graph.
///
/// Manifest and implicit-tiling loader state is retained by `loader` while the
/// walk requests shallow expansions on demand.
pub async fn walk<V, E>(loader: &TilesetLoader, visit: V) -> Result<(), Error>
where
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    walk_with_policy(loader, TraversalPolicy::default(), visit).await
}

/// Walk a tileset using explicit safety and resource limits.
pub async fn walk_with_policy<V, E>(
    loader: &TilesetLoader,
    policy: TraversalPolicy,
    mut visit: V,
) -> Result<(), Error>
where
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    let root = loader.root().await?;
    let mut state = WalkState::default();
    walk_item(
        loader,
        root,
        identity_transform(),
        false,
        &mut state,
        &mut visit,
        policy,
    )
    .await
}

#[derive(Default)]
struct WalkState {
    active_sources: HashSet<String>,
    nodes: usize,
}

fn walk_item<'a, V, E>(
    loader: &'a TilesetLoader,
    item: crate::TileRef,
    parent_transform: Mat4d,
    is_external_root: bool,
    state: &'a mut WalkState,
    visit: &'a mut V,
    policy: TraversalPolicy,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
where
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    Box::pin(async move {
        state.nodes = state.nodes.checked_add(1).ok_or(Error::TraversalLimit {
            max: MAX_TRAVERSAL_NODES,
        })?;
        if state.nodes > policy.max_nodes {
            return Err(Error::TraversalLimit {
                max: policy.max_nodes,
            });
        }

        let (tile, source_uri, depth, item_external_root) = loader.visit_data(&item);
        if depth > policy.max_depth {
            return Err(Error::TraversalLimit {
                max: policy.max_depth,
            });
        }
        let is_external_root = is_external_root || item_external_root;
        let source_key = source_uri.to_string();
        if is_external_root && !state.active_sources.insert(source_key.clone()) {
            return Err(Error::external_cycle(source_key));
        }

        let mut tile_visit = TileVisit {
            tile: &tile,
            source_uri: &source_uri,
            depth,
            is_external_root,
            parent_transform,
            core_local_transform: tile.transform,
            local_transform: tile.transform,
        };
        let control = visit(&mut tile_visit).map_err(Error::visitor)?;
        if matches!(
            control,
            TraversalControl::Stop | TraversalControl::SkipChildren
        ) {
            if is_external_root {
                state.active_sources.remove(&source_key);
            }
            return Ok(());
        }

        let world_transform = tile_visit.world_transform();
        let expansion = loader.expand(item).await?;
        for child in expansion.children {
            walk_item(loader, child, world_transform, false, state, visit, policy).await?;
        }
        if is_external_root {
            state.active_sources.remove(&source_key);
        }
        Ok(())
    })
}
