//! How many distinct needs coexist in one level 1 cell.
//!
//! The choice is decided for each cell and each bucket of need, and a unit
//! reads the answer of its bucket.[^1] **The width of the bucket is the
//! mechanism of that decision and not a detail of it.** Unbucketed, the key is
//! the exact need, and the distinct keys in a cell are bounded by the cohorts
//! standing in it rather than by anything the engine holds. This file measures
//! that bound.
//!
//! **No record sets the width, and no measurement chose it.** The measurement
//! register states that no fixture in this project produces the distribution
//! the choice rests on, because it needs settlements, home sites and a running
//! economy and the benchmark world holds none of the three.[^2] This file is
//! that fixture. An open decision holds what the project does with the
//! answer.[^3]
//!
//! **This is not a cost figure and no blocker governs it.** The simulation is
//! deterministic integer arithmetic, so the distribution below is the same on
//! every machine. A cost figure is not, and one blocker covers every cost
//! figure this project holds.[^4]
//!
//! The fixture drives the engine. It founds settlements, gives each unit a
//! home, and steps, so the needs it reads are the needs the economy
//! produced.[^5]
//!
//! **Two placements bound the answer, and neither is a prediction.** The mixed
//! placement gives neighbouring units different homes, so their stores differ
//! and their needs diverge. The clustered placement gives every unit of one
//! region the same home. The measurement register bounds its own collapse
//! figure the same way, with a packed placement and a scattered one.[^2]
//!
//! Run it and read the table:
//!
//! ```text
//! cargo test -p cachette-core --test need_spread -- --nocapture
//! ```
//!
//! # References
//!
//! [^1]: ADR-0097, the choice is decided for each cell and each bucket of need, decisions D1 and D2. `docs/adrs/draft/adr-0097-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
//! [^2]: Target platform costs, would the choice pass collapse if it decided for each cell. `docs/reference/graviton-costs.md`
//! [^3]: Decisions register, DEC-097. `docs/DECISIONS.md`
//! [^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
//! [^5]: Testing rules, section 5. `.claude/rules/testing.md`

use std::collections::{BTreeMap, BTreeSet};

use cachette_core::choose::NeedBuckets;
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::site::CommodityId;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The commodity that a unit eats. The set holds one.
const FOOD: CommodityId = CommodityId(0);

/// The extent of the measured world.
///
/// The extent gives many level 1 cells, so a median over the cells means
/// something. It is far below the target extent, and the quantity measured
/// here is a count for each cell rather than a total, so the extent does not
/// enter the answer.
const EXTENT: u32 = 256;

/// How many settlements the fixture founds.
///
/// A unit takes its ration from the store of its home, so two units with
/// different homes hold different needs. The count is what makes the mixed
/// placement able to diverge at all.
const SITES: usize = 64;

/// The units that the fixture aims to put in each occupied cell.
///
/// The target density of the project puts about this many units in a level 1
/// cell, and the measurement register states that figure with its
/// derivation.[^1]
///
/// # References
///
/// [^1]: Target platform costs, would the choice pass collapse if it decided for each cell. `docs/reference/graviton-costs.md`
const UNITS_FOR_EACH_CELL: usize = 64;

/// The frames at which the fixture reads the needs.
///
/// A need starts at the full value for every unit, so a run of no frames
/// measures the spawn and not the economy. **The spread is expected to be
/// transient rather than steady.** The decay takes a fixed amount and the
/// gain is a share of a store, so a cohort whose share is below the decay
/// falls to the floor and stays there, and one whose share is above it rises
/// to the ceiling and stays. The values between the two are what a store
/// produces while it empties, so the reading is taken more than once.
const SAMPLES: [usize; 5] = [4, 8, 16, 32, 64];

/// The deficit at which a unit ends.
///
/// No run of this file reaches it. A measurement of the needs of the living
/// needs the population to stay alive, and the default bound ends a starving
/// unit inside the frames above. The bound is a parameter of the rule and no
/// kernel holds one.
const FAR_BOUND: Fix32 = Fix32(NEED_FULL.0 * 4096);

/// The period of the economy in this fixture.
const PERIOD: u32 = 1;

/// Where a unit takes its home from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Placement {
    /// Neighbouring units take different homes, so their stores differ.
    Mixed,
    /// Every unit of one region takes one home.
    Clustered,
}

/// Returns the open ground of a world, in tile order.
fn open_ground(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Builds a world that consumes, and returns it.
///
/// Every site holds a store and a production rate, and no two neighbouring
/// sites hold the same pair. A world in which every site produced the same
/// amount would feed every cohort the same ration, and the needs would stay
/// together whatever the placement. That world would measure the fixture.
fn consuming_world(placement: Placement) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 42,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_economy_schedule(PERIOD, 0)
        .expect("the period is inside the range");
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            FAR_BOUND,
        )
        .expect("every rate is at or above zero"),
    );

    let ground = open_ground(&world);
    let cells = world.pyramid().len();
    let wanted = cells * UNITS_FOR_EACH_CELL;
    assert!(
        ground.len() > SITES * 2,
        "the world holds only {} open tiles",
        ground.len()
    );

    let sites: Vec<Entity> = (0..SITES)
        .map(|index| {
            let place = ground[index * (ground.len() / SITES)];
            let site = world
                .found_settlement(place, FactionId(0))
                .expect("the tile is free");
            // Every site holds a different pair, so no two cohorts draw the
            // same ration and the needs of their units can diverge.
            //
            // The rates straddle what a cohort of this size eats. A site that
            // produces above that keeps its people at the ceiling, one that
            // produces below it lets them fall to the floor, and the store
            // decides how long a site spends between the two. A fixture whose
            // sites all sat on one side of that line would measure the
            // fixture.
            world
                .set_production_rate(site, FOOD, Fix32(NEED_FULL.0 * (index as i32 % 8) / 2))
                .expect("the rate is at or above zero");
            world
                .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 * (1 + index as i32 % 11)))
                .expect("the commodity is in the set");
            site
        })
        .collect();

    let stride = (ground.len() / wanted).max(1);
    let mut placed = 0usize;
    for (ordinal, address) in ground.iter().step_by(stride).enumerate() {
        if placed >= wanted {
            break;
        }
        let Ok(unit) = world.spawn_soldier(*address, FactionId((ordinal % 2) as u16)) else {
            continue;
        };
        let site = match placement {
            // Neighbouring units land on consecutive homes.
            Placement::Mixed => sites[ordinal % SITES],
            // A run of neighbouring units lands on one home.
            Placement::Clustered => sites[(ordinal * SITES / wanted).min(SITES - 1)],
        };
        assert!(world.set_home_site(unit, Some(site)));
        placed += 1;
    }
    assert!(placed > cells, "the fixture placed only {placed} units");

    world
}

/// Returns the needs of each occupied level 1 cell, in cell order.
fn needs_for_each_cell(world: &World) -> BTreeMap<u32, Vec<Fix32>> {
    let arena = world.soldiers();
    let layout = world.pyramid().layout();
    let needs = arena.need_column();
    let mut held: BTreeMap<u32, Vec<Fix32>> = BTreeMap::new();
    for unit in arena.iter() {
        let Some(tile) = arena.tile(unit) else {
            continue;
        };
        let Some(key) = layout.key_of(tile) else {
            continue;
        };
        let cell = layout.block_of_key(key);
        held.entry(cell)
            .or_default()
            .push(needs[unit.index() as usize]);
    }
    held
}

/// Returns the collapse of a cell, in tenths, as a decimal string.
///
/// The arithmetic is integer division. **A ratio of two counts is not
/// simulated state and it is not aggregated state**, so nothing here reaches a
/// state hash. It stays integer anyway, because a float is banned from this
/// crate by name and by a script, and an exception in a measurement is how a
/// ban stops being a ban.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn collapse(units: usize, distinct: usize) -> String {
    if distinct == 0 {
        return String::from("none");
    }
    let tenths = units * 10 / distinct;
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// Returns the median of a set of counts.
fn median(mut counts: Vec<usize>) -> usize {
    counts.sort_unstable();
    counts[counts.len() / 2]
}

/// Reports the distinct needs and the distinct buckets of each cell.
///
/// Returns the largest number of distinct needs that any sample found in the
/// median cell, and the units of the median cell at that sample.
fn report(placement: Placement) -> (usize, usize) {
    let mut world = consuming_world(placement);
    println!("\n{placement:?} placement");
    println!("  | frame | occupied cells | units, median cell | distinct exact needs | at shift 10 | at shift 12 | at shift 14 |");
    println!("  |---|---|---|---|---|---|---|");

    let default = world.need_buckets();
    let mut best = (0usize, 0usize);
    let mut frame = 0usize;
    for sample in SAMPLES {
        while frame < sample {
            world.step(4).expect("the step must run");
            frame += 1;
        }
        let held = needs_for_each_cell(&world);
        if held.is_empty() {
            println!("  | {frame} | 0 | | | | |");
            continue;
        }
        let units = median(held.values().map(Vec::len).collect());
        let exact = median(distinct(&held, None));
        let at = |shift: u32| {
            let width = NeedBuckets::new(shift).expect("the exponent is inside the range");
            let seen = median(distinct(&held, Some(width)));
            format!("{seen} ({}x)", collapse(units, seen))
        };
        let _ = default;
        println!(
            "  | {frame} | {} | {units} | {exact} | {} | {} | {} |",
            held.len(),
            at(10),
            at(12),
            at(14)
        );
        if exact > best.1 {
            best = (units, exact);
        }
    }

    // The width sweep is taken at the last sample, where the economy has run
    // longest.
    let held = needs_for_each_cell(&world);
    let units = median(held.values().map(Vec::len).collect());
    println!("  width sweep at frame {frame}");
    println!("  | bucket exponent | buckets | distinct buckets, median cell | collapse |");
    println!("  |---|---|---|---|");
    for shift in [8u32, 10, 12, 14, 16] {
        let buckets = NeedBuckets::new(shift).expect("the exponent is inside the range");
        let seen = median(distinct(&held, Some(buckets)));
        println!(
            "  | {shift} | {} | {seen} | {} |",
            buckets.count(),
            collapse(units, seen)
        );
    }
    best
}

/// Returns the distinct need count of each cell, exact or bucketed.
fn distinct(held: &BTreeMap<u32, Vec<Fix32>>, buckets: Option<NeedBuckets>) -> Vec<usize> {
    held.values()
        .map(|cell| match buckets {
            None => cell
                .iter()
                .map(|need| need.0 as usize)
                .collect::<BTreeSet<_>>()
                .len(),
            Some(width) => cell
                .iter()
                .map(|need| width.bucket(*need))
                .collect::<BTreeSet<_>>()
                .len(),
        })
        .collect()
}

/// The distinct needs of a cell, measured rather than assumed.
///
/// **The assertions guard the fixture before they report anything.** A world
/// in which every unit held one need would show a collapse of the whole cell
/// at every width, and it would be measuring the fixture rather than the
/// engine.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
#[test]
fn a_world_that_consumes_spreads_the_needs_of_one_cell() {
    let (mixed_units, mixed_exact) = report(Placement::Mixed);
    let (clustered_units, clustered_exact) = report(Placement::Clustered);

    assert!(
        mixed_exact > 1,
        "the mixed fixture put one need in the median cell, so it measures the fixture"
    );
    assert!(
        mixed_exact <= mixed_units,
        "a cell cannot hold more distinct needs than it holds units"
    );
    assert!(
        clustered_exact <= clustered_units,
        "a cell cannot hold more distinct needs than it holds units"
    );
    assert!(
        mixed_exact >= clustered_exact,
        "the mixed placement must not spread the needs less than the clustered one, \
         because it gives neighbouring units different stores"
    );
}
