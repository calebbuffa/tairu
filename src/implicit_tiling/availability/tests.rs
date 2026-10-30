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
