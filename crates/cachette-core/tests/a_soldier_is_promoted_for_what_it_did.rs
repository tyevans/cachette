//! Promotion of a soldier into the character tier.
//!
//! The tests drive the engine and then read the result. A pass that only a
//! test calls proves that the pass works and not that the step reaches it.[^1]
//!
//! Each test states what the promotion depends on, and not only that it
//! repeats. A pass that promoted the wrong unit repeats perfectly.[^2]
//!
//! **The rank fixture separates three orders that usually agree.** A unit
//! spawned earlier holds a lower slot and a lower identity, and a unit that
//! gathered longer holds larger deeds. A fixture that let all three agree
//! could not tell the rank from the read order, and that defect has been paid
//! for once already.[^3] [^4]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^4]: Findings register, FND-270. `docs/FINDINGS.md`

use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// A world large enough to hold a deposit worth gathering.
const CONFIG: WorldConfig = WorldConfig {
    width: 64,
    height: 64,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The deeds that make a unit eligible in these fixtures.
///
/// **The tests state their own threshold rather than taking the default.** A
/// test that took the default would change meaning whenever a content
/// parameter moved, and the richest open tile of this world carries far less
/// than the default asks for.
const THRESHOLD: u64 = 8;

/// Builds a world that promotes on every tick, at a threshold this world can
/// reach.
///
/// The period is one, so a fixture does not have to count frames to reach the
/// pass. The interval is a parameter, and one test below changes it.
fn world() -> World {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_character_schedule(1, 0)
        .expect("the period is inside the range");
    world.set_deed_threshold(THRESHOLD);
    world
}

/// Returns the open address that carries the most food.
///
/// The fixture asserts that it found food worth gathering. A deposit is small
/// and it recovers, so a unit reaches the threshold by returning to it over
/// several frames rather than by emptying it once.
fn deposit(world: &World) -> Axial {
    let grid = world.grid();
    let mut best: Option<(u32, Axial)> = None;
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        let stock = world
            .tile_stock(address, ResourceKind::Food)
            .unwrap_or(cachette_core::resource::Amount::ZERO);
        if best.is_none_or(|(most, _)| stock.0 > most) {
            best = Some((stock.0, address));
        }
    }
    let (most, address) = best.expect("the world holds an open tile");
    assert!(
        most > 0,
        "the fixture found no food, so nothing below could gather"
    );
    address
}

/// Returns the open addresses that carry food, richest first.
///
/// The order breaks ties on the address, so the list is the same on every
/// run.
fn deposits(world: &World) -> Vec<(u32, Axial)> {
    let grid = world.grid();
    let mut found: Vec<(u32, Axial)> = Vec::new();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        let stock = world
            .tile_stock(address, ResourceKind::Food)
            .unwrap_or(cachette_core::resource::Amount::ZERO);
        if stock.0 > 0 {
            found.push((stock.0, address));
        }
    }
    found.sort_by_key(|(stock, address)| (u32::MAX - stock, address.q, address.r));
    found
}

/// Spawns a unit at an address and orders it to gather food.
fn gatherer(world: &mut World, at: Axial) -> Entity {
    let unit = world
        .spawn_soldier(at, FactionId(0))
        .expect("the tile must admit the unit");
    assert!(
        world.order_gather(unit, ResourceKind::Food),
        "the unit must take the order"
    );
    unit
}

/// Steps the world, asserting the invariants at each frame.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world lost an invariant");
    }
}

#[test]
fn a_unit_that_gathers_enough_becomes_a_character() {
    let mut world = world();
    let at = deposit(&world);
    let unit = gatherer(&mut world, at);
    assert_eq!(world.characters().len(), 0, "the world starts with nobody");

    run(&mut world, 40, 1);

    let deeds = world.unit_deeds(unit).expect("the unit must live");
    assert!(
        deeds >= world.deed_threshold(),
        "the fixture must gather past the threshold, and it reached {deeds}"
    );
    assert_eq!(world.characters().len(), 1, "the unit must become somebody");
    let character = world
        .unit_character(unit)
        .expect("the unit must live")
        .expect("the unit must carry a character");
    assert!(
        world.characters().contains(character),
        "the character must be one the arena holds"
    );
}

#[test]
fn a_unit_below_the_threshold_is_promoted_by_nobody() {
    let mut world = world();
    // The threshold is raised past anything this fixture can reach, so the
    // test names a unit that gathered and was still refused.
    world.set_deed_threshold(u64::MAX);
    let at = deposit(&world);
    let unit = gatherer(&mut world, at);

    run(&mut world, 40, 1);

    let deeds = world.unit_deeds(unit).expect("the unit must live");
    assert!(
        deeds > 0,
        "the fixture must gather something, or it tests an idle unit"
    );
    assert_eq!(world.characters().len(), 0, "nobody reached the threshold");
    assert_eq!(
        world.unit_character(unit).expect("the unit must live"),
        None,
        "a unit below the threshold carries no character"
    );
}

#[test]
fn a_unit_is_promoted_once_and_not_on_every_later_frame() {
    let mut world = world();
    let at = deposit(&world);
    let unit = gatherer(&mut world, at);
    run(&mut world, 40, 1);
    let first = world
        .unit_character(unit)
        .expect("the unit must live")
        .expect("the unit must carry a character");
    assert_eq!(world.characters().len(), 1, "one promotion so far");

    run(&mut world, 40, 1);

    assert_eq!(
        world.characters().len(),
        1,
        "a promoted unit must not be promoted again on every later frame"
    );
    assert_eq!(
        world.unit_character(unit).expect("the unit must live"),
        Some(first),
        "the unit must keep the character it was given"
    );
}

#[test]
fn the_promotion_ranks_by_deeds_and_not_by_the_order_the_units_are_read_in() {
    // The budget admits one character a frame, so the rank decides who it is
    // rather than everybody being promoted whatever the order.
    //
    // **The fixture makes the later unit the worthier one.** A unit spawned
    // first holds the lower slot and the lower identity, and read order and
    // identity order both name it. Only a rank by deeds names the other, so
    // this fixture tells the three apart.[^1]
    //
    // [^1]: Findings register, FND-270. `docs/FINDINGS.md`
    let mut world = world();
    world.set_promotion_budget(1);
    // Nobody is eligible while the fixture arranges the deeds.
    world.set_deed_threshold(u64::MAX);
    // **The two units stand on different ground.** A head start does not
    // separate them: the choice pass issues a gather order of its own on
    // every tick, so a unit that a test stops starts again by itself. What
    // the test can control is what each unit stands on, and the richer
    // deposit is what makes the later unit the worthier one.
    let ground = deposits(&world);
    assert!(
        ground.len() >= 2,
        "the fixture needs two deposits and found {}",
        ground.len()
    );
    let (rich, poor) = (ground[0], ground[ground.len() - 1]);
    assert!(
        rich.0 > poor.0,
        "the fixture needs a richer deposit and a poorer one, and both hold {}",
        rich.0
    );

    let read_first = gatherer(&mut world, poor.1);
    let worthier = gatherer(&mut world, rich.1);
    run(&mut world, 20, 1);

    // The fixture must produce the disagreement it claims to test.
    let first_slot = world
        .soldiers()
        .slot_of(read_first)
        .expect("the unit must live");
    let worthier_slot = world
        .soldiers()
        .slot_of(worthier)
        .expect("the unit must live");
    let first_deeds = world.unit_deeds(read_first).expect("the unit must live");
    let worthier_deeds = world.unit_deeds(worthier).expect("the unit must live");
    assert!(
        first_slot < worthier_slot,
        "read order must name the other unit first"
    );
    assert!(
        read_first.to_bits() < worthier.to_bits(),
        "identity order must name the other unit first"
    );
    assert!(
        worthier_deeds > first_deeds,
        "the rank must name a different unit from the other two orders, and the deeds are {worthier_deeds} against {first_deeds}"
    );

    // Both are now eligible, and the budget admits one.
    world.set_deed_threshold(1);
    run(&mut world, 1, 1);

    assert_eq!(
        world.promoted_log().len(),
        1,
        "the budget must admit exactly one"
    );
    assert_eq!(
        world.promoted_log()[0].unit,
        worthier.to_bits(),
        "the largest deeds must win, not the lowest slot and not the lowest identity"
    );
    assert_eq!(
        world.promoted_log()[0].deeds,
        worthier_deeds,
        "the event must report the deeds the pass ranked on"
    );
}

#[test]
fn a_unit_in_a_reused_slot_inherits_no_character() {
    // A slot that is freed and filled again carries a higher generation. The
    // link column holds a whole identity, so the character of the unit that
    // died there must not answer for the unit that took its place.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let at = deposit(&world);
    let first = gatherer(&mut world, at);
    run(&mut world, 40, 1);
    let character = world
        .unit_character(first)
        .expect("the unit must live")
        .expect("the fixture must promote the first unit");
    let slot = world
        .soldiers()
        .slot_of(first)
        .expect("the unit must be live");

    assert!(world.despawn_soldier(first), "the unit must be removed");
    let second = world
        .spawn_soldier(at, FactionId(0))
        .expect("the tile must admit the unit");
    assert_eq!(
        world.soldiers().slot_of(second),
        Some(slot),
        "the fixture needs the arena to reuse the slot, or it tests nothing"
    );
    assert_ne!(
        second.to_bits(),
        first.to_bits(),
        "the two identities must differ"
    );

    run(&mut world, 1, 1);

    assert_eq!(
        world.unit_character(second).expect("the unit must live"),
        None,
        "a unit in a reused slot carries none of the character of the dead one"
    );
    assert_eq!(
        world.unit_deeds(second).expect("the unit must live"),
        0,
        "a unit in a reused slot inherits none of the deeds of the dead one"
    );
    assert!(
        world.characters().contains(character),
        "the character of the dead unit still exists, unattached"
    );
}

#[test]
fn a_promoted_unit_chooses_and_moves_by_the_same_pass_as_any_other() {
    // The promotion adds a reference and never a second decision site. A unit
    // that carries a character is steered by the pass that steers every unit,
    // and nothing about the character row reaches it.[^1]
    //
    // [^1]: Decisions register, DEC-002. `docs/DECISIONS.md`
    let mut world = world();
    let at = deposit(&world);
    let promoted = gatherer(&mut world, at);
    run(&mut world, 40, 1);
    assert!(
        world
            .unit_character(promoted)
            .expect("the unit must live")
            .is_some(),
        "the fixture must promote the unit, or it compares two plain units"
    );
    let plain = gatherer(&mut world, at);

    run(&mut world, 20, 1);

    // Both units answer the same readers in the same way. The promoted one is
    // not held still, not given a different intent path, and not removed.
    assert!(
        world.soldiers().contains(promoted),
        "the promoted unit lives"
    );
    assert!(world.soldiers().contains(plain), "the plain unit lives");
    assert!(
        world.soldier_intent(promoted).is_some(),
        "the promoted unit is still steered by the choice pass"
    );
    assert!(
        world.unit_deeds(promoted).expect("live") > 0,
        "the promoted unit still gathers"
    );
    assert_eq!(
        world.unit_character(plain).expect("the unit must live"),
        None,
        "the plain unit carries nobody, so the two are the case this compares"
    );
}

#[test]
fn the_deeds_of_a_unit_never_fall() {
    // The eligibility scan reads a level and not an edge, so it is correct
    // only while the value rises. A rule that lowered the value would break
    // the scan in silence.[^1]
    //
    // [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D2. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    let mut world = world();
    let at = deposit(&world);
    let unit = gatherer(&mut world, at);
    let mut last = 0u64;
    let mut rose = false;
    for _ in 0..60 {
        run(&mut world, 1, 1);
        let Some(deeds) = world.unit_deeds(unit) else {
            break;
        };
        assert!(deeds >= last, "the deeds fell from {last} to {deeds}");
        rose |= deeds > last;
        last = deeds;
    }
    assert!(
        rose,
        "the fixture must raise the deeds, or it asserts nothing"
    );
}

#[test]
fn the_promotion_is_the_same_at_every_thread_count() {
    // The promotion must depend on what a unit did and never on the order the
    // threads finished in.[^1]
    //
    // [^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    let mut answers = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = world();
        let at = deposit(&world);
        for _ in 0..5 {
            gatherer(&mut world, at);
        }
        run(&mut world, 40, threads);
        assert!(
            !world.characters().is_empty(),
            "the fixture must promote somebody at {threads} threads"
        );
        answers.push((world.characters().len(), world.state_hash()));
    }
    assert_eq!(answers[0], answers[1], "one thread and two must agree");
    assert_eq!(answers[1], answers[2], "two threads and twelve must agree");
}

#[test]
fn a_character_outlives_the_unit_that_carried_it() {
    let mut world = world();
    let at = deposit(&world);
    let unit = gatherer(&mut world, at);
    run(&mut world, 40, 1);
    let character = world
        .unit_character(unit)
        .expect("the unit must live")
        .expect("the unit must carry a character");

    assert!(world.despawn_soldier(unit), "the unit must be removed");
    run(&mut world, 1, 1);

    assert!(
        world.characters().contains(character),
        "the character was created as its own entity and outlives the body"
    );
    assert_eq!(
        world.unit_character(unit),
        None,
        "the dead identity answers nothing"
    );
}
