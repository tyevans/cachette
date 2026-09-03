//! The proof that the determinism tests can fail.
//!
//! A determinism test compares one run against another run. A test that
//! compares a run against itself always passes and proves nothing. This
//! file perturbs the engine behind a test-only feature and asserts that the
//! comparison then reports a difference.
//!
//! Run it with the feature on:
//!
//! ```text
//! cargo test --package cachette-core --features probe-nondeterminism \
//!     --test determinism_probe
//! ```
//!
//! The whole file compiles to nothing when the feature is off.
//!
//! The feature makes the step join its output slots in reverse order. At
//! one thread there is one slot, so the order does not change. At more than
//! one thread the order changes, and the event log changes with it. That is
//! exactly the defect that ADR-0004 D1 forbids.[^1]
//!
//! The feature also makes admission read the intents in the order they
//! arrived rather than in the sorted order. Sorting by a stable key is what
//! makes the admitted set independent of the thread count, so a sound
//! admission absorbs the slot reversal and the thread-count test cannot fail
//! on it. With the sort removed, who enters a full tile follows the join
//! order, and the join order follows the thread count.[^2]
//!
//! The feature also perturbs the influence solve, in two ways, because the
//! two defects the solve must not carry are not visible to one test.
//!
//! The first stops the solve when a pass changed nothing, which is the
//! convergence test that the record forbids.[^3] It is deterministic across
//! thread counts, so the thread-count test passes over it and only a test of
//! the pass count sees it. That is the case the testing rule names: a defect
//! that repeats gives one answer at every thread count.[^4]
//!
//! The second makes a pass read a neighbour outside the run it is filling as
//! nothing, which is a stencil that lost its halo. The run boundary follows
//! the thread count, so the field follows it too.[^1]
//!
//! # References
//!
//! [^3]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
//! [^4]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`, and ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
#![cfg(feature = "probe-nondeterminism")]

use cachette_core::choose::{self, ChoiceSchedule};
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::influence::PASSES_FOR_EACH_SOLVE;
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::site::CommodityId;
use cachette_core::slots::{Candidate, Slots};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, CarryLoad, FactionId, Fix32, Grid, Terrain, World, WorldConfig};
use cachette_core::{CellSummary, Conductance, Influence, InfluenceField, TileIdx};

/// The scenario. It must hold more tiles than threads, so that a run at
/// twelve threads fills more than one output slot.
const CONFIG: WorldConfig = WorldConfig {
    width: 32,
    height: 32,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 4,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Runs one frame and returns the event log as bytes.
fn run(threads: usize) -> Vec<u8> {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world.step(threads).expect("the step must run");
    world.event_log_bytes().to_vec()
}

#[test]
fn the_thread_count_test_fails_when_the_order_rule_breaks() {
    let at_one = run(1);
    let at_twelve = run(12);
    assert!(!at_one.is_empty(), "the scenario must emit events");
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the event log, so the determinism test \
         has no proven failure mode"
    );
}

#[test]
fn the_perturbed_log_holds_the_same_events_in_a_different_order() {
    // The probe changes the order and nothing else. A probe that also
    // changed the content would prove less.
    let mut at_one = run(1);
    let mut at_twelve = run(12);
    assert_eq!(at_one.len(), at_twelve.len());
    at_one.sort_unstable();
    at_twelve.sort_unstable();
    assert_eq!(at_one, at_twelve);
}

/// Reduces the ranks to the lowest one, over the given number of threads.
///
/// Every rank is equal, so only the order decides which position wins. This
/// is the case that the slot rule exists for.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn tied_minimum(threads: usize) -> Option<Candidate<u32>> {
    let mut slots: Slots<Option<Candidate<u32>>> =
        Slots::filled(threads, None).expect("the thread count is not zero");
    for (index, slot) in slots.entries_mut().iter_mut().enumerate() {
        *slot = Some(Candidate::new(0, index as u32));
    }
    slots.minimum()
}

#[test]
fn the_slot_reduction_test_fails_when_the_order_rule_breaks() {
    // The probe reverses the combine order, so the highest slot now wins the
    // tie. The property test asserts the lowest slot wins, so it fails.
    assert_eq!(tied_minimum(1), Some(Candidate::new(0, 0)));
    assert_eq!(tied_minimum(12), Some(Candidate::new(0, 11)));
}

/// The extent that the terrain probe reads.
const TERRAIN_EXTENT: u32 = 192;

#[test]
fn the_key_field_test_fails_when_the_terrain_key_drops_the_row() {
    // The probe drops the row component of the lattice node key. The field
    // then varies along a row and is constant down a column.
    //
    // This defect is invisible to both determinism tests, because the world
    // it builds is identical on every run and at every thread count. Only a
    // test of the key itself sees it, which is the case the testing rule
    // names.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let grid = Grid::new(TERRAIN_EXTENT, TERRAIN_EXTENT).expect("the extent must describe a grid");
    let field = Terrain::new(0x0123_4567_89ab_cdef, grid);
    let column = TERRAIN_EXTENT as i32 / 2;

    let first = field
        .height(Axial::new(column, 0))
        .expect("the address is inside the world");
    for r in 1..TERRAIN_EXTENT as i32 {
        assert_eq!(
            field
                .height(Axial::new(column, r))
                .expect("the address is inside the world"),
            first,
            "the probe did not drop the row, so the key-field test has no \
             proven failure mode"
        );
    }

    // The perturbation is confined to one axis. A probe that changed both
    // would prove less.
    let row = TERRAIN_EXTENT as i32 / 2;
    let mut along: Vec<_> = (0..TERRAIN_EXTENT as i32)
        .map(|q| field.height(Axial::new(q, row)).expect("inside"))
        .collect();
    along.dedup();
    assert!(along.len() > 1, "the probe also removed the column");
}

/// The extent that the resource probe reads.
const RESOURCE_EXTENT: u32 = 192;

#[test]
fn the_stock_key_test_fails_when_the_resource_key_drops_the_row() {
    // The probe drops the row component of the tile address in the stock draw
    // key. The field then varies along a row and is constant down a column.
    //
    // This defect is invisible to both determinism tests, because the world it
    // builds is identical on every run and at every thread count. Only a test
    // of the key itself sees it, which is the case the testing rule names.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let field = World::new(WorldConfig {
        width: RESOURCE_EXTENT,
        height: RESOURCE_EXTENT,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let column = RESOURCE_EXTENT as i32 / 2;

    let first = field
        .original_stock(Axial::new(column, 0), ResourceKind::Food)
        .expect("the address is inside the world");
    for r in 1..RESOURCE_EXTENT as i32 {
        assert_eq!(
            field
                .original_stock(Axial::new(column, r), ResourceKind::Food)
                .expect("the address is inside the world"),
            first,
            "the probe did not drop the row, so the key-field test has no \
             proven failure mode"
        );
    }

    // The perturbation is confined to one axis. A probe that changed both
    // would prove less.
    let row = RESOURCE_EXTENT as i32 / 2;
    let mut along: Vec<_> = (0..RESOURCE_EXTENT as i32)
        .map(|q| {
            field
                .original_stock(Axial::new(q, row), ResourceKind::Food)
                .expect("inside")
        })
        .collect();
    along.dedup();
    assert!(along.len() > 1, "the probe also removed the column");
}

/// Builds a world whose units contend for a deposit, and returns what each of
/// them carries after one frame at the given thread count.
///
/// A crowd spread over many deposits contends for nothing, so the resolve
/// refuses nobody and the order it reads its intents in cannot matter. The
/// probe needs a deposit that runs out, and it must say so rather than assume
/// it.
fn gathered_after_a_frame(threads: usize) -> Vec<CarryLoad> {
    let mut world = World::new(WorldConfig {
        width: RESOURCE_EXTENT,
        height: RESOURCE_EXTENT,
        seed: 0x0cac_4e77_0072,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // The choice pass writes the gather order of a unit whose level 1 cell
    // chooses on that frame, and that write replaces the order a caller
    // gave.[^C] This fixture is about the order of a contest for one deposit,
    // so it puts the choice far enough apart that no cell of a gatherer
    // chooses on the frame under test. It asserts that below rather than
    // assuming it.
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    let schedule =
        ChoiceSchedule::new(choose::PERIOD_LOG2_CEILING).expect("the exponent is inside the range");
    world
        .set_choice_schedule(schedule.period_log2())
        .expect("the exponent is inside the range");

    let grid = world.grid();
    let kind = ResourceKind::Wood;
    let deposits: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| {
            world.admits_a_unit(*address) && world.tile_stock(*address, kind) > Some(Amount::ZERO)
        })
        .take(8)
        .collect();
    assert!(
        deposits.len() == 8,
        "the probe world holds only {} deposits",
        deposits.len()
    );

    let mut units = Vec::new();
    for address in deposits {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        assert!(capacity > 1, "one gatherer contends with nobody");
        for ordinal in 0..capacity {
            let unit = world
                .spawn_soldier(address, FactionId((ordinal % 2) as u16))
                .expect("the open tile admits a unit");
            assert!(world.order_gather(unit, kind));
            let layout = world.pyramid().layout();
            let tile = grid.index_of(address).expect("the address is a tile");
            let cell = layout.block_of_key(layout.key_of(tile).expect("the tile is a tile"));
            assert!(
                !schedule.chooses_now(cell, 1),
                "cell {cell} chooses on the frame under test, so the choice \
                 replaces the order this fixture gave"
            );
            units.push(unit);
        }
    }

    world.step(threads).expect("the step must run");
    assert!(
        world.gather_log().len() < units.len(),
        "every gatherer took a share, so the probe world holds no contest"
    );
    units
        .iter()
        .map(|unit| world.soldier_carry(*unit).expect("nothing despawned it"))
        .collect()
}

#[test]
fn the_gather_test_fails_when_the_sort_rule_breaks() {
    // The probe removes the sort from the gather resolve, so who takes the
    // last of a deposit follows the order the intents were joined in, and the
    // slot probe makes that order follow the thread count.
    //
    // This is the defect ADR-0073 D2 forbids, and it is invisible to a
    // reviewer: the resolve still grants up to the stock, still refuses the
    // rest, and still gives one answer on one machine at one thread count.
    let at_one = gathered_after_a_frame(1);
    let at_twelve = gathered_after_a_frame(12);
    assert!(!at_one.is_empty(), "the scenario must hold gatherers");
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the granted set, so the gather \
         thread-count assertion has no proven failure mode"
    );
    // The admission probe also runs in this build, so the units stand on
    // different tiles at each thread count and the totals taken differ as
    // well as the order. A companion test that held the total fixed would
    // therefore prove nothing here, and this file does not claim one.
}

/// The extent of the crowded world that the admission probe reads.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds open ground as well as water.
const CROWD_EXTENT: u32 = 96;

/// Builds a world whose units contend for their targets, and returns where
/// each of them stands after one frame at the given thread count.
///
/// A population spread over a world contends for nothing, so admission
/// refuses nobody and the order it reads its intents in cannot matter. The
/// probe needs a full tile, and it must say so rather than assume it.
fn crowded_after_a_frame(threads: usize) -> Vec<Axial> {
    let mut world = World::new(WorldConfig {
        width: CROWD_EXTENT,
        height: CROWD_EXTENT,
        seed: 0x0cac_4e77_0023,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // A unit takes an intent at the interval its level 1 cell schedules, and
    // it does not move before it has one. This fixture is about the order of
    // a contest, so it sets the interval to every tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");

    let grid = world.grid();
    let patch: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .filter(|address| address.q >= 8 && address.q < 20 && address.r >= 8 && address.r < 20)
        .collect();
    assert!(
        patch.len() >= 16,
        "the probe world holds only {} open tiles in its patch",
        patch.len()
    );

    let mut kept = Vec::new();
    for address in patch {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        assert!(capacity > 0, "an open tile admits no unit");
        for ordinal in 0..capacity {
            kept.push(
                world
                    .spawn_soldier(address, FactionId((ordinal % 2) as u16))
                    .expect("the open tile admits a unit"),
            );
        }
    }

    world.step(threads).expect("the step must run");
    kept.iter()
        .map(|soldier| {
            world
                .soldiers()
                .address(*soldier)
                .expect("nothing despawned the soldier")
        })
        .collect()
}

#[test]
fn the_admission_test_fails_when_the_sort_rule_breaks() {
    // The probe removes the sort from admission, so who enters a full tile
    // follows the order the intents were joined in, and the slot probe makes
    // that order follow the thread count.
    //
    // This is the defect ADR-0056 D3 forbids, and it is invisible to a
    // reviewer: the code still admits up to the capacity, still refuses the
    // rest, and still gives one answer on one machine at one thread count.[^1]
    //
    // [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    let at_one = crowded_after_a_frame(1);
    let at_twelve = crowded_after_a_frame(12);
    assert!(!at_one.is_empty(), "the scenario must hold soldiers");
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the admitted set, so the admission \
         thread-count assertion has no proven failure mode"
    );
}

#[test]
fn the_perturbed_admission_moves_the_same_number_of_units() {
    // The probe changes who is admitted and not how many. A probe that also
    // changed the count would prove less: the thread-count test would then
    // fail on the population rather than on the order.
    let at_one = crowded_after_a_frame(1);
    let at_twelve = crowded_after_a_frame(12);
    assert_eq!(
        at_one.len(),
        at_twelve.len(),
        "the probe changed the population as well as the order"
    );
}

/// The extent of the world that the founding probe reads.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds open ground as well as water.
const FOUNDING_EXTENT: u32 = 192;

#[test]
fn the_candidate_key_test_fails_when_the_founding_key_drops_the_row() {
    // The probe drops the row draw from the candidate key. Every candidate
    // then sits on one row of the world.
    //
    // This defect is invisible to both determinism tests, because the sample
    // it draws is identical on every run and at every thread count. Only a
    // test of the key itself sees it, which is the case the testing rule
    // names.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let world = World::new(WorldConfig {
        width: FOUNDING_EXTENT,
        height: FOUNDING_EXTENT,
        seed: 0x0cac_4e77_0061,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let survey = world
        .survey_founding(30, FactionId(0))
        .expect("the survey must run");

    let mut rows: Vec<i32> = survey
        .candidates()
        .iter()
        .map(|candidate| candidate.address().r)
        .collect();
    rows.sort_unstable();
    rows.dedup();
    assert_eq!(
        rows.len(),
        1,
        "the probe did not drop the row draw, so the candidate key test has \
         no proven failure mode"
    );

    // The perturbation is confined to one axis. A probe that changed both
    // would prove less.
    let mut columns: Vec<i32> = survey
        .candidates()
        .iter()
        .map(|candidate| candidate.address().q)
        .collect();
    columns.sort_unstable();
    columns.dedup();
    assert!(columns.len() > 1, "the probe also removed the column");
}

/// The extent that the consumption probe stands on.
const CONSUMPTION_EXTENT: u32 = 48;

/// The number of sites that the consumption probe founds.
///
/// The count is above the thread count, so a run at twelve threads fills
/// more than one output slot and the join order can differ.
const CONSUMPTION_SITES: usize = 24;

/// Founds sites that cannot feed their people, and returns the rationed log.
///
/// A site that serves every cohort emits no event, so a world of rich sites
/// would compare two empty logs. The fixture gives every site a store that
/// its people empty, and the caller asserts that the log holds something.
fn rationed_after_frames(threads: usize) -> Vec<u8> {
    let mut world = World::new(WorldConfig {
        width: CONSUMPTION_EXTENT,
        height: CONSUMPTION_EXTENT,
        seed: 42,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_economy_schedule(2, 0)
        .expect("the period is inside the range");
    let grid = world.grid();
    let ground: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(ground.len() > CONSUMPTION_SITES * 3);
    for index in 0..CONSUMPTION_SITES {
        let site = world
            .found_settlement(ground[index * 3], FactionId(0))
            .expect("the tile is free");
        for ordinal in 0..3 {
            let unit = world
                .spawn_soldier(ground[index * 3 + 1], FactionId((ordinal % 2) as u16))
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)));
        }
        world
            .set_settlement_store(site, CommodityId(0), Fix32(1 << 15))
            .expect("the commodity is in the set");
    }

    let mut bytes = Vec::new();
    for _ in 0..8 {
        world.step(threads).expect("the step must run");
        bytes.extend_from_slice(world.rationed_log_bytes());
    }
    bytes
}

#[test]
fn the_consumption_test_fails_when_the_join_order_breaks() {
    // The probe reverses the join of the output slots, so the rationed log
    // of the draw comes back in the reverse of the site order. The draw
    // itself is unchanged: each thread owns a contiguous span of sites and
    // the segments never cross a thread, so only the join moves.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let at_one = rationed_after_frames(1);
    let at_twelve = rationed_after_frames(12);
    assert!(!at_one.is_empty(), "the fixture must ration somebody");
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the rationed log, so the consumption \
         thread-count assertion has no proven failure mode"
    );
}

#[test]
fn the_perturbed_draw_holds_the_same_events_in_a_different_order() {
    // The probe changes the order and nothing else. A probe that also
    // changed what each site received would prove less.
    let mut at_one = rationed_after_frames(1);
    let mut at_twelve = rationed_after_frames(12);
    assert_eq!(at_one.len(), at_twelve.len());
    at_one.sort_unstable();
    at_twelve.sort_unstable();
    assert_eq!(at_one, at_twelve);
}

/// The extent of the world that the choice probe reads.
///
/// The extent is one level 1 cell, and every tile of it admits a unit, so
/// the open share of the cell is exactly one.
const CHOICE_EXTENT: u32 = 32;

#[test]
fn the_tie_break_test_fails_when_the_option_order_breaks() {
    // The probe scans the options from the top of the set, so the strict
    // comparison now gives a tie to the highest option index.
    //
    // This defect is invisible to both determinism tests, because the world
    // it builds is identical on every run and at every thread count. Only a
    // test that constructs a tie and names the winner sees it, which is the
    // case the testing rule names.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let mut world = World::new(WorldConfig {
        width: CHOICE_EXTENT,
        height: CHOICE_EXTENT,
        seed: 7,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    for option in [0u8, 2, 3] {
        world
            .set_option_weight(option, Fix32::MAX)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(1, Fix32::ZERO)
        .expect("the index is inside the set");

    let grid = world.grid();
    let mut units = Vec::new();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        for ordinal in 0..capacity {
            if let Ok(unit) = world.spawn_soldier(address, FactionId((ordinal % 2) as u16)) {
                units.push(unit);
            }
        }
    }
    assert!(units.len() > 1000, "the probe world holds too few units");
    world.step(2).expect("the step must run");
    world.step(2).expect("the step must run");

    let why = world
        .explain_choice(units[0])
        .expect("nothing despawned it");
    assert_eq!(
        why.scores[0], why.scores[3],
        "the probe world holds no tie, so the tie-break test has no proven \
         failure mode"
    );
    assert_eq!(
        why.best, 3,
        "the probe did not reverse the option order, so the tie-break test \
         has no proven failure mode"
    );

    // The perturbation moves the winner and nothing else. A probe that also
    // changed a score would prove less.
    assert!(why.scores[2] < why.scores[0], "the probe changed a score");
}

/// The extent of the world that the exit field probe reads.
///
/// The extent covers several level 1 blocks in each direction, so the lattice
/// of cells holds a cell with six neighbours.
const EXIT_EXTENT: u32 = 256;

/// The option index of the row that scores the units of a cell.
const EXIT_OPTION: u8 = 3;

#[test]
fn the_direction_tie_break_test_fails_when_the_direction_order_breaks() {
    // The probe scans the six directions from the top, so the strict
    // comparison now gives a tie between two equal neighbouring cells to the
    // highest direction index.
    //
    // This defect is invisible to both determinism tests. The field is derived
    // on the calling thread and it is identical on every run, so only a test
    // that builds two equal neighbours and names the winner sees it.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let mut world = World::new(WorldConfig {
        width: EXIT_EXTENT,
        height: EXIT_EXTENT,
        seed: 7,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");

    let cells = world.exit_field().cells();
    let layout = world.pyramid().layout();
    let open_of = |world: &World, cell: u32| -> Vec<Axial> {
        let edge = layout.block_edge();
        let first_column = (cell % layout.blocks_wide()) * edge;
        let first_row = (cell / layout.blocks_wide()) * edge;
        let mut found = Vec::new();
        for row in first_row..first_row + edge {
            for column in first_column..first_column + edge {
                let address = Axial::new(column as i32, row as i32);
                if world.admits_a_unit(address) {
                    found.push(address);
                }
            }
        }
        found
    };

    // A cell whose neighbours in the two directions under test both hold open
    // ground. The scan is ascending, so the answer is fixed.
    let (middle, low, high) = (0..cells.tile_count())
        .find_map(|cell| {
            let here = cells.address_of(TileIdx(cell))?;
            let low = cells.index_of(cells.neighbour(here, 0)?)?.0;
            let high = cells.index_of(cells.neighbour(here, 4)?)?.0;
            let usable = [cell, low, high]
                .iter()
                .all(|each| !open_of(&world, *each).is_empty());
            usable.then_some((cell, low, high))
        })
        .expect("the probe world holds no cell with these two neighbours open");

    for cell in [low, high] {
        for address in open_of(&world, cell) {
            for _ in 0..2 {
                world
                    .spawn_soldier(address, FactionId(0))
                    .expect("the open tile admits the unit");
            }
        }
    }
    world.rebuild_pyramid(1).expect("the rebuild must run");

    // The probe moves the winner and nothing else. The two neighbours read the
    // same value, so only the order decides.
    assert_eq!(
        world
            .pyramid()
            .cell(low)
            .and_then(CellSummary::units_for_each_open_tile),
        world
            .pyramid()
            .cell(high)
            .and_then(CellSummary::units_for_each_open_tile),
        "the probe world holds no tie, so the tie-break test has no proven \
         failure mode"
    );
    assert_eq!(
        world.exit_field().exit(middle, EXIT_OPTION),
        Some(Some(4)),
        "the probe did not reverse the direction order, so the tie-break test \
         has no proven failure mode"
    );
}

/// The extent that the starvation probe stands on.
const STARVE_EXTENT: u32 = 48;

/// The number of sites that the starvation probe founds.
///
/// The count puts marked units in more than one word of the death plane, so
/// a span of the plane belongs to more than one output slot.
const STARVE_SITES: usize = 24;

/// The number of frames that the starvation probe runs.
const STARVE_FRAMES: usize = 48;

/// Runs a world in which half the units starve, and returns the deaths.
///
/// The world founds sites of two kinds. Half produce more than their people
/// eat, and half produce nothing and start with a store that empties. The
/// bound of the rule is low, so the shortage ends a unit inside the run.
fn starved_after_frames(threads: usize) -> Vec<u8> {
    let mut world = World::new(WorldConfig {
        width: STARVE_EXTENT,
        height: STARVE_EXTENT,
        seed: 42,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_economy_schedule(2, 0)
        .expect("the period is inside the range");
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            NEED_FULL,
        )
        .expect("every rate is at or above zero"),
    );
    let grid = world.grid();
    let ground: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(
        ground.len() > STARVE_SITES * 4,
        "the probe world holds only {} open tiles",
        ground.len()
    );
    for index in 0..STARVE_SITES {
        let site = world
            .found_settlement(ground[index * 3], FactionId(0))
            .expect("the tile is free");
        for ordinal in 0..3u16 {
            let unit = world
                .spawn_soldier(ground[index * 3 + 1], FactionId(ordinal % 2))
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)));
        }
        if index % 2 == 0 {
            world
                .set_production_rate(site, CommodityId(0), Fix32::from_int(1))
                .expect("the rate is at or above zero");
        } else {
            world
                .set_settlement_store(site, CommodityId(0), Fix32(NEED_FULL.0 / 2))
                .expect("the commodity is in the set");
        }
    }
    let mut log = Vec::new();
    for _ in 0..STARVE_FRAMES {
        world.step(threads).expect("the step must run");
        log.extend_from_slice(world.starved_log_bytes());
    }
    log
}

#[test]
fn the_starvation_test_fails_when_the_scan_order_breaks() {
    // The probe reads the death plane through the output slots rather than
    // in ascending slot order, and the slot probe reverses the join. The
    // deaths then arrive in an order that follows the thread count.
    let at_one = starved_after_frames(1);
    let at_twelve = starved_after_frames(12);
    assert!(
        !at_one.is_empty(),
        "the probe world ended nobody, so the starvation test has no subject"
    );
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the death order, so the starvation \
         determinism test has no proven failure mode"
    );
}

#[test]
fn the_perturbed_scan_ends_the_same_units_in_a_different_order() {
    // The probe changes the order and nothing else. A probe that also
    // changed who died would prove less.
    let mut at_one = starved_after_frames(1);
    let mut at_twelve = starved_after_frames(12);
    assert_eq!(at_one.len(), at_twelve.len());
    at_one.sort_unstable();
    at_twelve.sort_unstable();
    assert_eq!(at_one, at_twelve);
}

/// The edge of the cell lattice that the influence probe reads.
///
/// The lattice holds more slots than the highest thread count below, so a run
/// at twelve threads fills more than one run and the run boundaries are real.
const INFLUENCE_EDGE: u32 = 24;

/// The cell that the influence probe injects at.
const INFLUENCE_SEAT: Axial = Axial::new(6, 8);

/// The solves that bring the influence probe to rest.
const INFLUENCE_SOLVES: usize = 30;

/// Builds a field with one source, and returns every cell after a run at one
/// thread count.
fn influence_after_a_run(threads: usize) -> Vec<u16> {
    let cells = Grid::new(INFLUENCE_EDGE, INFLUENCE_EDGE).expect("the extent describes a grid");
    let mut field = InfluenceField::new(cells, 2).expect("two factions are inside the ceiling");
    let mut plane = vec![Conductance::FREE; cells.tile_count() as usize];
    for row in 0..16 {
        let index = cells
            .index_of(Axial::new(12, row))
            .expect("the address is inside the lattice");
        plane[index.0 as usize] = Conductance(128);
    }
    field
        .set_conductance(plane)
        .expect("the plane covers the lattice");
    assert!(field.set_source(FactionId(0), INFLUENCE_SEAT, Influence::UNIT));
    for _ in 0..INFLUENCE_SOLVES {
        field.solve(threads).expect("the thread count is not zero");
    }

    let mut out = Vec::new();
    for faction in 0..field.faction_count() {
        for index in 0..cells.tile_count() {
            let address = cells
                .address_of(TileIdx(index))
                .expect("the index is inside the lattice");
            out.push(
                field
                    .at(FactionId(faction), address)
                    .expect("the faction and the cell are inside the field")
                    .0,
            );
        }
    }
    out
}

#[test]
fn the_influence_thread_count_test_fails_when_a_pass_loses_its_halo() {
    // The probe makes a pass read a neighbour outside the run it is filling
    // as nothing. At one thread the run is the whole plane, so the field is
    // unchanged. At twelve threads the run boundaries cut the stencil, and
    // the field follows the thread count.
    //
    // This is the defect ADR-0009 forbids, and it is invisible to a reviewer:
    // every cell still holds a plausible value, the solve still runs its
    // passes, and one machine at one thread count still gives one answer.
    let at_one = influence_after_a_run(1);
    let at_twelve = influence_after_a_run(12);
    assert!(
        at_one.iter().any(|value| *value > 0),
        "the probe fixture produced an empty field"
    );
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the field, so the influence thread-count \
         assertion has no proven failure mode"
    );
}

#[test]
fn the_perturbed_pass_leaves_the_source_cell_alone() {
    // The probe changes what a cell reads and nothing else. A probe that also
    // moved the source would prove less.
    let at_one = influence_after_a_run(1);
    let at_twelve = influence_after_a_run(12);
    assert_eq!(at_one.len(), at_twelve.len());
    let seat = (INFLUENCE_SEAT.r * INFLUENCE_EDGE as i32 + INFLUENCE_SEAT.q) as usize;
    assert_eq!(at_one[seat], Influence::UNIT.0);
    assert_eq!(at_twelve[seat], Influence::UNIT.0);
}

#[test]
fn the_pass_count_test_fails_when_the_solve_stops_on_a_convergence_test() {
    // The probe stops the solve when a pass changed nothing. A field that
    // holds nothing converges on its first pass, so the solve runs one pass
    // instead of the constant.
    //
    // This defect is invisible to the thread-count test, because the
    // comparison it reads is exact and its result does not depend on how the
    // work was split. Only a test of the pass count sees it, which is the
    // case the testing rule names.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let cells = Grid::new(INFLUENCE_EDGE, INFLUENCE_EDGE).expect("the extent describes a grid");
    let mut field = InfluenceField::new(cells, 1).expect("one faction is inside the ceiling");
    field.solve(1).expect("the thread count is not zero");
    assert!(
        field.passes() < u64::from(PASSES_FOR_EACH_SOLVE),
        "the probe did not stop the solve, so the pass-count test has no \
         proven failure mode"
    );

    // The stop reads the field and not the clock, so it fires only when the
    // field is at rest. A field that is still moving runs every pass.
    let mut moving = InfluenceField::new(cells, 1).expect("one faction is inside the ceiling");
    assert!(moving.set_source(FactionId(0), INFLUENCE_SEAT, Influence::UNIT));
    moving.solve(1).expect("the thread count is not zero");
    assert_eq!(moving.passes(), u64::from(PASSES_FOR_EACH_SOLVE));
}

#[test]
fn the_influence_thread_count_test_survives_the_convergence_stop_alone() {
    // The convergence stop on its own does not move the field between thread
    // counts: a field that is at rest is at rest at every thread count, and
    // the stop reads an exact comparison. The assertion below records that
    // the thread-count test is not the test which guards the pass count.[^1]
    //
    // [^1]: Findings register, FND-159. `docs/FINDINGS.md`
    let cells = Grid::new(INFLUENCE_EDGE, INFLUENCE_EDGE).expect("the extent describes a grid");
    let mut at_one = InfluenceField::new(cells, 1).expect("one faction is inside the ceiling");
    let mut at_twelve = InfluenceField::new(cells, 1).expect("one faction is inside the ceiling");
    for _ in 0..INFLUENCE_SOLVES {
        at_one.solve(1).expect("the thread count is not zero");
        at_twelve.solve(12).expect("the thread count is not zero");
    }
    assert_eq!(at_one.passes(), at_twelve.passes());
    for index in 0..cells.tile_count() {
        let address = cells
            .address_of(TileIdx(index))
            .expect("the index is inside the lattice");
        assert_eq!(
            at_one.at(FactionId(0), address),
            at_twelve.at(FactionId(0), address)
        );
    }
}
