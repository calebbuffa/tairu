//! Extension methods on [`Tileset`] and [`Tile`] matching the Cesium
//! `Cesium3DTiles::Tileset` and `Cesium3DTiles::Tile` augmented API.
//!
//! These are plain `impl` blocks - all methods are available directly on the
//! generated structs without any trait import.

use crate::generated::{Content, Tile, Tileset};
use crate::uri::is_external_tileset_uri;

impl Tile {
    /// Iterate every tile in the subtree rooted at `self` (depth-first,
    /// pre-order) including `self`.
    ///
    /// The callback receives a shared reference to each [`Tile`].
    pub fn for_each_tile<F: FnMut(&Tile)>(&self, mut f: F) {
        self.for_each_tile_with(&mut f);
    }

    fn for_each_tile_with(&self, f: &mut dyn FnMut(&Tile)) {
        f(self);
        for child in &self.children {
            child.for_each_tile_with(f);
        }
    }

    /// Mutable variant of [`for_each_tile`](Self::for_each_tile).
    pub fn for_each_tile_mut<F: FnMut(&mut Tile)>(&mut self, mut f: F) {
        self.for_each_tile_mut_with(&mut f);
    }

    fn for_each_tile_mut_with(&mut self, f: &mut dyn FnMut(&mut Tile)) {
        f(self);
        for child in &mut self.children {
            child.for_each_tile_mut_with(f);
        }
    }

    /// Iterate every [`Content`] in the subtree rooted at `self`
    /// (depth-first, pre-order).
    ///
    /// Both `tile.content` (single) and `tile.contents` (multi) are visited.
    pub fn for_each_content<F: FnMut(&Content)>(&self, mut f: F) {
        self.for_each_content_with(&mut f);
    }

    fn for_each_content_with(&self, f: &mut dyn FnMut(&Content)) {
        if let Some(c) = &self.content {
            f(c);
        }
        for c in &self.contents {
            f(c);
        }
        for child in &self.children {
            child.for_each_content_with(f);
        }
    }

    /// Mutable variant of [`for_each_content`](Self::for_each_content).
    pub fn for_each_content_mut<F: FnMut(&mut Content)>(&mut self, mut f: F) {
        self.for_each_content_mut_with(&mut f);
    }

    fn for_each_content_mut_with(&mut self, f: &mut dyn FnMut(&mut Content)) {
        if let Some(c) = &mut self.content {
            f(c);
        }
        for c in &mut self.contents {
            f(c);
        }
        for child in &mut self.children {
            child.for_each_content_mut_with(f);
        }
    }

    /// Returns `true` if this tile is a leaf tile.
    ///
    /// A leaf tile has:
    /// - No child tiles
    /// - No implicit tiling
    /// - No external tileset links in its content
    pub fn is_leaf(&self) -> bool {
        // Not a leaf if has children or implicit tiling
        if !self.children.is_empty() || self.implicit_tiling.is_some() {
            return false;
        }

        // Check that no content links to an external tileset
        let has_external_tileset = self
            .content
            .iter()
            .chain(&self.contents)
            .any(|c| is_external_tileset_uri(&c.uri));

        !has_external_tileset
    }
}

impl Tileset {
    /// Iterate every tile in the tileset tree (depth-first, pre-order).
    pub fn for_each_tile<F: FnMut(&Tile)>(&self, mut f: F) {
        self.root.for_each_tile(&mut f);
    }

    /// Mutable variant of [`for_each_tile`](Self::for_each_tile).
    pub fn for_each_tile_mut<F: FnMut(&mut Tile)>(&mut self, mut f: F) {
        self.root.for_each_tile_mut(&mut f);
    }

    /// Iterate every [`Content`] in the tileset tree (depth-first, pre-order).
    pub fn for_each_content<F: FnMut(&Content)>(&self, mut f: F) {
        self.root.for_each_content(&mut f);
    }

    /// Mutable variant of [`for_each_content`](Self::for_each_content).
    pub fn for_each_content_mut<F: FnMut(&mut Content)>(&mut self, mut f: F) {
        self.root.for_each_content_mut(&mut f);
    }

    /// Declare that an extension is used somewhere in this tileset.
    ///
    /// Idempotent - calling multiple times with the same name is safe.
    pub fn add_extension_used(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.extensions_used.contains(&name) {
            self.extensions_used.push(name);
        }
    }

    /// Declare that an extension is **required** (and implicitly used).
    ///
    /// Adds to both `extensionsRequired` and `extensionsUsed`. Idempotent.
    pub fn add_extension_required(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.extensions_required.contains(&name) {
            self.extensions_required.push(name.clone());
        }
        self.add_extension_used(name);
    }

    /// Remove a name from `extensionsUsed`. Also removes it from
    /// `extensionsRequired` if present. Idempotent.
    pub fn remove_extension_used(&mut self, name: &str) {
        self.extensions_used.retain(|n| n != name);
        self.extensions_required.retain(|n| n != name);
    }

    /// Remove a name from `extensionsRequired` only (keeps it in
    /// `extensionsUsed`). Idempotent.
    pub fn remove_extension_required(&mut self, name: &str) {
        self.extensions_required.retain(|n| n != name);
    }

    /// Returns `true` if the given extension name is in `extensionsUsed`.
    pub fn is_extension_used(&self, name: &str) -> bool {
        self.extensions_used.iter().any(|n| n == name)
    }

    /// Returns `true` if the given extension name is in `extensionsRequired`.
    pub fn is_extension_required(&self, name: &str) -> bool {
        self.extensions_required.iter().any(|n| n == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::{Asset, BoundingVolume};

    fn leaf(uri: &str) -> Tile {
        Tile {
            bounding_volume: BoundingVolume {
                sphere: Some([0.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            },
            geometric_error: 0.0,
            content: Some(Content {
                uri: uri.into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn tree() -> Tileset {
        let mut root = leaf("root.glb");
        root.children.push(leaf("child_a.glb"));
        root.children.push(leaf("child_b.glb"));
        Tileset {
            asset: Asset {
                version: "1.1".into(),
                ..Default::default()
            },
            geometric_error: 1000.0,
            root,
            ..Default::default()
        }
    }

    #[test]
    fn for_each_tile_visits_all() {
        let ts = tree();
        let mut uris: Vec<String> = Vec::new();
        ts.for_each_content(|c| uris.push(c.uri.clone()));
        assert_eq!(uris, ["root.glb", "child_a.glb", "child_b.glb"]);
    }

    #[test]
    fn for_each_tile_count() {
        let ts = tree();
        let mut count = 0;
        ts.for_each_tile(|_| count += 1);
        assert_eq!(count, 3); // root + 2 children
    }

    #[test]
    fn extension_helpers() {
        let mut ts = tree();
        ts.add_extension_required("3DTILES_metadata");
        assert!(ts.is_extension_required("3DTILES_metadata"));
        assert!(ts.is_extension_used("3DTILES_metadata"));

        ts.remove_extension_required("3DTILES_metadata");
        assert!(!ts.is_extension_required("3DTILES_metadata"));
        assert!(ts.is_extension_used("3DTILES_metadata")); // still used

        ts.remove_extension_used("3DTILES_metadata");
        assert!(!ts.is_extension_used("3DTILES_metadata"));
    }

    #[test]
    fn add_extension_used_idempotent() {
        let mut ts = tree();
        ts.add_extension_used("EXT_foo");
        ts.add_extension_used("EXT_foo");
        assert_eq!(
            ts.extensions_used
                .iter()
                .filter(|n| *n == "EXT_foo")
                .count(),
            1
        );
    }
}
