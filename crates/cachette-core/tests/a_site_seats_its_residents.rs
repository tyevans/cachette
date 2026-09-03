//! A site fills the positions it opened.
//!
//! The tests drive the engine and then read the positions. A pass that only a
//! test calls proves that the pass works and not that the step reaches it.[^1]
//!
//! Each test states what the seating depends on, and not only that it
//! repeats. A pass that seated the wrong unit repeats perfectly.[^2]
//!
//! Every fixture asserts that it produced the case it claims to test. The two
//! supply tests assert the shortage they need in each direction, because a
//! fixture that happened to balance would pass whatever the scan did.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// A world that holds ground on every tile the fixtures need.
const CONFIG: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The place every fixture founds on.
const PLACE: Axial = Axial::new(0, 0);

/// Builds a world that settles its positions on every tick, founds one site
/// on it, and runs the frame that opens the positions of that site.
///
/// **The resize pass opens the positions, and it runs inside the step.** A
/// fixture that read the row before the first frame would read an empty row
/// and prove nothing, so the fixture steps once and asserts that the site
/// opened something to be seated in.
fn one_site() -> (World, Entity) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_position_schedule(1, 0)
        .expect("the period is inside the range");
    let site = world
        .found_settlement(PLACE, FactionId(0))
        .expect("the tile is inside the world");
    run(&mut world, 1, 1);
    let (open, held) = seats(&world, site);
    assert!(open > 0, "the fixture must open a position");
    assert_eq!(
        held, 0,
        "the fixture founds no resident, so nobody is seated"
    );
    (world, site)
}

/// Spawns a unit on the founded tile and makes it live at the site.
fn resident(world: &mut World, site: Entity) -> Entity {
    let unit = world
        .spawn_soldier(PLACE, FactionId(0))
        .expect("the tile must admit the unit");
    assert!(
        world.set_home_site(unit, Some(site)),
        "the unit must take a home"
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

/// Returns how many positions of a site exist, and how many hold somebody.
fn seats(world: &World, site: Entity) -> (usize, usize) {
    let row = world.site_positions(site).expect("the site must be live");
    (
        row.iter().filter(|entry| entry.exists()).count(),
        row.iter()
            .filter(|entry| entry.exists() && entry.holder_bits() != 0)
            .count(),
    )
}

/// Returns the units that hold a position at a site, in row order.
fn holders(world: &World, site: Entity) -> Vec<u64> {
    world
        .site_positions(site)
        .expect("the site must be live")
        .iter()
        .filter(|entry| entry.exists() && entry.holder_bits() != 0)
        .map(|entry| entry.holder_bits())
        .collect()
}

#[test]
fn a_site_seats_the_units_that_live_in_it() {
    let (mut world, site) = one_site();
    let unit = resident(&mut world, site);
    run(&mut world, 1, 1);

    let (open, held) = seats(&world, site);
    assert!(open > 0, "the site must still hold positions");
    assert_eq!(held, 1, "the resident must take a position");
    assert_eq!(
        holders(&world, site),
        vec![unit.to_bits()],
        "the position must name the resident"
    );
}

#[test]
fn a_unit_that_lives_nowhere_takes_no_position() {
    let (mut world, site) = one_site();
    let stranger = world
        .spawn_soldier(PLACE, FactionId(0))
        .expect("the tile must admit the unit");

    run(&mut world, 1, 1);

    let (open, held) = seats(&world, site);
    assert!(open > 0, "the site must hold a position nobody took");
    assert_eq!(held, 0, "a unit that lives nowhere works nowhere");
    assert!(
        world.soldiers().contains(stranger),
        "the fixture must keep the unit alive, or it proves nothing"
    );
}

#[test]
fn a_site_with_more_residents_than_positions_fills_every_position() {
    let (mut world, site) = one_site();
    let (open, _) = seats(&world, site);

    // The fixture must reach the case it names. More residents than
    // positions is the case, so it spawns one more than the site opened.
    let mut units = Vec::new();
    for _ in 0..=open {
        units.push(resident(&mut world, site));
    }
    assert!(
        units.len() > open,
        "the fixture holds {} residents against {open} positions, which is not the case it tests",
        units.len()
    );

    run(&mut world, 1, 1);

    let (open_after, held) = seats(&world, site);
    assert_eq!(held, open_after, "every position must find somebody");
    assert!(
        held < units.len(),
        "the case needs a resident who found no position"
    );
}

#[test]
fn a_site_with_more_positions_than_residents_leaves_the_rest_open() {
    let (mut world, site) = one_site();
    let (open, _) = seats(&world, site);
    assert!(
        open > 1,
        "the case needs a site that opened more than one position"
    );
    let unit = resident(&mut world, site);
    let _ = open;

    run(&mut world, 1, 1);

    let (open_after, held) = seats(&world, site);
    assert_eq!(held, 1, "the one resident must take one position");
    assert!(
        open_after > held,
        "a position must be left open, or the case is not reached"
    );
    assert_eq!(
        holders(&world, site),
        vec![unit.to_bits()],
        "the held position must name the resident"
    );
}

#[test]
fn a_seated_unit_keeps_its_position_on_a_later_frame() {
    let (mut world, site) = one_site();
    let unit = resident(&mut world, site);
    run(&mut world, 1, 1);
    let first = holders(&world, site);
    assert_eq!(first, vec![unit.to_bits()], "the resident must be seated");

    run(&mut world, 8, 1);

    assert_eq!(
        holders(&world, site),
        first,
        "a later frame must not move a unit that already holds a position"
    );
}

#[test]
fn a_position_releases_a_unit_that_died_and_gives_it_to_another() {
    let (mut world, site) = one_site();
    let first = resident(&mut world, site);
    run(&mut world, 1, 1);
    assert_eq!(
        holders(&world, site),
        vec![first.to_bits()],
        "the first resident must be seated"
    );

    assert!(world.despawn_soldier(first), "the unit must be removed");
    let second = resident(&mut world, site);
    assert_ne!(
        second.to_bits(),
        first.to_bits(),
        "the two identities must differ, or the test proves nothing"
    );
    run(&mut world, 1, 1);

    assert_eq!(
        holders(&world, site),
        vec![second.to_bits()],
        "the position must name the living unit and never the dead one"
    );
}

#[test]
fn the_seating_is_the_same_at_every_thread_count() {
    // The seating must depend on the identity of a unit and never on the
    // order that the threads finished in. Two runs that differ only in the
    // thread count must name the same units in the same positions.[^1]
    //
    // [^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    let mut answers = Vec::new();
    for threads in [1usize, 2, 12] {
        let (mut world, site) = one_site();
        for _ in 0..6 {
            resident(&mut world, site);
        }
        run(&mut world, 4, threads);
        let held = holders(&world, site);
        assert!(!held.is_empty(), "the fixture must seat somebody");
        answers.push(held);
    }
    assert_eq!(answers[0], answers[1], "one thread and two must agree");
    assert_eq!(answers[1], answers[2], "two threads and twelve must agree");
}

#[test]
fn the_seating_depends_on_the_identity_and_not_on_the_order_the_units_are_read_in() {
    // A scan that seated in the order it read the units would give the first
    // position to the unit in the lowest slot. The order is a key vector
    // whose last field is the whole identity, so the answer follows the
    // identity. This test says which of the two the pass used.[^1]
    //
    // **The two orders agree in an ordinary fixture**, because a unit spawned
    // earlier holds both a lower slot and a lower identity. A fixture built
    // that way cannot tell the pass from the defect, and the first version of
    // this test was built that way and passed with the sort removed.[^2]
    //
    // The fixture separates them. A slot that is freed and filled again
    // carries a higher generation, so the unit in the lowest slot holds the
    // highest identity. Reading order then names one unit first and identity
    // order names another.
    //
    // [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    // [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    let (mut world, site) = one_site();
    let first = resident(&mut world, site);
    let second = resident(&mut world, site);
    let low = world
        .soldiers()
        .slot_of(first)
        .expect("the unit must be live");

    assert!(world.despawn_soldier(first), "the unit must be removed");
    let late = resident(&mut world, site);
    assert_eq!(
        world.soldiers().slot_of(late),
        Some(low),
        "the fixture needs the arena to reuse the slot, or the two orders agree"
    );
    assert!(
        late.to_bits() > second.to_bits(),
        "the reused slot must carry the higher identity, or the test proves nothing"
    );

    run(&mut world, 1, 1);

    let held = holders(&world, site);
    assert!(
        held.len() >= 2,
        "the fixture must seat both units, or the order is not tested"
    );
    assert_eq!(
        held[0],
        second.to_bits(),
        "the first position must go to the lowest identity and not to the lowest slot"
    );
    assert_eq!(
        held[1],
        late.to_bits(),
        "the second position must go to the higher identity"
    );
}
