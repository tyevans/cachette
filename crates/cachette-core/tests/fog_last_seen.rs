//! The tick on which a faction last saw a cell.
//!
//! The two fog layers say whether a faction sees a cell now and whether it
//! ever saw it. Neither says when. A remembered cell from five hundred ticks
//! ago is not current knowledge, and a reader that holds only the two layers
//! cannot tell the difference.[^1]
//!
//! The engine records one tick for each observed cell of the summary lattice,
//! for each faction. It records no tick for each tile, because a tick for
//! each tile for each faction is a field of the world indexed by the faction,
//! and the record refuses one.[^2]
//!
//! **Every test here asks what the value depends on.** A value that is
//! present proves nothing. A clock keyed on the wrong field gives the same
//! wrong answer on every thread and on every run, and neither determinism
//! test can see it.[^3]
//!
//! # References
//!
//! [^1]: Research report 42, what a policy should be able to see, section 5.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^3]: Testing rules, section 2. `.agents/rules/testing.md`

use cachette_core::{Axial, FactionId, TileKind, World, WorldConfig};

/// The extent of the fixture world.
///
/// The world is wide enough that two units stand far enough apart to share
/// no sight, and that a unit marched away from a cell cannot wander back into
/// it inside the tick count these tests run.
const EDGE: u32 = 40;

/// Builds the fixture world.
fn world() -> World {
    World::new(WorldConfig {
        width: EDGE,
        height: EDGE,
        seed: 11,
        faction_count: 3,
        unit_capacity: 64,
    })
    .expect("the extent must describe a world")
}

/// Returns the passable addresses of a world, in tile index order.
///
/// The generated ground holds water, and no unit stands on water.
fn dry_ground(world: &World) -> Vec<Axial> {
    let mut dry = Vec::new();
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            if world.tile_kind(here).is_some_and(TileKind::is_passable) {
                dry.push(here);
            }
        }
    }
    dry
}

/// Returns two passable addresses that share no sight and lie in two cells.
///
/// **The fixture must reach the case of two cells at two ages.** A pair in one
/// cell would pass a clock that recorded one tick for the whole world, and a
/// pair inside one sight would pass a clock that never aged at all.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn two_far_cells(world: &World) -> (Axial, Axial) {
    let dry = dry_ground(world);
    let near = dry[0];
    let far = *dry.last().expect("the fixture world holds passable ground");
    assert!(
        near.distance(far) > u32::from(world.sight_rules().ceiling()) * 2,
        "the two addresses must share no sight at any admitted radius"
    );
    assert_ne!(
        world.cell_covering(near),
        world.cell_covering(far),
        "the two addresses must lie in two cells of the lattice"
    );
    (near, far)
}

/// Returns the cell that one faction's only unit stands in.
fn cell_of(world: &World, unit: cachette_core::Entity) -> u32 {
    let place = world.soldiers().address(unit).expect("the unit is alive");
    world
        .cell_covering(place)
        .expect("the unit stands inside the world")
}

#[test]
fn a_cell_a_faction_has_never_seen_reports_no_age() {
    let mut world = world();
    let (near, far) = two_far_cells(&world);
    world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let unseen = world
        .cell_covering(far)
        .expect("the address is inside the world");
    assert_eq!(world.faction_cell_age(FactionId(0), unseen), None);
    assert_eq!(world.faction_cell_last_seen(FactionId(0), unseen), None);
    assert_eq!(world.faction_tile_age(FactionId(0), far), None);
}

#[test]
fn a_cell_the_faction_sees_now_reports_no_ticks_since() {
    let mut world = world();
    let (near, _) = two_far_cells(&world);
    let unit = world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let here = cell_of(&world, unit);
    assert_eq!(world.faction_cell_age(FactionId(0), here), Some(0));
    assert_eq!(
        world
            .faction_cell_last_seen(FactionId(0), here)
            .map(|t| t.0),
        Some(world.tick().0)
    );
}

#[test]
fn the_age_of_a_remembered_cell_grows_with_the_tick() {
    let mut world = world();
    let (near, far) = two_far_cells(&world);
    let unit = world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let left = cell_of(&world, unit);
    assert_eq!(world.faction_cell_age(FactionId(0), left), Some(0));
    let stamped = world
        .faction_cell_last_seen(FactionId(0), left)
        .expect("the faction saw the cell");

    world
        .place_soldier(unit, far)
        .expect("the address is inside the world");
    for expected in 1..=6u64 {
        world.step(1).expect("the step must run");
        assert_eq!(
            world.faction_cell_age(FactionId(0), left),
            Some(expected),
            "the age of a cell the faction left must grow by one on each tick"
        );
        assert_eq!(
            world
                .faction_cell_last_seen(FactionId(0), left)
                .map(|tick| tick.0),
            Some(stamped.0),
            "the tick of a cell the faction does not see must not move"
        );
    }
}

#[test]
fn the_tick_moves_again_when_the_faction_returns_to_the_cell() {
    let mut world = world();
    let (near, far) = two_far_cells(&world);
    let unit = world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let left = cell_of(&world, unit);
    let first = world
        .faction_cell_last_seen(FactionId(0), left)
        .expect("the faction saw the cell");

    world
        .place_soldier(unit, far)
        .expect("the address is inside the world");
    world.step(4).expect("the steps must run");
    assert!(world.faction_cell_age(FactionId(0), left).unwrap_or(0) > 0);

    world
        .place_soldier(unit, near)
        .expect("the address is inside the world");
    world.step(1).expect("the step must run");
    let second = world
        .faction_cell_last_seen(FactionId(0), left)
        .expect("the faction saw the cell again");
    assert!(
        second.0 > first.0,
        "a second sighting must move the tick: {} against {}",
        second.0,
        first.0
    );
    assert_eq!(world.faction_cell_age(FactionId(0), left), Some(0));
}

#[test]
fn the_tick_is_keyed_on_the_faction() {
    let mut world = world();
    let (near, far) = two_far_cells(&world);
    let first = world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let cell = cell_of(&world, first);
    world
        .place_soldier(first, far)
        .expect("the address is inside the world");
    world.step(5).expect("the steps must run");

    // The second faction reaches the cell five ticks after the first left it.
    let second = world
        .spawn_soldier(near, FactionId(1))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    assert_eq!(cell_of(&world, second), cell);

    let mine = world
        .faction_cell_last_seen(FactionId(0), cell)
        .expect("the first faction saw the cell");
    let theirs = world
        .faction_cell_last_seen(FactionId(1), cell)
        .expect("the second faction saw the cell");
    assert!(
        theirs.0 > mine.0,
        "a clock that dropped the faction from its key would give one tick to both: {} against {}",
        theirs.0,
        mine.0
    );
    assert_eq!(world.faction_cell_age(FactionId(1), cell), Some(0));
    assert!(world.faction_cell_age(FactionId(0), cell).unwrap_or(0) > 0);
    assert_eq!(world.faction_cell_age(FactionId(2), cell), None);
}

#[test]
fn the_tick_is_keyed_on_the_cell() {
    let mut world = world();
    let (near, far) = two_far_cells(&world);
    let unit = world
        .spawn_soldier(near, FactionId(0))
        .expect("the spawn must succeed");
    world.step(1).expect("the step must run");
    let first = cell_of(&world, unit);

    world
        .place_soldier(unit, far)
        .expect("the address is inside the world");
    world.step(5).expect("the steps must run");
    let second = cell_of(&world, unit);
    assert_ne!(first, second);

    let older = world
        .faction_cell_last_seen(FactionId(0), first)
        .expect("the faction saw the first cell");
    let newer = world
        .faction_cell_last_seen(FactionId(0), second)
        .expect("the faction saw the second cell");
    assert!(
        newer.0 > older.0,
        "a clock that dropped the cell from its key would give one tick to both: {} against {}",
        newer.0,
        older.0
    );
}

#[test]
fn two_thread_counts_record_one_clock() {
    let mut ages = Vec::new();
    let mut hashes = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = world();
        let (near, far) = two_far_cells(&world);
        let unit = world
            .spawn_soldier(near, FactionId(0))
            .expect("the spawn must succeed");
        let rival = world
            .spawn_soldier(far, FactionId(1))
            .expect("the spawn must succeed");
        world.step(threads).expect("the step must run");
        world
            .place_soldier(unit, far)
            .expect("the address is inside the world");
        world
            .place_soldier(rival, near)
            .expect("the address is inside the world");
        world.step(threads).expect("the step must run");
        world.step(threads).expect("the step must run");

        let mut run = Vec::new();
        for cell in 0..world.pyramid().len() as u32 {
            for number in 0..3u16 {
                run.push((
                    cell,
                    number,
                    world
                        .faction_cell_last_seen(FactionId(number), cell)
                        .map(|t| t.0),
                    world.faction_cell_age(FactionId(number), cell),
                ));
            }
        }
        ages.push(run);
        hashes.push(world.state_hash().finish());
    }
    assert_eq!(ages[0], ages[1], "one tick at one thread count and at two");
    assert_eq!(
        ages[1], ages[2],
        "one tick at two thread counts and at twelve"
    );
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
    assert!(
        ages[0].iter().any(|row| row.3 == Some(0)),
        "the fixture must reach a cell the faction sees now"
    );
    assert!(
        ages[0]
            .iter()
            .any(|row| matches!(row.3, Some(age) if age > 0)),
        "the fixture must reach a cell the faction only remembers"
    );
    assert!(
        ages[0].iter().any(|row| row.3.is_none()),
        "the fixture must reach a cell the faction has never seen"
    );
}
