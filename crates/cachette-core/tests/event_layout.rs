//! The event layout rules.
//!
//! Every event type is plain data: an explicit layout, declared padding, and
//! no boolean field.[^1] This test asserts the rules that the derive macro
//! cannot state.
//!
//! # References
//!
//! [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`

use cachette_core::event::CHANGE_KIND_RAISED;
use cachette_core::types::{FactionId, TileIdx};
use cachette_core::{Fix32, Tick, TileChanged};

#[test]
fn the_event_declares_every_padding_byte() {
    // A plain-data type has no implicit padding, so the size of the type is
    // the sum of the sizes of its fields.
    let declared = size_of::<Tick>()
        + size_of::<TileIdx>()
        + size_of::<Fix32>()
        + size_of::<FactionId>()
        + size_of::<u8>()
        + size_of::<[u8; 5]>();
    assert_eq!(size_of::<TileChanged>(), declared);
}

#[test]
fn the_event_moves_in_whole_registers() {
    // ADR-0011: size and align a migrating structure to 8 or 16 bytes.
    assert_eq!(size_of::<TileChanged>() % 8, 0);
    assert_eq!(align_of::<TileChanged>(), 8);
}

#[test]
fn a_new_event_holds_zero_padding() {
    let event = TileChanged::new(
        Tick(3),
        TileIdx(9),
        Fix32::ONE,
        FactionId(1),
        CHANGE_KIND_RAISED,
    );
    assert_eq!(event.padding, [0; 5]);
}

#[test]
fn the_event_round_trips_through_bytes() {
    let events = [
        TileChanged::new(Tick(1), TileIdx(0), Fix32::ZERO, FactionId(0), 1),
        TileChanged::new(Tick(1), TileIdx(1), Fix32::ONE, FactionId(2), 2),
    ];
    let bytes: &[u8] = bytemuck::cast_slice(&events);
    assert_eq!(bytes.len(), 2 * size_of::<TileChanged>());
    let restored: &[TileChanged] = bytemuck::cast_slice(bytes);
    assert_eq!(restored, &events);
}

#[test]
fn the_sort_key_orders_by_tick_then_tile() {
    let first = TileChanged::new(Tick(1), TileIdx(9), Fix32::ZERO, FactionId(0), 1);
    let second = TileChanged::new(Tick(2), TileIdx(0), Fix32::ZERO, FactionId(0), 1);
    assert!(first.sort_key() < second.sort_key());
}
