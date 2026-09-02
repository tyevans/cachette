//! Resolving an identity that a caller outside the crate handed back.
//!
//! A caller outside this crate cannot build an identity, because the
//! constructor is private to the crate.[^1] It names a soldier by the value
//! the engine gave it, and it can hand back a value the engine gave for a
//! soldier that has since died.
//!
//! The engine compares the generation the value carries against the
//! generation the arena holds for the slot. A mismatch means the soldier is
//! dead, and a dead identity resolves to nothing.[^2] The engine never
//! returns the soldier that now occupies the slot.[^3]
//!
//! The tests see only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/draft/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::{Axial, FactionId, IdentityError, ResourceKind, World, WorldConfig};

/// Builds a small world that admits a soldier at the origin.
fn world() -> World {
    World::new(WorldConfig {
        width: 8,
        height: 8,
        seed: 3,
        faction_count: 2,
    })
    .expect("the extent must describe a world")
}

#[test]
fn an_identity_the_engine_gave_resolves_to_the_same_entity() {
    let mut world = world();
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(1))
        .expect("the origin admits a soldier");

    let resolved = world
        .resolve_soldier(unit.to_bits())
        .expect("the identity is live");

    assert_eq!(resolved, unit);
    assert_eq!(resolved.to_bits(), unit.to_bits());
}

#[test]
fn an_identity_for_a_reused_slot_refuses_and_names_no_occupant() {
    // The defect this test exists for: a reader holds an identity, the
    // soldier dies, another soldier takes the slot, and the reader reports
    // on the new soldier with nothing failing.[^1] The fixture must
    // therefore reach the reuse. The assertion on the slot is what proves
    // it does; without it the test would pass over an arena that opened a
    // second slot and never exercise the comparison at all.[^2]
    //
    // [^1]: Testing Rules, section 2. `.claude/rules/testing.md`
    // [^2]: Testing Rules, section 2a. `.claude/rules/testing.md`
    let mut world = world();
    let dead = world
        .spawn_soldier(Axial::new(0, 0), FactionId(1))
        .expect("the origin admits a soldier");
    assert!(world.despawn_soldier(dead));

    let living = world
        .spawn_soldier(Axial::new(0, 0), FactionId(1))
        .expect("the origin admits a soldier again");

    assert_eq!(
        living.index(),
        dead.index(),
        "the fixture must reuse the slot, or the test proves nothing"
    );
    assert_ne!(living.to_bits(), dead.to_bits());

    match world.resolve_soldier(dead.to_bits()) {
        Err(IdentityError::Stale { slot, given, held }) => {
            assert_eq!(slot, dead.index());
            assert_eq!(given, dead.generation());
            assert_eq!(held, living.generation());
        }
        other => panic!("the dead identity must refuse, and it gave {other:?}"),
    }

    assert_eq!(
        world
            .resolve_soldier(living.to_bits())
            .expect("the living identity resolves"),
        living
    );
}

#[test]
fn a_dead_identity_refuses_before_the_slot_is_reused() {
    // The generation advances at the free, not at the next allocation, so
    // the identity fails at the moment the soldier dies.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let dead = world
        .spawn_soldier(Axial::new(0, 0), FactionId(1))
        .expect("the origin admits a soldier");
    assert!(world.despawn_soldier(dead));

    assert!(matches!(
        world.resolve_soldier(dead.to_bits()),
        Err(IdentityError::Stale { .. })
    ));
}

#[test]
fn zero_is_not_an_identity() {
    // The engine never gives out zero, because the handle holds a non-zero
    // value so that an optional handle stays one word wide.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let world = world();
    assert_eq!(
        world.resolve_soldier(0),
        Err(IdentityError::NotAnIdentity),
        "zero must not resolve"
    );
}

#[test]
fn a_slot_the_arena_never_opened_refuses() {
    let mut world = world();
    let unit = world
        .spawn_soldier(Axial::new(0, 0), FactionId(1))
        .expect("the origin admits a soldier");

    // The same generation, one slot past the end. A caller that composed an
    // identity from an index it chose lands here.
    let composed = (u64::from(unit.generation()) << 32) | u64::from(unit.index() + 1);

    assert_eq!(
        world.resolve_soldier(composed),
        Err(IdentityError::NoSuchSlot {
            slot: unit.index() + 1
        })
    );
}

#[test]
fn the_gather_log_names_a_unit_that_resolves() {
    // The gather event carries an identity, and this is the path that makes
    // the column at the boundary worth anything: a reader takes a value out
    // of the log and gives it back.[^1]
    //
    // [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D1. `docs/adrs/draft/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    let mut world = world();
    let mut spawned = Vec::new();
    for row in 0..8 {
        for column in 0..8 {
            if let Ok(unit) = world.spawn_soldier(Axial::new(column, row), FactionId(1)) {
                spawned.push(unit);
            }
        }
    }
    assert!(!spawned.is_empty(), "the world must admit a soldier");
    // Food is the kind that open ground carries the most of, so a world of
    // gatherers on open ground produces a grant.
    let food = ResourceKind::from_u8(0).expect("zero names a resource kind");
    for unit in &spawned {
        world.order_gather(*unit, food);
    }

    let mut seen = false;
    for _ in 0..8 {
        world.step(1).expect("the step must run");
        for event in world.gather_log() {
            seen = true;
            let unit = world
                .resolve_soldier(event.unit)
                .expect("the log names a live unit");
            assert_eq!(unit.to_bits(), event.unit);
        }
        if seen {
            break;
        }
    }
    assert!(seen, "the fixture must produce a gather event");
}
