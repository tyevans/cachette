//! A faction reads what happened to it lately, and who did it.
//!
//! The observation of a faction is a snapshot of one frame. A snapshot cannot
//! tell a faction that is gaining ground from one that is losing it, it cannot
//! say that a rival is taking a city now, and it cannot say that one rival
//! keeps killing the reader's people. The engine therefore keeps a decayed
//! count of each kind of event for each faction, and it names the faction that
//! caused each event that has a cause.[^1]
//!
//! **The failure this file exists to prevent is a missed step.** Every event
//! log of the engine holds one step and no more, because the step empties each
//! log before any system runs. An accumulator that advanced when a reader
//! asked, rather than when the step ran, would count one step in as many as
//! the reader skipped, and every value it published would stay plausible. The
//! first test drives one world at two read cadences and refuses any
//! difference.
//!
//! **The fixture produces the extreme and not the typical case.** A world of
//! one tile holds a crowd far past its capacity, so admission refuses every
//! step and nobody walks away.[^2] [^3] One heavy unit stands against the
//! crowd, its armour is above the crowd's attack, so the crowd delivers
//! exactly nothing and the heavy unit ends the same number on every step. The
//! arrival rate is therefore constant and known, which is what a decay test
//! needs.[^4]
//!
//! The tests see only the public crate interface.[^5]
//!
//! # References
//!
//! [^1]: The event history. `crates/cachette-core/src/event_memory.rs`
//! [^2]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^5]: Testing rules, section 6. `.agents/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::event_memory::{Decay, EventMemory, MemoryKind};
use cachette_core::faction_observation::{observation_schema, FieldRow};
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The thread counts that the determinism test runs at.
///
/// A determinism claim proved at one thread count proves nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 1. `.agents/rules/testing.md`
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The seed of a world whose one tile admits a unit.
const LAND_SEED: u64 = 1;

/// The type number of the light unit. It never reaches the heavy unit.
const BOWMAN: u8 = 0;

/// The type number of the heavy unit. Its armour is above the light unit's
/// attack, so the light unit contributes exactly zero against it.
const TANK: u8 = 1;

/// How many light units one heavy unit ends on each step.
///
/// The value is the attack of the heavy unit in whole units. It is a fixture
/// choice and not a balance figure, and the fixture states both below.
const FELLED_FOR_EACH_TANK: i64 = 4;

/// The fixed-point value of one, which every share of this block bounds at.
const ONE: i64 = Fix32::ONE.0 as i64;

/// Returns a worker row that fights with the given attack and armour.
const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour,
        ..WORKER_ROW
    }
}

/// Builds a world of one tile with the two fighting types defined, and puts
/// the two factions at war.
///
/// The light unit delivers one whole casualty and carries no armour. The heavy
/// unit delivers four and carries an armour above the light unit's attack. The
/// values are fixture content, and the caller states them here rather than the
/// engine holding them.
///
/// **Nobody in this fixture goes hungry.** A unit here belongs to no site, so
/// it draws from no store, and the default need rule would end every unit of
/// the fixture after a few hundred steps. A decay test runs for hundreds of
/// steps, so a fixture that starved would measure the shortage pass and not
/// the history. The need rule below is a fixture choice, and no test reads a
/// value of it.
fn battlefield(unit_capacity: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: 1,
        height: 1,
        seed: LAND_SEED,
        faction_count: 3,
        unit_capacity,
    })
    .expect("a world of one tile is a world");
    assert!(
        world.admits_a_unit(Axial::new(0, 0)),
        "the fixture needs ground that admits a unit"
    );
    declare_war(&mut world, 0, 1);
    world
        .define_unit_type(BOWMAN, fighter(Fix32::from_int(1), Fix32::ZERO))
        .expect("the light row is inside the table");
    world
        .define_unit_type(TANK, fighter(Fix32::from_int(4), Fix32::from_int(2)))
        .expect("the heavy row is inside the table");
    let never_hungry = NeedRule::new(
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::from_int(1),
    )
    .expect("no rate of the rule is below zero");
    world.set_need_rule(never_hungry);
    world
}

/// Puts two factions of a fixture at war, in both directions.
///
/// **A meeting resolves only across a pair at war.** Every fixture here is
/// about what a meeting costs and not about the gate, so each one declares the
/// war first. The edge is read from the world and not restated here.
fn declare_war(world: &mut World, first: u16, second: u16) {
    let war = world.relation_rules().war_edge - 1;
    assert!(world.set_relation(FactionId(first), FactionId(second), war));
    assert!(world.set_relation(FactionId(second), FactionId(first), war));
}

/// Spawns units of one faction and one type on one address.
fn spawn(
    world: &mut World,
    address: Axial,
    faction: u16,
    unit_type: u8,
    count: u32,
) -> Vec<Entity> {
    let kind = UnitTypeId::from_u8(unit_type).expect("the number names a row of the table");
    (0..count)
        .map(|_| {
            let unit = world
                .spawn_soldier(address, FactionId(faction))
                .expect("the ground admits a unit and the faction is inside the world");
            assert!(world.set_unit_type(unit, kind), "the unit is alive");
            unit
        })
        .collect()
}

/// Returns the row of one field of the observation schema.
///
/// The schema is a property of the layout and not of a world, so the reader
/// names no world.
fn row(name: &str) -> FieldRow {
    observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema must declare the field `{name}`"))
}

/// Reads one position of one field of the observation of one faction.
fn read(world: &World, faction: u16, name: &str, kind: MemoryKind) -> i64 {
    let field = row(name);
    let array = world
        .faction_observation(FactionId(faction))
        .expect("the number names a faction of the world");
    array[field.start as usize + kind.position()]
}

/// Reads the whole memory block of the observation of one faction.
///
/// The block is the five fields this file is about, and nothing else. A
/// comparison over the whole array would also compare the tick, which differs
/// between two worlds that ran a different number of steps.
fn memory_block(world: &World, faction: u16) -> Vec<i64> {
    let array = world
        .faction_observation(FactionId(faction))
        .expect("the number names a faction of the world");
    let mut out = Vec::new();
    for name in [
        "memory_recent",
        "memory_lasting",
        "memory_trend",
        "memory_worst_rival",
        "memory_concentration",
    ] {
        let field = row(name);
        let first = field.start as usize;
        out.extend_from_slice(&array[first..first + field.positions as usize]);
    }
    out
}

/// The engine advances the history on every step, whatever the reader does.
///
/// **This is the test that the accumulator cannot miss a step.** One world
/// reads the observation after every step. The other reads it once, at the
/// end. Both run the same number of steps of the same world, so the history
/// must agree. A history that advanced when a reader asked would count forty
/// steps in one world and one step in the other.
///
/// The discarded read inside the loop is the whole point of the fixture: it
/// happens on every step of one world and on no step of the other, and nothing
/// about it may reach the history.
#[test]
fn the_reader_cannot_change_what_the_history_holds() {
    const STEPS: u32 = 40;
    const CROWD: u32 = 4096;

    let mut read_every_step = battlefield(CROWD + 16);
    spawn(&mut read_every_step, Axial::new(0, 0), 0, BOWMAN, CROWD);
    spawn(&mut read_every_step, Axial::new(0, 0), 1, TANK, 1);
    let mut read_once = battlefield(CROWD + 16);
    spawn(&mut read_once, Axial::new(0, 0), 0, BOWMAN, CROWD);
    spawn(&mut read_once, Axial::new(0, 0), 1, TANK, 1);

    for _ in 0..STEPS {
        read_every_step.step(1).expect("the step runs");
        let _ = memory_block(&read_every_step, 0);
        read_once.step(1).expect("the step runs");
    }

    let seen = memory_block(&read_every_step, 0);
    let unseen = memory_block(&read_once, 0);
    assert!(
        seen.iter().any(|value| *value != 0),
        "the fixture must reach the case, and a block of zeroes measures nothing"
    );
    assert_eq!(
        seen, unseen,
        "the history must not depend on how often a caller reads it"
    );
}

/// The engine counts a loss that no reader ever asked about.
///
/// The world above proves that a reader changes nothing. This proves the other
/// half: a loss that happened on a step nobody read is still in the history at
/// the end. The battle runs for eight steps and the heavy unit then leaves,
/// and nobody reads the observation while any of that happens.
#[test]
fn a_loss_on_an_unread_step_reaches_the_history() {
    const CROWD: u32 = 4096;
    let mut world = battlefield(CROWD + 16);
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, CROWD);
    let tank = spawn(&mut world, Axial::new(0, 0), 1, TANK, 1);

    for _ in 0..8 {
        world.step(1).expect("the step runs");
    }
    assert!(world.despawn_soldier(tank[0]), "the heavy unit was alive");
    world.step(1).expect("the step runs");

    let felled = read(&world, 0, "memory_lasting", MemoryKind::OwnUnitsFelled);
    assert!(
        felled > 0,
        "eight steps of losses must be in the history, and the reader saw none of them"
    );
    let blamed = read(&world, 0, "memory_worst_rival", MemoryKind::OwnUnitsFelled);
    assert!(
        blamed > 0,
        "the history must name a faction as the cause of the losses"
    );
}

/// A constant arrival rate drives a counter to the rate shifted left by the
/// decay bits, and the counter never passes it.
///
/// The test drives the history at the extreme arrival rate of the world: one
/// arrival for every reserved unit slot, on every step. The counter must stop
/// at the declared bound and stay there.
#[test]
fn the_decay_reaches_its_bound_and_never_passes_it() {
    const EXTREME: i64 = 1 << 20;
    for length in Decay::ALL {
        for rate in [1i64, 7, EXTREME] {
            let mut memory = EventMemory::new(1);
            let bound = rate << length.bits();
            for _ in 0..8192 {
                memory.record(FactionId(0), MemoryKind::OwnUnitsFelled, rate);
                memory.advance();
                let held = memory.total(FactionId(0), MemoryKind::OwnUnitsFelled, length);
                assert!(
                    held <= bound,
                    "the {} counter reached {held} above the bound {bound} at a rate of {rate}",
                    length.name()
                );
            }
            assert_eq!(
                memory.total(FactionId(0), MemoryKind::OwnUnitsFelled, length),
                bound,
                "the {} counter must settle at the bound at a rate of {rate}",
                length.name()
            );
        }
    }
}

/// Every published position of the block stays inside the bounds the schema
/// declares.
///
/// A learner trains a function of the declared bounds, so a value outside them
/// is a value the learner cannot use. The battle drives the extreme: the whole
/// army of one faction falls, so every loss share reaches its own ceiling.
#[test]
fn every_published_position_holds_its_declared_bound() {
    const CROWD: u32 = 64;
    let mut world = battlefield(CROWD + 16);
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, CROWD);
    spawn(&mut world, Axial::new(0, 0), 1, TANK, 8);

    for _ in 0..64 {
        world.step(1).expect("the step runs");
        for seat in 0..3u16 {
            let array = world
                .faction_observation(FactionId(seat))
                .expect("the number names a faction of the world");
            for name in [
                "memory_recent",
                "memory_lasting",
                "memory_trend",
                "memory_worst_rival",
                "memory_concentration",
            ] {
                let field = row(name);
                let first = field.start as usize;
                for value in &array[first..first + field.positions as usize] {
                    assert!(
                        *value >= field.low && *value <= field.high,
                        "the field `{name}` published {value} outside {} to {}",
                        field.low,
                        field.high
                    );
                }
            }
        }
    }
}

/// A spike and a trend read differently.
///
/// The short memory rises faster than the long one, so a rate that has just
/// started reads a positive relation between the two. A rate that has just
/// stopped reads a negative one. A single counter cannot separate the two,
/// because a burst and a steady rate reach the same value.
///
/// The fixture holds the rate for long enough that the long memory catches the
/// short one. The heavy unit then leaves and the rate falls to nothing.
#[test]
fn a_spike_reads_differently_from_a_trend() {
    const CROWD: u32 = 4096;
    let mut world = battlefield(CROWD + 16);
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, CROWD);
    let tank = spawn(&mut world, Axial::new(0, 0), 1, TANK, 1);

    world.step(1).expect("the step runs");
    let spike = read(&world, 0, "memory_trend", MemoryKind::OwnUnitsFelled);
    assert!(
        spike > 0,
        "a rate that has just started must read above its own trend, and it read {spike}"
    );

    for _ in 0..512 {
        world.step(1).expect("the step runs");
    }
    assert!(world.despawn_soldier(tank[0]), "the heavy unit was alive");
    for _ in 0..16 {
        world.step(1).expect("the step runs");
    }
    let fading = read(&world, 0, "memory_trend", MemoryKind::OwnUnitsFelled);
    assert!(
        fading < 0,
        "a rate that has stopped must read below its own trend, and it read {fading}"
    );
    assert!(
        read(&world, 0, "memory_lasting", MemoryKind::OwnUnitsFelled) > 0,
        "the long memory must still hold the battle that the short one has forgotten"
    );
}

/// The published value does not follow the size of the army or of the world.
///
/// **This is the property that lets one policy read two worlds.** One world
/// holds an army of one size and loses four units on every step. The other
/// holds twice the army and loses eight. Both lose the same part of what they
/// have, so both must publish the same number. A raw count would publish four
/// and eight.
///
/// The test reads the short memory. The long memory settles over hundreds more
/// steps than the army survives, and the integer shift of the decay truncates
/// by a whole unit on each step, so two counters of different size differ
/// while they are still settling. The short memory settles inside the run, and
/// a settled counter is exactly the arrival rate shifted left by the decay
/// bits.
///
/// The two casualty assertions below check that the fixture supplies the
/// doubling it claims. A world whose casualty rate did not double would
/// measure nothing.
#[test]
fn the_published_value_is_free_of_scale() {
    const SMALL: u32 = 4096;
    const STEPS: i64 = 240;
    let mut small = battlefield(SMALL + 16);
    spawn(&mut small, Axial::new(0, 0), 0, BOWMAN, SMALL);
    spawn(&mut small, Axial::new(0, 0), 1, TANK, 1);
    let mut large = battlefield(2 * SMALL + 16);
    spawn(&mut large, Axial::new(0, 0), 0, BOWMAN, 2 * SMALL);
    spawn(&mut large, Axial::new(0, 0), 1, TANK, 2);

    for _ in 0..STEPS {
        small.step(1).expect("the step runs");
        large.step(1).expect("the step runs");
    }

    assert_eq!(
        i64::from(SMALL) - i64::from(small.population_of(FactionId(0))),
        STEPS * FELLED_FOR_EACH_TANK,
        "the small world must lose four units on every step"
    );
    assert_eq!(
        2 * i64::from(SMALL) - i64::from(large.population_of(FactionId(0))),
        2 * STEPS * FELLED_FOR_EACH_TANK,
        "the large world must lose eight units on every step"
    );

    let thin = read(&small, 0, "memory_recent", MemoryKind::OwnUnitsFelled);
    let thick = read(&large, 0, "memory_recent", MemoryKind::OwnUnitsFelled);
    assert!(thin > 0, "the short memory must reach the case");
    assert_eq!(
        thin, thick,
        "the short memory must not follow the size of the army"
    );
}

/// The concentration says whether one rival is the whole of the story.
///
/// One rival that gives all the harm gives the whole. Two that give it in
/// equal measure give a half. The reader tells one enemy from a field of them
/// by this position alone, and it names no seat.
///
/// **The fixture reads the agent side and not the subject side.** A meeting
/// names one faction as the cause of each group that lost units, and it breaks
/// a tie by the lowest seat number, so two rivals of equal strength standing
/// against the reader give the whole of the blame to one of them. The reader
/// here is the heavy side, so each rival is a losing group of its own and each
/// group names the reader as its cause. Two rivals of equal size then hold
/// exactly half of the reader's kills each, which is the split the test
/// claims.
///
/// The first assertion below checks that the fixture supplies that split. Two
/// rivals that lost a different number would give a concentration that no
/// exact value states.
#[test]
fn the_concentration_counts_the_rivals_the_harm_touched() {
    const CROWD: u32 = 200;
    let felled = MemoryKind::RivalUnitsFelled;

    let mut one_enemy = battlefield(2 * CROWD + 16);
    spawn(&mut one_enemy, Axial::new(0, 0), 0, TANK, 2);
    spawn(&mut one_enemy, Axial::new(0, 0), 1, BOWMAN, CROWD);
    one_enemy.step(1).expect("the step runs");

    let mut two_enemies = battlefield(2 * CROWD + 16);
    declare_war(&mut two_enemies, 0, 2);
    spawn(&mut two_enemies, Axial::new(0, 0), 0, TANK, 2);
    spawn(&mut two_enemies, Axial::new(0, 0), 1, BOWMAN, CROWD);
    spawn(&mut two_enemies, Axial::new(0, 0), 2, BOWMAN, CROWD);
    two_enemies.step(1).expect("the step runs");

    assert_eq!(
        two_enemies.population_of(FactionId(1)),
        two_enemies.population_of(FactionId(2)),
        "the two rivals must lose the same number of units"
    );
    assert!(
        read(&one_enemy, 0, "memory_recent", felled) > 0,
        "the fixture must reach the case, and a kill count of zero measures nothing"
    );

    assert_eq!(
        read(&one_enemy, 0, "memory_concentration", felled),
        ONE,
        "one rival that is the whole of the story gives the whole"
    );
    assert_eq!(
        read(&one_enemy, 0, "memory_worst_rival", felled),
        ONE,
        "one rival holds the whole of the worst share"
    );
    assert_eq!(
        read(&two_enemies, 0, "memory_concentration", felled),
        ONE / 2,
        "two rivals in equal measure give a half"
    );
    assert_eq!(
        read(&two_enemies, 0, "memory_worst_rival", felled),
        ONE / 2,
        "the worst of two equal rivals holds a half"
    );
}

/// The published value is the same at every thread count.
///
/// The history advances inside the step, so it is on the determinism path. A
/// value that followed a thread completion order would differ here.
#[test]
fn the_history_is_the_same_at_every_thread_count() {
    const CROWD: u32 = 4096;
    let mut answers = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = battlefield(CROWD + 16);
        spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, CROWD);
        spawn(&mut world, Axial::new(0, 0), 1, TANK, 4);
        for _ in 0..32 {
            world.step(threads).expect("the step runs");
        }
        let mut block = Vec::new();
        for seat in 0..3u16 {
            block.extend(memory_block(&world, seat));
        }
        answers.push((threads, block));
    }
    assert!(
        answers[0].1.iter().any(|value| *value != 0),
        "the fixture must reach the case, and a block of zeroes measures nothing"
    );
    for (threads, block) in &answers[1..] {
        assert_eq!(
            *block, answers[0].1,
            "the history at {threads} threads must equal the history at one thread"
        );
    }
}

/// One call writes the total row and the pair row, so the two cannot
/// disagree.
///
/// A caller that wrote one and forgot the other would leave a share of the
/// harm that no rival accounts for, and nothing would fail. A kind that names
/// no cause writes the total row alone, and the second half of the test reads
/// that case.
#[test]
fn a_blamed_arrival_reaches_both_rows() {
    let mut memory = EventMemory::new(4);
    memory.record_blamed(FactionId(0), FactionId(2), MemoryKind::OwnUnitsFelled, 5);
    memory.advance();
    assert_eq!(
        memory.total(FactionId(0), MemoryKind::OwnUnitsFelled, Decay::Recent),
        5
    );
    assert_eq!(
        memory.blamed_total(
            FactionId(0),
            FactionId(2),
            MemoryKind::OwnUnitsFelled,
            Decay::Recent
        ),
        5
    );
    memory.record_blamed(FactionId(0), FactionId(1), MemoryKind::OwnUnitsBurned, 3);
    memory.advance();
    assert_eq!(
        memory.blamed_total(
            FactionId(0),
            FactionId(1),
            MemoryKind::OwnUnitsBurned,
            Decay::Recent
        ),
        0
    );
    assert!(memory.total(FactionId(0), MemoryKind::OwnUnitsBurned, Decay::Recent) > 0);
}

/// The held tile delta reports a gain and a loss, and it reports each one
/// once.
///
/// Held ground carries no event. A tile changes hands because the reach of a
/// city moved, and the pass that rewrites the holder column writes one column
/// and no log, so the delta of the held count is the only reader of it.
#[test]
fn the_held_delta_reads_both_directions() {
    let mut memory = EventMemory::new(2);
    memory.note_held(FactionId(0), 40);
    memory.advance();
    assert_eq!(
        memory.total(FactionId(0), MemoryKind::GroundGained, Decay::Recent),
        40
    );
    assert_eq!(
        memory.total(FactionId(0), MemoryKind::OwnGroundLost, Decay::Recent),
        0
    );
    memory.note_held(FactionId(0), 25);
    memory.advance();
    assert_eq!(
        memory.total(FactionId(0), MemoryKind::OwnGroundLost, Decay::Recent),
        15
    );
}

/// Every kind that names a cause holds a pair position of its own.
///
/// The count of the pair positions is derived from the same match that gives
/// each one, so this checks that the match numbers its own answers densely. A
/// gap or a repeat would put two kinds in one pair counter.
#[test]
fn every_pair_position_is_its_own() {
    let mut seen = Vec::new();
    for kind in MemoryKind::ALL {
        let Some(slot) = kind.blame_position() else {
            continue;
        };
        assert!(
            !seen.contains(&slot),
            "{} repeats the pair position {slot}",
            kind.name()
        );
        seen.push(slot);
    }
    seen.sort_unstable();
    assert_eq!(
        seen,
        (0..seen.len()).collect::<Vec<usize>>(),
        "the pair positions must be the whole range below their count"
    );
}

/// The width of each field of the block follows the kind list and nothing
/// else.
///
/// A field whose length or bound followed the world shape would make one
/// policy unable to read two worlds. **The schema cannot follow a world**,
/// because it names no world, and two tests of the layout state that property
/// for the whole array. This test states the property that belongs to this
/// block alone: each of the five widths equals a count of event kinds, so a
/// kind added to the list moves the width with it.
#[test]
fn the_width_of_each_field_follows_the_kind_list() {
    let kinds = MemoryKind::ALL.len();
    let blamed = MemoryKind::ALL
        .iter()
        .filter(|kind| kind.blame_position().is_some())
        .count();
    assert!(
        blamed > 0 && blamed < kinds,
        "the fixture needs a kind list where some kinds name a cause and some \
         do not, because a width that equalled both counts would prove nothing"
    );
    for (name, wanted) in [
        ("memory_recent", kinds),
        ("memory_lasting", kinds),
        ("memory_trend", kinds),
        ("memory_worst_rival", blamed),
        ("memory_concentration", blamed),
    ] {
        assert_eq!(
            row(name).positions as usize,
            wanted,
            "the field `{name}` must hold one position for each kind it covers"
        );
    }
}
