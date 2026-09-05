//! The graded relation between two factions, and what reads it.
//!
//! **The contest resolves a meeting only across a pair at war.** The fixture
//! puts two factions on adjacent tiles with the largest attack the scale holds
//! against no armour, so the only thing that can stop a casualty is the
//! gate.[^1] [^2]
//!
//! **A determinism test cannot tell correct from consistently wrong.** The
//! controller draws once for each faction to decide a relation move, and one
//! test for each field of the key proves the field reaches the draw.[^3]
//!
//! The tests see only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decisions D3, D4, D5 and D6. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::controller::{self, FactionWeights, COMMAND_RELATION, WEIGHT_HIGH, WEIGHT_LOW};
use cachette_core::holding::Holder;
use cachette_core::relation::RelationError;
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, LEADER, WORKER, WORKER_ROW};
use cachette_core::{
    Axial, Entity, FactionId, Fix32, Influence, MoveRelationError, Tick, World, WorldConfig,
};

/// The thread counts that every stepping test in this file runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The seed of a world whose first tiles admit a unit. Each builder asserts
/// that the ground it uses admits one.
const LAND_SEED: u64 = 1;

const A: FactionId = FactionId(0);
const B: FactionId = FactionId(1);

/// Returns a worker row that fights with the given attack and armour.
const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour,
        ..WORKER_ROW
    }
}

/// Builds a world of a few tiles in a row, with two factions.
fn row_world(width: u32, unit_capacity: u32) -> World {
    let world = World::new(WorldConfig {
        width,
        height: 1,
        seed: LAND_SEED,
        faction_count: 2,
        unit_capacity,
    })
    .expect("a row of tiles is a world");
    for q in 0..width as i32 {
        assert!(
            world.admits_a_unit(Axial::new(q, 0)),
            "the fixture needs ground that admits a unit at {q}"
        );
    }
    world
}

/// Spawns units of one faction and one type on one address.
fn spawn(world: &mut World, address: Axial, faction: FactionId, kind: UnitTypeId, count: u32) {
    for _ in 0..count {
        let unit = world
            .spawn_soldier(address, faction)
            .expect("the ground admits a unit and the faction is inside the world");
        assert!(world.set_unit_type(unit, kind), "the unit is alive");
    }
}

/// Puts the pair below the war edge in one direction. The edge is read from
/// the world.
fn declare_war(world: &mut World, from: FactionId, to: FactionId) {
    let war = world.relation_rules().war_edge - 1;
    assert!(world.set_relation(from, to, war));
}

/// Builds the extreme the gate test needs: the largest attack the scale holds
/// on the tile beside a group with no armour. Nothing but the gate can stop a
/// casualty here.
fn maximum_attack_beside_no_armour() -> World {
    let mut world = row_world(2, 64);
    world
        .define_unit_type(0, fighter(Fix32::MAX, Fix32::ZERO))
        .expect("the row is inside the table");
    spawn(&mut world, Axial::new(0, 0), A, UnitTypeId(0), 4);
    spawn(&mut world, Axial::new(1, 0), B, UnitTypeId(0), 4);
    world
}

#[test]
fn the_contest_kills_nobody_at_peace_and_somebody_at_war() {
    for threads in THREAD_COUNTS {
        let mut world = maximum_attack_beside_no_armour();
        assert!(!world.at_war(A, B), "a new world is at peace");
        world.step(threads).expect("the step runs");
        assert!(
            world.fell_log().is_empty(),
            "the contest killed somebody across a pair at peace at {threads} threads"
        );
        assert_eq!(world.population_of(A) + world.population_of(B), 8);

        // The same world, the same tiles, and one direction of the pair now
        // in the war band. The other direction stays at peace, so the test
        // proves either direction is enough.
        declare_war(&mut world, A, B);
        assert!(world.at_war(A, B));
        assert!(world.at_war(B, A), "the war gate reads both directions");
        world.step(threads).expect("the step runs");
        assert!(
            !world.fell_log().is_empty(),
            "the contest killed nobody across a pair at war at {threads} threads"
        );
        assert!(world.check_invariants());
    }
}

#[test]
fn a_fallen_unit_lowers_the_victim_toward_the_killer() {
    // Only one side can reach: the bowman of A carries the attack and the
    // unit of B carries none, so every casualty is a B unit and every
    // grievance names A as the killer.
    let mut world = row_world(2, 64);
    world
        .define_unit_type(0, fighter(Fix32::MAX, Fix32::ZERO))
        .expect("the row is inside the table");
    world
        .define_unit_type(1, fighter(Fix32::ZERO, Fix32::ZERO))
        .expect("the row is inside the table");
    spawn(&mut world, Axial::new(0, 0), A, UnitTypeId(0), 1);
    spawn(&mut world, Axial::new(1, 0), B, UnitTypeId(1), 4);
    declare_war(&mut world, A, B);
    let before_b = world.relation(B, A).expect("the pair exists");
    let before_a = world.relation(A, B).expect("the pair exists");

    world.step(1).expect("the step runs");

    let fell = world.fell_log().len() as i32;
    assert!(fell > 0, "the fixture resolved nothing");
    let step = world.relation_rules().unit_fell;
    assert_eq!(
        world.relation(B, A),
        Some(before_b - step * fell),
        "the victim must cool toward the killer by one step for each unit"
    );
    assert_eq!(
        world.relation(A, B),
        Some(before_a),
        "the killer lost nobody, so its feeling does not move"
    );
}

#[test]
fn drift_returns_a_war_pair_to_peace_inside_a_bounded_tick_count() {
    let mut world = row_world(1, 8);
    let rules = world.relation_rules();
    let period = u64::from(rules.drift_period);
    let start = rules.war_edge - 1;
    assert!(world.set_relation(A, B, start));
    // Each due tick moves one drift step, so the distance to the peace edge
    // in steps, times the period, plus one period for the phase, bounds the
    // wait. The alliance direction has the same bound from the other side.
    let steps = i64::from((rules.peace_edge - start) / rules.drift);
    let bound = (steps as u64 + 1) * period;
    assert!(world.set_relation(B, A, rules.alliance_edge + 1));

    let mut crossings = 0usize;
    for _ in 0..bound {
        world.step(1).expect("the step runs");
        crossings += world.relation_log().len();
    }
    assert_eq!(
        world.relation(A, B),
        Some(rules.peace_edge),
        "the war pair must reach the peace edge and stop there"
    );
    assert_eq!(
        world.relation(B, A),
        Some(rules.alliance_edge - 1),
        "the alliance must drift to the top of the peace band and stop"
    );
    assert_eq!(
        crossings, 1,
        "the drift crossed the war edge once, so the log holds one event"
    );

    // A pair at peace stays where it is however long the drift runs.
    for _ in 0..(2 * period) {
        world.step(1).expect("the step runs");
    }
    assert_eq!(world.relation(A, B), Some(rules.peace_edge));
    assert_eq!(world.relation(B, A), Some(rules.alliance_edge - 1));

    // **A step wider than the distance left must stop at the edge and not
    // cross it.** A step of one can never overshoot, so the default rules
    // cannot reach this case, and a drift that forgot the stop would pass
    // the run above. The wide step is what supplies the extreme.[^1]
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    let mut wide = rules;
    wide.drift = 3;
    world.set_relation_rules(wide);
    assert!(world.set_relation(A, B, wide.peace_edge - 4));
    assert!(world.set_relation(B, A, wide.alliance_edge + 4));
    for _ in 0..(3 * period) {
        world.step(1).expect("the step runs");
    }
    assert_eq!(
        world.relation(A, B),
        Some(wide.peace_edge),
        "a wide drift step must stop at the peace edge"
    );
    assert_eq!(
        world.relation(B, A),
        Some(wide.alliance_edge - 1),
        "a wide drift step must stop at the top of the peace band"
    );
}

#[test]
fn a_crossing_of_the_war_edge_logs_exactly_once() {
    let mut world = row_world(1, 8);
    let rules = world.relation_rules();
    // Down into war: one event. Further down inside war: none. Up to the
    // edge, which is the first value outside war: one event. Up inside
    // peace: none.
    assert!(world.set_relation(A, B, rules.war_edge - 1));
    assert_eq!(world.relation_log().len(), 1);
    let event = world.relation_log()[0];
    assert_eq!((event.from_faction, event.to_faction), (A, B));
    assert!(
        event.band_after < event.band_before,
        "the crossing is a declaration"
    );
    assert_eq!(event.padding, [0; 2]);
    assert!(world.set_relation(A, B, rules.war_edge - 4));
    assert_eq!(
        world.relation_log().len(),
        1,
        "a move inside the war band logs nothing"
    );
    assert!(world.set_relation(A, B, rules.war_edge));
    assert_eq!(
        world.relation_log().len(),
        2,
        "the return to the edge is a crossing"
    );
    assert!(!world.relation_log()[1].is_declaration());
    assert!(world.set_relation(A, B, rules.alliance_edge));
    assert_eq!(
        world.relation_log().len(),
        2,
        "a move inside the other bands logs nothing"
    );

    // The step empties the log, and a frame in which nothing crossed holds
    // none.
    world.step(1).expect("the step runs");
    assert!(world.relation_log().is_empty());
}

#[test]
fn the_band_number_follows_the_edges() {
    let mut world = row_world(1, 8);
    let rules = world.relation_rules();
    for (value, band) in [
        (rules.war_edge - 1, 0u8),
        (rules.war_edge, 1),
        (rules.peace_edge, 2),
        (rules.alliance_edge, 3),
    ] {
        assert!(world.set_relation(A, B, value));
        assert_eq!(world.relation_band(A, B), Some(band), "at {value}");
    }
    assert_eq!(world.relation(A, FactionId(7)), None, "no such faction");
    assert!(
        !world.set_relation(A, A, 3),
        "a faction holds no relation toward itself"
    );
}

#[test]
fn move_relation_reads_the_command_reach_of_the_speaker_and_the_bound() {
    let mut world = row_world(2, 64);
    let bound = world.relation_rules().move_bound;
    let worker = world
        .spawn_soldier(Axial::new(0, 0), A)
        .expect("the ground admits a unit");
    assert!(world.set_unit_type(worker, WORKER));
    let leader = world
        .spawn_soldier(Axial::new(1, 0), A)
        .expect("the ground admits a unit");
    assert!(world.set_unit_type(leader, LEADER));
    let start = world.relation(A, B).expect("the pair exists");

    assert_eq!(
        world.move_relation(worker, B, -1),
        Err(MoveRelationError::Relation(RelationError::NoCommandReach)),
        "a type with a command reach of zero cannot move a relation"
    );
    assert_eq!(
        world.relation(A, B),
        Some(start),
        "a refused move changes nothing"
    );
    assert_eq!(
        world.move_relation(leader, B, bound + 1),
        Err(MoveRelationError::Relation(RelationError::StepAboveBound {
            step: bound + 1,
            bound
        }))
    );
    assert_eq!(
        world.move_relation(leader, B, -(bound + 1)),
        Err(MoveRelationError::Relation(RelationError::StepAboveBound {
            step: -(bound + 1),
            bound
        }))
    );
    assert_eq!(
        world.move_relation(leader, A, -1),
        Err(MoveRelationError::Relation(RelationError::SameFaction))
    );
    assert_eq!(
        world.move_relation(leader, FactionId(9), -1),
        Err(MoveRelationError::Relation(RelationError::NoSuchFaction(9)))
    );
    assert_eq!(world.move_relation(leader, B, -bound), Ok(start - bound));
    assert_eq!(world.relation(A, B), Some(start - bound));
    assert_eq!(world.move_relation(leader, B, bound), Ok(start));

    // A dead speaker is refused, and the identity does not resolve again.
    assert!(world.despawn_soldier(leader));
    assert_eq!(
        world.move_relation(leader, B, -1),
        Err(MoveRelationError::DeadUnit(leader))
    );
}

/// The weights of a faction whose war weight sits in the middle of the range,
/// so the relation draw answers yes on some ticks and no on others.
const fn middling() -> FactionWeights {
    FactionWeights {
        war: (WEIGHT_LOW + WEIGHT_HIGH) / 2,
        trade: WEIGHT_LOW,
        build: WEIGHT_LOW,
        renown: WEIGHT_LOW,
    }
}

/// The sequence of relation draws over a run of ticks, for one faction and
/// one draw index.
fn relation_draws(seed: u64, faction: FactionId, draw: u32, first_tick: u64) -> Vec<bool> {
    (first_tick..first_tick + 64)
        .map(|tick| controller::wants_relation_move(seed, Tick(tick), faction, draw, middling()))
        .collect()
}

#[test]
fn the_tick_is_in_the_relation_draw_key() {
    // A draw keyed on everything but the tick answers the same on every tick.
    let draws = relation_draws(7, A, 2, 1);
    assert!(
        draws.contains(&true),
        "the fixture never says yes: {draws:?}"
    );
    assert!(
        draws.contains(&false),
        "the fixture never says no: {draws:?}"
    );
}

#[test]
fn the_faction_is_in_the_relation_draw_key() {
    assert_ne!(
        relation_draws(7, A, 2, 1),
        relation_draws(7, B, 2, 1),
        "two factions on the same ticks must not draw alike"
    );
}

#[test]
fn the_draw_index_is_in_the_relation_draw_key() {
    assert_ne!(
        relation_draws(7, A, 2, 1),
        relation_draws(7, A, 3, 1),
        "two draw indexes on the same ticks must not draw alike"
    );
}

#[test]
fn the_seed_is_in_the_relation_draw_key() {
    assert_ne!(
        relation_draws(7, A, 2, 1),
        relation_draws(8, A, 2, 1),
        "two seeds on the same ticks must not draw alike"
    );
}

#[test]
fn the_war_weight_biases_the_relation_draw() {
    // A war weight at the bottom of the range says yes less often than one at
    // the top, over the same ticks and the same key. This is what makes the
    // weight a weight.
    let count = |war: u8| {
        (1..=256u64)
            .filter(|tick| {
                controller::wants_relation_move(
                    7,
                    Tick(*tick),
                    A,
                    2,
                    FactionWeights { war, ..middling() },
                )
            })
            .count()
    };
    assert!(count(WEIGHT_LOW) < count(WEIGHT_HIGH));
}

#[test]
fn the_rival_is_the_largest_other_faction_and_a_tie_goes_low() {
    let held = [(FactionId(0), 5i64), (FactionId(1), 9), (FactionId(2), 9)];
    assert_eq!(
        controller::rival_of(FactionId(0), held.iter().copied()),
        Some(FactionId(1))
    );
    assert_eq!(
        controller::rival_of(FactionId(1), held.iter().copied()),
        Some(FactionId(2))
    );
    assert_eq!(
        controller::rival_of(FactionId(0), [(FactionId(0), 5i64)].into_iter()),
        None,
        "a faction alone in the world has no rival"
    );
}

#[test]
fn the_controller_moves_a_relation_through_the_verb() {
    // The world seeds itself, so every faction has a seat. Every unit of the
    // first faction becomes a leader, so the faction has a speaker. The other
    // faction keeps workers and has none, so it plans no move and the verb
    // is never asked to refuse it.
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 3,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world.seed_world().expect("the world seeds once");
    let units: Vec<Entity> = world.soldiers().iter().collect();
    for unit in units {
        if world.soldiers().faction(unit) == Some(A) {
            assert!(world.set_unit_type(unit, LEADER));
        }
    }
    let start = world.relation(A, B).expect("the pair exists");
    let mut moves = 0i64;
    let mut by_b = 0usize;
    for _ in 0..200 {
        world.step(4).expect("the step runs");
        moves += world
            .subsystem_census()
            .iter()
            .find(|(name, _)| *name == "relation_moves")
            .map_or(0, |(_, count)| *count);
        by_b += world
            .controller_log()
            .iter()
            .filter(|command| command.kind == COMMAND_RELATION && command.faction == B)
            .count();
    }
    assert!(
        moves > 0,
        "the controller never moved a relation in 200 ticks"
    );
    assert_eq!(by_b, 0, "a faction with no speaker plans no move");
    assert!(
        world.relation(A, B).expect("the pair exists") <= start,
        "the moves go toward war and the drift cannot outrun them here"
    );
    assert_eq!(
        world.relation(B, A),
        Some(start),
        "the faction with no speaker moved nothing"
    );
}

#[test]
fn a_holder_below_the_guest_edge_refuses_a_guest() {
    // Two tiles in a row. B stands on the second and holds it after one
    // step. A stands on the first, chooses on every tick, and the only
    // neighbour inside the world is the tile B holds. When B is below the
    // guest edge toward A, A never arrives. When B is not, A arrives.
    const FRAMES: u32 = 64;
    for refuse in [true, false] {
        let mut world = row_world(2, 8);
        world
            .spawn_soldier(Axial::new(1, 0), B)
            .expect("the ground admits a unit");
        world.step(1).expect("the step runs");
        assert_eq!(
            world
                .tile_holder(Axial::new(1, 0))
                .and_then(Holder::faction),
            Some(B),
            "the fixture did not give the second tile to B"
        );
        if refuse {
            // The drift moves the pair one step toward peace on each due
            // tick, so a value one below the edge would reach the edge inside
            // the run and the refusal would end. The fixture sits far enough
            // below that the drift cannot reach the edge in the frames it
            // runs, and the assertion below proves it did not.
            let rules = world.relation_rules();
            let drifts = i32::try_from(u64::from(FRAMES) / u64::from(rules.drift_period) + 1)
                .expect("the count is small");
            let below = rules.guest_edge - 1 - rules.drift * drifts;
            assert!(world.set_relation(B, A, below));
        }
        // The guest arrives after B holds the tile, so the first step above
        // cannot hand the tile to the guest instead.
        world
            .set_choice_schedule(0)
            .expect("the exponent is inside the range");
        let guest = world
            .spawn_soldier(Axial::new(0, 0), A)
            .expect("the ground admits a unit");
        let held = world
            .grid()
            .index_of(Axial::new(1, 0))
            .expect("the address is inside the world");
        let mut arrived = false;
        for _ in 0..FRAMES {
            world.step(1).expect("the step runs");
            if world.soldiers().tile(guest) == Some(held) {
                arrived = true;
                break;
            }
        }
        assert_eq!(
            arrived, !refuse,
            "refuse={refuse}: the guest must arrive exactly when the holder permits it"
        );
        if refuse {
            assert!(
                world.relation(B, A).expect("the pair exists") < world.relation_rules().guest_edge,
                "the drift reached the edge inside the run, so the fixture measured the drift"
            );
        }
    }
}

#[test]
fn a_leader_at_peace_converts_nobody() {
    // The same shape as the conversion fixture: units of A stand where B
    // injects one reference unit of influence. At peace nobody converts, and
    // one step below the conversion edge somebody does.
    const EDGE: u32 = 128;
    let build = || {
        let mut world = World::new(WorldConfig {
            width: EDGE,
            height: EDGE,
            seed: 0x0cac_4e77_0132,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .expect("the extent must describe a world");
        let seat = (16..(EDGE as i32 - 16))
            .flat_map(|r| (16..(EDGE as i32 - 16)).map(move |q| Axial::new(q, r)))
            .find(|here| {
                (-6..=6).all(|dq| {
                    (-6..=6).all(|dr| world.admits_a_unit(Axial::new(here.q + dq, here.r + dr)))
                })
            })
            .expect("the world holds open ground");
        for offset in 0..6 {
            world
                .spawn_soldier(Axial::new(seat.q + offset, seat.r), A)
                .expect("the fixture must place a unit");
        }
        assert!(world.set_influence_source(B, seat, Influence::UNIT));
        world
    };

    let mut at_peace = build();
    let mut converted = 0usize;
    for _ in 0..8 {
        at_peace.step(2).expect("the step runs");
        converted += at_peace.converted_log().len();
    }
    assert_eq!(converted, 0, "a leader at peace converted somebody");

    let mut in_tension = build();
    let edge = in_tension.relation_rules().conversion_edge;
    assert!(in_tension.set_relation(B, A, edge - 1));
    let mut converted = 0usize;
    for _ in 0..8 {
        in_tension.step(2).expect("the step runs");
        converted += in_tension.converted_log().len();
    }
    assert!(
        converted > 0,
        "the fixture converts nobody even below the edge"
    );
    let step = in_tension.relation_rules().unit_converted;
    assert_eq!(
        in_tension.relation(A, B),
        Some(in_tension.relation_rules().peace_edge - step * converted as i32),
        "the faction that lost units cools toward the leader by one step each"
    );
}

#[test]
fn the_relation_enters_the_state_hash() {
    let mut one = row_world(1, 8);
    let mut other = row_world(1, 8);
    assert_eq!(one.state_hash().finish(), other.state_hash().finish());
    assert!(other.set_relation(A, B, 1));
    assert_ne!(
        one.state_hash().finish(),
        other.state_hash().finish(),
        "two worlds that differ in a relation must hash apart"
    );
    let mut rules = one.relation_rules();
    rules.war_edge -= 1;
    one.set_relation_rules(rules);
    let moved = one.state_hash().finish();
    one.set_relation_rules(other.relation_rules());
    assert_ne!(moved, one.state_hash().finish(), "the edges enter the hash");
}

#[test]
fn an_offer_and_a_counter_are_refused_across_a_pair_at_war() {
    // The war check runs before the presence check and before the search for
    // a live negotiation, so a world with no presence and nothing open answers
    // the war and nothing else. At peace the same calls fall through to the
    // refusal that comes next, which proves the gate is the war.
    use cachette_core::trade::{Consideration, TradeError};
    let mut world = row_world(2, 8);
    let give = Consideration::resource(0, 1);
    let take = Consideration::resource(1, 1);
    assert_eq!(
        world.offer_consideration(A, B, give.clone(), take.clone(), 4),
        Err(TradeError::NoPresence),
        "at peace the offer reaches the presence check"
    );
    assert_eq!(
        world.counter_consideration(A, B, give.clone(), take.clone()),
        Err(TradeError::NothingOpen),
        "at peace the counter reaches the search for a negotiation"
    );
    // One direction at war is enough, and the direction that is at peace is
    // refused too.
    declare_war(&mut world, B, A);
    assert_eq!(
        world.offer_consideration(A, B, give.clone(), take.clone(), 4),
        Err(TradeError::AtWar)
    );
    assert_eq!(
        world.offer_consideration(B, A, give.clone(), take.clone(), 4),
        Err(TradeError::AtWar)
    );
    assert_eq!(
        world.counter_consideration(A, B, give, take),
        Err(TradeError::AtWar)
    );
}
