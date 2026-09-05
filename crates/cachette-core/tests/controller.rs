//! The faction controller runs inside the step, and the game ends on
//! territory.
//!
//! Every test here drives the engine through `step` and reads the controller
//! log, the game end record or the census afterwards. None reaches into the
//! controller.[^1]
//!
//! The keyed draw gets one test for each field of its key, because the
//! determinism tests cannot tell a draw keyed on the wrong field from a
//! correct one.[^2] The fixture reaches two extremes: a faction with no seat,
//! and two factions that tie on territory.[^3]
//!
//! # References
//!
//! [^1]: Testing Rules, sections 5 and 6. `.agents/rules/testing.md`
//! [^2]: Testing Rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use std::collections::BTreeSet;

use cachette_core::controller::NO_SEAT;
use cachette_core::{
    Axial, ControllerCommand, FactionId, GameEnd, WinPath, World, WorldConfig, SUBSYSTEM_CENSUS,
};

const THREADS: usize = 2;

/// The people each founding settles. Small, so a step is cheap.
const GROUP: u32 = 8;

fn config(factions: u16, seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Founds one group for each of the first `seated` factions, at places the
/// founding survey accepts, far enough apart that the survey does not refuse
/// the second for the first.
fn seat(world: &mut World, seated: u16) {
    let grid = world.grid();
    let mut taken: Vec<Axial> = Vec::new();
    for faction in 0..seated {
        let mut founded = false;
        // The first pass keeps the seats apart. The second takes any place
        // the survey accepts, because the probe build perturbs the survey
        // and a fixture that depends on the spacing would fail there for a
        // reason that is not the controller.
        for spacing in [12, 0] {
            for index in 0..grid.tile_count() {
                let address =
                    Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
                if !world.admits_a_unit(address) {
                    continue;
                }
                if taken.iter().any(|place| {
                    (place.q - address.q).abs() < spacing || (place.r - address.r).abs() < spacing
                }) {
                    continue;
                }
                if world
                    .found_group_at(address, GROUP, FactionId(faction))
                    .is_ok()
                {
                    taken.push(address);
                    founded = true;
                    break;
                }
            }
            if founded {
                break;
            }
        }
        assert!(founded, "faction {faction} must find a place");
    }
}

fn log_of(world: &World) -> Vec<ControllerCommand> {
    world.controller_log().to_vec()
}

#[test]
fn a_faction_with_no_seat_receives_no_evaluation() {
    // The extreme: faction 1 has units and no seat. It founded nothing, so
    // the controller has nothing to plan around and must leave it alone.
    let mut world = World::new(config(2, 11)).expect("the extent describes a world");
    seat(&mut world, 1);
    let grid = world.grid();
    let open = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| world.admits_a_unit(*address) && address.q > 30 && address.r > 30)
        .expect("some ground admits a unit");
    world
        .spawn_soldier(open, FactionId(1))
        .expect("the address and the faction are valid");
    assert_eq!(world.seat(FactionId(1)), None);
    assert_ne!(world.seat(FactionId(0)), None);

    world.step(THREADS).expect("the step runs");
    let log = log_of(&world);
    assert!(!log.is_empty(), "the seated faction emits");
    assert!(
        log.iter().all(|entry| entry.faction == FactionId(0)),
        "the faction with no seat emits nothing: {log:?}"
    );
    assert_ne!(
        NO_SEAT, 0,
        "the sentinel is not a tile index a small world holds"
    );
}

#[test]
fn the_controller_emits_only_through_the_set_verbs_and_logs_each_command() {
    let mut world = World::new(config(2, 12)).expect("the extent describes a world");
    seat(&mut world, 2);
    world.set_controller_evaluations(3);
    world.step(THREADS).expect("the step runs");
    let log = log_of(&world);
    assert_eq!(log.len(), 6, "two factions, three evaluations each");
    for entry in &log {
        assert_eq!(entry.tick.0, 1);
        assert_eq!(entry.applied, 1, "a seated faction has units to order");
        assert!(
            entry.kind <= 1,
            "only a gather order or a build order exists"
        );
    }
    // The census sees the same six.
    let census = world.subsystem_census();
    let commands = census
        .iter()
        .find(|(name, _)| *name == "controller_commands")
        .expect("the row exists")
        .1;
    assert_eq!(commands, 6);
}

/// Collects the choice of one entry as a comparable pair.
fn choice(entry: &ControllerCommand) -> (u8, u8) {
    (entry.kind, entry.argument)
}

#[test]
fn the_draw_depends_on_the_tick() {
    let mut world = World::new(config(1, 13)).expect("the extent describes a world");
    seat(&mut world, 1);
    world.set_controller_evaluations(1);
    let mut seen = BTreeSet::new();
    for _ in 0..24 {
        world.step(THREADS).expect("the step runs");
        let log = log_of(&world);
        assert_eq!(log.len(), 1);
        seen.insert(choice(&log[0]));
    }
    assert!(
        seen.len() > 1,
        "one faction, one draw, twenty-four ticks: the choice must change with the tick"
    );
}

#[test]
fn the_draw_depends_on_the_faction() {
    let mut world = World::new(config(3, 17)).expect("the extent describes a world");
    seat(&mut world, 3);
    world.set_controller_evaluations(1);
    // The weights differ by faction, so the kind may differ even when the
    // draw does not. The argument comes from the high bits of the draw
    // alone, so two factions that chose one kind and two arguments prove
    // that the faction is in the key.
    let mut differed = false;
    for _ in 0..24 {
        world.step(THREADS).expect("the step runs");
        let log = log_of(&world);
        assert_eq!(log.len(), 3);
        for kind in [0u8, 1u8] {
            let arguments: BTreeSet<u8> = log
                .iter()
                .filter(|entry| entry.kind == kind)
                .map(|entry| entry.argument)
                .collect();
            if arguments.len() > 1 {
                differed = true;
            }
        }
    }
    assert!(
        differed,
        "three factions, one draw each: on some tick two factions of one kind must differ in argument"
    );
}

#[test]
fn the_draw_depends_on_the_draw_index() {
    let mut world = World::new(config(1, 15)).expect("the extent describes a world");
    seat(&mut world, 1);
    world.set_controller_evaluations(4);
    let mut differed = false;
    for _ in 0..24 {
        world.step(THREADS).expect("the step runs");
        let log = log_of(&world);
        assert_eq!(log.len(), 4);
        let choices: BTreeSet<(u8, u8)> = log.iter().map(choice).collect();
        if choices.len() > 1 {
            differed = true;
        }
    }
    assert!(
        differed,
        "one faction, four draws: on some tick two draws must choose differently"
    );
}

#[test]
fn the_weights_come_from_the_seed_and_sit_inside_the_range() {
    let a = World::new(config(2, 21)).expect("the extent describes a world");
    let b = World::new(config(2, 22)).expect("the extent describes a world");
    let same = World::new(config(2, 21)).expect("the extent describes a world");
    let weights_a = a.faction_weights(FactionId(0)).expect("faction 0 exists");
    assert_eq!(
        weights_a,
        same.faction_weights(FactionId(0))
            .expect("faction 0 exists")
    );
    let mut any_differs = false;
    for faction in 0..2 {
        let wa = a
            .faction_weights(FactionId(faction))
            .expect("the faction exists");
        let wb = b
            .faction_weights(FactionId(faction))
            .expect("the faction exists");
        for weight in [wa.war, wa.trade, wa.build, wa.renown] {
            assert!((cachette_core::WEIGHT_LOW..=cachette_core::WEIGHT_HIGH).contains(&weight));
        }
        if wa != wb {
            any_differs = true;
        }
    }
    assert!(any_differs, "two seeds must give two vectors somewhere");
    assert_eq!(a.faction_weights(FactionId(2)), None, "no such faction");
}

#[test]
fn a_faction_under_external_control_receives_no_evaluation() {
    let mut world = World::new(config(2, 16)).expect("the extent describes a world");
    seat(&mut world, 2);
    assert_eq!(world.is_externally_controlled(FactionId(1)), Some(false));
    assert!(world.set_externally_controlled(FactionId(1), true));
    assert_eq!(world.is_externally_controlled(FactionId(1)), Some(true));
    assert!(!world.set_externally_controlled(FactionId(7), true));

    let before = world.state_hash().finish();
    world.step(THREADS).expect("the step runs");
    let log = log_of(&world);
    assert!(!log.is_empty(), "faction 0 still emits");
    assert!(
        log.iter().all(|entry| entry.faction == FactionId(0)),
        "faction 1 is silenced: {log:?}"
    );
    // The flag is state, so the hash saw it change.
    let mut plain = World::new(config(2, 16)).expect("the extent describes a world");
    seat(&mut plain, 2);
    assert_ne!(plain.state_hash().finish(), before);
}

#[test]
fn the_commands_apply_in_faction_then_sequence_order() {
    // The probe build visits the factions and the draws backwards. The sort
    // is what makes this pass there, and removing it makes this fail there.
    let mut world = World::new(config(3, 17)).expect("the extent describes a world");
    seat(&mut world, 3);
    world.set_controller_evaluations(3);
    world.step(THREADS).expect("the step runs");
    let log = log_of(&world);
    assert_eq!(log.len(), 9);
    let keys: Vec<(FactionId, u32)> = log
        .iter()
        .map(|entry| (entry.faction, entry.sequence))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "the log is in (faction, sequence) order");
}

#[test]
fn the_game_ends_once_on_territory_and_the_controller_then_emits_nothing() {
    let mut world = World::new(config(2, 18)).expect("the extent describes a world");
    seat(&mut world, 2);
    world.set_tick_limit(3);
    assert_eq!(world.tick_limit(), 3);
    for _ in 0..2 {
        world.step(THREADS).expect("the step runs");
        assert!(!world.game_end().is_set(), "no end before the limit");
        assert!(!log_of(&world).is_empty());
    }
    world.step(THREADS).expect("the step runs");
    let end = world.game_end();
    assert!(end.is_set());
    assert_eq!(end.tick.0, 3);
    assert_eq!(end.win_path(), Some(WinPath::Territory));
    assert!(
        log_of(&world).is_empty(),
        "the tick that ends the game emits nothing"
    );
    let census = world.subsystem_census();
    assert_eq!(
        census
            .iter()
            .find(|(name, _)| *name == "game_ended")
            .map(|row| row.1),
        Some(1)
    );

    // Ten more ticks: the record does not move, the controller stays quiet,
    // and the world keeps stepping.
    let mut hashes = BTreeSet::new();
    for _ in 0..10 {
        world.step(THREADS).expect("the step runs");
        assert_eq!(world.game_end(), end, "the record is written once");
        assert!(
            log_of(&world).is_empty(),
            "the controller emits nothing after the end"
        );
        hashes.insert(world.state_hash().finish());
    }
    assert_eq!(world.tick().0, 13);
    assert!(hashes.len() > 1, "the world still moves after the end");
}

#[test]
fn a_territory_tie_goes_to_the_lowest_faction_and_a_lead_wins() {
    // The extreme: every faction holds the same count, here zero, because
    // nobody founded and nobody stands anywhere. The lowest identifier wins.
    let mut tied = World::new(config(3, 19)).expect("the extent describes a world");
    tied.set_tick_limit(1);
    tied.step(THREADS).expect("the step runs");
    let end = tied.game_end();
    assert!(end.is_set());
    assert_eq!(end.winner, FactionId(0));
    assert_eq!(tied.score(FactionId(0)), Some(0));
    assert_eq!(tied.score(FactionId(2)), Some(0));
    assert_eq!(tied.score(FactionId(3)), None);

    // A faction that holds ground beats one that holds none. Faction 1 is
    // the only one seated, so it is the only one that can hold anything, and
    // a tie at zero would have named faction 0 instead.
    let mut led = World::new(config(2, 20)).expect("the extent describes a world");
    {
        let grid = led.grid();
        let mut founded = false;
        for index in 0..grid.tile_count() {
            let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
            if led.found_group_at(address, GROUP, FactionId(1)).is_ok() {
                founded = true;
                break;
            }
        }
        assert!(founded);
    }
    led.set_tick_limit(40);
    for _ in 0..40 {
        led.step(THREADS).expect("the step runs");
    }
    let end = led.game_end();
    assert!(end.is_set());
    let score = led.score(FactionId(1)).expect("faction 1 exists");
    assert!(
        score > 0,
        "the seated faction holds ground by tick 40, holds {score}"
    );
    assert_eq!(end.winner, FactionId(1));
}

#[test]
fn the_empty_record_is_the_default_and_hashes() {
    let world = World::new(config(1, 23)).expect("the extent describes a world");
    assert_eq!(world.game_end(), GameEnd::EMPTY);
    assert_eq!(world.game_end().win_path(), None);
    // The tick limit and the evaluation count are read on every tick, so
    // they are in the hash.
    let mut other = World::new(config(1, 23)).expect("the extent describes a world");
    let before = other.state_hash().finish();
    other.set_tick_limit(5);
    let limit_changed = other.state_hash().finish();
    assert_ne!(before, limit_changed);
    other.set_controller_evaluations(9);
    assert_ne!(limit_changed, other.state_hash().finish());
}

#[test]
fn the_census_has_one_declaration_and_every_row_answers() {
    let names: Vec<&str> = SUBSYSTEM_CENSUS.iter().map(|row| row.name).collect();
    let unique: BTreeSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "no name appears twice");
    let mut world = World::new(config(2, 24)).expect("the extent describes a world");
    world.seed_world().expect("a fresh world seeds once");
    for _ in 0..4 {
        world.step(THREADS).expect("the step runs");
    }
    let census = world.subsystem_census();
    let read: Vec<&str> = census.iter().map(|(name, _)| *name).collect();
    assert_eq!(read, names, "the reader walks the table in table order");
    let count = |name: &str| {
        census
            .iter()
            .find(|(row, _)| *row == name)
            .map(|row| row.1)
            .expect("the row exists")
    };
    assert!(count("units") > 0);
    assert!(count("settlements") > 0);
    assert!(count("luxury_tiles") > 0);
    assert!(count("controller_commands") > 0);
    assert_eq!(count("game_ended"), 0);
}

#[test]
fn a_world_seeds_once_from_its_seed_and_the_verbs_still_serve_a_caller() {
    let mut seeded = World::new(config(3, 25)).expect("the extent describes a world");
    let outcomes = seeded.seed_world().expect("a fresh world seeds once");
    assert_eq!(outcomes.len(), 3);
    assert!(outcomes.iter().any(|outcome| outcome.founding().is_some()));
    assert!(seeded.luxuries_seeded());
    assert!(seeded.seed_world().is_err(), "a world seeds once");

    // The same seed gives the same world, and the founding verb gives the
    // same founding with the same group.
    let mut again = World::new(config(3, 25)).expect("the extent describes a world");
    again.seed_world().expect("a fresh world seeds once");
    assert_eq!(seeded.state_hash().finish(), again.state_hash().finish());
    let mut by_verb = World::new(config(3, 25)).expect("the extent describes a world");
    let verb_outcomes = by_verb.found_run_for_every_faction(cachette_core::FOUNDING_GROUP_DEFAULT);
    for (a, b) in outcomes.iter().zip(&verb_outcomes) {
        assert_eq!(
            a.founding().map(|founding| founding.place()),
            b.founding().map(|founding| founding.place())
        );
        assert_eq!(
            a.founding().map(|founding| founding.people().len()),
            b.founding().map(|founding| founding.people().len())
        );
    }
    assert!(!by_verb.luxuries_seeded());
}

#[test]
fn the_set_verbs_count_what_the_arena_refuses() {
    let mut world = World::new(config(1, 26)).expect("the extent describes a world");
    seat(&mut world, 1);
    let units: Vec<_> = world.soldiers().iter().collect();
    assert!(!units.is_empty());
    let refused = world.order_gather_set(&units, cachette_core::resource::ResourceKind::Wood);
    assert_eq!(refused, 0);
    let refused = world.order_build_set(&units, cachette_core::upgrade::UpgradeKind::Road);
    assert_eq!(refused, 0);
    let dead = units[0];
    assert!(world.despawn_soldier(dead));
    assert_eq!(
        world.order_gather_set(&[dead], cachette_core::resource::ResourceKind::Food),
        1
    );
    assert_eq!(
        world.set_unit_type_set(&[dead], cachette_core::unit_type::UnitTypeId(0)),
        1
    );
}
