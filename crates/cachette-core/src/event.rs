//! The event types.
//!
//! Every event type is plain data. That means an explicit layout, declared
//! padding, and no boolean field. Use a one-byte integer where a boolean is
//! wanted.[^1]
//!
//! Undeclared padding holds uninitialised bytes. Those bytes enter the state
//! hash and produce a nondeterminism that has no cause in the
//! simulation.[^1]
//!
//! The engine holds one append-only array for each event type. There is no
//! polymorphic container.[^2]
//!
//! # References
//!
//! [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
//! [^2]: ADR-0031, events live in type-segregated arenas of plain data. `docs/adrs/REGISTRY.md`

use bytemuck::{Pod, Zeroable};

use crate::types::{FactionId, Fix32, Tick, TileIdx};

/// The kind of change that a tile event reports.
///
/// The type is a one-byte integer and not an enumeration with a hidden
/// discriminant width, because the event that holds it must be plain data.
pub type ChangeKind = u8;

/// The change kind for a value that increased.
pub const CHANGE_KIND_RAISED: ChangeKind = 1;
/// The change kind for a value that decreased.
pub const CHANGE_KIND_LOWERED: ChangeKind = 2;

/// A tile value changed.
///
/// This is the one event type that the current stubs need. It exists to
/// prove the harnesses. Add the real event types with the systems that emit
/// them.
///
/// The layout is 8 + 4 + 4 + 2 + 1 + 5 bytes, which is 24 bytes at an
/// alignment of 8. The trailing array declares every padding byte, so the
/// type holds no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct TileChanged {
    /// The tick at which the change happened.
    pub tick: Tick,
    /// The tile that changed.
    pub tile: TileIdx,
    /// The value after the change.
    pub value: Fix32,
    /// The faction that owns the tile.
    pub faction: FactionId,
    /// The kind of change.
    pub kind: ChangeKind,
    /// The declared padding. Always zero.
    pub padding: [u8; 5],
}

impl TileChanged {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        tile: TileIdx,
        value: Fix32,
        faction: FactionId,
        kind: ChangeKind,
    ) -> Self {
        Self {
            tick,
            tile,
            value,
            faction,
            kind,
            padding: [0; 5],
        }
    }

    /// Returns the total sort key of the event.
    ///
    /// The key is the tick and then the tile index. No two events in one
    /// tick report the same tile, so the key leaves no tie.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub const fn sort_key(&self) -> (u64, u32) {
        (self.tick.0, self.tile.0)
    }
}
