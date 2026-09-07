//! A faction reads what it saw, and never what it cannot see.
//!
//! Every test here drives the world step. The step is what runs the
//! observation pass, so a test that called the pass itself would prove that
//! the pass works and not that a reader reaches it.[^1]
//!
//! **Every fixture asserts that it produced the case the test needs.** A
//! world chosen to look right supplies no extreme, so each fixture states
//! the distribution it needs and fails when the world does not give it.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::faction_view::{Admit, FactionTile, Sighting};
use cachette_core::{Axial, Entity, FactionId, SightRules, World, WorldConfig};

/// A world wide enough to hold two factions that never meet.
const TWO_SIDES: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// A world small enough for one camp to watch every tile of it.
///
/// The whole world is one block of the lattice, so a summary over the cell
/// covers the whole world.
const ONE_BLOCK: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The faction that watches in every fixture below.
const WATCHER: FactionId = FactionId(0);

/// The faction that the watcher must not read.
const STRANGER: FactionId = FactionId(1);

/// The exponent that keeps a unit still.
///
/// A unit takes a movement intent at the interval its cell schedules. A long
/// interval stops a unit taking one inside a short test, so each test below
/// measures the reader and not the movement pass.
const KEEP_STILL: u32 = 12;

/// Builds a world in which no unit takes a movement intent.
fn a_still_world(config: WorldConfig) -> World {
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world
}

/// Returns an address that admits a unit, near the one asked for.
///
/// The terrain comes from the seed, so the tile a test names may hold water.
/// The search is a spiral over the rings around the address, and it takes the
/// first tile of the lowest ring. The order is fixed, so two runs return one
/// answer.
fn ground_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no ground that admits a unit near {wanted:?}");
}

/// Puts a unit of one faction on a tile and returns its identity.
fn a_unit_at(world: &mut World, address: Axial, faction: FactionId) -> Entity {
    world
        .spawn_soldier(address, faction)
        .expect("the fixture places a unit on ground that admits one")
}

/// Returns the block of the fog lattice that covers an address.
fn block_of(world: &World, address: Axial) -> u32 {
    let layout = world.observation().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address lies inside the world");
    let key = layout.key_of(tile).expect("the tile carries a key");
    layout.block_of_key(key)
}

/// Returns how many units stand on a tile, whatever any faction can see.
fn truth_units_on(world: &World, address: Axial) -> usize {
    world
        .bridge()
        .count_on_tile(world.soldiers(), address)
        .expect("the derived structure describes the units")
}

/// Returns a tile the watcher sees that holds no unit.
///
/// The scan runs over the rings around the camp in a fixed order, so two runs
/// return one answer. It fails when the fixture holds no such tile, because a
/// test that cannot tell an empty place from an unknown one measures nothing.
fn watched_ground_with_no_unit(world: &World, camp: Axial) -> Axial {
    for ring in 1..=4i32 {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(camp.q + column, camp.r + row);
                if world.faction_sees_now(WATCHER, candidate)
                    && truth_units_on(world, candidate) == 0
                {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no watched tile that holds no unit near {camp:?}");
}

/// A faction that marched away reads a memory, and the memory holds no unit.
///
/// **This is the test the reader exists for.** A reader that answered the
/// truth would report the rival that moved in, and the two assertions below
/// would fail.
#[test]
fn a_faction_that_marched_away_reads_a_memory_and_not_the_truth() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let far = ground_near(&world, Axial::new(80, 80), 8);
    let rival_camp = ground_near(&world, Axial::new(10, 10), 8);

    let watcher_unit = a_unit_at(&mut world, home, WATCHER);
    let rival_unit = a_unit_at(&mut world, rival_camp, STRANGER);
    world.step(1).expect("the step runs");

    let seen = world
        .faction_tile(WATCHER, home)
        .expect("the address lies inside the world");
    assert_eq!(
        seen.sighting(),
        Sighting::Seen,
        "the fixture must give the watcher sight of its own ground first"
    );
    let ground_when_seen = seen.ground().expect("a seen tile carries its ground");
    assert_eq!(seen.units(), 1, "the watcher sees its own unit");

    // The watcher marches away. The rival then moves onto the tile the
    // watcher left. Nothing of the watcher watches that tile now.
    world
        .place_soldier(watcher_unit, far)
        .expect("the far ground admits a unit");
    world
        .place_soldier(rival_unit, home)
        .expect("the home ground admits a unit");
    world.step(1).expect("the step runs");

    assert_eq!(
        truth_units_on(&world, home),
        1,
        "the fixture must put a rival on the remembered tile"
    );
    assert!(
        !world.faction_sees_now(WATCHER, home),
        "the fixture must take the watcher out of sight of its old ground"
    );

    let memory = world
        .faction_tile(WATCHER, home)
        .expect("the address lies inside the world");
    assert_eq!(
        memory.sighting(),
        Sighting::Remembered,
        "a faction that saw a place and left reads a memory of it"
    );
    assert_eq!(
        memory.units(),
        0,
        "a remembered place reports no unit, whatever stands there now"
    );
    assert_eq!(
        memory.holder(),
        None,
        "a remembered place reports no holder"
    );
    assert_eq!(
        memory.ground(),
        Some(ground_when_seen),
        "a remembered place answers with the ground of that place"
    );
}

/// A place a faction never saw reads as nothing, and nothing is not zero.
#[test]
fn ground_a_faction_never_saw_reads_as_nothing() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let away = ground_near(&world, Axial::new(80, 80), 8);

    a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");

    assert!(
        !world.faction_has_seen(WATCHER, away),
        "the fixture must leave the far ground unseen"
    );
    let unseen = world
        .faction_tile(WATCHER, away)
        .expect("the address lies inside the world");
    assert_eq!(
        unseen,
        FactionTile::Never,
        "a place never seen reads nothing"
    );
    assert_eq!(
        unseen.ground(),
        None,
        "a place never seen carries no ground at all"
    );

    // A tile the faction watches and that holds no unit answers a zero. The
    // two answers must not be one answer, or a caller cannot tell an empty
    // place from an unknown one.
    let empty = watched_ground_with_no_unit(&world, home);
    let watched = world
        .faction_tile(WATCHER, empty)
        .expect("the address lies inside the world");
    assert_eq!(
        watched.sighting(),
        Sighting::Seen,
        "the fixture must pick a tile the watcher sees"
    );
    assert_eq!(watched.units(), 0, "the fixture must pick an empty tile");
    assert_ne!(
        watched, unseen,
        "an empty place a faction sees is not the same answer as an unknown place"
    );
}

/// A summary states nothing that its own tiles hide.
#[test]
fn a_summary_counts_no_unit_the_faction_cannot_see() {
    let mut world = a_still_world(TWO_SIDES);
    // Both camps sit inside one block of the lattice, and they lie further
    // apart than a unit sees. The cell therefore holds a unit the watcher
    // cannot see, which is the extreme this test needs.
    let home = ground_near(&world, Axial::new(34, 34), 4);
    let rival_camp = ground_near(&world, Axial::new(52, 52), 4);
    assert_eq!(
        block_of(&world, home),
        block_of(&world, rival_camp),
        "the fixture must put the two camps in one cell"
    );
    assert!(
        home.distance(rival_camp) > world.sight_rules().rounded(),
        "the fixture must put the two camps outside each other's sight"
    );

    a_unit_at(&mut world, home, WATCHER);
    a_unit_at(&mut world, rival_camp, STRANGER);
    world.step(1).expect("the step runs");

    let truth = world
        .summary_covering(home)
        .expect("the cell covers the address");
    assert_eq!(truth.units(), 2, "the whole cell holds both units");

    let masked = world
        .faction_summary_covering(WATCHER, home, Admit::SeenNow)
        .expect("the cell covers the address");
    assert_eq!(
        masked.summary().units(),
        1,
        "a summary counts only the units the faction sees"
    );
    assert!(
        masked.withheld() > 0,
        "the fixture must leave part of the cell unseen"
    );
    assert_eq!(
        masked.admitted() + masked.withheld(),
        truth.tiles(),
        "every tile of the cell is admitted or withheld"
    );
    assert_eq!(
        masked.summary().tiles(),
        masked.admitted(),
        "a masked summary counts the tiles it admitted and no others"
    );
    assert!(
        masked.summary().tiles() < truth.tiles(),
        "a partly seen cell reports fewer tiles than the whole cell"
    );
}

/// A summary over memory admits more ground and still reports no unit there.
#[test]
fn a_summary_over_memory_admits_the_ground_and_no_unit() {
    let mut world = a_still_world(TWO_SIDES);
    let first = ground_near(&world, Axial::new(34, 34), 4);
    let second = ground_near(&world, Axial::new(52, 52), 4);
    assert_eq!(
        block_of(&world, first),
        block_of(&world, second),
        "the fixture must put the two camps in one cell"
    );
    assert!(
        first.distance(second) > world.sight_rules().rounded() * 2,
        "the fixture must put the two camps outside each other's sight"
    );

    let unit = a_unit_at(&mut world, first, WATCHER);
    a_unit_at(&mut world, second, STRANGER);
    world.step(1).expect("the step runs");
    let seen_first = world
        .faction_summary_covering(WATCHER, first, Admit::SeenNow)
        .expect("the cell covers the address")
        .admitted();

    world
        .place_soldier(unit, second)
        .expect("the second camp admits a unit");
    world.step(1).expect("the step runs");

    let now = world
        .faction_summary_covering(WATCHER, first, Admit::SeenNow)
        .expect("the cell covers the address");
    let ever = world
        .faction_summary_covering(WATCHER, first, Admit::SeenEver)
        .expect("the cell covers the address");

    assert!(
        ever.admitted() > seen_first,
        "a march adds ground to what the memory admits"
    );
    assert!(
        ever.admitted() > now.admitted(),
        "the memory admits more ground than the present sight"
    );
    assert_eq!(
        ever.summary().units(),
        now.summary().units(),
        "a remembered tile adds no unit to a summary"
    );
    assert_eq!(
        ever.summary().tiles(),
        ever.admitted(),
        "a masked summary counts the tiles it admitted and no others"
    );
}

/// A faction that sees a whole cell reads exactly what the summary level holds.
///
/// The masked reader and the summary rebuild are two ways to add the same
/// tiles. The extreme that proves they agree is a cell the faction sees
/// whole, because the mask then withholds nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn a_cell_a_faction_sees_whole_reads_as_the_summary_level_holds_it() {
    // The world is one block of the lattice. The rules widen the sight to
    // the ceiling and let nothing block it, so one camp watches every tile.
    let mut world = a_still_world(ONE_BLOCK);
    world.set_sight_rules(SightRules::new(16, 1, 16, 0));

    let grid = world.grid();
    let mut camp = None;
    for row in 0..grid.height() as i32 {
        for column in 0..grid.width() as i32 {
            let address = Axial::new(column, row);
            if world.admits_a_unit(address) {
                a_unit_at(&mut world, address, WATCHER);
                camp.get_or_insert(address);
            }
        }
    }
    let camp = camp.expect("the fixture must place at least one unit");
    world.step(1).expect("the step runs");

    assert_eq!(
        world.faction_seen_now(WATCHER),
        i64::from(grid.tile_count()),
        "the fixture must give the watcher sight of every tile of the world"
    );

    let masked = world
        .faction_summary_covering(WATCHER, camp, Admit::SeenNow)
        .expect("the cell covers the address");
    let truth = world
        .summary_covering(camp)
        .expect("the cell covers the address");
    assert_eq!(
        masked.withheld(),
        0,
        "a cell the faction sees whole withholds no tile"
    );
    assert_eq!(
        masked.summary(),
        truth,
        "a cell the faction sees whole reads as the summary level holds it"
    );
}

/// A faction reads a rival only while its own units watch the ground.
#[test]
fn a_faction_reads_a_rival_only_while_it_watches_the_ground() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let away = ground_near(&world, Axial::new(80, 80), 8);

    let own = a_unit_at(&mut world, home, WATCHER);
    let rival = a_unit_at(&mut world, away, STRANGER);
    world.step(1).expect("the step runs");

    let seen = world.units_seen_by(WATCHER);
    assert_eq!(seen.len(), 1, "the watcher reads its own unit alone");
    assert_eq!(
        seen[0].unit, own,
        "the one row names the watcher's own unit"
    );
    assert_eq!(
        seen[0].faction, WATCHER,
        "the row names the faction it belongs to"
    );

    // The rival marches into the ground the watcher watches.
    let beside = watched_ground_with_no_unit(&world, home);
    world
        .place_soldier(rival, beside)
        .expect("the watched ground admits a unit");
    world.step(1).expect("the step runs");

    let seen = world.units_seen_by(WATCHER);
    assert_eq!(seen.len(), 2, "the watcher reads the rival that walked in");
    assert!(
        seen.iter()
            .any(|row| row.unit == rival && row.faction == STRANGER),
        "the rival row names the faction of the rival"
    );

    // The rival marches back out of the ground the watcher watches.
    world
        .place_soldier(rival, away)
        .expect("the distant ground admits a unit");
    world.step(1).expect("the step runs");

    let seen = world.units_seen_by(WATCHER);
    assert_eq!(
        seen.len(),
        1,
        "a unit that walked out of sight leaves the answer"
    );
}
