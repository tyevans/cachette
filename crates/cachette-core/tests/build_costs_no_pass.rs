//! What building a world stores, and what a frame adds to it.
//!
//! The product record for the ground states a cost shape: building a world
//! must not cost a pass over every tile before the first frame, so a
//! developer who changes a seed sees the new world at once.[^1] The record
//! reached `Accepted` while the engine did the opposite, because a statement
//! that nobody runs is not checked.[^2]
//!
//! This file checks what the build stores. A second file checks that the
//! build visits no tile, and it needs a test-only switch to count the
//! visits, so it lives on its own.[^3]
//!
//! **A timing assertion is forbidden here.** Wall clock on a loaded machine
//! is not evidence, and a test that asserted a build was fast would teach
//! everybody to ignore a red pipeline.[^4] Every assertion below is
//! structural.
//!
//! Neither file checks the whole build. Two other parts of the build still
//! visit every tile, and an item tracks them.[^5]
//!
//! # References
//!
//! [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^2]: Findings register, FND-086. `docs/FINDINGS.md`
//! [^3]: The build visit test. `crates/cachette-core/tests/build_visits_no_tile.rs`
//! [^4]: Testing rules, section 3. `.claude/rules/testing.md`
//! [^5]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`

use cachette_core::{Axial, TileIdx, World, WorldConfig};

/// Builds a square world of the given edge.
fn world_of(edge: u32) -> World {
    World::new(WorldConfig {
        width: edge,
        height: edge,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world")
}

#[test]
fn a_built_world_stores_no_tile_change() {
    // Nothing has changed a tile, so the field holds no entry. The count is
    // the same at every extent, which is the shape the record asks of the
    // build: it does not grow with the size of the world.
    for edge in [16u32, 64, 256] {
        assert_eq!(world_of(edge).stored_tile_changes(), 0);
    }
}

#[test]
fn a_frame_stores_only_the_tiles_it_changed() {
    let mut world = world_of(64);
    world.step(1).expect("the step must run");

    // Every event names a tile the frame changed, and one frame names a tile
    // once, so the stored count is the event count of the first frame.
    let changed = world.event_log().len();
    assert!(changed > 0, "the frame must change at least one tile");
    assert!(
        changed < world.tile_count(),
        "the frame must not change every tile, or the test proves nothing"
    );
    assert_eq!(world.stored_tile_changes(), changed);
}

#[test]
fn a_second_frame_adds_to_the_change_of_a_tile_it_changes_again() {
    let mut world = world_of(64);
    world.step(1).expect("the step must run");
    let after_one = world.stored_tile_changes();
    world.step(1).expect("the step must run");
    let after_two = world.stored_tile_changes();

    // The second frame changes some tiles the first frame changed. The
    // stored count therefore grows by fewer entries than the second frame
    // emitted, because a tile already stored takes no second entry.
    let second_frame = world.event_log().len();
    assert!(after_two > after_one, "the second frame must store more");
    assert!(
        after_two - after_one < second_frame,
        "a tile that both frames changed must hold one entry, not two"
    );
}

#[test]
fn a_tile_reads_one_value_whether_or_not_a_frame_touched_it() {
    let mut world = world_of(32);
    let before = world.copy_tile_values();
    world.step(3).expect("the step must run");
    let after = world.copy_tile_values();

    // The single-tile read and the whole-column copy answer with one value,
    // whether the tile holds a stored change or not.
    for (index, value) in after.iter().enumerate() {
        assert_eq!(world.tile_value_at(TileIdx(index as u32)), Some(*value));
    }

    // A tile that no event names must read what it read before the frame.
    let touched: Vec<u32> = world.event_log().iter().map(|event| event.tile.0).collect();
    let untouched = (0..world.tile_count() as u32)
        .find(|index| !touched.contains(index))
        .expect("some tile must be untouched");
    assert_eq!(
        before[untouched as usize], after[untouched as usize],
        "a tile no event names must not move"
    );
}

#[test]
fn a_read_outside_the_extent_names_no_tile() {
    let world = World::new(WorldConfig {
        width: 8,
        height: 4,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    assert!(world.tile_value_at(TileIdx(31)).is_some());
    assert!(world.tile_value_at(TileIdx(32)).is_none());
    assert!(world.tile_value(Axial::new(8, 0)).is_none());
}

#[test]
fn two_seeds_give_two_fields_and_one_seed_gives_one() {
    let base = WorldConfig {
        width: 32,
        height: 32,
        ..WorldConfig::default()
    };
    let first = World::new(base).expect("the extent must describe a world");
    let again = World::new(base).expect("the extent must describe a world");
    let other = World::new(WorldConfig {
        seed: base.seed ^ 0x5555,
        ..base
    })
    .expect("the extent must describe a world");

    assert_eq!(first.copy_tile_values(), again.copy_tile_values());
    assert_ne!(first.copy_tile_values(), other.copy_tile_values());
}

#[test]
fn the_tile_index_reaches_the_key_of_the_generated_value() {
    // The generated part is a keyed draw, and the tile index is one field of
    // the key. A key that dropped the index would give one value to every
    // tile, and both determinism tests would pass over that world, because
    // the defect repeats exactly.
    let world = world_of(32);
    let values = world.copy_tile_values();
    let first = values[0];
    assert!(
        values.iter().any(|value| *value != first),
        "two tiles must not hold one value"
    );
}
