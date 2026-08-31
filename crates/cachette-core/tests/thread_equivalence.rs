//! Thread-count equivalence.
//!
//! The test runs the same tick at one thread, at two threads and at twelve
//! threads. It compares the event log byte for byte. The record calls this
//! the highest-value test in the project.[^1]
//!
//! The test sees only the public crate API. It does not reach into an
//! internal module.[^2]
//!
//! To add a scenario, add a row to `SCENARIOS`. The test then runs the new
//! row at every thread count.
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: Testing policy. `docs/TESTING.md`

use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::site::CommodityId;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The thread counts that every scenario runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// A named scenario. Add a row to cover a new case.
const SCENARIOS: &[(&str, WorldConfig, u64)] = &[
    (
        "one tile",
        WorldConfig {
            width: 1,
            height: 1,
            seed: 1,
            faction_count: 1,
        },
        1,
    ),
    (
        "fewer tiles than threads",
        WorldConfig {
            width: 7,
            height: 1,
            seed: 0xdead_beef,
            faction_count: 2,
        },
        4,
    ),
    (
        "an uneven split",
        WorldConfig {
            width: 17,
            height: 59,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        },
        8,
    ),
    (
        "many tiles",
        WorldConfig {
            width: 256,
            height: 256,
            seed: 42,
            faction_count: 16,
        },
        4,
    ),
];

/// Runs the frames and returns the log of the last frame and the state hash.
fn run(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64) {
    let mut world = World::new(config).expect("the extent must describe a world");
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
    )
}

#[test]
fn the_event_log_is_identical_at_every_thread_count() {
    for (name, config, frames) in SCENARIOS {
        let (expected_log, expected_hash) = run(*config, *frames, THREAD_COUNTS[0]);
        for threads in &THREAD_COUNTS[1..] {
            let (log, hash) = run(*config, *frames, *threads);
            assert_eq!(
                log, expected_log,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                hash, expected_hash,
                "scenario {name}: the state hash differs at {threads} threads"
            );
        }
    }
}

#[test]
fn the_log_is_not_empty_for_a_large_scenario() {
    // A test that compares two empty logs proves nothing. This test fails
    // if the stub stops emitting events.
    let (log, _) = run(SCENARIOS[3].1, 1, 4);
    assert!(!log.is_empty(), "the scenario must emit events");
}

#[test]
fn a_step_at_zero_threads_returns_an_error() {
    let mut world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    assert!(world.step(0).is_err());
}

/// Fills a world with soldiers and returns the identities it kept.
///
/// The population is a fixed pattern, so it is the same on every run and at
/// every thread count. The function despawns part of what it spawns, so the
/// arena carries a free queue and a set of stale identities as well as a
/// live set.
fn populate(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let mut kept = Vec::new();
    let mut freed = Vec::new();
    // The pattern walks the tiles in index order and passes over the ground
    // that admits no unit. Water refuses a spawn, and which tiles hold water
    // is a property of the seed.[^1]
    //
    // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .take(97)
        .collect();
    for (index, address) in open.into_iter().enumerate() {
        let index = index as u32;
        // The faction must be one the world has. This line read `index % 5`
        // against a scenario of two factions, so the suite spawned soldiers
        // of factions the world did not hold and the invariant check passed.
        let ceiling = u32::from(world.config().faction_count.max(1));
        let faction = FactionId((index % ceiling) as u16);
        let soldier = world
            .spawn_soldier(address, faction)
            .expect("the address and the faction must be valid");
        if index % 3 == 2 {
            freed.push(soldier);
        } else {
            kept.push(soldier);
        }
    }
    for soldier in &freed {
        assert!(world.despawn_soldier(*soldier));
    }
    kept
}

/// Fills a patch of open ground to the capacity of each of its tiles.
///
/// A population spread over a world contends for nothing, so admission
/// refuses nobody and the equivalence proves only that the intents agree. A
/// tile that is oversubscribed is where determinism is easiest to lose: the
/// answer depends on the order in which admission sees the intents, and that
/// order must come from the sort and never from the thread that finished
/// first.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn crowd(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let patch: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .take(24)
        .collect();
    // A world of one tile, or a world of water, holds no crowd. The caller
    // passes over such a scenario rather than failing on it, and the test
    // asserts that some scenario did hold one.
    let mut kept = Vec::new();
    let ceiling = u32::from(world.config().faction_count.max(1));
    for address in patch {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        for ordinal in 0..capacity {
            kept.push(
                world
                    .spawn_soldier(address, FactionId((ordinal % ceiling) as u16))
                    .expect("the open tile admits a unit"),
            );
        }
    }
    kept
}

/// Runs the frames over a world whose units contend for their targets.
fn run_with_a_crowd(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let kept = crowd(&mut world);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    assert!(world.check_invariants());
    assert!(kept
        .iter()
        .all(|soldier| world.soldiers().contains(*soldier)));
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
        world.soldiers().len() as usize,
    )
}

#[test]
fn a_world_whose_units_contend_is_identical_at_every_thread_count() {
    // The item that added admission asks for this: the thread-count test must
    // cover a tile that is oversubscribed, because that is where determinism
    // is easiest to lose and hardest to see.
    let mut crowded = 0;
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_a_crowd(*config, *frames, THREAD_COUNTS[0]);
        if expected.2 == 0 {
            // The scenario has no open ground to crowd. A world of one water
            // tile is such a scenario, and it tests nothing here.
            continue;
        }
        crowded += 1;
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_a_crowd(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the live soldier count differs at {threads} threads"
            );
        }
    }
    assert!(
        crowded > 1,
        "only {crowded} scenario held a crowd, so the oversubscribed case is barely covered"
    );
}

/// The kind that the gathering scenario takes.
const GATHERED: ResourceKind = ResourceKind::Wood;

/// Fills the deposits of a patch with gatherers, and returns them.
///
/// The units stand on tiles that carry a stock, and every unit is told to
/// gather. The deposits are small against what the crowd demands, so the
/// resolve refuses somebody on every contested tile. That is where
/// determinism is easiest to lose: who takes the last of a deposit must come
/// from the sort and never from the thread that finished first.[^1] [^2]
///
/// The caller asserts that the fixture produced the contested case. A fixture
/// that only assumed it would measure itself.[^3]
///
/// # References
///
/// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2, a draft record. `docs/adrs/draft/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^3]: Findings register, FND-051. `docs/FINDINGS.md`
fn gatherers(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let deposits: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| {
            world.admits_a_unit(*address)
                && world.tile_stock(*address, GATHERED) > Some(Amount::ZERO)
        })
        .take(8)
        .collect();

    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut units = Vec::new();
    for address in deposits {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        for ordinal in 0..capacity {
            let unit = world
                .spawn_soldier(address, FactionId((ordinal % ceiling) as u16))
                .expect("the open tile admits a unit");
            assert!(world.order_gather(unit, GATHERED));
            units.push(unit);
        }
    }
    units
}

/// Runs the frames over a world whose units contend for a deposit.
fn run_with_gatherers(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let units = gatherers(&mut world);
    if units.is_empty() {
        return (Vec::new(), 0, 0);
    }
    let mut taken = 0usize;
    for frame in 0..frames {
        world.step(threads).expect("the step must run");
        taken += world.gather_log().len();
        if frame == 0 {
            // The fixture must produce the contested case. A frame in which
            // every gatherer took a share proves only that the resolve grants
            // something. The refused gatherer is what the sort decides.
            assert!(
                world.gather_log().len() < units.len(),
                "every one of {} gatherers took a share, so nobody contended",
                units.len()
            );
        }
    }
    assert!(world.check_invariants());
    let mut bytes = world.event_log_bytes().to_vec();
    bytes.extend_from_slice(world.gather_log_bytes());
    (bytes, world.state_hash().finish(), taken)
}

#[test]
fn a_world_whose_units_gather_is_identical_at_every_thread_count() {
    let mut contended = 0;
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_gatherers(*config, *frames, THREAD_COUNTS[0]);
        if expected.2 == 0 {
            // The scenario holds no deposit on open ground. A world of one
            // water tile is such a scenario, and it tests nothing here.
            continue;
        }
        contended += 1;
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_gatherers(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the grant count differs at {threads} threads"
            );
        }
    }
    assert!(
        contended > 0,
        "no scenario held a contested deposit, so the case is not covered"
    );
}

/// Runs the frames over a world that holds soldiers.
fn run_with_soldiers(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let kept = populate(&mut world);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    // Drive the step, then inspect the arena. A column set that no test
    // reaches through the engine is inert.[^1]
    //
    // [^1]: Findings register, FND-041. `docs/FINDINGS.md`
    assert!(world.check_invariants());
    assert!(kept
        .iter()
        .all(|soldier| world.soldiers().contains(*soldier)));
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
        world.soldiers().len() as usize,
    )
}

#[test]
fn a_world_that_holds_soldiers_is_identical_at_every_thread_count() {
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_soldiers(*config, *frames, THREAD_COUNTS[0]);
        assert!(
            expected.2 > 0,
            "scenario {name}: the world must hold soldiers"
        );
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_soldiers(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the live soldier count differs at {threads} threads"
            );
        }
    }
}

#[test]
fn the_soldier_columns_reach_the_state_hash() {
    // A hash that ignored the soldier columns would pass the golden test
    // while the arena changed underneath it.
    let config = SCENARIOS[2].1;
    let bare = World::new(config).expect("the extent must describe a world");
    let mut peopled = World::new(config).expect("the extent must describe a world");
    populate(&mut peopled);
    assert_ne!(bare.state_hash(), peopled.state_hash());

    let mut moved = World::new(config).expect("the extent must describe a world");
    let kept = populate(&mut moved);
    assert_eq!(moved.state_hash(), peopled.state_hash());
    let corner = Axial::new(
        (moved.grid().width() - 1) as i32,
        (moved.grid().height() - 1) as i32,
    );
    assert_ne!(moved.soldiers().address(kept[0]), Some(corner));
    assert_eq!(moved.place_soldier(kept[0], corner), Ok(true));
    assert_ne!(moved.state_hash(), peopled.state_hash());
}

/// Founds settlements over a world, and destroys part of what it founds.
///
/// The pattern is fixed, so it is the same on every run and at every thread
/// count. The losses matter: a run that only founds never exercises the
/// generation advance, the free queue, or a reused slot.[^1]
///
/// A settlement is fixed to a tile, and two settlements cannot stand on one
/// tile, so the pattern walks distinct tiles.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decisions D3 and D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
fn settle(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let tiles: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .take(23)
        .collect();
    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut kept = Vec::new();
    let mut lost = Vec::new();
    for (index, address) in tiles.into_iter().enumerate() {
        let index = index as u32;
        let settlement = world
            .found_settlement(address, FactionId((index % ceiling) as u16))
            .expect("the address and the faction must be valid");
        // A store of zero is a real state, so the fixture leaves some stores
        // at zero and writes others.[^1]
        //
        // [^1]: Findings register, FND-043. `docs/FINDINGS.md`
        if index % 2 == 1 {
            world
                .set_settlement_store(
                    settlement,
                    CommodityId(0),
                    Fix32::from_int((index % 11) as i16),
                )
                .expect("the commodity is in the set");
        }
        if index % 4 == 3 {
            lost.push((settlement, address));
        } else {
            kept.push(settlement);
        }
    }
    for (settlement, _) in &lost {
        assert!(world.destroy_settlement(*settlement));
    }
    // Found again on the freed tiles, so the run reuses freed slots.
    for (_, address) in &lost {
        kept.push(
            world
                .found_settlement(*address, FactionId(0))
                .expect("the tile is free again"),
        );
    }
    kept
}

/// Runs the frames over a world that holds settlements.
fn run_with_settlements(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let kept = settle(&mut world);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    // Drive the step, then inspect the arena. A column set that no test
    // reaches through the engine is inert.[^1]
    //
    // [^1]: Findings register, FND-041. `docs/FINDINGS.md`
    assert!(world.check_invariants());
    assert!(kept
        .iter()
        .all(|settlement| world.settlements().contains(*settlement)));
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
        world.settlements().len() as usize,
    )
}

#[test]
fn a_world_that_holds_settlements_is_identical_at_every_thread_count() {
    let mut settled = 0;
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_settlements(*config, *frames, THREAD_COUNTS[0]);
        assert!(
            expected.2 > 0,
            "scenario {name}: the fixture must found a settlement"
        );
        settled += 1;
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_settlements(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the live settlement count differs at {threads} threads"
            );
        }
    }
    assert_eq!(
        settled,
        SCENARIOS.len(),
        "every scenario must hold settlements"
    );
}

/// Founds sites that produce and that owe, and returns them.
///
/// The pattern gives some sites more upkeep than production, so a long run
/// reaches the site that cannot pay. That is where determinism is easiest to
/// lose in this pass: who fell short, and in what order the log records it,
/// must come from the slot order and never from the thread that finished
/// first.[^1]
///
/// The caller asserts that the fixture produced both cases. A fixture whose
/// stores never run low would measure itself.[^2]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: Findings register, FND-051. `docs/FINDINGS.md`
fn give_rates(world: &mut World) -> Vec<Entity> {
    let sites = settle(world);
    for (index, site) in sites.iter().enumerate() {
        let index = index as u32;
        // A rate of zero is a real rate, so part of the population earns
        // nothing and owes nothing.
        if index % 5 == 4 {
            continue;
        }
        world
            .set_production_rate(
                *site,
                CommodityId(0),
                Fix32::from_int((index % 4 + 1) as i16),
            )
            .expect("the rate is at or above zero");
        // The upkeep runs ahead of the production at many of these sites, so
        // their stores stay at zero and they fall short on every application.
        // A fixture whose shortfalls land on two adjacent slots cannot tell a
        // join in slot order from a join in the reverse of it.
        world
            .set_upkeep_rate(
                *site,
                CommodityId(0),
                Fix32::from_int((index % 3 + 2) as i16),
            )
            .expect("the rate is at or above zero");
    }
    sites
}

/// Runs the frames over a world whose sites produce and spend.
fn run_with_rates(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize, i64) {
    let mut world = World::new(config).expect("the extent must describe a world");
    world
        .set_economy_schedule(3, 1)
        .expect("the period is inside the range");
    let sites = give_rates(&mut world);
    let mut shortfalls = 0usize;
    // The log holds the last application only, and the last frame is not
    // always a tick that the schedule names. Gathering the bytes of every
    // frame is what makes this comparison read a log that holds something.
    let mut bytes = Vec::new();
    for _ in 0..frames.max(12) {
        world.step(threads).expect("the step must run");
        shortfalls += world.shortfall_log().len();
        bytes.extend_from_slice(world.shortfall_log_bytes());
    }
    assert!(world.check_invariants());
    assert!(sites.iter().all(|site| world.settlements().contains(*site)));
    bytes.extend_from_slice(world.event_log_bytes());
    (
        bytes,
        world.state_hash().finish(),
        shortfalls,
        world.rate_ledger().produced[0].0,
    )
}

#[test]
fn a_world_whose_sites_produce_is_identical_at_every_thread_count() {
    let mut produced = 0;
    let mut fell_short = 0;
    let mut logged = 0;
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_rates(*config, *frames, THREAD_COUNTS[0]);
        if expected.3 > 0 {
            produced += 1;
        }
        fell_short += expected.2;
        logged += expected.0.len();
        for threads in &THREAD_COUNTS[1..] {
            let got = run_with_rates(*config, *frames, *threads);
            assert_eq!(
                got.0, expected.0,
                "scenario {name}: the log differs at {threads} threads"
            );
            assert_eq!(
                got.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                got.2, expected.2,
                "scenario {name}: the shortfall count differs at {threads} threads"
            );
            assert_eq!(
                got.3, expected.3,
                "scenario {name}: the produced total differs at {threads} threads"
            );
        }
    }
    assert_eq!(
        produced,
        SCENARIOS.len(),
        "every scenario must hold a site that produces"
    );
    assert!(
        fell_short > 0,
        "no scenario reached a site that could not pay, so the case is not covered"
    );
    assert!(
        logged > 0,
        "the comparison read no shortfall byte, so it compared two empty logs"
    );
}

#[test]
fn the_settlement_columns_reach_the_state_hash() {
    // A hash that ignored the settlement columns would pass the golden test
    // while the arena changed underneath it.
    let config = SCENARIOS[2].1;
    let bare = World::new(config).expect("the extent must describe a world");
    let mut settled = World::new(config).expect("the extent must describe a world");
    let kept = settle(&mut settled);
    assert!(!kept.is_empty(), "the fixture must found a settlement");
    assert_ne!(bare.state_hash(), settled.state_hash());

    // A second world built the same way agrees, so the difference above is
    // the arena and not the run.
    let mut twin = World::new(config).expect("the extent must describe a world");
    settle(&mut twin);
    assert_eq!(twin.state_hash(), settled.state_hash());

    // The store column reaches the hash. The fixture leaves this store at
    // zero, so the write below is a real change.
    let mut stored = World::new(config).expect("the extent must describe a world");
    let stored_kept = settle(&mut stored);
    assert_eq!(
        stored
            .settlements()
            .store(stored_kept[0])
            .and_then(|store| store.quantity(CommodityId(0))),
        Some(Fix32::ZERO),
        "the fixture must leave this store at zero, or the write below changes nothing"
    );
    assert_eq!(
        stored.set_settlement_store(stored_kept[0], CommodityId(0), Fix32::from_int(3)),
        Ok(true)
    );
    assert_ne!(stored.state_hash(), settled.state_hash());

    // The generation column reaches the hash. A loss and a founding on the
    // same tile leave the same population on the same tiles, at a later
    // generation.
    let mut aged = World::new(config).expect("the extent must describe a world");
    let aged_kept = settle(&mut aged);
    let address = aged
        .settlements()
        .address(aged_kept[0])
        .expect("the settlement is live");
    assert!(aged.destroy_settlement(aged_kept[0]));
    aged.found_settlement(address, FactionId(0))
        .expect("the tile is free again");
    assert_eq!(aged.settlements().len(), settled.settlements().len());
    assert_ne!(aged.state_hash(), settled.state_hash());
}

/// Creates characters over a world, and removes part of what it creates.
///
/// The pattern is fixed, so it is the same on every run and at every thread
/// count. The losses matter: a run that only creates never exercises the
/// generation advance, the free queue, or a reused slot.[^1]
///
/// A living character carries no tile position, so the pattern needs no
/// ground and no address.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decisions D3 and D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
fn people(world: &mut World) -> Vec<Entity> {
    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut kept = Vec::new();
    let mut lost = Vec::new();
    for index in 0..31u32 {
        let character = world
            .create_character(FactionId((index % ceiling) as u16))
            .expect("the faction must be one the world holds");
        // A renown of zero is a real state, so the fixture leaves some at
        // zero and writes others.[^1]
        //
        // [^1]: Findings register, FND-043. `docs/FINDINGS.md`
        if index % 2 == 1 {
            assert!(world.set_character_renown(character, Fix32::from_int((index % 11) as i16)));
        }
        if index % 4 == 3 {
            lost.push(character);
        } else {
            kept.push(character);
        }
    }
    for character in &lost {
        assert!(world.remove_character(*character));
    }
    // Create again, so the run reuses freed slots.
    let mut reused = 0;
    for _ in &lost {
        let made = world
            .create_character(FactionId(0))
            .expect("the creation must succeed");
        if lost.iter().any(|old| old.index() == made.index()) {
            reused += 1;
        }
        kept.push(made);
    }
    assert!(
        reused > 0,
        "the fixture must reuse a freed slot, or the generation advance is untested"
    );
    kept
}

/// Runs the frames over a world that holds characters.
fn run_with_characters(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let kept = people(&mut world);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    // Drive the step, then inspect the arena. A column set that no test
    // reaches through the engine is inert.[^1]
    //
    // [^1]: Findings register, FND-041. `docs/FINDINGS.md`
    assert!(world.check_invariants());
    assert!(kept
        .iter()
        .all(|character| world.characters().contains(*character)));
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
        world.characters().len() as usize,
    )
}

#[test]
fn a_world_that_holds_characters_is_identical_at_every_thread_count() {
    let mut peopled = 0;
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_characters(*config, *frames, THREAD_COUNTS[0]);
        assert!(
            expected.2 > 0,
            "scenario {name}: the fixture must create a character"
        );
        peopled += 1;
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_characters(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the live character count differs at {threads} threads"
            );
        }
    }
    assert_eq!(
        peopled,
        SCENARIOS.len(),
        "every scenario must hold characters"
    );
}

#[test]
fn the_character_columns_reach_the_state_hash() {
    // A hash that ignored the character columns would pass the golden test
    // while the arena changed underneath it.
    let config = SCENARIOS[2].1;
    let bare = World::new(config).expect("the extent must describe a world");
    let mut peopled = World::new(config).expect("the extent must describe a world");
    let kept = people(&mut peopled);
    assert!(!kept.is_empty(), "the fixture must create a character");
    assert_ne!(bare.state_hash(), peopled.state_hash());

    // A second world built the same way agrees, so the difference above is
    // the arena and not the run.
    let mut twin = World::new(config).expect("the extent must describe a world");
    people(&mut twin);
    assert_eq!(twin.state_hash(), peopled.state_hash());

    // The renown column reaches the hash. The fixture leaves this renown at
    // zero, so the write below is a real change.
    let mut renowned = World::new(config).expect("the extent must describe a world");
    let renowned_kept = people(&mut renowned);
    assert_eq!(
        renowned.characters().renown(renowned_kept[0]),
        Some(Fix32::ZERO),
        "the fixture must leave this renown at zero, or the write below changes nothing"
    );
    assert!(renowned.set_character_renown(renowned_kept[0], Fix32::from_int(3)));
    assert_ne!(renowned.state_hash(), peopled.state_hash());

    // The generation column reaches the hash. A loss and a creation leave
    // the same population, at a later generation.
    let mut aged = World::new(config).expect("the extent must describe a world");
    let aged_kept = people(&mut aged);
    let faction = aged
        .characters()
        .faction(aged_kept[0])
        .expect("the character is live");
    assert!(aged.remove_character(aged_kept[0]));
    aged.create_character(faction)
        .expect("the creation must succeed");
    assert_eq!(aged.characters().len(), peopled.characters().len());
    assert_ne!(aged.state_hash(), peopled.state_hash());

    // The birth column reaches the hash. A character created after a step
    // carries a later tick, and nothing else in the two worlds differs.
    let mut early = World::new(config).expect("the extent must describe a world");
    early
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    early.step(1).expect("the step must run");
    let mut late = World::new(config).expect("the extent must describe a world");
    late.step(1).expect("the step must run");
    late.create_character(FactionId(0))
        .expect("the creation must succeed");
    assert_eq!(early.characters().len(), late.characters().len());
    assert_ne!(early.state_hash(), late.state_hash());
}
