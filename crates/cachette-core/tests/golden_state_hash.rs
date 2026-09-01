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

use cachette_core::site::CommodityId;
use cachette_core::terrain::TileKind;
use cachette_core::types::{FactionId, Fix32};
use cachette_core::{Axial, World, WorldConfig};

/// The number of frames that a scenario runs unless its row says otherwise.
const FRAMES: u64 = 32;

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
        },
        Population::Settled,
        FRAMES,
    ),
    (
        "founding",
        WorldConfig {
            width: 192,
            height: 192,
            seed: 0x0cac_4e77_0061,
            faction_count: 4,
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
/// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
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
    }
    let mut lines = String::new();
    lines.push_str(&format!("0 {}\n", world.state_hash()));
    for frame in 1..=frames {
        world.step(THREADS).expect("the step must run");
        lines.push_str(&format!("{frame} {}\n", world.state_hash()));
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
