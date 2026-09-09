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
//! # What carries the water
//!
//! **Each cell carries a wind, and the water rides it.** The wind is a
//! bounded integer vector over the two axes of the lattice, and it is
//! simulated state that enters the state hash.[^10] The pressure difference
//! across a cell accelerates the wind by a bounded step and never assigns it,
//! so a front keeps moving after the thing that raised it has eased. Drag
//! bleeds the momentum and a ceiling bounds the speed.[^10]
//!
//! The transport moves water between neighbouring cells. It is written as a
//! gather: a cell computes what it keeps and what each neighbour sends it
//! along that neighbour's wind, and no cell writes another cell. The quantity
//! a giver loses is the same integer that the receiver adds, because both
//! sides compute it from the same input planes with the same truncating
//! division.[^5] [^11] A cell whose wind is still sends the same quantity
//! every way, so a symmetric spread is the zero-wind case of one rule rather
//! than a second rule.
//!
//! **A symmetric kernel cannot carry a maximum.** It averages one away where
//! it stands. That is why the wind is a quantity the field holds rather than
//! a value on the old share.[^10]
//!
//! # What the ground does to the water
//!
//! **A low coast is hot and a high ridge is cold.** The heat of a cell rises
//! with the share of it that holds open water and falls with its mean height.
//! Nothing stores the heat: the solve derives it from the level 1 summary
//! that the step rebuilt, and three passes read it.[^12]
//!
//! # How much water the air of a cell holds
//!
//! **Warm air holds a lot of water and cold air holds very little.** The
//! capacity of a cell falls with its temperature, and three passes read it.
//! The lift raises water into the room below it. The fall rains out what
//! stands above it. The cloud term reads the air against it.
//!
//! That one rule is what puts weather at a high latitude. A cold region has a
//! small capacity, so a small quantity of water fills it and the sky goes
//! grey. A warm region has a large capacity, so the same water sits below the
//! mark and the sky stays clear. Air that travels toward a pole loses
//! capacity as it goes, so it turns into cloud on the way rather than being
//! destroyed at a boundary.
//!
//! **The capacity never creates or destroys water.** It is a bound that the
//! settle pass reads. The pass computes the water above the bound and moves
//! that quantity from the air plane to the ground plane of the same cell, as
//! an exact integer move. A cell that cools turns its vapour into cloud and
//! then into rain. Nothing scales the air toward the capacity and nothing
//! assigns the air to it, so the account holds through a temperature change
//! in either direction.
//!
//! **Water enters the air where the air has room.** A sea lifts into the room
//! below the capacity of its own cell, so a hot sea gives up more than a cold
//! one and the temperature reaches the lift through the capacity. The heat of
//! a cell also lifts water off its own ground, and what will not fit in the
//! room is what leaves the world.[^12]
//!
//! **Rain falls where the air is full.** The share that falls rises with the
//! air held against its own capacity. Cooling along the wind and a climb over
//! rising ground both shed capacity, so the near side of a ridge is wet and
//! the far side is dry.[^12]
//!
//! # Determinism
//!
//! **The solve runs a fixed number of wind passes and a fixed number of
//! transport passes.** It holds no convergence test and no time budget.[^6]
//! [^10]
//!
//! **Every draw is keyed on the tuple.** The weather system owns one system
//! identifier, and a lift draw keys on the frame and on the cell. One draw
//! serves one cell for one frame, in the way the contest draws once for each
//! tile rather than once for each unit.[^7]
//!
//! **Every pass writes disjoint output.** It reads the whole of one settled
//! plane and writes a contiguous run of another, so no two threads write one
//! cell and no atomic operation appears.[^8] [^9]
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
//! [^10]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
//! [^11]: ADR-0161, water rides the wind, and every transfer is an exact integer move. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
//! [^12]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::BlockLayout;
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOURS, NEIGHBOUR_COUNT};
use crate::holding::FactionMask;
use crate::padded::PaddedLattice;
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
    /// The caller asked for a cell side above the ceiling the scale carries.
    ScaleAboveCeiling(u32),
    /// The caller asked for a latitude span that does not fit on the globe.
    LatitudeSpanOutsideTheGlobe(i32),
    /// The caller asked for a relief that is not a positive height.
    ReliefNotPositive(i32),
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
    /// The caller named a cell that the lattice does not hold.
    NoSuchCell(u32),
    /// The caller asked for a storm outside the range the field carries.
    CycloneSettingOutOfRange,
    /// The field already carries as many storms as it holds.
    TooManyCyclones,
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
            Self::ScaleAboveCeiling(bits) => write!(
                formatter,
                "the weather scale {bits} is above the ceiling {SCALE_BITS_CEILING}"
            ),
            Self::LatitudeSpanOutsideTheGlobe(span) => write!(
                formatter,
                "the latitude span {span} does not fit between the two poles"
            ),
            Self::ReliefNotPositive(metres) => write!(
                formatter,
                "the relief {metres} is not a positive height in metres"
            ),
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
            Self::NoSuchCell(cell) => {
                write!(formatter, "the lattice holds no cell {cell}")
            }
            Self::CycloneSettingOutOfRange => write!(
                formatter,
                "a storm carries a depth of {CYCLONE_DEPTH_FLOOR} to {CYCLONE_DEPTH_CEILING}, a radius of 0 to {CYCLONE_RADIUS_CEILING}, and a life of 1 to {CYCLONE_LIFE_CEILING}"
            ),
            Self::TooManyCyclones => write!(
                formatter,
                "the field carries {CYCLONE_CEILING} storms at once"
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

/// The pitch of the weather lattice, as the tiles along one cell side.
///
/// **The resolution of the weather is a parameter of the world, and not a
/// constant welded to the level 1 geometry.** The scale holds the base-two
/// logarithm of the cell side in tiles, so a cell side is always a power of
/// two and the cell of a tile is a shift rather than a division.
///
/// [`Self::PER_TILE`] gives one weather cell to each tile. [`Self::LEVEL_1`]
/// gives one weather cell to each level 1 block, which is what the field
/// carried before the resolution became a parameter. The default is the level
/// 1 pitch, because the cost at the target scale is the reason the coarse
/// lattice was chosen and no measurement has displaced it.
///
/// **Three tuned quantities follow the scale**, because each of them means
/// something different at a different cell size: the tiles a cell side spans,
/// which the insolation reads to turn a cell row into a latitude, the divisor
/// that turns a temperature difference into a wind, and the transport pass
/// count. Each is derived from the scale by one formula, so no second
/// declaration of the tuning exists.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WeatherScale {
    bits: u32,
}

/// The largest cell side the scale carries, as a base-two logarithm.
///
/// A cell of this side spans 256 tiles, which is the whole of the
/// demonstration world along one axis. Nothing larger describes a lattice.
pub const SCALE_BITS_CEILING: u32 = 8;

/// The scale at which the tuning of the field was chosen.
///
/// The four derived quantities below return their stated values at this scale
/// exactly, so a world at the level 1 pitch behaves as it did before the
/// resolution became a parameter.
pub const REFERENCE_BITS: u32 = 5;

impl WeatherScale {
    /// One weather cell for each tile.
    pub const PER_TILE: Self = Self { bits: 0 };

    /// One weather cell for each level 1 block.
    pub const LEVEL_1: Self = Self {
        bits: REFERENCE_BITS,
    };

    /// The pitch a world takes when the caller states none.
    pub const DEFAULT: Self = Self::LEVEL_1;

    /// Builds a scale from the base-two logarithm of the cell side.
    ///
    /// # Errors
    ///
    /// Returns an error when the logarithm is above the ceiling.
    pub const fn from_bits(bits: u32) -> Result<Self, WeatherError> {
        if bits > SCALE_BITS_CEILING {
            return Err(WeatherError::ScaleAboveCeiling(bits));
        }
        Ok(Self { bits })
    }

    /// Returns the base-two logarithm of the cell side, in tiles.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.bits
    }

    /// Returns the tiles along one side of a cell.
    #[must_use]
    pub const fn side(self) -> u32 {
        1 << self.bits
    }

    /// Reports whether each tile carries its own weather cell.
    #[must_use]
    pub const fn is_per_tile(self) -> bool {
        self.bits == 0
    }

    /// Returns the transport passes that one solve runs at this scale.
    ///
    /// **A transport pass carries water at most one cell**, so the distance a
    /// solve moves water is the pass count, in cells. A finer lattice must
    /// run more passes to carry water the same distance in tiles, and the
    /// count therefore doubles for each halving of the cell side.
    ///
    /// **The ceiling is what stops that from being unaffordable.** The work
    /// of the stage is the pass count multiplied by the cell count, and both
    /// rise as the lattice gets finer, so an uncapped count would cost the
    /// square of the refinement. A capped count means that a fine lattice
    /// carries its water more slowly in tiles than a coarse one. The travel
    /// probe reports what that costs.[^1]
    ///
    /// # References
    ///
    /// [^1]: The weather travel probe. `crates/cachette-core/examples/weather_travel_probe.rs`
    #[must_use]
    pub const fn transport_passes(self) -> u32 {
        let asked = if self.bits >= REFERENCE_BITS {
            PASSES_AT_REFERENCE >> (self.bits - REFERENCE_BITS)
        } else {
            PASSES_AT_REFERENCE << (REFERENCE_BITS - self.bits)
        };
        if asked > PASS_CEILING {
            PASS_CEILING
        } else if asked == 0 {
            1
        } else {
            asked
        }
    }

    /// Returns the cells of margin the lattice carries on each of its four
    /// sides.
    ///
    /// **The lattice is larger than the world, and the margin is the extra.**
    /// Air leaves the world through one edge and, without a margin, nothing
    /// arrives through the other. The cells beside an edge are then starved
    /// of whatever the wind should carry in, and a mass can only be born
    /// inside the frame. The margin gives the border of the world real
    /// upwind. No reader sees a margin cell.[^1]
    ///
    /// # The derivation
    ///
    /// The margin must hold the distance a mass travels while it forms. A
    /// margin narrower than that delivers air that has not yet become
    /// anything, and the world border is starved again.
    ///
    /// The distance follows from two quantities the field already states.
    ///
    /// One transport pass sends a share of the air of a cell to each of the
    /// six neighbours, and the share rises with the part of the wind that
    /// points that way. The share is the same in every direction but for that
    /// wind term, so the net movement of one pass, in cells, is the wind term
    /// alone over the send denominator. At the speed ceiling that is
    /// `SPEED_CEILING / SEND_DENOMINATOR`. A solve runs `transport_passes` of
    /// them, so a solve carries air that many times as far.
    ///
    /// A sea cell lifts into the room below its own capacity, and it takes
    /// `LIFT_OF_ROOM_NUMERATOR` of `LIFT_OF_ROOM_DENOMINATOR` of that room on
    /// the frames it lifts, which is one frame in `LIFT_PERIOD`. So a mass
    /// forms in the frames one lift needs, times the lifts that fill the
    /// room. This is the time for a parcel that crosses a run of open sea. A
    /// single cell lifts less often than that, and the measurement in the
    /// commit body says what the margin then leaves undone.
    ///
    /// The margin is the product of the two, rounded up to whole cells.
    ///
    /// **The invariant is a distance in tiles, and the cell count follows.**
    /// The distance a solve carries air is not the same at every pitch,
    /// because the pass count runs into its own ceiling below the reference
    /// pitch. So the margin in tiles is larger at a coarse pitch than at a
    /// fine one, and that is the model reporting how far it actually carries
    /// air rather than an inconsistency.
    ///
    /// **The pass ceiling bounds this.** The pass count never rises above
    /// `PASS_CEILING`, so the margin never rises above the value that ceiling
    /// gives, whatever the pitch and whatever the extent of the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    pub const fn margin_cells(self) -> u32 {
        // The ticks a cell needs to fill its air from empty, rounded up. The
        // lift takes a share of the room each time, so the lifts that fill it
        // are the denominator over the numerator, and each lift waits a
        // period.
        let lifts =
            (LIFT_OF_ROOM_DENOMINATOR + LIFT_OF_ROOM_NUMERATOR - 1) / LIFT_OF_ROOM_NUMERATOR;
        let forming = lifts * LIFT_PERIOD as i64;
        let reach = (self.transport_passes() as i64) * (SPEED_CEILING as i64) * forming;
        ((reach + SEND_DENOMINATOR - 1) / SEND_DENOMINATOR) as u32
    }

    /// Returns the tiles of margin the lattice carries on each of its four
    /// sides.
    ///
    /// This is the margin in cells multiplied by the cell side. It is the
    /// distance the margin is worth, and it is what a reader should compare
    /// against a distance in the world.
    #[must_use]
    pub const fn margin_tiles(self) -> u64 {
        (self.margin_cells() as u64) * (self.side() as u64)
    }

    /// Returns what a heat difference across one cell is divided by.
    ///
    /// **The divisor does not follow the pitch, because the difference it
    /// reads does not.** It once carried a formula that shifted a reference
    /// value by the pitch, on the reasoning that a finer lattice spreads one
    /// map gradient over more cells and therefore holds a smaller difference
    /// between two neighbours. That reasoning holds for a field that is
    /// smooth at the pitch. The temperature is not one. The ground term reads
    /// the water and the height of the cell itself, so a coast holds a step
    /// of the whole water term between two neighbours at any pitch, and one
    /// cell for each tile reads the terrain of a single tile.
    ///
    /// **A divisor of one made the wind read that step, and the picture then
    /// held threads one cell wide.** The wind is the stiffest coupling in the
    /// field: it takes a bounded step toward the pressure on every pass, and
    /// the water rides it. A wind that answers a tile-scale step tears the
    /// water into threads that run along the flow, which is the shape that
    /// pure stretching with no mixing makes.
    ///
    /// The formula was in any case inert. It shifted the reference value
    /// right by the reference pitch before it shifted left by the asked
    /// pitch, so the first shift took it to zero and the clamp below returned
    /// one at every pitch. Nothing read the value the doc described.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn pressure_divisor(self) -> i64 {
        PRESSURE_DIVISOR
    }

    /// Returns the tiles along one side of a cell, as a wide number.
    ///
    /// The insolation works in tiles, because a latitude in cells means a
    /// different distance at every pitch.
    #[must_use]
    pub const fn side_tiles(self) -> i64 {
        self.side() as i64
    }
}

/// The hundredths of a degree in one degree of latitude.
///
/// **A latitude is a whole number of hundredths of a degree.** The unit is
/// small enough that one row of the finest lattice this engine supports gets
/// its own value, and large enough that the whole range from pole to pole
/// stays far inside a 32-bit number.
pub const LATITUDE_FINE: i32 = 100;

/// The latitude of a pole, in hundredths of a degree.
pub const LATITUDE_POLE: i32 = 90 * LATITUDE_FINE;

/// The hundredths of a degree in one whole turn.
const TURN_FINE: i64 = 4 * LATITUDE_POLE as i64;

/// The latitudes that the rows of a world stand at.
///
/// **The world states its own latitude span. The weather never reads the raw
/// row.** The row axis of the lattice carries a latitude, and the two ends of
/// that axis are the two ends of the span. A world that spans the whole globe
/// is a planet, with poles, a warm belt and a season that reverses across the
/// middle. A world that spans three degrees is one region of a planet, and
/// every latitude term in the field then goes flat and costs nothing.[^1]
///
/// **The two readings differ by one constant and not by a model.** That is
/// why the span is configuration. The engine holds one insolation geometry
/// and one banded circulation, and the span decides how much of them the
/// world sees.[^1]
///
/// The default is a whole planet: the centre stands at the equator and the
/// span runs from pole to pole. The project owner chose the planet reading
/// against the region reading, and a record holds the choice.[^2]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 9. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Latitudes {
    centre: i32,
    span: i32,
}

impl Latitudes {
    /// A whole planet. The centre stands at the equator and the span runs
    /// from pole to pole.
    pub const PLANET: Self = Self {
        centre: 0,
        span: 2 * LATITUDE_POLE,
    };

    /// The latitudes a world takes when the caller states none.
    ///
    /// **A world is a planet until somebody says otherwise.** The banded
    /// circulation, the subtropical deserts and the equatorial rain belt are
    /// what the project wants to see, and none of them exists inside a span
    /// of three degrees.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    pub const DEFAULT: Self = Self::PLANET;

    /// Builds a span from a centre latitude and a span, both in hundredths of
    /// a degree.
    ///
    /// # Errors
    ///
    /// Returns an error when the span is negative or wider than the globe,
    /// and when either end of the span passes a pole.
    pub const fn new(centre: i32, span: i32) -> Result<Self, WeatherError> {
        if span < 0 || span > 2 * LATITUDE_POLE {
            return Err(WeatherError::LatitudeSpanOutsideTheGlobe(span));
        }
        let low = centre - span / 2;
        let high = centre + span / 2;
        if low < -LATITUDE_POLE || high > LATITUDE_POLE {
            return Err(WeatherError::LatitudeSpanOutsideTheGlobe(span));
        }
        Ok(Self { centre, span })
    }

    /// Returns the centre latitude, in hundredths of a degree.
    #[must_use]
    pub const fn centre(self) -> i32 {
        self.centre
    }

    /// Returns the span from the first row to the last, in hundredths of a
    /// degree.
    #[must_use]
    pub const fn span(self) -> i32 {
        self.span
    }

    /// Returns the latitude of the middle of one row of a lattice.
    ///
    /// **The row is a row of the world and not of the whole lattice.** The
    /// lattice carries a margin, so a whole-lattice row and a world row are
    /// different things and a reader that confuses them samples the wrong
    /// cells.[^1]
    ///
    /// The first row stands at the low end of the span and the last row at
    /// the high end. A height of zero gives the centre.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    #[must_use]
    pub fn of_row(self, row: u32, height: u32) -> i32 {
        if height == 0 {
            return self.centre;
        }
        // The part of the span, measured from the middle, that the middle of
        // this row stands at. The numerator runs from one less than the
        // height to one less than the height the other way, so the two end
        // rows sit half a row inside the two ends of the span.
        let offset = sim_math::share(
            Accum(i64::from(self.span)),
            Accum(2 * i64::from(row) + 1 - i64::from(height)),
            Accum(2 * i64::from(height)),
        )
        .map_or(0, |value| value.0);
        (i64::from(self.centre) + offset).clamp(-i64::from(LATITUDE_POLE), i64::from(LATITUDE_POLE))
            as i32
    }
}

impl Default for Latitudes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The ground under one weather cell.
///
/// **The heat of a cell reads the ground, and the ground does not change.**
/// The three fields are the tiles the cell covers, the tiles of it that admit
/// a unit, and the sum of the heights of its tiles. Each of them is a pure
/// function of the world seed and the address, so the array is built once
/// when the world is built and never rebuilt.[^1]
///
/// **This replaces the level 1 summary as the source of the heat.** The level
/// 1 summary describes a block thirty-two tiles a side, and it is the wrong
/// source at any other weather pitch. It also carries four fields that no
/// weather pass reads, and the step rebuilds it on every tick for the readers
/// that do need them.[^2]
///
/// The type declares its layout. The wide field stands first, so the two
/// narrow ones fill the tail and the type needs no padding field.[^3]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^3]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct CellGround {
    /// The sum of the heights of the tiles the cell covers.
    ///
    /// The accumulator is 64 bits wide, because a tile field summed over a
    /// block of the target world overflows a 32-bit accumulator.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub height_total: i64,
    /// The sum of the heights of the tiles the cell covers that hold water.
    ///
    /// **The depth of the water is the one thing the other three fields
    /// cannot give.** The open tile count says how much of the cell is water
    /// and says nothing about how deep it is, so a shelf and an abyss read
    /// alike. The height total mixes the land and the water of one cell into
    /// one mean, so it cannot answer either question on its own. This field
    /// separates the two.
    ///
    /// The height of a water tile is the depth read the other way round. The
    /// water mark is the top of the range and zero is the deepest water.[^2]
    ///
    /// The accumulator is 64 bits wide, for the reason the height total is.
    ///
    /// # References
    ///
    /// [^2]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
    pub water_height_total: i64,
    /// The tiles the cell covers.
    pub tiles: i32,
    /// The tiles of the cell whose ground admits a unit.
    pub open_tiles: i32,
}

impl CellGround {
    /// A cell that covers no tile.
    pub const EMPTY: Self = Self {
        height_total: 0,
        water_height_total: 0,
        tiles: 0,
        open_tiles: 0,
    };

    /// Combines the ground of two cells.
    ///
    /// The operation is field-wise integer addition. It is exactly
    /// associative and commutative and its identity is [`Self::EMPTY`], so a
    /// fold over a set of tiles gives one answer whatever the order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            height_total: self.height_total.saturating_add(other.height_total),
            water_height_total: self
                .water_height_total
                .saturating_add(other.water_height_total),
            tiles: self.tiles.saturating_add(other.tiles),
            open_tiles: self.open_tiles.saturating_add(other.open_tiles),
        }
    }

    /// Builds the ground of one tile.
    ///
    /// **Every caller that folds a lattice reads this.** Two callers built
    /// the record from a tile, and a third field would have had to be added
    /// to both with nothing to fail when only one of them got it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn of_tile(tile: TerrainTile) -> Self {
        let height = sim_math::accumulate(Accum(0), tile.height).0;
        let passable = tile.kind.is_passable();
        Self {
            height_total: height,
            // The water height is the height of the tiles that hold water,
            // and nothing else. The heat reads it to tell a shallow shelf
            // from a deep ocean, which the tile count cannot do.
            water_height_total: if passable { 0 } else { height },
            tiles: 1,
            open_tiles: i32::from(passable),
        }
    }

    /// Returns the tiles the cell covers.
    #[must_use]
    pub const fn tiles(self) -> i64 {
        self.tiles as i64
    }

    /// Returns the tiles of the cell whose ground admits a unit.
    #[must_use]
    pub const fn open_tiles(self) -> i64 {
        self.open_tiles as i64
    }

    /// Returns the share of the cell that admits a unit.
    ///
    /// Returns `None` when the cell covers no tile.
    #[must_use]
    pub fn open_share(self) -> Option<Fix32> {
        if self.tiles <= 0 {
            return None;
        }
        Some(Fix32(clamp_to_fix(
            (self.open_tiles() << crate::types::FIX_FRACTIONAL_BITS) / self.tiles(),
        )))
    }

    /// Returns the tiles of the cell that hold water.
    ///
    /// Water is the only ground that admits no unit, so the tiles that admit
    /// none are the water tiles exactly.
    #[must_use]
    pub const fn water_tiles(self) -> i64 {
        (self.tiles - self.open_tiles) as i64
    }

    /// Returns the mean height of the cell.
    ///
    /// Returns `None` when the cell covers no tile.
    #[must_use]
    pub fn mean_height(self) -> Option<Fix32> {
        if self.tiles <= 0 {
            return None;
        }
        Some(Fix32(clamp_to_fix(self.height_total / self.tiles())))
    }

    /// Returns the mean height of the land of the cell.
    ///
    /// **The land of a cell is not the cell.** A coastal cell that reads its
    /// mean height over water and land together reports a height that no tile
    /// of it holds, and the heat then treats a beach beside an abyss as if it
    /// were a valley floor.
    ///
    /// Returns `None` when the cell holds no land.
    #[must_use]
    pub fn mean_land_height(self) -> Option<Fix32> {
        if self.open_tiles <= 0 {
            return None;
        }
        let land = self.height_total - self.water_height_total;
        Some(Fix32(clamp_to_fix(land / self.open_tiles())))
    }

    /// Returns how shallow the water of the cell is, from none to whole.
    ///
    /// The answer is whole at the water mark and none at the deepest water.
    /// **Nothing else in the field can tell a shelf from an abyss**, because
    /// a tile counts as water when its height falls below one mark and the
    /// count past that mark carries no depth at all.
    ///
    /// Returns `None` when the cell holds no water.
    #[must_use]
    pub fn shallowness(self) -> Option<Fix32> {
        let water = self.water_tiles();
        if water <= 0 {
            return None;
        }
        let mean = self.water_height_total / water;
        Some(Fix32(clamp_to_fix(
            (mean << crate::types::FIX_FRACTIONAL_BITS) / i64::from(HEIGHT_WATER.0),
        )))
    }
}

/// Clamps a wide value into the fixed-point range.
const fn clamp_to_fix(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < i32::MIN as i64 {
        i32::MIN
    } else {
        value as i32
    }
}

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

/// The wind over one cell of the lattice.
///
/// **The wind is simulated state, and it enters the state hash.** It is a
/// bounded integer vector in the fixed two-axis basis of the lattice. It is
/// not a floating point number, it is not an angle, and no part of it is a
/// fraction of anything.[^1] [^2]
///
/// A wind computed from the pressure of the moment is the pressure of the
/// moment: it stops when the pressure stops, and no front outlives its cause.
/// The field carries the wind so that a front keeps moving after the thing
/// that raised it has eased.[^1]
///
/// The type declares its layout, because the wind reaches the state hash. Two
/// `i32` fields fill eight bytes exactly, so the type needs no padding
/// field.[^3]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^3]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Wind {
    /// The part of the wind along the first lattice axis.
    pub q: i32,
    /// The part of the wind along the second lattice axis.
    pub r: i32,
}

impl Wind {
    /// No wind at all. A still cell sends the same water every way.
    pub const STILL: Self = Self { q: 0, r: 0 };

    /// Returns the third cube part, which the other two imply.
    ///
    /// The three parts sum to zero. The lattice has three axes and two of
    /// them are free, so this is derived and never stored.
    #[must_use]
    pub const fn s(self) -> i32 {
        -self.q - self.r
    }

    /// Reports whether the cell has no wind.
    #[must_use]
    pub const fn is_still(self) -> bool {
        self.q == 0 && self.r == 0
    }

    /// Returns the speed of the wind, in lattice steps.
    ///
    /// The speed is the lattice distance the vector spans. It is the largest
    /// of the three cube parts in magnitude, and the half sum below gives the
    /// same answer.
    #[must_use]
    pub const fn speed(self) -> i32 {
        (self.q.abs() + self.r.abs() + self.s().abs()) / 2
    }

    /// Returns twice the part of this wind that points along one direction.
    ///
    /// The direction is one of the six unit steps of the lattice. The answer
    /// is twice the speed when the wind points exactly that way, twice the
    /// speed the other way round, and it runs through the four values between
    /// in whole steps. Nothing here is a fraction, and no angle appears.
    #[must_use]
    pub const fn along(self, direction: Axial) -> i32 {
        self.q * direction.q + self.r * direction.r + self.s() * direction.s()
    }

    /// Returns the direction the wind points at most strongly.
    ///
    /// Returns `None` when the cell is still. A tie takes the lowest
    /// direction index, which is a fixed order and not a thread order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn heading(self) -> Option<usize> {
        if self.is_still() {
            return None;
        }
        let mut best = 0usize;
        let mut most = i32::MIN;
        for (direction, neighbour) in NEIGHBOURS.iter().enumerate() {
            let along = self.along(*neighbour);
            if along > most {
                most = along;
                best = direction;
            }
        }
        Some(best)
    }
}

/// Returns the direction that faces the other way.
const fn opposite(direction: usize) -> usize {
    (direction + NEIGHBOUR_COUNT / 2) % NEIGHBOUR_COUNT
}

/// Returns a wind whose speed is one step lower.
///
/// The largest cube part gives one step to the smallest. The three parts sum
/// to zero before and after, and the speed is the largest of them, so the
/// step lowers the speed by exactly one. A step that moved one axial part
/// alone would leave some winds at the same speed for ever.
fn shrink_one(wind: Wind) -> Wind {
    let parts = [wind.q, wind.r, wind.s()];
    let mut largest = 0usize;
    let mut smallest = 0usize;
    for axis in 1..parts.len() {
        if parts[axis] > parts[largest] {
            largest = axis;
        }
        if parts[axis] < parts[smallest] {
            smallest = axis;
        }
    }
    if largest == smallest {
        return Wind::STILL;
    }
    let mut next = parts;
    next[largest] -= 1;
    next[smallest] += 1;
    Wind {
        q: next[0],
        r: next[1],
    }
}

/// Returns a wind whose speed is at most a ceiling.
///
/// The scale truncates both parts, which can leave the speed one step above
/// the ceiling, so a fixed number of exact shrink steps follows it. The count
/// is fixed and it is not a convergence test.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
fn cap(wind: Wind, ceiling: i32) -> Wind {
    let speed = wind.speed();
    if speed <= ceiling {
        return wind;
    }
    let mut capped = Wind {
        q: narrow(sim_math::share(
            Accum(i64::from(wind.q)),
            Accum(i64::from(ceiling)),
            Accum(i64::from(speed)),
        )),
        r: narrow(sim_math::share(
            Accum(i64::from(wind.r)),
            Accum(i64::from(ceiling)),
            Accum(i64::from(speed)),
        )),
    };
    for _ in 0..CAP_SHRINK_STEPS {
        if capped.speed() <= ceiling {
            break;
        }
        capped = shrink_one(capped);
    }
    capped
}

/// Returns an accumulator as a lattice part.
///
/// The value is a wind part or a wind step, and both are far inside the
/// 32-bit range. The clamp states that rather than leaving a cast to say it.
fn narrow(value: Option<Accum>) -> i32 {
    let held = value.map_or(0, |accum| accum.0);
    held.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// Returns what drag leaves of one part of a wind.
///
/// Drag takes a share of the part and then one whole step more. **The share
/// alone never reaches zero**, because the division truncates, so a wind
/// under no pressure would coast for ever. The whole step is what brings a
/// cell back to rest. It is the same defect that stopped the ground drying,
/// in a second place.
fn bleed(part: i32) -> i32 {
    let kept = i64::from(part)
        - sim_math::share(
            Accum(i64::from(part)),
            Accum(DRAG_NUMERATOR),
            Accum(DRAG_DENOMINATOR),
        )
        .map_or(0, |taken| taken.0);
    (kept - kept.signum()) as i32
}

/// Returns what drag leaves of one wind.
///
/// Each part moves toward zero, and moving a part toward zero never raises
/// the speed. So drag can never carry a wind through the speed ceiling.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
fn drag(wind: Wind) -> Wind {
    Wind {
        q: bleed(wind.q),
        r: bleed(wind.r),
    }
}

/// Returns what a deflection adds to one wind.
///
/// **The world turns, so a wind that blows is pushed to one side.** The push
/// is at a right angle to the wind and it rises with the speed, which is what
/// a rotating planet does to anything that moves over it.
///
/// The answer is a share of the wind turned by one sixth of a turn. A sixth
/// is the only turn the lattice holds exactly: the step `(q, r)` turns into
/// `(-r, q + r)`, which is integer arithmetic and not an angle. A share of a
/// sixth is a smaller turn, because the sum of a vector and a small part of
/// its own sixth-turn points a little to the side of the original.
///
/// **The wind is counted in fine steps so that this share is not zero.** A
/// wind of one whole lattice step is eight fine steps, and a share of a
/// quarter of that is two, so even a slow wind turns.
///
/// **This is what makes a vortex, and nothing here names one.** Air runs
/// toward a warm cell, the deflection turns it aside, and it arrives running
/// round the cell rather than into it. A closed circulation is what a
/// deflected inflow is.
fn deflect(wind: Wind) -> Wind {
    // One sixth of a turn on the axial lattice.
    let turned = Wind {
        q: -wind.r,
        r: wind.q + wind.r,
    };
    Wind {
        q: narrow(sim_math::share(
            Accum(i64::from(turned.q)),
            Accum(DEFLECT_NUMERATOR),
            Accum(DEFLECT_DENOMINATOR),
        )),
        r: narrow(sim_math::share(
            Accum(i64::from(turned.r)),
            Accum(DEFLECT_NUMERATOR),
            Accum(DEFLECT_DENOMINATOR),
        )),
    }
}

/// The number of spread passes that one solve runs at the reference scale.
///
/// The count is fixed for a given scale. A solve runs it whatever the field
/// holds and whatever the thread count. It is not a budget and no measurement
/// chose it: it is the reach that one frame of weather adds, in cells.[^1]
///
/// **The count a solve runs is derived from the scale of the world**, and a
/// caller reads it there rather than here.[^2]
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
/// [^2]: The transport pass count of a scale. [`WeatherScale::transport_passes`]
pub const PASSES_AT_REFERENCE: u32 = 4;

/// The most transport passes that one solve runs, at any scale.
///
/// **The work of the stage is the pass count multiplied by the cell count.**
/// Both rise as the lattice gets finer, so a pass count that rose without a
/// bound would cost the square of the refinement. This is what turns that
/// square into a line, and the cost is that a fine lattice carries its water
/// more slowly across the map than a coarse one.
pub const PASS_CEILING: u32 = 32;

/// The number of wind passes that one solve runs.
///
/// The count is fixed. A solve runs it whatever the field holds and whatever
/// the thread count. It reads no clock, it tests no residual, and it takes no
/// branch on what the field holds.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
pub const WIND_PASSES_FOR_EACH_SOLVE: u32 = 2;

/// The steps of wind that make one whole lattice step of speed.
///
/// **The wind is counted finely, and the fine count is what lets it turn.**
/// A wind whose parts are whole lattice steps holds six directions and a
/// handful of speeds, so a term that turned it by a part of a step would
/// truncate to nothing at every speed under the divisor and the field could
/// hold no rotation at all. Counting the same wind in eighths gives the
/// deflection room to act without changing what any wind does to the water.
///
/// Every denominator that reads a wind carries this factor, so the share of
/// the air that one wind sends is what it was before the count was made
/// finer.
pub const WIND_FINE: i32 = 8;

/// The denominator of every share of the air that a transport pass sends.
const SEND_DENOMINATOR: i64 = 64 * WIND_FINE as i64;

/// The numerator a cell sends to each neighbour whatever its wind.
///
/// A cell whose wind is still sends this to every neighbour, so the old
/// symmetric behaviour is the zero-wind case of the directional rule rather
/// than a second rule.[^1]
///
/// # References
///
/// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
const SEND_BASE_NUMERATOR: i64 = 4 * WIND_FINE as i64;

/// The numerator that one step of wind along a direction adds.
const SEND_FOR_EACH_WIND_STEP: i64 = 1;

/// The largest speed the wind of a cell ever reaches, in lattice steps.
///
/// **The ceiling is load-bearing and it is not tidiness.** The share a cell
/// sends in one direction rises with the part of its wind that points that
/// way, so an unbounded wind would let a cell promise more water than it
/// holds. The ceiling is what bounds that share, and the assertion below is
/// what makes a reviewer able to find a violation.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^2]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
pub const SPEED_CEILING: i32 = 6 * WIND_FINE;

/// The most the projections of one wind onto the six directions add to, for
/// the directions the projection is positive on.
///
/// The projection of a wind onto a direction is twice the part of the wind
/// that points that way, so a projection reaches twice the speed. Only three
/// of the six can be positive at once, and their sum reaches four times the
/// speed and never passes it. A proof stands in the module test.
const POSITIVE_PROJECTION_CEILING: i64 = 4 * SPEED_CEILING as i64;

// **A cell never sends more water than it holds, at any wind.**
//
// This is decision D2 of the transport record, checked at build time rather
// than left to a reader. A change to the speed ceiling, to the base share or
// to the wind term that broke the property stops the build. The constant is
// anonymous, because a named one is dead code and a dead constant is not
// guaranteed to be checked.[^1]
//
// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
const _: () = assert!(
    NEIGHBOUR_COUNT as i64 * SEND_BASE_NUMERATOR
        + SEND_FOR_EACH_WIND_STEP * POSITIVE_PROJECTION_CEILING
        < SEND_DENOMINATOR
);

/// The top of the heat scale of a cell.
///
/// Heat is a whole number from zero to this. It is derived from the level 1
/// summary on every solve and it is never stored, so it cannot drift from the
/// world it came from.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
pub const HEAT_CEILING: i32 = 256;

/// The height below which a tile holds water, read and not restated.
///
/// The terrain declares the mark, and the shallowness of a cell is its mean
/// water height as a share of it. A second copy here would be a second
/// declaration of one value, and nothing would fail when the two
/// disagreed.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
use crate::terrain::{Terrain, TerrainTile, HEIGHT_WATER};

/// The degrees that the sun adds at the warmest place and the warmest moment.
///
/// **Two parts make it up, and the sum is what the heat scale reserves.** One
/// part is the belt of the latitude, which the annual mean of the insolation
/// gives. The other is the season, which the daily value against that annual
/// mean gives. The sun takes the same count away at the coldest place and the
/// coldest moment, so the term runs from this above zero to this below it.[^1]
/// [^2]
///
/// # References
///
/// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D3. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
pub const SEASON_SWING: i32 = LATITUDE_SWING + SEASON_ANOMALY_SWING;

/// The ticks in which the sun completes one whole swing.
///
/// **The season is one oscillation of the sun and not a band that marches.**
/// A band that stepped along an axis and wrapped would jump the whole width
/// of the map every lap, and the temperature field would hold a seam at that
/// wrap however smooth the profile across the band. A sun that swings between
/// two limits never wraps, so the field holds no seam at all. It also slows
/// and reverses at each limit, which leaves a quiet gradient that then
/// rebuilds the other way round.
///
/// **The period is a time and not a distance**, so it follows neither the
/// extent of the world nor the pitch of the lattice. A run of twenty thousand
/// ticks holds nine and a half of these, so a watcher sees the swing repeat.
/// One half of it lasts a thousand ticks, which is long enough for a place to
/// hold a wet part of the year and a dry part.
pub const SEASON_PERIOD_TICKS: i64 = 2048;

/// The published net effect of cloud on the energy the Earth keeps, in whole
/// watts for each square metre.
///
/// **The published effect is negative, and this is its size.** Cloud reflects
/// sunlight away and it holds heat in, and the reflection wins. The figure is
/// the whole-Earth net of the two, over the cover the Earth carries.[^1]
///
/// **The author of this constant did not read a primary source for it.** It
/// is the figure a reader meets in the standard account of the energy budget
/// of the Earth, and the research report that this module follows states no
/// figure of its own for cloud.[^2]
///
/// # References
///
/// [^1]: The net effect of cloud on the energy budget of the Earth, from the standard account of that budget. It is reported near twenty watts for each square metre.
/// [^2]: Research report 30, the published atmospheric math. `docs/research/reports/30-the-published-atmospheric-math.md`
const CLOUD_EFFECT_WATTS: i64 = 20;

/// The published share of the Earth that cloud covers, in hundredths.
///
/// **The effect above is the effect of this cover, and not of a whole sky.**
/// The term in this module answers a whole sky, so the build divides the one
/// by the other.[^1]
///
/// # References
///
/// [^1]: The mean cloud cover of the Earth, from the standard account of the energy budget of the Earth. It is reported near two thirds.
const CLOUD_COVER_FINE: i64 = 68;

/// The part of the way to the asked temperature that one pass moves.
const WARMTH_NUMERATOR: i64 = 1;

/// The whole of the part that one pass moves.
///
/// **The share is what makes the temperature lag its driver**, and the lag is
/// the whole reason the field carries the temperature rather than deriving
/// it. A pass that assigned the asked temperature would hold the driver of
/// the moment, and no warm parcel would outlive the cell it left.[^1]
///
/// # References
///
/// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
const WARMTH_DENOMINATOR: i64 = 8;

/// What the temperature divisor of the deepest water is multiplied by.
///
/// **This is the thermal inertia of the sea, and it is derived from a
/// published seasonal lag.** The warmest month over the ocean falls about two
/// months after the solstice, where over land it falls about one. A first
/// order lag driven by a yearly cycle lags its driver by the arc tangent of
/// two pi times the time constant over the period, so a two month lag on a
/// twelve month year gives a time constant near a quarter of the year. On the
/// season period this module carries, that is this multiple of the land
/// figure.
///
/// **The old value was eight and it did almost nothing.** At eight the sea
/// damps its own seasonal swing by about two percent, so a sea tracked the
/// season as closely as the land beside it and the two never parted. That was
/// harmless while a second term in the driver gave the sea a mean of its own,
/// and it stopped being harmless when a record removed that term and named
/// this lag as the thing that carries the sea.[^2] **A mechanism nominated to
/// carry an effect must be measured against the effect**, and this one was
/// not.
///
/// # References
///
/// [^2]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
const DEEP_WATER_LAG: i64 = 64;

/// The share of a temperature difference that one step of wind carries.
const CARRY_FOR_EACH_WIND_STEP: i64 = 1;

/// The whole of the share that one neighbour carries.
///
/// **The six shares sum to less than one whole.** The greatest total positive
/// projection of one wind over the six directions is the projection ceiling
/// below, and that ceiling divided by this is under one. So the new
/// temperature lies inside the range of the temperatures the pass read, and
/// the field cannot run away at any wind and over any number of passes.[^1]
///
/// # References
///
/// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D3. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
const CARRY_DENOMINATOR: i64 = 32 * WIND_FINE as i64;

// The carry cannot run away. The check is the same shape as the one the
// transport carries, and it fails the build rather than a test.
const _: () = assert!(POSITIVE_PROJECTION_CEILING * CARRY_FOR_EACH_WIND_STEP < CARRY_DENOMINATOR);

/// The temperature passes that one solve runs.
///
/// The count is fixed. No pass tests whether the field settled.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
pub const WARMTH_PASSES_FOR_EACH_SOLVE: u32 = 1;

/// The heat difference that one step of wind acceleration answers to.
///
/// The acceleration of a cell is the sum, over the six directions, of the
/// direction times the heat the neighbour holds above this cell. That sum is
/// divided by this before the step ceiling bounds it.
///
/// **A larger divisor makes a softer wind, and the wind is what tears the
/// water into threads.** The value is a balance between two things a watcher
/// wants. A small divisor gives a wind that answers the terrain of one cell,
/// and the water then holds threads one cell wide. A large divisor gives a
/// smooth wind and a smooth sky, and it also takes the rotation out of the
/// field: a probe reports no turning cell at all at twice this.
///
/// The value is the same at every pitch, and a caller reads it from the scale
/// rather than here, so that one site answers the question.[^1] A blocker
/// holds the question of what the wind should be worth, and no measurement
/// has chosen this.[^2]
///
/// # References
///
/// [^1]: The pressure divisor of a scale. [`WeatherScale::pressure_divisor`]
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
const PRESSURE_DIVISOR: i64 = 16 / WIND_FINE as i64;

/// The most the wind of a cell changes in one pass, in lattice steps.
///
/// **The step is what makes the wind lag the pressure**, and the lag is the
/// whole reason the field carries the wind at all. A pass that assigned the
/// asked-for wind would hold the pressure of the moment and no front would
/// outlive its cause.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
const WIND_STEP: i32 = 2 * WIND_FINE;

/// The share of the wind of a cell that drag takes in one pass.
const DRAG_NUMERATOR: i64 = 1;

/// The whole of the share that drag takes in one pass.
///
/// The drag settles the wind where the step and the share balance. Under the
/// step above the field settles below the speed ceiling, so a settled field
/// never reaches the ceiling and the ceiling is a guarantee rather than a
/// working part.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
const DRAG_DENOMINATOR: i64 = 4;

/// The shrink steps that bound one wind after a truncating scale.
///
/// A scale that truncates both components leaves the speed at most one step
/// above what it asked for. Two steps are therefore more than enough, and the
/// count is fixed rather than a loop that runs until a test passes.[^1]
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
const CAP_SHRINK_STEPS: u32 = 2;

/// The part of a sixth of a turn that the deflection adds to a wind each
/// pass.
///
/// **This is the one term that turns the flow, and nothing else in the field
/// can.** The pressure gradient points a wind at a warm cell and drag slows
/// it; neither can make it go round anything. A rotating world pushes a
/// moving parcel to one side, and that push is what turns an inflow into a
/// circulation.
///
/// The share is small on purpose. A large share would spin every cell
/// regardless of the flow, which is a stirred field and not a weather field.
/// A small one bends a wind that already blows and leaves a still cell still.
const DEFLECT_NUMERATOR: i64 = 1;

/// The whole of the deflection share above.
const DEFLECT_DENOMINATOR: i64 = 3;

/// The denominator of the share of the air that falls in one solve.
const FALL_DENOMINATOR: i64 = 64;

/// The numerator of the share that falls whatever the air holds.
const FALL_NUMERATOR_FLOOR: i64 = 1;

/// What air standing at its own capacity adds to the fall numerator.
///
/// **The fall reads the fullness of the air, not the temperature.** Cold air
/// well below its own small capacity is a clear winter sky, and it must not
/// rain. Warm air at its own large capacity is a tropical afternoon, and it
/// must. The old rule read the coldness of the cell directly, so any air that
/// reached a high latitude rained out at once and the poles held no water at
/// all.
///
/// **This term is what grades the sky.** A full sky drains fast and a clear
/// one drains slowly, so a cell whose supply is small against its own
/// capacity settles below the mark and paints as thin cloud. With a small
/// term every cell that gets any supply at all sits at its capacity, and the
/// cloud field is then two values and not a field.
const FALL_FOR_FULL_AIR: i64 = 12;

/// The whole of the capacity share that the cooling and the climb move.
const CAPACITY_SHED_WHOLE: i64 = 64;

/// The capacity that a cell keeps whatever the cooling and the climb ask.
///
/// A capacity of nothing would put the whole of the air of a cell onto its
/// ground in one solve, which is the instant destruction the temperature rule
/// used to do. The floor keeps the shed bounded.
const CAPACITY_SHED_KEPT_FLOOR: i64 = 8;

/// The capacity that a whole range of cooling along the wind takes away.
///
/// **Air that cools on its way here holds less than the cell it left.** So
/// the cooling makes cloud out of vapour that was invisible one cell back,
/// and the excess falls. This is the term that tells the near side of a ridge
/// from the far side.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const CAPACITY_SHED_FOR_COOLING: i64 = 24;

/// The capacity that a whole climb of the height range takes away.
///
/// **Air that meets rising ground goes up, it cools, and it then holds
/// less.** The flow is horizontal over a flat lattice, so nothing else in the
/// field lifts a parcel. This is the one term that couples the height map to
/// the flow.
///
/// The term is signed. Air that descends a slope warms, so its capacity rises
/// and it holds the water it carries. The lee of a range is dry for that
/// reason and not for a separate one.
///
/// **This is not a third dimension, and it does not need one.** A layered
/// field would hold a wind and a water plane for each layer, which multiplies
/// the whole stage by the layer count. One term against the gradient of the
/// cell the air enters gives orographic rain on the windward slope and a rain
/// shadow behind it, which is the behaviour a watcher recognises.
const CAPACITY_SHED_FOR_CLIMB: i64 = 32;

// A whole cooling and a whole climb together still leave the cell a capacity.
// The check fails the build rather than a test.
const _: () = assert!(
    CAPACITY_SHED_FOR_COOLING + CAPACITY_SHED_FOR_CLIMB
        <= CAPACITY_SHED_WHOLE - CAPACITY_SHED_KEPT_FLOOR
);
const _: () = assert!(FALL_NUMERATOR_FLOOR + FALL_FOR_FULL_AIR < FALL_DENOMINATOR);

/// The share of the water on the ground that leaves the world in one solve.
const DRY_DIVISOR: i64 = 32;

/// The drops that leave the ground of a cell in one solve beyond the share.
///
/// **The share alone never empties a cell.** The division truncates, so
/// ground holding fewer drops than the divisor loses nothing at all, and
/// water then accumulates everywhere and stays. This whole drop is what takes
/// the last of it. A cell holding less than this loses what it holds.
const DRY_FLOOR: i64 = 1;

/// The whole of the share of its ground water that a cell of full heat lifts.
///
/// The lift is the ground water times the heat of the cell over this. Water
/// lifted this way leaves the ground plane and arrives in the air plane of
/// the same cell, so it moves inside the account rather than entering it.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const GROUND_LIFT_DENOMINATOR: i64 = 8 * HEAT_CEILING as i64;

/// The part of the room below the capacity that the sea lifts into, when it
/// lifts.
///
/// **A sea evaporates into the room its own sky has, not into a fixed number
/// of drops.** The room is the capacity of the cell less the water already
/// standing over it, so the temperature reaches the lift through the
/// capacity and never on its own. A cold sea has a small sky, gives up a
/// small quantity, and fills that sky. A hot sea has a large sky, gives up a
/// large quantity, and fills that one.
///
/// **The old rule made the lift proportional to the temperature alone.** A
/// polar sea then raised almost nothing into a sky sized for the tropics, so
/// water could only be created near the sun and the high latitudes were
/// cloudless whatever else the field did.[^1]
///
/// The share is not the whole room. A sea that filled its sky on every lift
/// would leave every water cell painted as a whole sky, and the cloud field
/// would carry no shape over the ocean at all.
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const LIFT_OF_ROOM_NUMERATOR: i64 = 1;

/// The whole of the share of the room that the sea lifts into.
const LIFT_OF_ROOM_DENOMINATOR: i64 = 1;

const _: () = assert!(LIFT_OF_ROOM_NUMERATOR > 0);
const _: () = assert!(LIFT_OF_ROOM_NUMERATOR <= LIFT_OF_ROOM_DENOMINATOR);

/// The water that the air above a cell of full heat holds.
///
/// **This is the ceiling of the air plane, and it is the capacity of the
/// hottest cell the scale allows.** No cell holds more, whatever its
/// temperature and whatever the wind brings it, so a viewer that paints the
/// air reads its ramp from here.
///
/// Without a mark nothing bounded what the transport delivers into a cell
/// that several winds converge on. The lift alone was not enough, because it
/// bounds the source and not the sum of what arrives. A convergence cell then
/// climbed without a bound, and the plane held a maximum three orders of
/// magnitude above its own median.[^1]
///
/// A god may still put more than this into a cell in one call, because a
/// storm is an injection and the model carries that water away rather than
/// refusing it. The next solve rains out what the cell could not hold.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D3. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
pub const AIR_SATURATION: Drops = capacity_at(HEAT_CEILING);

/// The temperature scale of the field, as the hundredths of a degree Celsius
/// that one degree of warmth is worth.
///
/// **Every published curve needs a temperature, and the warmth of a cell is
/// an abstract count.** So the field declares one linear map from the count
/// to a temperature, in one place, and every published form reads it.[^1]
///
/// Half a kelvin for each count, with the bottom of the count at seventy
/// degrees below zero, covers every surface temperature a planet holds.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 1.5. `docs/research/reports/30-the-published-atmospheric-math.md`
pub const WARMTH_FINE: i32 = 50;

/// The temperature of a cell of no warmth, in hundredths of a degree Celsius.
pub const WARMTH_FLOOR: i32 = -70 * 100;

/// The water that the air holds at the freezing point of water.
///
/// **This is the anchor of the saturation curve and nothing else.** The
/// published curve gives a ratio between two temperatures, so one temperature
/// must carry a stated count of drops for the rest to follow. The freezing
/// point is the anchor, because it is the one temperature a reader recognises
/// without a table.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 1.5. `docs/research/reports/30-the-published-atmospheric-math.md`
const CAPACITY_AT_FREEZING: i64 = 2048;

/// The Magnus numerator of the published saturation curve, in ten-thousandths.
///
/// The published curve is `6.1094 * exp(17.625 * T / (243.04 + T))` in
/// hectopascals, with the temperature in degrees Celsius. The base-two
/// logarithm of it is `17.625 / ln 2` times the same rational function, and
/// this constant is that quotient in ten-thousandths.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 1.1 and 1.4. `docs/research/reports/30-the-published-atmospheric-math.md`
const MAGNUS_NUMERATOR: i64 = 254_275;

/// The whole of the Magnus numerator above.
const MAGNUS_FINE: i64 = 10_000;

/// The Magnus offset of the published curve, in hundredths of a degree.
const MAGNUS_OFFSET: i64 = 24_304;

/// The entries in the table of the fractional part of the exponent.
const POWER_ENTRIES: usize = 8;

/// Two raised to each eighth of one, in the fixed-point form of the project.
///
/// **The whole part of the exponent is a shift and the fraction is this
/// table.** The whole part is therefore exact at any temperature, and the
/// error of the curve does not grow with the range. Eight entries with a
/// linear interpolation hold the published curve to under a tenth of one part
/// in a thousand.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 1.4. `docs/research/reports/30-the-published-atmospheric-math.md`
const POWER_TABLE: [i64; POWER_ENTRIES + 1] = [
    65536, 71468, 77936, 84990, 92682, 101_070, 110_218, 120_194, 131_072,
];

/// The water that the air above a cell of no heat holds.
///
/// **Cold air holds very little, and that is why a cold sky is grey.** A
/// small quantity of water fills a small capacity, so the sky over a polar
/// plain stands at its own mark while it holds a twentieth of what a tropical
/// sky carries. A capacity of nothing would rain the whole of the air out in
/// one solve, so the floor is not zero.
///
/// **The floor is what the transport needs.** A cell sends each neighbour a
/// truncated share of its air, so a cell holding a few drops sends nothing
/// and the water never leaves the coast. The floor keeps the coldest sky
/// large enough to carry.
const CAPACITY_FLOOR: i64 = 2;

// The cold capacity sits under the hot one, so the capacity rises with the
// temperature over the whole scale. The check fails the build.
const _: () = assert!(CAPACITY_FLOOR > 0 && CAPACITY_FLOOR < AIR_SATURATION.0);

/// A whole sky, in the unit that the cloud share counts in.
///
/// The unit is 255ths, because that is what an overlay paints into one byte
/// of a channel. A reader that wants a percentage divides.
pub const CLOUD_SHARE_WHOLE: i64 = 255;

/// Returns the water that the air above one cell holds at a temperature.
///
/// **Warm air holds a lot of water and cold air holds very little.** That one
/// fact is what puts cloud at a high latitude. A cold region has a small
/// capacity, so a small quantity of water fills it and the sky goes grey. A
/// warm region has a large capacity, so the same water sits below the mark
/// and the sky stays clear.
///
/// **The curve is the published one.** It is the Magnus form of Alduchov and
/// Eskridge, which is the recommended set for meteorology and holds to about
/// four parts in a thousand from forty degrees below zero to fifty above
/// it.[^2]
///
/// The earlier curve doubled the capacity at a fixed temperature step. The
/// published curve does not: its doubling width runs from about seven kelvin
/// at the cold end to about fourteen at the warm end, and the best fixed
/// doubling is wrong by a third at the cold end. That is the end where a
/// player expects tundra and ice.[^2]
///
/// **The integer form is a rational exponent and a small table.** The base-two
/// logarithm of the published curve is one multiply and one divide. The whole
/// part of that logarithm is a shift and is exact, so the error does not grow
/// with the temperature range. Only the fraction reads a table, and the table
/// has eight entries. The worst error of the form against the published curve
/// is under a tenth of one part in a thousand, and the commit body holds the
/// measurement.[^2]
///
/// **This never creates or destroys water.** It is a bound that the settle
/// pass reads, and every quantity that pass moves is an exact integer move
/// between the air plane and the ground plane. A cell that cools turns its
/// vapour into cloud and then into rain, and the account holds through
/// both.[^1]
///
/// **This is public so that a test can move the temperature and watch the
/// answer move.**
///
/// # References
///
/// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
/// [^2]: Research report 30, the published atmospheric math, sections 1.1 to 1.5. `docs/research/reports/30-the-published-atmospheric-math.md`
#[must_use]
pub const fn capacity_at(warmth: i32) -> Drops {
    let heat = if warmth < 0 {
        0
    } else if warmth > HEAT_CEILING {
        HEAT_CEILING
    } else {
        warmth
    };
    // The temperature of the cell, in hundredths of a degree Celsius.
    let degrees = (heat as i64) * (WARMTH_FINE as i64) + (WARMTH_FLOOR as i64);
    // The base-two logarithm of the capacity against the capacity at the
    // freezing point, in the fixed-point form of the project. One multiply
    // and one divide, and no table.
    let one = Fix32::ONE.0 as i64;
    let exponent = match sim_math::share(
        Accum(one * MAGNUS_NUMERATOR),
        Accum(degrees),
        Accum(MAGNUS_FINE * (MAGNUS_OFFSET + degrees)),
    ) {
        Some(value) => value.0,
        None => 0,
    };
    // The whole part is a shift and the fraction reads the table. The shift
    // is arithmetic, so the whole part floors and the fraction stays
    // positive.
    let whole = exponent >> 16;
    let fraction = exponent - (whole << 16);
    let entry = (fraction * (POWER_ENTRIES as i64)) >> 16;
    let within = fraction * (POWER_ENTRIES as i64) - (entry << 16);
    let low = POWER_TABLE[entry as usize];
    let high = POWER_TABLE[entry as usize + 1];
    let power = low
        + match sim_math::share(Accum(high - low), Accum(within), Accum(one)) {
            Some(value) => value.0,
            None => 0,
        };
    // The multiply runs before the shift, so the shift keeps the accuracy
    // that the table gave.
    let shift = 16 - whole;
    let held = if shift >= 63 {
        0
    } else if shift >= 0 {
        (CAPACITY_AT_FREEZING * power) >> shift
    } else {
        (CAPACITY_AT_FREEZING * power) << (-shift)
    };
    if held < CAPACITY_FLOOR {
        Drops(CAPACITY_FLOOR)
    } else {
        Drops(held)
    }
}

/// Returns the capacity of air that cooled and climbed on its way to a cell.
///
/// **Air carries its own history, and the capacity must carry it too.** The
/// temperature of the cell alone describes air that has stood there. Air that
/// arrived from a warmer cell, or that rose over a slope, is colder than the
/// cell it came from and holds less than the cell it stands over. Both terms
/// shed a share of the capacity the temperature gives.
///
/// The climb is signed. Air that descends a slope warms, so its capacity
/// rises above the base and the lee of a range stays dry. The result never
/// passes the mark of the hottest cell, and a floor keeps it above nothing.
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
#[must_use]
pub fn travelling_capacity(base: Drops, cooling: i32, climb: i32) -> Drops {
    let cooled = i64::from(cooling.clamp(0, HEAT_CEILING));
    let by_cooling = sim_math::share(
        Accum(CAPACITY_SHED_FOR_COOLING),
        Accum(cooled),
        Accum(i64::from(HEAT_CEILING)),
    )
    .map_or(0, |value| value.0);
    let by_climb = sim_math::share(
        Accum(CAPACITY_SHED_FOR_CLIMB),
        Accum(i64::from(climb)),
        Accum(i64::from(Fix32::ONE.0)),
    )
    .map_or(0, |value| value.0);
    let shed = (by_cooling + by_climb).clamp(
        -CAPACITY_SHED_WHOLE,
        CAPACITY_SHED_WHOLE - CAPACITY_SHED_KEPT_FLOOR,
    );
    let kept = CAPACITY_SHED_WHOLE - shed;
    let held = sim_math::share(Accum(base.0), Accum(kept), Accum(CAPACITY_SHED_WHOLE))
        .map_or(0, |value| value.0);
    Drops(held.clamp(0, AIR_SATURATION.0))
}

/// How rarely a cell of open water lifts.
///
/// A cell draws once each frame. It lifts when the draw, taken below the tile
/// count of the cell multiplied by this, falls below the number of tiles of
/// that cell that admit no unit. A cell that is all water therefore lifts on
/// one frame in this many, and a cell with no water never lifts.
///
/// **The period must stay above one.** A sea lifts into the room below its
/// own capacity, and the transport empties much of that room in each solve,
/// so a long period leaves every sky far below its own mark and the map goes
/// clear. A period of one lifts on every frame, which takes the frame out of
/// the answer, and a draw that no longer depends on the frame is the defect
/// the keying rule exists to stop.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const LIFT_PERIOD: u64 = 2;

/// The draw index of the lift draw within a frame.
///
/// The weather system takes one draw for each cell in each frame, so this is
/// the only index it uses.
const LIFT_DRAW: u32 = 0;

/// The water on the ground at which a cell counts as wet.
///
/// A wet cell yields more to a unit that gathers from it. The value is a
/// content constant that no measurement chose, and a blocker holds the
/// question of what weather should be worth.[^1]
///
/// **The mark is a share of the ceiling of the plane, and the ceiling moved.**
/// The ceiling is the capacity of the hottest cell the scale allows, and it
/// rose by more than an order of magnitude when the capacity became the
/// published saturation curve.[^2] The mark rose with it, and it has to: a
/// mark left where it stood called every cell of every world wet, which is the
/// reading that once made wetness separate nothing.[^1]
///
/// **The share fell from a thirty-second to a sixty-fourth at the same time.**
/// The ceiling is the capacity of a cell at the top of the heat scale, which
/// is far above the temperature that any land of any world holds. So the
/// wettest cell of a world stands well under the ceiling, and a thirty-second
/// of the ceiling stood above the wettest cell of a coarse world. **Nobody has
/// measured the share again since the heat terms were derived**, and the
/// blocker that holds what weather should be worth governs it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D4. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
pub const WET_MARK: Drops = Drops(AIR_SATURATION.0 / 64);

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
const DROPS_FOR_EACH_STRENGTH: i64 = 2 * AIR_SATURATION.0;

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

/// Folds the terrain of the world into one entry for each cell of the world.
///
/// The result is in the cell index order of the world, and it holds no
/// margin. Every field it carries is a pure function of the world seed and
/// the address, so a caller folds it once when the world is built and never
/// again.[^1]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
#[must_use]
pub fn cell_ground_of(layout: BlockLayout, terrain: Terrain) -> Vec<CellGround> {
    let grid = layout.grid();
    let count = (layout.blocks_wide() as usize).saturating_mul(layout.blocks_high() as usize);
    let mut ground = vec![CellGround::EMPTY; count];
    for row in 0..grid.height() {
        for column in 0..grid.width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(tile) = terrain.tile(address) else {
                continue;
            };
            let Some(key) = grid.index_of(address).and_then(|at| layout.key_of(at)) else {
                continue;
            };
            let Some(slot) = ground.get_mut(layout.block_of_key(key) as usize) else {
                continue;
            };
            *slot = slot.combine(CellGround::of_tile(tile));
        }
    }
    ground
}

/// Folds the terrain of the world into one entry for each cell of the whole
/// lattice, margin included.
///
/// # What ground a margin cell carries
///
/// **A margin cell mirrors the cell of the world nearest to it.** It stands
/// outside the world, so no tile backs it and it has no ground of its own.
/// The mirror continues the character of the edge it stands beside: a coast
/// stays a coast and a ridge stays a ridge just outside the frame, so air
/// that enters the world has already been shaped by ground of the right kind.
///
/// **The alternative is open water, and it is wrong here.** Water is the only
/// ground that lifts, and the heat of a cell rises with the water share it
/// carries. A margin of open water is therefore an unbounded source of both
/// moisture and heat on all four sides, pressed against every edge of the
/// world at once. It would drown the border rather than feed it, and it would
/// drive a wind inward on every side, which no world has.
///
/// **A margin of nothing is wrong for the opposite reason.** An empty cell
/// carries no tile, so it is cold and it lifts nothing, and a ring of cold
/// dead cells is the starved border the margin exists to remove.
///
/// The mirror copies the whole entry, including the tile count. A cell of the
/// world at an edge that the cell side does not divide covers fewer tiles
/// than a full cell, and its mirror then covers as few. That is the intent:
/// the mirror is a copy of a neighbour, not an invention.
///
/// A margin of zero gives the fold of the world alone.
#[must_use]
pub fn ground_over_lattice(
    lattice: PaddedLattice,
    layout: BlockLayout,
    terrain: Terrain,
) -> Vec<CellGround> {
    let inner = cell_ground_of(layout, terrain);
    if lattice.is_bare() {
        return inner;
    }
    (0..lattice.whole().tile_count())
        .map(|cell| {
            lattice
                .nearest_inner(cell)
                .and_then(|at| inner.get(at as usize).copied())
                .unwrap_or(CellGround::EMPTY)
        })
        .collect()
}

/// Returns a fixed-point fraction held inside zero and one.
fn to_unit(fraction: Fix32) -> Fix32 {
    if fraction.0 < 0 {
        Fix32::ZERO
    } else if fraction.0 > Fix32::ONE.0 {
        Fix32::ONE
    } else {
        fraction
    }
}

/// Returns the part of a whole that a fixed-point fraction of one earns.
///
/// The arithmetic is a multiply and a truncating divide through the
/// arithmetic module.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn part_of(whole: i32, fraction: Fix32) -> i32 {
    narrow(sim_math::share(
        Accum(i64::from(whole)),
        Accum(i64::from(to_unit(fraction).0)),
        Accum(i64::from(Fix32::ONE.0)),
    ))
}

/// Returns the mean height of the land of a world, weighted by how much land
/// each cell holds.
///
/// **This is the zero of the relief term.** The balance temperature is an
/// observed global mean, and the land of a planet already stands at its mean
/// elevation inside that observation, so a term that cooled every cell from
/// sea level would count that elevation twice. A cell at this height therefore
/// receives the balance unchanged, one above it is colder, and one below it is
/// warmer.[^1]
///
/// **This reads the world and not the Earth, and that is a modelling choice.**
/// The strict reading of the rule would take the mean elevation of the Earth,
/// because that is what the published constants average over, and a world whose
/// land really does stand higher would then be genuinely colder. That reading is
/// correct physics and it leaves the height ceiling controlling the temperature
/// of the whole world, which is what cost this project two decisions. Reading
/// the world's own mean makes the ceiling decide how dramatic the ground looks
/// and nothing else. **The two agree at a ceiling where the world's land
/// happens to average what the Earth's does, and they part as the ceiling
/// rises.**
///
/// The answer is a reduction over the ground of the world in ascending cell
/// order, and the ground does not change, so it is the same integer on every
/// tick and at every thread count.[^2]
///
/// # References
///
/// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
/// [^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub fn mean_land_height_over(ground: &[CellGround]) -> Fix32 {
    let mut total = 0i64;
    let mut counted = 0i64;
    for cell in ground {
        let Some(height) = cell.mean_land_height() else {
            continue;
        };
        let land = i64::from(to_unit(cell.open_share().unwrap_or(Fix32::ZERO)).0);
        total += i64::from(to_unit(height).0) * land;
        counted += land;
    }
    if counted <= 0 {
        return Fix32::ZERO;
    }
    Fix32(clamp_to_fix(total / counted))
}

/// Returns what the relief of a cell takes off the balance temperature, in
/// hundredths of a degree.
///
/// **The answer is signed, and its zero is the mean land height of the
/// world.** Air cools as it rises, so ground above that mean stands below the
/// temperature the energy balance settles at, and ground below it stands
/// above. Open water carries no height, so it takes the whole of the
/// reference as a warming.[^4]
///
/// **The water of a cell does not appear here, and that is deliberate.** The
/// sea moderates a coast by holding its heat, not by sitting at a different
/// mean, and the field already carries that as a lag on how fast a cell
/// follows its driver. A second term for it would be one fact in two
/// places.[^5]
///
/// The ground comes from the ground array over the weather lattice, which the
/// world folds from the terrain once. **It does not come from the level 1
/// summary**, because that summary describes a block thirty-two tiles a side
/// and is the wrong source at any other weather pitch.[^1]
///
/// The heat has three readers in one solve: the pressure that drives the
/// wind, the evaporation, and the fall. It is one derived value, computed
/// once for each solve into a buffer that never crosses a tick boundary, and
/// storing it in one of the three would put a second declaration of it in the
/// tree.[^2] [^3]
///
/// A cell that covers no tile is cold.
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
/// [^4]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
/// [^5]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
/// [^2]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
/// [^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub fn relief_cooling_of(ground: CellGround, relief: HeightRange, reference: Fix32) -> i32 {
    if ground.tiles() <= 0 {
        return 0;
    }
    // **The share of the cell that holds land.** A tile is open when a unit
    // may stand on it, so the open share is the land share and not the water
    // share. Open water stands at the sea mark, which is the bottom of the
    // range, so it cools nothing.
    let land = ground.open_share().map_or(Fix32::ZERO, to_unit);

    // **The land term reads the height of the land, and not of the cell.** A
    // mean over both reported a height that no tile of it held, and a deep
    // sea then read as the lowest ground in the world.
    let land_height = ground.mean_land_height().map_or(Fix32::ZERO, to_unit);

    // **Air cools as it rises, at the published rate.** The height is a unit
    // fraction and the relief of the world is what the whole fraction is
    // worth, so the metres are the one multiplied by the other. The answer is
    // negative, because high ground is colder than the balance and never
    // warmer than it.[^3]
    //
    // [^3]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    // **The height stands against the mean land of the world and not against
    // sea level.** Ground at that mean receives the balance unchanged, ground
    // above it is colder, and ground below it is warmer.
    let metres = i64::from(part_of(relief.metres(), land_height))
        - i64::from(part_of(relief.metres(), reference));
    let cooling = narrow(sim_math::share(
        Accum(LAPSE_RATE_FINE * metres),
        Accum(1),
        Accum(METRES_IN_KILOMETRE),
    ));
    // A cell that is half sea and half mountain cools by half of what the
    // mountain alone would.
    -part_of(cooling, land)
}

/// Returns what the temperature divisor of a cell is multiplied by.
///
/// The answer is one over land and over water at the shoreline mark, and it
/// rises to the deep-water lag over the deepest water. It is graded by the
/// mean depth of the water and by the share of the cell that holds any, so a
/// coastal cell lags less than the open sea beyond it.
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
#[must_use]
pub fn lag_of(ground: CellGround) -> i64 {
    let water = ground
        .open_share()
        .map_or(Fix32::ZERO, |open| to_unit(Fix32(Fix32::ONE.0 - open.0)));
    let shallow = ground.shallowness().map_or(Fix32::ONE, to_unit);
    let deep = Fix32(Fix32::ONE.0 - shallow.0);
    // The extra lag is the whole extra multiplied by how deep the water is
    // and then by how much of the cell is water. A cell with no water gets
    // none of it, and neither does a cell whose water lies at the mark.
    let extra = i64::from(part_of(part_of((DEEP_WATER_LAG - 1) as i32, deep), water));
    1 + extra.max(0)
}

/// Returns where the sun stands at one tick, as a declination.
///
/// The declination is the latitude at which the sun stands overhead. It runs
/// from the obliquity of the world at one solstice to the same figure the
/// other way at the other, and it crosses the equator at each equinox. The
/// answer is in hundredths of a degree.
///
/// **The swing never wraps.** The declination follows a sine of the tick over
/// the season period, so the sun crosses the equator twice in a period, rests
/// at each limit and turns back. That is the simple published form, and the
/// report says it is enough here.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 4.1 and 11. `docs/research/reports/30-the-published-atmospheric-math.md`
#[must_use]
pub fn declination_at(tick: Tick) -> i32 {
    let wave = sim_math::sine(tick.0 as i64, SEASON_PERIOD_TICKS).unwrap_or(Fix32::ZERO);
    narrow(sim_math::share(
        Accum(i64::from(OBLIQUITY)),
        Accum(i64::from(wave.0)),
        Accum(i64::from(Fix32::ONE.0)),
    ))
}

/// Returns the degrees the sun adds to one cell at one tick.
///
/// **The term is the published insolation geometry and not a fall away from
/// where the sun stands.** The daily mean energy that the top of the
/// atmosphere receives has a closed form in the latitude and the declination,
/// and that form holds the polar day. At the solstice the pole receives about
/// a third more daily energy than the equator, and no function of the
/// distance from the sun can produce that shape. The earlier term was such a
/// function, and it gave a pole that was cold all year.[^1]
///
/// **Two parts make the term, because the two answer different questions.**
/// The annual mean of the geometry falls from the equator to the pole, and
/// that part is the climate belt of a latitude. The daily value against that
/// annual mean is the season, and that part reverses across the equator. They
/// carry separate amplitudes, because the mean stands in equilibrium and the
/// season is damped by the heat that the ground and the sea hold.[^2]
///
/// **The two normalisers are properties of the globe and not of the world.**
/// They are read at the equator, at a pole, and at the middle latitude of the
/// same geometry, whatever span the world states. So a world that spans three
/// degrees reads a nearly flat slice of the same table and the term goes
/// flat, rather than stretching a four percent change across the whole
/// scale.[^1]
///
/// **The sum of the two parts is normalised against what that sum can
/// actually reach, and not against the sum of the two amplitudes.** The belt
/// peaks at the equator, where the season is near nothing, and the season
/// peaks at the middle latitudes, where the belt is near nothing. So the two
/// amplitudes added together name a swing that no place and no moment holds.
/// The table therefore walks its own geometry once, records the highest and
/// the lowest sum it holds, and maps that range onto the swing the heat scale
/// reserves. The two amplitudes above are then the ratio between the belt and
/// the season, and the reserved swing is the whole of the sun term.[^3]
///
/// The term reads the tick and the latitude. It reads no clock and takes no
/// draw.
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 4.3, 4.4 and 9. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D3. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
/// [^3]: ADR-0177, the row axis of a world is a latitude that the world states, decision D5. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
#[must_use]
pub fn season_at(tick: Tick, latitude: i32) -> i32 {
    let table = insolation();
    table.shape_at(latitude, declination_at(tick))
}

/// Returns the pressure that the banded circulation adds at one latitude.
///
/// **The three cells of each hemisphere cannot emerge from this model, so the
/// model imposes them.** The middle cell of the three is thermally indirect
/// and eddy driven. A single-layer field has no vertical structure, so it has
/// no baroclinic eddies, so it never grows that cell. Three belts also need
/// three pressure extremes, and one temperature profile that falls from the
/// equator to the pole has two. The belts therefore never appear on their
/// own, whatever the field does.[^1]
///
/// **So the field adds one offset to the pressure before the gradient reads
/// it.** The offset is a low at the equator, a high at thirty degrees, a low
/// at sixty and a high at each pole, which is the published order of the
/// belts. It is one cosine of six times the latitude, so both hemispheres
/// carry the same shape without a second statement of it.[^1]
///
/// The high at thirty degrees is the mechanism that makes a desert belt. Air
/// leaves a high, so the subtropics diverge and the equator converges.
///
/// **The offset never reaches the temperature.** It reaches the wind and
/// nothing else, so the capacity of the air and the rain still read the
/// temperature that the sun and the ground gave.[^1]
///
/// The amplitude is a tuning constant. The report could not verify the
/// pressure of each belt, and a blocker holds what the wind should be
/// worth.[^1] [^2]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 5.1 and 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
#[must_use]
pub fn band_pressure_at(latitude: i32) -> i32 {
    // A quarter turn ahead of six times the latitude gives the cosine of six
    // times the latitude. The sign is negative, so the equator is a low.
    let phase = 6 * i64::from(latitude) + i64::from(LATITUDE_POLE);
    let wave = sim_math::sine(phase, TURN_FINE).unwrap_or(Fix32::ZERO);
    -narrow(sim_math::share(
        Accum(i64::from(BAND_SWING)),
        Accum(i64::from(wave.0)),
        Accum(i64::from(Fix32::ONE.0)),
    ))
}

/// The daily mean insolation of the globe, against the latitude and the
/// declination.
///
/// **The table is a pure function of the published geometry and of nothing
/// else.** It holds no world, no span and no pitch. A world reads a slice of
/// it that its own span chooses, so the region reading and the planet reading
/// differ by one constant rather than by a model.[^1]
///
/// The table is built once by integer arithmetic from the sine table that the
/// arithmetic module holds. It is never built with floating point, because
/// the result enters simulated state.[^2]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 4.4 and 9. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
struct Insolation {
    /// The daily mean insolation, in watts for each square metre, at each
    /// latitude band and each declination step.
    daily: Vec<i16>,
    /// The annual mean insolation at each latitude band.
    mean: Vec<i16>,
    /// The daily value at the solstice against the annual mean, at the middle
    /// latitude. It is the normaliser of the season part.
    reference: i16,
    /// The degrees that a saturated sky takes away from a cell.
    cloud_swing: i32,
}

/// The latitude bands that the insolation table holds.
const INSOLATION_BANDS: usize = 256;

/// The declination steps that the insolation table holds.
///
/// The table holds one more entry than this, because the last step is the
/// solstice itself and the interpolation reads the entry after the one it
/// starts from.
const DECLINATION_STEPS: usize = 64;

/// The tilt of the world, in hundredths of a degree.
///
/// This is the published obliquity of the Earth. It is what creates the
/// season, and the report keeps it and drops every other orbital term.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 4.1 and 4.2. `docs/research/reports/30-the-published-atmospheric-math.md`
const OBLIQUITY: i32 = 2344;

/// The solar constant, in watts for each square metre.
///
/// This is the published figure of the 2015 resolution of the International
/// Astronomical Union.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 4.1. `docs/research/reports/30-the-published-atmospheric-math.md`
const SOLAR_CONSTANT: i64 = 1361;

/// The ratio of a circumference to a diameter, in the fixed-point form of the
/// project.
///
/// The insolation form divides by it once. It is a mathematical constant and
/// not a measurement, so it belongs beside the arithmetic that reads it.
const PI_FIXED: i64 = 205_887;

/// The share of the incoming radiation that the planet keeps, in hundredths.
///
/// **Published, and held constant.** The published model this module takes its
/// belt from holds the albedo constant, so it carries no ice feedback and it
/// settles warmer at a pole than a planet with ice does. The engine inherits
/// that bias knowingly.[^1]
///
/// # References
///
/// [^1]: Balance register, the energy balance constants. `docs/reference/balance.md`
const ALBEDO_KEPT: i64 = 70;

/// The constant term of the outgoing radiation, in watts for each square
/// metre.[^1]
///
/// # References
///
/// [^1]: Balance register, the energy balance constants. `docs/reference/balance.md`
const OLR_INTERCEPT: i64 = 210;

/// The slope of the outgoing radiation, in hundredths of a watt for each
/// square metre for each degree.
///
/// **This is what turns a radiation anomaly into degrees**, so every term of
/// the driver that starts in watts reads it.[^1]
///
/// # References
///
/// [^1]: Balance register, the energy balance constants. `docs/reference/balance.md`
const OLR_SLOPE_FINE: i64 = 200;

/// The poleward diffusion of the published model, in hundredths of a watt for
/// each square metre for each degree.
///
/// **The engine never runs this diffusion.** It takes the profile the model
/// settles at, which is a closed form. A diffusion on the carried temperature
/// changed nothing at any affordable pass count, because the driver relaxes
/// each cell to a local value far faster than any affordable diffusion moves
/// heat between cells.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-602. `docs/FINDINGS.md`
const POLEWARD_DIFFUSION_FINE: i64 = 55;

/// What a radiation anomaly that varies with the latitude is divided by,
/// after the diffusion damps it.
///
/// **The diffusion damps each Legendre mode by its own order**, and the annual
/// mean insolation of a globe is a constant plus one second Legendre
/// polynomial to within a few percent. The second mode carries a factor of
/// six, so the belt divides by the slope plus six times the diffusion while a
/// term that does not vary with the latitude divides by the slope alone.[^1]
///
/// # References
///
/// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D2. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
const DAMPED_SLOPE_FINE: i64 = OLR_SLOPE_FINE + 6 * POLEWARD_DIFFUSION_FINE;

/// The share of the incoming radiation that ice reflects beyond bare ground,
/// in hundredths.
///
/// **Published.** A diffusive energy balance model that carries an ice albedo
/// takes the absorbed share as about 0.68 where there is no ice and about
/// 0.38 where there is, so ice reflects this much more than the ground it
/// covers.
const ICE_ALBEDO_DROP_FINE: i64 = 30;

/// The temperature at which half of a cell carries ice, in hundredths of a
/// degree.
///
/// **Published.** The ice line of the reference model stands here.
const ICE_MIDPOINT: i64 = -10 * DEGREE_FINE;

/// The temperature range over which a cell goes from bare to wholly iced, in
/// hundredths of a degree.
///
/// **This width is a stability condition and the record states it as one.**
/// The albedo term spans about 19 degrees between a bare cell and an iced
/// one. A cell that cools grows ice, and the ice cools it further, so the
/// feedback has a gain of that span divided by this width. **Above a gain of
/// one the loop runs away**, which is the published instability that takes a
/// planet to a frozen state. This width holds the gain near two thirds.
///
/// A narrower width is not more accurate. It is the same physics with the
/// runaway left in.
const ICE_RAMP: i64 = 30 * DEGREE_FINE;

/// The share of the world that carries ice in the published mean, in
/// hundredths.
///
/// **The balance is calibrated on a planet that already carries this ice**, so
/// the albedo term is an anomaly about it and not an addition to it. A cell at
/// this share moves nothing. Adding the whole ice albedo instead would count
/// the same ice twice, which is the defect the cloud term carried.[^1]
///
/// # References
///
/// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
const MEAN_ICE_FINE: i64 = 10;

/// The radiation that reaches the top of the atmosphere, in watts for each
/// square metre, averaged over the globe.
///
/// The albedo acts on what arrives and not on what is already absorbed, so
/// this is the quarter of the solar constant and not the absorbed share.
const INCIDENT_MEAN: i64 = SOLAR_CONSTANT / 4;

/// Returns the share of a cell that carries ice, in hundredths, from the
/// temperature the cell is carrying.
///
/// **The reader takes the carried temperature and never the asked one.** The
/// albedo changes what the world asks of a cell, so a term that read the asked
/// value would be a loop with itself. Reading the carried value makes the
/// feedback explicit: it runs once for each pass, at a count the solve fixes,
/// and no pass tests whether it settled.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[must_use]
pub fn ice_share_of(warmth: i32) -> i64 {
    let degrees = i64::from(warmth) * i64::from(WARMTH_FINE) + i64::from(WARMTH_FLOOR);
    let top = ICE_MIDPOINT + ICE_RAMP / 2;
    let apart = top - degrees;
    if apart <= 0 {
        return 0;
    }
    if apart >= ICE_RAMP {
        return 100;
    }
    apart * 100 / ICE_RAMP
}

/// Returns the degrees that the ice of a cell takes off the balance, in
/// hundredths.
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
#[must_use]
pub fn ice_forcing_of(warmth: i32) -> i64 {
    let anomaly = ice_share_of(warmth) - MEAN_ICE_FINE;
    // The watts the anomaly reflects away, and then the degrees those watts
    // are worth. The anomaly varies with the latitude, so it divides by the
    // damped slope in the way the belt does.
    let watts = ICE_ALBEDO_DROP_FINE * anomaly * INCIDENT_MEAN / (100 * 100);
    -watts * DEGREE_FINE * 100 / DAMPED_SLOPE_FINE
}

/// The hundredths of a degree in one whole degree.
const DEGREE_FINE: i64 = 100;

/// The warmth that water melts at.
///
/// **This is the zero of the freeze bank and it is a physical constant.** It
/// is not a mean that a measurement fitted, so a cell that never reaches it
/// never touches the clamp.[^1]
///
/// # References
///
/// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D6. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
pub const MELTING_WARMTH: i32 = warmth_of_hundredths(0);

/// The freezing that grew the ice which one warmth unit of rise melts.
///
/// **This is a balance value and a blocker governs it. It is not
/// published.** The deposit of the bank counts freezing over time and the
/// withdrawal spends warmth, so one constant converts between the two. That
/// constant is the latent heat of fusion divided by the heat capacity of the
/// surface, in the units of this field, and this project can verify neither
/// quantity. The balance register holds the row and how the value was
/// reached.[^1] [^2] [^3]
///
/// # References
///
/// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D4. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^3]: Balance register, the weather values. `docs/reference/balance.md`
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
pub const MELT_COST: i64 = 16;

/// The most freezing that one cell banks.
///
/// **The bound is arithmetic over two constants this module already
/// declares**, and it states no new figure. It is what one whole season
/// period deposits when a cell stands the whole of the heat scale below the
/// melting point for the whole of that period.
///
/// **The bound is what makes an ice cap possible and it is not a lever.** A
/// cell whose winter deposits more than its summer withdraws carries a
/// residue into the next year, the residue grows, and the summer of that cell
/// is then held for ever. This stops the count from running away.[^1]
///
/// # References
///
/// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D5. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
const FREEZE_BANK_CEILING: i64 = SEASON_PERIOD_TICKS * HEAT_CEILING as i64;

/// Returns the temperature that a cell may stand at, and the freeze bank that
/// it carries out of the solve.
///
/// **Ground that carries ice stays at the melting point until the ice is
/// gone.** The energy that arrives melts the ice rather than warming the
/// ground under it, and the latent heat of fusion of water is large. The polar
/// summer of a real planet stays at the melting point for this reason, and a
/// model without the term lets the polar summer run.[^1] [^2]
///
/// The deposit is the depth of the cell below the melting point, and it costs
/// the cell nothing. **The freezing of a real surface does stall it, and this
/// leaves that out**: a water surface stays near its freezing point while it
/// freezes over, and the ice then radiates from its own top and cools freely.
/// This field carries one temperature for a cell, and that temperature is the
/// surface. So a reader who expects a plateau at each end finds only one.[^3]
///
/// The withdrawal is one subtraction. **There is no melt loop and no trip
/// count that depends on the values.**[^4]
///
/// **The reader takes the settled temperature and not one step of it.** The
/// driver moves a cell and the wind then carries heat onto it, so a clamp
/// inside one of the two passes lets the other one lift a cell over the
/// melting point with ice still standing. A test found that hole.[^5]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decisions D2 and D3. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
/// [^2]: Findings register, FND-619. `docs/FINDINGS.md`
/// [^3]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D2. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
/// [^4]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decisions D1 and D2. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
/// [^5]: Findings register, FND-630. `docs/FINDINGS.md`
#[must_use]
pub fn frozen_hold(held: i32, bank: i64) -> (i32, i64) {
    // The deposit. A cell below the melting point grows ice for as long as it
    // stands there, and one solve adds the depth it stands below.
    let below = i64::from((MELTING_WARMTH - held).max(0));
    let bank = (bank + below).clamp(0, FREEZE_BANK_CEILING);
    let over = i64::from(held - MELTING_WARMTH);
    if over <= 0 || bank <= 0 {
        return (held, bank);
    }
    // The withdrawal. The cell gives up as much of its rise above the melting
    // point as the bank pays for, and each unit given up spends what that unit
    // of ice cost to grow.
    let paid = (bank / MELT_COST).min(over);
    (held - paid as i32, bank - paid * MELT_COST)
}

/// Returns the warmth that one temperature in hundredths of a degree stands
/// at on the heat scale.
///
/// **The scale is an affine map to a temperature and the module declares it
/// once.** This is the inverse of that map, so every term of the driver can be
/// stated in degrees and converted here.[^1]
///
/// # References
///
/// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D1. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
const fn warmth_of_hundredths(hundredths: i64) -> i32 {
    let steps = (hundredths - WARMTH_FLOOR as i64) / WARMTH_FINE as i64;
    if steps < 0 {
        0
    } else if steps > HEAT_CEILING as i64 {
        HEAT_CEILING
    } else {
        steps as i32
    }
}

/// The relief of a world, as the metres between its lowest ground and its
/// highest.
///
/// **A world states its own vertical range, in the way it states its latitude
/// span.** The terrain declares a height as a unit fraction, and this is what
/// the whole fraction is worth. A world that wants taller mountains changes
/// this value and not the physics.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
/// [^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeightRange {
    metres: i32,
}

impl HeightRange {
    /// The relief a world takes when the caller states none.
    ///
    /// The scale constants table holds the figure beside the tile edge.[^1]
    ///
    /// **A world states its own relief, so this is a default and not a
    /// constant of the physics.** A caller that wants a different vertical
    /// scale passes one.
    ///
    /// **The figure is what the lapse rate reads a cell against, and not what
    /// the lapse rate takes off the world.** The ground term is an anomaly
    /// about the mean land height, so the range sets how far cells stand
    /// apart and not how cold the world is.[^2]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    /// [^2]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    pub const DEFAULT: Self = Self { metres: 4000 };

    /// Builds a relief from a height in metres.
    ///
    /// # Errors
    ///
    /// Returns an error when the relief is not positive.
    pub const fn new(metres: i32) -> Result<Self, WeatherError> {
        if metres <= 0 {
            return Err(WeatherError::ReliefNotPositive(metres));
        }
        Ok(Self { metres })
    }

    /// Returns the relief in metres.
    #[must_use]
    pub const fn metres(self) -> i32 {
        self.metres
    }
}

/// The environmental lapse rate, in hundredths of a degree for each kilometre
/// of height.
///
/// **This is the published mean and not a chosen figure.** Air cools as it
/// rises, and the standard atmosphere uses this rate. The saturated rate is
/// lower and the dry rate is higher, and the research report recommends one
/// rate for a field at this scale because the difference across one ridge is
/// about two degrees and no watcher sees it.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 2.1 and 2.3. `docs/research/reports/30-the-published-atmospheric-math.md`
const LAPSE_RATE_FINE: i64 = 650;

/// The metres in one kilometre.
const METRES_IN_KILOMETRE: i64 = 1000;

/// The degrees that the belt of a latitude adds at the equator.
const LATITUDE_SWING: i32 = 40;

/// The degrees that the season adds at the middle latitude at the solstice.
const SEASON_ANOMALY_SWING: i32 = 40;

/// The pressure that the banded circulation adds at a subtropical high.
///
/// **No published amplitude stands behind this.** The report verified the
/// position of each belt and not its pressure, and a blocker holds the
/// question of what the wind should be worth.[^1] [^2]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, sections 5.1 and 12. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
const BAND_SWING: i32 = 32;

impl Insolation {
    /// Builds the table from the published geometry.
    fn build() -> Self {
        let mut daily = vec![0i16; INSOLATION_BANDS * (DECLINATION_STEPS + 1)];
        let mut mean = vec![0i16; INSOLATION_BANDS];
        for band in 0..INSOLATION_BANDS {
            let latitude = Self::latitude_of_band(band);
            for step in 0..=DECLINATION_STEPS {
                let declination = -i64::from(OBLIQUITY)
                    + 2 * i64::from(OBLIQUITY) * step as i64 / DECLINATION_STEPS as i64;
                daily[band * (DECLINATION_STEPS + 1) + step] =
                    clamp_to_watts(daily_insolation(latitude, declination as i32));
            }
            // The annual mean walks the year in equal steps of time, and not
            // in equal steps of declination. The declination is a sine of the
            // time, so the two are not the same average.
            let mut total = 0i64;
            for step in 0..DECLINATION_STEPS {
                let wave =
                    sim_math::sine(step as i64, DECLINATION_STEPS as i64).unwrap_or(Fix32::ZERO);
                let declination = narrow(sim_math::share(
                    Accum(i64::from(OBLIQUITY)),
                    Accum(i64::from(wave.0)),
                    Accum(i64::from(Fix32::ONE.0)),
                ));
                total += daily_insolation(latitude, declination);
            }
            mean[band] = clamp_to_watts(total / DECLINATION_STEPS as i64);
        }
        // **The normaliser is the largest anomaly the geometry holds
        // anywhere, and it stands at a pole.** A normaliser read at a chosen
        // latitude makes every latitude beyond it clamp, which replaces the
        // geometry with one number over the whole of the high latitudes.[^1]
        //
        // [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D3. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
        let mut reference = 1i64;
        for band in 0..INSOLATION_BANDS {
            for step in 0..=DECLINATION_STEPS {
                let apart = (i64::from(daily[band * (DECLINATION_STEPS + 1) + step])
                    - i64::from(mean[band]))
                .abs();
                if apart > reference {
                    reference = apart;
                }
            }
        }
        let mut table = Self {
            daily,
            mean,
            reference: clamp_to_watts(reference),
            // The derivation below needs a table to read. This is what it
            // writes.
            cloud_swing: 0,
        };
        // **What a whole sky takes away, derived and not written down.** The
        // sun term already states what one watt of insolation is worth in
        // degrees of warmth: the belt maps the annual mean range of the globe
        // onto twice the belt amplitude, and the normaliser then maps the sum
        // onto twice the swing the heat scale reserves. So the cloud reads
        // the published effect of cloud in watts and converts it on the same
        // scale. A cloud that shades the ground is worth what the sun it
        // shades is worth, and nothing else.
        let full_sky = CLOUD_EFFECT_WATTS * 100 / CLOUD_COVER_FINE.max(1);
        let hundredths = full_sky * DEGREE_FINE * 100 / OLR_SLOPE_FINE;
        table.cloud_swing =
            (hundredths / i64::from(WARMTH_FINE)).clamp(0, i64::from(HEAT_CEILING)) as i32;

        // **The coldest cell must not clamp at the bottom of the scale.** The
        // top of the scale needs no check, because the base is derived from
        // it. The check is here and not at a declaration, because the cloud
        // reads the table and a constant cannot read a table.
        assert!(
            table.cloud_swing < HEAT_CEILING,
            "a whole sky must not take the scale away"
        );
        table
    }

    /// Returns the sum of the belt and the season at one latitude and one
    /// declination, before the sum is normalised.
    fn shape_at(&self, latitude: i32, declination: i32) -> i32 {
        self.shape_of(self.daily_at(latitude, declination), self.mean_at(latitude))
    }

    /// Returns the sum of the belt and the season from one daily value and
    /// one annual mean.
    ///
    /// **The belt is absolute and the season is a swing about it.** The belt
    /// carries the level, so the driver needs no separate base. Nothing
    /// clamps: the season is normalised against the largest anomaly the
    /// geometry holds anywhere, and no latitude can exceed it.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decisions D2 and D3. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    /// [^2]: Research report 30, the published atmospheric math, section 4.3. `docs/research/reports/30-the-published-atmospheric-math.md`
    /// [^3]: Recurring Defect Shapes, redundant declaration sites. `.agents/rules/recurring-defects.md`
    /// [^4]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    fn shape_of(&self, daily: i64, mean: i64) -> i32 {
        // **The belt is the settled profile of the published energy balance,
        // and it is stated in degrees.** The globe keeps a share of what
        // reaches it, radiates a linear function of its temperature, and
        // diffuses heat along the latitude. The diffusion damps the second
        // Legendre mode by six times its own strength, and the annual mean
        // insolation is a constant plus that mode to within a few percent, so
        // the settled profile is arithmetic and not a solver.[^1]
        let global = SOLAR_CONSTANT * ALBEDO_KEPT / (4 * 100);
        let base = (global - OLR_INTERCEPT) * DEGREE_FINE * 100 / OLR_SLOPE_FINE;
        let absorbed = mean * ALBEDO_KEPT / 100;
        let belt = (absorbed - global) * DEGREE_FINE * 100 / DAMPED_SLOPE_FINE;
        // **The season is the daily value against the annual mean of the same
        // latitude**, so it is zero at the equinox everywhere. Its shape comes
        // from the geometry and its size is a balance value that a blocker
        // governs, because no published heat capacity this project could
        // verify fixes it.[^2]
        //
        // **The season is an absorbed anomaly, so the albedo scales it in the
        // way it scales the belt.** The belt multiplies the annual mean
        // insolation by the share the globe keeps, and the season multiplied
        // the anomaly by nothing at all. That is one value stated in two
        // places, and the copies disagreed: a pole reflects most of the sun
        // its polar day delivers, and the term gave it every watt.[^3] [^4]
        //
        // **The share is read at the annual mean of the latitude and not at
        // the temperature of the moment, so no loop exists.** The annual mean
        // is the belt above, which does not depend on the season, so the term
        // cannot drive itself. This is why it needs no stability condition,
        // where the ice term in the driver does.[^4]
        let annual = warmth_of_hundredths(base + belt);
        let kept = ALBEDO_KEPT - ICE_ALBEDO_DROP_FINE * ice_share_of(annual) / 100;
        let season = narrow(sim_math::share(
            Accum(i64::from(SEASON_ANOMALY_SWING) * kept / ALBEDO_KEPT),
            Accum(daily - mean),
            Accum(i64::from(self.reference).max(1)),
        ));
        annual + season
    }

    /// Returns the latitude of one band, in hundredths of a degree.
    fn latitude_of_band(band: usize) -> i32 {
        let span = 2 * i64::from(LATITUDE_POLE);
        (-i64::from(LATITUDE_POLE) + span * band as i64 / (INSOLATION_BANDS as i64 - 1)) as i32
    }

    /// Returns the band of one latitude, in bands scaled by the fixed-point
    /// unit, so that the caller can interpolate between two bands.
    fn band_of_latitude(latitude: i32) -> i64 {
        let one = i64::from(Fix32::ONE.0);
        let low = -i64::from(LATITUDE_POLE);
        let span = 2 * i64::from(LATITUDE_POLE);
        let scaled = (i64::from(latitude) - low) * (INSOLATION_BANDS as i64 - 1) * one / span;
        scaled.clamp(0, (INSOLATION_BANDS as i64 - 1) * one)
    }

    /// Returns the annual mean insolation at one latitude.
    fn mean_at(&self, latitude: i32) -> i64 {
        let one = i64::from(Fix32::ONE.0);
        let scaled = Self::band_of_latitude(latitude);
        let band = (scaled / one) as usize;
        let within = scaled - (band as i64) * one;
        let low = i64::from(self.mean[band]);
        let high = i64::from(self.mean[(band + 1).min(INSOLATION_BANDS - 1)]);
        low + (high - low) * within / one
    }

    /// Returns the daily mean insolation at one latitude and one declination.
    fn daily_at(&self, latitude: i32, declination: i32) -> i64 {
        let one = i64::from(Fix32::ONE.0);
        let scaled = Self::band_of_latitude(latitude);
        let band = (scaled / one) as usize;
        let within = scaled - (band as i64) * one;
        let below = self.at_band(band, declination);
        let above = self.at_band((band + 1).min(INSOLATION_BANDS - 1), declination);
        below + (above - below) * within / one
    }

    /// Returns the daily mean insolation of one band at one declination.
    fn at_band(&self, band: usize, declination: i32) -> i64 {
        let one = i64::from(Fix32::ONE.0);
        let span = 2 * i64::from(OBLIQUITY);
        let scaled =
            ((i64::from(declination) + i64::from(OBLIQUITY)) * DECLINATION_STEPS as i64 * one
                / span)
                .clamp(0, DECLINATION_STEPS as i64 * one);
        let step = (scaled / one) as usize;
        let within = scaled - (step as i64) * one;
        let row = band * (DECLINATION_STEPS + 1);
        let low = i64::from(self.daily[row + step]);
        let high = i64::from(self.daily[row + (step + 1).min(DECLINATION_STEPS)]);
        low + (high - low) * within / one
    }
}

/// Clamps a wide value into the watt range the table stores.
fn clamp_to_watts(value: i64) -> i16 {
    value.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16
}

/// Returns the table, building it on the first call.
///
/// **The table is immutable and it is a pure function of constants.** So one
/// build serves every field, every thread and every run, and the same index
/// gives the same value on every target.
fn insolation() -> &'static Insolation {
    static TABLE: std::sync::OnceLock<Insolation> = std::sync::OnceLock::new();
    TABLE.get_or_init(Insolation::build)
}

/// Returns the daily mean insolation at the top of the atmosphere.
///
/// The answer is in watts for each square metre. The latitude and the
/// declination are both in hundredths of a degree.
///
/// **This is the published closed form.** The half day length comes from the
/// latitude and the declination, and it is a whole half turn under a polar
/// day and nothing under a polar night. That is what makes the curve rise
/// again toward the summer pole.[^1]
///
/// The arithmetic goes through the arithmetic module. The sine reads the
/// table that module holds, and the inverse cosine searches the same
/// table.[^2]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 4.1. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
///
/// **This is public so that a test can hold it against the published
/// table.** The report states the daily mean at six latitudes at three points
/// of the year, and a test that reads this reads the geometry rather than the
/// term the temperature takes from it.
#[must_use]
pub fn daily_insolation(latitude: i32, declination: i32) -> i64 {
    let one = i64::from(Fix32::ONE.0);
    let quarter = i64::from(LATITUDE_POLE);
    let sine_of =
        |degrees: i64| i64::from(sim_math::sine(degrees, TURN_FINE).unwrap_or(Fix32::ZERO).0);
    let sin_latitude = sine_of(i64::from(latitude));
    let cos_latitude = sine_of(i64::from(latitude) + quarter);
    let sin_declination = sine_of(i64::from(declination));
    let cos_declination = sine_of(i64::from(declination) + quarter);
    // The two products of the published form. The half day length is the
    // inverse cosine of minus the first over the second, which avoids the
    // tangent and therefore avoids the pole.
    let along = sin_latitude * sin_declination / one;
    let across = cos_latitude * cos_declination / one;
    let half_day = if across <= 0 {
        if along > 0 {
            (sim_math::SINE_STEPS / 2) << 16
        } else {
            0
        }
    } else {
        let asked = (-along * one / across).clamp(-one, one);
        sim_math::arc_cosine_steps(Fix32(asked as i32))
    };
    // The first term of the published form. The half day length is in table
    // steps, and a half turn is half the steps, so the term divides by that.
    let first = (along * half_day / (sim_math::SINE_STEPS / 2)) >> 16;
    // The second term. The sine of the half day length, divided by the ratio
    // of a circumference to a diameter.
    let sine_of_half = i64::from(sim_math::sine_of_steps(half_day).0);
    let second = across * sine_of_half / one * one / PI_FIXED;
    SOLAR_CONSTANT * (first + second) / one
}

/// Returns the degrees the water in the air over one cell takes away.
///
/// **Cloud stands between the ground and the sun, and cloud is the air held
/// against what that air can hold.** Air well below its own capacity is
/// invisible vapour and shades nothing. Air at its own capacity is cloud,
/// whatever quantity of water that is.[^1]
///
/// The term stops at the whole swing, so a god who fills the sky cannot drive
/// the temperature below the bound.[^1]
///
/// **The whole swing is derived and it is not written down.** The sun term
/// states what one watt of insolation is worth in degrees of warmth, and the
/// build converts the published effect of cloud on the same scale. A written
/// figure here made a full sky worth about three times the published effect,
/// and it cooled the wet equator far more than the dry poles. So the equator
/// stood colder than the subtropics, and no land graded tropical.[^2] [^3]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D5. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
/// [^3]: Findings register, FND-586. `docs/FINDINGS.md`
#[must_use]
pub fn cloud_at(air: Drops, capacity: Drops) -> i32 {
    let swing = insolation().cloud_swing;
    // **The term is an anomaly about the mean cover, and never an absolute
    // subtraction.** The albedo the balance uses is the albedo of a planet
    // that already carries its mean cloud, so a term that took the whole
    // effect off every cell would count the same cloud twice. A cell under
    // the mean cover therefore warms and a cell above it cools.[^4]
    //
    // [^4]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    let cover = if capacity.0 <= 0 {
        CLOUD_SHARE_WHOLE
    } else {
        air.0.clamp(0, capacity.0) * CLOUD_SHARE_WHOLE / capacity.0
    };
    let mean = CLOUD_SHARE_WHOLE * CLOUD_COVER_FINE / 100;
    narrow(sim_math::share(
        Accum(i64::from(swing)),
        Accum(cover - mean),
        Accum(CLOUD_SHARE_WHOLE),
    ))
}

/// Returns the temperature that the world asks of one cell.
///
/// **Four terms drive it.** The ground term carries the height of the cell
/// and the water it holds, and a divisor keeps it from filling the scale on
/// its own. The season adds or takes degrees as the warm band goes past. The
/// cloud takes degrees where the air holds water. The sum is held inside the
/// scale.[^1]
///
/// The first two terms give a field that varies over the map. The season
/// gives a field that varies over time. The cloud is what makes the two
/// interact, because the air over a cell is the output of the transport that
/// the wind drives.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
#[must_use]
pub fn asked_warmth(relief: i32, season: i32, cloud: i32, held: i32) -> i32 {
    // **The relief arrives in hundredths of a degree and the scale steps by
    // half a degree.** The sun term already carries the level, so the relief
    // is a signed perturbation about it and never an addition to it.[^2]
    //
    // [^2]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decisions D1 and D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    let relief = (i64::from(relief) / i64::from(WARMTH_FINE)) as i32;
    // **The ice reads the temperature the cell is carrying.** A cold cell
    // grows ice, the ice reflects more than the ground it covers, and the
    // cell cools further. The term is an anomaly about the ice the published
    // balance already carries, so a cell at that share moves nothing.[^3]
    //
    // [^3]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    let ice = (ice_forcing_of(held) / i64::from(WARMTH_FINE)) as i32;
    (season + relief + ice - cloud).clamp(0, HEAT_CEILING)
}

/// Returns the numerator of the share of the air that falls on one cell.
///
/// **Rain is about how full the air is, not about how cold it is.** The
/// numerator rises with the air held against the capacity of that air. Air
/// well below its capacity gives the floor, which is a trace that keeps the
/// water cycling. Air at its capacity gives the whole term.
///
/// The settle pass already poured out everything above the capacity before it
/// asks this, so this term is the drizzle that a full sky gives and not the
/// downpour. The two together are what rain out air that cooled.
///
/// The old rule read the coldness of the cell. That made any air reaching a
/// high latitude rain out at once, so the poles were structurally
/// cloudless.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[must_use]
pub fn fall_numerator(air: Drops, capacity: Drops) -> i64 {
    if capacity.0 <= 0 {
        return FALL_NUMERATOR_FLOOR + FALL_FOR_FULL_AIR;
    }
    let held = air.0.clamp(0, capacity.0);
    let by_fullness = sim_math::share(Accum(FALL_FOR_FULL_AIR), Accum(held), Accum(capacity.0))
        .map_or(0, |value| value.0);
    FALL_NUMERATOR_FLOOR + by_fullness
}

/// A cyclone: a travelling low that the field carries as state.
///
/// # What this is, and what it is not
///
/// **A cyclone here is imposed. It does not form out of the field, and it
/// cannot.** A single-layer field grows no baroclinic eddies, so the three
/// circulation cells cannot emerge from it and the module imposes them
/// instead.[^1] The same argument holds one level down. A mesocyclone is an
/// instability of a layered atmosphere, and this atmosphere has one layer. So
/// the field places a low, carries it, and lets it die. Nothing here claims
/// that a storm grew.
///
/// **What is imposed is the pressure deficit and nothing else.** The wind
/// pass reads the deficit as it reads the belt of a latitude, and the
/// deflection then turns the inflow aside. A closed circulation is what a
/// deflected inflow is, and that part is the field's own arithmetic rather
/// than a shape written here.[^2]
///
/// # What a scale of one cell means
///
/// **A tornado is smaller than one cell of any lattice this engine builds,
/// and this type does not resolve one.** A cell of a world that spans pole to
/// pole over 128 rows is about 156 kilometres across.[^3] A tornado is under
/// one kilometre. So the small violent setting below is an intensity carried
/// on one cell, and it stands for a severe local storm rather than a funnel.
/// A reader who takes it for a resolved tornado is reading a claim that the
/// lattice cannot support.
///
/// The large setting is different. A tropical cyclone runs to several hundred
/// kilometres, which is a few cells, so the lattice does resolve its
/// structure and the figures under it mean something.
///
/// # The parameters
///
/// **One shape carries both settings.** The three fields below are what
/// separate the small violent storm from the large sustained one, and every
/// other quantity follows from them: the wind from the deficit over the
/// radius, the rain from the deficit at the cell, and the end from the life
/// and from the ground under the eye.
///
/// The type declares its layout, because it reaches the state hash. Seven
/// four-byte fields fill twenty-eight bytes exactly, so the type needs no
/// padding field.[^4]
///
/// # References
///
/// [^1]: The banded circulation. [`band_pressure_at`]
/// [^2]: The deflection. [`deflect`]
/// [^3]: Findings register, FND-618. `docs/FINDINGS.md`
/// [^4]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Cyclone {
    /// The position along the first lattice axis, in sub-cell steps of the
    /// whole lattice.
    pub q_fine: i32,
    /// The position along the second lattice axis, in the same steps.
    pub r_fine: i32,
    /// The pressure deficit at the eye, in the units that the temperature
    /// plane carries. It falls as the storm dies.
    pub depth: i32,
    /// The cells that the deficit reaches from the eye. A radius of zero is
    /// one cell.
    pub radius: i32,
    /// The solves that the storm lives from its birth.
    pub life: u32,
    /// The solves that it has lived.
    pub age: u32,
    /// The identity of the storm, which keys its wander draw.
    ///
    /// **The slot is not the identity.** A storm that dies frees its slot,
    /// and the next storm in that slot must not draw what the dead one drew.
    /// The field counts identities and never reuses one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing rules, section 2. `.agents/rules/testing.md`
    pub id: u32,
}

/// The settings that raise one cyclone.
///
/// **This is the parameter set, and the two constants below are two points in
/// it.** They are not two mechanisms and they are not two types. A caller may
/// name any other point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CycloneSetting {
    /// The pressure deficit at the eye, in the units that the temperature
    /// plane carries.
    pub depth: i32,
    /// The cells that the deficit reaches from the eye.
    pub radius: i32,
    /// The solves that the storm lives.
    pub life: u32,
}

impl CycloneSetting {
    /// A small violent storm, on one cell, over quickly.
    ///
    /// **This is not a resolved tornado.** One cell of a planet-wide lattice
    /// is two orders of magnitude wider than a funnel, so this setting is an
    /// intensity carried on a cell and nothing finer. It is the honest
    /// reading of a tornado-level storm at this pitch.
    pub const SEVERE: Self = Self {
        depth: CYCLONE_DEPTH_CEILING,
        radius: 0,
        life: 24,
    };

    /// A large sustained storm, over several cells, lasting many solves.
    ///
    /// The lattice does resolve this one. A tropical cyclone runs to several
    /// hundred kilometres, and a cell of a planet-wide lattice is about one
    /// hundred and fifty.
    pub const TROPICAL: Self = Self {
        depth: 40,
        radius: 3,
        life: 320,
    };

    /// A low on a front, shallower than a tropical cyclone.
    ///
    /// **This is the storm of the middle latitudes.** A real low there runs
    /// to more than a thousand kilometres and lives some days, and it forms
    /// on a temperature gradient rather than over a warm sea.[^1]
    ///
    /// The radius here is the reach on a lattice of the reference pitch. The
    /// genesis pass replaces it with the reach that its own lattice asks
    /// for.[^2]
    ///
    /// # References
    ///
    /// [^1]: Research report 30, the published atmospheric math, section 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
    /// [^2]: The reach of a storm. [`storm_reach`]
    pub const FRONTAL: Self = Self {
        depth: 32,
        radius: 3,
        life: 320,
    };

    /// Returns the setting with the radius replaced.
    ///
    /// **A storm has one size on the ground, and the lattice decides how many
    /// cells that is.** So the reach is not a property of the setting, and a
    /// caller that knows the lattice replaces it.[^1]
    ///
    /// # References
    ///
    /// [^1]: The reach of a storm. [`storm_reach`]
    #[must_use]
    pub const fn with_reach(self, reach: i32) -> Self {
        Self {
            depth: self.depth,
            radius: reach,
            life: self.life,
        }
    }

    /// Reports whether the setting is inside the range that the field
    /// carries.
    #[must_use]
    pub const fn is_in_range(self) -> bool {
        self.depth >= CYCLONE_DEPTH_FLOOR
            && self.depth <= CYCLONE_DEPTH_CEILING
            && self.radius >= 0
            && self.radius <= CYCLONE_RADIUS_CEILING
            && self.life > 0
            && self.life <= CYCLONE_LIFE_CEILING
    }
}

impl Cyclone {
    /// Returns the cell of the whole lattice that the eye stands over.
    #[must_use]
    pub const fn eye(self) -> Axial {
        Axial::new(
            self.q_fine.div_euclid(CYCLONE_FINE),
            self.r_fine.div_euclid(CYCLONE_FINE),
        )
    }

    /// Returns the pressure deficit that the storm puts on one cell.
    ///
    /// **The deficit is a cone.** It stands at the depth over the eye and
    /// falls in a straight line to nothing one cell beyond the radius. So the
    /// gradient, which is what the wind answers to, is the depth over the
    /// radius: a small deep storm is violent, and a large one of the same
    /// depth is broad and gentler. That is the whole difference between the
    /// two settings, and it is one division.
    ///
    /// **This is public so that a test can move one input and watch the
    /// answer move.**
    #[must_use]
    pub fn deficit_at(self, cell: Axial) -> i32 {
        let span = i64::from(self.eye().distance(cell));
        let reach = i64::from(self.radius) + 1;
        if span >= reach {
            return 0;
        }
        narrow(sim_math::share(
            Accum(i64::from(self.depth)),
            Accum(reach - span),
            Accum(reach),
        ))
    }

    /// Reports whether the storm is over.
    ///
    /// A storm ends on its age or on its depth. Each of the two is a
    /// comparison, and neither is a convergence test.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    #[must_use]
    pub const fn is_over(self) -> bool {
        self.age >= self.life || self.depth < CYCLONE_DEPTH_FLOOR
    }
}

/// The sub-cell steps that one cell of the lattice spans.
///
/// A storm carries its position in these steps, so it drifts across a cell
/// over several solves rather than jumping from one cell to the next.
const CYCLONE_FINE: i32 = 64;

/// The deepest eye that the field carries, in the units that the temperature
/// plane carries.
///
/// The belt of a latitude swings by thirty-two of the same units over the
/// whole globe, so a storm at this depth is a deeper low than any belt, and
/// it stands over one or a few cells rather than over a third of the world.
/// The value is a content constant that no measurement chose, and the blocker
/// that holds what the wind should be worth governs it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
pub const CYCLONE_DEPTH_CEILING: i32 = 96;

/// The shallowest eye that is still a storm. A storm that falls below it
/// ends.
pub const CYCLONE_DEPTH_FLOOR: i32 = 8;

/// The widest storm that the field carries, in cells from the eye.
///
/// **A storm has one size in degrees of latitude, so its size in cells
/// follows the lattice.** A fine lattice therefore asks for a larger radius
/// than a coarse one for the same storm, and this ceiling must reach what the
/// finest lattice asks.[^1]
///
/// # References
///
/// [^1]: The reach of a storm. [`storm_reach`]
pub const CYCLONE_RADIUS_CEILING: i32 = 32;

/// The longest life that the field carries, in solves.
pub const CYCLONE_LIFE_CEILING: u32 = 4096;

/// The storms that the field carries at once.
///
/// The stamp pass costs the footprint of every storm on every solve, so the
/// ceiling is what bounds that cost. It is a content constant that no
/// measurement chose.[^1]
///
/// **The ceiling does not follow the lattice, because a storm count is a
/// count of things on the ground rather than a density over cells.** A world
/// spans the same latitudes at every pitch, so it holds the same weather at
/// every pitch. A finer lattice draws each storm with more cells, and the
/// reach in cells carries that.[^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: The reach of a storm. [`storm_reach`]
pub const CYCLONE_CEILING: usize = 32;

/// What turns a wind into the sub-cell steps that a storm travels.
///
/// **This is a unit conversion and not a tuning constant.** A wind counts
/// eight fine steps to one lattice step, and a storm counts its position in
/// sub-cell steps of a cell. So the two constants that state those units are
/// what this is made of, and no third declaration of either exists.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
const CYCLONE_STEER_NUMERATOR: i64 = (CYCLONE_FINE / WIND_FINE) as i64;

/// The share of the steering flow that the eye travels at.
///
/// A real storm travels at about the speed of the flow it stands in, so the
/// share is the whole of it. The value is a content constant that no
/// measurement chose, and the blocker that holds what the wind should be
/// worth governs it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
const CYCLONE_STEER_DENOMINATOR: i64 = 1;

/// The sub-cell steps that the wander adds to each axis, either way.
///
/// The track of a real storm is not a straight line, and a straight line is
/// what a steering flow alone gives. The wander is a keyed draw and never a
/// thread-local one.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const CYCLONE_WANDER: i32 = 6;

/// The depth that a storm loses on each solve over land.
///
/// **A storm over land loses its source.** The engine does not model the
/// latent heat that feeds a real cyclone, so the decay is imposed in the way
/// that the storm itself is. A storm that made landfall dies within a few
/// tens of solves rather than lasting out its whole life.
const CYCLONE_LANDFALL_DECAY: i32 = 3;

/// The depth that a storm loses on each solve over a cold sea.
const CYCLONE_COLD_DECAY: i32 = 2;

/// The depth that a storm loses on each solve wherever it stands.
///
/// **Nothing here sustains a storm, so every storm is always dying.** The
/// figure is small, so a tropical storm over a warm sea still lives out most
/// of its life.
const CYCLONE_DECAY: i32 = 1;

/// The share of a cell that must admit a unit before the cell counts as land.
///
/// Water is the only ground that admits no unit, so a cell whose open share
/// stands above this mark is mostly land.
const CYCLONE_LAND_MARK: i64 = 1;

/// What that share is measured against.
const CYCLONE_LAND_WHOLE: i64 = 2;

/// The temperature at which the sea under a storm is warm enough to hold it.
///
/// The published figure for tropical cyclone genesis is a sea surface near
/// twenty-six degrees. The temperature plane counts half a degree in one unit
/// and stands at half the heat ceiling for a temperate cell, so this mark is
/// at the warm end of the scale rather than at the middle of it.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math. `docs/research/reports/30-the-published-atmospheric-math.md`
const CYCLONE_WARM_MARK: i32 = 3 * HEAT_CEILING / 5;

/// One in this many solves, one genesis attempt goes ahead.
///
/// **The attempt is a keyed draw, and the gates below it decide the rest.**
/// So the rate that a world sees is the product of this period, of the
/// attempts each solve makes, and of how much ground the gates admit.
const CYCLONE_GENESIS_PERIOD: u64 = 4;

/// The genesis attempts that one solve makes.
///
/// **One attempt each solve cannot fill a world.** A storm ends when its
/// depth falls under the floor, which takes tens of solves, so the standing
/// population is the genesis rate multiplied by that life. One attempt per
/// period, on one drawn cell, against gates that most cells fail, gave a
/// planet that stood empty most of the time.[^1]
///
/// Each attempt keys its two draws on an entity of its own, so the attempts
/// of one solve name different cells.[^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const CYCLONE_GENESIS_ATTEMPTS: u32 = 8;

/// The share of its capacity that the air over a cell must hold before a
/// storm may be raised there.
const CYCLONE_GENESIS_AIR_NUMERATOR: i64 = 1;

/// What that share is measured against.
const CYCLONE_GENESIS_AIR_DENOMINATOR: i64 = 4;

/// The draw index of the genesis attempt within a frame.
const GENESIS_DRAW: u32 = 1;

/// The draw index of the cell that a genesis attempt names.
const GENESIS_CELL_DRAW: u32 = 2;

/// The draw index of the first axis of the wander.
const WANDER_Q_DRAW: u32 = 3;

/// The draw index of the second axis of the wander.
const WANDER_R_DRAW: u32 = 4;

/// The entity that the genesis draws of the first attempt key on.
///
/// **Genesis has no entity, so it names one that no cell can name.** A cell
/// key is a 32-bit index, so this value collides with none of them.[^1]
///
/// Each attempt counts down from this value, so the attempts of one solve
/// hold different keys and the whole run of them stays clear of every cell
/// key and of every storm identity.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const GENESIS_ENTITY: u64 = u64::MAX;

/// Returns the entity that one genesis attempt keys its draws on.
const fn genesis_entity(attempt: u32) -> u64 {
    GENESIS_ENTITY - attempt as u64
}

/// The temperature difference across one whole degree of latitude that admits
/// a storm on a front.
///
/// **A storm grows on a temperature gradient, and a gradient is a difference
/// over a distance.** The gate therefore reads the difference between two
/// neighbours against the latitude that one cell spans, so one mark serves
/// every lattice pitch. A mark stated as a difference between neighbours
/// would admit storms everywhere on a coarse lattice and nowhere on a fine
/// one.[^1]
///
/// The unit is the unit of the temperature plane, and one step of that plane
/// is half a degree Celsius.[^2]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
/// [^2]: The temperature scale. [`WARMTH_FINE`]
const FRONT_MARK_FOR_EACH_DEGREE: i64 = 1;

/// The degrees of latitude that a storm reaches from its eye.
///
/// **A storm has one size on the ground, and the lattice decides how many
/// cells that is.** A tropical cyclone runs to some hundreds of kilometres
/// and an extratropical low to more than a thousand, so a reach of this many
/// degrees covers both at the fidelity this field carries.[^1]
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
const STORM_REACH_FINE: i64 = 11 * LATITUDE_FINE as i64;

/// Returns the cells that a storm reaches from its eye, on one lattice.
///
/// **The reach is a size on the ground, so the answer follows the latitude
/// that one row of the lattice spans.** A lattice of few rows spans many
/// degrees in each row and needs a small reach. A lattice of many rows needs
/// a large one for the same storm. A fixed reach in cells would draw a storm
/// of a different size on every world, and the picture would then depend on
/// the extent.[^1]
///
/// The answer is never zero, so a lattice too coarse to resolve a storm still
/// carries one cell of it.
///
/// # References
///
/// [^1]: Research report 30, the published atmospheric math, section 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
#[must_use]
pub fn storm_reach(latitudes: Latitudes, height: u32) -> i32 {
    if height == 0 || latitudes.span() <= 0 {
        return 1;
    }
    let cells = sim_math::share(
        Accum(STORM_REACH_FINE * i64::from(height)),
        Accum(1),
        Accum(i64::from(latitudes.span())),
    )
    .map_or(1, |value| value.0);
    cells.clamp(1, i64::from(CYCLONE_RADIUS_CEILING)) as i32
}

/// Returns the temperature difference between two neighbours that admits a
/// storm on a front, on one lattice.
///
/// **The mark is a gradient, so it follows the latitude that one row spans.**
/// A coarse lattice holds a large difference between two rows and needs a
/// large mark. The answer is never zero, so no lattice admits a front across
/// two cells of the same temperature.
#[must_use]
pub fn front_mark(latitudes: Latitudes, height: u32) -> i32 {
    if height == 0 || latitudes.span() <= 0 {
        return 1;
    }
    let steps = sim_math::share(
        Accum(FRONT_MARK_FOR_EACH_DEGREE * i64::from(latitudes.span())),
        Accum(1),
        Accum(i64::from(height) * i64::from(LATITUDE_FINE)),
    )
    .map_or(1, |value| value.0);
    steps.clamp(1, i64::from(HEAT_CEILING)) as i32
}

/// Reports whether one genesis attempt of one frame goes ahead.
///
/// The answer is a keyed draw on the world seed, the weather system, the
/// frame and the attempt. It holds no state, and it does not depend on which
/// thread asked.[^1]
///
/// **This is public so that a test can change one field of the key and watch
/// the answer move.**[^2]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: Testing rules, section 2. `.agents/rules/testing.md`
#[must_use]
pub fn cyclone_forms(seed: u64, tick: Tick, attempt: u32) -> bool {
    rng::draw_below(
        seed,
        rng::SYSTEM_WEATHER,
        tick.0,
        genesis_entity(attempt),
        GENESIS_DRAW,
        CYCLONE_GENESIS_PERIOD,
    ) == 0
}

/// Returns the cell of the whole lattice that a genesis attempt names.
///
/// The answer is a keyed draw over the cells of the lattice. The gates that
/// follow it decide whether that cell may hold a storm.
///
/// **This is public so that a test can change one field of the key and watch
/// the answer move.**
#[must_use]
pub fn cyclone_genesis_cell(seed: u64, tick: Tick, attempt: u32, cells: u32) -> u32 {
    if cells == 0 {
        return 0;
    }
    rng::draw_below(
        seed,
        rng::SYSTEM_WEATHER,
        tick.0,
        genesis_entity(attempt),
        GENESIS_CELL_DRAW,
        u64::from(cells),
    ) as u32
}

/// Returns the sub-cell steps that the track of one storm wanders this frame.
///
/// The draw is keyed on the world seed, the weather system, the frame and the
/// identity of the storm. **The identity is not the slot.** A storm that dies
/// frees its slot, and a storm born into that slot later must not repeat the
/// track of the dead one.[^1] [^2]
///
/// **This is public so that a test can change one field of the key and watch
/// the answer move.**[^2]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: Testing rules, section 2. `.agents/rules/testing.md`
#[must_use]
pub fn cyclone_wander(seed: u64, tick: Tick, id: u32) -> (i32, i32) {
    let span = (2 * CYCLONE_WANDER + 1) as u64;
    let along = rng::draw_below(
        seed,
        rng::SYSTEM_WEATHER,
        tick.0,
        u64::from(id),
        WANDER_Q_DRAW,
        span,
    ) as i32;
    let across = rng::draw_below(
        seed,
        rng::SYSTEM_WEATHER,
        tick.0,
        u64::from(id),
        WANDER_R_DRAW,
        span,
    ) as i32;
    (along - CYCLONE_WANDER, across - CYCLONE_WANDER)
}

/// The whole of the capacity that a cell outside every storm keeps.
const CYCLONE_KEPT_WHOLE: i64 = 64;

/// The share of its capacity that the deepest eye leaves the air.
const CYCLONE_KEPT_FLOOR: i64 = 8;

/// Returns the capacity of the air under a storm.
///
/// **A storm is forced ascent, and ascent rains.** The eye of the deepest
/// storm leaves the air an eighth of what its temperature would allow, so the
/// settle pass pours seven eighths of what stands there onto the ground. A
/// cell outside every footprint keeps the whole of its capacity and reads
/// nothing of this.
///
/// **The capacity is a bound and never an assignment.** The settle pass
/// computes the water above the bound and moves that quantity from the air
/// plane to the ground plane of the same cell, so the water account holds
/// through it.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
#[must_use]
pub fn cyclone_capacity(base: Drops, deficit: i32) -> Drops {
    if deficit <= 0 {
        return base;
    }
    let held = i64::from(deficit.clamp(0, CYCLONE_DEPTH_CEILING));
    let shed = sim_math::share(
        Accum(CYCLONE_KEPT_WHOLE - CYCLONE_KEPT_FLOOR),
        Accum(held),
        Accum(i64::from(CYCLONE_DEPTH_CEILING)),
    )
    .map_or(0, |value| value.0);
    share_of(base, CYCLONE_KEPT_WHOLE - shed, CYCLONE_KEPT_WHOLE)
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
    /// The cell lattice, and the margin of cells around the world that the
    /// solve steps and no reader sees.
    ///
    /// **The lattice is larger than the world.** The whole lattice is a hex
    /// grid at the pitch the scale states, widened on all four sides by the
    /// margin. The inner lattice is the part that covers the world. Every
    /// plane below is over the whole lattice, and every reader that names a
    /// cell names one of the whole lattice.[^1]
    ///
    /// A margin of zero makes the whole lattice the inner lattice, and the
    /// field then behaves as it did before the margin existed.
    ///
    /// # References
    ///
    /// [^1]: The padded lattice. [`PaddedLattice`]
    lattice: PaddedLattice,
    /// The pitch of the lattice, as the tiles along one cell side.
    ///
    /// **The resolution is a parameter and not a constant.** Three tuned
    /// quantities are derived from it, so the field reads them from here and
    /// holds no second copy of them.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    scale: WeatherScale,
    /// The latitudes that the rows of the world stand at.
    ///
    /// **The weather reads a latitude and never a raw row.** The span decides
    /// whether the world is a planet or one region of one, and it is the only
    /// thing that decides it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    latitudes: Latitudes,
    /// The pressure that the banded circulation adds to each row of the whole
    /// lattice.
    ///
    /// **The index is a row of the whole lattice and not a row of the
    /// world.** The lattice carries a margin, so the two are different, and a
    /// reader that confuses them shifts every belt by the margin.[^1]
    ///
    /// The table is a fixed function of the latitude of the row, so the field
    /// builds it once and the wind pass reads one entry for each cell.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    /// [^2]: ADR-0177, the row axis of a world is a latitude that the world states, decision D2. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    band: Vec<i32>,
    faction_count: u16,
    /// The relief of the world, which the lapse rate reads.
    ///
    /// **A world states it, in the way it states its latitude span.**[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D4. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    relief: HeightRange,
    /// The water in the air above each cell, in cell index order. It is empty
    /// until the first drop enters the world.
    air: Vec<Drops>,
    /// The water on the ground of each cell, in the same order.
    ground: Vec<Drops>,
    /// The write half of one transport pass.
    scratch: Vec<Drops>,
    /// The wind over each cell, in cell index order.
    ///
    /// **This is simulated state and it enters the state hash.** It exists
    /// over dry ground as well as wet, so it costs the lattice whether the
    /// air holds water or not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    wind: Vec<Wind>,
    /// The write half of one wind pass.
    ///
    /// A pass reads the settled plane above and writes this one. It never
    /// reads a plane the same pass is writing, so the answer does not depend
    /// on which cells a thread reached first.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D4. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    wind_scratch: Vec<Wind>,
    /// The temperature of each cell, in cell index order.
    ///
    /// **This is simulated state and it enters the state hash.** A solve
    /// moves it a share of the way toward the temperature the world asks for,
    /// and never assigns that temperature. The share is what gives the
    /// temperature its own history, and a parcel of warm air that the wind
    /// carries onto a cold ridge has to remember that it was warm.[^1]
    ///
    /// **This changes an earlier decision, which held that the heat is
    /// derived on every solve and stored nowhere.**[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    /// [^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    warmth: Vec<i32>,
    /// The write half of one temperature carry pass.
    ///
    /// A pass reads the settled plane above and writes this one. It never
    /// reads a plane the same pass is writing, so the answer does not depend
    /// on which cells a thread reached first.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D3. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    warmth_scratch: Vec<i32>,
    /// The freezing that each cell has taken and that its ice has not yet
    /// paid back.
    ///
    /// **This is simulated state and it enters the state hash.** A world that
    /// loads a saved bank and a world that recomputes one are different
    /// worlds, in the same way that the temperature makes them different.[^1]
    ///
    /// **The bank could not be derived.** The albedo term reads an ice share
    /// from the temperature a cell carries, and that reader holds no history:
    /// a cell that spent a winter frozen and a cell that reached the same
    /// temperature on this pass read alike. The clamp is a memory of the
    /// winter, so it needs a memory.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D1. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
    /// [^2]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
    frozen: Vec<i32>,
    /// The temperature passes that have run since the field was built.
    warmth_passes: u64,
    /// The wind passes that have run since the field was built.
    wind_passes: u64,
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
    /// The storms that the field is carrying, in ascending identity order.
    ///
    /// **This is simulated state and it enters the state hash.** A storm
    /// carries a position, a depth and an age, and the next solve reads all
    /// three.[^1]
    ///
    /// The order is the order the storms were raised in, and a storm that
    /// ends is removed without moving the ones before it. So the order is a
    /// fixed key and never a completion order.[^2]
    ///
    /// # References
    ///
    /// [^1]: A cyclone. [`Cyclone`]
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    cyclones: Vec<Cyclone>,
    /// The identity that the next storm takes.
    ///
    /// **A slot is reused and an identity is not.** The wander draw of a
    /// storm keys on the identity, so a storm born into the slot of a dead
    /// one would otherwise repeat the track of the dead one exactly.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing rules, section 2. `.agents/rules/testing.md`
    next_cyclone: u32,
    /// The pressure deficit that the storms put on each cell of the whole
    /// lattice.
    ///
    /// **This is derived and never carried.** One pass rebuilds the whole of
    /// it from the storms at the start of every solve, so it enters no state
    /// hash. It is stored rather than recomputed because the wind pass and
    /// the settle pass both read it, and a cell of it is a walk over every
    /// storm.
    ///
    /// The plane is empty while no storm stands, and every reader takes zero
    /// for a cell it does not hold.
    depression: Vec<i32>,
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
    /// **The lattice states its own margin, and the field steps the whole of
    /// it.** The caller decides how wide the margin is. A margin of zero
    /// gives the field the world and nothing more.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction count is above the ceiling the
    /// project supports.
    pub fn new(
        lattice: PaddedLattice,
        scale: WeatherScale,
        faction_count: u16,
    ) -> Result<Self, WeatherError> {
        Self::with_latitudes(lattice, scale, Latitudes::DEFAULT, faction_count)
    }

    /// Builds a field over a cell lattice, at a stated latitude span.
    ///
    /// **The span decides whether the world is a planet or one region of
    /// one.** A span from pole to pole gives poles, a banded circulation and
    /// a season that reverses across the equator. A narrow span gives a flat
    /// latitude term, and the climate then comes from the ground and the sea
    /// alone.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the faction count is above the ceiling the
    /// project supports.
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    pub fn with_latitudes(
        lattice: PaddedLattice,
        scale: WeatherScale,
        latitudes: Latitudes,
        faction_count: u16,
    ) -> Result<Self, WeatherError> {
        if faction_count > FACTION_CEILING {
            return Err(WeatherError::FactionCountAboveCeiling(faction_count));
        }
        // **The wind exists over dry ground.** So the wind plane is built with
        // the field and never lazily, unlike the two water planes. The record
        // states that cost and the project owner chose it.[^1]
        //
        // [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
        let count = lattice.whole().tile_count() as usize;
        // **The belt table is indexed by a row of the whole lattice.** The
        // latitude of a row is the latitude of the world row under it, so a
        // margin row beyond a pole carries the pressure of the pole. That is
        // the same rule the temperature already holds for a margin row.[^2]
        //
        // [^2]: The padded lattice. [`PaddedLattice::inner_row_of`]
        let rows = lattice.whole().height();
        let world_rows = lattice.inner().height();
        let ring = lattice.ring();
        let band = (0..rows)
            .map(|row| {
                let world_row = row.saturating_sub(ring).min(world_rows.saturating_sub(1));
                band_pressure_at(latitudes.of_row(world_row, world_rows))
            })
            .collect();
        Ok(Self {
            lattice,
            scale,
            latitudes,
            band,
            faction_count,
            relief: HeightRange::DEFAULT,
            air: Vec::new(),
            ground: Vec::new(),
            scratch: Vec::new(),
            wind: vec![Wind::STILL; count],
            wind_scratch: vec![Wind::STILL; count],
            // **A world starts at the middle of the scale, not at the
            // bottom.** A field that began at zero would warm from cold over
            // the first passes, and a reader would take that warming for
            // weather.
            warmth: vec![HEAT_CEILING / 2; count],
            warmth_scratch: vec![HEAT_CEILING / 2; count],
            // **A world starts with no ice.** A field that began with a bank
            // would hold a winter that never happened.
            frozen: vec![0; count],
            warmth_passes: 0,
            wind_passes: 0,
            raised: 0,
            storms: 0,
            cyclones: Vec::new(),
            next_cyclone: 0,
            depression: Vec::new(),
            evaporated: 0,
            ready: vec![Tick(0); faction_count as usize],
            passes: 0,
        })
    }

    /// Returns the whole cell lattice that the solve steps.
    ///
    /// **This is larger than the world when the field carries a margin.**
    /// Read [`WeatherField::lattice`] and take the inner lattice for the part
    /// that covers the world.
    #[must_use]
    pub const fn cells(&self) -> Grid {
        self.lattice.whole()
    }

    /// Returns the lattice, with the margin it carries.
    ///
    /// A reader that must turn a cell index into a place in the world goes
    /// through this, because a cell index names a cell of the whole lattice
    /// and the world stands inside it.
    #[must_use]
    pub const fn lattice(&self) -> PaddedLattice {
        self.lattice
    }

    /// Returns the pitch of the lattice, as the tiles along one cell side.
    #[must_use]
    pub const fn scale(&self) -> WeatherScale {
        self.scale
    }

    /// Returns the latitudes that the rows of the world stand at.
    #[must_use]
    pub const fn latitudes(&self) -> Latitudes {
        self.latitudes
    }

    /// Returns the latitude of one cell of the whole lattice.
    ///
    /// **The argument is a cell of the whole lattice, and the answer is the
    /// latitude of the world row under it.** A margin cell beyond a pole
    /// carries the latitude of the pole, in the same way that it carries the
    /// temperature term of the pole.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    #[must_use]
    pub fn latitude_at(&self, cell: u32) -> i32 {
        self.latitudes.of_row(
            self.lattice.inner_row_of(cell),
            self.lattice.inner().height(),
        )
    }

    /// Returns the pressure that the banded circulation adds at one cell of
    /// the whole lattice.
    ///
    /// **This never reaches the temperature.** It reaches the wind and
    /// nothing else.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D2. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    #[must_use]
    pub fn band_at(&self, cell: u32) -> i32 {
        let Some(address) = self.cells().address_of(TileIdx(cell)) else {
            return 0;
        };
        self.band
            .get(address.r.max(0) as usize)
            .copied()
            .unwrap_or(0)
    }

    /// Returns the transport passes that one solve runs on this field.
    #[must_use]
    pub const fn transport_passes(&self) -> u32 {
        self.scale.transport_passes()
    }

    /// Returns the transport passes that have run since the field was built.
    #[must_use]
    pub const fn passes(&self) -> u64 {
        self.passes
    }

    /// Returns the wind passes that have run since the field was built.
    #[must_use]
    pub const fn wind_passes(&self) -> u64 {
        self.wind_passes
    }

    /// Returns the wind over one cell.
    ///
    /// Returns [`Wind::STILL`] when the cell is outside the lattice.
    #[must_use]
    pub fn wind_at(&self, cell: u32) -> Wind {
        self.wind.get(cell as usize).copied().unwrap_or(Wind::STILL)
    }

    /// Returns every entry of the wind plane, in cell index order.
    #[must_use]
    pub fn wind_plane(&self) -> &[Wind] {
        &self.wind
    }

    /// Returns the largest speed any cell of the lattice carries.
    #[must_use]
    pub fn fastest(&self) -> i32 {
        self.wind.iter().map(|wind| wind.speed()).max().unwrap_or(0)
    }

    /// Returns the temperature of one cell, from zero to the heat ceiling.
    ///
    /// Returns zero when the cell is outside the lattice. The temperature is
    /// stored state, so it answers what the last solve left rather than what
    /// the terrain implies.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    #[must_use]
    pub fn warmth_at(&self, cell: u32) -> i32 {
        self.warmth.get(cell as usize).copied().unwrap_or(0)
    }

    /// Returns every entry of the temperature plane, in cell index order.
    #[must_use]
    pub fn warmth_plane(&self) -> &[i32] {
        &self.warmth
    }

    /// Returns the freezing that each cell of the whole lattice has taken and
    /// that its ice has not yet paid back.
    ///
    /// A cell that has never stood below the melting point reads zero, and
    /// the clamp does nothing to such a cell.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D6. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
    #[must_use]
    pub fn frozen_plane(&self) -> &[i32] {
        &self.frozen
    }

    /// Returns the freeze bank of one cell of the whole lattice.
    #[must_use]
    pub fn frozen_at(&self, cell: u32) -> i32 {
        self.frozen.get(cell as usize).copied().unwrap_or(0)
    }

    /// Returns the temperature passes that have run since the field was built.
    #[must_use]
    pub const fn warmth_passes(&self) -> u64 {
        self.warmth_passes
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

    /// Returns the largest quantity of water the air above any cell holds.
    ///
    /// It is the capacity of a cell at the top of the temperature scale. No
    /// cell of any temperature stands above it, so a reader that needs one
    /// figure for the whole plane takes this one. **A reader that paints
    /// cloud must not take it**, because a cold cell fills its own small sky
    /// at a small part of this, and a ramp against this figure paints that
    /// sky black. Take the cloud share instead.
    ///
    /// The reader exists so that the ceiling has one declaration site.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn air_ceiling(&self) -> i64 {
        AIR_SATURATION.0
    }

    /// Returns the share of the sky over one cell that a watcher sees as
    /// cloud, from none to [`CLOUD_SHARE_WHOLE`].
    ///
    /// **Cloud is the air held against what the air of that cell can hold**,
    /// and not the air held against a mark that every cell shares. Every
    /// reader that paints cloud takes this one, so the rule has one
    /// declaration site.[^1]
    ///
    /// Returns none when the cell lies outside the lattice, and when the
    /// field holds no water at all.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn cloud_share_at(&self, cell: u32) -> i64 {
        let held = self.air_at(cell).0;
        let capacity = self.capacity_at_cell(cell).0.max(1);
        (held * CLOUD_SHARE_WHOLE / capacity).clamp(0, CLOUD_SHARE_WHOLE)
    }

    /// Returns the water that the air above one cell holds when it is full.
    ///
    /// The answer follows the temperature of that cell. Warm air holds a lot
    /// and cold air holds very little, so a polar cell reports a small
    /// figure. It is the figure a watcher needs beside the air of a cell,
    /// because the air alone does not say whether the sky is grey.
    ///
    /// The figure is the capacity of a cell that the air has stood over. Air
    /// that cooled or climbed on its way holds less than this, and the settle
    /// pass rains that difference out.
    #[must_use]
    pub fn capacity_at_cell(&self, cell: u32) -> Drops {
        capacity_at(self.warmth.get(cell as usize).copied().unwrap_or(0))
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

    /// Returns the water in the air over the whole lattice.
    ///
    /// **The total holds the margin as well as the world.** The margin lifts
    /// water and holds water, and the account that says the field moves water
    /// and never scales it is an account over everything the field steps. A
    /// total that dropped the margin would not balance against the water the
    /// field raised.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    #[must_use]
    pub fn air_total(&self) -> Accum {
        total_of(&self.air)
    }

    /// Returns the water on the ground over the whole lattice.
    ///
    /// **The total holds the margin as well as the world**, for the reason
    /// the air total states.
    #[must_use]
    pub fn ground_total(&self) -> Accum {
        total_of(&self.ground)
    }

    /// Returns the number of cells of the world whose ground is wet.
    ///
    /// **The count covers the world and never the margin.** It is a census
    /// reading that a watcher compares against the cell count of the world,
    /// and a count that held the margin would stand above that.[^1]
    ///
    /// # References
    ///
    /// [^1]: The padded lattice. [`PaddedLattice`]
    #[must_use]
    pub fn wet_cells(&self) -> u32 {
        self.lattice
            .inner_cells()
            .into_iter()
            .filter(|cell| self.ground_at(*cell).0 >= WET_MARK.0)
            .count() as u32
    }

    /// Returns the water in the air over each cell of the world, in the cell
    /// index order of the world.
    ///
    /// **This crops the margin away.** A watcher indexes the result by the
    /// cell columns of the world, so the margin must not stand in it. Read
    /// [`WeatherField::air_plane`] for the whole plane that the solve steps.
    ///
    /// The result is empty when no water has entered the world.
    #[must_use]
    pub fn air_over_world(&self) -> Vec<Drops> {
        self.crop(&self.air, Drops::ZERO)
    }

    /// Returns the water on the ground of each cell of the world, in the cell
    /// index order of the world.
    ///
    /// **This crops the margin away**, in the same way the air reader does.
    ///
    /// The result is empty when no water has entered the world.
    #[must_use]
    pub fn ground_over_world(&self) -> Vec<Drops> {
        self.crop(&self.ground, Drops::ZERO)
    }

    /// Returns the temperature of each cell of the world, in the cell index
    /// order of the world.
    ///
    /// **This crops the margin away.** A drawing that took the coldest and
    /// the warmest entry of the whole plane could take either from a cell
    /// that no watcher can point at, and the colour scale of the map would
    /// then answer to ground outside the frame.
    #[must_use]
    pub fn warmth_over_world(&self) -> Vec<i32> {
        self.crop(&self.warmth, 0)
    }

    /// Returns the entries of one plane that cover the world, in the cell
    /// index order of the world.
    ///
    /// An empty plane crops to an empty result, because a field that holds no
    /// water allocates no plane.
    fn crop<T: Copy>(&self, plane: &[T], absent: T) -> Vec<T> {
        if plane.is_empty() {
            return Vec::new();
        }
        self.lattice
            .inner_cells()
            .into_iter()
            .map(|cell| plane.get(cell as usize).copied().unwrap_or(absent))
            .collect()
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
        ground: &[CellGround],
        threads: usize,
    ) -> Result<(), WeatherError> {
        if threads == 0 {
            return Err(WeatherError::ZeroThreads);
        }
        if ground.len() != self.cells().tile_count() as usize {
            return Err(WeatherError::LatticeMismatch);
        }
        // The storms move, weaken and die before anything reads them, and the
        // field then tries to raise one. The deficit plane is stamped last,
        // so the wind pass and the settle pass below both read the storms of
        // this frame rather than the storms of the last one.[^5]
        //
        // [^5]: A cyclone. [`Cyclone`]
        self.drift_cyclones(tick, seed, ground);
        self.raise_storms(tick, seed, ground);
        self.stamp_cyclones();
        // The temperature of every cell moves before anything reads it. Four
        // readers follow: the pressure that drives the wind, the lift, the
        // fall, and the carry itself.[^1] [^3]
        //
        // **The temperature is stored, so it carries its own history.** The
        // ground and the season drive it, and the wind then carries it, so a
        // warm parcel outlives the cell it left.[^3]
        //
        // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
        // [^3]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decisions D1, D2 and D3. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
        for _ in 0..WARMTH_PASSES_FOR_EACH_SOLVE {
            self.warm(tick, ground);
            self.carry(threads);
            // **Ground that carries ice cannot rise above the melting point
            // until the ice is gone.** The pass runs after the driver and the
            // carry, because both of them move the temperature.[^4]
            //
            // [^4]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decisions D2 and D3. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
            self.melt();
            self.warmth_passes = self.warmth_passes.saturating_add(1);
        }
        // **The wind blows over dry ground too.** The wind passes run before
        // the branch below, so a field holding no water still carries a wind,
        // and the first storm raised into it rides the wind that is already
        // there rather than one that starts at rest.[^2]
        //
        // [^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
        for _ in 0..WIND_PASSES_FOR_EACH_SOLVE {
            self.blow(threads);
            self.wind_passes = self.wind_passes.saturating_add(1);
        }
        self.lift(tick, seed, ground);
        if self.air.is_empty() {
            return Ok(());
        }
        // **The pass count follows the resolution.** A pass carries water one
        // cell, so a finer lattice needs more passes to carry water the same
        // distance in tiles. The count is fixed for a given scale and it
        // holds no convergence test.[^4]
        //
        // [^4]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
        for _ in 0..self.scale.transport_passes() {
            self.transport(threads);
            self.passes = self.passes.saturating_add(1);
        }
        self.settle(ground);
        Ok(())
    }

    /// Returns the storms that the field is carrying.
    #[must_use]
    pub fn cyclones(&self) -> &[Cyclone] {
        &self.cyclones
    }

    /// Returns the storms that the field has raised over its whole life.
    ///
    /// The count is the identity that the next storm takes, and no identity
    /// is reused, so it counts every storm that ever stood.
    #[must_use]
    pub const fn cyclones_raised(&self) -> u32 {
        self.next_cyclone
    }

    /// Returns the pressure deficit that the storms put on one cell.
    ///
    /// The answer is zero for a cell that no storm reaches, and zero for
    /// every cell while no storm stands.
    #[must_use]
    pub fn depression_at(&self, cell: u32) -> i32 {
        self.depression.get(cell as usize).copied().unwrap_or(0)
    }

    /// Raises a storm over one cell of the whole lattice.
    ///
    /// **This is an authoring verb and not a faction power.** It names no
    /// congregation, so the gate that holds the divine power does not govern
    /// it: a caller that places a storm is the author of the world rather
    /// than a god inside it.[^1] The power that a faction wields is the one
    /// that puts water over ground it holds.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the cell lies outside the lattice, when the
    /// setting lies outside the range the field carries, and when the field
    /// already carries as many storms as it holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    pub fn raise_cyclone(
        &mut self,
        cell: u32,
        setting: CycloneSetting,
    ) -> Result<Cyclone, WeatherError> {
        let Some(address) = self.cells().address_of(TileIdx(cell)) else {
            return Err(WeatherError::NoSuchCell(cell));
        };
        if !setting.is_in_range() {
            return Err(WeatherError::CycloneSettingOutOfRange);
        }
        if self.cyclones.len() >= CYCLONE_CEILING {
            return Err(WeatherError::TooManyCyclones);
        }
        Ok(self.place_cyclone(address, setting))
    }

    /// Puts one storm on the field, at the middle of a cell.
    ///
    /// The identity comes from the counter and never from the slot, so a
    /// storm born into the slot of a dead one draws its own track.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing rules, section 2. `.agents/rules/testing.md`
    fn place_cyclone(&mut self, address: Axial, setting: CycloneSetting) -> Cyclone {
        let middle = CYCLONE_FINE / 2;
        let storm = Cyclone {
            q_fine: address.q * CYCLONE_FINE + middle,
            r_fine: address.r * CYCLONE_FINE + middle,
            depth: setting.depth,
            radius: setting.radius,
            life: setting.life,
            age: 0,
            id: self.next_cyclone,
        };
        self.next_cyclone = self.next_cyclone.saturating_add(1);
        self.cyclones.push(storm);
        storm
    }

    /// Moves every storm, weakens it, and drops the ones that are over.
    ///
    /// **A storm travels on the flow around it and not on its own wind.** The
    /// wind at the eye is the storm's own circulation, so a storm steered by
    /// it would chase its own tail. The steering flow is the mean wind over
    /// the six cells one step beyond the radius, and a symmetric circulation
    /// sums to nothing over that ring, so what is left of the sum is the flow
    /// that the storm sits in.
    ///
    /// **Nothing here sustains a storm.** The depth falls on every solve, and
    /// it falls faster over land and over a cold sea. So a storm always ends,
    /// and the three ways it ends are a comparison rather than a convergence
    /// test.[^1]
    ///
    /// The pass walks the storms in ascending slot order, which is the order
    /// they were raised in.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn drift_cyclones(&mut self, tick: Tick, seed: u64, ground: &[CellGround]) {
        if self.cyclones.is_empty() {
            return;
        }
        let cells = self.cells();
        let mut carried: Vec<Cyclone> = Vec::with_capacity(self.cyclones.len());
        for storm in &self.cyclones {
            let mut storm = *storm;
            let steer = self.steering_at(storm);
            let (wander_q, wander_r) = cyclone_wander(seed, tick, storm.id);
            storm.q_fine += narrow(sim_math::share(
                Accum(i64::from(steer.q)),
                Accum(CYCLONE_STEER_NUMERATOR),
                Accum(CYCLONE_STEER_DENOMINATOR),
            )) + wander_q;
            storm.r_fine += narrow(sim_math::share(
                Accum(i64::from(steer.r)),
                Accum(CYCLONE_STEER_NUMERATOR),
                Accum(CYCLONE_STEER_DENOMINATOR),
            )) + wander_r;
            storm.age = storm.age.saturating_add(1);
            storm.depth -= CYCLONE_DECAY;

            // A storm that walked off the lattice is gone. The margin is what
            // gives it somewhere to go, and a storm that crossed the margin
            // has left the world for good.
            let Some(at) = cells.index_of(storm.eye()) else {
                continue;
            };
            let index = at.0 as usize;
            if let Some(under) = ground.get(index) {
                let open = under.open_tiles() * CYCLONE_LAND_WHOLE;
                if open > under.tiles() * CYCLONE_LAND_MARK {
                    storm.depth -= CYCLONE_LANDFALL_DECAY;
                }
            }
            let heat = self.warmth.get(index).copied().unwrap_or(0);
            if heat < CYCLONE_WARM_MARK {
                storm.depth -= CYCLONE_COLD_DECAY;
            }
            if storm.is_over() {
                continue;
            }
            carried.push(storm);
        }
        self.cyclones = carried;
    }

    /// Returns the flow that one storm travels on.
    ///
    /// The answer is the mean wind over the six cells one step beyond the
    /// radius of the storm, in the six directions. A cell outside the lattice
    /// is left out of the mean, and a storm that finds no such cell reads the
    /// wind under its own eye.
    fn steering_at(&self, storm: Cyclone) -> Wind {
        let cells = self.cells();
        let eye = storm.eye();
        let reach = storm.radius + 1;
        let mut sum_q = 0i64;
        let mut sum_r = 0i64;
        let mut found = 0i64;
        for step in NEIGHBOURS {
            let around = Axial::new(eye.q + step.q * reach, eye.r + step.r * reach);
            let Some(at) = cells.index_of(around) else {
                continue;
            };
            let wind = self.wind.get(at.0 as usize).copied().unwrap_or(Wind::STILL);
            sum_q += i64::from(wind.q);
            sum_r += i64::from(wind.r);
            found += 1;
        }
        if found == 0 {
            return cells
                .index_of(eye)
                .and_then(|at| self.wind.get(at.0 as usize).copied())
                .unwrap_or(Wind::STILL);
        }
        Wind {
            q: narrow(sim_math::share(Accum(sum_q), Accum(1), Accum(found))),
            r: narrow(sim_math::share(Accum(sum_r), Accum(1), Accum(found))),
        }
    }

    /// Tries to raise storms, over the warm sea and on the fronts.
    ///
    /// **Nothing emerges here, and nothing can.** A single-layer field grows
    /// no baroclinic eddies, so a low cannot form out of it. This pass places
    /// each low, and the gates below only decide where a placed one is
    /// plausible.[^1]
    ///
    /// **The pass imposes the eddies for the same reason the field imposes
    /// the belts.** A latitude-only forcing gives a field that is constant
    /// along every row, and the only term of this model that varies along a
    /// row and moves is a storm. So a field with too few storms holds three
    /// belts and nothing else, whatever else it does.[^5]
    ///
    /// Two gates admit a storm, and each states a published condition that
    /// this field can read.
    ///
    /// The first gate is a tropical cyclone: a sea rather than land, a warm
    /// sea, and air that already holds a share of what it can carry. **The
    /// latitude gate is not modelled.** A real cyclone does not form within
    /// about five degrees of the equator, because the deflection vanishes
    /// there; the deflection of this field is the same at every latitude, so
    /// there is nothing for that gate to read.[^2]
    ///
    /// The second gate is a storm on a front: a temperature gradient across
    /// the cell above the mark that the lattice sets. This is where the
    /// travelling weather of the middle latitudes lives, and the first gate
    /// admits none of it, because the middle latitudes are neither warm
    /// enough nor all sea.[^4] [^5]
    ///
    /// The pass makes a fixed count of attempts and each takes two keyed
    /// draws, whatever the field holds. So it costs the same on every frame
    /// and it takes no branch that a thread could change.[^3]
    ///
    /// The attempts run in ascending order and each reads the storms that the
    /// ones before it placed, so the order is fixed.[^6]
    ///
    /// # References
    ///
    /// [^1]: The banded circulation. [`band_pressure_at`]
    /// [^2]: The deflection. [`deflect`]
    /// [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^4]: The front mark. [`front_mark`]
    /// [^5]: Research report 30, the published atmospheric math, section 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
    /// [^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn raise_storms(&mut self, tick: Tick, seed: u64, ground: &[CellGround]) {
        let cells = self.cells();
        let reach = storm_reach(self.latitudes, self.lattice.inner().height());
        let mark = front_mark(self.latitudes, self.lattice.inner().height());
        for attempt in 0..CYCLONE_GENESIS_ATTEMPTS {
            let cell = cyclone_genesis_cell(seed, tick, attempt, cells.tile_count());
            if !cyclone_forms(seed, tick, attempt) {
                continue;
            }
            if self.cyclones.len() >= CYCLONE_CEILING {
                continue;
            }
            let Some(setting) = self.genesis_at(cell, ground, reach, mark) else {
                continue;
            };
            let Some(address) = cells.address_of(TileIdx(cell)) else {
                continue;
            };
            self.place_cyclone(address, setting);
        }
    }

    /// Returns the storm that one cell admits, if it admits one.
    ///
    /// The first gate is a warm sea holding wet air, and the second is a
    /// temperature gradient across the cell. A cell that passes neither
    /// carries no storm.
    fn genesis_at(
        &self,
        cell: u32,
        ground: &[CellGround],
        reach: i32,
        mark: i32,
    ) -> Option<CycloneSetting> {
        let index = cell as usize;
        let under = *ground.get(index)?;
        if under.tiles() <= 0 {
            return None;
        }
        let heat = self.warmth.get(index).copied().unwrap_or(0);
        // A sea cell, and not a coast. Water is the only ground that admits
        // no unit.
        let is_sea = under.open_tiles() * CYCLONE_LAND_WHOLE <= under.tiles();
        // The air must already hold a share of what it can carry. A dry sky
        // has nothing for a storm to rain out, and a storm over one would be
        // a low with no weather under it.
        let air = self.air.get(index).copied().unwrap_or(Drops::ZERO);
        let asked = sim_math::share(
            Accum(capacity_at(heat).0),
            Accum(CYCLONE_GENESIS_AIR_NUMERATOR),
            Accum(CYCLONE_GENESIS_AIR_DENOMINATOR),
        )
        .map_or(0, |value| value.0);
        if is_sea && heat >= CYCLONE_WARM_MARK && air.0 >= asked {
            return Some(CycloneSetting::TROPICAL.with_reach(reach));
        }
        if self.front_at(cell) >= mark {
            return Some(CycloneSetting::FRONTAL.with_reach(reach));
        }
        None
    }

    /// Returns the largest temperature difference between one cell and its
    /// neighbours.
    ///
    /// **This is the baroclinicity that the field can read.** A neighbour
    /// outside the lattice is left out, so a cell at the edge reads the
    /// neighbours it has. The walk is in direction order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn front_at(&self, cell: u32) -> i32 {
        let cells = self.cells();
        let Some(address) = cells.address_of(TileIdx(cell)) else {
            return 0;
        };
        let here = self.warmth.get(cell as usize).copied().unwrap_or(0);
        let mut widest = 0;
        for direction in 0..NEIGHBOUR_COUNT {
            let Some(beside) = cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = cells.index_of(beside) else {
                continue;
            };
            let there = self.warmth.get(at.0 as usize).copied().unwrap_or(0);
            widest = widest.max((there - here).abs());
        }
        widest
    }

    /// Rebuilds the deficit plane from the storms.
    ///
    /// **The plane is derived, and this pass derives the whole of it.** No
    /// part of the previous frame survives, so a storm that ended leaves
    /// nothing behind and the ambient field returns to what it was.
    ///
    /// The pass walks the storms in slot order and the cells of each
    /// footprint in ascending row and column order. Both orders are
    /// fixed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn stamp_cyclones(&mut self) {
        if self.cyclones.is_empty() {
            self.depression.clear();
            return;
        }
        let cells = self.cells();
        let count = cells.tile_count() as usize;
        if self.depression.len() == count {
            self.depression.fill(0);
        } else {
            self.depression = vec![0; count];
        }
        for storm in &self.cyclones {
            let eye = storm.eye();
            let reach = storm.radius;
            for r in (eye.r - reach)..=(eye.r + reach) {
                for q in (eye.q - reach)..=(eye.q + reach) {
                    let address = Axial::new(q, r);
                    let Some(at) = cells.index_of(address) else {
                        continue;
                    };
                    let deficit = storm.deficit_at(address);
                    if deficit <= 0 {
                        continue;
                    }
                    let slot = &mut self.depression[at.0 as usize];
                    *slot = slot.saturating_add(deficit).clamp(0, CYCLONE_DEPTH_CEILING);
                }
            }
        }
    }

    /// Moves the temperature of every cell toward what the world asks.
    ///
    /// **The pass never assigns the asked temperature.** It moves the stored
    /// one a fixed share of the way, and the share is what makes the field
    /// lag its driver. It also adds one whole degree of the remaining
    /// difference, because the share alone truncates to nothing once the two
    /// are close and the field would then stop short for ever.[^1]
    ///
    /// The pass walks the cells in ascending index order and reads nothing it
    /// is writing.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn warm(&mut self, tick: Tick, ground: &[CellGround]) {
        // **The latitude band belongs to the world, not to the whole
        // lattice.** The row and the height below are the row and the height
        // of the inner lattice, so the poles of the world stay at the first
        // and the last row of the world however wide the margin is. A margin
        // cell beyond a pole reads the pole row.[^3]
        //
        // [^3]: The padded lattice. [`PaddedLattice::inner_row_of`]
        let height = self.lattice.inner().height();
        let latitudes = self.latitudes;
        // **The zero of the relief term, read once for the pass.** It is a
        // reduction over ground that never changes, so it is the same integer
        // on every tick.
        let reference = mean_land_height_over(ground);
        for (cell, under) in ground.iter().enumerate() {
            let row = self.lattice.inner_row_of(cell as u32);
            let air = self.air.get(cell).copied().unwrap_or(Drops::ZERO);
            // **The cloud reads the air against the capacity of this cell**,
            // and the capacity comes from the temperature the cell carried
            // into the pass. So a cold cell holding a little water is
            // overcast, and a warm cell holding the same water is clear.
            let capacity = capacity_at(self.warmth.get(cell).copied().unwrap_or(0));
            let asked = asked_warmth(
                relief_cooling_of(*under, self.relief, reference),
                season_at(tick, latitudes.of_row(row, height)),
                cloud_at(air, capacity),
                self.warmth.get(cell).copied().unwrap_or(0),
            );
            let Some(held) = self.warmth.get_mut(cell) else {
                continue;
            };
            let apart = i64::from(asked - *held);
            // **Deep water holds its temperature, and land does not.** The
            // divisor grows with the depth of the water the cell holds, so a
            // deep sea moves a small part of the way toward what the world
            // asks while the coast beside it moves the whole of it. The two
            // then part as the season turns, and the coastal wind reverses
            // with them.[^3]
            let step = narrow(sim_math::share(
                Accum(apart),
                Accum(WARMTH_NUMERATOR),
                Accum(WARMTH_DENOMINATOR * lag_of(*under)),
            ));
            // The whole degree that takes the last of the difference. It
            // never overshoots, because it is bounded by what is left.
            let step = (i64::from(step) + apart.signum()).clamp(-apart.abs(), apart.abs());
            *held = narrow(Some(Accum(i64::from(*held) + step))).clamp(0, HEAT_CEILING);
        }
    }

    /// Holds every cell that carries ice at the melting point.
    ///
    /// **The pass runs after the driver and after the carry**, so it reads the
    /// temperature that the whole solve settled on. A clamp inside one of the
    /// two passes lets the other one lift a cell over the melting point with
    /// ice still standing.[^1] [^2]
    ///
    /// The pass walks the cells in ascending index order and reads nothing it
    /// is writing.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decisions D2 and D3. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
    /// [^2]: Findings register, FND-630. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn melt(&mut self) {
        for (cell, bank) in self.frozen.iter_mut().enumerate() {
            let Some(held) = self.warmth.get_mut(cell) else {
                continue;
            };
            let (settled, banked) = frozen_hold(*held, i64::from(*bank));
            *held = settled;
            *bank = banked as i32;
        }
    }

    /// Runs one temperature carry pass over the lattice.
    ///
    /// **The wind carries the temperature.** A cell takes, from each
    /// neighbour, a share of the difference between the two temperatures, and
    /// the share rises with the part of that neighbour's wind that points
    /// toward the cell. So a wind off warm water warms the ground it reaches,
    /// and a wind off a cold ridge cools it.[^1]
    ///
    /// **The six shares sum to less than one whole**, so the answer lies
    /// inside the range of the temperatures the pass read and the field
    /// cannot run away.[^1]
    ///
    /// The temperature is not water and this pass does not conserve it. Two
    /// cells at one temperature give one temperature and not two, so the
    /// exact-move rule of the transport does not apply here.[^1]
    ///
    /// The pass is a gather. It writes only the cell it is computing, so a
    /// parallel pass writes disjoint output and needs no atomic
    /// operation.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D3. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn carry(&mut self, threads: usize) {
        let count = self.warmth.len();
        if count == 0 {
            return;
        }
        let pass = WarmthPass {
            cells: self.cells(),
            warmth: &self.warmth,
            wind: &self.wind,
        };
        run_in_chunks(count, threads, &mut self.warmth_scratch, |low, out| {
            pass.fill(low, out);
        });
        self.warmth.copy_from_slice(&self.warmth_scratch);
    }

    /// Runs one wind pass over the lattice.
    ///
    /// **The pressure difference accelerates the wind. It never assigns it.**
    /// A cell takes a bounded step toward what the pressure across it asks
    /// for, and the step is what makes the wind lag the pressure. Drag bleeds
    /// the momentum before the step is added, and the speed ceiling bounds
    /// what is left.[^1] [^2]
    ///
    /// Hot air rises and the pressure falls where it does, so the wind
    /// accelerates toward the hotter neighbour. The acceleration is the sum,
    /// over the six directions, of the direction times the heat that
    /// neighbour holds above this cell.
    ///
    /// The pass is a gather. It reads the settled wind plane and the heat
    /// buffer, and it writes only the cell it is computing, so a parallel
    /// pass writes disjoint output and needs no atomic operation.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    /// [^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    /// [^3]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn blow(&mut self, threads: usize) {
        let count = self.wind.len();
        if count == 0 {
            return;
        }
        let pass = WindPass {
            cells: self.cells(),
            wind: &self.wind,
            warmth: &self.warmth,
            band: &self.band,
            depression: &self.depression,
            pressure_divisor: self.scale.pressure_divisor(),
        };
        run_in_chunks(count, threads, &mut self.wind_scratch, |low, out| {
            pass.fill(low, out);
        });
        self.wind.copy_from_slice(&self.wind_scratch);
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
    fn lift(&mut self, tick: Tick, seed: u64, ground: &[CellGround]) {
        let mut raised = 0i64;
        for (cell, under) in ground.iter().enumerate() {
            // The key is the one the lattice states, not the raw index. A
            // cell of the world carries the same key at every margin width,
            // so widening the margin does not redraw the world.[^2]
            //
            // [^2]: The padded lattice. [`PaddedLattice::draw_key`]
            let key = self.lattice.draw_key(cell as u32);
            if !cell_lifts(seed, tick, key, under.tiles(), under.open_tiles()) {
                continue;
            }
            let heat = self.warmth.get(cell).copied().unwrap_or(0);
            self.prepare();
            // **A sea evaporates into the room its own sky has.** The room is
            // the capacity of this cell less the water already standing over
            // it, and the capacity follows the temperature. So a hot sea
            // still gives up more than a cold one, and the temperature
            // reaches the lift through the capacity rather than on its
            // own.[^1]
            //
            // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            //
            // **The room is also the back pressure on the source.** Without
            // it nothing bounds what enters the air, so the source overwhelms
            // every sink and the total only climbs. A full cell lifts
            // nothing. The transport carries the water away and the cell then
            // lifts again, so a coast is a steady supply rather than a
            // runaway one.
            //
            // The old rule raised a fixed quantity of drops in proportion to
            // the heat. A polar sea then raised almost nothing, so no water
            // was ever created at a high latitude.
            let room = capacity_at(heat).0 - self.air[cell].0;
            if room <= 0 {
                continue;
            }
            let lifted = sim_math::share(
                Accum(room),
                Accum(LIFT_OF_ROOM_NUMERATOR),
                Accum(LIFT_OF_ROOM_DENOMINATOR),
            )
            .map_or(0, |value| value.0)
            .min(room);
            if lifted <= 0 {
                continue;
            }
            self.air[cell] = self.air[cell].combine(Drops(lifted));
            raised = raised.saturating_add(lifted);
        }
        self.raised = self.raised.saturating_add(raised);
    }

    /// Allocates the planes, if they are not allocated already.
    fn prepare(&mut self) {
        if !self.air.is_empty() {
            return;
        }
        let count = self.cells().tile_count() as usize;
        self.air = vec![Drops::ZERO; count];
        self.ground = vec![Drops::ZERO; count];
        self.scratch = vec![Drops::ZERO; count];
    }

    /// Runs one transport pass over the air plane.
    ///
    /// **Water rides the wind.** A cell computes, for each neighbour, the
    /// quantity that neighbour sends it along that neighbour's wind, and it
    /// adds those to what it kept. It writes only itself, so no two threads
    /// write one cell and the pass needs no atomic operation.[^1] [^2]
    ///
    /// **The pass conserves water exactly.** The receiver computes the
    /// giver's share from the same input planes with the same truncating
    /// division that the giver used, so the two ends of an edge reach one
    /// integer. The remainder stays with the giver, and several cells sending
    /// into one is a sum of integers that each pair already agreed on.[^2]
    ///
    /// A pass carries water at most one cell, so the reach of a solve is the
    /// pass count rather than the arithmetic.[^2]
    ///
    /// The cells are visited in ascending index and the neighbours in
    /// direction order. Both orders are fixed.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    /// [^2]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decisions D1 and D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn transport(&mut self, threads: usize) {
        let count = self.air.len();
        if count == 0 {
            return;
        }
        let pass = Pass {
            cells: self.cells(),
            air: &self.air,
            wind: &self.wind,
        };
        run_in_chunks(count, threads, &mut self.scratch, |low, out| {
            pass.fill(low, out);
        });
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
    fn settle(&mut self, ground: &[CellGround]) {
        let mut dried = 0i64;
        for cell in 0..ground.len() {
            let heat = self.warmth.get(cell).copied().unwrap_or(0);

            // **The air of a cell holds what its own temperature allows, and
            // it rains out what it cannot hold.** Warm air holds a lot and
            // cold air holds very little, so a parcel that travels toward a
            // pole loses capacity as it goes and turns into cloud on the way
            // rather than being destroyed at a boundary.[^3]
            //
            // Two other terms shed capacity beside the temperature. The
            // cooling is the heat of the cell the air came from above the
            // heat of this one, and the cell it came from is the neighbour
            // that the wind of this cell points away from. The climb is the
            // slope the air went up, and it is signed, so air that descends a
            // slope warms and holds what it carries. That pair is what makes
            // the near side of a ridge wet and the far side dry.[^1] [^5]
            //
            // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            // [^5]: ADR-0162, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            //
            // **The capacity is a bound, never an assignment.** The pass
            // computes the water above the capacity and moves that quantity
            // from the air plane to the ground plane. It is an exact integer
            // move, so what the air loses the ground gains, and the account
            // holds whether the cell warmed or cooled since the last
            // solve.[^4] Nothing here scales the air toward the capacity, and
            // nothing sets the air to it.
            //
            // Without a bound nothing limited what the transport delivers
            // into a cell that several winds converge on. Such a cell climbed
            // without a bound, and it held three orders of magnitude above
            // the median of the plane.[^3] **This is also what ends a
            // storm.** The convergence that builds one is the same thing that
            // empties it.
            //
            // [^3]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D3. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            // [^4]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
            let cooling = self.cooling_at(cell, heat);
            let climb = self.climb_at(cell, ground);
            // **A storm is forced ascent, and ascent rains.** The deficit
            // takes a further share of the capacity, so the air under an eye
            // pours out most of what it holds. The move is a bound and never
            // an assignment, so the water account holds through it.[^7]
            //
            // [^7]: A cyclone. [`cyclone_capacity`]
            let capacity = cyclone_capacity(
                travelling_capacity(capacity_at(heat), cooling, climb),
                self.depression.get(cell).copied().unwrap_or(0),
            );
            let held = self.air[cell];
            let poured = Drops((held.0 - capacity.0).max(0));
            self.air[cell] = Drops(held.0 - poured.0);
            self.ground[cell] = self.ground[cell].combine(poured);

            // **What is left drizzles in proportion to how full it is.** A
            // full sky gives up a share each solve and a clear one gives up
            // the floor, which is the trace that keeps the water cycling.
            let air = self.air[cell];
            let numerator = fall_numerator(air, capacity);
            let fallen = share_of(air, numerator, FALL_DENOMINATOR);
            self.air[cell] = Drops(air.0 - fallen.0);
            self.ground[cell] = self.ground[cell].combine(fallen);

            // **Water leaves the ground of a cell at one site, and the room
            // above it decides where that water goes.** Two terms take it off
            // the ground. The heat of the cell lifts a share, so a place is
            // dried by its own sun and not only by time.[^2] Time takes a
            // share and one whole drop more, because the share alone
            // truncates to nothing on ground holding less than the divisor,
            // and water then accumulated everywhere and never left.
            //
            // [^2]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            //
            // **What fits in the room goes back into the air of the same
            // cell, and only what will not fit leaves the world.** The room
            // is the capacity, so it is small over a cold cell and large over
            // a warm one. A cold inland plain therefore keeps its water
            // cycling between its ground and its own small sky, and it stands
            // grey. The old order ran the two terms apart and sent the whole
            // of the second one out of the world, so a cold inland cell
            // drained itself and held no cloud at all.
            //
            // Both moves are exact integer moves. What the ground loses, the
            // air and the evaporated total gain between them, and the two
            // shares are computed from one figure.[^6]
            //
            // [^6]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
            let wet = self.ground[cell];
            let by_heat = share_of(wet, i64::from(heat), GROUND_LIFT_DENOMINATOR).0;
            let by_time = share_of(wet, 1, DRY_DIVISOR).0 + DRY_FLOOR;
            let leaving = (by_heat + by_time).clamp(0, wet.0);
            self.ground[cell] = Drops(wet.0 - leaving);
            let room = (capacity.0 - self.air[cell].0).max(0);
            let returned = leaving.min(room);
            self.air[cell] = self.air[cell].combine(Drops(returned));
            dried = dried.saturating_add(leaving - returned);
        }
        self.evaporated = self.evaporated.saturating_add(dried);
    }

    /// Returns the cooling that the air arriving at one cell met.
    ///
    /// The air came from the neighbour that the wind points away from. The
    /// cooling is the heat of that neighbour above the heat of this cell, and
    /// it is never negative: air that warms on the way rains no less than air
    /// that held its temperature, it simply gains no extra term.
    ///
    /// A still cell met no cooling, and neither did a cell whose upwind
    /// neighbour lies outside the lattice.
    fn climb_at(&self, cell: usize, ground: &[CellGround]) -> i32 {
        let wind = self.wind.get(cell).copied().unwrap_or(Wind::STILL);
        let Some(heading) = wind.heading() else {
            return 0;
        };
        let Some(address) = self.cells().address_of(TileIdx(cell as u32)) else {
            return 0;
        };
        let Some(upwind) = self.cells().neighbour(address, opposite(heading)) else {
            return 0;
        };
        let Some(at) = self.cells().index_of(upwind) else {
            return 0;
        };
        let here = ground
            .get(cell)
            .and_then(|under| under.mean_height())
            .unwrap_or(Fix32::ZERO);
        let there = ground
            .get(at.0 as usize)
            .and_then(|under| under.mean_height())
            .unwrap_or(Fix32::ZERO);
        // The rise the parcel made, scaled by how hard the wind pushed it up
        // the slope. A slow wind rides over a slope and a fast one is forced
        // up it, so the speed belongs in the term.
        let rise = i64::from(here.0) - i64::from(there.0);
        narrow(sim_math::share(
            Accum(rise),
            Accum(i64::from(wind.speed().clamp(0, SPEED_CEILING))),
            Accum(i64::from(SPEED_CEILING)),
        ))
    }

    fn cooling_at(&self, cell: usize, heat: i32) -> i32 {
        let Some(heading) = self
            .wind
            .get(cell)
            .copied()
            .unwrap_or(Wind::STILL)
            .heading()
        else {
            return 0;
        };
        let Some(address) = self.cells().address_of(TileIdx(cell as u32)) else {
            return 0;
        };
        let Some(upwind) = self.cells().neighbour(address, opposite(heading)) else {
            return 0;
        };
        let Some(at) = self.cells().index_of(upwind) else {
            return 0;
        };
        let there = self.warmth.get(at.0 as usize).copied().unwrap_or(0);
        (there - heat).max(0)
    }

    /// Folds the field into a state hash.
    ///
    /// The order is the two running totals, then the readiness of each
    /// faction, then the air plane, then the ground plane, each in ascending
    /// slot order. The order is fixed and the hash is order-sensitive, so a
    /// reader does not have to prove that the order does not matter.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash
            .write_u64(u64::from(self.faction_count))
            .write_u64(self.passes)
            .write_u64(self.wind_passes)
            .write_u64(self.warmth_passes)
            .write_u64(self.raised as u64)
            .write_u64(self.evaporated as u64);
        for ready in &self.ready {
            running = running.write_u64(ready.0);
        }
        // **The wind is state that the next step reads, so it enters the
        // hash.** A world that loads a saved wind and a world that recomputes
        // one are different worlds.[^2]
        //
        // [^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
        // **The temperature is state that the next step reads, so it enters
        // the hash.** A world that loads a saved temperature and a world that
        // recomputes one are different worlds, in the same way that the wind
        // makes them different.[^3]
        //
        // [^3]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
        // **A storm is state that the next step reads, so it enters the
        // hash.** The identity counter goes in beside it, because two fields
        // holding the same storms and a different counter give the next storm
        // a different draw key.[^4]
        //
        // [^4]: A cyclone. [`Cyclone`]
        running
            .write_u64(u64::from(self.next_cyclone))
            .write_u64(self.cyclones.len() as u64)
            .write(bytemuck::cast_slice(&self.cyclones))
            .write(bytemuck::cast_slice(&self.air))
            .write(bytemuck::cast_slice(&self.ground))
            .write(bytemuck::cast_slice(&self.wind))
            .write(bytemuck::cast_slice(&self.warmth))
            // **The freeze bank is state that the next step reads, so it
            // enters the hash.** Two cells at one temperature and a different
            // bank move differently on the next pass.[^5]
            //
            // [^5]: ADR-0190, ground that carries ice stays at the melting point until the ice is gone, decision D1. `docs/adrs/draft/adr-0190-ground-that-carries-ice-stays-at-the-melting-point-until-the-ice-is-gone.md`
            .write(bytemuck::cast_slice(&self.frozen))
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

/// Runs one gather pass over a plane, in chunks, at a thread count.
///
/// A chunk is a contiguous run of the output plane, and a worker writes only
/// its own run. So no two workers write one entry and the pass needs no
/// atomic operation. The run is named by its position, and nothing in the
/// body reads which worker called it, so the answer does not depend on the
/// thread count or on which chunk finished first.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn run_in_chunks<T, F>(count: usize, threads: usize, out: &mut [T], fill: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync,
{
    // **One thread runs the pass where it stands.** A caller that asked for
    // one thread and got a spawned one paid for the spawn and gained nothing,
    // and the spawn cost more than the pass on a small lattice. The answer is
    // the same either way, because one worker takes one chunk that covers the
    // whole plane.
    if threads <= 1 || count <= threads {
        fill(0, out);
        return;
    }
    let chunk_len = count.div_ceil(threads).max(1);
    let fill = &fill;
    crate::parallel::fan_out_each({
        let mut start = 0usize;
        out.chunks_mut(chunk_len).map(move |chunk| {
            let low = start;
            start += chunk.len();
            move || fill(low, chunk)
        })
    });
}

/// Returns the water that one cell sends one way over one pass.
///
/// The quantity rises with the part of the wind of the giver that points that
/// way. A still cell sends the base share every way, so the old symmetric
/// behaviour is the zero-wind case of this rule and not a second rule.[^1]
///
/// **A giver and a receiver both call this, with the same water, the same
/// wind and the same direction.** So both reach one integer and the edge
/// conserves exactly. The remainder stays with the giver, because the share
/// truncates.[^1]
///
/// # References
///
/// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
fn sent(air: Drops, wind: Wind, direction: usize) -> Drops {
    let along = i64::from(wind.along(NEIGHBOURS[direction]).max(0));
    let numerator = SEND_BASE_NUMERATOR + along * SEND_FOR_EACH_WIND_STEP;
    share_of(air, numerator, SEND_DENOMINATOR)
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

impl Pass<'_> {
    /// Fills one run of the scratch plane.
    ///
    /// The run is named by its position in the plane. Nothing in the body
    /// reads which thread called it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn fill(&self, start: usize, out: &mut [Drops]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let here = self.air[index];
            let blowing = self.wind[index];

            let mut kept = here.0;
            let mut taken = Drops::ZERO;
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let at = at.0 as usize;
                // What this cell sends that way, along its own wind.
                kept -= sent(here, blowing, direction).0;
                // What the neighbour sends back this way, along its own wind.
                // The neighbour faces this cell the other way round, so the
                // direction is the opposite one. Both ends compute this from
                // the same settled planes, so they reach one integer.
                taken = taken.combine(sent(self.air[at], self.wind[at], opposite(direction)));
            }
            *cell = Drops(kept).combine(taken);
        }
    }
}

/// What one temperature carry pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct WarmthPass<'a> {
    cells: Grid,
    warmth: &'a [i32],
    wind: &'a [Wind],
}

impl WarmthPass<'_> {
    /// Fills one run of the temperature scratch plane.
    ///
    /// The run is named by its position in the plane. Nothing in the body
    /// reads which thread called it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn fill(&self, start: usize, out: &mut [i32]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let here = self.warmth.get(index).copied().unwrap_or(0);
            *cell = here;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let mut moved = 0i64;
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let at = at.0 as usize;
                // The part of that neighbour's wind that points at this cell.
                // The neighbour faces this cell the other way round, so the
                // direction is the opposite one.
                let along = i64::from(
                    self.wind
                        .get(at)
                        .copied()
                        .unwrap_or(Wind::STILL)
                        .along(NEIGHBOURS[opposite(direction)])
                        .max(0),
                );
                if along == 0 {
                    continue;
                }
                let there = self.warmth.get(at).copied().unwrap_or(0);
                moved += i64::from(narrow(sim_math::share(
                    Accum(i64::from(there - here)),
                    Accum(along * CARRY_FOR_EACH_WIND_STEP),
                    Accum(CARRY_DENOMINATOR),
                )));
            }
            *cell = narrow(Some(Accum(i64::from(here) + moved))).clamp(0, HEAT_CEILING);
        }
    }
}

/// What one wind pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct WindPass<'a> {
    cells: Grid,
    wind: &'a [Wind],
    warmth: &'a [i32],
    /// The pressure that the banded circulation adds at each row of the whole
    /// lattice.
    ///
    /// **The index is a row of the whole lattice.** The pass reads the row
    /// out of the address it already computed, so it needs no second walk and
    /// no plane over the cells.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D2. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    band: &'a [i32],
    /// The pressure deficit that the storms put on each cell.
    ///
    /// **A storm is a low, and the pass reads it as it reads a belt.** The
    /// plane is empty while no storm stands, and the pass then takes zero for
    /// every cell.[^1]
    ///
    /// # References
    ///
    /// [^1]: A cyclone. [`Cyclone`]
    depression: &'a [i32],
    /// What the pressure sum is divided by. It follows the cell side, because
    /// a finer lattice holds a smaller difference between two neighbours.
    pressure_divisor: i64,
}

impl WindPass<'_> {
    /// Returns what the wind of one cell answers to.
    ///
    /// **The pressure of a cell is its temperature less the offset that the
    /// banded circulation puts at its latitude.** Hot air rises and the
    /// pressure falls where it does, so a high temperature and a low band
    /// offset both draw the wind in. The offset is what puts a divergent belt
    /// at thirty degrees and a convergent one at the equator, which no
    /// temperature profile gives on its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D2. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    fn drive(&self, index: usize, address: Axial) -> i32 {
        let warmth = self.warmth.get(index).copied().unwrap_or(0);
        let band = self
            .band
            .get(address.r.max(0) as usize)
            .copied()
            .unwrap_or(0);
        // **A storm is a low, and the pass reads it as it reads a belt.** So
        // the wind answers to the deficit through the arithmetic that is
        // already here, and the deflection then turns the inflow into a
        // circulation. Nothing here writes a rotation.[^2]
        //
        // [^2]: A cyclone. [`Cyclone`]
        let storm = self.depression.get(index).copied().unwrap_or(0);
        warmth - band - storm
    }

    /// Fills one run of the wind scratch plane.
    ///
    /// The order is drag, then the acceleration the pressure asks for, then
    /// the ceiling. Drag runs first, because a pass that accelerated before
    /// bleeding would let a steady pressure difference add the same step for
    /// ever and the field would never settle.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    fn fill(&self, start: usize, out: &mut [Wind]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let index = start + offset;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let here = self.drive(index, address);

            // The acceleration the pressure across this cell asks for. Hot
            // air rises and the pressure falls where it does, so the sum
            // points toward the hotter neighbours.
            let mut asked_q = 0i64;
            let mut asked_r = 0i64;
            for (direction, step) in NEIGHBOURS.iter().enumerate() {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let there = self.drive(at.0 as usize, neighbour);
                let pull = i64::from(there - here);
                asked_q += i64::from(step.q) * pull;
                asked_r += i64::from(step.r) * pull;
            }
            let step = cap(
                Wind {
                    q: narrow(sim_math::share(
                        Accum(asked_q),
                        Accum(1),
                        Accum(self.pressure_divisor),
                    )),
                    r: narrow(sim_math::share(
                        Accum(asked_r),
                        Accum(1),
                        Accum(self.pressure_divisor),
                    )),
                },
                WIND_STEP,
            );
            let carried = drag(self.wind[index]);
            // **The world turns, so the wind is pushed to one side.** The
            // push acts on the wind the cell already carries, which is why a
            // still cell stays still and a fast one turns hard. A flow that
            // runs at a warm cell arrives running round it instead, and a
            // circulation is what that is.
            let side = deflect(carried);
            *cell = cap(
                Wind {
                    q: carried.q + step.q + side.q,
                    r: carried.r + step.r + side.r,
                },
                SPEED_CEILING,
            );
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
