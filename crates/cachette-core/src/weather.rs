//! The weather field.
//!
//! Weather is water. The field holds two quantities over the level 1 cell
//! lattice: the water that stands in the air above a cell, and the water that
//! has fallen onto the ground of that cell. Both are counted in drops, and a
//! drop is a whole number. Nothing here is a rate and nothing here is a
//! fraction of anything.[^1]
//!
//! **The field lives on the level 1 cell lattice, not on the tile.** A meeting
//! between two factions resolves at the tile, because a cell covers a block of
//! tiles and a fight resolved at the cell kills units spread over all of
//! them.[^2] That argument does not carry over. Weather is larger than a
//! block, so a cell samples a field that varies slowly rather than smearing
//! events that are distinct. A field at tile pitch would also cost the whole
//! world on every frame, and the product record rejects that shape.[^3] [^4]
//!
//! # What the field conserves
//!
//! **A pass creates no water and destroys none.** The field holds two running
//! totals beside the two planes. The raised total counts every drop that has
//! ever entered the air, from the sea or from a god. The evaporated total
//! counts every drop that has left the ground. The account below is exact at
//! every moment, and a check reports it:
//!
//! ```text
//! raised == air total + ground total + evaporated
//! ```
//!
//! The spread moves water between neighbouring cells. It is written as a
//! gather: a cell computes what it keeps and what each neighbour hands it,
//! and no cell writes another cell. The quantity a giver loses is the same
//! integer that the receiver adds, because both sides compute it from the
//! same input plane with the same truncating division.[^5]
//!
//! # What the ground does to the water
//!
//! **High ground takes more water out of the air.** The share of the air that
//! falls on a cell rises with the mean height of that cell. **The sea puts
//! water into the air.** A cell draws once each frame, and the odds that it
//! lifts water follow the share of its tiles that admit no unit, which is the
//! water share of the cell.
//!
//! # Determinism
//!
//! **The solve runs a fixed number of spread passes.** It holds no
//! convergence test and no time budget.[^6]
//!
//! **Every draw is keyed on the tuple.** The weather system owns one system
//! identifier, and a lift draw keys on the frame and on the cell. One draw
//! serves one cell for one frame, in the way the contest draws once for each
//! tile rather than once for each unit.[^7]
//!
//! **A spread pass writes disjoint output.** It reads the whole of one plane
//! and writes a contiguous run of the other, so no two threads write one cell
//! and no atomic operation appears.[^8] [^9]
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0121, a meeting between two factions resolves at the tile, decision D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
//! [^3]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
//! [^4]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
//! [^5]: ADR-0141, a weather pass moves water and never scales it, decision D1. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
//! [^6]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
//! [^7]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^8]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOURS, NEIGHBOUR_COUNT};
use crate::holding::FactionMask;
use crate::pyramid::CellSummary;
use crate::rng;
use crate::sim_math;
use crate::types::{Accum, FactionId, Fix32, Tick, TileIdx, FACTION_CEILING};

/// The reason that the field refused a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherError {
    /// The caller asked for a solve at no threads.
    ZeroThreads,
    /// The faction count is above the ceiling the project supports.
    FactionCountAboveCeiling(u16),
    /// The summaries do not cover the cell lattice.
    LatticeMismatch,
    /// The caller named a faction that this world does not hold.
    NoSuchFaction(u16),
    /// The caller named a place outside the world.
    PlaceOutsideWorld(Axial),
    /// The caller named a place whose cell holds no ground of that faction.
    GroundNotHeld(Axial),
    /// The caller named more places than one call carries.
    TooManyPlaces(usize),
    /// The strength is zero, or above the ceiling.
    StrengthOutOfRange(u8),
    /// The faction inflicted weather too recently.
    StillCooling {
        /// The first tick at which the faction may inflict weather again.
        ready_at: Tick,
    },
}

impl core::fmt::Display for WeatherError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a weather solve needs at least one thread"),
            Self::FactionCountAboveCeiling(count) => write!(
                formatter,
                "the faction count {count} is above the ceiling {FACTION_CEILING}"
            ),
            Self::LatticeMismatch => {
                write!(formatter, "the summaries do not cover the cell lattice")
            }
            Self::NoSuchFaction(faction) => {
                write!(formatter, "this world holds no faction {faction}")
            }
            Self::PlaceOutsideWorld(place) => write!(
                formatter,
                "the place ({}, {}) is outside the world",
                place.q, place.r
            ),
            Self::GroundNotHeld(place) => write!(
                formatter,
                "the faction holds no ground in the cell that covers ({}, {})",
                place.q, place.r
            ),
            Self::TooManyPlaces(count) => write!(
                formatter,
                "one call carries {PLACES_CEILING} places, and the caller named {count}"
            ),
            Self::StrengthOutOfRange(strength) => write!(
                formatter,
                "the strength {strength} is outside the range 1 to {STRENGTH_CEILING}"
            ),
            Self::StillCooling { ready_at } => write!(
                formatter,
                "the faction may inflict weather again at tick {}",
                ready_at.0
            ),
        }
    }
}

impl std::error::Error for WeatherError {}

/// A quantity of water, counted in drops.
///
/// A drop is a whole number, and the unit is the same everywhere in this
/// module. The value is 64 bits wide, because a level 1 cell stands over a
/// block of tiles and the project widens an accumulator at that level.[^1]
///
/// The combine is saturating addition. It is exactly associative and
/// commutative, and its identity is zero, so a fold over a set of
/// contributions gives one answer whatever the order.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Drops(pub i64);

impl Drops {
    /// No water at all. It is the identity of the combine.
    pub const ZERO: Self = Self(0);

    /// Combines two quantities of water.
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Returns the quantity as an accumulator.
    #[must_use]
    pub const fn to_accum(self) -> Accum {
        Accum(self.0)
    }
}

/// A temperature, counted in whole degrees.
///
/// The scale is this module's own. Zero is the cold end of the range, and a
/// low cell over open water sits near [`TEMPERATURE_AT_SEA_LEVEL`]. The
/// value is a whole number and it is not a fixed-point fraction, because
/// simulated state holds no floating point number and this field enters the
/// state hash.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Degrees(pub i32);

impl Degrees {
    /// The cold end of the scale.
    pub const ZERO: Self = Self(0);
}

/// The wind over one cell, as a vector in the two-axis basis of the lattice.
///
/// The two components are the axial components of the lattice, and they are
/// whole numbers. The wind is not an angle, it is not a speed and a bearing,
/// and no part of it is a fraction of anything.[^1] [^2]
///
/// The type declares its layout, because the wind reaches the state hash.
/// Two `i32` fields fill eight bytes exactly, so the type needs no padding
/// field.[^3]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^3]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Wind {
    /// The component along the first axis of the lattice.
    pub q: i32,
    /// The component along the second axis of the lattice.
    pub r: i32,
}

impl Wind {
    /// No wind at all.
    pub const STILL: Self = Self { q: 0, r: 0 };

    /// Returns how much of this wind points along one of the six directions.
    ///
    /// The answer is a whole number. It is the dot product of this wind with
    /// the unit offset of that direction, taken in the three-axis form of the
    /// lattice, where the third axis is the negated sum of the other two. A
    /// wind of one step along a direction answers two for that direction, one
    /// for each direction beside it, and the negatives of both for the three
    /// on the other side.
    ///
    /// **The six answers cancel exactly.** The six directions are three
    /// opposite pairs, and the answer for a direction is the negative of the
    /// answer for its opposite, because the offsets are negatives of each
    /// other and the dot product is linear. The transport rule rests on
    /// that.[^1]
    ///
    /// A direction outside the six answers zero.
    ///
    /// **This is public so that a test can read the quantity the transport
    /// share is built from.**
    ///
    /// # References
    ///
    /// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
    #[must_use]
    pub const fn along(self, direction: usize) -> i32 {
        if direction >= NEIGHBOUR_COUNT {
            return 0;
        }
        let offset = NEIGHBOURS[direction];
        // The three-axis form of an axial pair is (q, -q - r, r). The dot
        // product of two of those, gathered onto the two stored components,
        // is the expression below.
        let first = offset.q - (-offset.q - offset.r);
        let second = offset.r - (-offset.q - offset.r);
        self.q * first + self.r * second
    }

    /// Returns this wind with both components inside the ceiling.
    #[must_use]
    const fn clamped(self) -> Self {
        Self {
            q: clamp_i32(self.q, -WIND_COMPONENT_CEILING, WIND_COMPONENT_CEILING),
            r: clamp_i32(self.r, -WIND_COMPONENT_CEILING, WIND_COMPONENT_CEILING),
        }
    }
}

/// Clamps a whole number into a closed range.
const fn clamp_i32(value: i32, low: i32, high: i32) -> i32 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

/// Clamps an accumulator-width whole number into a closed range.
const fn clamp_i64(value: i64, low: i64, high: i64) -> i64 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

/// The number of spread passes that one solve runs.
///
/// The count is fixed. A solve runs it whatever the field holds and whatever
/// the thread count. It is not a budget and no measurement chose it: it is
/// the reach that one frame of weather adds, in cells.[^1]
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
pub const PASSES_FOR_EACH_SOLVE: u32 = 2;

/// The number of wind passes that one solve runs.
///
/// The count is fixed. The solve reads no clock, tests no residual, and takes
/// no branch on what the field holds.[^1] The value is a balance row.[^2]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: Balance register, the weather section. `docs/reference/balance.md`
pub const WIND_PASSES_FOR_EACH_SOLVE: u32 = 2;

/// The denominator of the share of the air that one pass hands a neighbour.
const TRANSPORT_DENOMINATOR: i64 = 512;

/// The numerator of the share a cell hands a neighbour when it has no wind.
///
/// **This value carries the conservation proof.** The share a cell hands one
/// neighbour is this base plus the wind of that cell along that direction.
/// The six directions are three opposite pairs, and the wind along a
/// direction is the negative of the wind along its opposite, so the six
/// additions cancel exactly and the six numerators sum to six times this
/// base, whatever the wind. Six times this base is below the denominator, so
/// a cell never hands away more than it holds.[^1]
///
/// The base is at or above [`WIND_ALONG_CEILING`], so no numerator is ever
/// negative and no numerator is ever clamped. A clamp would break the
/// cancellation and the sum would rise above six times the base.
///
/// # References
///
/// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
const TRANSPORT_BASE_NUMERATOR: i64 = 36;

/// The denominator of the share of the air that falls in one solve.
const FALL_DENOMINATOR: i64 = 64;

/// The numerator of the share that falls on air that meets no cooling at all.
const FALL_NUMERATOR_FLOOR: i64 = 1;

/// The degrees of cooling that add one to the fall numerator.
const DEGREES_FOR_EACH_RAIN_STEP: i64 = 8;

/// The most that the cooling along the wind adds to the fall numerator.
const FALL_NUMERATOR_COOLING_CEILING: i64 = 14;

/// The degrees below the sea-level temperature that add one to the fall
/// numerator.
const DEGREES_FOR_EACH_COLD_STEP: i64 = 24;

/// The most that cold ground adds to the fall numerator.
const FALL_NUMERATOR_COLD_CEILING: i64 = 10;

/// The share of the water on the ground that leaves it in one solve.
///
/// The division truncates, so a cell holding fewer drops than this keeps
/// them. Ground that is barely wet stays barely wet until something moves it.
const DRY_DIVISOR: i64 = 32;

/// The drops that the sea lifts into the air over one cell, when it lifts.
///
/// The lift is the whole of this only at the top of the lift heat range. A
/// colder cell lifts a truncated share of it, and a cell at or below zero
/// degrees lifts nothing.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const LIFT_DROPS: i64 = 256;

/// The temperature at which a sea lift raises the whole of [`LIFT_DROPS`].
const LIFT_HEAT_DENOMINATOR: i64 = 256;

/// The denominator of the share of the ground water that the heat lifts.
///
/// A cell lifts its own temperature over this denominator, as a share of the
/// water on its ground. The water moves from the ground plane into the air
/// plane of the same cell, so it stays inside the account.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const GROUND_LIFT_DENOMINATOR: i64 = 3072;

/// The largest size that either component of a wind reaches.
///
/// The wind pass clamps both components into this range on every pass, so
/// the speed of the wind of any cell never passes the ceiling, whatever the
/// pressure difference and however long it stands.[^1] The value is a
/// balance row.[^2]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: Balance register, the weather section. `docs/reference/balance.md`
pub const WIND_COMPONENT_CEILING: i32 = 12;

/// The largest size that [`Wind::along`] reaches for a clamped wind.
///
/// The dot product of a wind with a unit direction gathers onto the two
/// stored components with whole coefficients, and the largest of those
/// coefficients is three in size for one component and none. So the size of
/// the answer never passes three times the component ceiling. **This is
/// derived from the component ceiling and never declared beside it**, and a
/// test walks the whole clamped range and both bounds to prove it.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const WIND_ALONG_CEILING: i32 = 3 * WIND_COMPONENT_CEILING;

/// The most that one wind pass changes either component of a wind.
///
/// The pressure difference accelerates the wind by a bounded step. It never
/// assigns the wind the pressure asks for, and the lag is the reason the
/// field carries the wind at all.[^1] The value is a balance row.[^2]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: Balance register, the weather section. `docs/reference/balance.md`
const WIND_ACCELERATION_STEP: i32 = 3;

/// The numerator of the share of the speed that the drag takes each pass.
const WIND_DRAG_NUMERATOR: i32 = 1;

/// The denominator of the share of the speed that the drag takes each pass.
///
/// The drag runs before the acceleration. Without it a wind under a steady
/// pressure difference would gain the same step every pass and never
/// stop.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
const WIND_DRAG_DENOMINATOR: i32 = 4;

/// The degrees of temperature difference that ask for one step of wind.
///
/// The pressure of a cell follows its heat, so the difference in temperature
/// across a cell is the pressure difference that accelerates the wind. Air
/// rises where it is hot, so the wind runs toward the warmer neighbour.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
const DEGREES_FOR_EACH_PRESSURE_STEP: i32 = 24;

/// The temperature that a cell at the bottom of the height range settles at,
/// before the season and the cloud.
pub const TEMPERATURE_AT_SEA_LEVEL: i32 = 256;

/// The degrees that the whole height range takes off the temperature.
const TEMPERATURE_FALL_FOR_FULL_HEIGHT: i32 = 224;

/// The degrees that open water adds to the temperature of a whole cell.
const TEMPERATURE_RISE_FOR_FULL_WATER: i32 = 32;

/// The ticks of one whole season cycle.
///
/// The cycle is the one driving term that varies with time. Without it the
/// heat of a cell is a function of the terrain alone, the terrain does not
/// move, and the wind settles into a field that never changes.[^1]
///
/// # References
///
/// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decision D2. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
pub const SEASON_TICKS: u64 = 512;

/// The degrees that the season adds at its warmest and takes at its coldest.
const SEASON_SWING: i32 = 96;

/// The drops in the air that cool a cell by one degree.
const DROPS_FOR_EACH_CLOUD_STEP: i64 = 24;

/// The most that the water in the air takes off the temperature of a cell.
const CLOUD_COOLING_CEILING: i32 = 96;

/// The denominator of the share of the gap that a cell closes in one pass.
///
/// The stored temperature moves toward the temperature the terrain, the
/// season and the sky ask for. It never takes that temperature outright, so
/// the field lags its driver and a warm parcel outlives the thing that
/// warmed it.[^1]
///
/// # References
///
/// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
const TEMPERATURE_RELAX_DIVISOR: i32 = 4;

/// The denominator of the share of a temperature gap that the wind carries.
///
/// A neighbour hands this cell a share of the difference between the two
/// temperatures, and the share rises with the part of that neighbour's wind
/// that points this way. Six neighbours at the wind ceiling hand less than
/// the whole difference, so the new temperature stays inside the range of
/// the old ones and the field cannot run away.
const TEMPERATURE_ADVECT_DENOMINATOR: i32 = 128;

/// The coldest temperature that the field holds.
const TEMPERATURE_FLOOR: i32 = -512;

/// The warmest temperature that the field holds.
const TEMPERATURE_CEILING: i32 = 512;

/// How rarely a cell of open water lifts.
///
/// A cell draws once each frame. It lifts when the draw, taken below the tile
/// count of the cell multiplied by this, falls below the number of tiles of
/// that cell that admit no unit. A cell that is all water therefore lifts on
/// one frame in this many, and a cell with no water never lifts.
const LIFT_PERIOD: u64 = 8;

/// The draw index of the lift draw within a frame.
///
/// The weather system takes one draw for each cell in each frame, so this is
/// the only index it uses.
const LIFT_DRAW: u32 = 0;

/// The water on the ground at which a cell counts as wet.
///
/// A wet cell yields more to a unit that gathers from it. The blocker that
/// asks what weather should be worth still holds the value, and the balance
/// register holds the row.[^1] [^2]
///
/// **The value is provisional and a measurement chose it.** A run of the
/// demonstration world showed the ground water of the sixty-four cells
/// running over a wide range, and rising and falling with the season. The
/// mark sits inside that range rather than below all of it, so some cells
/// are wet and some are dry at one tick. The measuring script reports the
/// distribution, and the commit body holds the reading.[^3]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: Balance register, the wet mark. `docs/reference/balance.md`
/// [^3]: The weather measuring script. `scripts/weather_travel.py`
pub const WET_MARK: Drops = Drops(192);

/// The largest strength that a god may inflict.
///
/// The strength is a small whole number and never a quantity of drops. A
/// caller that named the drops directly could put any amount of water into
/// the world at once.[^1]
///
/// # References
///
/// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D2. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
pub const STRENGTH_CEILING: u8 = 4;

/// The drops that one point of strength puts into the air over one cell.
const DROPS_FOR_EACH_STRENGTH: i64 = 4096;

/// The most places that one call may name.
///
/// The verb is set-valued, and this bound is what stops one call from
/// covering the world.[^1]
///
/// # References
///
/// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D3. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
pub const PLACES_CEILING: usize = 256;

/// The ticks that a faction waits between one storm and the next.
///
/// The value is a content constant that no measurement chose, and a blocker
/// holds the question of what the power should cost.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
pub const COOLDOWN_TICKS: u64 = 64;

/// Reports whether the sea over one cell lifts water on one frame.
///
/// The answer is a keyed draw. The key is the world seed, the weather system,
/// the frame, the cell and the draw index, so it holds no state and it does
/// not depend on which thread asked.[^1]
///
/// The bound of the draw is the tile count of the cell multiplied by the lift
/// period, and the cell lifts when the draw falls below the number of tiles
/// that admit no unit. Water is the only ground that admits no unit, so the
/// odds follow the water share of the cell exactly.[^2]
///
/// A cell that covers no tile never lifts, because the bound is then zero.
///
/// **This is public so that a test can change one field of the key and watch
/// the answer move.** A draw keyed on the wrong field gives the same wrong
/// answer on every thread and on every run, and neither determinism test can
/// see it.[^3]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: The terrain capacity table. `crates/cachette-core/src/terrain.rs`
/// [^3]: Testing rules, section 2. `.claude/rules/testing.md`
#[must_use]
pub fn cell_lifts(seed: u64, tick: Tick, cell: u32, tiles: i64, open_tiles: i64) -> bool {
    let sea = tiles.saturating_sub(open_tiles);
    if sea <= 0 || tiles <= 0 {
        return false;
    }
    let bound = (tiles as u64).saturating_mul(LIFT_PERIOD);
    let draw = rng::draw_below(
        seed,
        rng::SYSTEM_WEATHER,
        tick.0,
        u64::from(cell),
        LIFT_DRAW,
        bound,
    );
    draw < sea as u64
}

/// Returns the degrees that the season adds at one tick.
///
/// The cycle is a triangle. It rises from the coldest offset to the warmest
/// over half of the season, and it falls back over the other half. The
/// arithmetic is whole and the function reads no clock, so two worlds at one
/// tick agree.[^1]
///
/// **This is public so that a test can show the field changes over time.** A
/// temperature that varies only over space is a static field, and no test of
/// one tick can tell the two apart.
///
/// # References
///
/// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decision D2. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
#[must_use]
pub fn season_offset(tick: Tick) -> i32 {
    let half = SEASON_TICKS / 2;
    let phase = tick.0 % SEASON_TICKS;
    let climbed = if phase < half {
        phase
    } else {
        SEASON_TICKS - phase
    };
    let risen = sim_math::share(
        Accum(2 * i64::from(SEASON_SWING)),
        Accum(climbed as i64),
        Accum(half as i64),
    )
    .map_or(0, |value| value.0);
    sim_math::offset(-SEASON_SWING, risen as i32)
}

/// Returns the temperature that the world asks one cell to hold.
///
/// The answer is the sum of four terms. The terrain gives two: the mean
/// height of the cell takes degrees away, and the share of the cell that
/// holds open water adds them back. The season adds or takes degrees by the
/// tick. The water in the air over the cell takes degrees away, because
/// cloud stands between the ground and the sun.[^1]
///
/// **Nothing stores this.** The stored temperature moves toward it, and the
/// gap between the two is the memory of the field.
///
/// **This is public so that a test can read the driver apart from the
/// field.**
///
/// # References
///
/// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decisions D1 and D2. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
#[must_use]
pub fn asked_temperature(summary: CellSummary, air: Drops, tick: Tick) -> Degrees {
    let height = summary.mean_height().unwrap_or(Fix32::ZERO);
    let fall = degrees_for_share(TEMPERATURE_FALL_FOR_FULL_HEIGHT, height);
    let water = summary.open_share().unwrap_or(Fix32::ZERO);
    let rise = degrees_for_share(TEMPERATURE_RISE_FOR_FULL_WATER, water);
    let cloud = clamp_i64(
        air.0 / DROPS_FOR_EACH_CLOUD_STEP,
        0,
        i64::from(CLOUD_COOLING_CEILING),
    ) as i32;
    let mut degrees = sim_math::offset(TEMPERATURE_AT_SEA_LEVEL, -fall);
    degrees = sim_math::offset(degrees, rise);
    degrees = sim_math::offset(degrees, season_offset(tick));
    degrees = sim_math::offset(degrees, -cloud);
    Degrees(clamp_i32(degrees, TEMPERATURE_FLOOR, TEMPERATURE_CEILING))
}

/// Returns the part of a whole number of degrees that a fixed-point share
/// earns.
///
/// The share is a fraction of one, so the arithmetic is a multiply and a
/// truncating divide through the arithmetic module.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn degrees_for_share(degrees: i32, share: Fix32) -> i32 {
    let clamped = clamp_i32(share.0, 0, Fix32::ONE.0);
    sim_math::share(
        Accum(i64::from(degrees)),
        Accum(i64::from(clamped)),
        Accum(i64::from(Fix32::ONE.0)),
    )
    .map_or(0, |value| value.0 as i32)
}

/// Returns the numerator of the share that a cell hands one neighbour.
///
/// The answer is the base numerator plus the part of the wind of the giver
/// that points that way. The base is at or above the ceiling of that part,
/// so the answer is never negative and no clamp ever runs.
///
/// **The six answers of one cell sum to six times the base, whatever the
/// wind.** The six directions are three opposite pairs, and the wind along a
/// direction is the negative of the wind along its opposite, so the six wind
/// terms cancel exactly. Six times the base is below the denominator, so the
/// sum a cell hands away is below what it holds, for every wind.[^1]
///
/// **This is public so that a test can prove the bound directly.**
///
/// # References
///
/// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
#[must_use]
pub fn transport_numerator(wind: Wind, direction: usize) -> i64 {
    TRANSPORT_BASE_NUMERATOR + i64::from(wind.along(direction))
}

/// Returns the denominator that every transport numerator stands over.
#[must_use]
pub const fn transport_denominator() -> i64 {
    TRANSPORT_DENOMINATOR
}

/// Returns the numerator of the share of the air that falls on one cell.
///
/// The share rises with the cooling that the air meets. The first argument
/// is the temperature of the cell the air stands over, and the second is the
/// temperature of the cell the air came from. Air that arrives colder than
/// it left drops more of what it carries. Air over cold ground drops more
/// than air over warm ground.[^1]
///
/// One rule gives two behaviours. Air that crosses from warm water onto a
/// cold ridge cools and rains, so the near side of high ground is wet. It
/// reaches the far side carrying less, so the far side is dry.
///
/// **This is public so that a test can read the rule apart from the field.**
///
/// # References
///
/// [^1]: ADR-0162, water falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[must_use]
pub fn fall_numerator(here: Degrees, upwind: Degrees) -> i64 {
    let cooling = i64::from(upwind.0).saturating_sub(i64::from(here.0));
    let by_cooling = clamp_i64(
        cooling / DEGREES_FOR_EACH_RAIN_STEP,
        0,
        FALL_NUMERATOR_COOLING_CEILING,
    );
    let below = i64::from(TEMPERATURE_AT_SEA_LEVEL).saturating_sub(i64::from(here.0));
    let by_cold = clamp_i64(
        below / DEGREES_FOR_EACH_COLD_STEP,
        0,
        FALL_NUMERATOR_COLD_CEILING,
    );
    FALL_NUMERATOR_FLOOR + by_cooling + by_cold
}

/// Returns the denominator that every fall numerator stands over.
#[must_use]
pub const fn fall_denominator() -> i64 {
    FALL_DENOMINATOR
}

/// Returns the drops that one sea lift raises at one temperature.
///
/// A hot cell lifts more than a cold one. A cell at or below the cold end of
/// the scale lifts nothing at all.[^1]
///
/// **This is public so that a test can read the rule apart from the field.**
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[must_use]
pub fn lift_drops(temperature: Degrees) -> i64 {
    let warmth = clamp_i64(i64::from(temperature.0), 0, LIFT_HEAT_DENOMINATOR);
    sim_math::share(
        Accum(LIFT_DROPS),
        Accum(warmth),
        Accum(LIFT_HEAT_DENOMINATOR),
    )
    .map_or(0, |value| value.0)
}

/// Returns the drops that the heat of a cell lifts off its own ground.
///
/// The water leaves the ground plane and arrives in the air plane of the
/// same cell. It stays inside the account, so it raises neither running
/// total.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[must_use]
pub fn ground_lift_drops(ground: Drops, temperature: Degrees) -> i64 {
    if ground.0 <= 0 {
        return 0;
    }
    let warmth = clamp_i64(i64::from(temperature.0), 0, GROUND_LIFT_DENOMINATOR);
    sim_math::share(
        ground.to_accum(),
        Accum(warmth),
        Accum(GROUND_LIFT_DENOMINATOR),
    )
    .map_or(0, |value| value.0)
}

/// What one call to the divine power did.
///
/// The report is returned rather than logged, because the call is a verb of
/// the control plane and the answer belongs to the caller that made it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Storm {
    /// The number of cells that took water. Two places in one cell count
    /// once.
    pub cells: u32,
    /// The drops that the call put into the air, over every cell.
    pub drops: i64,
    /// The first tick at which the faction may inflict weather again.
    pub ready_at: Tick,
}

/// The weather of the world, over the level 1 cell lattice.
///
/// The field holds one plane of air and one plane of ground. Both planes are
/// empty until something puts water into the world, so a world in which
/// nothing has happened stores nothing.[^1]
///
/// # References
///
/// [^1]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
#[derive(Clone, Debug)]
pub struct WeatherField {
    /// The cell lattice. It is a hex grid at the pitch of one level 1 cell.
    cells: Grid,
    faction_count: u16,
    /// The water in the air above each cell, in cell index order. It is empty
    /// until the first drop enters the world.
    air: Vec<Drops>,
    /// The water on the ground of each cell, in the same order.
    ground: Vec<Drops>,
    /// The write half of one spread pass.
    scratch: Vec<Drops>,
    /// The wind over each cell, in the same order.
    ///
    /// **The wind is carried state and it enters the state hash.** A wind
    /// computed from the pressure of the moment is the pressure of the
    /// moment, and no front would outlive its cause.[^1] [^2]
    ///
    /// The plane covers the whole lattice from the moment the field is
    /// built, because a wind exists over dry ground.
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    wind: Vec<Wind>,
    /// The write half of one wind pass.
    wind_scratch: Vec<Wind>,
    /// The temperature of each cell, in the same order.
    ///
    /// **The temperature is carried state and it enters the state hash.** It
    /// lags the temperature that the terrain, the season and the sky ask
    /// for, and the wind carries it between cells, so it cannot be rebuilt
    /// from the tick and the terrain.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    temperature: Vec<Degrees>,
    /// The write half of one temperature pass.
    temperature_scratch: Vec<Degrees>,
    /// Whether any solve has run over this field.
    ///
    /// The first solve sets the temperature of every cell to what the world
    /// asks for, rather than letting a field of zeros warm up over many
    /// ticks. The flag is one byte and never a `bool`, because it reaches
    /// the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    warmed: u8,
    /// Every drop that has ever entered the air, from the sea or from a god.
    raised: i64,
    /// The storms a god has raised over the life of the field.
    ///
    /// The count is a census reading. No later frame reads it, so it enters
    /// no hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    storms: i64,
    /// Every drop that has ever left the ground.
    evaporated: i64,
    /// The first tick at which each faction may inflict weather again.
    ready: Vec<Tick>,
    /// The spread passes that have run since the field was built.
    passes: u64,
}

impl WeatherField {
    /// Builds a field over a cell lattice, holding no water.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction count is above the ceiling the
    /// project supports.
    pub fn new(cells: Grid, faction_count: u16) -> Result<Self, WeatherError> {
        if faction_count > FACTION_CEILING {
            return Err(WeatherError::FactionCountAboveCeiling(faction_count));
        }
        // The wind and the temperature cover the whole lattice from the
        // start. A wind exists over dry ground, and the storage follows the
        // lattice whether the air holds water or not.[^1]
        //
        // [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, consequences. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
        let count = cells.tile_count() as usize;
        Ok(Self {
            cells,
            faction_count,
            air: Vec::new(),
            ground: Vec::new(),
            scratch: Vec::new(),
            wind: vec![Wind::STILL; count],
            wind_scratch: vec![Wind::STILL; count],
            temperature: vec![Degrees::ZERO; count],
            temperature_scratch: vec![Degrees::ZERO; count],
            warmed: 0,
            raised: 0,
            storms: 0,
            evaporated: 0,
            ready: vec![Tick(0); faction_count as usize],
            passes: 0,
        })
    }

    /// Returns the cell lattice the field covers.
    #[must_use]
    pub const fn cells(&self) -> Grid {
        self.cells
    }

    /// Returns the spread passes that have run since the field was built.
    #[must_use]
    pub const fn passes(&self) -> u64 {
        self.passes
    }

    /// Reports whether the field holds no water at all.
    ///
    /// A field that holds none has allocated no plane, and a solve over it
    /// does the lift draw and stops.
    #[must_use]
    pub fn is_dry(&self) -> bool {
        self.air.is_empty()
    }

    /// Returns the water in the air above one cell.
    ///
    /// Returns [`Drops::ZERO`] when the cell is outside the lattice, and when
    /// the field holds no water at all.
    #[must_use]
    pub fn air_at(&self, cell: u32) -> Drops {
        self.air.get(cell as usize).copied().unwrap_or(Drops::ZERO)
    }

    /// Returns the water on the ground of one cell.
    ///
    /// Returns [`Drops::ZERO`] when the cell is outside the lattice, and when
    /// the field holds no water at all.
    #[must_use]
    pub fn ground_at(&self, cell: u32) -> Drops {
        self.ground
            .get(cell as usize)
            .copied()
            .unwrap_or(Drops::ZERO)
    }

    /// Reports whether the ground of one cell is wet.
    ///
    /// This is the one reader that a simulation pass takes. The gather resolve
    /// asks it about the cell that covers the tile a unit stands on.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    #[must_use]
    pub fn cell_is_wet(&self, cell: u32) -> bool {
        self.ground_at(cell).0 >= WET_MARK.0
    }

    /// Returns the wind over one cell.
    ///
    /// The answer is a vector in the two-axis basis of the lattice. Returns
    /// [`Wind::STILL`] when the cell is outside the lattice.
    ///
    /// **This is one of the readers that another pass takes from the
    /// weather.** The field answers for the level 1 cell, so every tile of
    /// one cell answers the same.
    #[must_use]
    pub fn wind_at(&self, cell: u32) -> Wind {
        self.wind.get(cell as usize).copied().unwrap_or(Wind::STILL)
    }

    /// Returns the temperature of one cell.
    ///
    /// The unit is a whole degree on this module's own scale. Returns
    /// [`Degrees::ZERO`] when the cell is outside the lattice.
    ///
    /// **This is one of the readers that another pass takes from the
    /// weather.** The field answers for the level 1 cell, so every tile of
    /// one cell answers the same.
    #[must_use]
    pub fn temperature_at(&self, cell: u32) -> Degrees {
        self.temperature
            .get(cell as usize)
            .copied()
            .unwrap_or(Degrees::ZERO)
    }

    /// Returns the wind over every cell, in cell index order.
    #[must_use]
    pub fn wind_plane(&self) -> &[Wind] {
        &self.wind
    }

    /// Returns the temperature of every cell, in cell index order.
    #[must_use]
    pub fn temperature_plane(&self) -> &[Degrees] {
        &self.temperature
    }

    /// Returns the cell that the air over one cell came from.
    ///
    /// The air over a cell travels along the wind of that cell, so it came
    /// from the neighbour that lies against the wind. The pass walks the six
    /// directions in the fixed direction order and takes the first one that
    /// the wind points away from most. A cell with no wind, and a cell at
    /// the edge whose upwind neighbour is off the map, answers itself.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn upwind_of(&self, cell: u32) -> u32 {
        let wind = self.wind_at(cell);
        if wind == Wind::STILL {
            return cell;
        }
        let Some(address) = self.cells.address_of(TileIdx(cell)) else {
            return cell;
        };
        let mut best = cell;
        let mut least = 0i32;
        for direction in 0..NEIGHBOUR_COUNT {
            let along = wind.along(direction);
            if along >= least {
                continue;
            }
            let Some(neighbour) = self.cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = self.cells.index_of(neighbour) else {
                continue;
            };
            least = along;
            best = at.0;
        }
        best
    }

    /// Returns every entry of the air plane, in cell index order.
    ///
    /// The slice is empty when the field holds no water.
    #[must_use]
    pub fn air_plane(&self) -> &[Drops] {
        &self.air
    }

    /// Returns every entry of the ground plane, in cell index order.
    ///
    /// The slice is empty when the field holds no water.
    #[must_use]
    pub fn ground_plane(&self) -> &[Drops] {
        &self.ground
    }

    /// Returns every drop that has ever entered the air.
    #[must_use]
    pub const fn raised(&self) -> i64 {
        self.raised
    }

    /// Returns the storms a god has raised over the life of the field.
    ///
    /// A storm is one call of the divine power that put water into the air.
    /// The count never falls, so a zero says that no god ever acted. The
    /// water it raised dries, and this count does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-498. `docs/FINDINGS.md`
    #[must_use]
    pub const fn storms(&self) -> i64 {
        self.storms
    }

    /// Returns every drop that has ever left the ground.
    #[must_use]
    pub const fn evaporated(&self) -> i64 {
        self.evaporated
    }

    /// Returns the water in the air over the whole world.
    #[must_use]
    pub fn air_total(&self) -> Accum {
        total_of(&self.air)
    }

    /// Returns the water on the ground over the whole world.
    #[must_use]
    pub fn ground_total(&self) -> Accum {
        total_of(&self.ground)
    }

    /// Returns the number of cells whose ground is wet.
    #[must_use]
    pub fn wet_cells(&self) -> u32 {
        self.ground
            .iter()
            .filter(|drops| drops.0 >= WET_MARK.0)
            .count() as u32
    }

    /// Returns the first tick at which one faction may inflict weather.
    ///
    /// Returns `None` when the faction is outside the set the field holds.
    #[must_use]
    pub fn ready_at(&self, faction: FactionId) -> Option<Tick> {
        self.ready.get(faction.0 as usize).copied()
    }

    /// Reports whether the water account balances.
    ///
    /// Every drop that entered the air is in the air, on the ground, or
    /// counted as evaporated. Nothing else can have happened to it, so the
    /// three totals and the running total agree exactly.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    #[must_use]
    pub fn check_account(&self) -> bool {
        let held = sim_math::combine(self.air_total(), self.ground_total());
        let accounted = sim_math::combine(held, Accum(self.evaporated));
        accounted.0 == self.raised
            && self.air.iter().all(|drops| drops.0 >= 0)
            && self.ground.iter().all(|drops| drops.0 >= 0)
    }

    /// Puts weather over a set of places, at the command of a god.
    ///
    /// The faction is the congregation the god directs. Each place names a
    /// tile, and the water lands on the level 1 cell that covers it, so two
    /// places in one cell are one place.
    ///
    /// **The call is all or nothing.** Every place is resolved, every gate is
    /// checked, and the cooldown is checked, before anything is written. One
    /// refusal leaves the field exactly as it was.[^1]
    ///
    /// **A god acts only where its own people hold the ground.** The cell that
    /// covers a place must hold at least one tile of the faction. This is the
    /// gate that the project puts on speaking to another faction, and a divine
    /// power that ignored it would be the one action that escaped the
    /// rule.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the faction is outside the set this world holds,
    /// when the caller names more places than one call carries, when the
    /// strength is outside its range, when a place lies outside the world,
    /// when the faction holds no ground in the cell of a place, and when the
    /// faction inflicted weather too recently.
    ///
    /// # References
    ///
    /// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D3. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    /// [^2]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    pub fn inflict(
        &mut self,
        faction: FactionId,
        places: &[Axial],
        strength: u8,
        tick: Tick,
        ground: &Ground<'_>,
    ) -> Result<Storm, WeatherError> {
        if faction.0 >= self.faction_count {
            return Err(WeatherError::NoSuchFaction(faction.0));
        }
        if places.len() > PLACES_CEILING {
            return Err(WeatherError::TooManyPlaces(places.len()));
        }
        if strength == 0 || strength > STRENGTH_CEILING {
            return Err(WeatherError::StrengthOutOfRange(strength));
        }
        let waiting = self.ready[faction.0 as usize];
        if tick.0 < waiting.0 {
            return Err(WeatherError::StillCooling { ready_at: waiting });
        }

        // Every place is resolved and every gate is checked before the first
        // write. The cells are then sorted, so the write order does not
        // depend on the order the caller named them in, and a cell that two
        // places name is written once.[^1]
        //
        // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        let mut targets: Vec<u32> = Vec::with_capacity(places.len());
        for place in places {
            let cell = ground
                .cell_of(*place)
                .ok_or(WeatherError::PlaceOutsideWorld(*place))?;
            if !ground.holds(*place, faction) {
                return Err(WeatherError::GroundNotHeld(*place));
            }
            targets.push(cell);
        }
        targets.sort_unstable();
        targets.dedup();

        let drops = i64::from(strength) * DROPS_FOR_EACH_STRENGTH;
        if !targets.is_empty() {
            self.prepare();
        }
        let mut raised = 0i64;
        for cell in &targets {
            let Some(slot) = self.air.get_mut(*cell as usize) else {
                continue;
            };
            *slot = slot.combine(Drops(drops));
            raised = raised.saturating_add(drops);
        }
        self.raised = self.raised.saturating_add(raised);
        // **One call that put water into the air is one storm.** A call that
        // named no place raises nothing and is no storm, so the count and
        // the water agree.
        if raised > 0 {
            self.storms = self.storms.saturating_add(1);
        }
        let ready_at = Tick(tick.0.saturating_add(COOLDOWN_TICKS));
        self.ready[faction.0 as usize] = ready_at;
        Ok(Storm {
            cells: targets.len() as u32,
            drops: raised,
            ready_at,
        })
    }

    /// Runs one solve, which is one frame of weather.
    ///
    /// The solve lifts water from the sea, spreads the air a fixed number of
    /// passes, drops part of the air onto the ground, and dries part of the
    /// ground. Nothing calls it but the step, and it needs no caller beyond
    /// that: the world changes on its own.[^1]
    ///
    /// **A dry world does the lift and stops.** No plane is allocated and no
    /// pass runs until something puts water into the world.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for a solve at no threads, and
    /// when the summaries do not cover the cell lattice.
    ///
    /// # References
    ///
    /// [^1]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
    pub fn solve(
        &mut self,
        tick: Tick,
        seed: u64,
        summaries: &[CellSummary],
        threads: usize,
    ) -> Result<(), WeatherError> {
        if threads == 0 {
            return Err(WeatherError::ZeroThreads);
        }
        if summaries.len() != self.cells.tile_count() as usize {
            return Err(WeatherError::LatticeMismatch);
        }
        // The temperature runs first, because the pressure that accelerates
        // the wind follows it, and because the lift and the fall both read
        // it. It is one derived driver with three readers, and this is the
        // one place that composes them.[^1]
        //
        // [^1]: ADR-0162, water enters the air where it is hot, consequences. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
        self.warm(tick, summaries, threads);
        self.blow(threads);
        self.lift(tick, seed, summaries);
        if self.air.is_empty() {
            return Ok(());
        }
        for _ in 0..PASSES_FOR_EACH_SOLVE {
            self.spread(threads);
            self.passes = self.passes.saturating_add(1);
        }
        self.settle();
        Ok(())
    }

    /// Moves the temperature of every cell.
    ///
    /// The pass reads a settled plane and writes a separate one, so it never
    /// reads a half-written field and the answer does not depend on which
    /// cells a thread reached first.[^1] [^2]
    ///
    /// The first solve of a field sets the temperature outright, rather than
    /// letting a plane of zeros warm up over many ticks.
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn warm(&mut self, tick: Tick, summaries: &[CellSummary], threads: usize) {
        if self.warmed == 0 {
            for (cell, summary) in summaries.iter().enumerate() {
                let air = self.air.get(cell).copied().unwrap_or(Drops::ZERO);
                self.temperature[cell] = asked_temperature(*summary, air, tick);
            }
            self.warmed = 1;
            return;
        }
        let pass = WarmPass {
            cells: self.cells,
            tick,
            summaries,
            air: &self.air,
            wind: &self.wind,
            temperature: &self.temperature,
        };
        run_pass(pass, &mut self.temperature_scratch, threads);
        self.temperature.copy_from_slice(&self.temperature_scratch);
    }

    /// Runs the wind passes of one solve.
    ///
    /// Each pass takes the drag off the speed, adds the bounded step that
    /// the pressure difference asks for, and clamps both components at the
    /// ceiling. The count is fixed: the pass reads no clock, tests no
    /// residual, and takes no branch on what the field holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decisions D2, D3 and D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    fn blow(&mut self, threads: usize) {
        for _ in 0..WIND_PASSES_FOR_EACH_SOLVE {
            let pass = WindPass {
                cells: self.cells,
                wind: &self.wind,
                temperature: &self.temperature,
            };
            run_pass(pass, &mut self.wind_scratch, threads);
            self.wind.copy_from_slice(&self.wind_scratch);
        }
    }

    /// Lifts water from the sea into the air.
    ///
    /// The pass walks the cells in ascending index order and takes one keyed
    /// draw for each one. The draw is keyed on the cell, so the answer does
    /// not depend on which thread asked and the pass needs no thread at
    /// all.[^1]
    ///
    /// The pass allocates the planes on the first cell that lifts, and never
    /// before that.
    ///
    /// # References
    ///
    /// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    fn lift(&mut self, tick: Tick, seed: u64, summaries: &[CellSummary]) {
        let mut raised = 0i64;
        for (cell, summary) in summaries.iter().enumerate() {
            let heat = self.temperature[cell];
            // The sea lift raises water that was never in the account
            // before, so it raises the running total. It rises with the heat
            // of the cell, and a cold cell raises nothing.[^1]
            //
            // [^1]: ADR-0162, water enters the air where it is hot, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            if cell_lifts(
                seed,
                tick,
                cell as u32,
                summary.tiles(),
                summary.open_tiles(),
            ) {
                let drops = lift_drops(heat);
                if drops > 0 {
                    self.prepare();
                    self.air[cell] = self.air[cell].combine(Drops(drops));
                    raised = raised.saturating_add(drops);
                }
            }
            // The ground lift moves water that is already in the account, so
            // it raises no running total. The ground of a hot cell therefore
            // dries into its own air.[^1]
            let Some(wet) = self.ground.get(cell).copied() else {
                continue;
            };
            let taken = ground_lift_drops(wet, heat);
            if taken > 0 {
                self.ground[cell] = Drops(wet.0 - taken);
                self.air[cell] = self.air[cell].combine(Drops(taken));
            }
        }
        self.raised = self.raised.saturating_add(raised);
    }

    /// Allocates the planes, if they are not allocated already.
    fn prepare(&mut self) {
        if !self.air.is_empty() {
            return;
        }
        let count = self.cells.tile_count() as usize;
        self.air = vec![Drops::ZERO; count];
        self.ground = vec![Drops::ZERO; count];
        self.scratch = vec![Drops::ZERO; count];
    }

    /// Runs one spread pass over the air plane.
    ///
    /// The pass is a gather. A cell computes what it keeps and what each
    /// neighbour hands it, and it writes only itself, so no two threads write
    /// one cell and the pass needs no atomic operation.[^1]
    ///
    /// **The pass conserves water exactly.** A cell hands each neighbour the
    /// truncated eighth of what it holds, and the receiver adds the same
    /// integer, because both compute it from the same input plane. The
    /// remainder stays with the giver.[^2]
    ///
    /// The cells are visited in ascending index and the neighbours in
    /// direction order. Both orders are fixed.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    /// [^2]: ADR-0141, a weather pass moves water and never scales it, decision D1. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn spread(&mut self, threads: usize) {
        let pass = Pass {
            cells: self.cells,
            air: &self.air,
            wind: &self.wind,
        };
        run_pass(pass, &mut self.scratch, threads);
        self.air.copy_from_slice(&self.scratch);
    }

    /// Drops part of the air onto the ground, and dries part of the ground.
    ///
    /// The pass walks the cells in ascending index order. Every quantity it
    /// moves is a truncated integer share, so what leaves the air is exactly
    /// what arrives on the ground, and what leaves the ground is exactly what
    /// the evaporated total gains.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    fn settle(&mut self) {
        let mut dried = 0i64;
        for cell in 0..self.air.len() {
            let here = self.temperature[cell];
            let upwind = self.temperature[self.upwind_of(cell as u32) as usize];
            let numerator = fall_numerator(here, upwind);
            let air = self.air[cell];
            let fallen = share_of(air, numerator, FALL_DENOMINATOR);
            self.air[cell] = Drops(air.0 - fallen.0);
            self.ground[cell] = self.ground[cell].combine(fallen);

            let wet = self.ground[cell];
            let left = share_of(wet, 1, DRY_DIVISOR);
            self.ground[cell] = Drops(wet.0 - left.0);
            dried = dried.saturating_add(left.0);
        }
        self.evaporated = self.evaporated.saturating_add(dried);
    }

    /// Folds the field into a state hash.
    ///
    /// The order is the two running totals, then the readiness of each
    /// faction, then the air plane, the ground plane, the wind plane and the
    /// temperature plane, each in ascending slot order. **The wind and the
    /// temperature are carried state that a later pass reads, so both enter
    /// the hash.**[^2] The order is fixed and the hash is order-sensitive, so a
    /// reader does not have to prove that the order does not matter.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash
            .write_u64(u64::from(self.faction_count))
            .write_u64(self.passes)
            .write_u64(self.raised as u64)
            .write_u64(self.evaporated as u64);
        for ready in &self.ready {
            running = running.write_u64(ready.0);
        }
        running
            .write_u64(u64::from(self.warmed))
            .write(bytemuck::cast_slice(&self.air))
            .write(bytemuck::cast_slice(&self.ground))
            .write(bytemuck::cast_slice(&self.wind))
            .write(bytemuck::cast_slice(&self.temperature))
    }
}

/// Returns the truncated share of a quantity of water.
///
/// The arithmetic goes through the arithmetic module, whose intermediate
/// product is 128 bits wide and therefore exact for every input that a
/// 64-bit accumulator holds.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn share_of(water: Drops, part: i64, whole: i64) -> Drops {
    sim_math::share(water.to_accum(), Accum(part), Accum(whole))
        .map_or(Drops::ZERO, |value| Drops(value.0))
}

/// Returns the sum of a plane.
///
/// Integer addition is exactly associative and commutative, so the sum does
/// not depend on the order. The order is ascending anyway, because a reader
/// should not have to prove that again.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn total_of(plane: &[Drops]) -> Accum {
    let mut total = Accum(0);
    for drops in plane {
        total = sim_math::combine(total, drops.to_accum());
    }
    total
}

/// One pass that fills a run of an output plane.
///
/// A pass reads a settled plane and writes a separate one. It writes only
/// the cells of its own run, so two threads never write one cell and no
/// atomic operation appears.[^1]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
trait Fill: Copy + Send + Sync {
    /// What the pass writes.
    type Out: Send;

    /// Fills one run of the output plane, named by its position.
    fn fill(&self, start: usize, out: &mut [Self::Out]);
}

/// Runs one pass over a whole output plane.
///
/// The runs are contiguous and they cover the plane in ascending order.
/// Nothing in a pass reads which thread called it, and no pass reads the
/// order in which the threads finished.[^1]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
fn run_pass<F: Fill>(pass: F, out: &mut [F::Out], threads: usize) {
    let count = out.len();
    if count <= threads {
        pass.fill(0, out);
        return;
    }
    let chunk_len = count.div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut start = 0usize;
        for chunk in out.chunks_mut(chunk_len) {
            let low = start;
            start += chunk.len();
            scope.spawn(move || pass.fill(low, chunk));
        }
    });
}

/// What one transport pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct Pass<'a> {
    cells: Grid,
    air: &'a [Drops],
    wind: &'a [Wind],
}

impl Fill for Pass<'_> {
    type Out = Drops;

    /// Fills one run of the air plane.
    ///
    /// **The pass is a gather along the wind of the giver.** A cell computes,
    /// for each neighbour, the quantity that the neighbour sends it along
    /// that neighbour's wind, and it adds those quantities to what it
    /// kept.[^1]
    ///
    /// The two ends of an edge reach one integer, because both compute the
    /// giver's share from the same settled plane, with the same numerator
    /// and the same truncating division. The remainder stays with the
    /// giver.[^1]
    ///
    /// A cell whose wind is still sends the same quantity every way, so the
    /// isotropic behaviour is the zero-wind case of this rule.
    ///
    /// # References
    ///
    /// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
    fn fill(&self, start: usize, out: &mut [Drops]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let here = self.air[index];
            let mine = self.wind[index];

            let mut kept = here.0;
            let mut taken = Drops::ZERO;
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let slot = at.0 as usize;
                // What this cell sends that neighbour, along its own wind.
                kept -= share_of(
                    here,
                    transport_numerator(mine, direction),
                    TRANSPORT_DENOMINATOR,
                )
                .0;
                // What that neighbour sends this cell, along its wind. The
                // direction from the neighbour back to here is the opposite
                // of the direction from here to the neighbour.
                let back = (direction + NEIGHBOUR_COUNT / 2) % NEIGHBOUR_COUNT;
                let there = self.air[slot];
                taken = taken.combine(share_of(
                    there,
                    transport_numerator(self.wind[slot], back),
                    TRANSPORT_DENOMINATOR,
                ));
            }
            *cell = Drops(kept).combine(taken);
        }
    }
}

/// What one wind pass reads.
#[derive(Clone, Copy)]
struct WindPass<'a> {
    cells: Grid,
    wind: &'a [Wind],
    temperature: &'a [Degrees],
}

impl Fill for WindPass<'_> {
    type Out = Wind;

    /// Fills one run of the wind plane.
    ///
    /// The pass takes the drag share off the speed, adds the bounded step
    /// that the pressure difference asks for, and clamps both components at
    /// the ceiling. The pressure of a cell follows its heat, so the wind
    /// runs toward the warmer neighbour.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decisions D2 and D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    fn fill(&self, start: usize, out: &mut [Wind]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let here = self.temperature[index];
            let mut push_q = 0i32;
            let mut push_r = 0i32;
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let gap = sim_math::offset(self.temperature[at.0 as usize].0, -here.0);
                let steps = gap / DEGREES_FOR_EACH_PRESSURE_STEP;
                let offset_of = NEIGHBOURS[direction];
                push_q = sim_math::offset(push_q, sim_math::offset_by_count(steps, 1) * offset_of.q);
                push_r = sim_math::offset(push_r, sim_math::offset_by_count(steps, 1) * offset_of.r);
            }
            let step_q = clamp_i32(push_q, -WIND_ACCELERATION_STEP, WIND_ACCELERATION_STEP);
            let step_r = clamp_i32(push_r, -WIND_ACCELERATION_STEP, WIND_ACCELERATION_STEP);

            let was = self.wind[index];
            let drag_q = was.q * WIND_DRAG_NUMERATOR / WIND_DRAG_DENOMINATOR;
            let drag_r = was.r * WIND_DRAG_NUMERATOR / WIND_DRAG_DENOMINATOR;
            *cell = Wind {
                q: sim_math::offset(sim_math::offset(was.q, -drag_q), step_q),
                r: sim_math::offset(sim_math::offset(was.r, -drag_r), step_r),
            }
            .clamped();
        }
    }
}

/// What one temperature pass reads.
#[derive(Clone, Copy)]
struct WarmPass<'a> {
    cells: Grid,
    tick: Tick,
    summaries: &'a [CellSummary],
    air: &'a [Drops],
    wind: &'a [Wind],
    temperature: &'a [Degrees],
}

impl Fill for WarmPass<'_> {
    type Out = Degrees;

    /// Fills one run of the temperature plane.
    ///
    /// The pass does two things. It moves the stored temperature a share of
    /// the way toward what the terrain, the season and the sky ask for. It
    /// then takes, from each neighbour, a share of the difference between
    /// the two temperatures, and that share rises with the part of the
    /// neighbour's wind that points this way.[^1]
    ///
    /// Six neighbours at the wind ceiling hand less than the whole
    /// difference, so the answer stays inside the range of the temperatures
    /// it read and the field cannot run away.
    ///
    /// # References
    ///
    /// [^1]: ADR-0165, the temperature of a cell is carried state that a season and the sky drive, decisions D1, D2 and D3. `docs/adrs/draft/adr-0165-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    fn fill(&self, start: usize, out: &mut [Degrees]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let air = self.air.get(index).copied().unwrap_or(Drops::ZERO);
            let asked = asked_temperature(self.summaries[index], air, self.tick);
            let here = self.temperature[index];
            let gap = sim_math::offset(asked.0, -here.0);
            let mut degrees = sim_math::offset(here.0, gap / TEMPERATURE_RELAX_DIVISOR);

            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let slot = at.0 as usize;
                // The neighbour blows toward this cell when its wind points
                // along the direction from itself back to here.
                let back = (direction + NEIGHBOUR_COUNT / 2) % NEIGHBOUR_COUNT;
                let carried = self.wind[slot].along(back);
                if carried <= 0 {
                    continue;
                }
                let difference = sim_math::offset(self.temperature[slot].0, -here.0);
                let moved = sim_math::share(
                    Accum(i64::from(difference)),
                    Accum(i64::from(carried)),
                    Accum(i64::from(TEMPERATURE_ADVECT_DENOMINATOR)),
                )
                .map_or(0, |value| value.0) as i32;
                degrees = sim_math::offset(degrees, moved);
            }
            *cell = Degrees(clamp_i32(degrees, TEMPERATURE_FLOOR, TEMPERATURE_CEILING));
        }
    }
}

/// What the divine power reads about the ground.
///
/// The gate asks two questions of the world: which cell covers a place, and
/// whether the faction holds ground in that cell. The field asks them through
/// this view rather than holding a second copy of the holder column, because
/// one fact in two places is the defect shape this project meets most
/// often.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
pub struct Ground<'a> {
    /// The tile grid of the world.
    pub grid: Grid,
    /// Returns the level 1 cell that covers a tile.
    pub cell_of: &'a dyn Fn(TileIdx) -> Option<u32>,
    /// Returns the factions that hold ground in the cell covering a tile.
    pub holders_near: &'a dyn Fn(Axial) -> Option<FactionMask>,
}

impl Ground<'_> {
    /// Returns the level 1 cell that covers a place.
    fn cell_of(&self, place: Axial) -> Option<u32> {
        let tile = self.grid.index_of(place)?;
        (self.cell_of)(tile)
    }

    /// Reports whether a faction holds ground in the cell covering a place.
    fn holds(&self, place: Axial, faction: FactionId) -> bool {
        (self.holders_near)(place).is_some_and(|mask| mask.contains(faction))
    }
}
