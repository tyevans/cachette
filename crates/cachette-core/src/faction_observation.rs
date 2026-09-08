//! One flat array for one faction, and the schema that declares its layout.
//!
//! A learner reads one array on every decision, and it trains a function of
//! that array's layout. The engine therefore returns one flat array of signed
//! integers for one faction, and one schema that says where every field of it
//! starts.[^1] [^2]
//!
//! # The schema is the only declaration of the layout
//!
//! The schema names each field, the position where the field starts, the
//! number of positions it holds, its integer width, and the lower and upper
//! bound of each of those positions.[^1] The engine builds the schema and the
//! array from one field list, and the writer indexes the array through the
//! schema. **No file outside this module states a position.** A caller that
//! needs one asks for the schema.
//!
//! A field of the array is contiguous. A field that holds one position for
//! each cell of the lattice holds those positions in ascending cell order,
//! and the next field starts after the last of them. A caller therefore
//! reaches cell `c` of a field by adding `c` to the start of that field, and
//! it needs no stride and no second table.
//!
//! # Every bound comes from the world parameters
//!
//! The length of the array is a function of the faction count, the tile
//! count, the reserved unit count, the board size and the cell count. **No
//! length and no bound follows the population.**[^2] A faction that loses
//! every unit reads an array of the same length as a faction that holds a
//! million.
//!
//! A bound that the width of the value alone limits reads as the whole range
//! of the width. The engine states no tighter bound for a running fixed-point
//! total, because a tighter one would be a measured figure and a blocker
//! governs every measured figure of this project.[^3]
//!
//! # What the array says about a place the faction has never seen
//!
//! A flat array cannot refuse one position, so it answers with zero and it
//! says how much of the cell it holds.[^4] Two positions of each cell carry
//! that: one counts the tiles the faction sees this frame, and one counts the
//! tiles it has ever seen. A cell whose two counts are zero states nothing at
//! all, and every other position of that cell is zero. A caller therefore
//! tells an unobserved cell from an empty one, in the same way the tile
//! reader lets it tell the two apart.[^4]
//!
//! The engine applies the sight rule inside the reader. **No argument widens
//! the answer.**[^5]
//!
//! # The quantity a win reader compares
//!
//! The standing of a faction reports the work toward a victory claim. The
//! wonder reader compares the claim itself, and the two are not the same
//! quantity.[^6] A learner reads this array, so the array carries both: the
//! work, and the claim the reader compares.
//!
//! # Every faction-indexed field is addressed relative to the reader
//!
//! A field that holds one position for each faction is addressed by the
//! distance from the faction that reads, and never by a seat number.[^10]
//! Position zero of such a field names the reader itself. Position `k` names
//! the faction `k` seats after the reader, counting round the seats. The
//! reader therefore finds its own quantities at one place, whatever seat it
//! holds, and it finds its first rival at the next place.
//!
//! **A seat number in the array would make the array mean two things.** A
//! league seats one policy in one seat for one game and in another seat for
//! the next, so a policy that learned a seat number reads another faction's
//! quantities under the same weight.[^11] The array was never wrong; it was
//! addressed inconsistently between readers, and a policy cannot learn a
//! relation from that.
//!
//! The order is a rotation and not a sort. A sort by a game quantity would
//! give a policy a meaningful order, and it would also move a rival between
//! positions from one decision to the next, because the quantity moves.[^10]
//!
//! # Which space every index lives in
//!
//! The map block is indexed by the cell index of the block lattice. The fog
//! layer, the summary level and this array share that one lattice, so a cell
//! index here is a fog block index and a summary cell index.[^7] **That
//! lattice carries no margin.** A cell index maps onto a world address with
//! no offset, by the two divisions the lattice states. A lattice that carries
//! a margin has two address spaces, and a reader that confuses them samples
//! the wrong cells and fails nowhere.[^8]
//!
//! A cell at the edge of the world covers fewer tiles than a whole block,
//! because the edge cuts it. One position of each cell states how many tiles
//! the cell covers, so a caller never divides by the wrong count.
//!
//! # What the read costs
//!
//! The standing and the boards come from the aggregates the engine already
//! keeps, and they walk no cell.
//!
//! **A cell the faction has never seen a tile of costs no tile work.** The
//! block form of the fog layer answers for the whole block, so the walk runs
//! over the cells the faction observed and never over the whole lattice. The
//! reader reads the layer of the faction once, not once for each cell.
//!
//! **A cell the faction has observed part of costs a walk over the tiles of
//! that cell.** The record asks the reader to start no pass over the tiles,
//! and to take the summary the pyramid rebuilt instead.[^2] The rebuilt cell
//! counts tiles the faction has not seen, so this reader cannot take it
//! without stating what those tiles hide. The walk therefore stands, and it
//! is bounded by the ground the faction has walked rather than by the world.
//! A derived per-faction level would remove it, and that level is state the
//! step would carry for every faction, whether or not a learner reads it.
//!
//! # Determinism
//!
//! One pass builds the array, on the calling thread. It visits the observed
//! cells in ascending cell order, which the fog layer already holds. Nothing
//! reads a thread and nothing reads a completion order.[^9]
//!
//! # References
//!
//! [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
//! [^4]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^6]: Findings register, FND-568. `docs/FINDINGS.md`
//! [^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^8]: Findings register, FND-569. `docs/FINDINGS.md`
//! [^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^10]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
//! [^11]: Findings register, FND-647. `docs/FINDINGS.md`

use crate::controller::WEIGHT_COUNT;
use crate::event_layout::ColumnKind;
use crate::event_memory::{Decay, BLAMED_KIND_COUNT, KIND_COUNT};
use crate::faction_memory_observation as memory;
use crate::faction_view::{Admit, BlockMask, FactionViewError};
use crate::types::{FactionId, Fix32};
use crate::world::World;

/// The version of the observation layout.
///
/// A field added, removed, relengthened or rebounded changes the meaning of a
/// stored weight file, so the engine carries this integer beside the schema.
/// A learner that loads a policy under another version must stop.[^1]
///
/// Raise this number whenever the field list, a length rule or a bound rule
/// changes.
///
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
pub const OBSERVATION_VERSION: u32 = 5;

/// Returns the faction that one position of a faction-indexed field names.
///
/// **A faction-indexed field is addressed by the distance from the reader,
/// and never by a seat number.** Position zero names the reader. Position
/// `offset` names the faction `offset` seats after the reader, counting round
/// the seats of the world.[^1]
///
/// This function is the only statement of that mapping inside the engine.
/// Every writer of a faction-indexed field calls it, so no field can address
/// its positions its own way.[^2]
///
/// **It is not public, because no caller outside the engine reads it yet.** A
/// caller that decodes the array holds the seat of the reader at the faction
/// field, and the rule is one addition and one remainder. Publishing a reader
/// that nothing invokes would ship an inert capability.[^3]
///
/// A faction count of zero has no seat to name, so the answer is the reader.
///
/// # References
///
/// [^1]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^3]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
const fn faction_at_offset(reader: FactionId, offset: u32, faction_count: u32) -> FactionId {
    if faction_count == 0 {
        return reader;
    }
    FactionId(((reader.0 as u32 + offset) % faction_count) as u16)
}

/// One field of the observation array.
///
/// **This list is the whole layout.** The schema derives every start from it,
/// and the writer fills the array through a match that the compiler checks
/// for coverage. A field added here reaches both without a second edit.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObsField {
    /// The tick the world stands at.
    Tick,
    /// The tick the run stops at, or zero when no limit is set.
    TickLimit,
    /// The faction that reads the array.
    Faction,
    /// One when the engine recorded a game end, and zero before it.
    GameOver,
    /// The tiles the faction holds. The territory reader compares it.
    HeldTiles,
    /// The seats the faction holds, its own and every rival's.
    SeatsHeld,
    /// The units of the faction that are alive.
    LiveUnits,
    /// The people the faction holds.
    Population,
    /// The stores of every settlement of the faction, as raw Q16.16.
    StoreTotal,
    /// The highest renown of a live character, as raw Q16.16.
    BestRenown,
    /// The work done toward the furthest wonder on ground the faction holds.
    WonderProgress,
    /// The victory claim that stands on ground the faction holds.
    ///
    /// **The wonder reader compares this, and it does not compare the work.**
    /// A learner that read the work alone could not see the thing that ends
    /// its game.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-568. `docs/FINDINGS.md`
    WonderClaim,
    /// The relation of the faction toward each faction, by the distance from
    /// the reader.
    ///
    /// Position zero holds the relation of the reader toward itself. Position
    /// `k` holds its relation toward the faction `k` seats after it. **No
    /// position of this field names a seat number.**[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
    Relation,
    /// The good each board row offers, over every faction.
    ///
    /// The field holds one block of board rows for each faction, and the
    /// blocks run in the order the relation field runs: block zero is the
    /// board of the reader, and block `k` is the board of the faction `k`
    /// seats after it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
    BoardGood,
    /// The quantity each board row offers, in the block order the good field
    /// states.
    BoardQuantity,
    /// The good each board row asks for, in the block order the good field
    /// states.
    BoardWants,
    /// The good each board row wants paid in, in the block order the good
    /// field states.
    BoardAskingGood,
    /// The quantity each board row asks for, in the block order the good
    /// field states.
    BoardAskingQuantity,
    /// The weight vector of the faction, in the order the vector declares.
    Weight,
    /// The tiles of each cell the faction sees this frame.
    CellSeenNow,
    /// The tiles of each cell the faction has ever seen.
    CellSeenEver,
    /// The tiles of the world that each cell covers.
    CellTiles,
    /// The observed tiles of each cell whose ground admits a unit.
    CellOpenTiles,
    /// The units of the faction on the tiles of each cell it sees now.
    ///
    /// **The pair of unit counts is relative to the faction that reads.** A
    /// field with one position for each faction multiplies the world by the
    /// faction count, and the record refuses one.[^1] A reader that wants
    /// the units of a named rival cannot have them, and a reader that wants
    /// to tell its own army from an invading one reads this pair.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: Findings register, FND-636. `docs/FINDINGS.md`
    CellOwnUnits,
    /// The units of every other faction on the tiles of each cell the
    /// faction sees now.
    ///
    /// An ally and an invader both count here. The relation field separates
    /// the two, and a relation is not a property of a tile.
    CellOtherUnits,
    /// The tiles of each cell the faction holds, over the tiles seen now.
    CellOwnHeldTiles,
    /// The tiles of each cell another faction holds, over the tiles seen
    /// now.
    ///
    /// A tile that nobody holds counts in neither this field nor the own
    /// field, so the two do not sum to the observed tile count.
    CellOtherHeldTiles,
    /// The value of the observed tiles of each cell, as raw Q16.16.
    CellValueTotal,
    /// The height of the observed tiles of each cell, as raw Q16.16.
    CellHeightTotal,
    /// The food of the observed tiles of each cell, as raw Q16.16.
    CellFoodTotal,
    /// The share of its own stock that each kind of event moved lately, over
    /// the short memory.
    ///
    /// **The array is a snapshot, and this is the memory beside it.** A
    /// snapshot cannot tell a faction that is gaining ground from one that is
    /// losing it, and it cannot say that a rival is taking a city now.[^1]
    ///
    /// Each position holds one kind of event, in the order the kind list
    /// declares. The value is the part of the stock of the reader that the
    /// events of one step move, so a small world and a large one with the
    /// same event rate read the same value.[^2]
    ///
    /// # References
    ///
    /// [^1]: The event history. [`crate::event_memory`]
    /// [^2]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    MemoryRecent,
    /// The same share of each kind of event, over the long memory.
    MemoryLasting,
    /// The signed relation between the short memory and the long memory of
    /// each kind.
    ///
    /// **This is the position that separates a spike from a trend.** Both
    /// memories reach the same share for the same constant arrival rate, so
    /// this reads zero while the rate holds, positive while it rises and
    /// negative while it falls. A rival that takes a city this minute raises
    /// the short memory alone. A rival that keeps killing the people of the
    /// reader raises both.
    MemoryTrend,
    /// The share of each kind of event that the single worst rival caused,
    /// over the long memory.
    ///
    /// The field holds one position for each kind that names a faction as its
    /// cause. **No position names a seat.** A league seats one policy in
    /// different seats between games, so a policy that learned a seat number
    /// would read another faction's quantities under the same weight.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-647. `docs/FINDINGS.md`
    MemoryWorstRival,
    /// How concentrated the cause of each kind of event is over the rivals,
    /// over the long memory.
    ///
    /// The value is the sum of the squared rival shares. Two rivals in equal
    /// measure give one half, and one rival that does everything gives one. A
    /// reader tells one enemy from a field of them by this position alone.
    MemoryConcentration,
}

impl ObsField {
    /// Every field, in the order the array holds them.
    pub const ALL: [Self; 35] = [
        Self::Tick,
        Self::TickLimit,
        Self::Faction,
        Self::GameOver,
        Self::HeldTiles,
        Self::SeatsHeld,
        Self::LiveUnits,
        Self::Population,
        Self::StoreTotal,
        Self::BestRenown,
        Self::WonderProgress,
        Self::WonderClaim,
        Self::Relation,
        Self::BoardGood,
        Self::BoardQuantity,
        Self::BoardWants,
        Self::BoardAskingGood,
        Self::BoardAskingQuantity,
        Self::Weight,
        Self::CellSeenNow,
        Self::CellSeenEver,
        Self::CellTiles,
        Self::CellOpenTiles,
        Self::CellOwnUnits,
        Self::CellOtherUnits,
        Self::CellOwnHeldTiles,
        Self::CellOtherHeldTiles,
        Self::CellValueTotal,
        Self::CellHeightTotal,
        Self::CellFoodTotal,
        Self::MemoryRecent,
        Self::MemoryLasting,
        Self::MemoryTrend,
        Self::MemoryWorstRival,
        Self::MemoryConcentration,
    ];

    /// Returns the name of the field.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Tick => "tick",
            Self::TickLimit => "tick_limit",
            Self::Faction => "faction",
            Self::GameOver => "game_over",
            Self::HeldTiles => "held_tiles",
            Self::SeatsHeld => "seats_held",
            Self::LiveUnits => "live_units",
            Self::Population => "population",
            Self::StoreTotal => "store_total",
            Self::BestRenown => "best_renown",
            Self::WonderProgress => "wonder_progress",
            Self::WonderClaim => "wonder_claim",
            Self::Relation => "relation",
            Self::BoardGood => "board_good",
            Self::BoardQuantity => "board_quantity",
            Self::BoardWants => "board_wants",
            Self::BoardAskingGood => "board_asking_good",
            Self::BoardAskingQuantity => "board_asking_quantity",
            Self::Weight => "weight",
            Self::CellSeenNow => "cell_seen_now",
            Self::CellSeenEver => "cell_seen_ever",
            Self::CellTiles => "cell_tiles",
            Self::CellOpenTiles => "cell_open_tiles",
            Self::CellOwnUnits => "cell_own_units",
            Self::CellOtherUnits => "cell_other_units",
            Self::CellOwnHeldTiles => "cell_own_held_tiles",
            Self::CellOtherHeldTiles => "cell_other_held_tiles",
            Self::CellValueTotal => "cell_value_total",
            Self::CellHeightTotal => "cell_height_total",
            Self::CellFoodTotal => "cell_food_total",
            Self::MemoryRecent => "memory_recent",
            Self::MemoryLasting => "memory_lasting",
            Self::MemoryTrend => "memory_trend",
            Self::MemoryWorstRival => "memory_worst_rival",
            Self::MemoryConcentration => "memory_concentration",
        }
    }

    /// Returns how many positions the field holds, for one world shape.
    const fn positions(self, shape: WorldShape) -> u32 {
        match self {
            Self::Tick
            | Self::TickLimit
            | Self::Faction
            | Self::GameOver
            | Self::HeldTiles
            | Self::SeatsHeld
            | Self::LiveUnits
            | Self::Population
            | Self::StoreTotal
            | Self::BestRenown
            | Self::WonderProgress
            | Self::WonderClaim => 1,
            Self::Relation => shape.faction_count,
            Self::BoardGood
            | Self::BoardQuantity
            | Self::BoardWants
            | Self::BoardAskingGood
            | Self::BoardAskingQuantity => shape.faction_count * shape.board_rows,
            Self::Weight => WEIGHT_COUNT,
            Self::CellSeenNow
            | Self::CellSeenEver
            | Self::CellTiles
            | Self::CellOpenTiles
            | Self::CellOwnUnits
            | Self::CellOtherUnits
            | Self::CellOwnHeldTiles
            | Self::CellOtherHeldTiles
            | Self::CellValueTotal
            | Self::CellHeightTotal
            | Self::CellFoodTotal => shape.cell_count,
            Self::MemoryRecent | Self::MemoryLasting | Self::MemoryTrend => KIND_COUNT as u32,
            Self::MemoryWorstRival | Self::MemoryConcentration => BLAMED_KIND_COUNT as u32,
        }
    }

    /// Returns the lowest and the highest value each position may hold.
    const fn bounds(self, shape: WorldShape) -> (i64, i64) {
        let byte = 255;
        let whole = u32::MAX as i64;
        match self {
            Self::Tick | Self::TickLimit | Self::WonderProgress => (0, i64::MAX),
            Self::Faction => (0, shape.faction_count as i64 - 1),
            Self::GameOver | Self::WonderClaim => (0, 1),
            Self::HeldTiles => (0, shape.tile_count as i64),
            Self::SeatsHeld => (0, shape.faction_count as i64),
            Self::LiveUnits | Self::Population | Self::CellOwnUnits | Self::CellOtherUnits => {
                (0, shape.unit_capacity as i64)
            }
            Self::StoreTotal => (0, i64::MAX),
            Self::BestRenown => (0, Fix32::MAX.0 as i64),
            Self::Relation => (i32::MIN as i64, i32::MAX as i64),
            Self::BoardGood | Self::BoardWants | Self::BoardAskingGood | Self::Weight => (0, byte),
            Self::BoardQuantity | Self::BoardAskingQuantity => (0, whole),
            Self::CellSeenNow
            | Self::CellSeenEver
            | Self::CellTiles
            | Self::CellOpenTiles
            | Self::CellOwnHeldTiles
            | Self::CellOtherHeldTiles => (0, shape.cell_tiles as i64),
            Self::CellValueTotal | Self::CellFoodTotal => {
                (-shape.cell_accumulator(), shape.cell_accumulator())
            }
            Self::CellHeightTotal => (0, shape.cell_accumulator()),
            Self::MemoryRecent
            | Self::MemoryLasting
            | Self::MemoryWorstRival
            | Self::MemoryConcentration => (0, memory::ONE),
            Self::MemoryTrend => (-memory::ONE, memory::ONE),
        }
    }
}

/// The world parameters that fix every length and every bound.
///
/// **Not one of these follows the population.** The reserved unit count is
/// the count the world reserved at construction, and it does not change when
/// a unit lives or dies.[^1]
///
/// # References
///
/// [^1]: ADR-0084, the world reserves the unit columns at construction, decision D1. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorldShape {
    faction_count: u32,
    tile_count: u32,
    unit_capacity: u32,
    board_rows: u32,
    cell_count: u32,
    cell_tiles: u32,
}

impl WorldShape {
    /// Reads the shape of one world.
    fn of(world: &World) -> Self {
        let layout = world.observation().layout();
        let edge = layout.block_edge();
        Self {
            faction_count: u32::from(world.faction_count().max(1)),
            tile_count: world.grid().tile_count(),
            unit_capacity: world.config().unit_capacity,
            board_rows: u32::from(world.board_rows()),
            cell_count: layout.block_count(),
            cell_tiles: edge.saturating_mul(edge),
        }
    }

    /// Returns the widest a fixed-point total over one cell reaches.
    ///
    /// A cell holds at most one whole block of tiles, and one tile carries a
    /// fixed-point value of at most the width of that value. The product of
    /// the two is a bound that the world parameters give, and it is not a
    /// measured figure.
    const fn cell_accumulator(self) -> i64 {
        (self.cell_tiles as i64).saturating_mul(Fix32::MAX.0 as i64)
    }
}

/// One row of the observation schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldRow {
    /// The field the row describes.
    pub field: ObsField,
    /// The position the field starts at.
    pub start: u32,
    /// The positions the field holds.
    pub positions: u32,
    /// The lowest value any position of the field may hold.
    pub low: i64,
    /// The highest value any position of the field may hold.
    pub high: i64,
}

impl FieldRow {
    /// Returns the name of the field.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.field.name()
    }

    /// Returns how a reader reads each position of the field.
    ///
    /// Every position of the array is a signed eight-byte integer. The array
    /// holds no floating point number, because the simulation holds none.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn kind(&self) -> ColumnKind {
        ColumnKind::I64
    }

    /// Returns the width of one position, in bytes.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.kind().width()
    }
}

/// The declared layout of the observation array of one world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationSchema {
    version: u32,
    length: u32,
    rows: Vec<FieldRow>,
}

impl ObservationSchema {
    /// Returns the version of the layout.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Returns how many positions the whole array holds.
    #[must_use]
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// Returns one row for each field, in the order the array holds them.
    #[must_use]
    pub fn rows(&self) -> &[FieldRow] {
        &self.rows
    }

    /// Returns the row of one field, by name.
    #[must_use]
    pub fn row(&self, name: &str) -> Option<FieldRow> {
        self.rows.iter().copied().find(|row| row.name() == name)
    }
}

/// What one cell of the lattice contributes to the array.
#[derive(Clone, Copy, Debug, Default)]
struct CellRow {
    seen_now: i64,
    seen_ever: i64,
    tiles: i64,
    open_tiles: i64,
    own_units: i64,
    other_units: i64,
    own_held_tiles: i64,
    other_held_tiles: i64,
    value_total: i64,
    height_total: i64,
    food_total: i64,
}

impl World {
    /// Returns the declared layout of the observation array of this world.
    ///
    /// The schema names each field, where it starts, how many positions it
    /// holds, its integer width, and the bounds of each position.[^1] The
    /// engine builds the schema and the array from one field list, so a
    /// caller that decodes by arithmetic over this schema cannot disagree
    /// with the array.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    #[must_use]
    pub fn observation_schema(&self) -> ObservationSchema {
        let shape = WorldShape::of(self);
        let mut rows = Vec::with_capacity(ObsField::ALL.len());
        let mut start = 0u32;
        for field in ObsField::ALL {
            let positions = field.positions(shape);
            let (low, high) = field.bounds(shape);
            rows.push(FieldRow {
                field,
                start,
                positions,
                low,
                high,
            });
            start = start.saturating_add(positions);
        }
        ObservationSchema {
            version: OBSERVATION_VERSION,
            length: start,
            rows,
        }
    }

    /// Returns the observation of one faction, as one flat array.
    ///
    /// The array holds what that faction observes, and nothing else. **No
    /// argument widens the answer.** A caller that wants the truth of the
    /// world calls a reader that names no faction.[^1]
    ///
    /// The schema of the same world declares where every field of the array
    /// sits.[^2] The length is a function of the world parameters and never
    /// of the population.[^3]
    ///
    /// A cell the faction has never seen a tile of reads as zero in every
    /// position, and the two count positions of that cell state that the
    /// faction holds none of it.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction of this world, and
    /// when the derived unit structure does not describe the units. **The
    /// error names the cause.** A stale structure names the revision it holds
    /// and the revision the arena holds.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^4]: Findings register, FND-647. `docs/FINDINGS.md`
    pub fn faction_observation(&self, faction: FactionId) -> Result<Vec<i64>, FactionViewError> {
        let standing = self
            .standing(faction)
            .ok_or(FactionViewError::NoSuchFaction(faction))?;
        let schema = self.observation_schema();
        let shape = WorldShape::of(self);
        let cells = self.observed_cells(faction)?;

        let claim = self
            .victory_claims()
            .get(usize::from(faction.0))
            .map_or(0, |pair| pair.0);
        let weights = self
            .faction_weights(faction)
            .ok_or(FactionViewError::NoSuchFaction(faction))?;
        let weight_bytes = [
            i64::from(weights.war),
            i64::from(weights.trade),
            i64::from(weights.build),
            i64::from(weights.renown),
            i64::from(weights.settle),
        ];

        let mut out = vec![0i64; schema.length() as usize];
        for row in schema.rows() {
            let first = row.start as usize;
            let span = &mut out[first..first + row.positions as usize];
            match row.field {
                ObsField::Tick => span[0] = widen(self.tick().0),
                ObsField::TickLimit => span[0] = widen(self.tick_limit()),
                ObsField::Faction => span[0] = i64::from(faction.0),
                ObsField::GameOver => span[0] = i64::from(self.game_end().is_set()),
                ObsField::HeldTiles => span[0] = standing.held_tiles,
                ObsField::SeatsHeld => span[0] = standing.seats_held,
                ObsField::LiveUnits => span[0] = standing.live_units,
                ObsField::Population => span[0] = i64::from(self.population_of(faction)),
                ObsField::StoreTotal => span[0] = standing.store_total,
                ObsField::BestRenown => span[0] = standing.best_renown,
                ObsField::WonderProgress => span[0] = standing.wonder_progress,
                ObsField::WonderClaim => span[0] = claim,
                ObsField::Relation => {
                    for (offset, place) in span.iter_mut().enumerate() {
                        let other = faction_at_offset(faction, offset as u32, shape.faction_count);
                        *place = i64::from(self.relation(faction, other).unwrap_or(0));
                    }
                }
                ObsField::BoardGood
                | ObsField::BoardQuantity
                | ObsField::BoardWants
                | ObsField::BoardAskingGood
                | ObsField::BoardAskingQuantity => {
                    for offset in 0..shape.faction_count {
                        let other = faction_at_offset(faction, offset, shape.faction_count);
                        let board = self.market(other);
                        let base = (offset * shape.board_rows) as usize;
                        for (offset, advert) in board.iter().enumerate() {
                            let Some(place) = span.get_mut(base + offset) else {
                                continue;
                            };
                            *place = match row.field {
                                ObsField::BoardGood => i64::from(advert.good),
                                ObsField::BoardQuantity => i64::from(advert.quantity),
                                ObsField::BoardWants => i64::from(advert.wants),
                                ObsField::BoardAskingGood => i64::from(advert.asking_good),
                                _ => i64::from(advert.asking_quantity),
                            };
                        }
                    }
                }
                ObsField::Weight => {
                    for (place, value) in span.iter_mut().zip(weight_bytes) {
                        *place = value;
                    }
                }
                ObsField::CellSeenNow => scatter(span, &cells, |cell| cell.seen_now),
                ObsField::CellSeenEver => scatter(span, &cells, |cell| cell.seen_ever),
                ObsField::CellTiles => scatter(span, &cells, |cell| cell.tiles),
                ObsField::CellOpenTiles => scatter(span, &cells, |cell| cell.open_tiles),
                ObsField::CellOwnUnits => scatter(span, &cells, |cell| cell.own_units),
                ObsField::CellOtherUnits => scatter(span, &cells, |cell| cell.other_units),
                ObsField::CellOwnHeldTiles => {
                    scatter(span, &cells, |cell| cell.own_held_tiles);
                }
                ObsField::CellOtherHeldTiles => {
                    scatter(span, &cells, |cell| cell.other_held_tiles);
                }
                ObsField::CellValueTotal => scatter(span, &cells, |cell| cell.value_total),
                ObsField::CellHeightTotal => scatter(span, &cells, |cell| cell.height_total),
                ObsField::CellFoodTotal => scatter(span, &cells, |cell| cell.food_total),
                ObsField::MemoryRecent => {
                    memory::write_shares(self, faction, Decay::Recent, span);
                }
                ObsField::MemoryLasting => {
                    memory::write_shares(self, faction, Decay::Lasting, span);
                }
                ObsField::MemoryTrend => memory::write_trends(self, faction, span),
                ObsField::MemoryWorstRival => memory::write_worst_rival(self, faction, span),
                ObsField::MemoryConcentration => {
                    memory::write_concentration(self, faction, span);
                }
            }
        }
        Ok(out)
    }

    /// Builds one row for each cell of the lattice, over what one faction
    /// observes.
    ///
    /// **This is the whole-set read, and it is not a per-cell read in a
    /// loop.** It reads the two layers of the faction once. It then visits
    /// only the cells that at least one of the two layers names, in ascending
    /// cell order. A cell that neither layer names keeps the row it was built
    /// with, which states no tile and no value, and it costs no tile work.
    ///
    /// The tile count of a cell is a property of the lattice and not of the
    /// faction, so every row carries it.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// units, and when the world holds no tile at an address of a cell.
    fn observed_cells(&self, faction: FactionId) -> Result<Vec<CellRow>, FactionViewError> {
        let observation = self.observation();
        let count = observation.layout().block_count();
        let mut cells = vec![CellRow::default(); count as usize];
        for (block, cell) in cells.iter_mut().enumerate() {
            cell.tiles = i64::from(observation.tiles_in_block(block as u32));
        }

        let visible = observation.visible_layer(faction);
        let remembered = observation.remembered_layer(faction);
        let observed = merge(
            visible.map_or(&[][..], |layer| layer.populated_blocks()),
            remembered.map_or(&[][..], |layer| layer.populated_blocks()),
        );
        for block in observed {
            let mask = BlockMask::new(
                visible.and_then(|layer| layer.block(block)),
                remembered.and_then(|layer| layer.block(block)),
            );
            let masked = self.masked_block(&mask, faction, block, Admit::SeenEver)?;
            let summary = masked.summary();
            let Some(cell) = cells.get_mut(block as usize) else {
                continue;
            };
            cell.seen_now = i64::from(visible.map_or(0, |layer| layer.block_population(block)));
            cell.seen_ever = masked.admitted();
            cell.open_tiles = summary.open_tiles();
            cell.own_units = masked.own_units();
            cell.other_units = masked.other_units();
            cell.own_held_tiles = masked.own_held_tiles();
            cell.other_held_tiles = masked.other_held_tiles();
            cell.value_total = summary.value_total().0;
            cell.height_total = summary.height_total().0;
            cell.food_total = summary.food_total().0;
        }
        Ok(cells)
    }
}

/// Writes one field of every cell into its own contiguous span.
fn scatter(span: &mut [i64], cells: &[CellRow], read: impl Fn(&CellRow) -> i64) {
    for (place, cell) in span.iter_mut().zip(cells) {
        *place = read(cell);
    }
}

/// Merges two ascending cell lists into one ascending list without repeats.
fn merge(first: &[u32], second: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(first.len() + second.len());
    let (mut left, mut right) = (0usize, 0usize);
    while left < first.len() && right < second.len() {
        match first[left].cmp(&second[right]) {
            std::cmp::Ordering::Less => {
                out.push(first[left]);
                left += 1;
            }
            std::cmp::Ordering::Greater => {
                out.push(second[right]);
                right += 1;
            }
            std::cmp::Ordering::Equal => {
                out.push(first[left]);
                left += 1;
                right += 1;
            }
        }
    }
    out.extend_from_slice(&first[left..]);
    out.extend_from_slice(&second[right..]);
    out
}

/// Widens an unsigned count to the signed width the array holds.
const fn widen(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}
