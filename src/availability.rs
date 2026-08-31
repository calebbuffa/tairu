//! Subtree availability for 3D Tiles implicit tiling.
//!
//! A *subtree* is a fixed-depth slice of an implicit tile tree.  Its
//! availability data describes, for every tile/content/child-subtree position
//! within the slice, whether that item exists.  Availability is stored either
//! as a single constant (all available / all unavailable) or as a packed
//! bitstream where each bit corresponds to one Morton-indexed position.
//!
//! # Bit-index formula
//!
//! For a tile at relative `(level, morton_id)` inside a subtree:
//!
//! ```text
//! prefix = (child_count^level - 1) / (child_count - 1)
//!        = sum of tiles in levels 0 .. level-1
//! bit_index = prefix + morton_id
//! byte = data[bit_index / 8]
//! bit  = (byte >> (bit_index % 8)) & 1
//! ```
//!
//! For child-subtree availability the "level" is always 0, so `prefix = 0`
//! and `bit_index = morton_id` directly (the root of each potential child
//! subtree is one level below the bottom of the current subtree).

// Re-export SubdivisionScheme so existing code using `crate::availability::SubdivisionScheme` works.
use crate::generated::SubdivisionScheme;

/// Quadtree tile coordinates — local to avoid external math-library deps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadtreeTileId {
    /// Level within the quadtree (0 = root).
    pub level: u32,
    /// X coordinate at this level.
    pub x: u32,
    /// Y coordinate at this level.
    pub y: u32,
}

impl QuadtreeTileId {
    /// Construct a tile ID from its level and coordinates.
    pub fn new(level: u32, x: u32, y: u32) -> Self {
        Self { level, x, y }
    }

    /// Morton index for (x, y) within this tile's level.
    pub fn morton_index(self) -> u64 {
        morton_2d(self.x, self.y) as u64
    }

    /// Tile coordinates relative to the given subtree-root tile.
    pub fn relative_to(self, subtree_id: Self) -> Self {
        debug_assert!(self.level >= subtree_id.level);
        let diff = self.level - subtree_id.level;
        Self {
            level: diff,
            x: self.x.wrapping_sub(subtree_id.x << diff),
            y: self.y.wrapping_sub(subtree_id.y << diff),
        }
    }

    /// Morton index of `child` relative to this subtree's bottom level.
    pub fn relative_morton_index(self, child: Self) -> u64 {
        let rel = child.relative_to(self);
        morton_2d(rel.x, rel.y) as u64
    }

    /// The four child tile IDs at `level + 1`, in Morton order.
    pub fn children(self) -> [QuadtreeTileId; 4] {
        let cl = self.level + 1;
        let (bx, by) = (self.x * 2, self.y * 2);
        [
            QuadtreeTileId::new(cl, bx, by),
            QuadtreeTileId::new(cl, bx + 1, by),
            QuadtreeTileId::new(cl, bx, by + 1),
            QuadtreeTileId::new(cl, bx + 1, by + 1),
        ]
    }

    /// The root tile ID of the fixed-size subtree that contains this tile.
    pub fn subtree_root(self, subtree_levels: u32) -> QuadtreeTileId {
        let subtree_level = (self.level / subtree_levels) * subtree_levels;
        let levels_down = self.level - subtree_level;
        QuadtreeTileId::new(subtree_level, self.x >> levels_down, self.y >> levels_down)
    }
}

/// Octree tile coordinates — local to avoid external math-library deps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OctreeTileId {
    /// Level within the octree (0 = root).
    pub level: u32,
    /// X coordinate at this level.
    pub x: u32,
    /// Y coordinate at this level.
    pub y: u32,
    /// Z coordinate at this level.
    pub z: u32,
}

impl OctreeTileId {
    /// Construct a tile ID from its level and coordinates.
    pub fn new(level: u32, x: u32, y: u32, z: u32) -> Self {
        Self { level, x, y, z }
    }

    /// Morton index for (x, y, z) within this tile's level.
    pub fn morton_index(self) -> u64 {
        morton_3d(self.x, self.y, self.z) as u64
    }

    /// Tile coordinates relative to the given subtree-root tile.
    pub fn relative_to(self, subtree_id: Self) -> Self {
        debug_assert!(self.level >= subtree_id.level);
        let diff = self.level - subtree_id.level;
        Self {
            level: diff,
            x: self.x.wrapping_sub(subtree_id.x << diff),
            y: self.y.wrapping_sub(subtree_id.y << diff),
            z: self.z.wrapping_sub(subtree_id.z << diff),
        }
    }

    /// Morton index of `child` relative to this subtree's bottom level.
    pub fn relative_morton_index(self, child: Self) -> u64 {
        let rel = child.relative_to(self);
        morton_3d(rel.x, rel.y, rel.z) as u64
    }

    /// The eight child tile IDs at `level + 1`, in Morton order.
    pub fn children(self) -> [OctreeTileId; 8] {
        let cl = self.level + 1;
        let (bx, by, bz) = (self.x * 2, self.y * 2, self.z * 2);
        [
            OctreeTileId::new(cl, bx, by, bz),
            OctreeTileId::new(cl, bx + 1, by, bz),
            OctreeTileId::new(cl, bx, by + 1, bz),
            OctreeTileId::new(cl, bx + 1, by + 1, bz),
            OctreeTileId::new(cl, bx, by, bz + 1),
            OctreeTileId::new(cl, bx + 1, by, bz + 1),
            OctreeTileId::new(cl, bx, by + 1, bz + 1),
            OctreeTileId::new(cl, bx + 1, by + 1, bz + 1),
        ]
    }

    /// The root tile ID of the fixed-size subtree that contains this tile.
    pub fn subtree_root(self, subtree_levels: u32) -> OctreeTileId {
        let subtree_level = (self.level / subtree_levels) * subtree_levels;
        let d = self.level - subtree_level;
        OctreeTileId::new(subtree_level, self.x >> d, self.y >> d, self.z >> d)
    }
}

impl SubdivisionScheme {
    pub(crate) fn child_count(self) -> u64 {
        match self {
            Self::Quadtree | Self::S2 => 4,
            Self::Octree => 8,
        }
    }

    /// The power-of-two exponent used to compute the number of tiles at a
    /// given level: `tiles_at_level = 1 << (power * level)`.
    pub(crate) fn power(self) -> u32 {
        match self {
            Self::Quadtree | Self::S2 => 2,
            Self::Octree => 3,
        }
    }
}

/// A single availability entry: either a constant or a packed bitstream.
#[derive(Debug, Clone)]
pub enum AvailabilityView {
    /// All tiles share the same state (`true` = all available).
    Constant(bool),
    /// Per-tile availability packed as a bit array (LSB-first within each byte).
    Bitstream(Vec<u8>),
}

impl AvailabilityView {
    /// Return `true` if the bit at `bit_index` is set (for `Bitstream`),
    /// or return the constant for `Constant`.
    fn is_set(&self, bit_index: u64) -> bool {
        match self {
            Self::Constant(v) => *v,
            Self::Bitstream(data) => {
                let byte_idx = (bit_index / 8) as usize;
                if byte_idx >= data.len() {
                    return false;
                }
                (data[byte_idx] >> (bit_index % 8)) & 1 == 1
            }
        }
    }

    /// Set or clear the bit at `bit_index` (no-op for `Constant`).
    fn set_bit(&mut self, bit_index: u64, value: bool) {
        if let Self::Bitstream(data) = self {
            let byte_idx = (bit_index / 8) as usize;
            if byte_idx < data.len() {
                let shift = (bit_index % 8) as u8;
                if value {
                    data[byte_idx] |= 1u8 << shift;
                } else {
                    data[byte_idx] &= !(1u8 << shift);
                }
            }
        }
    }

    /// Expand a `Constant` into a `Bitstream` large enough for `bit_count`
    /// bits, initialising every bit to the constant value.  A `Bitstream` is
    /// left unchanged.
    fn expand_constant(&mut self, bit_count: u64) {
        if let Self::Constant(v) = *self {
            let byte_count = bit_count.div_ceil(8) as usize;
            let fill = if v { 0xFF } else { 0x00 };
            *self = Self::Bitstream(vec![fill; byte_count]);
        }
    }
}

/// Availability index for one subtree in an implicit tile tree.
///
/// Wraps three [`AvailabilityView`]s - tile, content, and child-subtree
/// availability - and exposes query/mutation methods that accept either
/// absolute tile IDs (converted internally to relative Morton indices) or
/// raw `(relative_level, morton_id)` pairs.
///
/// Constructed via [`SubtreeAvailability::new`] or the convenience
/// constructors [`SubtreeAvailability::all_available`] /
/// [`SubtreeAvailability::all_unavailable`].
#[derive(Debug, Clone)]
pub struct SubtreeAvailability {
    scheme: SubdivisionScheme,
    levels: u32,
    tile_availability: AvailabilityView,
    child_subtree_availability: AvailabilityView,
    /// One entry per content layer (most tilesets have exactly one).
    content_availability: Vec<AvailabilityView>,
}

impl SubtreeAvailability {
    /// Construct from pre-built availability views.
    ///
    /// `content_availability` must have at least one element.
    pub fn new(
        scheme: SubdivisionScheme,
        levels: u32,
        tile_availability: AvailabilityView,
        child_subtree_availability: AvailabilityView,
        content_availability: Vec<AvailabilityView>,
    ) -> Option<Self> {
        if content_availability.is_empty() {
            return None;
        }
        Some(Self {
            scheme,
            levels,
            tile_availability,
            child_subtree_availability,
            content_availability,
        })
    }

    /// Create a subtree where every tile is available and no content/child
    /// subtrees are available.
    pub fn all_available(scheme: SubdivisionScheme, levels: u32) -> Self {
        Self {
            scheme,
            levels,
            tile_availability: AvailabilityView::Constant(true),
            child_subtree_availability: AvailabilityView::Constant(false),
            content_availability: vec![AvailabilityView::Constant(false)],
        }
    }

    /// Create a subtree where nothing is available.
    pub fn all_unavailable(scheme: SubdivisionScheme, levels: u32) -> Self {
        Self {
            scheme,
            levels,
            tile_availability: AvailabilityView::Constant(false),
            child_subtree_availability: AvailabilityView::Constant(false),
            content_availability: vec![AvailabilityView::Constant(false)],
        }
    }

    /// Tile availability for a quadtree tile, given the ID of the subtree containing it.
    pub fn is_tile_available_quad(
        &self,
        subtree_id: QuadtreeTileId,
        tile_id: QuadtreeTileId,
    ) -> bool {
        let relative = tile_id.relative_to(subtree_id);
        self.is_tile_available(relative.level, relative.morton_index())
    }

    /// Tile availability for an octree tile, given the ID of the subtree containing it.
    pub fn is_tile_available_oct(&self, subtree_id: OctreeTileId, tile_id: OctreeTileId) -> bool {
        let relative = tile_id.relative_to(subtree_id);
        self.is_tile_available(relative.level, relative.morton_index())
    }

    /// Tile availability at a relative level and Morton index within this subtree.
    pub fn is_tile_available(&self, relative_level: u32, morton_id: u64) -> bool {
        self.is_available(relative_level, morton_id, &self.tile_availability)
    }

    /// Mark a quadtree tile available/unavailable, given the ID of the subtree containing it.
    pub fn set_tile_available_quad(
        &mut self,
        subtree_id: QuadtreeTileId,
        tile_id: QuadtreeTileId,
        available: bool,
    ) {
        let relative = tile_id.relative_to(subtree_id);
        self.set_available(relative.level, relative.morton_index(), available, false);
    }

    /// Mark an octree tile available/unavailable, given the ID of the subtree containing it.
    pub fn set_tile_available_oct(
        &mut self,
        subtree_id: OctreeTileId,
        tile_id: OctreeTileId,
        available: bool,
    ) {
        let relative = tile_id.relative_to(subtree_id);
        self.set_available(relative.level, relative.morton_index(), available, false);
    }

    /// Mark a tile available/unavailable at a relative level and Morton index.
    pub fn set_tile_available(&mut self, relative_level: u32, morton_id: u64, available: bool) {
        self.set_available(relative_level, morton_id, available, false);
    }

    /// Content availability for a quadtree tile's `content_id`-th content.
    pub fn is_content_available_quad(
        &self,
        subtree_id: QuadtreeTileId,
        tile_id: QuadtreeTileId,
        content_id: usize,
    ) -> bool {
        let relative = tile_id.relative_to(subtree_id);
        self.is_content_available(relative.level, relative.morton_index(), content_id)
    }

    /// Content availability for an octree tile's `content_id`-th content.
    pub fn is_content_available_oct(
        &self,
        subtree_id: OctreeTileId,
        tile_id: OctreeTileId,
        content_id: usize,
    ) -> bool {
        let relative = tile_id.relative_to(subtree_id);
        self.is_content_available(relative.level, relative.morton_index(), content_id)
    }

    /// Content availability at a relative level, Morton index, and content index.
    pub fn is_content_available(
        &self,
        relative_level: u32,
        morton_id: u64,
        content_id: usize,
    ) -> bool {
        match self.content_availability.get(content_id) {
            Some(view) => self.is_available(relative_level, morton_id, view),
            None => false,
        }
    }

    /// Check whether the child subtree identified by its Morton index
    /// (relative to this subtree root) is available.
    pub fn is_child_subtree_available(&self, relative_morton_id: u64) -> bool {
        self.child_subtree_availability.is_set(relative_morton_id)
    }

    /// Child-subtree availability by quadtree tile IDs.
    pub fn is_child_subtree_available_quad(
        &self,
        this_subtree_id: QuadtreeTileId,
        child_subtree_id: QuadtreeTileId,
    ) -> bool {
        let morton = this_subtree_id.relative_morton_index(child_subtree_id);
        self.is_child_subtree_available(morton)
    }

    /// Child-subtree availability by octree tile IDs.
    pub fn is_child_subtree_available_oct(
        &self,
        this_subtree_id: OctreeTileId,
        child_subtree_id: OctreeTileId,
    ) -> bool {
        let morton = this_subtree_id.relative_morton_index(child_subtree_id);
        self.is_child_subtree_available(morton)
    }

    /// Mark the child subtree at a relative Morton index available/unavailable.
    pub fn set_child_subtree_available(&mut self, relative_morton_id: u64, available: bool) {
        // For child-subtree availability: the layout is a flat bitstream
        // (conceptually level 0 of the *next* subtree level), so prefix = 0.
        let total_child_subtrees = 1u64 << (self.scheme.power() * self.levels);
        self.child_subtree_availability
            .expand_constant(total_child_subtrees);
        self.child_subtree_availability
            .set_bit(relative_morton_id, available);
    }

    /// Direct access to the raw child-subtree availability view.
    ///
    /// Used by [`super::quadtree_availability::QuadtreeAvailability`] to
    /// determine the child node count and compute the compressed child index.
    pub fn child_subtree_view(&self) -> &AvailabilityView {
        &self.child_subtree_availability
    }

    /// Returns `true` if any tile within `range` is available in this subtree.
    ///
    /// Mirrors `CesiumGeometry::QuadtreeTileRectangularRange` query semantics.
    pub fn any_tile_available_in_range(&self, range: &QuadtreeTileRectangularRange) -> bool {
        for y in range.minimum_y..=range.maximum_y {
            for x in range.minimum_x..=range.maximum_x {
                let morton = morton_2d(x, y) as u64;
                if self.is_tile_available(range.level, morton) {
                    return true;
                }
            }
        }
        false
    }

    /// The sum of `child_count^i` for `i in 0..level`.
    ///
    /// This is the bit-index offset for the first tile at `level`:
    /// `(child_count^level - 1) / (child_count - 1)`
    fn prefix_for_level(&self, level: u32) -> u64 {
        let child_count = self.scheme.child_count();
        let tiles_at_level = 1u64 << (self.scheme.power() * level);
        // sum of geometric series: (r^n - 1) / (r - 1)
        (tiles_at_level - 1) / (child_count - 1)
    }

    fn is_available(&self, relative_level: u32, morton_id: u64, view: &AvailabilityView) -> bool {
        let tiles_at_level = 1u64 << (self.scheme.power() * relative_level);
        if morton_id >= tiles_at_level {
            return false;
        }
        let prefix = self.prefix_for_level(relative_level);
        view.is_set(prefix + morton_id)
    }

    /// Mutate `tile_availability` or `content_availability[content_id]`.
    /// `is_content` drives which view is mutated; tile availability is used
    /// when `is_content` is false.
    fn set_available(
        &mut self,
        relative_level: u32,
        morton_id: u64,
        value: bool,
        is_content: bool,
    ) {
        // Total bits needed = prefix_for_level(levels) = total tiles in subtree.
        let total_tiles = self.prefix_for_level(self.levels);
        let prefix = self.prefix_for_level(relative_level);
        let bit_index = prefix + morton_id;

        let view = if is_content {
            let Some(v) = self.content_availability.first_mut() else {
                return;
            };
            v
        } else {
            &mut self.tile_availability
        };

        view.expand_constant(total_tiles);
        view.set_bit(bit_index, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_subtree(all_avail: bool) -> SubtreeAvailability {
        if all_avail {
            SubtreeAvailability::all_available(SubdivisionScheme::Quadtree, 2)
        } else {
            SubtreeAvailability::all_unavailable(SubdivisionScheme::Quadtree, 2)
        }
    }

    #[test]
    fn constant_all_available() {
        let sa = quad_subtree(true);
        // Root
        assert!(sa.is_tile_available(0, 0));
        // All 4 level-1 tiles
        for m in 0..4 {
            assert!(sa.is_tile_available(1, m));
        }
    }

    #[test]
    fn constant_all_unavailable() {
        let sa = quad_subtree(false);
        assert!(!sa.is_tile_available(0, 0));
        for m in 0..4 {
            assert!(!sa.is_tile_available(1, m));
        }
    }

    #[test]
    fn set_tile_available_expands_constant() {
        let mut sa = quad_subtree(false);
        // Mark only the root available.
        sa.set_tile_available(0, 0, true);
        assert!(sa.is_tile_available(0, 0));
        assert!(!sa.is_tile_available(1, 0));
    }

    #[test]
    fn set_and_query_individual_tiles() {
        let mut sa = quad_subtree(false);
        sa.set_tile_available(0, 0, true);
        sa.set_tile_available(1, 2, true); // bottom-left level-1 child
        assert!(sa.is_tile_available(0, 0));
        assert!(!sa.is_tile_available(1, 0));
        assert!(!sa.is_tile_available(1, 1));
        assert!(sa.is_tile_available(1, 2));
        assert!(!sa.is_tile_available(1, 3));
    }

    #[test]
    fn child_subtree_initially_unavailable() {
        let sa = quad_subtree(true);
        // child_subtree_availability defaults to Constant(false)
        assert!(!sa.is_child_subtree_available(0));
    }

    #[test]
    fn set_child_subtree_available() {
        let mut sa = quad_subtree(true);
        sa.set_child_subtree_available(3, true);
        assert!(!sa.is_child_subtree_available(0));
        assert!(sa.is_child_subtree_available(3));
    }

    #[test]
    fn is_tile_available_via_quad_ids() {
        let mut sa = SubtreeAvailability::all_unavailable(SubdivisionScheme::Quadtree, 2);
        let root = QuadtreeTileId::new(0, 0, 0);
        let child = QuadtreeTileId::new(1, 1, 0); // Morton index = 1
        sa.set_tile_available_quad(root, child, true);
        assert!(sa.is_tile_available_quad(root, child));
        assert!(!sa.is_tile_available_quad(root, QuadtreeTileId::new(1, 0, 0)));
    }

    #[test]
    fn octree_all_available() {
        let sa = SubtreeAvailability::all_available(SubdivisionScheme::Octree, 1);
        assert!(sa.is_tile_available(0, 0));
        for m in 0..8 {
            assert!(sa.is_tile_available(1, m));
        }
    }

    #[test]
    fn new_requires_nonempty_content() {
        assert!(
            SubtreeAvailability::new(
                SubdivisionScheme::Quadtree,
                2,
                AvailabilityView::Constant(true),
                AvailabilityView::Constant(false),
                vec![],
            )
            .is_none()
        );
    }
}

/// 2-D Morton index for `(x, y)`: x-bits in odd positions, y-bits in even.
/// Both `x` and `y` must fit in 16 bits.
fn morton_2d(x: u32, y: u32) -> u32 {
    fn spread(mut v: u32) -> u32 {
        v &= 0x0000_FFFF;
        v = (v | (v << 8)) & 0x00FF_00FF;
        v = (v | (v << 4)) & 0x0F0F_0F0F;
        v = (v | (v << 2)) & 0x3333_3333;
        v = (v | (v << 1)) & 0x5555_5555;
        v
    }
    spread(x) | (spread(y) << 1)
}

/// Count the number of set bits at positions `0..bit_pos` (exclusive) in
/// `data`. Equivalent to the *rank* of position `bit_pos` in the bitstream.
pub(crate) fn rank_before(data: &[u8], bit_pos: u32) -> u32 {
    let byte_idx = (bit_pos / 8) as usize;
    let bit_idx = bit_pos % 8;
    let full: u32 = data.iter().take(byte_idx).map(|b| b.count_ones()).sum();
    let partial = data
        .get(byte_idx)
        .map(|&b| (b & ((1u8 << bit_idx).wrapping_sub(1))).count_ones())
        .unwrap_or(0);
    full + partial
}

/// Bitmask of known availability states for a single tile.
///
/// Mirrors `CesiumGeometry::TileAvailabilityFlags`.
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
pub struct AvailabilityNode {
    /// Loaded subtree data; `None` while the subtree file is still loading.
    pub subtree: Option<SubtreeAvailability>,
    /// Child nodes whose length equals the number of available child subtrees.
    pub child_nodes: Vec<Option<Box<AvailabilityNode>>>,
}

impl AvailabilityNode {
    fn new() -> Self {
        Self {
            subtree: None,
            child_nodes: Vec::new(),
        }
    }

    /// Attach a loaded subtree and pre-allocate the correct number of child
    /// node slots.
    fn set_loaded_subtree(&mut self, subtree: SubtreeAvailability, max_children: usize) {
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
/// Wraps an in-memory tree of [`AvailabilityNode`]s (one per loaded subtree
/// file) and answers per-tile [`TileAvailabilityFlags`] queries.
///
/// Mirrors `CesiumGeometry::QuadtreeAvailability`.
pub struct QuadtreeAvailability {
    subtree_levels: u32,
    maximum_level: u32,
    /// `4^subtree_levels` - the maximum number of child subtrees per node.
    maximum_children_subtrees: usize,
    root: Option<Box<AvailabilityNode>>,
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

                if subtree.is_tile_available(levels_left, relative_morton) {
                    flags |= TileAvailabilityFlags::TILE_AVAILABLE;
                }
                if subtree.is_content_available(levels_left, relative_morton, 0) {
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
    fn add_subtree_inner(
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
    fn resolve_child_subtree(subtree: &SubtreeAvailability, child_morton: u32) -> (bool, usize) {
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

#[cfg(test)]
mod quadtree_availability_tests {
    use super::*;

    fn root_subtree_all_tiles_no_children(levels: u32) -> SubtreeAvailability {
        SubtreeAvailability::new(
            SubdivisionScheme::Quadtree,
            levels,
            AvailabilityView::Constant(true),
            AvailabilityView::Constant(false),
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap()
    }

    #[test]
    fn empty_root_level_0_is_implicitly_available() {
        let qa = QuadtreeAvailability::new(2, 4);
        let flags = qa.compute_availability(QuadtreeTileId {
            level: 0,
            x: 0,
            y: 0,
        });
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
    }

    #[test]
    fn empty_tracker_non_root_returns_empty() {
        let qa = QuadtreeAvailability::new(2, 4);
        let flags = qa.compute_availability(QuadtreeTileId {
            level: 1,
            x: 0,
            y: 0,
        });
        assert!(flags.is_empty());
    }

    #[test]
    fn beyond_maximum_level_returns_empty() {
        let qa = QuadtreeAvailability::new(2, 2);
        let flags = qa.compute_availability(QuadtreeTileId {
            level: 3,
            x: 0,
            y: 0,
        });
        assert!(flags.is_empty());
    }

    #[test]
    fn root_subtree_all_tiles_available() {
        let mut qa = QuadtreeAvailability::new(2, 4);
        let subtree = root_subtree_all_tiles_no_children(2);
        assert!(qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0
            },
            subtree
        ));

        // Root tile should be reachable, tile available, subtree loaded.
        let flags = qa.compute_availability(QuadtreeTileId {
            level: 0,
            x: 0,
            y: 0,
        });
        assert!(flags.contains(TileAvailabilityFlags::REACHABLE));
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_LOADED));
    }

    #[test]
    fn tile_within_root_subtree_reachable_and_available() {
        let mut qa = QuadtreeAvailability::new(2, 4);
        let subtree = root_subtree_all_tiles_no_children(2);
        qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0,
            },
            subtree,
        );

        // A 2-level subtree covers relative levels 0 and 1 (levels_left in [0,1]).
        // Level 2 is the root of a child subtree - NOT within this subtree.
        for &(level, x, y) in &[(1u32, 0u32, 0u32), (1, 1, 0), (1, 1, 1)] {
            let flags = qa.compute_availability(QuadtreeTileId { level, x, y });
            assert!(
                flags.contains(TileAvailabilityFlags::REACHABLE),
                "level={level} x={x} y={y} should be REACHABLE"
            );
            assert!(
                flags.contains(TileAvailabilityFlags::TILE_AVAILABLE),
                "level={level} x={x} y={y} should be TILE_AVAILABLE"
            );
        }
    }

    #[test]
    fn child_subtree_root_not_yet_loaded_reports_available_not_loaded() {
        // 2-level root subtree, child subtrees at level 2 all marked available.
        let mut qa = QuadtreeAvailability::new(2, 6);
        let subtree = SubtreeAvailability::new(
            SubdivisionScheme::Quadtree,
            2,
            AvailabilityView::Constant(true),
            AvailabilityView::Constant(true), // all child subtrees available
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap();
        qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0,
            },
            subtree,
        );

        // Level 2 is the root of a child subtree; it's available but not loaded.
        let flags = qa.compute_availability(QuadtreeTileId {
            level: 2,
            x: 0,
            y: 0,
        });
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
        // SUBTREE_LOADED should NOT be set - the subtree file wasn't added.
        assert!(!flags.contains(TileAvailabilityFlags::SUBTREE_LOADED));
    }

    #[test]
    fn add_root_twice_returns_false_second_time() {
        let mut qa = QuadtreeAvailability::new(2, 4);
        let s1 = root_subtree_all_tiles_no_children(2);
        let s2 = root_subtree_all_tiles_no_children(2);
        assert!(qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0
            },
            s1
        ));
        assert!(!qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0
            },
            s2
        ));
    }

    #[test]
    fn bitstream_child_availability_selects_correct_slot() {
        // 1-level subtree: one root tile, 4 potential child subtrees (at level 1).
        // Mark only Morton index 0 (x=0, y=0) child subtree as available.
        let bit_byte: u8 = 0b0000_0001; // only bit 0 set
        let root_subtree = SubtreeAvailability::new(
            SubdivisionScheme::Quadtree,
            1,
            AvailabilityView::Constant(true),
            AvailabilityView::Bitstream(vec![bit_byte]),
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap();

        let mut qa = QuadtreeAvailability::new(1, 4);
        qa.add_subtree(
            QuadtreeTileId {
                level: 0,
                x: 0,
                y: 0,
            },
            root_subtree,
        );

        // Child subtree at (1,0,0) is available and can be added.
        let child_subtree = root_subtree_all_tiles_no_children(1);
        assert!(
            qa.add_subtree(
                QuadtreeTileId {
                    level: 1,
                    x: 0,
                    y: 0
                },
                child_subtree
            ),
            "should successfully add child at morton-0"
        );

        // Adding at (1,1,0) should fail - not marked available.
        let child_subtree2 = root_subtree_all_tiles_no_children(1);
        assert!(
            !qa.add_subtree(
                QuadtreeTileId {
                    level: 1,
                    x: 1,
                    y: 0
                },
                child_subtree2
            ),
            "child at morton-2 not available, add should return false"
        );
    }

    #[test]
    fn flags_bitor_and_contains() {
        let f = TileAvailabilityFlags::TILE_AVAILABLE | TileAvailabilityFlags::REACHABLE;
        assert!(f.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(f.contains(TileAvailabilityFlags::REACHABLE));
        assert!(!f.contains(TileAvailabilityFlags::CONTENT_AVAILABLE));
    }

    #[test]
    fn flags_default_is_empty() {
        assert!(TileAvailabilityFlags::default().is_empty());
    }
}

/// Insert two 0-bits of spacing between each bit of a 10-bit value.
#[inline]
fn spread3(mut i: u32) -> u32 {
    i = (i ^ (i << 16)) & 0x030000ff;
    i = (i ^ (i << 8)) & 0x0300f00f;
    i = (i ^ (i << 4)) & 0x030c30c3;
    i = (i ^ (i << 2)) & 0x09249249;
    i
}

/// 3-D Morton index for `(x, y, z)`.
/// x-bits in positions 0, 3, 6, …; y-bits in 1, 4, 7, …; z-bits in 2, 5, 8, …
/// Each coordinate must fit in 10 bits (<= level 9 within a subtree of depth 10).
#[inline]
fn morton_3d(x: u32, y: u32, z: u32) -> u32 {
    spread3(x) | (spread3(y) << 1) | (spread3(z) << 2)
}

/// A single node in the octree availability tree.
pub struct OctreeAvailabilityNode {
    /// Loaded subtree; `None` while the subtree file is still in flight.
    pub subtree: Option<SubtreeAvailability>,
    /// Child nodes (compressed vector - one slot per available child subtree).
    pub child_nodes: Vec<Option<Box<OctreeAvailabilityNode>>>,
}

impl OctreeAvailabilityNode {
    fn new() -> Self {
        Self {
            subtree: None,
            child_nodes: Vec::new(),
        }
    }

    fn set_loaded_subtree(&mut self, subtree: SubtreeAvailability, max_children: usize) {
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
/// Wraps an in-memory tree of [`OctreeAvailabilityNode`]s and answers
/// per-tile [`TileAvailabilityFlags`] queries.
pub struct OctreeAvailability {
    subtree_levels: u32,
    maximum_level: u32,
    /// `8^subtree_levels` - maximum child subtrees per node.
    maximum_children_subtrees: usize,
    root: Option<Box<OctreeAvailabilityNode>>,
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

                if subtree.is_tile_available(levels_left, rel_morton) {
                    flags |= TileAvailabilityFlags::TILE_AVAILABLE;
                }
                if subtree.is_content_available(levels_left, rel_morton, 0) {
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

    fn add_subtree_inner(
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
    fn resolve_child_subtree(subtree: &SubtreeAvailability, child_morton: u32) -> (bool, usize) {
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

#[cfg(test)]
mod octree_availability_tests {
    use super::*;

    fn oct_subtree_all_tiles(levels: u32) -> SubtreeAvailability {
        SubtreeAvailability::new(
            SubdivisionScheme::Octree,
            levels,
            AvailabilityView::Constant(true),
            AvailabilityView::Constant(false),
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap()
    }

    fn root() -> OctreeTileId {
        OctreeTileId {
            level: 0,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    #[test]
    fn empty_root_implicitly_available() {
        let oa = OctreeAvailability::new(2, 6);
        let flags = oa.compute_availability(root());
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
    }

    #[test]
    fn empty_non_root_returns_empty() {
        let oa = OctreeAvailability::new(2, 6);
        let flags = oa.compute_availability(OctreeTileId {
            level: 1,
            x: 0,
            y: 0,
            z: 0,
        });
        assert!(flags.is_empty());
    }

    #[test]
    fn beyond_max_level_returns_empty() {
        let oa = OctreeAvailability::new(2, 2);
        let flags = oa.compute_availability(OctreeTileId {
            level: 3,
            x: 0,
            y: 0,
            z: 0,
        });
        assert!(flags.is_empty());
    }

    #[test]
    fn root_subtree_tiles_available() {
        let mut oa = OctreeAvailability::new(2, 6);
        assert!(oa.add_subtree(root(), oct_subtree_all_tiles(2)));

        let flags = oa.compute_availability(root());
        assert!(flags.contains(TileAvailabilityFlags::REACHABLE));
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_LOADED));
    }

    #[test]
    fn tiles_within_root_subtree_available() {
        let mut oa = OctreeAvailability::new(2, 6);
        oa.add_subtree(root(), oct_subtree_all_tiles(2));

        // Level 1 is within the 2-level subtree (levels_left == 1 < 2).
        for (x, y, z) in [(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 1)] {
            let tid = OctreeTileId { level: 1, x, y, z };
            let flags = oa.compute_availability(tid);
            assert!(
                flags.contains(TileAvailabilityFlags::TILE_AVAILABLE),
                "{tid:?}"
            );
            assert!(flags.contains(TileAvailabilityFlags::REACHABLE), "{tid:?}");
        }
    }

    #[test]
    fn add_root_twice_fails() {
        let mut oa = OctreeAvailability::new(2, 6);
        assert!(oa.add_subtree(root(), oct_subtree_all_tiles(2)));
        assert!(!oa.add_subtree(root(), oct_subtree_all_tiles(2)));
    }

    #[test]
    fn child_subtree_available_not_loaded() {
        let mut oa = OctreeAvailability::new(2, 6);
        let subtree = SubtreeAvailability::new(
            SubdivisionScheme::Octree,
            2,
            AvailabilityView::Constant(true),
            AvailabilityView::Constant(true), // all 64 child subtrees available
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap();
        oa.add_subtree(root(), subtree);

        // Level 2 is the root of the first child subtree - available, not loaded.
        let flags = oa.compute_availability(OctreeTileId {
            level: 2,
            x: 0,
            y: 0,
            z: 0,
        });
        assert!(flags.contains(TileAvailabilityFlags::TILE_AVAILABLE));
        assert!(flags.contains(TileAvailabilityFlags::SUBTREE_AVAILABLE));
        assert!(!flags.contains(TileAvailabilityFlags::SUBTREE_LOADED));
    }

    #[test]
    fn morton_3d_root_is_zero() {
        assert_eq!(morton_3d(0, 0, 0), 0);
    }

    #[test]
    fn morton_3d_axes_independent() {
        // x=1,y=0,z=0 -> bit0 of each coordinate -> spread3(1)|0|0 = 1
        assert_eq!(morton_3d(1, 0, 0), 1);
        // x=0,y=1,z=0 -> 0|spread3(1)<<1|0 = 2
        assert_eq!(morton_3d(0, 1, 0), 2);
        // x=0,y=0,z=1 -> 0|0|spread3(1)<<2 = 4
        assert_eq!(morton_3d(0, 0, 1), 4);
        // x=1,y=1,z=1 -> 1|2|4 = 7
        assert_eq!(morton_3d(1, 1, 1), 7);
    }

    #[test]
    fn bitstream_child_selects_correct_slot() {
        // 1-level octree subtree: 8 potential child subtrees.
        // Mark only Morton index 0 (x=0,y=0,z=0) as available.
        let root_subtree = SubtreeAvailability::new(
            SubdivisionScheme::Octree,
            1,
            AvailabilityView::Constant(true),
            AvailabilityView::Bitstream(vec![0b0000_0001]), // only bit 0 set
            vec![AvailabilityView::Constant(false)],
        )
        .unwrap();

        let mut oa = OctreeAvailability::new(1, 4);
        oa.add_subtree(root(), root_subtree);

        // Adding at (1,0,0,0) - morton 0 - should succeed.
        assert!(oa.add_subtree(
            OctreeTileId {
                level: 1,
                x: 0,
                y: 0,
                z: 0
            },
            oct_subtree_all_tiles(1),
        ));

        // Adding at (1,1,0,0) - morton 1 - should fail (not in bitstream).
        assert!(!oa.add_subtree(
            OctreeTileId {
                level: 1,
                x: 1,
                y: 0,
                z: 0
            },
            oct_subtree_all_tiles(1),
        ));
    }
}

/// A rectangular range of tile coordinates at a given zoom level.
///
/// Mirrors `CesiumGeometry::QuadtreeTileRectangularRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadtreeTileRectangularRange {
    /// Zoom level (0 = root).
    pub level: u32,
    /// Minimum tile X coordinate (inclusive).
    pub minimum_x: u32,
    /// Minimum tile Y coordinate (inclusive).
    pub minimum_y: u32,
    /// Maximum tile X coordinate (inclusive).
    pub maximum_x: u32,
    /// Maximum tile Y coordinate (inclusive).
    pub maximum_y: u32,
}
