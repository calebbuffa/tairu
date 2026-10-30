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

use crate::generated::SubdivisionScheme;

/// A tile identifier that can be addressed within a subtree.
pub trait SubtreeTileId: Copy {
    /// The subdivision scheme represented by this identifier.
    const SCHEME: SubdivisionScheme;

    /// Return the relative level and Morton index from `subtree_id`.
    fn relative_position(self, subtree_id: Self) -> (u32, u64);
}

/// Quadtree tile coordinates - local to avoid external math-library deps.
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
        if subtree_levels == 0 {
            return self;
        }
        let subtree_level = (self.level / subtree_levels) * subtree_levels;
        let levels_down = self.level - subtree_level;
        QuadtreeTileId::new(subtree_level, self.x >> levels_down, self.y >> levels_down)
    }
}

impl SubtreeTileId for QuadtreeTileId {
    const SCHEME: SubdivisionScheme = SubdivisionScheme::Quadtree;

    fn relative_position(self, subtree_id: Self) -> (u32, u64) {
        let relative = self.relative_to(subtree_id);
        (relative.level, relative.morton_index())
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

fn bit_count(scheme: SubdivisionScheme, level: u32) -> Option<u64> {
    scheme
        .power()
        .checked_mul(level)
        .and_then(|shift| (shift < u64::BITS).then_some(1u64 << shift))
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
    pub(crate) fn from_parts(
        scheme: SubdivisionScheme,
        levels: u32,
        tile_availability: AvailabilityView,
        child_subtree_availability: AvailabilityView,
        content_availability: Vec<AvailabilityView>,
    ) -> Self {
        Self {
            scheme,
            levels,
            tile_availability,
            child_subtree_availability,
            content_availability,
        }
    }

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
        Some(Self::from_parts(
            scheme,
            levels,
            tile_availability,
            child_subtree_availability,
            content_availability,
        ))
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

    /// Tile availability for a typed tile ID.
    pub fn is_tile_available<T: SubtreeTileId>(&self, subtree_id: T, tile_id: T) -> bool {
        debug_assert!(
            self.scheme == T::SCHEME
                || (self.scheme == SubdivisionScheme::S2
                    && T::SCHEME == SubdivisionScheme::Quadtree)
        );
        let (level, morton) = tile_id.relative_position(subtree_id);
        self.is_tile_available_at(level, morton)
    }

    /// Tile availability at a relative level and Morton index within this subtree.
    pub fn is_tile_available_at(&self, relative_level: u32, morton_id: u64) -> bool {
        self.is_available(relative_level, morton_id, &self.tile_availability)
    }

    /// Mark a typed tile available/unavailable.
    pub fn set_tile_available<T: SubtreeTileId>(
        &mut self,
        subtree_id: T,
        tile_id: T,
        available: bool,
    ) {
        debug_assert!(
            self.scheme == T::SCHEME
                || (self.scheme == SubdivisionScheme::S2
                    && T::SCHEME == SubdivisionScheme::Quadtree)
        );
        let (level, morton) = tile_id.relative_position(subtree_id);
        self.set_available(level, morton, available, false);
    }

    /// Mark a tile available/unavailable at a relative level and Morton index.
    pub fn set_tile_available_at(&mut self, relative_level: u32, morton_id: u64, available: bool) {
        self.set_available(relative_level, morton_id, available, false);
    }

    /// Content availability for a typed tile ID.
    pub fn is_content_available<T: SubtreeTileId>(
        &self,
        subtree_id: T,
        tile_id: T,
        content_id: usize,
    ) -> bool {
        debug_assert!(
            self.scheme == T::SCHEME
                || (self.scheme == SubdivisionScheme::S2
                    && T::SCHEME == SubdivisionScheme::Quadtree)
        );
        let (level, morton) = tile_id.relative_position(subtree_id);
        self.is_content_available_at(level, morton, content_id)
    }

    /// Content availability at a relative level, Morton index, and content index.
    pub fn is_content_available_at(
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

    /// Number of content availability layers in this subtree.
    ///
    /// Zero means `contentAvailability` was omitted; omitted availability
    /// makes every content unavailable.
    pub fn content_availability_count(&self) -> usize {
        self.content_availability.len()
    }

    /// Check whether the child subtree identified by its Morton index
    /// (relative to this subtree root) is available.
    pub fn is_child_subtree_available_at(&self, relative_morton_id: u64) -> bool {
        self.child_subtree_availability.is_set(relative_morton_id)
    }

    /// Child-subtree availability by typed tile IDs.
    pub fn is_child_subtree_available<T: SubtreeTileId>(
        &self,
        this_subtree_id: T,
        child_subtree_id: T,
    ) -> bool {
        debug_assert!(
            self.scheme == T::SCHEME
                || (self.scheme == SubdivisionScheme::S2
                    && T::SCHEME == SubdivisionScheme::Quadtree)
        );
        let (_, morton) = child_subtree_id.relative_position(this_subtree_id);
        self.is_child_subtree_available_at(morton)
    }

    /// Mark the child subtree at a relative Morton index available/unavailable.
    pub fn set_child_subtree_available(&mut self, relative_morton_id: u64, available: bool) {
        // For child-subtree availability: the layout is a flat bitstream
        // (conceptually level 0 of the *next* subtree level), so prefix = 0.
        let Some(total_child_subtrees) = bit_count(self.scheme, self.levels) else {
            return;
        };
        self.child_subtree_availability
            .expand_constant(total_child_subtrees);
        self.child_subtree_availability
            .set_bit(relative_morton_id, available);
    }

    /// Direct access to the raw child-subtree availability view.
    ///
    /// Used by [`QuadtreeAvailability`] to
    /// determine the child node count and compute the compressed child index.
    pub fn child_subtree_view(&self) -> &AvailabilityView {
        &self.child_subtree_availability
    }

    /// Returns `true` if any tile within `range` is available in this subtree.
    pub fn any_tile_available_in_range(&self, range: &QuadtreeTileRectangularRange) -> bool {
        for y in range.minimum_y..=range.maximum_y {
            for x in range.minimum_x..=range.maximum_x {
                let morton = morton_2d(x, y) as u64;
                if self.is_tile_available_at(range.level, morton) {
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
    fn prefix_for_level(&self, level: u32) -> Option<u64> {
        let child_count = self.scheme.child_count();
        let tiles_at_level = bit_count(self.scheme, level)?;
        // sum of geometric series: (r^n - 1) / (r - 1)
        Some((tiles_at_level - 1) / (child_count - 1))
    }

    fn is_available(&self, relative_level: u32, morton_id: u64, view: &AvailabilityView) -> bool {
        let Some(tiles_at_level) = bit_count(self.scheme, relative_level) else {
            return false;
        };
        if morton_id >= tiles_at_level {
            return false;
        }
        let Some(prefix) = self.prefix_for_level(relative_level) else {
            return false;
        };
        let Some(bit_index) = prefix.checked_add(morton_id) else {
            return false;
        };
        view.is_set(bit_index)
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
        let Some(total_tiles) = self.prefix_for_level(self.levels) else {
            return;
        };
        let Some(prefix) = self.prefix_for_level(relative_level) else {
            return;
        };
        let Some(bit_index) = prefix.checked_add(morton_id) else {
            return;
        };

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

/// A rectangular range of tile coordinates at a given zoom level.
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

mod octree;
pub use octree::*;

mod quadtree;
pub use quadtree::*;
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
    fn zero_subtree_levels_do_not_divide_or_shift_panic() {
        let id = QuadtreeTileId::new(4, 3, 2);
        assert_eq!(id.subtree_root(0), id);
        let subtree = SubtreeAvailability::all_unavailable(SubdivisionScheme::Quadtree, u32::MAX);
        assert!(!subtree.is_tile_available_at(0, 0));
        assert!(!subtree.is_child_subtree_available_at(0));
    }

    #[test]
    fn constant_all_available() {
        let sa = quad_subtree(true);
        // Root
        assert!(sa.is_tile_available_at(0, 0));
        // All 4 level-1 tiles
        for m in 0..4 {
            assert!(sa.is_tile_available_at(1, m));
        }
    }

    #[test]
    fn constant_all_unavailable() {
        let sa = quad_subtree(false);
        assert!(!sa.is_tile_available_at(0, 0));
        for m in 0..4 {
            assert!(!sa.is_tile_available_at(1, m));
        }
    }

    #[test]
    fn set_tile_available_expands_constant() {
        let mut sa = quad_subtree(false);
        // Mark only the root available.
        sa.set_tile_available_at(0, 0, true);
        assert!(sa.is_tile_available_at(0, 0));
        assert!(!sa.is_tile_available_at(1, 0));
    }

    #[test]
    fn set_and_query_individual_tiles() {
        let mut sa = quad_subtree(false);
        sa.set_tile_available_at(0, 0, true);
        sa.set_tile_available_at(1, 2, true); // bottom-left level-1 child
        assert!(sa.is_tile_available_at(0, 0));
        assert!(!sa.is_tile_available_at(1, 0));
        assert!(!sa.is_tile_available_at(1, 1));
        assert!(sa.is_tile_available_at(1, 2));
        assert!(!sa.is_tile_available_at(1, 3));
    }

    #[test]
    fn child_subtree_initially_unavailable() {
        let sa = quad_subtree(true);
        // child_subtree_availability defaults to Constant(false)
        assert!(!sa.is_child_subtree_available_at(0));
    }

    #[test]
    fn set_child_subtree_available() {
        let mut sa = quad_subtree(true);
        sa.set_child_subtree_available(3, true);
        assert!(!sa.is_child_subtree_available_at(0));
        assert!(sa.is_child_subtree_available_at(3));
    }

    #[test]
    fn is_tile_available_via_quad_ids() {
        let mut sa = SubtreeAvailability::all_unavailable(SubdivisionScheme::Quadtree, 2);
        let root = QuadtreeTileId::new(0, 0, 0);
        let child = QuadtreeTileId::new(1, 1, 0); // Morton index = 1
        sa.set_tile_available(root, child, true);
        assert!(sa.is_tile_available(root, child));
        assert!(!sa.is_tile_available(root, QuadtreeTileId::new(1, 0, 0)));
    }

    #[test]
    fn octree_all_available() {
        let sa = SubtreeAvailability::all_available(SubdivisionScheme::Octree, 1);
        assert!(sa.is_tile_available_at(0, 0));
        for m in 0..8 {
            assert!(sa.is_tile_available_at(1, m));
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

    mod quadtree {
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

    mod octree {
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
}
