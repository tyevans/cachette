//! Ranked positions at a site.
//!
//! The tests drive the engine and then read the positions. A position that
//! only a test writes proves that the storage works and not that the step
//! reaches it.[^1]
//!
//! Each test states what the answer depends on, and not only that the answer
//! repeats. A position that named a slot index rather than an identity
//! repeats perfectly, and it reports the wrong unit.[^2]
//!
//! Every fixture asserts that it produced the case that it claims to test.
//! The slot reuse test asserts that the arena did reuse the slot, because a
//! test that never reuses one passes whatever the position stores.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::position;
use cachette_core::position::{PositionError, PositionTable, POSITIONS_PER_SITE};
use cachette_core::site::{CommodityId, Store};
use cachette_core::{
    Axial, Entity, FactionId, Fix32, Grid, ResourceKind, Terrain, Tick, TileIdx, TileKind, World,
    WorldConfig,
};

/// The commodity that every fixture writes.
const GOOD: CommodityId = CommodityId(0);

/// A world that holds ground on every tile the fixtures need.
const CONFIG: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Builds a world that rebalances on every tick, and founds one site.
///
/// The period is one, so a fixture does not have to count frames to reach
/// the pass. The interval is a parameter, and one test below changes it.
fn one_site() -> (World, Entity) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_position_schedule(1, 0)
        .expect("the period is inside the range");
    let site = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the tile is inside the world");
    (world, site)
}

/// Steps the world and asserts that the invariants hold at each frame.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world lost an invariant");
    }
}

/// Returns the index of the first position of a site, and asserts that the
/// site holds one.
fn first_position(world: &World, site: Entity) -> usize {
    let row = world.site_positions(site).expect("the site must be live");
    let index = row
        .iter()
        .position(|entry| entry.exists())
        .expect("the fixture must give the site a position");
    assert_eq!(
        index, 0,
        "the positions of a site sit at the front of a row"
    );
    index
}

#[test]
fn a_site_answers_what_positions_it_holds_and_what_kind_each_one_is() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);

    let row = world.site_positions(site).expect("the site must be live");
    assert_eq!(row.len(), POSITIONS_PER_SITE);
    let held: Vec<_> = row.iter().filter(|entry| entry.exists()).collect();
    assert!(
        !held.is_empty(),
        "a site that lacks everything must open positions"
    );
    for entry in &held {
        assert!(
            entry.kind().is_some(),
            "every position must name a kind of work"
        );
    }
    // The ranks of one kind run from zero upward with nothing missing.
    for kind in ResourceKind::ALL {
        let mut ranks: Vec<u8> = held
            .iter()
            .filter(|entry| entry.kind() == Some(kind))
            .map(|entry| entry.rank())
            .collect();
        ranks.sort_unstable();
        let expected: Vec<u8> = (0..ranks.len() as u8).collect();
        assert_eq!(ranks, expected, "the ranks of one kind must be dense");
    }
}

#[test]
fn a_position_holds_one_unit_or_nobody() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let index = first_position(&world, site);

    // Nobody is a state that the row represents. The position exists, it
    // names a kind, and it holds no unit.
    let row = world.site_positions(site).expect("the site must be live");
    assert!(row[index].exists());
    assert_eq!(row[index].holder_bits(), 0);
    assert_eq!(world.position_holder(site, index), None);

    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    world
        .seat_in_position(site, index, unit)
        .expect("the site holds a position at that index");
    assert_eq!(world.position_holder(site, index), Some(unit));
    assert!(world.check_invariants());
}

#[test]
fn one_unit_holds_at_most_one_position_at_one_site() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    world
        .seat_in_position(site, 0, unit)
        .expect("the site holds a position at that index");
    assert_eq!(
        world.seat_in_position(site, 1, unit),
        Err(PositionError::UnitHoldsAnother(0))
    );
}

#[test]
fn a_position_never_reports_the_unit_that_took_the_slot_of_its_holder() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let index = first_position(&world, site);

    // The unit under test must not stand in slot zero. A holder field of
    // zero means nobody, so a position that stored a bare slot index would
    // read slot zero as a vacancy and the assertion below would never see
    // the wrong unit. The filler takes slot zero and stays alive.
    let filler = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    assert_eq!(filler.index(), 0, "the filler must take slot zero");
    let first = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    assert_ne!(
        first.index(),
        0,
        "the unit under test must not be slot zero"
    );
    world
        .seat_in_position(site, index, first)
        .expect("the site holds a position at that index");
    assert_eq!(world.position_holder(site, index), Some(first));

    assert!(world.despawn_soldier(first), "the unit must be removed");
    let second = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");

    // The fixture must reach the case. A test that never reuses the slot
    // passes whatever the position stores.
    assert_eq!(
        second.index(),
        first.index(),
        "the fixture must make the arena reuse the slot"
    );
    assert_ne!(
        second.generation(),
        first.generation(),
        "the reused slot must carry a later generation"
    );

    // The position named a whole identity, so it does not answer for the
    // unit that took the slot.
    assert_eq!(world.position_holder(site, index), None);
    assert_ne!(world.position_holder(site, index), Some(second));
}

#[test]
fn the_invariant_check_fails_when_a_position_names_a_unit_that_is_gone() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let index = first_position(&world, site);
    // The filler takes slot zero, so the unit under test carries a holder
    // field that a vacancy cannot be mistaken for.
    world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    assert_ne!(unit.index(), 0, "the unit under test must not be slot zero");
    world
        .seat_in_position(site, index, unit)
        .expect("the site holds a position at that index");
    assert!(world.check_invariants());

    assert!(world.despawn_soldier(unit), "the unit must be removed");
    assert!(
        !world.check_invariants(),
        "a position that names a unit that is gone must fail the check"
    );

    // The step releases the holder, so the world holds its invariants again
    // at the next barrier. The release runs on every frame, and not on the
    // rebalance interval.
    run(&mut world, 1, 1);
    assert_eq!(world.position_holder(site, index), None);
}

#[test]
fn the_step_releases_a_dead_holder_on_a_tick_the_rebalance_does_not_name() {
    // The release runs on every frame. The rebalance runs on the interval.
    // A fixture that rebalanced on every tick could not tell the two apart,
    // so this one gives the rebalance a long period and kills the holder
    // between two applications of it.
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let index = first_position(&world, site);
    world
        .set_position_schedule(64, 0)
        .expect("the period is inside the range");

    world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    world
        .seat_in_position(site, index, unit)
        .expect("the site holds a position at that index");
    assert!(world.despawn_soldier(unit), "the unit must be removed");

    // The fixture must reach the case. The next tick must not be one the
    // rebalance runs on, or the release would ride on the rebalance and the
    // test would prove nothing.
    let next = Tick(world.tick().0 + 1);
    assert!(
        !world.position_schedule().due(next),
        "the fixture must step onto a tick the rebalance does not name"
    );
    run(&mut world, 1, 1);
    assert_eq!(world.position_holder(site, index), None);
    assert!(world.check_invariants());
}

#[test]
fn a_site_that_is_destroyed_keeps_no_position() {
    let (mut world, site) = one_site();
    run(&mut world, 1, 1);
    let index = first_position(&world, site);
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    world
        .seat_in_position(site, index, unit)
        .expect("the site holds a position at that index");

    let slot = world
        .settlements()
        .slot_of(site)
        .expect("the site must be live");
    assert!(world.destroy_settlement(site), "the site must be removed");
    assert_eq!(world.site_positions(site), None);
    let row = world
        .positions()
        .row(slot)
        .expect("the slot stays in the table");
    assert!(
        row.iter().all(|entry| !entry.exists()),
        "a lost site leaves no position behind"
    );
    assert!(world.check_invariants());

    // The site founded next in that slot inherits no staff.
    let next = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the tile is free again");
    assert_eq!(
        world.settlements().slot_of(next),
        Some(slot),
        "the fixture must make the arena reuse the slot"
    );
    let row = world.site_positions(next).expect("the site must be live");
    assert!(row.iter().all(|entry| entry.holder_bits() == 0));
}

#[test]
fn a_site_that_lacks_nothing_holds_no_position() {
    let (mut world, site) = one_site();
    // The site wants nothing, so it lacks nothing.
    world
        .prefer_at_sites(&[site], ResourceKind::Food, Fix32::ZERO)
        .expect("the site is live");
    world
        .prefer_at_sites(&[site], ResourceKind::Wood, Fix32::ZERO)
        .expect("the site is live");
    world
        .prefer_at_sites(&[site], ResourceKind::Stone, Fix32::ZERO)
        .expect("the site is live");
    run(&mut world, 1, 1);

    let row = world.site_positions(site).expect("the site must be live");
    assert!(
        row.iter().all(|entry| !entry.exists()),
        "a site that lacks nothing opens no position"
    );
}

#[test]
fn every_position_of_a_site_is_filled_when_the_site_lacks_one_kind() {
    let (mut world, site) = one_site();
    world
        .prefer_at_sites(&[site], ResourceKind::Wood, Fix32::ZERO)
        .expect("the site is live");
    world
        .prefer_at_sites(&[site], ResourceKind::Stone, Fix32::ZERO)
        .expect("the site is live");
    run(&mut world, 1, 1);

    let row = world.site_positions(site).expect("the site must be live");
    let held: Vec<_> = row.iter().filter(|entry| entry.exists()).collect();
    let capacity = TileKind::Plain.capacity() as usize;
    assert_eq!(
        held.len(),
        capacity,
        "a site that lacks one kind gives every position to it"
    );
    assert!(held
        .iter()
        .all(|entry| entry.kind() == Some(ResourceKind::Food)));
}

#[test]
fn the_number_of_positions_of_a_kind_follows_what_the_site_lacks() {
    let (mut world, site) = one_site();
    // The site wants food alone, so every position goes to food.
    world
        .prefer_at_sites(&[site], ResourceKind::Wood, Fix32::ZERO)
        .expect("the site is live");
    world
        .prefer_at_sites(&[site], ResourceKind::Stone, Fix32::ZERO)
        .expect("the site is live");
    run(&mut world, 1, 1);
    let before = world
        .site_positions(site)
        .expect("the site must be live")
        .iter()
        .filter(|entry| entry.kind() == Some(ResourceKind::Food))
        .count();
    assert!(before > 0, "the fixture must open a position of that kind");

    // The store now holds what the site wanted, so the site lacks nothing
    // and the count falls. The answer depends on what the site holds, and
    // not on the preference alone.
    let target = world
        .site_preference(site)
        .expect("the site must be live")
        .target(ResourceKind::Food);
    world
        .set_settlement_store(site, GOOD, target)
        .expect("the site is live");
    run(&mut world, 1, 1);
    let after = world
        .site_positions(site)
        .expect("the site must be live")
        .iter()
        .filter(|entry| entry.kind() == Some(ResourceKind::Food))
        .count();
    assert_eq!(after, 0, "a site that holds what it wanted needs nobody");
    assert_ne!(before, after);
}

#[test]
fn the_rebalance_interval_is_a_schedule_parameter() {
    let (mut world, site) = one_site();
    world
        .set_position_schedule(8, 4)
        .expect("the period is inside the range");
    assert_eq!(world.position_schedule().period(), 8);
    assert_eq!(world.position_schedule().phase(), 4);

    // Three ticks are not enough to reach the phase, so no position exists.
    run(&mut world, 3, 1);
    let row = world.site_positions(site).expect("the site must be live");
    assert!(
        row.iter().all(|entry| !entry.exists()),
        "a tick the schedule does not name must open no position"
    );

    // The fourth tick is the phase, so the pass runs.
    run(&mut world, 1, 1);
    let row = world.site_positions(site).expect("the site must be live");
    assert!(
        row.iter().any(|entry| entry.exists()),
        "the tick the schedule names must open a position"
    );
}

#[test]
fn a_unit_keeps_its_position_across_a_rebalance_that_keeps_the_position() {
    let (mut world, site) = one_site();
    world
        .prefer_at_sites(&[site], ResourceKind::Wood, Fix32::ZERO)
        .expect("the site is live");
    world
        .prefer_at_sites(&[site], ResourceKind::Stone, Fix32::ZERO)
        .expect("the site is live");
    run(&mut world, 1, 1);
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(0))
        .expect("the tile admits a unit");
    world
        .seat_in_position(site, 0, unit)
        .expect("the site holds a position at that index");

    run(&mut world, 4, 1);
    assert_eq!(
        world.position_holder(site, 0),
        Some(unit),
        "a position that survives a rebalance keeps its holder"
    );
}

#[test]
fn the_row_width_of_a_site_follows_the_terrain_capacity_table() {
    // The two bounds come from one place. Raising a capacity in the terrain
    // table raises the width of a row, because the width is folded from that
    // table rather than written beside it.
    let largest = TileKind::ALL
        .iter()
        .map(|kind| kind.capacity())
        .max()
        .expect("the kind set holds a kind");
    assert_eq!(POSITIONS_PER_SITE, largest as usize);
}

#[test]
fn no_site_holds_more_positions_than_its_ground_admits() {
    let (mut world, site) = one_site();
    run(&mut world, 2, 1);
    let tile = world
        .settlements()
        .tile(site)
        .expect("the site must be live");
    let capacity = world
        .terrain()
        .tile_at(tile)
        .expect("the tile is inside the world")
        .kind
        .capacity() as usize;
    let held = world
        .site_positions(site)
        .expect("the site must be live")
        .iter()
        .filter(|entry| entry.exists())
        .count();
    assert!(held <= capacity, "a site cannot staff more than it holds");
    assert!(world.check_invariants());
}

#[test]
fn the_capacity_check_fails_when_a_row_is_wider_than_the_ground() {
    // The world cannot reach this state today, because a founding refuses
    // ground that admits nobody. The check is still the thing that fails if
    // a pass ever writes a row wider than the ground under the site, so the
    // test builds the state that the world excludes.
    // The fixture world is wider than the one the other tests use, because
    // the small one carries no water and the check needs ground that admits
    // nobody.
    let grid = Grid::new(64, 64).expect("the extent describes a grid");
    let terrain = Terrain::new(CONFIG.seed, grid);
    let water = water_tile(terrain).expect("the fixture must find ground that admits nobody");
    let plain = plain_tile(terrain).expect("the fixture must find ground that admits a unit");

    // The rebalance opens a full row over ground that admits units.
    let mut table = PositionTable::new();
    table.open_to(1);
    position::rebalance(&mut table, &[1], &[plain], &[Store::default()], terrain, 1)
        .expect("the columns agree");
    assert!(
        table.count(0) > 0,
        "the fixture must open a position, or the check has nothing to refuse"
    );
    assert_eq!(table.check_capacity(&[plain], &[1], terrain), Ok(true));

    // The same row over ground that admits nobody fails the check.
    assert_eq!(
        table.check_capacity(&[water], &[1], terrain),
        Ok(false),
        "a row wider than the ground must fail the check"
    );
}

#[test]
fn the_rebalance_opens_no_position_over_ground_that_admits_nobody() {
    // The world cannot found a site on this ground, so the rebalance never
    // meets the case inside a step. The pass still reads the capacity table
    // rather than the row width, and this is the test that fails when it
    // stops doing that. Nothing else can fail on it: every kind a settlement
    // may stand on carries the same capacity as the row width today.
    let grid = Grid::new(64, 64).expect("the extent describes a grid");
    let terrain = Terrain::new(CONFIG.seed, grid);
    let water = water_tile(terrain).expect("the fixture must find ground that admits nobody");
    let plain = plain_tile(terrain).expect("the fixture must find ground that admits a unit");

    let mut table = PositionTable::new();
    table.open_to(2);
    position::rebalance(
        &mut table,
        &[1, 1],
        &[water, plain],
        &[Store::default(), Store::default()],
        terrain,
        1,
    )
    .expect("the columns agree");
    assert_eq!(
        table.count(0),
        0,
        "ground that admits nobody opens no position"
    );
    assert!(
        table.count(1) > 0,
        "the fixture must open a position on the other slot, or it measures nothing"
    );
}

/// Returns the first tile of a world whose ground admits nobody.
fn water_tile(terrain: Terrain) -> Option<TileIdx> {
    tile_of_kind(terrain, |kind| kind.capacity() == 0)
}

/// Returns the first tile of a world whose ground admits a unit.
fn plain_tile(terrain: Terrain) -> Option<TileIdx> {
    tile_of_kind(terrain, |kind| kind.capacity() > 0)
}

/// Returns the first tile whose kind answers the test.
fn tile_of_kind(terrain: Terrain, wanted: impl Fn(TileKind) -> bool) -> Option<TileIdx> {
    (0..terrain.grid().tile_count()).map(TileIdx).find(|tile| {
        terrain
            .tile_at(*tile)
            .is_some_and(|ground| wanted(ground.kind))
    })
}

#[test]
fn the_position_tables_agree_at_every_thread_count() {
    // ADR-0001 D4, read against the positions. The pass writes one row for
    // each site, and the rows of a span belong to one thread, so nothing
    // here may depend on the thread count.
    let mut tables = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = World::new(CONFIG).expect("the extent must describe a world");
        world
            .set_position_schedule(2, 1)
            .expect("the period is inside the range");
        // Many sites, so a span of sites crosses a thread boundary at every
        // thread count the test uses. One site would put every row in one
        // span and the test would measure nothing.
        let mut sites = Vec::new();
        for q in 0..6 {
            for r in 0..6 {
                if let Ok(site) = world.found_settlement(Axial::new(q, r), FactionId(0)) {
                    sites.push(site);
                }
            }
        }
        assert!(
            sites.len() > 12,
            "the fixture must found more sites than the largest thread count"
        );
        // The sites must not want the same thing, or every row would hold
        // the same answer and a defect that swapped two rows would pass.
        for (index, site) in sites.iter().enumerate() {
            let target = Fix32(Fix32::ONE.0 * (index as i32 % 5));
            world
                .prefer_at_sites(&[*site], ResourceKind::Wood, target)
                .expect("the site is live");
        }
        run(&mut world, 6, threads);
        tables.push(world.positions().rows().to_vec());
    }
    assert_eq!(tables[0], tables[1], "one thread and two must agree");
    assert_eq!(tables[0], tables[2], "one thread and twelve must agree");
    assert!(
        tables[0].iter().any(|entry| entry.exists()),
        "the fixture must open a position, or the test compares two empty tables"
    );
}
