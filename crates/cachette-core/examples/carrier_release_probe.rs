//! Reads what the carrier economy of the demonstration world did.
//!
//! The controller sends every unit of the demonstration world. A unit the
//! control plane sent climbs its destination plane and reads no option row,
//! so a unit that arrives and is never released neither gathers nor
//! delivers. The probe reports the four figures that separate a working
//! carrier economy from a stopped one.[^1]
//!
//! The figures are the heaviest load any live unit holds, the number of
//! units the choice calls laden, the number that hold the option which
//! carries a load home, and the total the engine delivered.
//!
//! The probe drives the step. It calls no pass of its own.[^2]
//!
//! # References
//!
//! [^1]: Findings register, FND-572. `docs/FINDINGS.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::choose::CarryClass;
use cachette_core::resource::ResourceKind;
use cachette_core::{World, WorldConfig};

/// The option index of the row that carries a load home.
const DELIVER: u8 = 0;

/// Returns the value of a named argument, or the fallback.
fn number(name: &str, fallback: u64) -> u64 {
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == name {
            return args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(fallback);
        }
    }
    fallback
}

fn main() {
    let ticks = number("--ticks", 300);
    let mut world = World::new(WorldConfig {
        width: 256,
        height: 256,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        ..Default::default()
    })
    .expect("the world builds");
    let outcomes = world.found_run_for_every_faction(64);
    let founded = outcomes.iter().filter(|it| it.founding().is_some()).count();
    for _ in 0..ticks {
        world.step(2).expect("the step runs");
    }

    let mut top = 0u32;
    let mut laden = 0usize;
    let mut delivering = 0usize;
    let mut sent = 0usize;
    for unit in world.soldiers().iter() {
        let held: u32 = ResourceKind::ALL
            .iter()
            .map(|kind| {
                world
                    .soldiers()
                    .carry(unit)
                    .map_or(0, |load| load.of(*kind).0)
            })
            .sum();
        top = top.max(held);
        if world.carry_class(unit) == Some(CarryClass::Laden) {
            laden += 1;
        }
        if world.soldier_intent(unit) == Some(Some(DELIVER)) {
            delivering += 1;
        }
        if matches!(world.sent_to(unit), Some(Some(_))) {
            sent += 1;
        }
    }
    let delivered: u64 = world.delivered_carry().iter().sum();
    let mark = world.carry_mark().0;
    println!("{founded} factions founded, carry mark {mark}, at {ticks} ticks");
    println!("top load {top}, laden {laden}, delivering {delivering}, delivered {delivered}");
    println!("still sent {sent}");
}
