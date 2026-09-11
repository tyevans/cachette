//! Part-built wonder work is fragile.
//!
//! A trained policy won about a third of the games of three factions by
//! building one wonder and doing nothing else, because nothing a rival did
//! undid the work. Wonder work now loses a stated amount on each tick that no
//! builder adds to it, and it returns to nothing when its ground changes
//! holder.[^1]
//!
//! **Each fixture here is built for the case it tests.** The city stands on an
//! island, whose every neighbour refuses a unit, so an invader stands where the
//! fixture puts it. The work stands on the nearest open tile that the island
//! city holds. A raze therefore cannot remove it by the older rule, which burns
//! the upgrade on the tile of the site and no other.[^2]
//!
//! The tests drive the step and the public verbs, because the step is what must
//! invoke the rule.[^3]
//!
//! # References
//!
//! [^1]: ADR-0206, a part-built wonder decays when nobody works it. `docs/adrs/draft/adr-0206-a-part-built-wonder-decays-when-nobody-works-it.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::holding::{Holder, ReachRules};
use cachette_core::site::SiegeRules;
use cachette_core::upgrade::{UpgradeCategory, BUILD_RATE, WONDER_DECAY};
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the worlds below.
///
/// The extent is wide enough that the generator puts water in it, because
/// every fixture here needs a tile whose every neighbour refuses a unit.
const EXTENT: u32 = 192;

/// The work these fixtures ask a wonder for.
///
/// The builders of a fixture add far less than this, so the work stays part
/// built for as long as a test needs it. The test reads no balance figure, so
/// this is a fixture choice.
const PART_BUILT_WORK: u32 = 2400;

/// The work the finished wonder of one fixture asks for.
///
/// A full tile of builders passes it in the first few ticks of the build.
const FINISHED_WORK: u32 = 16;

/// The ticks a fixture builds for before a test starts.
const BUILD_TICKS: u32 = 10;

/// The siege work that one resident costs a besieger, in these fixtures.
const SIEGE_WORK: i64 = 4;

/// The times over the capture work that a raze costs, in these fixtures.
const SIEGE_MULTIPLE: i64 = 4;

/// The most ticks any fixture here presses a siege for.
///
/// A site of one resident costs four work to take and sixteen to burn, and a
/// besieger does one work a tick. A test that reaches this bound has found a
/// siege that never ends.
const PRESS_BOUND: u32 = 200;

/// The reach that puts every city of a faction within reach of every tile.
///
/// A taker then keeps the city it captures rather than burning it, and the
/// nearest city decides the holder of the work.
const REACH_TOGETHER: u32 = EXTENT;

/// The ticks a test leaves work alone to show that nothing takes from it.
const IDLE_TICKS: u32 = 20;

/// Returns every address of a world, in tile index order.
fn addresses(field: &World) -> Vec<Axial> {
    let grid = field.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns an island: an open tile whose every neighbour refuses a unit.
fn island(field: &World) -> Option<Axial> {
    addresses(field).into_iter().find(|address| {
        field.admits_a_unit(*address)
            && field
                .grid()
                .neighbours(*address)
                .iter()
                .all(|side| side.is_none_or(|next| !field.admits_a_unit(next)))
    })
}

/// Returns the work on one tile, or `None` when the tile carries no entry.
fn work_at(field: &World, address: Axial) -> Option<i64> {
    field.upgrade_at(address).map(|site| site.progress.0)
}

/// Returns the holder of one tile.
fn holder_of(field: &World, address: Axial) -> Holder {
    field
        .tile_holder(address)
        .expect("the tile is inside the world")
}

/// Steps the world until the condition holds, and returns the steps it ran.
///
/// The last step it runs is the step that made the condition true.
fn step_until(field: &mut World, bound: u32, mut done: impl FnMut(&World) -> bool) -> u32 {
    let mut ran = 0;
    while ran < bound && !done(field) {
        field.step(1).expect("the step must run");
        ran += 1;
    }
    ran
}

/// What a fixture holds: an island city of faction zero, a rival city far
/// away, and a full tile of builders that has built on the ground beside the
/// island for a few ticks.
struct Fixture {
    field: World,
    seat: Axial,
    site: Entity,
    ground: Axial,
    builders: Vec<Entity>,
}

/// Builds the fixture, or returns `None` when the seed cannot hold it.
///
/// The fixture holds the builders on their tile by the build order, and it
/// keeps the choice pass away from the run, so every builder adds work on
/// every tick of the build.
///
/// **The caller drives both factions.** The built-in controller orders every
/// idle unit of its faction onto work on a schedule of its own. A builder a
/// test stopped would then build again, and the test would read the
/// controller and not the rule.
fn fixture(seed: u64, category: UpgradeCategory, wonder_work: u32) -> Option<Fixture> {
    let mut field = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .ok()?;
    field.set_win_readers_enabled(false);
    for faction in [FactionId(0), FactionId(1)] {
        assert!(field.set_externally_controlled(faction, true));
    }
    field
        .set_choice_schedule(cachette_core::choose::PERIOD_LOG2_CEILING)
        .ok()?;
    field.set_reach_rules(ReachRules::new(REACH_TOGETHER, 1, REACH_TOGETHER));
    field.set_siege_rules(SiegeRules::new(SIEGE_WORK, SIEGE_MULTIPLE));
    assert!(field.set_wonder_work(wonder_work));
    let seat = island(&field)?;
    let ground = addresses(&field)
        .into_iter()
        .filter(|address| *address != seat && field.admits_a_unit(*address))
        .min_by_key(|address| address.distance(seat))?;
    let far = |address: &Axial, from: &[Axial]| {
        field.admits_a_unit(*address) && from.iter().all(|place| address.distance(*place) > 8)
    };
    let rival = addresses(&field)
        .into_iter()
        .find(|address| far(address, &[seat, ground]))?;
    let away = addresses(&field)
        .into_iter()
        .find(|address| far(address, &[seat, ground, rival]))?;
    field.found_group_at(rival, 1, FactionId(1)).ok()?;
    let founding = field.found_group_at(seat, 1, FactionId(0)).ok()?;
    let site = founding.settlement();
    let resident = *founding.people().first()?;
    field.place_soldier(resident, away).ok()?;
    field.step(1).ok()?;
    if holder_of(&field, ground).faction() != Some(FactionId(0)) {
        return None;
    }
    let room = field.tile_capacity(ground)?;
    let mut builders = Vec::new();
    for _ in 0..room {
        let unit = field.spawn_soldier(ground, FactionId(0)).ok()?;
        field.order_build(unit, category).ok()?;
        builders.push(unit);
    }
    for _ in 0..BUILD_TICKS {
        field.step(1).ok()?;
    }
    field.upgrade_at(ground)?;
    Some(Fixture {
        field,
        seat,
        site,
        ground,
        builders,
    })
}

/// Returns the first seed that builds the fixture, or fails the test.
fn any_seed(category: UpgradeCategory, wonder_work: u32) -> u64 {
    for seed in 0..60u64 {
        if fixture(seed, category, wonder_work).is_some() {
            return seed;
        }
    }
    panic!("no seed below 60 gives an island with ground beside it to build on");
}

/// Builds a fixture of part-built wonder work, and returns it with its seed.
///
/// The work never decays in this fixture, so a test that changes the holder
/// reads the reset alone. The builders are gone, so no lease and no garrison
/// of theirs holds the ground.
fn idle_wonder() -> (u64, Fixture) {
    let seed = any_seed(UpgradeCategory::WONDER, PART_BUILT_WORK);
    let mut built =
        fixture(seed, UpgradeCategory::WONDER, PART_BUILT_WORK).expect("the seed builds it");
    built.field.set_wonder_decay(0);
    for unit in &built.builders {
        assert!(built.field.despawn_soldier(*unit));
    }
    (seed, built)
}

/// Returns the work that the idle wonder of one seed holds after some ticks
/// in which nobody besieges, razes or moves the ground.
///
/// This is the control of the three reset tests. It says what the work does
/// without the act, so each test reads the act and not the tick.
fn control_work(ticks: u32) -> Option<i64> {
    let (_, mut control) = idle_wonder();
    for _ in 0..ticks {
        control.field.step(1).expect("the step must run");
    }
    assert_eq!(
        holder_of(&control.field, control.ground).faction(),
        Some(FactionId(0)),
        "the control lost its ground with nobody acting on it"
    );
    work_at(&control.field, control.ground)
}

#[test]
fn unattended_wonder_work_loses_the_decay_on_every_tick_until_it_is_gone() {
    let seed = any_seed(UpgradeCategory::WONDER, PART_BUILT_WORK);
    let Fixture {
        mut field,
        ground,
        builders,
        ..
    } = fixture(seed, UpgradeCategory::WONDER, PART_BUILT_WORK).expect("the seed builds it");
    assert_eq!(
        field.wonder_decay(),
        WONDER_DECAY,
        "a world starts with the decay the register states"
    );
    let decay = i64::from(field.wonder_decay());
    assert!(decay > 0, "the default decay takes nothing");
    let built = work_at(&field, ground).expect("the builders made an entry");
    assert!(
        built > 2 * decay,
        "the fixture built too little to watch the work fall"
    );
    for unit in &builders {
        assert!(field.stop_build(*unit));
    }
    let mut expected = built;
    let mut ticks = 0u32;
    while expected > 0 {
        field.step(1).expect("the step must run");
        ticks += 1;
        expected = (expected - decay).max(0);
        if expected > 0 {
            assert_eq!(
                work_at(&field, ground),
                Some(expected),
                "the work did not fall by the decay on tick {ticks}"
            );
        }
    }
    assert_eq!(
        field.upgrade_at(ground),
        None,
        "wonder work at no level that reached nothing still stands"
    );
    assert_eq!(
        holder_of(&field, ground).faction(),
        Some(FactionId(0)),
        "the ground changed hands, so the test read a reset and not the decay"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_builder_on_the_work_stops_the_decay() {
    let seed = any_seed(UpgradeCategory::WONDER, PART_BUILT_WORK);
    let Fixture {
        mut field,
        ground,
        builders,
        ..
    } = fixture(seed, UpgradeCategory::WONDER, PART_BUILT_WORK).expect("the seed builds it");
    assert!(field.wonder_decay() > 0, "the default decay takes nothing");
    for unit in &builders[1..] {
        assert!(field.stop_build(*unit));
    }
    let mut expected = work_at(&field, ground).expect("the builders made an entry");
    for tick in 1..=IDLE_TICKS {
        field.step(1).expect("the step must run");
        expected += BUILD_RATE;
        assert_eq!(
            work_at(&field, ground),
            Some(expected),
            "one builder on the work did not add exactly its rate on tick {tick}"
        );
    }
}

#[test]
fn a_capture_resets_wonder_work_on_the_tick_the_ground_changes_hands() {
    let (_, built) = idle_wonder();
    let Fixture {
        mut field,
        seat,
        site,
        ground,
        ..
    } = built;
    let before = work_at(&field, ground).expect("the builders made an entry");
    assert!(before > 0);
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    let ran = step_until(&mut field, PRESS_BOUND, |field| {
        field.settlement_faction(site) == Some(FactionId(1))
    });
    assert!(ran < PRESS_BOUND, "the siege never took the site");
    assert_eq!(
        holder_of(&field, ground).faction(),
        Some(FactionId(1)),
        "the ground did not follow the city"
    );
    assert_eq!(
        field.upgrade_at(ground),
        None,
        "the taker received the wonder work of the faction that lost the city"
    );
    assert_eq!(
        control_work(ran),
        Some(before),
        "the work moved with no capture, so the test did not read the capture"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_raze_resets_wonder_work_beyond_the_tile_of_the_site() {
    let (_, built) = idle_wonder();
    let Fixture {
        mut field,
        seat,
        site,
        ground,
        ..
    } = built;
    assert!(work_at(&field, ground).is_some_and(|work| work > 0));
    field
        .spawn_soldier(seat, FactionId(1))
        .expect("the island admits a unit");
    field.step(1).expect("the step must run");
    field
        .order_raze(site, FactionId(1))
        .expect("a siege of the razer stands against the site");
    let ran = 1 + step_until(&mut field, PRESS_BOUND, |field| {
        field.settlement_on(seat).is_none()
    });
    assert!(ran < PRESS_BOUND, "the ordered raze never burned the site");
    assert_ne!(
        holder_of(&field, ground).faction(),
        Some(FactionId(0)),
        "the razed city kept the ground beside it"
    );
    assert_eq!(
        field.upgrade_at(ground),
        None,
        "wonder work outlived the raze of the city that held its ground"
    );
    assert!(
        control_work(ran).is_some_and(|work| work > 0),
        "the work went with no raze, so the test did not read the raze"
    );
    assert!(field.check_invariants());
}

#[test]
fn ground_that_falls_to_nobody_resets_wonder_work() {
    let (_, built) = idle_wonder();
    let Fixture {
        mut field, ground, ..
    } = built;
    assert!(work_at(&field, ground).is_some_and(|work| work > 0));
    field.set_reach_rules(ReachRules::new(0, 1, 0));
    field.step(1).expect("the step must run");
    assert!(
        holder_of(&field, ground).is_nobody(),
        "the ground is still held with no city in reach"
    );
    assert_eq!(
        field.upgrade_at(ground),
        None,
        "wonder work outlived the loss of its ground"
    );
    assert!(
        control_work(1).is_some_and(|work| work > 0),
        "the work went with the ground still held"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_finished_wonder_stands_whatever_happens_to_its_ground() {
    let seed = any_seed(UpgradeCategory::WONDER, FINISHED_WORK);
    let Fixture {
        mut field,
        ground,
        builders,
        ..
    } = fixture(seed, UpgradeCategory::WONDER, FINISHED_WORK).expect("the seed builds it");
    assert_eq!(
        field.finished_upgrade(ground),
        Some(UpgradeCategory::WONDER),
        "the fixture never finished the wonder"
    );
    assert!(field.wonder_decay() > 0, "the default decay takes nothing");
    for unit in &builders {
        assert!(field.despawn_soldier(*unit));
    }
    for _ in 0..IDLE_TICKS {
        field.step(1).expect("the step must run");
    }
    assert_eq!(
        field.finished_upgrade(ground),
        Some(UpgradeCategory::WONDER),
        "a finished wonder that nobody works was taken away"
    );
    field.set_reach_rules(ReachRules::new(0, 1, 0));
    field.step(1).expect("the step must run");
    assert!(
        holder_of(&field, ground).is_nobody(),
        "the ground is still held with no city in reach"
    );
    assert_eq!(
        field.finished_upgrade(ground),
        Some(UpgradeCategory::WONDER),
        "a finished wonder did not stand when its ground changed holder"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_part_built_store_keeps_its_work_alone_and_across_a_change_of_holder() {
    let seed = any_seed(UpgradeCategory::STORE, PART_BUILT_WORK);
    let Fixture {
        mut field,
        ground,
        builders,
        ..
    } = fixture(seed, UpgradeCategory::STORE, PART_BUILT_WORK).expect("the seed builds it");
    assert!(field.wonder_decay() > 0, "the default decay takes nothing");
    let built = work_at(&field, ground).expect("the builders made an entry");
    assert!(built > 0);
    assert_eq!(
        field.finished_upgrade(ground),
        None,
        "the fixture finished the store, so it tests no part-built work"
    );
    for unit in &builders {
        assert!(field.despawn_soldier(*unit));
    }
    for _ in 0..IDLE_TICKS {
        field.step(1).expect("the step must run");
    }
    assert_eq!(
        work_at(&field, ground),
        Some(built),
        "part-built work toward a row with no claim lost work while nobody worked it"
    );
    field.set_reach_rules(ReachRules::new(0, 1, 0));
    field.step(1).expect("the step must run");
    assert!(
        holder_of(&field, ground).is_nobody(),
        "the ground is still held with no city in reach"
    );
    assert_eq!(
        work_at(&field, ground),
        Some(built),
        "part-built work toward a row with no claim did not change hands with the ground"
    );
    assert!(field.check_invariants());
}

#[test]
fn the_decay_enters_the_state_hash() {
    let mut first = World::new(WorldConfig::DEFAULT).expect("the default world builds");
    let mut second = World::new(WorldConfig::DEFAULT).expect("the default world builds");
    assert_eq!(first.state_hash().finish(), second.state_hash().finish());
    second.set_wonder_decay(WONDER_DECAY + 1);
    assert_ne!(
        first.state_hash().finish(),
        second.state_hash().finish(),
        "two worlds that decay wonder work at two rates hash the same"
    );
    first.set_wonder_decay(WONDER_DECAY + 1);
    assert_eq!(first.state_hash().finish(), second.state_hash().finish());
}
