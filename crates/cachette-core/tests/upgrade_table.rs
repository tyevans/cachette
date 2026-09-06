//! The upgrade table: a category, a ground fit and a level.
//!
//! An upgrade is a row of a table that the world is built with. A row is one
//! category at one level. It names the ground it fits, the work it takes and
//! the columns a pass reads.[^1]
//!
//! These tests go through the public interface of the world.[^2] Each one
//! reaches an extreme of the table: a ground that no row fits, a category at
//! its top, a tile that carries another category, and a row that a caller
//! wrote at run time.[^3]
//!
//! # References
//!
//! [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 to D4. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^2]: Testing rules, section 6. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::choose::{self, ChoiceSchedule};
use cachette_core::holding::ReachRules;
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::terrain::TileKind;
use cachette_core::upgrade::{
    BuildRefusal, UpgradeCategory, UpgradeRow, DEFAULT_UPGRADE_TABLE, UPGRADE_LEVEL_COUNT,
};
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent that these tests read.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds every kind of ground. A world smaller than
/// that spacing holds one terrain, and a test over it measures the
/// fixture.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const WIDTH: u32 = 192;
/// The number of rows of that extent.
const HEIGHT: u32 = 192;
/// The seed that these tests read.
const SEED: u64 = 102;

/// Builds the world under test.
///
/// The fixture founds one city of the building faction and gives it a reach
/// that covers the world, so a test measures the table and never the ground
/// rule.[^1]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
fn world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    let reach = WIDTH + HEIGHT;
    world.set_reach_rules(ReachRules::new(reach, 1, reach));
    let seat = addresses()
        .into_iter()
        .find(|address| world.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    world
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    world.step(1).expect("the step must run");
    world
}

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// Returns a tile whose every neighbour refuses a unit, of one ground kind.
///
/// A unit on such a tile never moves, so a test puts a builder there and
/// knows it is still there when the advance runs.
fn island_of(world: &World, ground: TileKind) -> Option<Axial> {
    addresses().into_iter().find(|address| {
        world.tile_kind(*address) == Some(ground)
            && world.admits_a_unit(*address)
            && world
                .grid()
                .neighbours(*address)
                .iter()
                .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
    })
}

/// Returns the island of forest ground that these tests build on.
fn island(world: &World) -> Axial {
    island_of(world, TileKind::Forest).expect("the world must hold an island of forest")
}

/// Returns a tile of one ground kind that admits a unit.
fn ground_of(world: &World, ground: TileKind) -> Axial {
    addresses()
        .into_iter()
        .find(|address| world.tile_kind(*address) == Some(ground) && world.admits_a_unit(*address))
        .expect("the world must hold the ground")
}

/// Puts one soldier on a tile.
fn soldier(world: &mut World, address: Axial) -> Entity {
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit")
}

/// Zones one project, puts one soldier on the tile, and tells it to build.
///
/// **The plan comes first.** A category whose row asks for no held ground is
/// laid only inside a project, so a fixture that ordered the build alone
/// would measure the refusal. The plan refuses to zone a category that asks
/// for held ground on ground nobody holds, so the zone call may fail and the
/// build order then answers for itself.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
fn builder(world: &mut World, address: Axial, category: UpgradeCategory) -> Entity {
    let unit = soldier(world, address);
    let _ = world.zone_project(FactionId(0), address, category);
    world
        .order_build(unit, category)
        .expect("the ground fits the category");
    unit
}

/// Steps the world until the level on a tile rises, or the patience runs out.
fn step_until_level(world: &mut World, address: Axial, level: u8, patience: u64) -> u64 {
    for taken in 1..=patience {
        world.step(1).expect("the step must run");
        if world.upgrade_level(address) >= level {
            return taken;
        }
    }
    panic!("the level {level} did not stand after {patience} ticks");
}

// ---------------------------------------------------------------------------
// The ground fit
// ---------------------------------------------------------------------------

#[test]
fn a_category_the_ground_does_not_fit_is_refused_and_one_that_fits_is_not() {
    // The fixture stands the builder on high ground, which is the extreme of
    // the ground fit: the road and the terrace stop there and the wonder does
    // not.[^1]
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    let mut field = world(SEED);
    let high = ground_of(&field, TileKind::Mountain);
    let unit = soldier(&mut field, high);

    assert_eq!(
        field.order_build(unit, UpgradeCategory::ROAD),
        Err(BuildRefusal::GroundDoesNotFit {
            category: UpgradeCategory::ROAD,
            ground: TileKind::Mountain,
        }),
        "the road fits high ground"
    );
    assert_eq!(
        field.order_build(unit, UpgradeCategory::TERRACE),
        Err(BuildRefusal::GroundDoesNotFit {
            category: UpgradeCategory::TERRACE,
            ground: TileKind::Mountain,
        }),
        "the terrace fits high ground"
    );
    // The refusal is of the row and not of the ground. A category whose row
    // fits high ground is accepted on the same tile.
    assert_eq!(field.order_build(unit, UpgradeCategory::WONDER), Ok(()));
    assert_eq!(field.build_order(unit), Some(Some(UpgradeCategory::WONDER)));
}

#[test]
fn a_refused_order_builds_nothing_when_the_world_steps() {
    // The verb refuses at the moment of the order, and the pass applies the
    // same resolution on every step. A refused order that reached the pass
    // would finish anyway.
    let mut field = world(SEED);
    let high = ground_of(&field, TileKind::Mountain);
    let unit = soldier(&mut field, high);
    assert!(field.order_build(unit, UpgradeCategory::ROAD).is_err());
    for _ in 0..12 {
        field.step(1).expect("the step must run");
    }
    assert_eq!(field.upgrade_at(high), None);
    assert!(field.upgrade_sites().is_empty());
}

#[test]
fn the_pass_resolves_the_row_again_on_every_step() {
    // **The verb and the pass call one resolution.** The order below is
    // accepted, and then the caller rewrites the row so that it no longer
    // fits the ground under the builder. The pass must refuse the order it
    // once accepted.[^1]
    //
    // The assertion reads the level and not the work done. The work done
    // returns to zero at a raise and counts up again, so a build that ran on
    // can show the work it showed before.[^2]
    //
    // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    // [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
    let mut field = world(SEED);
    let address = island(&field);
    assert_eq!(field.tile_kind(address), Some(TileKind::Forest));
    builder(&mut field, address, UpgradeCategory::ROAD);
    field.step(1).expect("the step must run");
    let started = field
        .upgrade_at(address)
        .expect("the builder started")
        .progress
        .0;
    assert!(started > 0);
    assert_eq!(field.upgrade_level(address), 0);

    let first = DEFAULT_UPGRADE_TABLE
        .row(UpgradeCategory::ROAD, 1)
        .expect("the default table holds the row");
    let narrowed = UpgradeRow {
        ground_fit: cachette_core::upgrade::ground_bit(TileKind::Plain),
        ..first
    };
    field
        .define_upgrade_row(UpgradeCategory::ROAD.to_u8(), 1, narrowed)
        .expect("the category and the level are inside the table");
    // The run is longer than the work of the row, so a build that ran on
    // would stand at a level by the end of it.
    for _ in 0..u64::from(first.work) + 4 {
        field.step(1).expect("the step must run");
    }
    assert_eq!(
        field.upgrade_level(address),
        0,
        "the pass advanced a build that the row no longer fits"
    );
    assert_eq!(
        field
            .upgrade_at(address)
            .expect("the entry stays")
            .progress
            .0,
        started,
        "the pass advanced a build that the row no longer fits"
    );
}

#[test]
fn no_row_fits_water_and_nothing_stands_on_it() {
    // Water holds nobody, so no unit ever stands there to build. The table
    // states the same thing: no row fits water.
    assert_eq!(TileKind::Water.capacity(), 0);
    for row in DEFAULT_UPGRADE_TABLE.rows() {
        assert!(
            !row.exists() || !row.fits(TileKind::Water),
            "a row fits the ground that holds nobody"
        );
    }
    let field = world(SEED);
    let wet = ground_of_water(&field);
    assert!(!field.admits_a_unit(wet));
    assert_eq!(field.tile_capacity(wet), Some(0));
}

/// Returns a tile of water.
fn ground_of_water(world: &World) -> Axial {
    addresses()
        .into_iter()
        .find(|address| world.tile_kind(*address) == Some(TileKind::Water))
        .expect("the world must hold water")
}

// ---------------------------------------------------------------------------
// The level
// ---------------------------------------------------------------------------

#[test]
fn a_level_rises_in_place_and_the_entry_count_does_not_grow() {
    let mut field = world(SEED);
    let address = island(&field);
    builder(&mut field, address, UpgradeCategory::ROAD);

    let first = DEFAULT_UPGRADE_TABLE
        .row(UpgradeCategory::ROAD, 1)
        .expect("the default table holds the first level");
    let second = DEFAULT_UPGRADE_TABLE
        .row(UpgradeCategory::ROAD, 2)
        .expect("the default table holds the second level");

    let to_first = step_until_level(&mut field, address, 1, i64::from(first.work) as u64 + 4);
    assert_eq!(to_first, u64::from(first.work), "one builder adds one work");
    assert_eq!(field.upgrade_sites().len(), 1);
    // The work done returns to zero when the level rises.
    let site = field.upgrade_at(address).expect("the unit built here");
    assert_eq!(site.level, 1);
    assert_eq!(site.progress.0, 0);

    let to_second = step_until_level(&mut field, address, 2, i64::from(second.work) as u64 + 4);
    assert_eq!(
        to_second,
        u64::from(second.work),
        "the second level takes its own work"
    );
    assert_eq!(
        field.upgrade_sites().len(),
        1,
        "the raise wrote a second entry"
    );
    assert_eq!(field.upgrade_level(address), 2);
    assert!(field.check_invariants());
}

#[test]
fn the_second_level_changes_its_own_column() {
    let mut field = world(SEED);
    let address = island(&field);
    let ground = field
        .tile_kind(address)
        .expect("the address is inside the world")
        .capacity();
    builder(&mut field, address, UpgradeCategory::ROAD);

    let first = DEFAULT_UPGRADE_TABLE
        .row(UpgradeCategory::ROAD, 1)
        .expect("the default table holds the first level");
    let second = DEFAULT_UPGRADE_TABLE
        .row(UpgradeCategory::ROAD, 2)
        .expect("the default table holds the second level");
    assert!(second.capacity_change > first.capacity_change);

    step_until_level(&mut field, address, 1, u64::from(first.work) + 4);
    assert_eq!(field.tile_capacity(address), Some(first.capacity_change));
    assert!(first.capacity_change > ground);

    step_until_level(&mut field, address, 2, u64::from(second.work) + 4);
    assert_eq!(
        field.tile_capacity(address),
        Some(second.capacity_change),
        "the second level did not reach the capacity reader"
    );
}

#[test]
fn a_category_at_its_top_refuses_the_next_order() {
    let mut field = world(SEED);
    let address = island(&field);
    let unit = builder(&mut field, address, UpgradeCategory::ROAD);
    let top = DEFAULT_UPGRADE_TABLE.top_level(UpgradeCategory::ROAD);
    assert_eq!(top as usize, UPGRADE_LEVEL_COUNT);

    let patience = DEFAULT_UPGRADE_TABLE.largest_work() as u64 * 4;
    step_until_level(&mut field, address, top, patience);

    assert_eq!(
        field.order_build(unit, UpgradeCategory::ROAD),
        Err(BuildRefusal::CategoryAtTop {
            category: UpgradeCategory::ROAD,
            level: top,
        })
    );
    // A builder at the top banks nothing, so the work done stays at zero.
    for _ in 0..8 {
        field.step(1).expect("the step must run");
    }
    let site = field.upgrade_at(address).expect("the road stands here");
    assert_eq!(site.level, top);
    assert_eq!(site.progress.0, 0);
    assert!(field.check_invariants());
}

#[test]
fn a_tile_that_carries_another_category_refuses_the_order() {
    let mut field = world(SEED);
    let address = island(&field);
    builder(&mut field, address, UpgradeCategory::ROAD);
    let other = soldier(&mut field, address);
    assert_eq!(
        field.order_build(other, UpgradeCategory::TERRACE),
        Ok(()),
        "the tile carries nothing yet"
    );
    field.step(1).expect("the step must run");

    let asked = soldier(&mut field, address);
    assert_eq!(
        field.order_build(asked, UpgradeCategory::TERRACE),
        Err(BuildRefusal::TileHoldsAnother {
            standing: UpgradeCategory::ROAD,
            asked: UpgradeCategory::TERRACE,
        }),
        "the lowest category took the tile"
    );
}

// ---------------------------------------------------------------------------
// A pass reads a column and never a name
// ---------------------------------------------------------------------------

/// The work the written row asks for. It is small, so the test steps little.
const WRITTEN_WORK: u32 = 3;
/// The yield the written row adds.
///
/// The value is bounded by the stock that an island of this world carries. A
/// unit takes the rate or the stock, whichever is smaller, so a larger yield
/// would measure the stock rather than the column.
const WRITTEN_YIELD: u32 = 2;

#[test]
fn the_gather_pass_honours_a_row_that_no_code_names() {
    // **This is the test of the rule that no pass branches on a category.**
    // The row below is written at run time, into the open category. No code
    // in the engine names that category, and no code holds a rule for it. The
    // gather resolve must still add its yield column.[^1]
    //
    // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    let kind = ResourceKind::Wood;
    let plain = world(SEED);
    let address = island_with_stock(&plain, kind);

    let mut bare = world(SEED);
    let idle = soldier(&mut bare, address);
    hold_the_choice(&mut bare, 1);
    assert!(bare.order_gather(idle, kind));
    bare.step(1).expect("the step must run");
    let without = bare
        .soldier_carry(idle)
        .expect("the unit is live")
        .of(kind)
        .0;

    let mut written = world(SEED);
    written
        .define_upgrade_row(
            UpgradeCategory::OPEN.to_u8(),
            1,
            UpgradeRow {
                ground_fit: cachette_core::upgrade::FITS_EVERY_LAND,
                work: WRITTEN_WORK,
                yield_change: WRITTEN_YIELD,
                ..UpgradeRow::NONE
            },
        )
        .expect("the category and the level are inside the table");
    let mason = builder(&mut written, address, UpgradeCategory::OPEN);
    step_until_level(&mut written, address, 1, u64::from(WRITTEN_WORK) + 4);
    assert!(written.stop_build(mason));

    let taker = soldier(&mut written, address);
    hold_the_choice(&mut written, 1);
    assert!(written.order_gather(taker, kind));
    written.step(1).expect("the step must run");
    let with = written
        .soldier_carry(taker)
        .expect("the unit is live")
        .of(kind)
        .0;

    assert_eq!(
        with,
        without + WRITTEN_YIELD,
        "the gather resolve did not read the column of a row it does not name"
    );
}

#[test]
fn a_written_row_enters_the_state_hash() {
    // Two worlds built with different tables never hash the same.[^1]
    //
    // [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    let plain = world(SEED);
    let mut written = world(SEED);
    assert_eq!(plain.state_hash(), written.state_hash());
    written
        .define_upgrade_row(
            UpgradeCategory::OPEN.to_u8(),
            1,
            UpgradeRow {
                ground_fit: cachette_core::upgrade::FITS_EVERY_LAND,
                work: WRITTEN_WORK,
                yield_change: WRITTEN_YIELD,
                ..UpgradeRow::NONE
            },
        )
        .expect("the category and the level are inside the table");
    assert_ne!(plain.state_hash(), written.state_hash());
    assert!(written.check_invariants());
}

#[test]
fn the_table_refuses_a_category_and_a_level_it_does_not_hold() {
    let mut field = world(SEED);
    assert!(field
        .define_upgrade_row(u8::MAX, 1, UpgradeRow::NONE)
        .is_err());
    assert!(field
        .define_upgrade_row(UpgradeCategory::OPEN.to_u8(), 0, UpgradeRow::NONE)
        .is_err());
    assert!(field
        .define_upgrade_row(
            UpgradeCategory::OPEN.to_u8(),
            UPGRADE_LEVEL_COUNT as u8 + 1,
            UpgradeRow::NONE
        )
        .is_err());
}

#[test]
fn a_category_with_no_row_is_refused() {
    let mut field = world(SEED);
    let address = island(&field);
    let unit = soldier(&mut field, address);
    assert_eq!(
        field.order_build(unit, UpgradeCategory::OPEN),
        Err(BuildRefusal::CategoryAtTop {
            category: UpgradeCategory::OPEN,
            level: 0,
        })
    );
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn a_world_of_levels_is_the_same_at_every_thread_count() {
    let one = raised(1);
    let two = raised(2);
    let many = raised(12);
    assert_eq!(one.state_hash(), two.state_hash());
    assert_eq!(two.state_hash(), many.state_hash());
    assert_eq!(one.upgrade_sites(), two.upgrade_sites());
    assert_eq!(two.upgrade_sites(), many.upgrade_sites());
    // The fixture reaches a second level, so the comparison covers the raise
    // and not the first build alone.
    assert!(
        one.upgrade_sites().iter().any(|site| site.level > 1),
        "no entry reached a second level, so the fixture measures the first"
    );
}

/// Builds a world in which several tiles carry a raised upgrade.
fn raised(threads: usize) -> World {
    let mut field = world(SEED);
    let open: Vec<Axial> = addresses()
        .into_iter()
        .filter(|address| field.tile_kind(*address) == Some(TileKind::Plain))
        .take(60)
        .collect();
    // The plan zones every open tile, because a road is laid only inside a
    // project and the builders wander between the tiles.[^2]
    //
    // [^2]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    field.set_plan_rules(field.plan_rules().with_bound(open.len() as u32));
    for address in &open {
        let _ = field.zone_project(FactionId(0), *address, UpgradeCategory::ROAD);
    }
    for address in &open {
        for _ in 0..4 {
            let unit = soldier(&mut field, *address);
            if field.order_build(unit, UpgradeCategory::ROAD).is_err() {
                continue;
            }
        }
    }
    for _ in 0..24 {
        field.step(threads).expect("the step must run");
    }
    field
}

// ---------------------------------------------------------------------------
// The fixtures the gather test needs
// ---------------------------------------------------------------------------

/// Returns an island tile that carries a stock of the kind.
fn island_with_stock(world: &World, kind: ResourceKind) -> Axial {
    addresses()
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world.original_stock(*address, kind) >= Some(Amount(4 + WRITTEN_YIELD))
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .unwrap_or_else(|| panic!("no island carries {}", best_stock(world, kind)))
}

/// Returns the largest stock of the kind on any island.
fn best_stock(world: &World, kind: ResourceKind) -> u32 {
    addresses()
        .into_iter()
        .filter(|address| {
            world.admits_a_unit(*address)
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .filter_map(|address| world.original_stock(address, kind))
        .map(|amount| amount.0)
        .max()
        .unwrap_or(0)
}

/// Puts the choice far enough apart that it does not replace a gather order.
///
/// **The choice pass writes the gather order of a unit whose level 1 cell
/// chooses on that frame, and that write replaces the order a caller
/// gave.**[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^2]: Findings register, FND-228. `docs/FINDINGS.md`
fn hold_the_choice(world: &mut World, frames: u64) {
    let schedule =
        ChoiceSchedule::new(choose::PERIOD_LOG2_CEILING).expect("the exponent is inside the range");
    world
        .set_choice_schedule(schedule.period_log2())
        .expect("the exponent is inside the range");
    let now = world.tick().0;
    for cell in 0..world.pyramid().len() as u32 {
        for ahead in 1..=frames {
            assert!(
                !schedule.chooses_now(cell, now + ahead),
                "cell {cell} chooses inside the run, so the choice replaces \
                 the gather order that this test gave"
            );
        }
    }
}
