use super::*;

/// Insert two 0-bits of spacing between each bit of a 10-bit value.
#[inline]
pub(super) fn spread3(mut i: u32) -> u32 {
    i = (i ^ (i << 16)) & 0x030000ff;
    i = (i ^ (i << 8)) & 0x0300f00f;
    i = (i ^ (i << 4)) & 0x030c30c3;
    i = (i ^ (i << 2)) & 0x09249249;
    i
}

/// 3-D Morton index for `(x, y, z)`.
/// x-bits in positions 0, 3, 6, ...; y-bits in 1, 4, 7, ...; z-bits in 2, 5, 8, ...
/// Each coordinate must fit in 10 bits (<= level 9 within a subtree of depth 10).
#[inline]
pub(super) fn morton_3d(x: u32, y: u32, z: u32) -> u32 {
    spread3(x) | (spread3(y) << 1) | (spread3(z) << 2)
}

/// A single node in the octree availability tree.
pub(super) struct OctreeAvailabilityNode {
    /// Loaded subtree; `None` while the subtree file is still in flight.
    pub(super) subtree: Option<SubtreeAvailability>,
    /// Child nodes (compressed vector - one slot per available child subtree).
    pub(super) child_nodes: Vec<Option<Box<OctreeAvailabilityNode>>>,
}

impl OctreeAvailabilityNode {
    pub(super) fn new() -> Self {
        Self {
            subtree: None,
            child_nodes: Vec::new(),
        }
    }

    pub(super) fn set_loaded_subtree(&mut self, subtree: SubtreeAvailability, max_children: usize) {
        let count = match subtree.child_subtree_view() {
            AvailabilityView::Constant(true) => max_children,
            AvailabilityView::Constant(false) => 0,
            AvailabilityView::Bitstream(data) => data.iter().map(|b| b.count_ones() as usize).sum(),
        };
        self.subtree = Some(subtree);
        self.child_nodes = (0..count).map(|_| None).collect();
    }
}

/// Tree-level availability tracker for an octree implicit tileset.
///
/// Wraps an in-memory octree availability tree and answers
/// per-tile [`TileAvailabilityFlags`] queries.
pub struct OctreeAvailability {
    pub(super) subtree_levels: u32,
    pub(super) maximum_level: u32,
    /// `8^subtree_levels` - maximum child subtrees per node.
    pub(super) maximum_children_subtrees: usize,
    pub(super) root: Option<Box<OctreeAvailabilityNode>>,
}

impl OctreeAvailability {
    /// Create an empty availability tracker.
    pub fn new(subtree_levels: u32, maximum_level: u32) -> Self {
        let maximum_children_subtrees = 1usize << (3 * subtree_levels);
        Self {
            subtree_levels,
            maximum_level,
            maximum_children_subtrees,
            root: None,
        }
    }

    /// Determine the currently known availability state of `tile_id`.
    pub fn compute_availability(&self, tile_id: OctreeTileId) -> TileAvailabilityFlags {
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
        let mut node: &OctreeAvailabilityNode = root;

        loop {
            let subtree = match &node.subtree {
                Some(s) => s,
                None => {
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
            let mask: u32 = if levels_left == 0 {
                0
            } else if levels_left < 32 {
                !(!0u32 << levels_left)
            } else {
                !0u32
            };

            if levels_left < self.subtree_levels {
                let mut flags = TileAvailabilityFlags::REACHABLE;

                let rel_morton =
                    morton_3d(tile_id.x & mask, tile_id.y & mask, tile_id.z & mask) as u64;

                if subtree.is_tile_available_at(levels_left, rel_morton) {
                    flags |= TileAvailabilityFlags::TILE_AVAILABLE;
                }
                if subtree.is_content_available_at(levels_left, rel_morton, 0) {
                    flags |= TileAvailabilityFlags::CONTENT_AVAILABLE;
                }
                if levels_left == 0 {
                    flags |= TileAvailabilityFlags::SUBTREE_AVAILABLE
                        | TileAvailabilityFlags::SUBTREE_LOADED;
                }
                return flags;
            }

            let levels_left_after = levels_left - self.subtree_levels;
            let child_morton = morton_3d(
                (tile_id.x & mask) >> levels_left_after,
                (tile_id.y & mask) >> levels_left_after,
                (tile_id.z & mask) >> levels_left_after,
            );

            let (child_available, child_index) = Self::resolve_child_subtree(subtree, child_morton);

            if !child_available {
                return TileAvailabilityFlags::REACHABLE;
            }

            let next = node.child_nodes.get(child_index).and_then(|n| n.as_deref());

            match next {
                Some(child_node) => {
                    node = child_node;
                    level += self.subtree_levels;
                }
                None => {
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
    /// `tile_id.level` must be a multiple of `subtree_levels`.
    /// Returns `true` on success.
    pub fn add_subtree(&mut self, tile_id: OctreeTileId, subtree: SubtreeAvailability) -> bool {
        if tile_id.level == 0 {
            if self.root.is_some() {
                return false;
            }
            let mut node = Box::new(OctreeAvailabilityNode::new());
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

    pub(super) fn add_subtree_inner(
        node: &mut OctreeAvailabilityNode,
        tile_id: OctreeTileId,
        level: u32,
        subtree_levels: u32,
        max_children: usize,
        new_subtree: SubtreeAvailability,
    ) -> bool {
        let (child_available, child_index, levels_left_after) = {
            let Some(current_subtree) = &node.subtree else {
                return false;
            };
            let levels_left = tile_id.level.saturating_sub(level);
            if levels_left < subtree_levels {
                return false;
            }
            let mask: u32 = if levels_left < 32 {
                !(!0u32 << levels_left)
            } else {
                !0u32
            };
            let after = levels_left - subtree_levels;
            let child_morton = morton_3d(
                (tile_id.x & mask) >> after,
                (tile_id.y & mask) >> after,
                (tile_id.z & mask) >> after,
            );
            let (avail, idx) = Self::resolve_child_subtree(current_subtree, child_morton);
            (avail, idx, after)
        };

        if !child_available || child_index >= node.child_nodes.len() {
            return false;
        }

        if levels_left_after == 0 {
            if node.child_nodes[child_index].is_some() {
                return false;
            }
            let mut new_node = Box::new(OctreeAvailabilityNode::new());
            new_node.set_loaded_subtree(new_subtree, max_children);
            node.child_nodes[child_index] = Some(new_node);
            return true;
        }

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

    /// Resolve `(is_available, compressed_child_index)` from a child's Morton
    /// position and the current subtree's child availability view.
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
