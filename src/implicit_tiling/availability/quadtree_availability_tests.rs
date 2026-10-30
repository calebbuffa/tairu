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
