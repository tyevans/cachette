//! A faction that does nothing becomes a target.
//!
//! The controller used to name one target only: the other faction with the
//! most held ground. A faction that took no action held the least ground, so
//! no controller ever declared on it. The weakest seat of a game was the one
//! seat nobody marched on.
//!
//! The controller now names a prey as well: the weakest other faction whose
//! held ground it overmatches by a stated factor. A faction with a prey moves
//! its relation toward war against the prey on every tick, and it draws
//! nothing for that move. The ratio is a balance value.[^1]
//!
//! The engine test drives the whole step and reads the relation afterwards.
//! It reaches into nothing.[^2] The fixture puts one faction under external
//! control and gives it no action, which is the extreme this rule exists
//! for.[^3]
//!
//! **The ratio is the switch that proves the test can fail.** A ratio of zero
//! takes the rule out of the game and restores the rule that shipped before
//! it. The last test asserts that the same world then leaves the idle faction
//! at peace.
//!
//! # References
//!
//! [^1]: Balance register, the overmatch ratio. `docs/reference/balance.md`
//! [^2]: Testing Rules, sections 5 and 6. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::controller::prey_of;
use cachette_core::{Axial, FactionId, Fix32, World, WorldConfig};

const THREADS: usize = 2;

/// The people each founding settles.
const GROUP: u32 = 8;

/// How many ticks the idle faction runs before the test reads the relation.
///
/// The hunt starts only when one faction holds the stated multiple of the
/// ground of another, and the factions start level. The count is the span in
/// which the acting factions pull ahead and then walk the relation down to
/// the war edge.
const TICKS: usize = 400;

/// The factor that names a prey in the test, as a raw Q16.16 value.
///
/// The test states the engine default rather than a value of its own, so a
/// change to the default moves the test with it.
const RATIO: i32 = cachette_core::controller::OVERMATCH_RATIO_DEFAULT;

fn config(factions: u16, seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Founds one group for each faction, at places the founding survey accepts,
/// far enough apart that the survey does not refuse the second for the first.
fn seat(world: &mut World, seated: u16) {
    let grid = world.grid();
    let mut taken: Vec<Axial> = Vec::new();
    for faction in 0..seated {
        let mut founded = false;
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

/// Builds a world of three factions and leaves the first one idle.
fn idle_seat_world(seed: u64, ratio: i32) -> World {
    let mut world = World::new(config(3, seed)).expect("the extent describes a world");
    seat(&mut world, 3);
    world.set_overmatch_ratio(ratio);
    assert!(world.set_externally_controlled(FactionId(0), true));
    world
}

/// Reports whether any acting faction is in the war band with the idle one.
fn hunted(world: &World) -> bool {
    (1..3).any(|other| world.at_war(FactionId(0), FactionId(other)))
}

#[test]
fn the_prey_is_the_weakest_faction_a_holding_overmatches() {
    let held = [
        (FactionId(0), 100i64),
        (FactionId(1), 60),
        (FactionId(2), 40),
        (FactionId(3), 49),
    ];
    let ratio = Fix32(RATIO);
    // Faction 0 holds twice the ground of faction 2 and of faction 3, and
    // less than twice the ground of faction 1. The weakest of the two it
    // overmatches is faction 2.
    assert_eq!(
        prey_of(FactionId(0), ratio, held.iter().copied()),
        Some(FactionId(2))
    );
    // Faction 1 overmatches nobody, because twice 40 is above 60.
    assert_eq!(prey_of(FactionId(1), ratio, held.iter().copied()), None);
}

#[test]
fn a_tie_between_two_prey_goes_to_the_lowest_faction() {
    let held = [
        (FactionId(0), 100i64),
        (FactionId(1), 20),
        (FactionId(2), 20),
    ];
    assert_eq!(
        prey_of(FactionId(0), Fix32(RATIO), held.iter().copied()),
        Some(FactionId(1))
    );
}

#[test]
fn a_faction_that_holds_nothing_hunts_nobody() {
    let held = [(FactionId(0), 0i64), (FactionId(1), 0)];
    assert_eq!(
        prey_of(FactionId(0), Fix32(RATIO), held.iter().copied()),
        None
    );
}

#[test]
fn a_ratio_of_zero_names_no_prey() {
    let held = [(FactionId(0), 100i64), (FactionId(1), 1)];
    assert_eq!(prey_of(FactionId(0), Fix32(0), held.iter().copied()), None);
}

#[test]
fn an_idle_faction_reaches_the_war_band() {
    let mut world = idle_seat_world(21, RATIO);
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
    }
    assert!(
        hunted(&world),
        "an idle faction must be hunted, and the holdings were {:?}",
        (0..3)
            .map(|faction| world.score(FactionId(faction)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_ratio_of_zero_leaves_an_idle_faction_at_peace() {
    let mut world = idle_seat_world(21, 0);
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
    }
    assert!(
        !hunted(&world),
        "the rule that shipped before the hunt left an idle faction alone"
    );
}
