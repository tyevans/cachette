//! A unit changes faction.
//!
//! Conversion changes the faction of a unit outright. The unit keeps its
//! identity, so every structure that names it still names it, and every total
//! that follows the faction must move with it.[^1]
//!
//! The tests drive the engine. The step is obligated to run the pass, so a
//! test that called the pass itself would prove the pass works and not that
//! anything reaches it.[^2]
//!
//! **The fixture is built for the case, not for a plausible world.** The
//! influence source sits at one reference unit at the cell the units stand
//! in, and the units belong to the faction with no source at all. That is the
//! largest margin the field can produce, so the pass converts a whole group
//! rather than a fraction of one. A fixture with two weak sources would
//! convert nobody, and every assertion below would then measure the
//! fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D1. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
//! [^2]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::conversion::{converts, remainder_draw, rotation_offset, DrawKey};
use cachette_core::types::FACTION_CEILING;
use cachette_core::{
    Axial, Entity, FactionId, Holder, Influence, Tick, TileIdx, World, WorldConfig,
};

/// The extent of every world here.
///
/// It covers more than one level 1 cell, so the cell that carries the source
/// is not the whole world.
const EDGE: u32 = 256;

/// The faction that holds no source. Its units are the ones that convert.
const OLD: FactionId = FactionId(0);

/// The faction that holds the source. Its belief takes the units.
const NEW: FactionId = FactionId(1);

/// The corner that the search for open ground starts at.
///
/// The generator makes water, so no fixed address is certain to admit a unit.
/// The fixture searches instead, and a search that started at the origin
/// would sit on the edge of the world where a neighbour is missing.
const SEARCH_ORIGIN: i32 = 32;

/// The corner that a search for a second, distant place starts at.
///
/// It is far from the first origin, so a unit placed there stands outside
/// every holding that a fixture builds at the first one.
const DISTANT_ORIGIN: i32 = 160;

/// The reach of the patch of ground that the presence fixture fills.
///
/// A holding must be wide enough that its middle tile has six neighbours
/// that hold with it, because a challenger on the edge of a holding meets
/// fewer.
const PATCH: i32 = 4;

/// How many units the fixture puts on the ground it finds.
const UNITS: u32 = 6;

/// How many steps the fixture runs before it reads the outcome.
///
/// The solve runs a fixed pass count on every frame, so the field climbs over
/// several frames. The count is generous rather than tuned, because a test
/// that depended on the exact count would be a test of the count.
const STEPS: usize = 12;

/// Builds a world that holds two factions.
fn world(seed: u64) -> World {
    World::new(WorldConfig {
        width: EDGE,
        height: EDGE,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world")
}

/// Returns the first address whose square of the given reach admits a unit
/// everywhere.
///
/// The ground is generated, so the fixture cannot name a tile and assume it
/// is open. It searches, and it fails loudly when the world holds no such
/// place.
fn open_ground(world: &World, reach: i32) -> Axial {
    open_ground_from(world, SEARCH_ORIGIN, reach)
}

/// Returns the first open address at or after one origin.
///
/// A test that wants two places far apart names two origins. Two calls to one
/// search would return one address twice.
fn open_ground_from(world: &World, origin: i32, reach: i32) -> Axial {
    let limit = EDGE as i32 - reach - 1;
    for r in origin..limit {
        for q in origin..limit {
            let here = Axial::new(q, r);
            let open = (-reach..=reach).all(|dq| {
                (-reach..=reach).all(|dr| world.admits_a_unit(Axial::new(q + dq, r + dr)))
            });
            if open {
                return here;
            }
        }
    }
    panic!("the world holds no open ground of reach {reach}");
}

/// Builds a world whose units all belong to the old faction, and whose new
/// faction injects one reference unit of influence where they stand.
fn believers(seed: u64) -> (World, Axial, Vec<Entity>) {
    let mut world = world(seed);
    let seat = open_ground(&world, UNITS as i32);
    let mut units = Vec::new();
    for offset in 0..UNITS {
        let address = Axial::new(seat.q + offset as i32, seat.r);
        units.push(
            world
                .spawn_soldier(address, OLD)
                .expect("the fixture must place a unit"),
        );
    }
    assert!(world.set_influence_source(NEW, seat, Influence::UNIT));
    // A leader at peace with a faction converts none of its units, so the
    // fixture puts the leader in tension toward the old faction. The edge is
    // read from the world and not restated here.[^1]
    //
    // [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    let tension = world.relation_rules().peace_edge - 1;
    assert!(world.set_relation(NEW, OLD, tension));
    (world, seat, units)
}

/// Returns the live population of every faction of the world.
fn population(world: &World) -> (u32, u32) {
    (world.population_of(OLD), world.population_of(NEW))
}

#[test]
fn the_field_takes_the_units_and_every_faction_total_follows() {
    let (mut world, _seat, units) = believers(0x0cac_4e77_0132);
    let (before_old, before_new) = population(&world);
    assert_eq!(before_old, UNITS);
    assert_eq!(before_new, 0);

    let mut converted = 0usize;
    for _ in 0..STEPS {
        world.step(4).expect("the step must run");
        converted += world.converted_log().len();
        // The arena keeps one count for each faction, and this check recounts
        // and compares. A pass that moved a unit and left the count alone
        // fails here.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        assert!(
            world.check_invariants(),
            "the world broke an invariant while the field converted units"
        );
        assert!(
            world.cohorts_describe_the_units(),
            "the cohorts stopped describing the units"
        );
    }

    let (after_old, after_new) = population(&world);
    assert!(converted > 0, "the field converted nobody");
    assert!(after_new > 0, "the new faction gained nobody");
    assert!(after_old < before_old, "the old faction lost nobody");
    assert_eq!(
        after_old + after_new,
        before_old + before_new,
        "conversion changed how many units are alive"
    );
    for unit in &units {
        assert!(
            world.soldier_faction(*unit).is_some(),
            "a converted unit lost its identity"
        );
    }
}

#[test]
fn a_converted_unit_keeps_its_identity_and_loses_its_orders() {
    let (mut world, _seat, units) = believers(0x0cac_4e77_0133);
    let first = units[0];
    assert!(world.order_gather(first, cachette_core::ResourceKind::Wood));
    let carried = world.soldier_carry(first);

    world
        .convert_units(&units, NEW)
        .expect("the set names live units of a faction the world holds");

    assert_eq!(world.soldier_faction(first), Some(NEW));
    assert_eq!(
        world.gather_order(first),
        Some(None),
        "the unit kept an order that the old faction gave it"
    );
    assert_eq!(
        world.soldier_carry(first),
        carried,
        "the unit lost the load it was carrying"
    );
    assert_eq!(world.population_of(NEW), UNITS);
    assert_eq!(world.population_of(OLD), 0);
    assert!(world.check_invariants());
}

#[test]
fn the_verb_is_all_or_nothing() {
    let (mut world, _seat, units) = believers(0x0cac_4e77_0134);
    let dead = units[0];
    assert!(world.despawn_soldier(dead));

    let refused = world.convert_units(&units, NEW);
    assert!(refused.is_err(), "the verb took a set holding a dead unit");
    assert_eq!(
        world.population_of(NEW),
        0,
        "the verb changed a unit before it refused the set"
    );

    let refused = world.convert_units(&units[1..], FactionId(9));
    assert!(refused.is_err(), "the verb took a faction of no world");
    assert_eq!(world.population_of(NEW), 0);
}

#[test]
fn the_verb_converts_nobody_twice() {
    let (mut world, _seat, units) = believers(0x0cac_4e77_0135);
    world.convert_units(&units, NEW).expect("the set is live");
    assert_eq!(world.population_of(NEW), UNITS);

    world
        .convert_units(&units, NEW)
        .expect("the set is still live");
    assert_eq!(
        world.population_of(NEW),
        UNITS,
        "the second call moved the count again"
    );
    assert!(world.check_invariants());
}

#[test]
fn the_presence_relation_stops_calling_a_convert_a_foreigner() {
    // The relation names the factions that stand on the ground of another
    // faction. A unit that changed hands stands on its own ground now, so the
    // bit that named it must clear.
    //
    // The old faction keeps a second unit far from the holding, so the
    // relation has somebody left to report. A test whose only unit of the old
    // faction converted would assert against a faction with no units, and it
    // would pass whatever the relation did.
    let mut world = world(0x0cac_4e77_0136);
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    let corner = open_ground(&world, PATCH);
    for row in -PATCH..=PATCH {
        for column in -PATCH..=PATCH {
            let address = Axial::new(corner.q + column, corner.r + row);
            let _ = world.spawn_soldier(address, NEW);
        }
    }
    for _ in 0..6 {
        world.step(4).expect("the step must run");
    }
    assert_eq!(
        world.tile_holder(corner).and_then(Holder::faction),
        Some(NEW),
        "the fixture must leave the new faction holding the tile the guest visits"
    );

    // The holder of a tile raises seven for each of its own units standing
    // there, so a lone guest cannot take the tile from three of them.
    for _ in 0..3 {
        world
            .spawn_soldier(corner, NEW)
            .expect("the ground admits a unit");
    }
    let guest = world
        .spawn_soldier(corner, OLD)
        .expect("the ground admits a unit");
    let far = open_ground_from(&world, DISTANT_ORIGIN, 1);
    let distant = world
        .spawn_soldier(far, OLD)
        .expect("the ground admits a unit");
    assert_ne!(far, corner, "the distant unit stands inside the holding");

    world.step(4).expect("the step must run");
    assert!(
        world
            .stands_in_territory(OLD, NEW)
            .expect("the relation describes the arena"),
        "the fixture never put the guest on the ground of the other faction"
    );

    world
        .convert_units(&[guest], NEW)
        .expect("the guest is alive");
    // The relation was folded before the conversion, so it now describes an
    // arena that has moved on. It must refuse rather than answer, because an
    // answer that states it is fresh and is not is worse than a
    // refusal.[^1]
    //
    // [^1]: Findings register, FND-433. `docs/FINDINGS.md`
    assert!(
        world.stands_in_territory(OLD, NEW).is_err(),
        "the relation answered from an arena that a conversion had moved on"
    );
    world.step(4).expect("the step must run");
    assert_eq!(
        world.soldier_faction(guest),
        Some(NEW),
        "the guest did not change faction"
    );
    assert!(
        world.soldier_faction(distant) == Some(OLD),
        "the old faction lost the unit that was far from the holding"
    );
    assert!(
        !world
            .stands_in_territory(OLD, NEW)
            .expect("the relation describes the arena"),
        "the relation still calls the convert a foreigner"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_converted_unit_does_not_flip_back_while_the_field_stands_still() {
    // Strict dominance is what stops the loop. After the change the leading
    // faction is the unit's own, so the margin against it is zero.
    let (mut world, _seat, _units) = believers(0x0cac_4e77_0137);
    let mut ever_left_the_new_faction = false;
    for _ in 0..(STEPS * 4) {
        world.step(4).expect("the step must run");
        for event in world.converted_log() {
            if event.from == NEW {
                ever_left_the_new_faction = true;
            }
        }
    }
    assert_eq!(
        world.population_of(OLD),
        0,
        "the field left some unit with the old faction"
    );
    assert!(
        !ever_left_the_new_faction,
        "a unit converted away from the faction that leads the field"
    );
    // The pass converts nobody once the world has settled, so the log is
    // empty on the frames that follow.
    world.step(4).expect("the step must run");
    assert!(
        world.converted_log().is_empty(),
        "the pass converted somebody in a settled world"
    );
}

#[test]
fn every_field_of_the_draw_key_reaches_the_draw() {
    // A draw keyed on the wrong field gives the same wrong answer on every
    // run, and neither determinism test can see it. This asserts that each
    // field changes the value.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let key = DrawKey {
        seed: 0x0cac_4e77_0138,
        tick: Tick(7),
    };
    let tile = TileIdx(4321);
    let base = remainder_draw(key, tile, OLD);

    let other_seed = DrawKey {
        seed: key.seed ^ 1,
        tick: key.tick,
    };
    assert_ne!(base, remainder_draw(other_seed, tile, OLD), "the seed");

    let other_tick = DrawKey {
        seed: key.seed,
        tick: Tick(key.tick.0 + 1),
    };
    assert_ne!(base, remainder_draw(other_tick, tile, OLD), "the frame");

    assert_ne!(
        base,
        remainder_draw(key, TileIdx(tile.0 + 1), OLD),
        "the tile"
    );
    assert_ne!(base, remainder_draw(key, tile, NEW), "the group");
    assert_ne!(
        base,
        rotation_offset(key, tile, OLD, u32::from(Influence::UNIT.0)),
        "the two draws of one group share a value"
    );
}

#[test]
fn the_rotation_never_leaves_the_group() {
    let key = DrawKey {
        seed: 0x0cac_4e77_0139,
        tick: Tick(3),
    };
    for count in [1u32, 2, 7, 64, 1000] {
        for faction in 0..FACTION_CEILING {
            let offset = rotation_offset(key, TileIdx(11), FactionId(faction), count);
            assert!(offset < u64::from(count));
        }
    }
}

#[test]
fn the_margin_alone_decides_how_many_convert() {
    // The count is exact arithmetic on the margin. The draw names which units
    // convert, and it decides at most one more.
    let half = Influence::UNIT.0 / 2;
    assert_eq!(converts(Influence::UNIT.0, 100, 0), 100);
    assert_eq!(converts(0, 100, 0), 0);
    let low = converts(half, 100, u64::from(Influence::UNIT.0) - 1);
    let high = converts(half, 100, 0);
    assert!(low == 49 || low == 50, "the floor moved: {low}");
    assert!(high - low <= 1, "the remainder converted more than one");
}
