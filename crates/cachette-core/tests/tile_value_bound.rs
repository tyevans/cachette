//! The declared range of a tile value.
//!
//! A tile value is a generated part plus a stored delta, and a pass adds to
//! that delta on every tick.[^1] Without a bound the sum is a walk with no
//! end, so a value on a late tick stands in no range that a reader can name.
//! The observation publishes every value as a share, a signed relation or a
//! compressed magnitude, and a share needs a denominator.[^2]
//!
//! **The fixture writes the extreme rather than the typical case.** The tile
//! value pass of the engine moves a tile by two units of the last place on a
//! tick, and the range is over sixteen million units wide, so no run of any
//! practical length reaches a bound. A fixture that only stepped the world
//! would measure the pass and never the clamp.[^3] These tests therefore
//! write the field through its own public interface with a delta that
//! overshoots, and a separate test steps the world to state that the pass
//! keeps the range.
//!
//! # References
//!
//! [^1]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D1. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
//! [^2]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D2. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::{
    Axial, Fix32, Grid, TileIdx, TileValues, World, WorldConfig, TILE_VALUE_CEILING,
    TILE_VALUE_FLOOR,
};

/// The extent of the fixture world.
const EDGE: u32 = 12;

/// The seed of the fixture world.
const SEED: u64 = 5;

/// The ticks that the long run steps.
const TICKS: u32 = 400;

/// Builds the fixture world.
fn world() -> World {
    World::new(WorldConfig {
        width: EDGE,
        height: EDGE,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 16,
    })
    .expect("the extent must describe a world")
}

/// Builds a value field over the extent, ready to be written.
fn field() -> TileValues {
    let mut values = TileValues::new(
        SEED,
        Grid::new(EDGE, EDGE).expect("the extent describes a grid"),
    );
    values.prepare();
    values
}

/// Adds one delta to every tile of a field, through its worker chunks.
///
/// The write goes through the same path the step uses, so the test drives the
/// real writer and not a mechanism of its own.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
fn add_to_every_tile(values: &mut TileValues, delta: Fix32) -> Vec<Fix32> {
    let mut held = Vec::new();
    let mut changed = 0i64;
    for mut chunk in values.chunks_mut(7) {
        for index in chunk.start()..chunk.end() {
            held.push(
                chunk
                    .add(TileIdx(index), delta)
                    .expect("the tile lies inside the chunk"),
            );
        }
        changed += chunk.changed();
    }
    values.absorb_changed(changed);
    held
}

#[test]
fn the_ceiling_is_the_top_of_the_generated_range() {
    let world = world();
    let mut highest = TILE_VALUE_FLOOR;
    for index in 0..world.grid().tile_count() {
        let generated = TileValues::generated(SEED, TileIdx(index));
        assert!(
            generated.0 >= TILE_VALUE_FLOOR.0 && generated.0 <= TILE_VALUE_CEILING.0,
            "the generated part of tile {index} stands outside the declared range"
        );
        if generated.0 > highest.0 {
            highest = generated;
        }
    }
    assert!(
        highest.0 > TILE_VALUE_CEILING.0 / 2,
        "the fixture must reach the upper half of the generated range"
    );
}

#[test]
fn a_write_that_overshoots_the_ceiling_reads_the_ceiling_back() {
    let mut values = field();
    let held = add_to_every_tile(&mut values, Fix32::MAX);
    assert!(!held.is_empty(), "the fixture must write at least one tile");
    for (index, value) in held.iter().enumerate() {
        assert_eq!(
            *value, TILE_VALUE_CEILING,
            "tile {index} passed the ceiling"
        );
        assert_eq!(
            values.at(TileIdx(index as u32)),
            Some(TILE_VALUE_CEILING),
            "the field reads back a value the write did not return for tile {index}"
        );
    }
    assert!(values.check_invariants());
}

#[test]
fn a_write_that_undershoots_the_floor_reads_the_floor_back() {
    let mut values = field();
    let held = add_to_every_tile(&mut values, Fix32::MIN);
    assert!(!held.is_empty(), "the fixture must write at least one tile");
    for (index, value) in held.iter().enumerate() {
        assert_eq!(
            *value, TILE_VALUE_FLOOR,
            "tile {index} fell below the floor"
        );
        assert_eq!(values.at(TileIdx(index as u32)), Some(TILE_VALUE_FLOOR));
    }
    assert!(values.check_invariants());
}

#[test]
fn a_second_write_at_the_bound_moves_nothing() {
    let mut values = field();
    add_to_every_tile(&mut values, Fix32::MAX);
    let again = add_to_every_tile(&mut values, Fix32::MAX);
    for value in &again {
        assert_eq!(*value, TILE_VALUE_CEILING);
    }
    let zero = add_to_every_tile(&mut values, Fix32::ZERO);
    for value in &zero {
        assert_eq!(*value, TILE_VALUE_CEILING);
    }
    assert!(values.check_invariants());
}

#[test]
fn a_write_inside_the_range_lands_where_it_asked() {
    let mut values = field();
    let step = Fix32(1_024);
    let held = add_to_every_tile(&mut values, step);
    for (index, value) in held.iter().enumerate() {
        let generated = TileValues::generated(SEED, TileIdx(index as u32));
        let wanted = generated.0 + step.0;
        assert!(
            wanted <= TILE_VALUE_CEILING.0,
            "the fixture must stay inside the range for tile {index}"
        );
        assert_eq!(
            value.0, wanted,
            "a write inside the range must not be clamped for tile {index}"
        );
    }
    assert!(values.check_invariants());
}

#[test]
fn the_step_keeps_every_tile_inside_the_range_over_a_long_run() {
    let mut world = world();
    for tick in 0..TICKS {
        world.step(2).expect("the step must run");
        assert!(
            world.check_invariants(),
            "the world broke its invariants on tick {tick}"
        );
    }

    let mut moved = 0usize;
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            let tile = world
                .grid()
                .index_of(here)
                .expect("the address is inside the world");
            let value = world
                .tile_value_at(tile)
                .expect("the address is inside the world");
            assert!(
                value.0 >= TILE_VALUE_FLOOR.0 && value.0 <= TILE_VALUE_CEILING.0,
                "tile ({column}, {row}) stands outside the range at {}",
                value.0
            );
            if value != TileValues::generated(SEED, tile) {
                moved += 1;
            }
        }
    }
    assert!(
        moved > 0,
        "the fixture must move at least one tile away from its generated value"
    );
    assert!(
        world.stored_tile_changes() > 0,
        "the fixture must store a change, or the range check reads generated values only"
    );
}

#[test]
fn a_run_at_three_thread_counts_holds_one_hash() {
    let mut hashes = Vec::new();
    let mut totals = Vec::new();
    for threads in [1usize, 3, 12] {
        let mut world = world();
        for _ in 0..64 {
            world.step(threads).expect("the step must run");
        }
        assert!(world.check_invariants());
        hashes.push(world.state_hash().finish());
        totals.push(world.stored_tile_changes());
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
    assert_eq!(totals[0], totals[1]);
    assert_eq!(totals[1], totals[2]);
    assert!(totals[0] > 0, "the fixture must store a change");
}
