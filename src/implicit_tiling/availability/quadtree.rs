use super::*;

/// Bitmask of known availability states for a single tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TileAvailabilityFlags(pub u8);

impl TileAvailabilityFlags {
    /// The tile itself is available and can be loaded.
    pub const TILE_AVAILABLE: Self = Self(1);
    /// The tile's renderable content is available.
    pub const CONTENT_AVAILABLE: Self = Self(2);
    /// A subtree rooted at this tile is available.
    pub const SUBTREE_AVAILABLE: Self = Self(4);
    /// The subtree rooted at this tile is fully loaded.
    pub const SUBTREE_LOADED: Self = Self(8);
    /// The tile is reachable through the availability tree.
    pub const REACHABLE: Self = Self(16);

    /// Returns an empty (zeroed) flags value.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns `true` when no flags are set.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` when all flags in `other` are set on `self`.
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for TileAvailabilityFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for TileAvailabilityFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A single node in the availability tree, corresponding to one subtree-file.
///
/// `subtree == None` means the subtree is known to be available but has not
/// yet been loaded. `child_nodes[i]` is `None` when the i-th available child
/// subtree has not been added yet.
pub(super) struct AvailabilityNode {
    /// Loaded subtree data; `None` while the subtree file is still loading.
    pub(super) subtree: Option<SubtreeAvailability>,
    /// Child nodes whose length equals the number of available child subtrees.
    pub(super) child_nodes: Vec<Option<Box<AvailabilityNode>>>,
}

impl AvailabilityNode {
    pub(super) fn new() -> Self {
        Self {
            subtree: None,
            child_nodes: Vec::new(),
        }
    }

    /// Attach a loaded subtree and pre-allocate the correct number of child
    /// node slots.
    pub(super) fn set_loaded_subtree(&mut self, subtree: SubtreeAvailability, max_children: usize) {
        let child_count = match subtree.child_subtree_view() {
            AvailabilityView::Constant(true) => max_children,
            AvailabilityView::Constant(false) => 0,
            AvailabilityView::Bitstream(data) => data.iter().map(|b| b.count_ones() as usize).sum(),
        };
        self.subtree = Some(subtree);
        self.child_nodes = (0..child_count).map(|_| None).collect();
    }
}

/// Tree-level availability tracker for a quadtree implicit tileset.
///
/// Wraps an in-memory availability tree (one node per loaded subtree
/// file) and answers per-tile [`TileAvailabilityFlags`] queries.
pub struct QuadtreeAvailability {
    pub(super) subtree_levels: u32,
    pub(super) maximum_level: u32,
    /// `4^subtree_levels` - the maximum number of child subtrees per node.
    pub(super) maximum_children_subtrees: usize,
    pub(super) root: Option<Box<AvailabilityNode>>,
}

impl QuadtreeAvailability {
    /// Create an empty availability tracker.
    ///
    /// `subtree_levels` is the depth of each subtree file;
    /// `maximum_level` is the deepest level carried by the tileset.
    pub fn new(subtree_levels: u32, maximum_level: u32) -> Self {
        let maximum_children_subtrees = 1usize << (subtree_levels * 2);
        Self {
            subtree_levels,
            maximum_level,
            maximum_children_subtrees,
            root: None,
        }
    }

    /// Determine the currently known availability state of `tile_id`.
    ///
    /// Returns a [`TileAvailabilityFlags`] bitmask. An empty (zero) result
    /// means the tile's existence cannot be confirmed from what is loaded.
    pub fn compute_availability(&self, tile_id: QuadtreeTileId) -> TileAvailabilityFlags {
        // Before the root subtree is loaded, the root tile and its subtree are
        // implicitly available.
        if self.root.is_none() && tile_id.level == 0 {
            return TileAvailabilityFlags::TILE_AVAILABLE
                | TileAvailabilityFlags::SUBTREE_AVAILABLE;
        }
        let Some(root) = self.root.as_deref() else {
            return TileAvailabilityFlags::empty();
        };
        if tile_id.level > self.maximum_level {
            return TileAvailabilityFlags::empty();
        }

        let mut level: u32 = 0;
        let mut node: &AvailabilityNode = root;

        // Loop terminates: `level` increases by `subtree_levels` each iteration,
        // and we return once `level > tile_id.level` or no child is found.
        loop {
            debug_assert!(level <= self.maximum_level + self.subtree_levels);
            let subtree = match &node.subtree {
                Some(s) => s,
                None => {
                    // Node exists but subtree file not yet loaded.
                    if tile_id.level == level {
                        return TileAvailabilityFlags::TILE_AVAILABLE
                            | TileAvailabilityFlags::SUBTREE_AVAILABLE;
                    }
                    return TileAvailabilityFlags::empty();
                }
            };

            if tile_id.level < level {
                return TileAvailabilityFlags::empty();
            }

            let levels_left = tile_id.level - level;
            // Mask keeping only the `levels_left` low-order coordinate bits.
            let mask: u32 = if levels_left == 0 {
                0
            } else if levels_left < 32 {
                !(!0u32 << levels_left)
            } else {
                !0u32
            };

            if levels_left < self.subtree_levels {
                // The tile falls within this subtree - read from its bitstreams.
                let mut flags = TileAvailabilityFlags::REACHABLE;

                let rel_x = tile_id.x & mask;
                let rel_y = tile_id.y & mask;
                let relative_morton = morton_2d(rel_x, rel_y) as u64;

                if subtree.is_tile_available_at(levels_left, relative_morton) {
                    flags |= TileAvailabilityFlags::TILE_AVAILABLE;
                }
                if subtree.is_content_available_at(levels_left, relative_morton, 0) {
                    flags |= TileAvailabilityFlags::CONTENT_AVAILABLE;
                }
                // At the subtree root (levelsLeft == 0) the subtree is loaded.
                if levels_left == 0 {
                    flags |= TileAvailabilityFlags::SUBTREE_AVAILABLE
                        | TileAvailabilityFlags::SUBTREE_LOADED;
                }
                return flags;
            }

            // Need to descend to a child subtree.
            let levels_left_after = levels_left - self.subtree_levels;
            let child_rel_x = (tile_id.x & mask) >> levels_left_after;
            let child_rel_y = (tile_id.y & mask) >> levels_left_after;
            let child_morton = morton_2d(child_rel_x, child_rel_y);

            let (child_available, child_index) = Self::resolve_child_subtree(subtree, child_morton);

            if !child_available {
                return TileAvailabilityFlags::REACHABLE;
            }

            // Reborrow child_nodes (the subtree borrow ended above).
            let next = node.child_nodes.get(child_index).and_then(|n| n.as_deref());

            match next {
                Some(child_node) => {
                    node = child_node;
                    level += self.subtree_levels;
                }
                None => {
                    // Child slot exists (available) but not yet loaded.
                    if tile_id.level == level + self.subtree_levels {
                        return TileAvailabilityFlags::TILE_AVAILABLE
                            | TileAvailabilityFlags::SUBTREE_AVAILABLE;
                    }
                    return TileAvailabilityFlags::empty();
                }
            }
        }
    }

    /// Insert a loaded subtree at the tile given by `tile_id`.
    ///
    /// `tile_id` must fall exactly on a subtree boundary (i.e.
    /// `tile_id.level` must be a multiple of `subtree_levels`).
    ///
    /// Returns `true` on success.
    pub fn add_subtree(&mut self, tile_id: QuadtreeTileId, subtree: SubtreeAvailability) -> bool {
        if tile_id.level == 0 {
            if self.root.is_some() {
                return false; // root already set
            }
            let mut node = Box::new(AvailabilityNode::new());
            node.set_loaded_subtree(subtree, self.maximum_children_subtrees);
            self.root = Some(node);
            return true;
        }

        let Some(root) = self.root.as_mut() else {
            return false;
        };

        Self::add_subtree_inner(
            root,
            tile_id,
            0,
            self.subtree_levels,
            self.maximum_children_subtrees,
            subtree,
        )
    }

    /// Recursive insertion helper.
    pub(super) fn add_subtree_inner(
        node: &mut AvailabilityNode,
        tile_id: QuadtreeTileId,
        level: u32,
        subtree_levels: u32,
        max_children: usize,
        new_subtree: SubtreeAvailability,
    ) -> bool {
        // Compute child index - resolved from the immutable subtree borrow.
        let (child_available, child_index, levels_left_after) = {
            let Some(current_subtree) = &node.subtree else {
                return false;
            };
            let levels_left = tile_id.level.saturating_sub(level);
            if levels_left < subtree_levels {
                // Not on a subtree boundary.
                return false;
            }
            let mask: u32 = if levels_left < 32 {
                !(!0u32 << levels_left)
            } else {
                !0u32
            };
            let after = levels_left - subtree_levels;
            let child_rel_x = (tile_id.x & mask) >> after;
            let child_rel_y = (tile_id.y & mask) >> after;
            let child_morton = morton_2d(child_rel_x, child_rel_y);
            let (avail, idx) = Self::resolve_child_subtree(current_subtree, child_morton);
            (avail, idx, after)
        }; // subtree borrow ends here

        if !child_available || child_index >= node.child_nodes.len() {
            return false;
        }

        if levels_left_after == 0 {
            // Direct child - place the new node here.
            if node.child_nodes[child_index].is_some() {
                return false; // already added
            }
            let mut new_node = Box::new(AvailabilityNode::new());
            new_node.set_loaded_subtree(new_subtree, max_children);
            node.child_nodes[child_index] = Some(new_node);
            return true;
        }

        // Need to recurse into the existing child node.
        match node.child_nodes[child_index].as_mut() {
            Some(child_node) => Self::add_subtree_inner(
                child_node,
                tile_id,
                level + subtree_levels,
                subtree_levels,
                max_children,
                new_subtree,
            ),
            None => false,
        }
    }

    /// Given a subtree and a child's Morton index, return
    /// `(is_available, compressed_child_index)`.
    ///
    /// For `Constant(true)`, the uncompressed Morton index is the array index.
    /// For `Bitstream`, the array index is the *rank* (popcount before `m`).
    pub(super) fn resolve_child_subtree(
        subtree: &SubtreeAvailability,
        child_morton: u32,
    ) -> (bool, usize) {
        match subtree.child_subtree_view() {
            AvailabilityView::Constant(true) => (true, child_morton as usize),
            AvailabilityView::Constant(false) => (false, 0),
            AvailabilityView::Bitstream(data) => {
                let byte_idx = (child_morton / 8) as usize;
                let bit_idx = child_morton % 8;
                let bit_set = data
                    .get(byte_idx)
                    .map(|&b| (b >> bit_idx) & 1 == 1)
                    .unwrap_or(false);
                if bit_set {
                    let rank = rank_before(data, child_morton) as usize;
                    (true, rank)
                } else {
                    (false, 0)
                }
            }
        }
    }
}
