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
use crate::hex::{Axial, Grid};
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

/// The share of the air over a cell that one pass hands to each neighbour.
///
/// A cell has at most six neighbours, and six eighths is below one whole, so
/// a cell never hands away more than it holds. The division truncates, and
/// the remainder stays where it was.
const SPREAD_DIVISOR: i64 = 8;

/// The denominator of the share of the air that falls in one solve.
const FALL_DENOMINATOR: i64 = 16;

/// The numerator of the share that falls on ground at the bottom of the
/// height range.
const FALL_NUMERATOR_FLOOR: i64 = 1;

/// The most that the height of the ground adds to the fall numerator.
///
/// The numerator runs from the floor above to the floor plus this, so the
/// highest ground takes four times as much out of the air as the lowest
/// ground does.
const FALL_NUMERATOR_RISE: i64 = 3;

/// The share of the water on the ground that leaves it in one solve.
///
/// The division truncates, so a cell holding fewer drops than this keeps
/// them. Ground that is barely wet stays barely wet until something moves it.
const DRY_DIVISOR: i64 = 32;

/// The drops that the sea lifts into the air over one cell, when it lifts.
const LIFT_DROPS: i64 = 256;

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

/// Returns the numerator of the share of the air that falls on one cell.
///
/// The numerator rises with the mean height of the cell. Ground at the bottom
/// of the height range takes the floor, and ground at the top takes the floor
/// plus the whole rise. A cell that covers no tile has no mean height, and it
/// takes the floor.
///
/// The height is a fixed-point fraction of one, so the arithmetic is a
/// multiply and a truncating divide through the arithmetic module.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub fn fall_numerator(summary: CellSummary) -> i64 {
    let Some(height) = summary.mean_height() else {
        return FALL_NUMERATOR_FLOOR;
    };
    let clamped = if height.0 < 0 {
        Fix32::ZERO
    } else if height.0 > Fix32::ONE.0 {
        Fix32::ONE
    } else {
        height
    };
    let rise = sim_math::share(
        Accum(i64::from(clamped.0)),
        Accum(FALL_NUMERATOR_RISE),
        Accum(i64::from(Fix32::ONE.0)),
    )
    .map_or(0, |value| value.0);
    FALL_NUMERATOR_FLOOR + rise
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
    /// Every drop that has ever entered the air, from the sea or from a god.
    raised: i64,
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
        Ok(Self {
            cells,
            faction_count,
            air: Vec::new(),
            ground: Vec::new(),
            scratch: Vec::new(),
            raised: 0,
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
        self.lift(tick, seed, summaries);
        if self.air.is_empty() {
            return Ok(());
        }
        for _ in 0..PASSES_FOR_EACH_SOLVE {
            self.spread(threads);
            self.passes = self.passes.saturating_add(1);
        }
        self.settle(summaries);
        Ok(())
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
            self.prepare();
            self.air[cell] = self.air[cell].combine(Drops(LIFT_DROPS));
            raised = raised.saturating_add(LIFT_DROPS);
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
        let cells = self.cells;
        let count = self.air.len();
        let pass = Pass {
            cells,
            air: &self.air,
        };
        if count <= threads {
            pass.fill(0, &mut self.scratch);
        } else {
            let chunk_len = count.div_ceil(threads).max(1);
            std::thread::scope(|scope| {
                let mut start = 0usize;
                for chunk in self.scratch.chunks_mut(chunk_len) {
                    let low = start;
                    start += chunk.len();
                    scope.spawn(move || pass.fill(low, chunk));
                }
            });
        }
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
        for (cell, summary) in summaries.iter().enumerate() {
            let numerator = fall_numerator(*summary);
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
            .write_u64(self.raised as u64)
            .write_u64(self.evaporated as u64);
        for ready in &self.ready {
            running = running.write_u64(ready.0);
        }
        running
            .write(bytemuck::cast_slice(&self.air))
            .write(bytemuck::cast_slice(&self.ground))
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

/// What one spread pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct Pass<'a> {
    cells: Grid,
    air: &'a [Drops],
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
            let given = share_of(here, 1, SPREAD_DIVISOR);

            let mut kept = here.0;
            let mut taken = Drops::ZERO;
            for neighbour in self.cells.neighbours(address).into_iter().flatten() {
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                // The cell hands this neighbour what it hands every
                // neighbour, and the neighbour hands back the eighth of what
                // it holds. Both integers come from the same input plane, so
                // the two ends of the edge agree exactly.
                kept -= given.0;
                let there = self.air[at.0 as usize];
                taken = taken.combine(share_of(there, 1, SPREAD_DIVISOR));
            }
            *cell = Drops(kept).combine(taken);
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
