//! What a unit builds on a tile.
//!
//! The test goes through the public crate API. It reaches into no internal
//! module.[^1]
//!
//! Four families of test live here.
//!
//! The first family asks what the storage costs. A world in which nobody
//! built stores nothing, and the advance reads the sites rather than the
//! world, so its cost does not follow the tile count.
//!
//! The second family asks what the progress depends on. A determinism test
//! cannot tell a correct accumulator from a consistently wrong one, so the
//! tests here assert the arithmetic: several units add exactly, work
//! survives an interruption, and a build does not finish in the tick it
//! started.[^2]
//!
//! The third family asks what a finished upgrade changes, and asserts that
//! destroying it returns the tile exactly.
//!
//! The fourth family drives the engine over a crowded world and inspects
//! what it did. A capability that nothing reaches through the engine ships
//! inert.[^3] The crowd is also the fixture that reaches the extremes: it
//! holds finished builds, builds that stopped partway, several builds
//! advancing in one tick, and a large majority of tiles that carry no
//! upgrade at all.[^4]
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::terrain::TileKind;
use cachette_core::upgrade::{
    capacity_with, gather_rate_with, largest_work, UpgradeKind, UPGRADE_KIND_COUNT,
};
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use proptest::prelude::*;

/// The extent that most tests read.
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
/// The seed that most tests read.
///
/// The world it makes holds an island: a tile whose every neighbour refuses a
/// unit. A unit on it never moves, so a test can put named units on one tile
/// and know they are still there when the advance runs. Most worlds hold
/// none, and this one was found by a scan.
const SEED: u64 = 102;

/// Builds the world under test.
fn world(seed: u64, width: u32, height: u32) -> World {
    let mut world = World::new(WorldConfig {
        width,
        height,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // A unit takes an intent at the interval its level 1 cell schedules, and
    // it does not move before it has one. A test that wants units to move
    // sets the interval to every tick.[^1]
    //
    // [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

/// Returns every address of an extent, in row-major order.
fn addresses(width: u32, height: u32) -> Vec<Axial> {
    let mut all = Vec::with_capacity((width * height) as usize);
    for r in 0..height {
        for q in 0..width {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// Returns an island tile of the world.
///
/// An island is a tile whose every neighbour refuses a unit, so a unit on it
/// never moves. A test can then put named units on one tile and know they are
/// still there when the advance runs.
fn island(world: &World, width: u32, height: u32) -> Axial {
    addresses(width, height)
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .expect("the world must hold an island")
}

/// Returns an island tile that carries a stock of the kind.
fn island_with_stock(world: &World, kind: ResourceKind) -> Axial {
    addresses(WIDTH, HEIGHT)
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world.original_stock(*address, kind)
                    >= Some(Amount(4 + UpgradeKind::Terrace.gather_bonus()))
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .expect("the world must hold an island that carries the kind")
}

/// Puts one soldier on a tile and tells it to build.
fn builder(world: &mut World, address: Axial, kind: UpgradeKind) -> Entity {
    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(world.order_build(unit, kind));
    unit
}

// ---------------------------------------------------------------------------
// What the storage costs
// ---------------------------------------------------------------------------

#[test]
fn a_world_that_built_nothing_stores_no_upgrade() {
    // Storage grows with the number of upgrades, not with the number of
    // tiles. A world nobody built in must therefore hold no entry at all.
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    for _ in 0..8 {
        field.step(2).expect("the step must run");
    }
    assert!(field.check_invariants());
    assert!(
        field.upgrade_sites().is_empty(),
        "the world stored {} entries without a build",
        field.upgrade_sites().len()
    );
}

#[test]
fn one_build_in_a_large_world_stores_one_entry() {
    // The map holds one entry for each improved tile and none for any other.
    // The world below holds tens of thousands of tiles and one upgrade.
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    builder(&mut field, address, UpgradeKind::Road);
    field.step(1).expect("the step must run");

    assert_eq!(field.upgrade_sites().len(), 1);
    assert!(field.grid().tile_count() > 1000);
}

#[test]
fn the_advance_reads_the_sites_and_not_the_world() {
    // One site in a large world costs what one site in a small world costs.
    // The advance walks the builders and the sites. It takes no grid and no
    // tile count, so it cannot read a tile that carries no upgrade.
    let mut small = world(SEED, 16, 16);
    let small_at = addresses(16, 16)
        .into_iter()
        .find(|address| small.admits_a_unit(*address))
        .expect("the small world holds open ground");
    builder(&mut small, small_at, UpgradeKind::Road);
    small.step(1).expect("the step must run");

    let mut large = world(SEED, WIDTH, HEIGHT);
    let large_at = island(&large, WIDTH, HEIGHT);
    builder(&mut large, large_at, UpgradeKind::Road);
    large.step(1).expect("the step must run");

    assert_eq!(small.upgrade_sites().len(), 1);
    assert_eq!(large.upgrade_sites().len(), 1);
    assert_eq!(
        small.last_build_visits(),
        large.last_build_visits(),
        "the advance cost follows the tile count and not the site count"
    );
    assert!(large.grid().tile_count() > 100 * small.grid().tile_count());
}

#[test]
fn a_tick_that_builds_nothing_reads_nothing() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    field
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    field.step(1).expect("the step must run");
    assert_eq!(field.last_build_visits(), 0);
}

// ---------------------------------------------------------------------------
// What the progress depends on
// ---------------------------------------------------------------------------

#[test]
fn a_build_does_not_finish_in_the_tick_it_started() {
    // Work that takes many ticks is the whole point of the shape. A build
    // that finished at once would let nothing hold state between ticks.
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    builder(&mut field, address, UpgradeKind::Road);
    field.step(1).expect("the step must run");

    let site = field.upgrade_at(address).expect("one unit built here");
    assert!(
        !site.is_complete(),
        "one tick finished a build that asks for {} work",
        UpgradeKind::Road.work()
    );
    assert_eq!(field.finished_upgrade(address), None);
    assert!(site.remaining() > 0);
}

#[test]
fn one_unit_finishes_a_build_after_the_work_the_kind_asks_for() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    builder(&mut field, address, UpgradeKind::Road);

    let ticks = UpgradeKind::Road.work();
    for _ in 0..ticks - 1 {
        field.step(1).expect("the step must run");
    }
    assert_eq!(field.finished_upgrade(address), None);
    field.step(1).expect("the step must run");
    assert_eq!(field.finished_upgrade(address), Some(UpgradeKind::Road));
    assert!(field.check_invariants());
}

#[test]
fn two_units_add_their_work_exactly_at_every_thread_count() {
    // Integer addition is order-free, so two contributions in one tick give
    // the same total however the threads produced them.
    for threads in [1usize, 2, 12] {
        let mut one = world(SEED, WIDTH, HEIGHT);
        let address = island(&one, WIDTH, HEIGHT);
        builder(&mut one, address, UpgradeKind::Terrace);
        one.step(threads).expect("the step must run");
        let single = one
            .upgrade_at(address)
            .expect("one unit built here")
            .progress;

        let mut two = world(SEED, WIDTH, HEIGHT);
        builder(&mut two, address, UpgradeKind::Terrace);
        builder(&mut two, address, UpgradeKind::Terrace);
        two.step(threads).expect("the step must run");
        let pair = two
            .upgrade_at(address)
            .expect("two units built here")
            .progress;

        assert_eq!(
            pair.0,
            single.0 * 2,
            "two builders did not add exactly at {threads} threads"
        );
    }
}

#[test]
fn the_progress_is_the_same_at_every_thread_count() {
    let mut hashes = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut field = world(SEED, WIDTH, HEIGHT);
        let address = island(&field, WIDTH, HEIGHT);
        builder(&mut field, address, UpgradeKind::Terrace);
        builder(&mut field, address, UpgradeKind::Terrace);
        builder(&mut field, address, UpgradeKind::Terrace);
        for _ in 0..5 {
            field.step(threads).expect("the step must run");
        }
        hashes.push(field.state_hash());
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
}

#[test]
fn unfinished_work_persists_when_a_unit_stops_and_starts_again() {
    // The world holds no memory of what a unit did unless something stores
    // it. A build that restarted from nothing would make holding ground for
    // a long time worth nothing.
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    let unit = builder(&mut field, address, UpgradeKind::Terrace);

    field.step(1).expect("the step must run");
    field.step(1).expect("the step must run");
    let partway = field
        .upgrade_at(address)
        .expect("the unit built here")
        .progress;
    assert!(partway.0 > 0);

    // The unit stops. The work stays on the tile and nothing advances.
    assert!(field.stop_build(unit));
    for _ in 0..4 {
        field.step(1).expect("the step must run");
    }
    assert_eq!(
        field.upgrade_at(address).expect("the site stayed").progress,
        partway,
        "an idle tick moved a site that nobody was building"
    );

    // The unit starts again. The work continues from where it stopped.
    assert!(field.order_build(unit, UpgradeKind::Terrace));
    field.step(1).expect("the step must run");
    let after = field.upgrade_at(address).expect("the site stayed").progress;
    assert!(
        after.0 > partway.0,
        "the work restarted rather than continued"
    );
}

#[test]
fn a_dead_builder_leaves_the_work_it_did() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    let unit = builder(&mut field, address, UpgradeKind::Terrace);
    field.step(1).expect("the step must run");
    let partway = field
        .upgrade_at(address)
        .expect("the unit built here")
        .progress;

    assert!(field.despawn_soldier(unit));
    field.step(1).expect("the step must run");
    assert_eq!(
        field.upgrade_at(address).expect("the site stayed").progress,
        partway
    );
    assert!(field.check_invariants());
}

#[test]
fn a_tile_carries_one_upgrade_and_the_first_kind_wins() {
    // Two upgrades on one tile would make the return of a destroyed upgrade
    // a question with more than one answer.
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    builder(&mut field, address, UpgradeKind::Terrace);
    builder(&mut field, address, UpgradeKind::Road);
    field.step(1).expect("the step must run");

    let site = field.upgrade_at(address).expect("the units built here");
    assert_eq!(
        site.kind,
        UpgradeKind::Road,
        "the lowest kind takes the tile"
    );
    assert_eq!(field.upgrade_sites().len(), 1);
    // Only the builder of the kind that took the tile contributed.
    assert_eq!(site.progress.0, 1);
}

// ---------------------------------------------------------------------------
// What a finished upgrade changes
// ---------------------------------------------------------------------------

#[test]
fn the_catalogue_holds_more_than_one_kind_and_each_number_names_one() {
    // The catalogue holds more than one kind. One kind would let a reader
    // believe an upgrade is a scalar on the tile rather than a row in a
    // table.
    assert_eq!(UpgradeKind::ALL.len(), UPGRADE_KIND_COUNT);
    assert!(UpgradeKind::ALL.len() > 1);
    for kind in UpgradeKind::ALL {
        assert_eq!(UpgradeKind::from_u8(kind.to_u8()), Some(kind));
        assert!(kind.work() > 0);
    }
    assert_eq!(UpgradeKind::from_u8(UPGRADE_KIND_COUNT as u8), None);
    // Every kind asks for more work than one builder adds in one tick.
    assert!(UpgradeKind::ALL.iter().all(|kind| kind.work() > 1));
    // The two kinds change different properties of a tile. One kind would
    // read as a scalar on the tile rather than a row in a table.
    assert!(UpgradeKind::ALL
        .iter()
        .any(|kind| kind.capacity().is_none()));
    assert!(UpgradeKind::ALL
        .iter()
        .any(|kind| kind.capacity().is_some()));
    assert!(UpgradeKind::ALL.iter().any(|kind| kind.gather_bonus() > 0));
}

#[test]
fn ground_that_admits_nobody_stays_closed_under_every_upgrade() {
    // An upgrade changes how many a tile holds. It never changes whether the
    // tile holds anybody, so every caller that asks about passability alone
    // stays correct.
    assert_eq!(TileKind::Water.capacity(), 0);
    for kind in UpgradeKind::ALL {
        assert_eq!(capacity_with(0, Some(kind)), 0);
    }
}

#[test]
fn a_finished_road_raises_what_the_tile_admits() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    let ground = field
        .tile_kind(address)
        .expect("the address is inside the world")
        .capacity();
    assert_eq!(field.tile_capacity(address), Some(ground));

    builder(&mut field, address, UpgradeKind::Road);
    for _ in 0..UpgradeKind::Road.work() {
        field.step(1).expect("the step must run");
    }
    assert_eq!(field.finished_upgrade(address), Some(UpgradeKind::Road));
    assert_eq!(field.tile_capacity(address), UpgradeKind::Road.capacity());
    assert!(UpgradeKind::Road.capacity() > Some(ground));
}

#[test]
fn a_site_under_construction_changes_nothing() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    let ground = field
        .tile_kind(address)
        .expect("the address is inside the world")
        .capacity();
    builder(&mut field, address, UpgradeKind::Road);
    field.step(1).expect("the step must run");

    assert!(field.upgrade_at(address).is_some());
    assert_eq!(field.finished_upgrade(address), None);
    assert_eq!(field.tile_capacity(address), Some(ground));
}

#[test]
fn a_finished_terrace_raises_what_a_unit_takes_from_the_tile() {
    // The upgrade changes what the tile yields, and the change is read
    // through the same resolve that reads the unimproved rate.
    let mut plain = world(SEED, WIDTH, HEIGHT);
    let kind = ResourceKind::Wood;
    let address = island_with_stock(&plain, kind);

    let idle = plain
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(plain.order_gather(idle, kind));
    plain.step(1).expect("the step must run");
    let without = plain
        .soldier_carry(idle)
        .expect("the unit is live")
        .of(kind)
        .0;

    let mut improved = world(SEED, WIDTH, HEIGHT);
    let mason = builder(&mut improved, address, UpgradeKind::Terrace);
    for _ in 0..UpgradeKind::Terrace.work() {
        improved.step(1).expect("the step must run");
    }
    assert_eq!(
        improved.finished_upgrade(address),
        Some(UpgradeKind::Terrace)
    );
    assert!(improved.stop_build(mason));
    let taker = improved
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(improved.order_gather(taker, kind));
    improved.step(1).expect("the step must run");
    let with = improved
        .soldier_carry(taker)
        .expect("the unit is live")
        .of(kind)
        .0;

    assert!(UpgradeKind::Terrace.gather_bonus() > 0);
    assert_eq!(
        with,
        without + UpgradeKind::Terrace.gather_bonus(),
        "the finished upgrade did not reach the gather resolve"
    );
}

#[test]
fn destroying_an_upgrade_returns_the_tile_to_the_value_it_had() {
    let mut field = world(SEED, WIDTH, HEIGHT);
    let address = island(&field, WIDTH, HEIGHT);
    let before = field
        .tile_capacity(address)
        .expect("the address is inside the world");
    let hash_before = field.state_hash();

    let unit = builder(&mut field, address, UpgradeKind::Road);
    for _ in 0..UpgradeKind::Road.work() {
        field.step(1).expect("the step must run");
    }
    assert_ne!(field.tile_capacity(address), Some(before));

    assert!(field.stop_build(unit));
    assert!(field.destroy_upgrade(address));
    assert_eq!(field.tile_capacity(address), Some(before));
    assert_eq!(field.upgrade_at(address), None);
    assert_eq!(field.finished_upgrade(address), None);
    assert!(field.upgrade_sites().is_empty());
    assert!(field.check_invariants());
    // The world stored the difference from the world the generator made, so
    // removing the entry removes the whole of it. Nothing else held a copy.
    assert_ne!(hash_before, field.state_hash(), "the tick did not move");

    // Destroying nothing removes nothing.
    assert!(!field.destroy_upgrade(address));
    assert!(!field.destroy_upgrade(Axial::new(-1, -1)));
}

// ---------------------------------------------------------------------------
// The accumulator bound
// ---------------------------------------------------------------------------

proptest! {
    // Each case builds a world of the full extent and steps it, so the case
    // count is small. The property is over the arithmetic of one site, and
    // the range of each input is narrow, so a wide sample buys nothing.
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The progress never leaves the range that its kind allows.
    ///
    /// The bound is the largest work in the catalogue, folded from the
    /// catalogue rather than written down here. An unclamped accumulator
    /// lets a builder bank surplus it can never spend, and the surplus then
    /// overflows and enters the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    #[test]
    fn the_progress_never_leaves_the_bound(
        crowd in 1usize..12,
        ticks in 1usize..30,
        kind_at in 0usize..UPGRADE_KIND_COUNT,
    ) {
        let kind = UpgradeKind::ALL[kind_at];
        let mut field = world(SEED, WIDTH, HEIGHT);
        let address = island(&field, WIDTH, HEIGHT);
        for _ in 0..crowd {
            builder(&mut field, address, kind);
        }
        for _ in 0..ticks {
            field.step(1).expect("the step must run");
        }
        let site = field.upgrade_at(address).expect("the crowd built here");
        prop_assert!(site.progress.0 >= 0);
        prop_assert!(site.progress.0 <= kind.work());
        prop_assert!(site.progress.0 <= largest_work());
        prop_assert!(field.check_invariants());
    }
}

#[test]
fn the_bound_is_folded_from_the_catalogue() {
    let most = UpgradeKind::ALL
        .iter()
        .map(|kind| kind.work())
        .max()
        .expect("the catalogue holds a kind");
    assert_eq!(largest_work(), most);
}

#[test]
fn the_composition_functions_add_the_row_of_the_finished_kind() {
    for kind in UpgradeKind::ALL {
        assert_eq!(capacity_with(8, None), 8);
        assert_eq!(
            capacity_with(8, Some(kind)),
            kind.capacity().unwrap_or(8).max(8)
        );
        assert_eq!(gather_rate_with(4, None), 4);
        assert_eq!(gather_rate_with(4, Some(kind)), 4 + kind.gather_bonus());
    }
}

// ---------------------------------------------------------------------------
// The crowd
// ---------------------------------------------------------------------------

/// Builds a crowded world and lets every unit build wherever it stands.
///
/// The units wander, so the fixture reaches the cases the assertions need:
/// builds that finished, builds that a unit walked away from partway,
/// several sites advancing in one tick, and a large majority of tiles that
/// carry no upgrade at all. A fixture copied from the demonstration world
/// would reach none of them.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn crowd(threads: usize) -> World {
    let mut field = world(SEED, 48, 48);
    let open: Vec<Axial> = addresses(48, 48)
        .into_iter()
        .filter(|address| field.admits_a_unit(*address))
        .collect();
    assert!(open.len() > 40, "the crowd needs open ground to stand on");
    for address in open.iter().take(120) {
        let unit = field
            .spawn_soldier(*address, FactionId(0))
            .expect("the ground admits a unit");
        // The kind alternates by the column, so the world holds both.
        let kind = UpgradeKind::ALL[(address.q as usize) % UPGRADE_KIND_COUNT];
        assert!(field.order_build(unit, kind));
    }
    for _ in 0..40 {
        field.step(threads).expect("the step must run");
    }
    field
}

#[test]
fn a_crowd_leaves_a_mark_on_a_few_tiles_and_not_on_the_rest() {
    let field = crowd(2);
    let sites = field.upgrade_sites();
    assert!(!sites.is_empty(), "the crowd built nothing");
    assert!(
        sites.len() < field.grid().tile_count() as usize,
        "every tile carries an upgrade, so the storage is dense"
    );
    // The sparse case is the one the storage exists for: most tiles carry
    // nothing at all.
    assert!(
        sites.len() * 2 < field.grid().tile_count() as usize,
        "half the world carries an upgrade, so the fixture does not reach the sparse case"
    );
    // The map rises and names each tile once.
    assert!(sites.windows(2).all(|pair| pair[0].tile.0 < pair[1].tile.0));
    assert!(field.check_invariants());
}

#[test]
fn the_crowd_reaches_a_finished_build_and_a_build_that_stopped_partway() {
    let field = crowd(2);
    let sites = field.upgrade_sites();
    assert!(
        sites.iter().any(|site| site.is_complete()),
        "no build finished, so the fixture does not reach the finished case"
    );
    assert!(
        sites.iter().any(|site| !site.is_complete()),
        "every build finished, so the fixture does not reach the partway case"
    );
}

#[test]
fn the_crowd_advances_several_builds_in_one_tick() {
    // A tick that advanced one site at a time would not exercise the merge.
    let mut field = crowd(2);
    let before: Vec<i64> = field
        .upgrade_sites()
        .iter()
        .map(|site| site.progress.0)
        .collect();
    field.step(2).expect("the step must run");
    let moved = field
        .upgrade_sites()
        .iter()
        .zip(&before)
        .filter(|(site, was)| site.progress.0 != **was)
        .count();
    assert!(moved > 1, "only {moved} sites advanced in one tick");
}

#[test]
fn the_crowd_gives_the_same_world_at_every_thread_count() {
    let one = crowd(1);
    let two = crowd(2);
    let many = crowd(12);
    assert_eq!(one.state_hash(), two.state_hash());
    assert_eq!(two.state_hash(), many.state_hash());
    assert_eq!(one.upgrade_sites(), two.upgrade_sites());
    assert_eq!(two.upgrade_sites(), many.upgrade_sites());
}

#[test]
fn no_tile_of_the_crowd_holds_more_than_its_capacity() {
    let field = crowd(2);
    for address in addresses(48, 48) {
        let Some(capacity) = field.tile_capacity(address) else {
            continue;
        };
        let standing = field
            .soldier_count_on(address)
            .expect("the address is inside the world");
        assert!(
            standing <= capacity as usize,
            "a tile holds {standing} units against a capacity of {capacity}"
        );
    }
}

#[test]
fn admission_reads_the_capacity_that_a_road_raised() {
    // Admission reads the ground table and the upgrade table through one
    // function. A tile with a finished road therefore admits more units than
    // its ground alone allows.
    //
    // The comparison is against the same world without the road, so the
    // fixture cannot pass by crowding alone.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let plain = crowded_tile(false);
    let paved = crowded_tile(true);
    let ground = TileKind::Plain.capacity() as usize;

    assert!(
        plain <= ground,
        "the unimproved tile held {plain} units against a ground capacity of {ground}"
    );
    assert!(
        paved > ground,
        "the paved tile held {paved} units, which the ground alone already allows"
    );
    assert!(
        paved
            <= UpgradeKind::Road
                .capacity()
                .expect("a road raises the capacity") as usize
    );
}

/// Crowds one tile and returns the most units that ever stood on it.
///
/// Every unit is told to build a road when the caller asks for a paved
/// world, so the crowd paves the tile it crowds. Nothing holds a unit in
/// place: the units wander, and the tile they wander onto most is the one
/// the neighbours feed.
///
/// The two worlds run the same seed and spawn the same units in the same
/// order, so the movement draws are identical and only the capacity differs.
fn crowded_tile(paved: bool) -> usize {
    let mut field = world(SEED, 48, 48);
    let (target, sides) = crowdable(&field);

    // The neighbours carry far more units than the target can ever hold, so
    // admission is what decides the count and not the supply.
    for side in &sides {
        for _ in 0..24 {
            let unit = field
                .spawn_soldier(*side, FactionId(0))
                .expect("a spawn may over-fill a tile");
            if paved {
                assert!(field.order_build(unit, UpgradeKind::Road));
            }
        }
    }

    let mut most = 0usize;
    for _ in 0..40 {
        field.step(1).expect("the step must run");
        most = most.max(
            field
                .soldier_count_on(target)
                .expect("the address is inside the world"),
        );
    }
    if paved {
        assert_eq!(
            field.finished_upgrade(target),
            Some(UpgradeKind::Road),
            "the crowd did not pave the tile it crowded"
        );
    } else {
        assert_eq!(field.upgrade_at(target), None);
    }
    most
}

/// Returns a tile of level open ground whose every neighbour is the same.
///
/// The neighbours hold the crowd, so every one of them must admit a unit.
/// The ground is one kind, so the ground capacity of the target is a value
/// the test can name.
fn crowdable(world: &World) -> (Axial, Vec<Axial>) {
    for address in addresses(48, 48) {
        if world.tile_kind(address) != Some(TileKind::Plain) {
            continue;
        }
        let sides: Vec<Axial> = world
            .grid()
            .neighbours(address)
            .into_iter()
            .flatten()
            .collect();
        if sides.len() == 6
            && sides
                .iter()
                .all(|side| world.tile_kind(*side) == Some(TileKind::Plain))
        {
            return (address, sides);
        }
    }
    panic!("the world holds no level tile surrounded by level ground");
}
