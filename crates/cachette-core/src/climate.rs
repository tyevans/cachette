//! The climate field.
//!
//! A climate is what the weather leaves behind. The engine runs the weather
//! forward over an empty world for a fixed number of ticks, accumulates the
//! temperature and the standing water of every weather cell, and stores the
//! two totals. The stored field is small. It holds one entry for each weather
//! cell, and the weather lattice holds about sixteen thousand cells at the
//! target scale.
//!
//! **The terrain then reads the climate of the cell that covers an address.**
//! So the ground stays a pure function of the seed, the address and one small
//! stored field, and no map of tiles is stored.[^1]
//!
//! # The loop, and how this module breaks it
//!
//! The weather reads the ground. The heat of a cell rises with its open water
//! share and falls with its mean height.[^2] If the ground then read the
//! weather, the two would form a loop, and a loop that runs twice builds a
//! different world each time.
//!
//! **The spin reads the base ground only.** The ground array that the spin
//! hands the weather is folded from the terrain that the seed alone gives.
//! This module never hands the weather a climate-shaped ground. The
//! classification runs once, after the spin, and never feeds back. There is no
//! convergence test, because the spin runs a fixed tick count.[^3]
//!
//! The climate changes the moisture of a tile and never its height. The height
//! and the open water share are the only two terrain fields the weather reads,
//! and the two kinds that the moisture selects between are both passable. So
//! the ground array is the same whether it is folded from the base terrain or
//! from the climate-shaped terrain. A test states that.
//!
//! # Determinism
//!
//! The field is a pure function of the seed, the extent, the weather pitch and
//! the tick count. The weather solve is keyed on the seed and the tick, it
//! holds no thread-local state, and it gives one answer at any thread
//! count.[^4] The spin walks the ticks in ascending order and the cells in
//! ascending index order. The stored field enters the state hash.[^5]
//!
//! Every value here is an integer. No item in this module uses a
//! floating-point type, and every arithmetic step goes through the arithmetic
//! module.[^6]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
//! [^3]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
//! [^4]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::BlockLayout;
use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::padded::PaddedLattice;
use crate::sim_math;
use crate::terrain::Terrain;
use crate::types::{Accum, Fix32, Tick, FIX_FRACTIONAL_BITS};
use crate::weather::{
    cell_ground_of, ground_over_lattice, CellGround, WeatherError, WeatherField, WeatherScale,
    HEAT_CEILING,
};

/// The ticks that the climate spin runs.
///
/// The count is fixed. A spin that ran to a convergence test would end at a
/// different tick on a different world, and the record forbids that.[^1]
///
/// The value comes from a probe in this crate that ran the spin and reported
/// the tick at which the per-cell readings stopped moving. The commit that set
/// the value holds the output of the probe.
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
pub const SPIN_TICKS: u64 = 256;

/// The ticks at the head of the spin whose readings the field discards.
///
/// The temperature of a cell starts at zero and climbs toward what the ground
/// asks, and the air starts empty. A mean taken over the whole spin therefore
/// holds the cold, dry opening of the run, and that is a transient rather than
/// a climate. The field drops the opening and averages what follows.
pub const WARM_UP_TICKS: u64 = 64;

/// One cell of the climate field.
///
/// The type is plain data with a declared layout, because the field enters the
/// state hash and undeclared padding puts uninitialised bytes into it.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct CellClimate {
    /// The temperature of the cell summed over every counted tick.
    ///
    /// The accumulator is 64 bits wide. A 32-bit accumulator would overflow
    /// at a long spin, and an accumulator must not depend on a margin.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub warmth_total: i64,
    /// The water standing on the cell summed over every counted tick.
    pub wetness_total: i64,
    /// The most water the cell held on any counted tick.
    pub wettest: i64,
    /// The least water the cell held on any counted tick.
    pub driest: i64,
    /// The ticks that the three totals above counted.
    pub counted: i64,
}

impl CellClimate {
    /// A cell that no tick counted.
    pub const EMPTY: Self = Self {
        warmth_total: 0,
        wetness_total: 0,
        wettest: 0,
        driest: i64::MAX,
        counted: 0,
    };

    /// Adds one tick of readings to the cell.
    ///
    /// The operation is field-wise, and each field is a sum, a maximum or a
    /// minimum. All three combine exactly and in any order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn observe(self, warmth: i64, wetness: i64) -> Self {
        Self {
            warmth_total: self.warmth_total.saturating_add(warmth),
            wetness_total: self.wetness_total.saturating_add(wetness),
            wettest: if wetness > self.wettest {
                wetness
            } else {
                self.wettest
            },
            driest: if wetness < self.driest {
                wetness
            } else {
                self.driest
            },
            counted: self.counted.saturating_add(1),
        }
    }

    /// Returns the mean temperature of the cell.
    ///
    /// Returns zero when no tick counted the cell.
    #[must_use]
    pub const fn mean_warmth(self) -> i64 {
        if self.counted <= 0 {
            return 0;
        }
        self.warmth_total / self.counted
    }

    /// Returns the mean water standing on the cell.
    ///
    /// Returns zero when no tick counted the cell.
    #[must_use]
    pub const fn mean_wetness(self) -> i64 {
        if self.counted <= 0 {
            return 0;
        }
        self.wetness_total / self.counted
    }

    /// Returns the spread between the wettest counted tick and the driest.
    ///
    /// Returns zero when no tick counted the cell. A cell with a wide spread
    /// has a season. A cell with a narrow one is wet, or dry, all year.
    #[must_use]
    pub const fn season_spread(self) -> i64 {
        if self.counted <= 0 {
            return 0;
        }
        self.wettest - self.driest
    }
}

/// The climate of one world.
///
/// The field holds one entry for each weather cell, and the layout that says
/// which cell covers a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClimateField {
    layout: BlockLayout,
    cells: Vec<CellClimate>,
    ticks: u64,
}

impl ClimateField {
    /// Builds a climate that says nothing.
    ///
    /// Every address reads as temperate, so a world that holds this field
    /// generates the terrain that the seed alone gives. A world that asks for
    /// no spin holds this.
    #[must_use]
    pub const fn quiet(layout: BlockLayout) -> Self {
        Self {
            layout,
            cells: Vec::new(),
            ticks: 0,
        }
    }

    /// Runs the weather forward over an empty world and stores what it left.
    ///
    /// The ground that the spin hands the weather is folded from the terrain
    /// that the seed alone gives. **Nothing the spin computes reaches that
    /// fold**, so the run is not a loop and running it twice gives one answer.
    ///
    /// The world is empty. No unit stands in it and no god inflicts weather on
    /// it, so the only water that enters the air is the water the sea lifts.
    ///
    /// # Errors
    ///
    /// Returns an error when the weather refuses to build or to solve.
    pub fn spin(
        terrain: Terrain,
        scale: WeatherScale,
        ticks: u64,
        threads: usize,
    ) -> Result<Self, WeatherError> {
        let grid = terrain.grid();
        let layout =
            BlockLayout::new(grid, scale.bits()).map_err(|_| WeatherError::LatticeMismatch)?;
        let cell_lattice = Grid::new(layout.blocks_wide(), layout.blocks_high())
            .map_err(|_| WeatherError::LatticeMismatch)?;
        // **The spin runs over the same padded lattice the world runs over.**
        // A spin over a bare lattice would starve its own border, and the
        // climate it stored would then hold that starvation as if it were
        // weather.[^2]
        //
        // [^2]: The padded lattice. `crates/cachette-core/src/padded.rs`
        let lattice = PaddedLattice::new(cell_lattice, scale.margin_cells())
            .map_err(|_| WeatherError::LatticeMismatch)?;
        let ground = ground_over_lattice(lattice, layout, terrain);
        let mut weather = WeatherField::new(lattice, scale, 1)?;
        let mut cells = vec![CellClimate::EMPTY; cell_lattice.tile_count() as usize];
        // The spin walks the ticks in ascending order, and the reading pass
        // walks the cells in ascending index order. Neither order depends on
        // the order in which a thread finished.[^1]
        //
        // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        for tick in 1..=ticks {
            weather.solve(Tick(tick), terrain.seed(), &ground, threads)?;
            if tick <= WARM_UP_TICKS {
                continue;
            }
            // The field is one entry for each cell of the world, and the
            // weather planes hold the margin as well. The reading goes
            // through the lattice, so no entry of the field reads a margin
            // cell.
            for (cell, slot) in cells.iter_mut().enumerate() {
                let Some(index) = lattice.whole_of_inner(cell as u32) else {
                    continue;
                };
                *slot = slot.observe(
                    i64::from(weather.warmth_at(index)),
                    weather.ground_at(index).0,
                );
            }
        }
        Ok(Self {
            layout,
            cells,
            ticks,
        })
    }

    /// Returns the ticks that the spin ran.
    #[must_use]
    pub const fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Returns the block layout of the weather lattice.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns every cell of the field, in cell index order.
    #[must_use]
    pub fn cells(&self) -> &[CellClimate] {
        &self.cells
    }

    /// Reports whether the field says nothing.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns the climate of one cell.
    #[must_use]
    pub fn cell(&self, cell: u32) -> Option<CellClimate> {
        self.cells.get(cell as usize).copied()
    }

    /// Returns the cell that covers a tile address.
    #[must_use]
    pub fn cell_of(&self, address: Axial) -> Option<u32> {
        let tile = self.layout.grid().index_of(address)?;
        let key = self.layout.key_of(tile)?;
        Some(self.layout.block_of_key(key))
    }

    /// Returns the climate over a tile address.
    ///
    /// Returns [`Climate::TEMPERATE`] when the field says nothing, so a world
    /// with no spin reads the terrain that the seed alone gives.
    #[must_use]
    pub fn at(&self, address: Axial) -> Climate {
        self.at_reference(address, self.wetness_reference())
    }

    /// Returns the climate over a tile address, against a stated reference.
    ///
    /// A caller that reads many addresses takes the reference once and hands
    /// it to each call, because the reference folds the whole field and a
    /// caller that took it for each tile would fold the field once for each
    /// tile.
    #[must_use]
    pub fn at_reference(&self, address: Axial, reference: i64) -> Climate {
        if self.cells.is_empty() {
            return Climate::TEMPERATE;
        }
        let Some(cell) = self.cell_of(address).and_then(|at| self.cell(at)) else {
            return Climate::TEMPERATE;
        };
        Climate::of(cell, reference)
    }

    /// Returns the mean standing water over every cell that the spin counted.
    ///
    /// **The reference is the world, and it is not a constant.** How much
    /// water stands on a cell depends on the weather model, on the pitch of
    /// the lattice and on how much sea the world holds. A fixed threshold
    /// would call every cell of an ocean world wet, and every cell of a dry
    /// world dry. A share against the mean asks the question that matters: is
    /// this cell wetter, or drier, than the rest of this world?
    #[must_use]
    pub fn wetness_reference(&self) -> i64 {
        let mut total = Accum(0);
        let mut counted = 0i64;
        for cell in &self.cells {
            if cell.counted <= 0 {
                continue;
            }
            total = sim_math::combine(total, Accum(cell.mean_wetness()));
            counted += 1;
        }
        if counted <= 0 {
            return 0;
        }
        total.0 / counted
    }

    /// Folds the field into the state hash.
    ///
    /// The step reads the field through the terrain, so it is state and it
    /// enters the hash.[^1] The cells enter in index order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash
            .write_u64(self.ticks)
            .write_u64(self.cells.len() as u64);
        for cell in &self.cells {
            running = running.write(bytemuck::bytes_of(cell));
        }
        running
    }
}

/// What the climate of one cell says to the ground under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Climate {
    /// The temperature of the cell, from zero to the weather heat ceiling.
    pub warmth: i32,
    /// What the climate adds to the generated moisture of a tile.
    ///
    /// The value is a signed fixed-point offset. It is positive where the cell
    /// stands wetter than the world, and negative where it stands drier.
    pub moisture_offset: Fix32,
}

impl Climate {
    /// The climate that changes nothing.
    ///
    /// The warmth is the middle of the weather heat range and the offset is
    /// zero, so a tile under this climate is the tile that the seed alone
    /// gives.
    pub const TEMPERATE: Self = Self {
        warmth: HEAT_CEILING / 2,
        moisture_offset: Fix32::ZERO,
    };

    /// Returns what a cell says, against the wetness of the world around it.
    ///
    /// The offset is the distance of the cell from the reference, as a share
    /// of the reference, scaled by the reach. A cell twice as wet as the world
    /// takes the whole reach upward. A cell that holds no water at all takes
    /// the whole reach downward.
    #[must_use]
    pub fn of(cell: CellClimate, reference: i64) -> Self {
        if cell.counted <= 0 || reference <= 0 {
            return Self::TEMPERATE;
        }
        let apart = cell.mean_wetness() - reference;
        let share = sim_math::share(
            Accum(i64::from(MOISTURE_REACH.0)),
            Accum(apart),
            Accum(reference),
        )
        .map_or(0, |value| value.0);
        let bounded = share.clamp(-i64::from(MOISTURE_REACH.0), i64::from(MOISTURE_REACH.0));
        Self {
            warmth: cell.mean_warmth().clamp(0, i64::from(HEAT_CEILING)) as i32,
            moisture_offset: Fix32(bounded as i32),
        }
    }

    /// Reports whether the cell stands hot.
    #[must_use]
    pub const fn is_hot(self) -> bool {
        self.warmth >= HOT_MARK
    }

    /// Reports whether the cell stands cold.
    #[must_use]
    pub const fn is_cold(self) -> bool {
        self.warmth <= COLD_MARK
    }

    /// Reports whether the cell stands drier than the world around it.
    #[must_use]
    pub const fn is_dry(self) -> bool {
        self.moisture_offset.0 < 0
    }
}

/// The largest moisture that a climate adds to a tile, or takes from it.
///
/// The value is a quarter of the unit range. The generated moisture field is a
/// sum of four octaves spread about the middle of the range, so a quarter of
/// the range moves a cell across the forest threshold without flattening the
/// field inside the cell. **The ground that the seed drew is still visible
/// under the climate**, and that is the point. A wet region grows more forest.
/// It does not become one solid block of forest.
pub const MOISTURE_REACH: Fix32 = Fix32(1 << (FIX_FRACTIONAL_BITS - 2));

/// The temperature at or above which a cell stands hot.
///
/// The value is three quarters of the weather heat range.
pub const HOT_MARK: i32 = HEAT_CEILING * 3 / 4;

/// The temperature at or below which a cell stands cold.
///
/// The value is one quarter of the weather heat range.
pub const COLD_MARK: i32 = HEAT_CEILING / 4;

/// Folds the terrain of a world into the ground under each weather cell.
///
/// **The terrain here carries no climate.** The type holds the seed and the
/// extent and nothing else, so it cannot carry one. This is the fold that
/// breaks the loop, and it stays sound only while that stays true.
#[must_use]
pub fn base_ground_of(layout: BlockLayout, terrain: Terrain) -> Vec<CellGround> {
    cell_ground_of(layout, terrain)
}
