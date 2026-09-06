//! The register of logs and the declared event layouts must agree.
//!
//! One fact lives in two places here: the engine declares an event layout,
//! and the binding registers a log that carries that event. Neither list
//! derives from the other, because a layout can exist for an event the world
//! keeps no array of. A check that fails when the copies disagree is the
//! thing that makes the second site safe.[^1]
//!
//! # References
//!
//! [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`

use cachette_core::event_layout::declared_event_layouts;
use _core::logs::{log_names, log_of, unknown_log_message, LOGS};

#[test]
fn every_declared_event_has_a_log() {
    let names = log_names();
    let missing: Vec<&str> = declared_event_layouts()
        .iter()
        .map(|event| event.name)
        .filter(|name| !names.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "these events declare a layout and no log reads them: {missing:?}"
    );
}

#[test]
fn every_log_names_a_declared_event() {
    let declared: Vec<&str> = declared_event_layouts()
        .iter()
        .map(|event| event.name)
        .collect();
    let strays: Vec<&str> = log_names()
        .into_iter()
        .filter(|name| !declared.contains(name))
        .collect();
    assert!(
        strays.is_empty(),
        "these logs name an event the engine does not declare: {strays:?}"
    );
}

#[test]
fn the_logs_have_distinct_names() {
    let mut names = log_names();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "two logs give one name");
}

#[test]
fn the_register_is_not_empty() {
    assert!(!LOGS.is_empty(), "the register offers no log");
}

#[test]
fn a_name_no_log_has_is_refused() {
    assert!(log_of("no_such_log").is_none());
    let message = unknown_log_message("no_such_log");
    assert!(message.contains("no_such_log"), "{message}");
    for name in log_names() {
        assert!(
            message.contains(name),
            "the message hides `{name}`: {message}"
        );
    }
}

#[test]
fn the_new_logs_are_registered() {
    // The three gaps this work closed. A later change that drops one of them
    // fails here rather than in a demonstration.
    for name in ["upgrade_collapsed", "upgrade_finished", "settlement_founded"] {
        assert!(log_of(name).is_some(), "the register lost `{name}`");
    }
}
