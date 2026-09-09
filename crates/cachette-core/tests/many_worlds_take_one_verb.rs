//! Many worlds in one process take one verb each, over threads, and the
//! observation of every decision matches the layout the engine declares.
//!
//! A baseline run plays one constant-preference policy over many seeds. It
//! builds one world for each pair of a candidate and a seed, holds them all
//! in one process, spreads them over worker threads, and reads the
//! observation and the legality answer of one faction before each decision.
//! A run of that shape aborted with a stack canary failure, and no test
//! covered the shape.[^1]
//!
//! This test drives the shape from Rust alone. It builds many worlds, gives
//! each of them one action row, steps them over threads with the stride a
//! batch uses, and asserts three things on every decision.
//!
//! 1. The observation holds exactly the positions the schema declares.
//! 2. Every position sits inside the bounds its own field declares.
//! 3. The legality answer holds exactly one byte for each row of the action
//!    table.
//!
//! **The first two assertions are the ones that catch a writer that runs
//! past its field.** The array and the schema come from one field list, so a
//! field whose writer writes more positions than the field declares moves
//! every field above it, and a length that disagrees with the declared
//! length says so at the public interface.[^2]
//!
//! The rows this test drives are the rows the aborted run was scoring. Row
//! zero is the no-op, and a policy takes its own row when the row is legal
//! and the no-op otherwise, which is what the baseline does.
//!
//! **This test asserts nothing about time.** It runs a fixed number of
//! decisions over a fixed number of worlds, and it reads no clock.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: Testing rules, section 3. `.agents/rules/testing.md`

use cachette_core::{FactionId, World, WorldConfig};

/// The extent of the world the baseline plays.
const EXTENT: u32 = 48;

/// The factions the baseline world holds.
const FACTIONS: u16 = 3;

/// The seat the learner plays.
const SEAT: FactionId = FactionId(0);

/// The tick limit of one episode of the baseline.
const TICK_LIMIT: u64 = 600;

/// How many ticks one decision covers.
const DECISION_INTERVAL: u64 = 5;

/// How many decisions this test takes.
///
/// **The interpreter runs the same test over one decision.** Miri executes
/// this test to check the memory model and the thread model, and it runs the
/// engine some hundreds of times slower than the machine does. One decision
/// over one seed reaches every read, every write and every join that the
/// longer run reaches, and it reaches them once.[^1]
///
/// # References
///
/// [^1]: ADR-0041, a crate split enforces the boundary at compile time. `docs/adrs/draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md`
#[cfg(not(miri))]
const DECISIONS: u64 = 24;

/// How many decisions this test takes under the interpreter.
#[cfg(miri)]
const DECISIONS: u64 = 1;

/// The first seed of the set the aborted run played.
const FIRST_SEED: u64 = 50_000;

/// How many seeds each row plays.
#[cfg(not(miri))]
const SEEDS: u64 = 4;

/// How many seeds each row plays under the interpreter.
#[cfg(miri)]
const SEEDS: u64 = 1;

/// The action rows the aborted run was scoring when it aborted.
const ROWS: [u32; 4] = [16, 17, 18, 19];

/// How many worker threads carry the worlds.
const WORKERS: usize = 4;

/// One world, and the row the policy that plays it prefers.
struct Seat {
    /// The world of this pair.
    world: World,
    /// The row this policy takes whenever the row is legal.
    row: u32,
}

/// Builds one world of the baseline shape, seeded and given to the caller.
fn build(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: FACTIONS,
        ..WorldConfig::default()
    })
    .expect("the baseline extent must describe a world");
    world
        .seed_world()
        .expect("the baseline extent must seat every faction");
    world.set_win_readers_enabled(true);
    world.set_tick_limit(TICK_LIMIT);
    world.set_externally_controlled(SEAT, true);
    world
}

/// Reads the observation and the legality answer, and checks both against the
/// layouts the world declares.
fn check_layouts(world: &World) {
    let schema = cachette_core::faction_observation::observation_schema();
    let values = world
        .faction_observation(SEAT)
        .expect("the seat must hold an observation");
    assert_eq!(
        values.len(),
        schema.length() as usize,
        "the observation holds {} positions and the schema declares {}",
        values.len(),
        schema.length()
    );
    for row in schema.rows() {
        let first = row.start as usize;
        let last = first + row.positions as usize;
        assert!(
            last <= values.len(),
            "the field {} covers positions {first} to {last} of an array of {}",
            row.name(),
            values.len()
        );
        for (offset, value) in values[first..last].iter().enumerate() {
            assert!(
                *value >= row.low && *value <= row.high,
                "position {offset} of the field {} holds {value}, and the field \
                 declares the range {} to {}",
                row.name(),
                row.low,
                row.high
            );
        }
    }
    let legal = world
        .legal_actions(SEAT)
        .expect("the seat must hold a legality answer");
    assert_eq!(
        legal.len(),
        world.action_schema().length() as usize,
        "the legality answer holds {} bytes and the table holds {} rows",
        legal.len(),
        world.action_schema().length()
    );
}

/// Takes one decision on one world, then runs it for the decision interval.
fn decide(seat: &mut Seat) {
    check_layouts(&seat.world);
    let legal = seat
        .world
        .legal_actions(SEAT)
        .expect("the seat must hold a legality answer");
    let taken = match legal.get(seat.row as usize) {
        Some(1) => seat.row,
        _ => 0,
    };
    seat.world.act(SEAT, taken);
    for _ in 0..DECISION_INTERVAL {
        seat.world
            .step(1)
            .expect("one thread is a legal thread count");
    }
}

/// Plays every row over every seed, over workers, and checks every layout.
///
/// The worlds are spread over the workers by the stride a batch uses, so each
/// index belongs to one worker and no two workers reach one world.
#[test]
fn many_worlds_take_one_verb_and_the_layouts_hold() {
    let mut seats: Vec<Seat> = Vec::new();
    for row in ROWS {
        for offset in 0..SEEDS {
            seats.push(Seat {
                world: build(FIRST_SEED + offset),
                row,
            });
        }
    }
    for _ in 0..DECISIONS {
        let mut chunks: Vec<Vec<&mut Seat>> = (0..WORKERS).map(|_| Vec::new()).collect();
        for (index, seat) in seats.iter_mut().enumerate() {
            chunks[index % WORKERS].push(seat);
        }
        std::thread::scope(|scope| {
            for chunk in &mut chunks {
                scope.spawn(move || {
                    for seat in chunk.iter_mut() {
                        decide(seat);
                    }
                });
            }
        });
    }
}
