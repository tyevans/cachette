//! The declared layout of an event type.
//!
//! An event type is plain data with an explicit layout and declared
//! padding.[^1] A reader outside the engine must know which fields the type
//! holds, where each one starts, how wide it is, and how to read it as a
//! number. The binding layer hands that reader one column for each field.
//!
//! Before this module the binding wrote the field list a second time, once
//! for each log. A field added to an event and not added to the binding never
//! reached the reader, and nothing failed.[^2] This module gives the field
//! list one home. The binding builds every column from it, so the binding
//! names no field.
//!
//! The declaration takes the field names. It does not take an offset or a
//! width. The compiler supplies both from the type itself, so a declared
//! width cannot disagree with the field it describes. What the declaration
//! can still get wrong is coverage: a field that nobody declared. The checker
//! here finds that, because the declared fields must fill the type exactly,
//! and a test asserts it for every declared event.[^3]
//!
//! # References
//!
//! [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
//! [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^3]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`

use bytemuck::Pod;

use crate::holding::Holder;
use crate::types::{Accum, FactionId, Fix32, Tick, TileIdx};
use crate::unit_type::UnitTypeId;

/// How a reader reads one field as a number.
///
/// The kind names the element type of the column that the field crosses in.
/// No kind is a floating point type. A fixed-point field crosses as its raw
/// integer.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColumnKind {
    /// An unsigned one-byte field.
    U8,
    /// An unsigned two-byte field.
    U16,
    /// An unsigned four-byte field.
    U32,
    /// An unsigned eight-byte field.
    U64,
    /// A signed four-byte field.
    I32,
    /// A signed eight-byte field.
    I64,
}

impl ColumnKind {
    /// The width of the field in bytes.
    #[must_use]
    pub const fn width(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
        }
    }

    /// The name of the matching NumPy element type.
    #[must_use]
    pub const fn numpy_name(self) -> &'static str {
        match self {
            Self::U8 => "uint8",
            Self::U16 => "uint16",
            Self::U32 => "uint32",
            Self::U64 => "uint64",
            Self::I32 => "int32",
            Self::I64 => "int64",
        }
    }
}

/// A field type that a reader reads as one number.
///
/// The declaration asks the field type for its kind. The declaration
/// therefore states the name of the field, and nothing else about its shape.
pub trait ColumnElement {
    /// How a reader reads the type as a number.
    const KIND: ColumnKind;
}

macro_rules! column_element {
    ($($type:ty => $kind:ident),* $(,)?) => {
        $(
            impl ColumnElement for $type {
                const KIND: ColumnKind = ColumnKind::$kind;
            }
        )*
    };
}

column_element! {
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    i32 => I32,
    i64 => I64,
    Tick => U64,
    TileIdx => U32,
    FactionId => U16,
    Fix32 => I32,
    Accum => I64,
    Holder => U16,
    UnitTypeId => U8,
}

/// One field of an event type.
///
/// A field either crosses to the reader as a column, or it is declared
/// padding. Padding holds no value and crosses nowhere. The declaration
/// still names it, because the declared fields must fill the type exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventField {
    /// The name of the field in the Rust source.
    pub field: &'static str,
    /// The name of the column, or `None` when the field is padding.
    pub column: Option<&'static str>,
    /// Where the field starts, in bytes from the start of the event.
    pub offset: usize,
    /// How wide the field is, in bytes.
    pub width: usize,
    /// How a reader reads the field, or `None` when the field is padding.
    pub kind: Option<ColumnKind>,
}

/// An event type that states its own fields.
///
/// The engine holds one append-only array for each event type.[^1] This trait
/// says what one record of such an array holds, in order.
///
/// # References
///
/// [^1]: ADR-0031, events live in type-segregated arenas of plain data. `docs/adrs/REGISTRY.md`
pub trait EventLayout: Pod {
    /// The name of the event, for a message and for a schema.
    const EVENT_NAME: &'static str;
    /// Every field of the event, in the order the type declares them.
    const FIELDS: &'static [EventField];
}

/// One declared event type, for a reader that walks every one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventDescriptor {
    /// The name of the event.
    pub name: &'static str,
    /// The size of one record in bytes.
    pub size: usize,
    /// The alignment of one record in bytes.
    pub align: usize,
    /// Every field of the event, in declaration order.
    pub fields: &'static [EventField],
}

/// Declares the layout of every event type, and builds the register of them.
///
/// Give each type, the name of the event, and each field in declaration
/// order. A field that crosses to the reader gives its column name as a
/// string. A padding field gives the word `pad`.
///
/// The declaration takes no offset and no width. The macro reads both from
/// the type. The macro also builds the register, so a type declared here is a
/// type the checker walks. There is no second list to hold in step.
#[macro_export]
macro_rules! event_layouts {
    (
        $(
            $event:ty, $name:literal, { $($field:ident : $field_type:ty => $column:tt),* $(,)? }
        );* $(;)?
    ) => {
        $(
            impl $crate::event_layout::EventLayout for $event {
                const EVENT_NAME: &'static str = $name;
                const FIELDS: &'static [$crate::event_layout::EventField] = &[
                    $(
                        $crate::event_layout::EventField {
                            field: stringify!($field),
                            column: $crate::event_column_name!($field, $column),
                            offset: ::core::mem::offset_of!($event, $field),
                            width: ::core::mem::size_of::<$field_type>(),
                            kind: $crate::event_column_kind!($field_type, $column),
                        },
                    )*
                ];
            }
        )*

        /// Every event type that declares a layout.
        #[must_use]
        pub fn declared_event_layouts() -> &'static [$crate::event_layout::EventDescriptor] {
            &[
                $(
                    $crate::event_layout::EventDescriptor {
                        name: $name,
                        size: ::core::mem::size_of::<$event>(),
                        align: ::core::mem::align_of::<$event>(),
                        fields: <$event as $crate::event_layout::EventLayout>::FIELDS,
                    },
                )*
            ]
        }
    };
}

/// Gives the column name of one declared field. Not called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! event_column_name {
    ($field:ident, pad) => {
        None
    };
    ($field:ident, $column:literal) => {
        Some($column)
    };
}

/// Gives the column kind of one declared field. Not called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! event_column_kind {
    ($field_type:ty, pad) => {
        None
    };
    ($field_type:ty, $column:literal) => {
        Some(<$field_type as $crate::event_layout::ColumnElement>::KIND)
    };
}

/// Reports every way a declared layout fails to describe its type.
///
/// An empty answer means that the declared fields start at byte zero, follow
/// one another with no gap, and end at the last byte of the type. A field
/// that somebody added to the type and did not declare leaves the last byte
/// short, so this reports it. A test calls this for every declared event, and
/// that test is what fails when the type and the declaration disagree.
#[must_use]
pub fn layout_defects(event: &EventDescriptor) -> Vec<String> {
    let mut defects = Vec::new();
    let name = event.name;
    if event.fields.is_empty() {
        defects.push(format!("{name} declares no field"));
        return defects;
    }
    let mut reached = 0usize;
    for field in event.fields {
        if field.offset != reached {
            defects.push(format!(
                "{name}: the field `{}` starts at byte {} and the declaration reached byte {}",
                field.field, field.offset, reached
            ));
        }
        if let Some(kind) = field.kind {
            if kind.width() != field.width {
                defects.push(format!(
                    "{name}: the field `{}` is {} bytes wide and crosses as {}",
                    field.field,
                    field.width,
                    kind.numpy_name()
                ));
            }
        }
        reached = field.offset + field.width;
    }
    if reached != event.size {
        let size = event.size;
        defects.push(format!(
            "{name}: the declaration covers {reached} bytes of {size}. A field of \
             the type is not declared, or a declared width is wrong."
        ));
    }
    let mut columns: Vec<&str> = event
        .fields
        .iter()
        .filter_map(|field| field.column)
        .collect();
    let count = columns.len();
    columns.sort_unstable();
    columns.dedup();
    if columns.len() != count {
        defects.push(format!("{name}: two fields give one column name"));
    }
    defects
}

macro_rules! column_reader {
    ($name:ident, $type:ty, $width:literal, $what:literal) => {
        #[doc = concat!("Reads one ", $what, " field of every record of a log.")]
        ///
        /// The field must belong to the layout of the log. The layout test
        /// asserts that every declared field lies inside its own type, so a
        /// field that this crate declares is always in range.
        ///
        /// # Panics
        ///
        /// Panics when the field lies outside the record.
        #[must_use]
        pub fn $name<T: EventLayout>(log: &[T], field: &EventField) -> Vec<$type> {
            let mut column = Vec::with_capacity(log.len());
            for event in log {
                let bytes = bytemuck::bytes_of(event);
                let mut raw = [0u8; $width];
                raw.copy_from_slice(&bytes[field.offset..field.offset + $width]);
                column.push(<$type>::from_ne_bytes(raw));
            }
            column
        }
    };
}

column_reader!(column_u8, u8, 1, "one-byte unsigned");
column_reader!(column_u16, u16, 2, "two-byte unsigned");
column_reader!(column_u32, u32, 4, "four-byte unsigned");
column_reader!(column_u64, u64, 8, "eight-byte unsigned");
column_reader!(column_i32, i32, 4, "four-byte signed");
column_reader!(column_i64, i64, 8, "eight-byte signed");

/// The layout of every event type that crosses to the control plane.
///
/// This declaration is the one place that names the fields of an event. The
/// binding builds the columns from it, so the binding repeats no field name.
mod declarations {
    use crate::campaign::CampaignEvent;
    use crate::cohort::{SiteRationed, UnitStarved};
    use crate::contest::UnitFell;
    use crate::conversion::UnitConverted;
    use crate::event::{
        ChangeKind, ResourceTaken, SettlementFounded, TileChanged, UpgradeCollapsed,
        UpgradeFinished, WearCause,
    };
    use crate::holding::Holder;
    use crate::promotion::UnitPromoted;
    use crate::rates::SiteShortfall;
    use crate::relation::RelationCrossed;
    use crate::trade::TradeSpoken;
    use crate::types::{Accum, FactionId, Fix32, Tick, TileIdx};
    use crate::unit_type::UnitTypeId;

    crate::event_layouts! {
        TileChanged, "tile_changed", {
            tick: Tick => "tick",
            tile: TileIdx => "tile",
            value: Fix32 => "value",
            holder: Holder => "holder",
            kind: ChangeKind => "kind",
            padding: [u8; 5] => pad,
        };

        ResourceTaken, "resource_taken", {
            tick: Tick => "tick",
            unit: u64 => "unit",
            tile: TileIdx => "tile",
            amount: u32 => "amount",
            kind: u8 => "kind",
            padding: [u8; 7] => pad,
        };

        UnitFell, "unit_fell", {
            tick: Tick => "tick",
            unit: u64 => "unit",
            tile: TileIdx => "tile",
            faction: FactionId => "faction",
            unit_type: UnitTypeId => "unit_type",
            padding: [u8; 1] => pad,
        };

        UnitStarved, "unit_starved", {
            tick: Tick => "tick",
            unit: u64 => "unit",
            deficit: Fix32 => "deficit",
            padding: [u8; 4] => pad,
        };

        SiteShortfall, "site_shortfall", {
            tick: Tick => "tick",
            site: u64 => "site",
            amount: Fix32 => "amount",
            commodity: u16 => "commodity",
            padding: [u8; 2] => pad,
        };

        SiteRationed, "site_rationed", {
            tick: Tick => "tick",
            site: u64 => "site",
            demanded: Accum => "demanded",
            granted: Accum => "granted",
            commodity: u16 => "commodity",
            padding: [u8; 6] => pad,
        };

        UnitPromoted, "unit_promoted", {
            tick: Tick => "tick",
            unit: u64 => "unit",
            character: u64 => "character",
            deeds: u64 => "deeds",
            faction: FactionId => "faction",
            padding: [u8; 6] => pad,
        };

        RelationCrossed, "relation_crossed", {
            tick: Tick => "tick",
            from_faction: FactionId => "from_faction",
            to_faction: FactionId => "to_faction",
            band_before: u8 => "band_before",
            band_after: u8 => "band_after",
            padding: [u8; 2] => pad,
        };

        UnitConverted, "unit_converted", {
            tick: Tick => "tick",
            unit: u64 => "unit",
            tile: TileIdx => "tile",
            from: FactionId => "from_faction",
            to: FactionId => "to_faction",
        };

        TradeSpoken, "trade_spoken", {
            tick: Tick => "tick",
            proposer: u16 => "proposer",
            responder: u16 => "responder",
            act: u8 => "act",
            status: u8 => "status",
            padding: [u8; 2] => pad,
        };

        UpgradeCollapsed, "upgrade_collapsed", {
            tick: Tick => "tick",
            tile: TileIdx => "tile",
            holder: Holder => "holder",
            category: u8 => "category",
            level: u8 => "level",
            cause: WearCause => "cause",
            padding: [u8; 7] => pad,
        };

        UpgradeFinished, "upgrade_finished", {
            tick: Tick => "tick",
            tile: TileIdx => "tile",
            holder: Holder => "holder",
            category: u8 => "category",
            level: u8 => "level",
        };

        SettlementFounded, "settlement_founded", {
            tick: Tick => "tick",
            settlement: u64 => "settlement",
            tile: TileIdx => "tile",
            faction: FactionId => "faction",
            padding: [u8; 2] => pad,
        };

        CampaignEvent, "campaign_event", {
            tick: Tick => "tick",
            objective_tile: u32 => "objective_tile",
            cohort_size: u32 => "cohort_size",
            faction: FactionId => "faction",
            kind: u8 => "kind",
            objective_kind: u8 => "objective_kind",
            padding: [u8; 4] => pad,
        };
    }
}

pub use declarations::declared_event_layouts;
