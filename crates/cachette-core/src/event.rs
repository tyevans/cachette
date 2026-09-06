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

use crate::holding::Holder;
use crate::types::{FactionId, Fix32, Tick, TileIdx};

/// A unit took an amount from a tile.
///
/// The event reports one grant of the gather resolve. A watcher reads the log
/// to see where a resource went, and the amount is exact, so a sum over the
/// log balances against the ledger.[^2]
///
/// The layout is 8 + 8 + 4 + 4 + 1 + 7 bytes, which is 32 bytes at an
/// alignment of 8. The trailing array declares every padding byte, so the type
/// holds no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D4. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct ResourceTaken {
    /// The tick at which the unit took the amount.
    pub tick: Tick,
    /// The unit that took the amount, as its identity in bits.
    pub unit: u64,
    /// The tile that the amount came from.
    pub tile: TileIdx,
    /// The amount that the unit took. It is never zero.
    pub amount: u32,
    /// The kind of resource, as its number.
    pub kind: u8,
    /// The declared padding. Always zero.
    pub padding: [u8; 7],
}

impl ResourceTaken {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, unit: u64, tile: TileIdx, amount: u32, kind: u8) -> Self {
        Self {
            tick,
            unit,
            tile,
            amount,
            kind,
            padding: [0; 7],
        }
    }
}

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
    /// Who holds the tile, as the step left it.
    ///
    /// The value names a faction, or nobody. The holder type states the value
    /// for nobody, and that value sits above the faction ceiling, so no
    /// faction collides with it.[^2]
    ///
    /// # References
    ///
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub holder: Holder,
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
        holder: Holder,
        kind: ChangeKind,
    ) -> Self {
        Self {
            tick,
            tile,
            value,
            holder,
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

/// What ended an upgrade.
///
/// The type is a one-byte integer and not an enumeration with a hidden
/// discriminant width, because the event that holds it must be plain
/// data.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
pub type WearCause = u8;

/// The wear of the tick came from the weather alone.
pub const WEAR_CAUSE_WEATHER: WearCause = 1;
/// The wear of the tick came from a hostile army alone.
pub const WEAR_CAUSE_ARMY: WearCause = 2;
/// The wear of the tick came from the weather and from a hostile army.
pub const WEAR_CAUSE_BOTH: WearCause = 3;
/// A caller ordered the destruction. No wear ended it.
pub const WEAR_CAUSE_ORDERED: WearCause = 4;

/// An upgrade is gone from its tile.
///
/// **An upgrade has two sinks, and both write this event.** The wear pass
/// takes the last of the condition, and a caller orders the destruction. The
/// cause column says which. The entry is removed and the tile returns to the
/// world the generator made, so no reader can ask the world afterwards what
/// stood there. The event is the only record of it.
///
/// The layout is 8 + 4 + 2 + 1 + 1 + 1 + 7 bytes, which is 24 bytes at an
/// alignment of 8. The trailing array declares every padding byte, so the
/// type holds no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UpgradeCollapsed {
    /// The tick at which the upgrade went.
    pub tick: Tick,
    /// The tile that carried it.
    pub tile: TileIdx,
    /// Who held the tile as the wear pass read it.
    ///
    /// The value names a faction, or nobody. The holder type states the
    /// value for nobody, and that value sits above the faction ceiling.
    pub holder: Holder,
    /// The category that stood there, as its number.
    pub category: u8,
    /// The level that stood there. Zero means that the first level was still
    /// under construction, which only an ordered destruction reaches.
    pub level: u8,
    /// What ended it.
    pub cause: WearCause,
    /// The declared padding. Always zero.
    pub padding: [u8; 7],
}

impl UpgradeCollapsed {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        tile: TileIdx,
        holder: Holder,
        category: u8,
        level: u8,
        cause: WearCause,
    ) -> Self {
        Self {
            tick,
            tile,
            holder,
            category,
            level,
            cause,
            padding: [0; 7],
        }
    }
}

/// An upgrade reached a new level, and that level now stands on the tile.
///
/// A level one event says that the first level of the category finished, so
/// something stands on ground that carried nothing. A higher level says that
/// what stood there grew. The wonder category claims the wealth-or-wonder
/// end, so a watcher reads a wonder from the category column and needs no
/// second reader for it.
///
/// The layout is 8 + 4 + 2 + 1 + 1 bytes, which is 16 bytes at an alignment
/// of 8. The type needs no padding byte, and it holds none.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UpgradeFinished {
    /// The tick at which the level finished.
    pub tick: Tick,
    /// The tile that carries it.
    pub tile: TileIdx,
    /// Who holds the tile as the build pass left it.
    pub holder: Holder,
    /// The category that rose, as its number.
    pub category: u8,
    /// The level that now stands there. It is never zero.
    pub level: u8,
}

impl UpgradeFinished {
    /// Builds an event.
    #[must_use]
    pub const fn new(tick: Tick, tile: TileIdx, holder: Holder, category: u8, level: u8) -> Self {
        Self {
            tick,
            tile,
            holder,
            category,
            level,
        }
    }
}

/// A settlement was founded.
///
/// The engine held a count of the settlements and nothing else. A watcher
/// saw the count rise and could not say whose settlement it was or where it
/// stood, and a count that falls when a settlement is lost hides a founding
/// in the same tick.
///
/// The layout is 8 + 8 + 4 + 2 + 2 bytes, which is 24 bytes at an alignment
/// of 8. The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct SettlementFounded {
    /// The tick at which the settlement was founded.
    pub tick: Tick,
    /// The settlement, as its identity in bits.
    ///
    /// The value is the whole identity and not a slot index. A slot index
    /// survives the loss of what it named.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D1. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    pub settlement: u64,
    /// The tile it stands on.
    pub tile: TileIdx,
    /// The faction that founded it.
    pub faction: FactionId,
    /// The declared padding. Always zero.
    pub padding: [u8; 2],
}

impl SettlementFounded {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, settlement: u64, tile: TileIdx, faction: FactionId) -> Self {
        Self {
            tick,
            settlement,
            tile,
            faction,
            padding: [0; 2],
        }
    }
}
