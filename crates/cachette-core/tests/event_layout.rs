//! The event layout rules.
//!
//! Every event type is plain data: an explicit layout, declared padding, and
//! no boolean field.[^1] The engine declares the fields of each event in one
//! place, and the binding layer builds a column for each of them from that
//! declaration.[^3]
//!
//! The declaration takes no offset and no width. The compiler reads both from
//! the type. What the declaration can still get wrong is coverage. A field
//! that somebody adds to an event and does not declare compiles, crosses to
//! nobody, and leaves the declaration short of the last byte. The first test
//! below is what fails then.
//!
//! # References
//!
//! [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
//! [^2]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
//! [^3]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`

use cachette_core::conversion::UnitConverted;
use cachette_core::event::CHANGE_KIND_RAISED;
use cachette_core::event_layout::{declared_event_layouts, layout_defects};
use cachette_core::types::{FactionId, TileIdx};
use cachette_core::{Fix32, Holder, Tick, TileChanged};

#[test]
fn every_declared_event_covers_its_own_type() {
    // The declaration must reach the last byte of the type. A field added to
    // the type and not declared stops it short, and this is what says so.
    let mut defects: Vec<String> = Vec::new();
    for event in declared_event_layouts() {
        defects.extend(layout_defects(event));
    }
    assert!(defects.is_empty(), "{}", defects.join("\n"));
}

#[test]
fn every_declared_event_moves_in_whole_registers() {
    // ADR-0011: size and align a migrating structure to 8 or 16 bytes.
    for event in declared_event_layouts() {
        assert_eq!(event.size % 8, 0, "{} is {} bytes", event.name, event.size);
    }
}

#[test]
fn every_declared_event_names_its_columns() {
    // A field that crosses gives a column name and a column kind. A padding
    // field gives neither. Nothing in between is valid.
    for event in declared_event_layouts() {
        for field in event.fields {
            assert_eq!(
                field.column.is_some(),
                field.kind.is_some(),
                "{}: the field `{}` names one of a column and a kind",
                event.name,
                field.field
            );
        }
    }
}

#[test]
fn the_declared_events_have_distinct_names() {
    let mut names: Vec<&str> = declared_event_layouts()
        .iter()
        .map(|event| event.name)
        .collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count);
}

#[test]
fn a_new_event_holds_zero_padding() {
    let event = TileChanged::new(
        Tick(3),
        TileIdx(9),
        Fix32::ONE,
        Holder::of(FactionId(1)),
        CHANGE_KIND_RAISED,
    );
    assert_eq!(event.padding, [0; 5]);
}

#[test]
fn the_event_round_trips_through_bytes() {
    let events = [
        TileChanged::new(Tick(1), TileIdx(0), Fix32::ZERO, Holder::NOBODY, 1),
        TileChanged::new(Tick(1), TileIdx(1), Fix32::ONE, Holder::of(FactionId(2)), 2),
    ];
    let bytes: &[u8] = bytemuck::cast_slice(&events);
    assert_eq!(bytes.len(), 2 * size_of::<TileChanged>());
    let restored: &[TileChanged] = bytemuck::cast_slice(bytes);
    assert_eq!(restored, &events);
}

#[test]
fn the_sort_key_orders_by_tick_then_tile() {
    let first = TileChanged::new(Tick(1), TileIdx(9), Fix32::ZERO, Holder::NOBODY, 1);
    let second = TileChanged::new(Tick(2), TileIdx(0), Fix32::ZERO, Holder::NOBODY, 1);
    assert!(first.sort_key() < second.sort_key());
}

#[test]
fn a_unit_type_row_holds_no_padding_at_all() {
    // The table enters the state hash as raw bytes, so a padding byte in a
    // row would put an uninitialised byte into the hash.
    // Every column is four bytes wide, so the size is the column count times
    // four. The column count comes from the row declaration, so a column
    // added there is counted here without a second edit.
    use cachette_core::unit_type::{UnitTypeRow, UNIT_TYPE_COLUMN_COUNT};
    assert_eq!(
        size_of::<UnitTypeRow>(),
        UNIT_TYPE_COLUMN_COUNT * size_of::<Fix32>()
    );
}

#[test]
fn the_conversion_event_round_trips_through_bytes() {
    let events = [
        UnitConverted::new(Tick(4), 9, TileIdx(1), FactionId(0), FactionId(2)),
        UnitConverted::new(Tick(4), 11, TileIdx(3), FactionId(2), FactionId(0)),
    ];
    let bytes: &[u8] = bytemuck::cast_slice(&events);
    assert_eq!(bytes.len(), 2 * size_of::<UnitConverted>());
    let restored: &[UnitConverted] = bytemuck::cast_slice(bytes);
    assert_eq!(restored, &events);
}
