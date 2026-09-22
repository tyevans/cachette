//! A rival that nears a win becomes the target of the built-in controller.
//!
//! **A player could build a wonder and nobody disrupted it.** The controller
//! named two targets: the rival with the most held ground, and the weakest
//! faction it overmatched. A faction that put every unit on one wonder was
//! neither, so no controller moved against it.[^1]
//!
//! The controller now reads the reading of every rival on every win path. A
//! rival whose reading reaches the win threat share is the target. The
//! relation move goes to it on every tick, and a march on it aims at the city
//! whose ground holds its wonder. The share is a balance value.[^2]
//!
//! Every test drives the world step and reads the controller log or the
//! campaign register afterwards.[^3] **Each fixture gives the old rule a
//! different answer from the new one.** The rival that nears a win is never
//! the rival by held ground and never the prey, and the city that holds the
//! wonder is never the nearest city of its faction.[^4]
//!
//! # References
//!
//! [^1]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
//! [^2]: Balance register, the win threat share. `docs/reference/balance.md`
//! [^3]: Testing Rules, section 5. `.agents/rules/testing.md`
//! [^4]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::controller::{WEIGHT_HIGH, WEIGHT_LOW, WIN_THREAT_SHARE_DEFAULT};
use cachette_core::faction_observation::observation_schema;
use cachette_core::unit_type::LEADER;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, Verb, WinPath, World, WorldConfig};

const THREADS: usize = 2;

/// The faction the built-in controller plays in every fixture below.
const WATCHER: FactionId = FactionId(0);

/// The faction that nears a win.
const LEADING: FactionId = FactionId(1);

/// The faction that holds the most ground, or that takes a seat.
const OTHER: FactionId = FactionId(2);

/// The exponent that keeps a unit still, so a fixture holds its shape.
const KEEP_STILL: u32 = 12;

/// A tick limit no fixture here reaches.
const FAR_LIMIT: u64 = 100_000;

/// The people each seat founds with.
const GROUP: u32 = 8;

/// The work this file asks a wonder for.
///
/// **The fixture states its own requirement.** A small requirement lets a few
/// builders cross the share in a few dozen steps.
const FIXTURE_WONDER_WORK: u32 = 240;

/// The builders that work the wonder.
const BUILDERS: u32 = 3;

/// The most steps a fixture lets the builders work before it gives up.
const BUILD_BOUND: u32 = 400;

/// The ticks in which a test reads every relation move of the watcher.
const WINDOW: u32 = 8;

/// Builds a world in which no unit takes a movement intent, no game ends on
/// the tick limit, and every faction but the watcher is under external
/// control.
///
/// **The watcher starts under external control as well.** It plans nothing
/// while the fixture builds, so no march it raised before the test reads it
/// can hold its campaign register.
fn a_still_world(factions: u16, seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: 96,
        height: 96,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_tick_limit(FAR_LIMIT);
    assert!(world.set_wonder_work(FIXTURE_WONDER_WORK));
    for faction in 0..factions {
        assert!(world.set_externally_controlled(FactionId(faction), true));
    }
    world
}

/// Returns the addresses that admit a unit, in rings about the one asked for.
fn ground_near(world: &World, wanted: Axial) -> Vec<Axial> {
    let mut found = Vec::new();
    for ring in 0..=14i32 {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    found.push(candidate);
                }
            }
        }
    }
    found
}

/// Founds the seat of one faction near one address, and returns the place.
fn seat_near(world: &mut World, faction: FactionId, wanted: Axial) -> Axial {
    for place in ground_near(world, wanted) {
        if world.found_group_at(place, GROUP, faction).is_ok() {
            return place;
        }
    }
    panic!("faction {} finds no seat near {wanted:?}", faction.0);
}

/// Founds a further city of one faction near one address, and returns the
/// place and the city.
fn city_near(world: &mut World, faction: FactionId, wanted: Axial) -> (Axial, Entity) {
    for place in ground_near(world, wanted) {
        let Ok(unit) = world.spawn_soldier(place, faction) else {
            continue;
        };
        if world.found_settlement(place, faction).is_ok() {
            let city = world
                .settlement_on(place)
                .expect("the fixture just founded a city here");
            return (place, city);
        }
        assert!(world.despawn_soldier(unit), "the unit is alive");
    }
    panic!("faction {} finds no city near {wanted:?}", faction.0);
}

/// Gives the watcher a leader, because the relation verb takes a speaker.
fn give_a_leader(world: &mut World, place: Axial) {
    let unit = world
        .spawn_soldier(place, WATCHER)
        .expect("the seat admits a unit");
    assert!(world.set_unit_type(unit, LEADER));
}

/// Writes the war weight and the campaign weight of the watcher.
fn weigh_the_watcher(world: &mut World, war: u8, renown: u8) {
    let drawn = world
        .faction_weights(WATCHER)
        .expect("the world holds the watcher");
    let weights = cachette_core::FactionWeights {
        war,
        renown,
        ..drawn
    };
    assert!(world.set_faction_weights(WATCHER, weights));
}

/// Puts builders of one faction on one tile and orders each to build a
/// wonder.
fn put_builders(world: &mut World, faction: FactionId, tile: Axial) {
    for _ in 0..BUILDERS {
        let unit = world
            .spawn_soldier(tile, faction)
            .expect("the ground admits a unit");
        world
            .order_build(unit, UpgradeCategory::WONDER)
            .expect("the builder stands on ground its faction holds");
    }
}

/// Returns the progress share of the furthest wonder of one faction, as the
/// wonder lookup states it.
///
/// **A fixture reads the lookup and not the reading the controller takes.** A
/// defect in that reading must turn the assertion of a test red, and a
/// fixture that read the same reading would fail first and hide it.
fn wonder_share(world: &World, faction: FactionId) -> i32 {
    world
        .wonder_sites()
        .iter()
        .filter(|site| site.holder == Some(faction))
        .map(|site| site.progress_share().0)
        .max()
        .unwrap_or(0)
}

/// Reads the first position of one field of the observation of one faction.
fn published(world: &World, faction: FactionId, name: &str) -> i64 {
    let field = observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema must declare the field `{name}`"));
    let array = world
        .faction_observation(faction)
        .expect("the number names a faction of the world");
    array[field.start as usize]
}

/// Steps the world until the wonder of the leading faction passes the test
/// the caller gives, and returns its share.
fn build_until(world: &mut World, enough: impl Fn(i32) -> bool) -> i32 {
    for _ in 0..BUILD_BOUND {
        world.step(THREADS).expect("the step runs");
        let share = wonder_share(world, LEADING);
        if enough(share) {
            return share;
        }
    }
    panic!("the builders never brought the wonder to the reading the fixture asks for");
}

/// Returns the faction number of every relation move the watcher made on the
/// last step, in the order they applied.
fn relation_targets(world: &World) -> Vec<u32> {
    let schema = world.action_schema();
    world
        .controller_log()
        .iter()
        .filter(|entry| entry.faction == WATCHER)
        .filter_map(|entry| schema.decode(entry.action))
        .filter(|(verb, _)| *verb == Verb::Relation)
        .map(|(_, arguments)| arguments[0])
        .collect()
}

/// A world of three factions in which the leading faction builds a wonder
/// and the other faction holds the most ground.
///
/// **The other faction founds two cities, so it is the rival by held ground.**
/// The overmatch ratio of the watcher is zero, so it names no prey. Before
/// this rule the watcher could move its relation toward the other faction
/// alone.
fn a_wonder_underway(seed: u64, enough: impl Fn(i32) -> bool) -> (World, Axial) {
    let mut world = a_still_world(3, seed);
    let home = seat_near(&mut world, WATCHER, Axial::new(12, 48));
    let wonder = seat_near(&mut world, LEADING, Axial::new(48, 16));
    seat_near(&mut world, OTHER, Axial::new(48, 80));
    city_near(&mut world, OTHER, Axial::new(84, 80));
    give_a_leader(&mut world, home);
    for a in [WATCHER, LEADING, OTHER] {
        for b in [WATCHER, LEADING, OTHER] {
            world.meet(a, b);
        }
    }
    assert!(world.set_faction_overmatch_ratio(WATCHER, 0));
    world.step(THREADS).expect("the step runs");
    put_builders(&mut world, LEADING, wonder);
    build_until(&mut world, enough);
    assert!(
        world.score(OTHER) > world.score(LEADING),
        "the other faction must hold the most ground, or the old rule would \
         name the leading faction too"
    );
    (world, wonder)
}

/// A rival whose wonder reaches the share becomes the target of the relation
/// move, on every tick.
#[test]
fn a_rival_that_nears_the_wonder_path_becomes_the_target() {
    let (mut world, _) = a_wonder_underway(0x5ee_0001, |share| share >= WIN_THREAT_SHARE_DEFAULT);
    weigh_the_watcher(&mut world, WEIGHT_HIGH, WEIGHT_LOW);
    assert!(world.set_externally_controlled(WATCHER, false));
    for _ in 0..WINDOW {
        world.step(THREADS).expect("the step runs");
        assert!(
            !world.game_end().is_set(),
            "the wonder must stay unfinished while the test reads the moves"
        );
        assert_eq!(
            relation_targets(&world),
            vec![u32::from(LEADING.0)],
            "the watcher must move its relation toward the rival that nears \
             the wonder path, and toward nobody else, on every tick"
        );
    }
}

/// A rival whose wonder is below the share does not change the target.
///
/// **The watcher moves its relation often, so the test can see a target.** It
/// holds the highest war weight, and its rival by held ground is the other
/// faction. A rule that took any wonder as a threat would move the relation
/// toward the builder.
#[test]
fn a_rival_below_the_share_does_not_change_the_target() {
    let (mut world, _) = a_wonder_underway(0x5ee_0001, |share| share > 0);
    weigh_the_watcher(&mut world, WEIGHT_HIGH, WEIGHT_LOW);
    assert!(world.set_externally_controlled(WATCHER, false));
    let mut moved_toward_the_rival = 0;
    for _ in 0..WINDOW * 2 {
        world.step(THREADS).expect("the step runs");
        let share = wonder_share(&world, LEADING);
        assert!(
            share > 0 && share < WIN_THREAT_SHARE_DEFAULT,
            "the fixture must hold the wonder below the share, and it reads {share}"
        );
        let targets = relation_targets(&world);
        assert!(
            !targets.contains(&u32::from(LEADING.0)),
            "a rival below the share is not a target, and the watcher moved \
             toward it"
        );
        moved_toward_the_rival += targets
            .iter()
            .filter(|target| **target == u32::from(OTHER.0))
            .count();
    }
    assert!(
        moved_toward_the_rival > 0,
        "the watcher must move toward its rival by held ground at least once, \
         or the test cannot see which target it takes"
    );
}

/// The march on a rival that nears the wonder path aims at the city whose
/// ground holds the wonder, and not at the nearer city of that rival.
///
/// **The near city is the seat of the rival and the nearest enemy city to the
/// watcher.** The rule that shipped before this one marched on it.
#[test]
fn the_march_on_a_wonder_aims_at_the_city_that_holds_it() {
    let mut world = a_still_world(2, 0x5ee_0002);
    let home = seat_near(&mut world, WATCHER, Axial::new(12, 48));
    let near = seat_near(&mut world, LEADING, Axial::new(34, 48));
    let (far, _) = city_near(&mut world, LEADING, Axial::new(80, 48));
    assert!(
        home.distance(near) < home.distance(far),
        "the fixture must put the city that holds the wonder further from \
         the watcher than the other city of the rival"
    );
    give_a_leader(&mut world, home);
    for _ in 0..4 {
        world
            .spawn_soldier(home, WATCHER)
            .expect("the seat admits a unit");
    }
    world.step(THREADS).expect("the step runs");
    put_builders(&mut world, LEADING, far);
    build_until(&mut world, |share| share >= WIN_THREAT_SHARE_DEFAULT);

    let edge = world.relation_rules().war_edge;
    assert!(world.set_relation(WATCHER, LEADING, edge - 1));
    assert!(world.set_relation(LEADING, WATCHER, edge - 1));
    weigh_the_watcher(&mut world, WEIGHT_LOW, WEIGHT_HIGH);
    assert!(world.set_externally_controlled(WATCHER, false));
    let far_tile = world
        .grid()
        .index_of(far)
        .expect("the city lies inside the world");
    let near_tile = world
        .grid()
        .index_of(near)
        .expect("the seat lies inside the world");

    let mut raised = None;
    for _ in 0..40 {
        world.step(THREADS).expect("the step runs");
        assert!(
            !world.game_end().is_set(),
            "the wonder must stay unfinished until the watcher raises"
        );
        if let Some(row) = world.campaigns_of(WATCHER).iter().find(|row| row.is_set()) {
            raised = Some(*row);
            break;
        }
    }
    let row = raised.expect("the watcher must raise a campaign against the rival at war");
    assert_ne!(
        row.objective_tile, near_tile.0,
        "the march went to the nearest city of the rival, and not to the wonder"
    );
    assert_eq!(
        row.objective_tile, far_tile.0,
        "the march must aim at the city whose ground holds the wonder"
    );
}

/// A rival that nears the domination path becomes the target, ahead of the
/// prey.
///
/// **The fixture names a prey.** The third faction loses its city, so the
/// watcher overmatches it and the rule that shipped before this one moved
/// the relation toward it on every tick. The other faction takes the seat of
/// the third, and holds two of the three seats of the game.
#[test]
fn a_rival_that_nears_another_win_path_becomes_the_target() {
    let mut world = a_still_world(3, 0x5ee_0003);
    let home = seat_near(&mut world, WATCHER, Axial::new(12, 48));
    let taken = seat_near(&mut world, LEADING, Axial::new(48, 16));
    seat_near(&mut world, OTHER, Axial::new(48, 80));
    for a in [WATCHER, LEADING, OTHER] {
        for b in [WATCHER, LEADING, OTHER] {
            world.meet(a, b);
        }
    }
    give_a_leader(&mut world, home);
    world.step(THREADS).expect("the step runs");

    let units: Vec<Entity> = world.soldiers().iter().collect();
    for unit in units {
        if world.soldiers().faction(unit) == Some(LEADING) {
            assert!(world.despawn_soldier(unit), "the unit is alive");
        }
    }
    let refuge = ground_near(&world, Axial::new(84, 20))[0];
    world
        .spawn_soldier(refuge, LEADING)
        .expect("the refuge admits a unit");
    let city = world
        .settlement_on(taken)
        .expect("the seat holds the city of the faction");
    assert!(world.destroy_settlement(city));
    world
        .spawn_soldier(taken, OTHER)
        .expect("the seat admits a unit");
    world
        .found_settlement(taken, OTHER)
        .expect("the seat admits a city");
    world.step(THREADS).expect("the step runs");

    assert!(
        !world.is_eliminated(LEADING),
        "the faction that lost its seat keeps a unit, so it stays in the game \
         and its seat stays in the ask"
    );
    assert!(
        published(&world, OTHER, "domination_progress") >= i64::from(WIN_THREAT_SHARE_DEFAULT),
        "the fixture must bring the other faction to the share on the \
         domination path, as the observation publishes it"
    );
    assert_eq!(
        wonder_share(&world, OTHER),
        0,
        "the fixture must name the domination path and no wonder"
    );
    assert_eq!(
        world.score(LEADING),
        Some(0),
        "the faction that lost its city must hold no ground, so it is the prey"
    );

    weigh_the_watcher(&mut world, WEIGHT_LOW, WEIGHT_LOW);
    assert!(world.set_externally_controlled(WATCHER, false));
    for _ in 0..WINDOW {
        world.step(THREADS).expect("the step runs");
        assert_eq!(
            relation_targets(&world),
            vec![u32::from(OTHER.0)],
            "the watcher must move its relation toward the rival that nears \
             the domination path, ahead of the prey"
        );
    }
}

/// A share of zero takes the rule out of the game, and the watcher then moves
/// toward nobody that nears a win.
///
/// **This is the switch that proves the first test can fail.** The world is
/// the world of that test, and the one input that changes is the share.
#[test]
fn a_share_of_zero_takes_the_rule_out_of_the_game() {
    let (mut world, _) = a_wonder_underway(0x5ee_0001, |share| share >= WIN_THREAT_SHARE_DEFAULT);
    weigh_the_watcher(&mut world, WEIGHT_HIGH, WEIGHT_LOW);
    assert!(world.set_faction_win_threat_share(WATCHER, 0));
    assert!(world.set_externally_controlled(WATCHER, false));
    for _ in 0..WINDOW {
        world.step(THREADS).expect("the step runs");
        assert!(
            !relation_targets(&world).contains(&u32::from(LEADING.0)),
            "a watcher with a share of zero names no win threat"
        );
    }
}

/// The reading the controller takes is the progress the observation
/// publishes to each faction about itself.
///
/// **The two read one set of readers, and this test fails when they part.**
/// The controller must not state a second rule for how far a faction has
/// come.
#[test]
fn the_controller_reads_the_progress_the_observation_publishes() {
    let (world, _) = a_wonder_underway(0x5ee_0001, |share| share >= WIN_THREAT_SHARE_DEFAULT);
    let fields = [
        (WinPath::Domination, "domination_progress"),
        (WinPath::Territory, "ground_progress"),
        (WinPath::Wonder, "wonder_track_progress"),
        (WinPath::Renown, "renown_progress"),
    ];
    let readings = world.win_readings();
    for faction in 0..3u16 {
        for (path, name) in fields {
            assert_eq!(
                i64::from(readings[usize::from(faction)][path.index()].0),
                published(&world, FactionId(faction), name),
                "faction {faction} reads one progress on the {} path",
                path.name()
            );
        }
    }
    assert!(
        readings[usize::from(LEADING.0)][WinPath::Wonder.index()].0 > 0,
        "the fixture must hold a wonder, or the wonder reading compares zero \
         with zero"
    );
}
