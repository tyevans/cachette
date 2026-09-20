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
use cachette_core::trade::Consideration;
use cachette_core::upgrade::{UpgradeCategory, UpgradeSite, BUILD_RATE, WONDER_DECAY};
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

/// What a fixture holds: an island city, a rival city far away, and a full
/// tile of builders that has built on the ground beside the island for a few
/// ticks.
struct Fixture {
    field: World,
    seat: Axial,
    site: Entity,
    ground: Axial,
    builders: Vec<Entity>,
}

/// The places of a fixture world.
///
/// The seat is an island. The ground is the nearest open tile to it. The
/// rival city and the resident of the seat stand far from both, and far from
/// each other.
struct Places {
    seat: Axial,
    ground: Axial,
    rival: Axial,
    away: Axial,
}

/// Builds a world of these fixtures with nothing founded in it, or returns
/// `None` when the seed cannot hold one.
///
/// The fixture keeps the choice pass away from the run, so a unit stays
/// where the fixture puts it. Every city reaches every tile, so the nearest
/// city decides the holder of a tile.
///
/// **The caller drives both factions.** The built-in controller orders every
/// idle unit of its faction onto work on a schedule of its own. A builder a
/// test stopped would then build again, and the test would read the
/// controller and not the rule.
fn open_world(seed: u64, wonder_work: u32) -> Option<World> {
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
    Some(field)
}

/// Returns the places of a fixture world, or `None` when the world has no
/// island with open ground beside it.
fn places(field: &World) -> Option<Places> {
    let seat = island(field)?;
    let ground = addresses(field)
        .into_iter()
        .filter(|address| *address != seat && field.admits_a_unit(*address))
        .min_by_key(|address| address.distance(seat))?;
    let far = |address: &Axial, from: &[Axial]| {
        field.admits_a_unit(*address) && from.iter().all(|place| address.distance(*place) > 8)
    };
    let rival = addresses(field)
        .into_iter()
        .find(|address| far(address, &[seat, ground]))?;
    let away = addresses(field)
        .into_iter()
        .find(|address| far(address, &[seat, ground, rival]))?;
    Some(Places {
        seat,
        ground,
        rival,
        away,
    })
}

/// Founds the rival city and then the island city, steps once, and returns
/// the world, its places and the island city.
///
/// Returns `None` when the ground beside the island does not go to the owner
/// of the island.
fn seated(
    seed: u64,
    wonder_work: u32,
    owner: FactionId,
    rival: FactionId,
) -> Option<(World, Places, Entity)> {
    let mut field = open_world(seed, wonder_work)?;
    let at = places(&field)?;
    field.found_group_at(at.rival, 1, rival).ok()?;
    let founding = field.found_group_at(at.seat, 1, owner).ok()?;
    let site = founding.settlement();
    let resident = *founding.people().first()?;
    field.place_soldier(resident, at.away).ok()?;
    field.step(1).ok()?;
    if holder_of(&field, at.ground).faction() != Some(owner) {
        return None;
    }
    Some((field, at, site))
}

/// Fills one tile with builders of one faction, each with an order to build
/// one category there.
///
/// The build order holds each builder on its tile, so every builder adds
/// work on every tick of the build.
fn crew(
    field: &mut World,
    ground: Axial,
    owner: FactionId,
    category: UpgradeCategory,
) -> Option<Vec<Entity>> {
    let room = field.tile_capacity(ground)?;
    let mut builders = Vec::new();
    for _ in 0..room {
        let unit = field.spawn_soldier(ground, owner).ok()?;
        field.order_build(unit, category).ok()?;
        builders.push(unit);
    }
    Some(builders)
}

/// Builds the fixture with one faction on the island and the other far away,
/// or returns `None` when the seed cannot hold it.
fn fixture_of(
    seed: u64,
    category: UpgradeCategory,
    wonder_work: u32,
    owner: FactionId,
    rival: FactionId,
) -> Option<Fixture> {
    let (mut field, at, site) = seated(seed, wonder_work, owner, rival)?;
    let builders = crew(&mut field, at.ground, owner, category)?;
    for _ in 0..BUILD_TICKS {
        field.step(1).ok()?;
    }
    field.upgrade_at(at.ground)?;
    Some(Fixture {
        field,
        seat: at.seat,
        site,
        ground: at.ground,
        builders,
    })
}

/// Builds the fixture with faction zero on the island, or returns `None`
/// when the seed cannot hold it.
fn fixture(seed: u64, category: UpgradeCategory, wonder_work: u32) -> Option<Fixture> {
    fixture_of(seed, category, wonder_work, FactionId(0), FactionId(1))
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
    for _ in 0..2 {
        field
            .spawn_soldier(seat, FactionId(1))
            .expect("the island admits a unit");
    }
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

/// The ticks a land contract in these fixtures has before its deadline.
///
/// Every contract here settles on the first step after it binds, so this only
/// has to outlast the fixture.
const GIFT_TERM: u32 = 1000;

/// Returns an open tile that one faction holds, that carries no city, and
/// that is none of the tiles named.
fn held_tile(field: &World, faction: FactionId, except: &[Axial]) -> Option<Axial> {
    addresses(field).into_iter().find(|address| {
        field.admits_a_unit(*address)
            && !except.contains(address)
            && field.settlement_on(*address).is_none()
            && holder_of(field, *address).faction() == Some(faction)
    })
}

/// Binds a contract by which one faction owes another one tile of ground,
/// for one relation step.
///
/// No unit carries either side, so the tile changes hands at the settlement
/// of contracts on the next step, and that settlement runs after the spread.
/// Each party speaks only while one of its units stands on the ground of the
/// other, so this spawns one unit for each party, away from the named tiles.
fn bind_a_gift(
    field: &mut World,
    giver: FactionId,
    taker: FactionId,
    gift: Axial,
    except: &[Axial],
) {
    let tile = field
        .grid()
        .index_of(gift)
        .expect("the gift is inside the world");
    let there = held_tile(field, taker, except).expect("the taker holds open ground");
    field
        .spawn_soldier(there, giver)
        .expect("the ground of the taker admits a unit");
    field
        .offer_consideration(
            giver,
            taker,
            Consideration::land(vec![tile]),
            Consideration::relation(0, 1),
            GIFT_TERM,
        )
        .expect("the giver holds the gift and stands on the ground of the taker");
    let here = held_tile(field, giver, except).expect("the giver holds open ground");
    field
        .spawn_soldier(here, taker)
        .expect("the ground of the giver admits a unit");
    field
        .accept_trade(taker, giver)
        .expect("the taker stands on the ground of the giver");
}

/// Builds the fixture of a finished wonder with one faction on the island,
/// and takes its builders away.
fn finished_and_left(seed: u64, owner: FactionId, rival: FactionId) -> Fixture {
    let mut built = fixture_of(seed, UpgradeCategory::WONDER, FINISHED_WORK, owner, rival)
        .expect("the seed builds the fixture with either faction on the island");
    assert_eq!(
        built.field.finished_upgrade(built.ground),
        Some(UpgradeCategory::WONDER),
        "the fixture never finished the wonder"
    );
    for unit in &built.builders {
        assert!(built.field.despawn_soldier(*unit));
    }
    built
}

/// Two worlds differ only in which faction built a standing upgrade, and the
/// ground under it ends with one faction. The two upgrades must be one value.
///
/// **This is the reviewer test of the record that an upgrade changes hands
/// with the ground.** It fails when an upgrade carries a faction of its
/// own.[^1] A holder stored on the entry at the build differs between the two
/// worlds, and nothing rewrites it on a standing level, so the two entries
/// differ.
///
/// The ground changes hands by a gift, and the gift runs after the spread, so
/// the island city does not take it back on the tick it is read.
///
/// **The pattern below names every field of the entry and holds no rest
/// pattern.** A field added to the entry therefore stops this file from
/// compiling. The author of that field must then answer the record here, in
/// the pattern, before anything else runs.
///
/// # References
///
/// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
#[test]
fn an_upgrade_carries_no_faction_of_its_own() {
    let seed = any_seed(UpgradeCategory::WONDER, FINISHED_WORK);
    let (zero, one) = (FactionId(0), FactionId(1));
    let mut kept = finished_and_left(seed, zero, one);
    let mut given = finished_and_left(seed, one, zero);
    assert_eq!(
        kept.ground, given.ground,
        "the two worlds put the ground in two places"
    );
    let ground = given.ground;
    bind_a_gift(&mut given.field, one, zero, ground, &[given.seat, ground]);
    kept.field.step(1).expect("the step must run");
    given.field.step(1).expect("the step must run");
    for field in [&kept.field, &given.field] {
        assert_eq!(
            holder_of(field, ground).faction(),
            Some(zero),
            "the ground does not end with one faction in both worlds"
        );
    }
    let site = given
        .field
        .upgrade_at(ground)
        .expect("the gift took the upgrade off the ground");
    assert_eq!(
        kept.field.upgrade_at(ground),
        Some(site),
        "two worlds that differ only in which faction built a standing upgrade hold two different upgrades"
    );
    let UpgradeSite {
        tile,
        category,
        level,
        progress,
        condition,
    } = site;
    assert_eq!(given.field.grid().index_of(ground), Some(tile));
    assert_eq!(category, UpgradeCategory::WONDER);
    assert!(level > 0, "the wonder does not stand");
    assert_eq!(
        progress.0, 0,
        "a wonder at the top of its category holds work"
    );
    assert!(condition.0 > 0, "a standing wonder holds no condition");
    assert!(kept.field.check_invariants());
    assert!(given.field.check_invariants());
}

/// Runs a world in which a rival owes the island faction the ground under its
/// wonder work, and returns the holder of that ground, the work on it after
/// the gift settles, and the work before.
///
/// The rival holds the ground first, because the island city does not stand
/// yet, and it binds itself to give the ground away. The island city then
/// takes the ground by the spread, and its builders put work on it.
///
/// When the ground is lost, the spread of the last step gives it to nobody,
/// and the gift gives it back to the island faction later in the same step.
/// The two ends of that step then name one holder.
fn gift_back(seed: u64, lose_the_ground: bool) -> (Holder, Option<i64>, i64) {
    let (island, rival) = (FactionId(0), FactionId(1));
    let mut field = open_world(seed, PART_BUILT_WORK).expect("the seed opens a world");
    let at = places(&field).expect("the seed has an island");
    field
        .found_group_at(at.rival, 1, rival)
        .expect("the rival city founds");
    field.step(1).expect("the step must run");
    assert_eq!(
        holder_of(&field, at.ground).faction(),
        Some(rival),
        "the rival does not hold the ground before the island city stands"
    );
    let tile = field
        .grid()
        .index_of(at.ground)
        .expect("the ground is inside the world");
    let visitor = field
        .spawn_soldier(at.ground, island)
        .expect("the ground admits a unit");
    field
        .offer_consideration(
            island,
            rival,
            Consideration::relation(0, 1),
            Consideration::land(vec![tile]),
            GIFT_TERM,
        )
        .expect("the rival holds the ground, and a unit of the island faction stands on it");
    assert!(field.despawn_soldier(visitor));
    let founding = field
        .found_group_at(at.seat, 1, island)
        .expect("the island admits a city");
    let resident = *founding.people().first().expect("the city has a resident");
    field
        .place_soldier(resident, at.away)
        .expect("the far tile admits a unit");
    field.step(1).expect("the step must run");
    assert_eq!(
        holder_of(&field, at.ground).faction(),
        Some(island),
        "the island city did not take the ground beside it"
    );
    let builders = crew(&mut field, at.ground, island, UpgradeCategory::WONDER)
        .expect("the ground admits builders");
    for _ in 0..BUILD_TICKS {
        field.step(1).expect("the step must run");
    }
    field.set_wonder_decay(0);
    for unit in &builders {
        assert!(field.despawn_soldier(*unit));
    }
    let built = work_at(&field, at.ground).expect("the builders made an entry");
    assert!(built > 0);
    let there = held_tile(&field, island, &[at.seat, at.ground])
        .expect("the island faction holds open ground");
    field
        .spawn_soldier(there, rival)
        .expect("the ground of the island faction admits a unit");
    field
        .accept_trade(rival, island)
        .expect("the rival stands on the ground of the island faction");
    if lose_the_ground {
        field.set_reach_rules(ReachRules::new(0, 1, 0));
    }
    field.step(1).expect("the step must run");
    assert!(field.check_invariants());
    (
        holder_of(&field, at.ground),
        work_at(&field, at.ground),
        built,
    )
}

/// Ground that changes holder and changes back inside one step resets its
/// wonder work.
///
/// **The rule reads each change of the holder column, and not only the two
/// ends of the step.**[^1] The spread gives the ground to nobody, and a gift
/// of land gives it back to the same faction before the wonder work pass
/// runs. A rule that compared the holder at the start of the step with the
/// holder at the pass would see one holder and keep the work.
///
/// The control binds the same gift and keeps the ground. The gift then names
/// the holder the ground already has, so the ground never moves, and the work
/// stays. That shows that the gift alone does not reset the work.
///
/// # References
///
/// [^1]: ADR-0206, a part-built wonder decays when nobody works it, decision D2. `docs/adrs/draft/adr-0206-a-part-built-wonder-decays-when-nobody-works-it.md`
#[test]
fn ground_that_changes_holder_and_back_in_one_step_resets_wonder_work() {
    let seed = any_seed(UpgradeCategory::WONDER, PART_BUILT_WORK);
    let (holder, work, built) = gift_back(seed, false);
    assert_eq!(
        holder.faction(),
        Some(FactionId(0)),
        "the gift moved the ground away from the island faction"
    );
    assert_eq!(
        work,
        Some(built),
        "a gift of ground that the faction already holds changed its wonder work"
    );
    let (holder, work, _) = gift_back(seed, true);
    assert_eq!(
        holder.faction(),
        Some(FactionId(0)),
        "the gift did not give the ground back after the spread took it"
    );
    assert_eq!(
        work, None,
        "ground that changed holder and changed back in one step kept its wonder work"
    );
}

/// Runs the first tick of a wonder build, and returns the holder of the
/// ground and the work on it after that tick.
///
/// When the ground is lost, the spread of that tick gives it to nobody after
/// the build has made the first work.
fn first_tick_of_a_build(seed: u64, lose_the_ground: bool) -> (Holder, Option<i64>) {
    let (mut field, at, _) = seated(seed, PART_BUILT_WORK, FactionId(0), FactionId(1))
        .expect("the seed builds the fixture");
    field.set_wonder_decay(0);
    assert_eq!(field.upgrade_at(at.ground), None);
    crew(&mut field, at.ground, FactionId(0), UpgradeCategory::WONDER)
        .expect("the ground admits builders");
    if lose_the_ground {
        field.set_reach_rules(ReachRules::new(0, 1, 0));
    }
    field.step(1).expect("the step must run");
    assert!(field.check_invariants());
    (holder_of(&field, at.ground), work_at(&field, at.ground))
}

/// Work that the build starts on the tick its ground changes holder returns to
/// nothing on that tick.
///
/// **The watch on the ground starts after the build.** The build makes the
/// entry, and the spread then moves the ground in the same step. A watch that
/// started before the build would not know the entry, and the new holder
/// would inherit the work.[^1]
///
/// The control runs the same tick with the ground kept, and the work stands.
///
/// # References
///
/// [^1]: ADR-0206, a part-built wonder decays when nobody works it, decision D4. `docs/adrs/draft/adr-0206-a-part-built-wonder-decays-when-nobody-works-it.md`
#[test]
fn work_the_build_starts_on_the_tick_its_ground_changes_holder_resets() {
    let seed = any_seed(UpgradeCategory::WONDER, PART_BUILT_WORK);
    let (holder, work) = first_tick_of_a_build(seed, false);
    assert_eq!(holder.faction(), Some(FactionId(0)));
    assert!(
        work.is_some_and(|work| work > 0),
        "the first tick of the build made no work, so the test reads nothing"
    );
    let (holder, work) = first_tick_of_a_build(seed, true);
    assert!(
        holder.is_nobody(),
        "the ground is still held with no city in reach"
    );
    assert_eq!(
        work, None,
        "work the build started on the tick its ground fell to nobody outlived that tick"
    );
}
