//! A learner acts by one integer, and the engine says which integers are
//! legal.
//!
//! # What these tests hold
//!
//! An accepted record says that an action is one integer that indexes a
//! bounded table the engine declares, that the engine answers which rows are
//! legal at this tick, and that one log records the choice of the built-in
//! controller and the action of a learner in one encoding.[^1] [^2] [^3] A
//! second record says that the integer is a mixed radix over the argument
//! positions each verb declares.[^4]
//!
//! These tests drive the engine through its public interface. A learner acts
//! by one integer, and the world changes.
//!
//! # The agreement test
//!
//! The record forbids a row that the answer allows and the verb then
//! refuses.[^2] The verb that takes an action does not read the answer
//! first, so the two can disagree, and one test compares them over a seed
//! set. A fixture that models the typical case supplies no refused row, so
//! the world below runs long enough for the factions to seat, to build, to
//! trade and to march.[^5]
//!
//! # References
//!
//! [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^4]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decisions D1 to D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
//! [^5]: Testing Rules, sections 2 and 2a. `.agents/rules/testing.md`

use cachette_core::action::{ActionSchema, ActionShape, CandidateKind, Verb};
use cachette_core::faction_view::Sighting;
use cachette_core::{Axial, FactionId, World, WorldConfig};

const THREADS: usize = 2;

/// The people each founding settles. Small, so a step is cheap.
const GROUP: u32 = 8;

/// Builds a world of one shape, one faction count and one seed.
fn config(factions: u16, seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Founds one group for each faction, at places the founding survey accepts.
///
/// The seats are kept apart on the first pass, so that the survey does not
/// refuse the second for the first. The second pass takes any place the
/// survey accepts.
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

/// Runs a world far enough that the factions hold sites, stores and units.
fn warm(factions: u16, seed: u64, ticks: u32) -> World {
    let mut world = World::new(config(factions, seed)).expect("the extent describes a world");
    seat(&mut world, factions);
    for _ in 0..ticks {
        world.step(THREADS).expect("the step runs");
    }
    world
}

#[test]
fn the_table_holds_one_row_for_a_verb_that_declares_no_position() {
    let schema = ActionSchema::of(ActionShape { faction_count: 4 });
    for row in schema.rows() {
        if row.positions.is_empty() {
            assert_eq!(
                row.rows,
                1,
                "the verb {} declares no position, so it holds one row",
                row.name()
            );
        }
    }
    // The no-op is row zero, and it declares no position.
    let no_op = schema.row(Verb::NoOp).expect("the table holds the no-op");
    assert_eq!(no_op.first, 0);
    assert_eq!(no_op.rows, 1);
    assert!(no_op.positions.is_empty());
}

#[test]
fn no_bound_follows_the_population() {
    // Two worlds of one faction count and two unit capacities declare one
    // table. A bound that followed the population would move.
    let small = ActionSchema::of(ActionShape { faction_count: 3 });
    let same = ActionSchema::of(ActionShape { faction_count: 3 });
    assert_eq!(small, same);
    // The faction count is the one world parameter the table reads, so a
    // world of more factions holds a longer table.
    let wide = ActionSchema::of(ActionShape { faction_count: 9 });
    assert_eq!(wide.length(), small.length() + 6);
    for row in wide.rows() {
        for position in &row.positions {
            assert!(
                position.bound > 0,
                "the position of {} has no bound",
                row.name()
            );
            if position.candidate == CandidateKind::Faction {
                assert_eq!(position.bound, 9);
            }
        }
    }
}

#[test]
fn every_action_encodes_and_decodes_back_to_itself() {
    // The round trip is arithmetic over the schema alone. It reads no world
    // and no table of its own.
    for factions in [1u32, 2, 5, 17] {
        let schema = ActionSchema::of(ActionShape {
            faction_count: factions,
        });
        assert!(schema.length() > 0);
        for action in 0..schema.length() {
            let (verb, arguments) = schema
                .decode(action)
                .unwrap_or_else(|| panic!("the table holds the row {action}"));
            let row = schema.row(verb).expect("the schema holds the verb");
            assert_eq!(
                arguments.len(),
                row.positions.len(),
                "the verb {} declares {} positions",
                row.name(),
                row.positions.len()
            );
            for (position, argument) in row.positions.iter().zip(&arguments) {
                assert!(
                    *argument < position.bound,
                    "the argument {argument} of {} is at or above its bound {}",
                    row.name(),
                    position.bound
                );
            }
            let again = schema
                .encode(verb, &arguments)
                .expect("the arguments are inside their bounds");
            assert_eq!(again, action, "the round trip must give one integer back");
        }
        // A row above the table decodes to nothing.
        assert!(schema.decode(schema.length()).is_none());
    }
}

#[test]
fn a_learner_acts_by_one_integer_and_the_world_changes() {
    let mut world = warm(2, 41, 6);
    let schema = world.action_schema();
    // The gather verb takes one resource kind. The learner sends one
    // integer, and the units of its faction take the order.
    let action = schema
        .encode(Verb::Gather, &[1])
        .expect("the gather verb takes one kind");
    let before = world.controller_log().len();
    let applied = world.act(FactionId(0), action);
    assert!(applied, "a faction with live units takes a gather order");
    let log = world.controller_log();
    assert_eq!(
        log.len(),
        before + 1,
        "the engine writes one row for the action"
    );
    let row = log.last().expect("the row is written");
    // **The row carries the whole action.** No field of it lives elsewhere.
    assert_eq!(row.action, action);
    assert_eq!(row.faction, FactionId(0));
    assert_eq!(row.applied, 1);
    assert_eq!(row.padding, [0; 5]);
    assert_eq!(
        schema.decode(row.action),
        Some((Verb::Gather, vec![1])),
        "the log row decodes back to the verb and the argument"
    );
}

#[test]
fn the_no_op_is_always_legal_and_changes_nothing() {
    let mut world = warm(3, 7, 4);
    for number in 0..3u16 {
        let answer = world
            .legal_actions(FactionId(number))
            .expect("the faction is of this world");
        assert_eq!(answer[0], 1, "the no-op is always legal");
        assert!(answer.iter().any(|byte| *byte == 1));
    }
    let hash = world.state_hash();
    assert!(world.act(FactionId(0), 0), "the no-op is taken");
    assert_eq!(
        world.state_hash(),
        hash,
        "the no-op changes no simulated state"
    );
}

#[test]
fn the_answer_names_one_byte_for_each_row() {
    let world = warm(4, 19, 3);
    let schema = world.action_schema();
    let answer = world
        .legal_actions(FactionId(1))
        .expect("the faction is of this world");
    assert_eq!(answer.len(), schema.length() as usize);
    assert!(answer.iter().all(|byte| *byte <= 1));
    // A faction the world does not hold gets no answer.
    assert!(world.legal_actions(FactionId(9)).is_none());
}

#[test]
fn a_relation_row_against_the_acting_faction_is_never_legal() {
    // The relation position runs over every faction of the world, and the
    // acting faction is a padding row of that list. A padding row reads as
    // not legal.
    let world = warm(3, 23, 5);
    let schema = world.action_schema();
    let answer = world
        .legal_actions(FactionId(2))
        .expect("the faction is of this world");
    let own = schema
        .encode(Verb::Relation, &[2])
        .expect("the relation verb takes one faction");
    assert_eq!(
        answer[own as usize], 0,
        "a faction may not move its relation toward itself"
    );
}

#[test]
fn the_answer_and_the_verb_agree_over_a_seed_set() {
    // **This is the test the record asks for.** A row the answer allows and
    // the verb then refuses is a defect.[^1]
    //
    // The fixture runs several seeds and several ticks, so the world offers
    // refused rows as well as taken ones. A world that only ever answered
    // yes would measure the fixture rather than the rule.
    //
    // [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    let mut allowed = 0u32;
    let mut refused = 0u32;
    for seed in [3u64, 11, 29, 47, 101] {
        for ticks in [0u32, 2, 9] {
            let base = warm(2, seed, ticks);
            let schema = base.action_schema();
            for faction in 0..2u16 {
                let answer = base
                    .legal_actions(FactionId(faction))
                    .expect("the faction is of this world");
                for action in 0..schema.length() {
                    // Each row runs on its own copy of the world, because a
                    // verb that acts changes what the next row would answer.
                    let mut world = base.clone();
                    let took = world.act(FactionId(faction), action);
                    let says = answer[action as usize] == 1;
                    if says {
                        allowed += 1;
                    } else {
                        refused += 1;
                    }
                    assert_eq!(
                        says,
                        took,
                        "seed {seed}, tick {ticks}, faction {faction}, action {action}: \
                         the answer said {says} and the verb did {took} for the verb {:?}",
                        schema.verb_of(action)
                    );
                }
            }
        }
    }
    // The fixture must supply both answers, or the assertion above never
    // meets the case it guards.
    assert!(allowed > 0, "the fixture supplied no legal row");
    assert!(refused > 0, "the fixture supplied no refused row");
}

#[test]
fn the_answer_names_no_campaign_objective_the_faction_cannot_see() {
    // **The fixture supplies the case.** Two factions sit far apart and go
    // to war without meeting. Neither has observed a settlement of the
    // other, so neither may march, whatever stands in the world.
    //
    // Put the defect back: let the objective reader take every settlement
    // rather than the observed ones, and this test fails.[^1]
    //
    // [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
    let mut world = warm(2, 61, 2);
    let war = world.relation_rules().war_edge - 8;
    assert!(world.set_relation(FactionId(0), FactionId(1), war));
    assert!(world.set_relation(FactionId(1), FactionId(0), war));
    assert!(world.at_war(FactionId(0), FactionId(1)));

    // The world does hold a rival settlement, so the answer is not zero for
    // want of a target.
    let rival_seat = world
        .seat(FactionId(1))
        .expect("the rival holds a seat to march on");
    let rival_place = world
        .grid()
        .address_of(rival_seat)
        .expect("the seat is inside the world");

    // Faction zero has never seen it.
    assert_eq!(
        world
            .faction_tile(FactionId(0), rival_place)
            .map(|seen| seen.sighting()),
        Some(Sighting::Never),
        "the fixture must hide the rival seat from faction 0"
    );

    let schema = world.action_schema();
    let campaign = schema
        .encode(Verb::Campaign, &[])
        .expect("the campaign verb takes no argument");
    let answer = world
        .legal_actions(FactionId(0))
        .expect("the faction is of this world");
    assert_eq!(
        answer[campaign as usize], 0,
        "the answer named an objective the faction has never seen"
    );
    // The verb agrees, so the two do not disagree in the other direction.
    assert!(!world.act(FactionId(0), campaign));
}
