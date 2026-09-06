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
//! **Water enters the air where it is hot.** The sea lifts more from a hot
//! cell than from a cold one, and the heat of a cell also lifts water off its
//! own ground back into its own air.[^12] The sea stops lifting when the air
//! over a cell reaches the saturation mark, and that bound is what keeps the
//! source from overwhelming the sink.
//!
//! **Rain falls where the air cools.** The share of the air that falls rises
//! as the cell gets colder, and rises again with the cooling the air met on
//! its way here. So the near side of a ridge is wet and the far side is
//! dry.[^12]
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
        for direction in 0..NEIGHBOUR_COUNT {
            let along = self.along(NEIGHBOURS[direction]);
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
    (kept - i64::from(kept.signum())) as i32
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

/// The number of spread passes that one solve runs.
///
/// The count is fixed. A solve runs it whatever the field holds and whatever
/// the thread count. It is not a budget and no measurement chose it: it is
/// the reach that one frame of weather adds, in cells.[^1]
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
pub const PASSES_FOR_EACH_SOLVE: u32 = 4;

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

/// The denominator of every share of the air that a transport pass sends.
const SEND_DENOMINATOR: i64 = 64;

/// The numerator a cell sends to each neighbour whatever its wind.
///
/// A cell whose wind is still sends this to every neighbour, so the old
/// symmetric behaviour is the zero-wind case of the directional rule rather
/// than a second rule.[^1]
///
/// # References
///
/// [^1]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
const SEND_BASE_NUMERATOR: i64 = 1;

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
pub const SPEED_CEILING: i32 = 6;

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

/// What open water adds to the heat of a cell, at the top of its range.
const HEAT_FROM_WATER: i32 = 128;

/// What low ground adds to the heat of a cell, at the bottom of the height
/// range.
const HEAT_FROM_LOW_GROUND: i32 = 128;

/// The heat difference that one step of wind acceleration answers to.
///
/// The acceleration of a cell is the sum, over the six directions, of the
/// direction times the heat the neighbour holds above this cell. That sum is
/// divided by this before the step ceiling bounds it.
const PRESSURE_DIVISOR: i64 = 32;

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
const WIND_STEP: i32 = 2;

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

/// The denominator of the share of the air that falls in one solve.
const FALL_DENOMINATOR: i64 = 64;

/// The numerator of the share that falls whatever the air met.
const FALL_NUMERATOR_FLOOR: i64 = 1;

/// The most that the coldness of the cell adds to the fall numerator.
const FALL_FOR_COLD_GROUND: i64 = 6;

/// The most that cooling along the wind adds to the fall numerator.
///
/// The cooling term is larger than the coldness term, because it is the one
/// that tells the near side of a ridge from the far side. Height alone
/// cannot.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const FALL_FOR_COOLING: i64 = 16;

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

/// The drops that the sea lifts into the air over a cell of full heat.
///
/// A colder sea gives up less, in proportion to its heat.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
const LIFT_DROPS: i64 = 256;

/// The water in the air above one cell at which the sea stops lifting.
///
/// **This is the back pressure on the source.** Without it nothing limits how
/// much water enters the air, the source overwhelms every sink, and the total
/// in the air only climbs. A cell lifts what brings its air up to this mark
/// and no more, so the water the sea can put into the world is bounded by the
/// mark times the lattice.
///
/// A god is not bound by it. A storm is an injection, and the model carries
/// that water away rather than refusing it.[^1]
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D3. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
pub const AIR_SATURATION: Drops = Drops(2048);

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
/// A wet cell yields more to a unit that gathers from it. The value is a
/// content constant that no measurement chose, and a blocker holds the
/// question of what weather should be worth.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
pub const WET_MARK: Drops = Drops(64);

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

/// Returns the heat of one cell, from zero to the heat ceiling.
///
/// **A low coast is hot and a high ridge is cold.** The heat rises with the
/// share of the cell that holds open water and falls with the mean height of
/// the cell. Both come from the level 1 summary that the step rebuilt before
/// the solve ran, so the heat needs no storage and it cannot drift from the
/// world.[^1]
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
/// [^2]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
/// [^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub fn heat_of(summary: CellSummary) -> i32 {
    if summary.tiles() <= 0 {
        return 0;
    }
    // Water is the only ground that admits no unit, so the share of the cell
    // that admits none is its water share exactly.
    let open = summary.open_share().map_or(Fix32::ZERO, to_unit);
    let water = Fix32(Fix32::ONE.0 - open.0);
    let height = summary.mean_height().map_or(Fix32::ZERO, to_unit);
    let low = Fix32(Fix32::ONE.0 - height.0);
    part_of(HEAT_FROM_WATER, water) + part_of(HEAT_FROM_LOW_GROUND, low)
}

/// Returns the numerator of the share of the air that falls on one cell.
///
/// **Rain falls where the air cools.** Two terms say so. The first rises as
/// the cell itself gets colder, so air over cold ground drops more than air
/// over warm ground. The second rises with the cooling the air met on its way
/// here, which is the heat of the cell it came from above the heat of this
/// one.
///
/// That one rule gives two behaviours nobody writes down. Air crossing from
/// warm water onto a cold ridge cools and rains, so the near side of high
/// ground is wet. It reaches the far side carrying less, so the far side is
/// dry. The old rule read the mean height alone, and height without the
/// direction of travel cannot tell one side of a ridge from the other.[^1]
///
/// **This is public so that a test can move one input and watch the answer
/// move.**
///
/// # References
///
/// [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
#[must_use]
pub fn fall_numerator(heat: i32, cooling: i32) -> i64 {
    let cold = i64::from((HEAT_CEILING - heat).clamp(0, HEAT_CEILING));
    let cooled = i64::from(cooling.clamp(0, HEAT_CEILING));
    let whole = Accum(i64::from(HEAT_CEILING));
    let by_cold = sim_math::share(Accum(cold), Accum(FALL_FOR_COLD_GROUND), whole)
        .map_or(0, |value| value.0);
    let by_cooling =
        sim_math::share(Accum(cooled), Accum(FALL_FOR_COOLING), whole).map_or(0, |value| value.0);
    FALL_NUMERATOR_FLOOR + by_cold + by_cooling
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
    /// The heat of each cell, rebuilt at the head of every solve.
    ///
    /// **This is not state.** The solve fills it from the level 1 summaries
    /// before it reads it, and no later frame reads what a previous frame
    /// left. It therefore crosses no tick boundary and it enters no hash. The
    /// buffer is kept rather than allocated each solve, and that is a
    /// allocation and not a fact.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    heat: Vec<i32>,
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
        // **The wind exists over dry ground.** So the wind plane is built with
        // the field and never lazily, unlike the two water planes. The record
        // states that cost and the project owner chose it.[^1]
        //
        // [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
        let count = cells.tile_count() as usize;
        Ok(Self {
            cells,
            faction_count,
            air: Vec::new(),
            ground: Vec::new(),
            scratch: Vec::new(),
            wind: vec![Wind::STILL; count],
            wind_scratch: vec![Wind::STILL; count],
            heat: vec![0; count],
            wind_passes: 0,
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
        // The heat of every cell is rebuilt before anything reads it. Three
        // readers follow: the pressure that drives the wind, the lift, and
        // the fall.[^1]
        //
        // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
        self.warm(summaries);
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
        self.lift(tick, seed, summaries);
        if self.air.is_empty() {
            return Ok(());
        }
        for _ in 0..PASSES_FOR_EACH_SOLVE {
            self.transport(threads);
            self.passes = self.passes.saturating_add(1);
        }
        self.settle(summaries);
        Ok(())
    }

    /// Rebuilds the heat of every cell from the level 1 summaries.
    ///
    /// The pass walks the cells in ascending index order and reads nothing it
    /// is writing. It stores no fact: the buffer it fills is rewritten at the
    /// head of the next solve and no frame reads what a previous frame left
    /// there.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    fn warm(&mut self, summaries: &[CellSummary]) {
        self.heat.clear();
        self.heat.extend(summaries.iter().copied().map(heat_of));
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
            cells: self.cells,
            wind: &self.wind,
            heat: &self.heat,
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
    fn lift(&mut self, tick: Tick, seed: u64, summaries: &[CellSummary]) {
        let mut raised = 0i64;
        for (cell, summary) in summaries.iter().enumerate() {
            if !cell_lifts(
                seed,
                tick,
                cell as u32,
                summary.tiles(),
                summary.open_tiles(),
            ) {
                continue;
            }
            // **A hot sea gives up more than a cold one.** The quantity is
            // the heat of the cell as a share of the whole heat range.[^1]
            //
            // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            let heat = self.heat.get(cell).copied().unwrap_or(0);
            let wanted = sim_math::share(
                Accum(LIFT_DROPS),
                Accum(i64::from(heat)),
                Accum(i64::from(HEAT_CEILING)),
            )
            .map_or(0, |value| value.0);
            if wanted <= 0 {
                continue;
            }
            self.prepare();
            // **The air over a cell holds only so much, and that is the back
            // pressure on the source.** Without it nothing bounds what enters
            // the air, so the source overwhelms every sink and the total only
            // climbs. A saturated cell lifts nothing. The transport carries
            // the water away and the cell then lifts again, so a coast is a
            // steady supply rather than a runaway one.
            let room = AIR_SATURATION.0 - self.air[cell].0;
            let lifted = wanted.min(room);
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
        let count = self.cells.tile_count() as usize;
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
            cells: self.cells,
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
    fn settle(&mut self, summaries: &[CellSummary]) {
        let mut dried = 0i64;
        for cell in 0..summaries.len() {
            let heat = self.heat.get(cell).copied().unwrap_or(0);

            // **Rain falls where the air cools.** The cooling is the heat of
            // the cell the air came from above the heat of this one. The cell
            // it came from is the neighbour that the wind of this cell points
            // away from, and a still cell met no cooling on the way.[^1]
            //
            // [^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            let cooling = self.cooling_at(cell, heat);
            let numerator = fall_numerator(heat, cooling);
            let air = self.air[cell];
            let fallen = share_of(air, numerator, FALL_DENOMINATOR);
            self.air[cell] = Drops(air.0 - fallen.0);
            self.ground[cell] = self.ground[cell].combine(fallen);

            // **The heat of a cell lifts water off its own ground.** So a
            // place can be dried by its own heat and not only by time, and an
            // inland cell gives water back instead of only receiving it. The
            // water moves inside the account rather than entering it, because
            // it was already counted when it first entered the air.[^2]
            //
            // [^2]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D1. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
            let wet = self.ground[cell];
            let back = share_of(wet, i64::from(heat), GROUND_LIFT_DENOMINATOR);
            let back = Drops(back.0.min(wet.0));
            self.ground[cell] = Drops(wet.0 - back.0);
            self.air[cell] = self.air[cell].combine(back);

            // **Drying takes a share and then one whole drop more.** The
            // share alone truncates to nothing on ground that holds less than
            // the divisor, so water accumulated everywhere and never left.
            // The whole drop is what takes the last of it, and a cell holding
            // less than one drop loses what it holds.
            let wet = self.ground[cell];
            let share = share_of(wet, 1, DRY_DIVISOR);
            let leaving = (share.0 + DRY_FLOOR).min(wet.0).max(0);
            self.ground[cell] = Drops(wet.0 - leaving);
            dried = dried.saturating_add(leaving);
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
    fn cooling_at(&self, cell: usize, heat: i32) -> i32 {
        let Some(heading) = self.wind.get(cell).copied().unwrap_or(Wind::STILL).heading() else {
            return 0;
        };
        let Some(address) = self.cells.address_of(TileIdx(cell as u32)) else {
            return 0;
        };
        let Some(upwind) = self.cells.neighbour(address, opposite(heading)) else {
            return 0;
        };
        let Some(at) = self.cells.index_of(upwind) else {
            return 0;
        };
        let there = self.heat.get(at.0 as usize).copied().unwrap_or(0);
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
        running
            .write(bytemuck::cast_slice(&self.air))
            .write(bytemuck::cast_slice(&self.ground))
            .write(bytemuck::cast_slice(&self.wind))
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
    if count <= threads {
        fill(0, out);
        return;
    }
    let chunk_len = count.div_ceil(threads).max(1);
    let fill = &fill;
    std::thread::scope(|scope| {
        let mut start = 0usize;
        for chunk in out.chunks_mut(chunk_len) {
            let low = start;
            start += chunk.len();
            scope.spawn(move || fill(low, chunk));
        }
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

/// What one wind pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct WindPass<'a> {
    cells: Grid,
    wind: &'a [Wind],
    heat: &'a [i32],
}

impl WindPass<'_> {
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
            let here = self.heat.get(index).copied().unwrap_or(0);

            // The acceleration the pressure across this cell asks for. Hot
            // air rises and the pressure falls where it does, so the sum
            // points toward the hotter neighbours.
            let mut asked_q = 0i64;
            let mut asked_r = 0i64;
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(neighbour) = self.cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let there = self.heat.get(at.0 as usize).copied().unwrap_or(0);
                let pull = i64::from(there - here);
                asked_q += i64::from(NEIGHBOURS[direction].q) * pull;
                asked_r += i64::from(NEIGHBOURS[direction].r) * pull;
            }
            let step = cap(
                Wind {
                    q: narrow(sim_math::share(
                        Accum(asked_q),
                        Accum(1),
                        Accum(PRESSURE_DIVISOR),
                    )),
                    r: narrow(sim_math::share(
                        Accum(asked_r),
                        Accum(1),
                        Accum(PRESSURE_DIVISOR),
                    )),
                },
                WIND_STEP,
            );
            let carried = drag(self.wind[index]);
            *cell = cap(
                Wind {
                    q: carried.q + step.q,
                    r: carried.r + step.r,
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
