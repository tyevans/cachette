//! The golden state hash.
//!
//! The test hashes the whole simulation state each frame and compares the
//! sequence against a stored file.[^1] The padding rule stops the test from
//! failing falsely: an undeclared padding byte is uninitialised, and an
//! uninitialised byte enters the hash.[^2]
//!
//! To record a new golden file, set the environment variable
//! `CACHETTE_UPDATE_GOLDEN` to `1` and run the test. Read the difference
//! before you commit it. A changed golden file is a changed simulation.
//!
//! To add a scenario, add a row to `SCENARIOS` and record the file.
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`

use std::path::PathBuf;

use cachette_core::position::WORK_COMMODITY;
use cachette_core::resource::{Amount, RecoveryRules, ResourceKind};
use cachette_core::site::CommodityId;
use cachette_core::terrain::TileKind;
use cachette_core::types::{FactionId, Fix32};
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::{Axial, World, WorldConfig};

/// Returns a worker row that fights with the given attack and armour.
///
/// The other columns are the worker's, so the units of the fixture gather and
/// build as they did before the row was widened.
const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour,
        ..WORKER_ROW
    }
}

/// The number of frames that a scenario runs unless its row says otherwise.
const FRAMES: u64 = 32;

/// The food that the gathering scenario puts in the store of its site.
///
/// The value only has to outlast the frames the scenario runs, so that the
/// people of the site keep the need they were spawned with.
const STOCKED: Fix32 = Fix32(2000 << 16);

/// The number of frames that a wide scenario runs.
///
/// The shoreline scenario and the founding scenario are here for their
/// extent, not for their duration. The shoreline extent gives a soldier
/// water to meet, and the founding extent gives the engine a good place and
/// a poor one to choose between. A world that wide costs its tile count on
/// every frame, and the state hash test collects most of its cost there.
///
/// Eight frames keep what the extent buys. The soldiers of the shoreline
/// scenario stand over the whole world at frame 0, so a soldier that
/// neighbours water tries to enter it in the first frames. The founding
/// happens before frame 1. What a long run covers, and a short one does
/// not, is the behaviour that accumulates over frames, and the narrow
/// scenarios cover that at the full count for a small part of the cost.
///
/// The budget for this cost is in the development budget register.[^1]
///
/// # References
///
/// [^1]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
const WIDE_FRAMES: u64 = 8;

/// The thread count that records the golden file. The thread-count
/// equivalence test proves that the count does not change the result.
const THREADS: usize = 4;

/// A named scenario. The name is the golden file name.
///
/// The third and fourth scenarios hold soldiers. The first two leave the
/// arena empty, so their hashes cover the tile columns and an empty arena
/// only. A golden file that never sees a populated arena cannot catch a
/// change to how the soldier state is represented, which is the whole purpose
/// of a golden file.[^1]
///
/// The fourth scenario is wider than the coarsest lattice spacing of the
/// generator, so its world holds water as well as open ground. The third is
/// narrower and holds no water at all. A suite of narrow worlds alone cannot
/// see a change to the rule that the ground refuses a unit, because no
/// soldier in it ever meets water.[^2] [^3]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
const SCENARIOS: &[(&str, WorldConfig, Population, u64)] = &[
    (
        "small",
        WorldConfig {
            width: 16,
            height: 16,
            seed: 7,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Empty,
        FRAMES,
    ),
    (
        "default",
        WorldConfig {
            width: 64,
            height: 64,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Empty,
        FRAMES,
    ),
    (
        "soldiers",
        WorldConfig {
            width: 24,
            height: 24,
            seed: 0xfeed_face,
            faction_count: 3,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Spread,
        FRAMES,
    ),
    (
        "shoreline",
        WorldConfig {
            width: 96,
            height: 96,
            seed: 0x0cac_4e77_0068,
            faction_count: 3,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Spread,
        WIDE_FRAMES,
    ),
    (
        "crowd",
        WorldConfig {
            width: 96,
            height: 96,
            seed: 0x0cac_4e77_0023,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Crowd,
        FRAMES,
    ),
    (
        "settlements",
        WorldConfig {
            width: 32,
            height: 32,
            seed: 0x0cac_4e77_0052,
            faction_count: 3,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Settled,
        FRAMES,
    ),
    (
        "gathering",
        WorldConfig {
            width: 48,
            height: 48,
            seed: 0x0cac_4e77_0123,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Gathering,
        FRAMES,
    ),
    (
        "contested",
        WorldConfig {
            width: 32,
            height: 32,
            seed: 0x0cac_4e77_0345,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Contested,
        FRAMES,
    ),
    (
        "founding",
        WorldConfig {
            width: 192,
            height: 192,
            seed: 0x0cac_4e77_0061,
            faction_count: 4,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        },
        Population::Founded,
        WIDE_FRAMES,
    ),
];

/// How a scenario fills its world.
///
/// A world whose units never contend for a tile cannot see the admission
/// rule. The spread scenarios put one unit here and one there, so no target
/// ever reaches its capacity and a golden file taken from them does not move
/// when the rule changes. The crowd scenario fills a patch of ground to the
/// capacity of each tile, so admission must refuse.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Population {
    /// No unit at all. The hash covers the tile columns and an empty arena.
    Empty,
    /// Units spread over the open ground, contending for nothing.
    Spread,
    /// A patch of ground filled to the capacity of each of its tiles.
    Crowd,
    /// Settlements founded over the world, with part of them destroyed.
    ///
    /// A golden file that never sees a settlement cannot catch a change to
    /// how the settlement state is represented.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    Settled,
    /// A run founded with a small group, in a place the engine chose.
    ///
    /// The founding is one of two ways to people a world, and this is the
    /// scenario that covers it.[^1] The other scenarios spawn directly, and
    /// they stay as they are, so a change to the founding moves this file and
    /// leaves the others as the control.
    ///
    /// The extent is wider than the coarsest lattice spacing of the
    /// generator, so the founding has both a good place and a poor one to
    /// choose between.[^2]
    ///
    /// # References
    ///
    /// [^1]: Open decisions register, DEC-030. `docs/DECISIONS.md`
    /// [^2]: Findings register, FND-054. `docs/FINDINGS.md`
    Founded,
    /// Gatherers on deposits, in a world whose deposits recover fast.
    ///
    /// No other scenario gathers, so no other scenario stores a take and no
    /// other scenario recovers one. A golden file that never sees a stored
    /// take cannot catch a change to how the take is represented, and cannot
    /// see recovery at all.[^1]
    ///
    /// The periods are short, so the frames of the scenario reach the case. A
    /// scenario that ran under the default periods would gather and never
    /// recover, and the file would then cover half of the rule.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    Gathering,
    /// Two factions of two unit types, standing on the tiles they share.
    ///
    /// No other scenario holds a meeting, so no other scenario resolves one
    /// and no other golden file moves when the resolution changes. The world
    /// carries a unit type table, because a world whose table nobody filled
    /// holds no contest at all.[^1]
    ///
    /// One pair of types cannot reach the other, and one pair reaches both
    /// ways, so the file covers the threshold and the exchange rather than
    /// one of the two.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    Contested,
}

/// Puts gatherers on deposits and makes the deposits recover fast.
///
/// The units fill each deposit to the capacity of its tile, so a deposit runs
/// out and the resolve must refuse somebody. The world then holds a stored
/// take, which is what recovery ages away.
///
/// The caller asserts that the fixture found deposits. A fixture that found
/// none would record a file that covers nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn gather(world: &mut World) {
    // The periods are a parameter of the kind, so the scenario states them
    // and the engine holds no test value.
    world.set_recovery_rules(
        RecoveryRules::from_ticks([Some(5), Some(7), None]).expect("no period is zero"),
    );
    // The promotion threshold is a parameter of the kind in the same way the
    // recovery periods are, so the scenario states it and the engine holds no
    // test value. **The default is far above what this world reaches**: the
    // best unit here gathers 5 over the frames of the scenario, against a
    // default of 24, so a scenario that took the default would record a file
    // that no promotion ever moves.[^2]
    //
    // [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    world.set_deed_threshold(4);
    // **The choice interval is a parameter of the scenario, for the same
    // reason the threshold above is.** A unit of this world chooses about
    // once over the frames the scenario runs, so it forages, gathers, and
    // never chooses again. It therefore never reaches the option that carries
    // a load home, and the golden file could not move when that option
    // changed. A short interval makes the unit choose again while it still
    // stands in this world.[^3]
    //
    // [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
    world
        .set_choice_schedule(2)
        .expect("the exponent is inside the range");
    // **The carry mark is a parameter of the scenario for the same reason.**
    // The deposits of this world hold between one and ten units each, and the
    // largest load any unit reaches over the frames of the scenario is four,
    // against a default mark far above that. A scenario that took the default
    // would record a file that no unit carrying a load home ever moves.[^3]
    world.set_carry_mark(Amount(2));
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    // Each kind is chosen on purpose. A search for the first kind that a tile
    // carries picked stone on every tile of this world, so the scenario
    // gathered only the kind that never recovers and covered half of the rule
    // while looking complete.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut deposits: Vec<(Axial, ResourceKind)> = Vec::new();
    for kind in [ResourceKind::Food, ResourceKind::Wood, ResourceKind::Stone] {
        let wanted = if kind == ResourceKind::Stone { 2 } else { 5 };
        let found: Vec<(Axial, ResourceKind)> = open
            .iter()
            .filter(|address| world.original_stock(**address, kind) > Some(Amount::ZERO))
            .filter(|address| !deposits.iter().any(|(held, _)| held == *address))
            .take(wanted)
            .map(|address| (*address, kind))
            .collect();
        assert_eq!(
            found.len(),
            wanted,
            "the gathering scenario found only {} deposits of {kind:?}",
            found.len()
        );
        deposits.extend(found);
    }
    // **One deposit carries a site, and its gatherers call it home.** Without
    // that pair no unit of this scenario ever stands on the tile of its own
    // site holding a load, so the delivery writes nothing and the golden file
    // cannot move when the delivery changes. A golden file that cannot move is
    // a guard that has already stopped working, and the promotion pass closed
    // the same gap the same way.[^2]
    //
    // [^2]: Backlog item 0279, let a golden scenario reach the position pass. `docs/backlog/proposed/0279-let-a-golden-scenario-reach-the-position-pass.md`
    let mut home = None;
    for (address, kind) in deposits {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        if home.is_none() && kind == ResourceKind::Food {
            home = world.found_settlement(address, FactionId(0)).ok();
        }
        for ordinal in 0..capacity {
            let unit = world
                .spawn_soldier(address, FactionId((ordinal % 2) as u16))
                .expect("the open tile admits a unit");
            assert!(world.order_gather(unit, kind));
            // **A homed unit must stand away from its own site as well.** A
            // unit spawned on the site tile delivers on every tick, so its
            // load never reaches the mark and it never takes the option that
            // carries a load home. The golden file could then not move when
            // that option changed, which was measured rather than
            // assumed.[^3]
            if let Some(site) = home.filter(|_| ordinal % 2 == 0) {
                assert!(world.set_home_site(unit, Some(site)));
            }
        }
    }
    assert!(
        home.is_some(),
        "the gathering scenario seated no site, so no unit stands at home"
    );
    // **The site holds a store, so its people are fed.** The option that
    // carries a load home is driven by the need a unit still holds, so a
    // starving unit forages instead. Every unit of this scenario starved
    // inside the frames it runs, and the option was therefore unreachable
    // however much a unit carried. That was measured rather than
    // assumed.[^3]
    if let Some(site) = home {
        world
            .set_settlement_store(site, WORK_COMMODITY[ResourceKind::Food.index()], STOCKED)
            .expect("the site takes a store");
    }
}

/// Founds a run: one group for each faction, in a place the engine chose.
///
/// The size of each group is an input to the run, and it is not the
/// population the world is sized for.[^1] The run founds one group for each
/// faction the world holds, and each founding keeps a minimum distance from
/// the foundings before it.[^2]
///
/// # References
///
/// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
/// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
fn found(world: &mut World) {
    let outcomes = world.found_run_for_every_faction(30);
    assert!(
        outcomes
            .iter()
            .all(cachette_core::FoundingOutcome::is_seated),
        "the sample must hold a place that admits every group"
    );
    for outcome in &outcomes {
        let founding = outcome.founding().expect("the faction is seated");
        assert_eq!(
            founding.people().len(),
            30,
            "the founding placed a group of another size"
        );
    }
}

/// Founds settlements over a world, and destroys part of them.
///
/// The losses matter. A run that only founds never exercises the generation
/// advance, the free queue, or a reused slot.[^1]
///
/// Part of the stores stay at zero, because zero is a real state and the
/// golden file must cover it.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decisions D3 and D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
fn settle(world: &mut World) {
    let grid = world.grid();
    let tiles: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| (address.q + address.r) % 5 == 0)
        .collect();
    assert!(
        tiles.len() >= 64,
        "the scenario found only {} tiles to settle",
        tiles.len()
    );
    let mut lost = Vec::new();
    for (step, address) in tiles.into_iter().take(96).enumerate() {
        let settlement = world
            .found_settlement(address, FactionId((step % 3) as u16))
            .expect("the founding must succeed");
        if step % 2 == 1 {
            world
                .set_settlement_store(
                    settlement,
                    CommodityId(0),
                    Fix32::from_int((step % 13) as i16),
                )
                .expect("the commodity is in the set");
        }
        if step % 3 == 0 {
            lost.push((settlement, address));
        }
    }
    for (settlement, _) in &lost {
        assert!(world.destroy_settlement(*settlement));
    }
    for (_, address) in lost.iter().take(8) {
        world
            .found_settlement(*address, FactionId(0))
            .expect("the founding must reuse a freed slot");
    }
}

/// Fills a patch of open ground to the capacity of each tile.
///
/// The patch is taken in index order, so the units sit beside each other and
/// every draw names a tile a neighbour wants. The capacity is read from the
/// ground rather than named here, because the value is content.[^1]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
fn crowd(world: &mut World) {
    let grid = world.grid();
    let patch: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .filter(|address| address.q >= 8 && address.q < 20 && address.r >= 8 && address.r < 20)
        .collect();
    assert!(
        patch.len() >= 16,
        "the crowd scenario found only {} open tiles in its patch",
        patch.len()
    );
    for address in patch {
        let capacity = world.tile_kind(address).map_or(0, TileKind::capacity);
        for ordinal in 0..capacity {
            world
                .spawn_soldier(address, FactionId((ordinal % 2) as u16))
                .expect("the open tile admits a unit");
        }
    }
}

/// Seats two factions on one patch of ground, with a unit type table.
///
/// Each tile of the patch holds units of both factions, so every tile of it is
/// contested and the resolution runs on all of them. The tiles are filled past
/// their capacity, so the units stay where they were put and the scenario
/// keeps holding meetings for the frames it runs.[^1]
///
/// The table is content, and the scenario states it. The engine holds no
/// value of its own.[^2]
///
/// # References
///
/// [^1]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
/// [^2]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
fn contest(world: &mut World) {
    // The contest resolves a meeting only across a pair at war, so the
    // scenario declares one. The edge is read from the world.[^3]
    //
    // [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    let war = world.relation_rules().war_edge - 1;
    assert!(world.set_relation(FactionId(0), FactionId(1), war));
    // The light type reaches the light type and never the heavy one. The
    // heavy type reaches both. One frame therefore covers the threshold that
    // refuses and the exchange that does not.
    world
        .define_unit_type(0, fighter(Fix32::from_int(1), Fix32(Fix32::ONE.0 / 2)))
        .expect("the row is inside the table");
    world
        .define_unit_type(1, fighter(Fix32(Fix32::ONE.0 * 3 / 2), Fix32::from_int(2)))
        .expect("the row is inside the table");
    let grid = world.grid();
    let patch: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .take(48)
        .collect();
    assert!(
        patch.len() >= 16,
        "the contested scenario found only {} open tiles",
        patch.len()
    );
    for (ordinal, address) in patch.iter().enumerate() {
        // Each tile holds twelve units of two factions, which is above the
        // capacity of ordinary ground, so admission refuses every step onto
        // it and the meeting holds.
        for seat in 0..12u32 {
            let faction = FactionId((seat % 2) as u16);
            let unit = world
                .spawn_soldier(*address, faction)
                .expect("the open tile admits a unit");
            // The type pattern shifts with the tile, so the patch holds tiles
            // whose pair cannot reach and tiles whose pair reaches both ways.
            let kind = UnitTypeId::from_u8(((seat as usize + ordinal) % 2) as u8)
                .expect("the number names a row of the table");
            assert!(world.set_unit_type(unit, kind), "the unit is alive");
        }
    }
}

/// Fills a world with soldiers, and frees some of them.
///
/// The frees matter. A run that only spawns never exercises the generation
/// advance, the free queue, or a reused slot, so the golden file would miss
/// a change to any of them.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decisions D3 and D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
fn populate(world: &mut World) {
    // The ground refuses a soldier on water, so the pattern walks the open
    // tiles in index order rather than the whole grid.[^2] The order is the
    // index order of the grid, which is fixed.[^3]
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(
        open.len() >= 64,
        "the scenario left only {} open tiles, too few to people",
        open.len()
    );
    let mut freed = Vec::new();
    for step in 0..64usize {
        let faction = FactionId((step % 3) as u16);
        let soldier = world
            .spawn_soldier(open[step * 7 % open.len()], faction)
            .expect("the spawn must succeed");
        if step % 3 == 0 {
            freed.push(soldier);
        }
    }
    for soldier in freed {
        assert!(world.despawn_soldier(soldier));
    }
    for step in 0..8usize {
        world
            .spawn_soldier(open[step * 11 % open.len()], FactionId(0))
            .expect("the respawn must reuse a freed slot");
    }
}

/// Returns the path of the golden file for one scenario.
fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("state-hash-{name}.txt"))
}

/// Runs a scenario and returns one hash line for each frame.
fn hash_sequence(config: WorldConfig, population: Population, frames: u64) -> String {
    let mut world = World::new(config).expect("the extent must describe a world");
    match population {
        Population::Empty => {}
        Population::Spread => populate(&mut world),
        Population::Crowd => crowd(&mut world),
        Population::Settled => settle(&mut world),
        Population::Founded => found(&mut world),
        Population::Gathering => gather(&mut world),
        Population::Contested => contest(&mut world),
    }
    // The count before the first frame. The contested scenario asserts
    // against it, so the assertion never restates a number the fixture owns.
    let seated: u32 = (0..config.faction_count)
        .map(|faction| world.population_of(FactionId(faction)))
        .sum();
    let mut lines = String::new();
    lines.push_str(&format!("0 {}\n", world.state_hash()));
    for frame in 1..=frames {
        world.step(THREADS).expect("the step must run");
        lines.push_str(&format!("{frame} {}\n", world.state_hash()));
    }
    if population == Population::Gathering {
        // The file must cover what it claims to cover. A run that stored no
        // take, or that recovered nothing, would record a file that moves for
        // every reason except this one.[^1]
        //
        // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
        assert!(
            !world.depletion().is_empty(),
            "the gathering scenario stored no take"
        );
        assert!(
            ResourceKind::ALL
                .iter()
                .any(|kind| world.depletion().returned(*kind).0 > 0),
            "the gathering scenario recovered nothing"
        );
        // A promotion reads what a unit gathered, so this is the one
        // scenario that can reach it. A file recorded from a run that
        // promoted nobody would move for every reason except the one this
        // asserts.[^1]
        assert!(
            !world.characters().is_empty(),
            "the gathering scenario promoted nobody"
        );
    }
    if population == Population::Contested {
        // The file must cover what it claims to cover. A run in which nobody
        // fell would record a file that moves for every reason except the
        // resolution of a meeting.[^1]
        //
        // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
        let standing: u32 = (0..config.faction_count)
            .map(|faction| world.population_of(FactionId(faction)))
            .sum();
        assert!(
            standing < seated,
            "the contested scenario ended nobody: {seated} were seated and {standing} stand"
        );
    }
    lines
}

/// Reports whether the run must record the golden files.
fn recording() -> bool {
    std::env::var("CACHETTE_UPDATE_GOLDEN").as_deref() == Ok("1")
}

#[test]
fn the_state_hash_matches_the_golden_file() {
    for (name, config, population, frames) in SCENARIOS {
        let produced = hash_sequence(*config, *population, *frames);
        let path = golden_path(name);

        if recording() {
            std::fs::create_dir_all(path.parent().expect("the file has a parent"))
                .expect("the directory must exist");
            std::fs::write(&path, &produced).expect("the file must be written");
            continue;
        }

        let stored = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "scenario {name}: cannot read {}: {error}. \
                 Set CACHETTE_UPDATE_GOLDEN=1 to record it.",
                path.display()
            )
        });

        assert_eq!(
            produced, stored,
            "scenario {name}: the state hash sequence changed. \
             A changed sequence is a changed simulation. Read the difference, \
             then set CACHETTE_UPDATE_GOLDEN=1 to record it."
        );
    }
}

#[test]
fn the_hash_changes_when_the_state_changes() {
    // A hash that never changes would pass the golden test and prove
    // nothing.
    let mut world = World::new(SCENARIOS[1].1).expect("the extent must describe a world");
    let before = world.state_hash().finish();
    world.step(THREADS).expect("the step must run");
    assert_ne!(before, world.state_hash().finish());
}
