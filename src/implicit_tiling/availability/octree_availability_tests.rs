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
