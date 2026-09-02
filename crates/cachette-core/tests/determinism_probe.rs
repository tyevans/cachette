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
//! # References
//!
//! [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`, and ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
#![cfg(feature = "probe-nondeterminism")]

use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::site::CommodityId;
use cachette_core::slots::{Candidate, Slots};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, CarryLoad, FactionId, Fix32, Grid, Terrain, World, WorldConfig};

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
    // A unit takes an intent at the interval its level 1 cell schedules, and
    // it does not move before it has one. This fixture is about the order of
    // a contest, so it sets the interval to every tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
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
