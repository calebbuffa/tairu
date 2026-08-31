//! Generic traversal engine with implicit expanders, no lazy-json dependency.

use crate::Error;
use crate::availability::{
    OctreeAvailability, OctreeTileId, QuadtreeAvailability, QuadtreeTileId, TileAvailabilityFlags,
};
use crate::generated::{BoundingVolume, Content, SubdivisionScheme, Tile};
use crate::reader::TileParseError;
use crate::reader::from_slice;
use crate::subtree::parse_subtree;
use crate::uri::{Uri, is_external_tileset_uri};
use std::collections::HashSet;

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

/// Result of expanding a node into traversal children.
#[derive(Debug, Clone)]
pub enum ExpansionResult<TNode> {
    Ready(Vec<TNode>),
    Pending,
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
///
/// The complete source tile remains available through [`TileVisit::tile`].
/// `local_transform` is the effective transform used for this traversal and
/// may be replaced before descendants are expanded.
pub struct TileVisit<'a> {
    /// The complete source tile being visited.
    pub tile: &'a Tile,
    /// The tileset URI this tile came from.
    pub source_uri: &'a Uri,
    /// Traversal depth (0 for the root of the top-level tileset).
    pub depth: usize,
    /// Whether this tile is the root of an external tileset.
    pub is_external_root: bool,
    /// The accumulated world transform of the parent tile.
    pub parent_transform: Mat4d,
    /// The tile's own core transform (matrix, or composed TRS).
    pub core_local_transform: Mat4d,
    local_transform: Mat4d,
}

impl TileVisit<'_> {
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

fn external_uris(tile: &Tile) -> impl Iterator<Item = &str> {
    tile.content
        .iter()
        .map(|content| content.uri.as_str())
        .chain(tile.contents.iter().map(|content| content.uri.as_str()))
        .filter(|uri| is_external_tileset_uri(uri))
}

/// Walks a tileset with standard child and external-tileset expansion.
///
/// The fetch closure receives resolved resource URIs. The visitor receives
/// each complete source tile before its descendants are expanded.
pub fn walk<F, V, E>(root_uri: impl Into<Uri>, fetch: &mut F, mut visit: V) -> Result<(), Error>
where
    F: FnMut(&str) -> Result<Vec<u8>, E>,
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let root_uri = root_uri.into();
    let bytes = fetch(&root_uri.to_string())
        .map_err(|source| Error::fetch(root_uri.to_string(), source))?;
    let tileset =
        from_slice(&bytes).map_err(|source| Error::tile_parse(root_uri.to_string(), source))?;
    let mut active_sources = HashSet::new();
    active_sources.insert(root_uri.to_string());
    walk_tile(
        &tileset.root,
        &root_uri,
        identity_transform(),
        0,
        true,
        &mut active_sources,
        fetch,
        &mut visit,
    )
}

// Internal recursive walk; the context (transform, depth, sources, visitor)
// is threaded through parameters by design.
#[allow(clippy::too_many_arguments)]
fn walk_tile<F, V, E>(
    tile: &Tile,
    source_uri: &Uri,
    parent_transform: Mat4d,
    depth: usize,
    is_external_root: bool,
    active_sources: &mut HashSet<String>,
    fetch: &mut F,
    visit: &mut V,
) -> Result<(), Error>
where
    F: FnMut(&str) -> Result<Vec<u8>, E>,
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut tile_visit = TileVisit {
        tile,
        source_uri,
        depth,
        is_external_root,
        parent_transform,
        core_local_transform: tile.transform,
        local_transform: tile.transform,
    };
    let control = visit(&mut tile_visit).map_err(Error::visitor)?;
    if !matches!(control, TraversalControl::Continue) {
        return Ok(());
    }

    let world_transform = tile_visit.world_transform();
    if let Some(implicit) = &tile.implicit_tiling {
        walk_implicit_tile(
            tile,
            implicit,
            source_uri,
            world_transform,
            depth,
            active_sources,
            fetch,
            visit,
        )?;
        return Ok(());
    }

    for child in &tile.children {
        walk_tile(
            child,
            source_uri,
            world_transform,
            depth + 1,
            false,
            active_sources,
            fetch,
            visit,
        )?;
    }

    for relative_uri in external_uris(tile) {
        let resolved_uri = source_uri.resolve(relative_uri);
        let resolved_key = resolved_uri.to_string();
        if !active_sources.insert(resolved_key.clone()) {
            return Err(Error::external_cycle(resolved_key));
        }
        let result = (|| {
            let bytes = fetch(&resolved_uri.to_string())
                .map_err(|source| Error::fetch(resolved_key.clone(), source))?;
            let tileset = from_slice(&bytes)
                .map_err(|source| Error::tile_parse(resolved_key.clone(), source))?;
            walk_tile(
                &tileset.root,
                &resolved_uri,
                world_transform,
                depth + 1,
                true,
                active_sources,
                fetch,
                visit,
            )
        })();
        active_sources.remove(&resolved_key);
        result?;
    }
    Ok(())
}

// Internal recursive walk; the context (implicit tiling, transform, depth,
// sources, visitor) is threaded through parameters by design.
#[allow(clippy::too_many_arguments)]
fn walk_implicit_tile<F, V, E>(
    tile: &Tile,
    implicit: &crate::generated::ImplicitTiling,
    source_uri: &Uri,
    world_transform: Mat4d,
    depth: usize,
    _active_sources: &mut HashSet<String>,
    fetch: &mut F,
    visit: &mut V,
) -> Result<(), Error>
where
    F: FnMut(&str) -> Result<Vec<u8>, E>,
    V: FnMut(&mut TileVisit<'_>) -> Result<TraversalControl, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let content_template = tile
        .content
        .as_ref()
        .map(|content| content.uri.clone())
        .or_else(|| tile.contents.first().map(|content| content.uri.clone()));
    let Some(content_template) = content_template else {
        return Ok(());
    };
    let subtree_template = implicit.subtrees.uri.clone();
    match implicit.subdivision_scheme {
        SubdivisionScheme::Quadtree => {
            let mut expander = ImplicitQuadtreeExpander::new(
                tile.bounding_volume.clone(),
                source_uri.clone(),
                content_template,
                subtree_template,
                implicit.subtree_levels as u32,
                implicit.available_levels as u32,
            );
            let mut stack = vec![ImplicitQuadtileNode {
                id: QuadtreeTileId::new(0, 0, 0),
                bounding_volume: tile.bounding_volume.clone(),
                geometric_error: tile.geometric_error,
                world_transform,
                content_uri: None,
            }];
            while let Some(node) = stack.pop() {
                match expander.expand_node(&node) {
                    ExpansionResult::Pending => {
                        let uri = node
                            .id
                            .subtree_root(expander.subtree_levels)
                            .resolve_url(source_uri, &expander.subtree_uri_template);
                        let bytes = fetch(&uri.to_string())
                            .map_err(|source| Error::fetch(uri.to_string(), source))?;
                        expander
                            .register_subtree(node.id.subtree_root(expander.subtree_levels), &bytes)
                            .map_err(|source| Error::tile_parse(uri.to_string(), source))?;
                        stack.push(node);
                    }
                    ExpansionResult::Ready(children) => {
                        for child in children {
                            if child.id.level == 0 {
                                continue;
                            }
                            let synthetic = Tile {
                                bounding_volume: child.bounding_volume.clone(),
                                geometric_error: child.geometric_error,
                                content: child.content_uri.clone().map(|uri| Content {
                                    uri,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            };
                            let mut tile_visit = TileVisit {
                                tile: &synthetic,
                                source_uri,
                                depth: depth + child.id.level as usize,
                                is_external_root: false,
                                parent_transform: child.world_transform,
                                core_local_transform: identity_transform(),
                                local_transform: identity_transform(),
                            };
                            match visit(&mut tile_visit).map_err(Error::visitor)? {
                                TraversalControl::Continue => stack.push(child),
                                TraversalControl::SkipChildren => {}
                                TraversalControl::Stop => return Ok(()),
                            }
                        }
                    }
                }
            }
        }
        SubdivisionScheme::Octree => {
            let mut expander = ImplicitOctreeExpander::new(
                tile.bounding_volume.clone(),
                source_uri.clone(),
                content_template,
                subtree_template,
                implicit.subtree_levels as u32,
                implicit.available_levels as u32,
            );
            let mut stack = vec![ImplicitOctreeNode {
                id: OctreeTileId::new(0, 0, 0, 0),
                bounding_volume: tile.bounding_volume.clone(),
                geometric_error: tile.geometric_error,
                world_transform,
                content_uri: None,
            }];
            while let Some(node) = stack.pop() {
                match expander.expand_node(&node) {
                    ExpansionResult::Pending => {
                        let uri = node
                            .id
                            .subtree_root(expander.subtree_levels)
                            .resolve_url(source_uri, &expander.subtree_uri_template);
                        let bytes = fetch(&uri.to_string())
                            .map_err(|source| Error::fetch(uri.to_string(), source))?;
                        expander
                            .register_subtree(node.id.subtree_root(expander.subtree_levels), &bytes)
                            .map_err(|source| Error::tile_parse(uri.to_string(), source))?;
                        stack.push(node);
                    }
                    ExpansionResult::Ready(children) => {
                        for child in children {
                            if child.id.level == 0 {
                                continue;
                            }
                            let synthetic = Tile {
                                bounding_volume: child.bounding_volume.clone(),
                                geometric_error: child.geometric_error,
                                content: child.content_uri.clone().map(|uri| Content {
                                    uri,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            };
                            let mut tile_visit = TileVisit {
                                tile: &synthetic,
                                source_uri,
                                depth: depth + child.id.level as usize,
                                is_external_root: false,
                                parent_transform: child.world_transform,
                                core_local_transform: identity_transform(),
                                local_transform: identity_transform(),
                            };
                            match visit(&mut tile_visit).map_err(Error::visitor)? {
                                TraversalControl::Continue => stack.push(child),
                                TraversalControl::SkipChildren => {}
                                TraversalControl::Stop => return Ok(()),
                            }
                        }
                    }
                }
            }
        }
        SubdivisionScheme::S2 => return Ok(()),
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ImplicitQuadtileNode {
    pub id: QuadtreeTileId,
    pub bounding_volume: BoundingVolume,
    pub geometric_error: f64,
    pub world_transform: Mat4d,
    pub content_uri: Option<String>,
}

pub struct ImplicitQuadtreeExpander {
    root_bounding_volume: BoundingVolume,
    source_uri: Uri,
    content_uri_template: String,
    subtree_uri_template: String,
    subtree_levels: u32,
    available_levels: u32,
    availability: QuadtreeAvailability,
}

impl ImplicitQuadtreeExpander {
    pub fn new(
        root_bounding_volume: BoundingVolume,
        source_uri: Uri,
        content_uri_template: String,
        subtree_uri_template: String,
        subtree_levels: u32,
        available_levels: u32,
    ) -> Self {
        Self {
            root_bounding_volume,
            source_uri,
            content_uri_template,
            subtree_uri_template,
            subtree_levels,
            available_levels,
            availability: QuadtreeAvailability::new(subtree_levels, available_levels),
        }
    }

    fn expand_node(&self, node: &ImplicitQuadtileNode) -> ExpansionResult<ImplicitQuadtileNode> {
        let flags = self.availability.compute_availability(node.id);
        if !flags.contains(TileAvailabilityFlags::TILE_AVAILABLE) {
            return ExpansionResult::Ready(vec![]);
        }
        if flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE)
            && !flags.contains(TileAvailabilityFlags::SUBTREE_LOADED)
        {
            return ExpansionResult::Pending;
        }
        if node.id.level + 1 >= self.available_levels {
            return ExpansionResult::Ready(vec![]);
        }
        let children = node
            .id
            .children()
            .into_iter()
            .filter_map(|child_id| {
                let child_flags = self.availability.compute_availability(child_id);
                if !child_flags.contains(TileAvailabilityFlags::TILE_AVAILABLE) {
                    return None;
                }
                Some(ImplicitQuadtileNode {
                    id: child_id,
                    bounding_volume: child_id.subdivide_bounding_volume(&self.root_bounding_volume),
                    geometric_error: node.geometric_error * 0.5,
                    world_transform: node.world_transform,
                    content_uri: if child_flags.contains(TileAvailabilityFlags::CONTENT_AVAILABLE) {
                        Some(
                            child_id
                                .resolve_url(&self.source_uri, &self.content_uri_template)
                                .to_string(),
                        )
                    } else {
                        None
                    },
                })
            })
            .collect();
        ExpansionResult::Ready(children)
    }

    fn register_subtree(
        &mut self,
        subtree_root: QuadtreeTileId,
        data: &[u8],
    ) -> Result<(), TileParseError> {
        let avail = parse_subtree(data, SubdivisionScheme::Quadtree, self.subtree_levels)
            .map_err(|e| TileParseError::Validation(e.to_string()))?;
        self.availability.add_subtree(subtree_root, avail);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ImplicitOctreeNode {
    pub id: OctreeTileId,
    pub bounding_volume: BoundingVolume,
    pub geometric_error: f64,
    pub world_transform: Mat4d,
    pub content_uri: Option<String>,
}

pub struct ImplicitOctreeExpander {
    root_bounding_volume: BoundingVolume,
    source_uri: Uri,
    content_uri_template: String,
    subtree_uri_template: String,
    subtree_levels: u32,
    available_levels: u32,
    availability: OctreeAvailability,
}

impl ImplicitOctreeExpander {
    pub fn new(
        root_bounding_volume: BoundingVolume,
        source_uri: Uri,
        content_uri_template: String,
        subtree_uri_template: String,
        subtree_levels: u32,
        available_levels: u32,
    ) -> Self {
        Self {
            root_bounding_volume,
            source_uri,
            content_uri_template,
            subtree_uri_template,
            subtree_levels,
            available_levels,
            availability: OctreeAvailability::new(subtree_levels, available_levels),
        }
    }

    fn expand_node(&self, node: &ImplicitOctreeNode) -> ExpansionResult<ImplicitOctreeNode> {
        let flags = self.availability.compute_availability(node.id);
        if !flags.contains(TileAvailabilityFlags::TILE_AVAILABLE) {
            return ExpansionResult::Ready(vec![]);
        }
        if flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE)
            && !flags.contains(TileAvailabilityFlags::SUBTREE_LOADED)
        {
            return ExpansionResult::Pending;
        }
        if node.id.level + 1 >= self.available_levels {
            return ExpansionResult::Ready(vec![]);
        }
        let children = node
            .id
            .children()
            .into_iter()
            .filter_map(|child_id| {
                let child_flags = self.availability.compute_availability(child_id);
                if !child_flags.contains(TileAvailabilityFlags::TILE_AVAILABLE) {
                    return None;
                }
                Some(ImplicitOctreeNode {
                    id: child_id,
                    bounding_volume: child_id.subdivide_bounding_volume(&self.root_bounding_volume),
                    geometric_error: node.geometric_error * 0.5,
                    world_transform: node.world_transform,
                    content_uri: if child_flags.contains(TileAvailabilityFlags::CONTENT_AVAILABLE) {
                        Some(
                            child_id
                                .resolve_url(&self.source_uri, &self.content_uri_template)
                                .to_string(),
                        )
                    } else {
                        None
                    },
                })
            })
            .collect();
        ExpansionResult::Ready(children)
    }

    fn register_subtree(
        &mut self,
        subtree_root: OctreeTileId,
        data: &[u8],
    ) -> Result<(), TileParseError> {
        let avail = parse_subtree(data, SubdivisionScheme::Octree, self.subtree_levels)
            .map_err(|e| TileParseError::Validation(e.to_string()))?;
        self.availability.add_subtree(subtree_root, avail);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn tileset_with_root(root: serde_json::Value) -> Vec<u8> {
        serde_json::json!({
            "asset": { "version": "1.1" },
            "geometricError": 1.0,
            "root": root
        })
        .to_string()
        .into_bytes()
    }

    fn root(children: serde_json::Value) -> serde_json::Value {
        let mut root = serde_json::json!({
            "boundingVolume": { "sphere": [0, 0, 0, 1] },
            "geometricError": 1.0,
        });
        if !children.as_array().is_some_and(Vec::is_empty) {
            root["children"] = children;
        }
        root
    }

    #[test]
    fn walk_uses_identity_for_omitted_transforms() {
        let bytes = tileset_with_root(root(serde_json::json!([])));
        let mut resources = HashMap::from([(String::from("memory://root.json"), bytes)]);
        let mut fetch = |uri: &str| {
            let key = uri.to_string();
            resources
                .remove(&key)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, key))
        };
        let mut visited = 0;

        walk("memory://root.json", &mut fetch, |tile| {
            visited += 1;
            assert_eq!(tile.world_transform()[12], 0.0);
            Ok::<_, std::io::Error>(TraversalControl::Continue)
        })
        .unwrap();

        assert_eq!(visited, 1);
    }

    #[test]
    fn walk_replacement_transform_reaches_external_root() {
        let external_uri = "memory://nested.json";
        let root_bytes = tileset_with_root(root(serde_json::json!([
            {
                "boundingVolume": { "sphere": [0, 0, 0, 1] },
                "geometricError": 0.5,
                "content": { "uri": external_uri }
            }
        ])));
        let nested_bytes = tileset_with_root(root(serde_json::json!([])));
        let mut resources = HashMap::from([
            (String::from("memory://root.json"), root_bytes),
            (String::from(external_uri), nested_bytes),
        ]);
        let mut fetch = |uri: &str| {
            let key = uri.to_string();
            resources
                .remove(&key)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, key))
        };
        let mut external_parent = None;

        walk("memory://root.json", &mut fetch, |tile| {
            if tile.is_external_root {
                external_parent = Some(tile.parent_transform[12]);
            } else if tile.depth == 1 {
                tile.replace_local_transform([
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 1.0,
                ]);
            }
            Ok::<_, std::io::Error>(TraversalControl::Continue)
        })
        .unwrap();

        assert_eq!(external_parent, Some(10.0));
    }
}
