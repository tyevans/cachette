//! The whole-world hash covers every parameter that the step reads.
//!
//! The engine hashes its whole state each frame, and the golden test
//! compares that value against a stored file.[^1] A value that the step
//! reads on every tick and that the hash does not cover lets two worlds
//! hash the same and diverge on the next tick. The hash then reports the
//! effect and never the cause, one or more ticks late.[^2]
//!
//! Each test below builds two worlds that agree in everything, changes one
//! parameter of one world through the public interface, and asserts that
//! the two hashes differ. The change writes no tile and moves no unit, so
//! only the parameter can move the hash.
//!
//! The golden file cannot do this work. It notices that something changed.
//! It cannot say which input the hash stopped depending on.[^3]
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: Findings register, FND-480 and FND-537. `docs/FINDINGS.md`
//! [^3]: Testing rules, section 2. `.claude/rules/testing.md`

use cachette_core::resource::{Amount, RecoveryRules, ResourceKind, RESOURCE_KIND_COUNT};
use cachette_core::{Fix32, World, WorldConfig};

/// The scenario. It is small, because no test here steps the world.
const CONFIG: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 4,
    unit_capacity: 1024,
};

/// Builds two worlds that agree in everything.
fn two_worlds() -> (World, World) {
    (
        World::new(CONFIG).expect("the extent must describe a world"),
        World::new(CONFIG).expect("the extent must describe a world"),
    )
}

#[test]
fn the_recovery_rules_enter_the_hash() {
    let (plain, mut changed) = two_worlds();
    assert_eq!(plain.state_hash(), changed.state_hash());
    changed.set_recovery_rules(RecoveryRules::NONE);
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the recovery rules must reach the hash"
    );
}

#[test]
fn one_changed_recovery_period_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    let mut periods = [None; RESOURCE_KIND_COUNT];
    for kind in ResourceKind::ALL {
        periods[kind.index()] = plain.recovery_rules().period_of(kind);
    }
    let first = ResourceKind::ALL[0];
    periods[first.index()] = Some(periods[first.index()].unwrap_or(1).wrapping_add(1));
    let rules = RecoveryRules::from_ticks(periods).expect("no period is zero");
    changed.set_recovery_rules(rules);
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the period of one kind must reach the hash"
    );
}

#[test]
fn the_choice_schedule_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    let period_log2 = plain.choice_schedule().period_log2();
    changed
        .set_choice_schedule(period_log2 + 1)
        .expect("the exponent is inside the range");
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the choice schedule must reach the hash"
    );
}

#[test]
fn the_need_bucket_width_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    let shift = plain.need_buckets().shift();
    changed
        .set_need_buckets(shift + 1)
        .expect("the exponent is inside the range");
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the bucket width must reach the hash"
    );
}

#[test]
fn every_option_weight_enters_the_hash() {
    let mut option = 0u8;
    while let Some(weight) = World::new(CONFIG)
        .expect("the extent must describe a world")
        .option_weight(option)
    {
        let (plain, mut changed) = two_worlds();
        let other = if weight == Fix32::ZERO {
            Fix32::ONE
        } else {
            Fix32::ZERO
        };
        changed
            .set_option_weight(option, other)
            .expect("the option is in the set");
        assert_ne!(
            plain.state_hash(),
            changed.state_hash(),
            "the weight of option {option} must reach the hash"
        );
        option += 1;
    }
    assert!(option > 0, "the option set must hold at least one option");
}

#[test]
fn the_carry_mark_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    let mark = plain.carry_mark();
    changed.set_carry_mark(Amount(mark.0.wrapping_add(1)));
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the carry mark must reach the hash"
    );
}

#[test]
fn the_land_list_bound_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    let bound = plain.land_list_bound();
    changed.set_land_list_bound(bound.wrapping_add(1));
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the land list bound must reach the hash"
    );
}

#[test]
fn an_empty_luxury_seed_enters_the_hash() {
    let (plain, mut changed) = two_worlds();
    changed
        .seed_luxuries(&[])
        .expect("the world takes one seed");
    assert_ne!(
        plain.state_hash(),
        changed.state_hash(),
        "the luxury seed flag must reach the hash"
    );
}
