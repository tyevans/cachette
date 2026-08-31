//! The terrain field.
//!
//! Terrain is a pure function of the world seed and the tile address. The
//! engine stores no terrain map. A caller asks for a tile and the module
//! computes it, so the memory cost of the terrain field is the size of the
//! seed and the extent, at any tile count.
//!
//! Every value here is an integer or a Q16.16 fixed-point value, and every
//! arithmetic step goes through the arithmetic module.[^1] [^2] No item in
//! this module uses a floating-point type.
//!
//! Every lattice value comes from the counter-based generator, keyed on the
//! tuple of system, frame, entity and draw index.[^3] The frame slot holds a
//! constant, because terrain does not change with time. The entity slot
//! holds the lattice node address. The draw slot holds the field and the
//! octave, so the height field and the moisture field never correlate.
//!
//! The address is the whole input, so two callers that visit the world in
//! different orders, on different thread counts, read the same world.[^4]
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use crate::hex::{Axial, Grid};
use crate::rng;
use crate::sim_math;
use crate::types::{Fix32, TileIdx, FIX_FRACTIONAL_BITS};

/// The frame that every terrain draw is keyed on.
///
/// Terrain does not change with time, so the frame slot of the key holds one
/// constant. The slot stays in the key because the key shape is fixed by the
/// record, and a system that later grows a time-varying field takes the tick
/// here rather than adding a slot.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub const TERRAIN_FRAME: u64 = 0;

/// The number of octaves that each field sums.
const OCTAVES: u32 = 4;

/// The lattice shift of the coarsest octave.
///
/// The lattice spacing of octave zero is two raised to this power, measured
/// in tiles. Each later octave halves the spacing.
const COARSEST_SHIFT: u32 = 6;

/// The draw index of the first octave of the height field.
const DRAW_HEIGHT: u32 = 0;

/// The draw index of the first octave of the moisture field.
///
/// The gap between the two bases is wider than the octave count, so no
/// height octave and no moisture octave ever share a draw index. Two draws
/// that share a key are the same draw.
const DRAW_MOISTURE: u32 = 32;

/// The scale that restores a summed field to the unit range.
///
/// The octave weights halve, so four octaves sum to fifteen sixteenths of
/// the unit range. This value is sixteen fifteenths in Q16.16.
const NORMALISER: Fix32 = Fix32(69_905);

/// The middle of the unit range.
const HALF: Fix32 = Fix32(1 << (FIX_FRACTIONAL_BITS - 1));

/// The factor that spreads the summed field about the middle of the range.
///
/// The value is seven quarters in Q16.16.
const CONTRAST: Fix32 = Fix32(114_688);

/// The height below which a tile holds water.
const HEIGHT_WATER: Fix32 = Fix32(26_214);

/// The height below which a tile is level ground.
const HEIGHT_LEVEL: Fix32 = Fix32(40_632);

/// The height below which a tile is a hill.
const HEIGHT_HILL: Fix32 = Fix32(51_118);

/// The moisture at or above which level ground carries forest.
const MOISTURE_FOREST: Fix32 = Fix32(34_078);

/// The number of terrain kinds.
pub const KIND_COUNT: usize = 5;

/// The kind of a tile.
///
/// The kind is derived from the height field and the moisture field. It is
/// not stored, so it is not a column and it has no width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TileKind {
    /// Open water. No unit stands here.
    Water = 0,
    /// Level open ground.
    #[default]
    Plain = 1,
    /// Level ground under trees.
    Forest = 2,
    /// Rising ground.
    Hill = 3,
    /// High ground.
    Mountain = 4,
}

impl TileKind {
    /// Returns the kind as a small integer.
    ///
    /// A caller that writes the kind into a buffer needs the number. The
    /// numbering is stable, because a state hash and a viewer both read it.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Reports whether a unit may stand on a tile of this kind.
    ///
    /// This module states which tiles admit a unit. It does not state what a
    /// tile costs to cross. The cost multiplier is an open choice, and the
    /// register holds it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-017. `docs/DECISIONS.md`
    #[must_use]
    pub const fn is_passable(self) -> bool {
        !matches!(self, Self::Water)
    }
}

/// One generated tile.
///
/// The value is computed on demand and returned by value. Nothing in the
/// engine holds an array of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainTile {
    /// The height, as a fraction of the full range. The value is at least
    /// zero and below one.
    pub height: Fix32,
    /// The moisture, on the same scale as the height.
    pub moisture: Fix32,
    /// The kind, derived from the two fields.
    pub kind: TileKind,
}

/// The terrain of one world.
///
/// The type holds the seed and the extent, and nothing else. It allocates
/// nothing, whatever the tile count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Terrain {
    seed: u64,
    grid: Grid,
}

impl Terrain {
    /// Builds the terrain of a world.
    #[must_use]
    pub const fn new(seed: u64, grid: Grid) -> Self {
        Self { seed, grid }
    }

    /// Returns the seed that the terrain draws from.
    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    /// Returns the extent of the terrain.
    #[must_use]
    pub const fn grid(self) -> Grid {
        self.grid
    }

    /// Returns the tile at an address.
    ///
    /// Returns `None` when the address lies outside the world. The world
    /// does not wrap, so an address outside the extent names no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    #[must_use]
    pub fn tile(self, address: Axial) -> Option<TerrainTile> {
        if !self.grid.contains(address) {
            return None;
        }
        Some(generate(self.seed, address))
    }

    /// Returns the tile at an index.
    #[must_use]
    pub fn tile_at(self, index: TileIdx) -> Option<TerrainTile> {
        let address = self.grid.address_of(index)?;
        Some(generate(self.seed, address))
    }

    /// Returns the kind of the tile at an address.
    #[must_use]
    pub fn kind(self, address: Axial) -> Option<TileKind> {
        Some(self.tile(address)?.kind)
    }

    /// Returns the height of the tile at an address.
    #[must_use]
    pub fn height(self, address: Axial) -> Option<Fix32> {
        Some(self.tile(address)?.height)
    }
}

/// Generates one tile from the seed and the address.
///
/// The function reads nothing else. It is the whole of the terrain.
#[must_use]
pub fn generate(seed: u64, address: Axial) -> TerrainTile {
    let height = field(seed, DRAW_HEIGHT, address);
    let moisture = field(seed, DRAW_MOISTURE, address);
    TerrainTile {
        height,
        moisture,
        kind: classify(height, moisture),
    }
}

/// Returns the kind that a height and a moisture give.
///
/// The order of the tests is the order of the thresholds, so exactly one
/// branch runs and the kinds partition the range.
#[must_use]
pub const fn classify(height: Fix32, moisture: Fix32) -> TileKind {
    if height.0 < HEIGHT_WATER.0 {
        TileKind::Water
    } else if height.0 < HEIGHT_LEVEL.0 {
        if moisture.0 >= MOISTURE_FOREST.0 {
            TileKind::Forest
        } else {
            TileKind::Plain
        }
    } else if height.0 < HEIGHT_HILL.0 {
        TileKind::Hill
    } else {
        TileKind::Mountain
    }
}

/// Returns one field at an address, in the unit range.
///
/// The field is a weighted sum of octaves. The weights halve, so the coarse
/// octave sets the shape of a continent and the fine octaves roughen its
/// edge. The sum is then scaled back into the unit range.
fn field(seed: u64, base: u32, address: Axial) -> Fix32 {
    let mut total = Fix32::ZERO;
    let mut weight = Fix32(1 << (FIX_FRACTIONAL_BITS - 1));
    let mut octave = 0;
    while octave < OCTAVES {
        let value = octave_value(seed, base + octave, COARSEST_SHIFT - octave, address);
        total = sim_math::add(total, sim_math::mul(value, weight));
        weight = Fix32(weight.0 >> 1);
        octave += 1;
    }
    let scaled = sim_math::mul(total, NORMALISER);
    // A sum of independent draws clusters around the middle of the range, so
    // the raw field never reaches either end and the outer kinds would have
    // no share of the world. The contrast step spreads the field about the
    // midpoint and clamps what leaves the range. The flat parts at the ends
    // are the deep water and the high peaks.
    let centred = sim_math::sub(scaled, HALF);
    let expanded = sim_math::add(HALF, sim_math::mul(centred, CONTRAST));
    if expanded.0 >= Fix32::ONE.0 {
        Fix32(Fix32::ONE.0 - 1)
    } else if expanded.0 < 0 {
        Fix32::ZERO
    } else {
        expanded
    }
}

/// Returns one octave of value noise at an address.
///
/// The four corners of the lattice cell that holds the address each carry a
/// drawn value. The result interpolates between them, and the interpolation
/// weight passes through a smooth curve so that the field has no visible
/// lattice seam.
fn octave_value(seed: u64, draw_index: u32, shift: u32, address: Axial) -> Fix32 {
    let mask = (1i32 << shift) - 1;
    let cell_q = address.q >> shift;
    let cell_r = address.r >> shift;
    let weight_q = smooth(lattice_fraction(address.q & mask, shift));
    let weight_r = smooth(lattice_fraction(address.r & mask, shift));

    let low = interpolate(
        corner(seed, draw_index, cell_q, cell_r),
        corner(seed, draw_index, cell_q.saturating_add(1), cell_r),
        weight_q,
    );
    let high = interpolate(
        corner(seed, draw_index, cell_q, cell_r.saturating_add(1)),
        corner(
            seed,
            draw_index,
            cell_q.saturating_add(1),
            cell_r.saturating_add(1),
        ),
        weight_q,
    );
    interpolate(low, high, weight_r)
}

/// Returns the position inside a lattice cell, in the unit range.
const fn lattice_fraction(offset: i32, shift: u32) -> Fix32 {
    Fix32((offset << FIX_FRACTIONAL_BITS) >> shift)
}

/// Returns the drawn value of one lattice node, in the unit range.
///
/// The key names the node, not the tile, so every tile inside a cell reads
/// the same four corners. The top sixteen bits of the draw become the
/// fractional part of the value, which puts it at or above zero and below
/// one.
fn corner(seed: u64, draw_index: u32, cell_q: i32, cell_r: i32) -> Fix32 {
    let node = node_key(cell_q, cell_r);
    let raw = rng::draw(seed, rng::SYSTEM_TERRAIN, TERRAIN_FRAME, node, draw_index);
    Fix32((raw >> 48) as i32)
}

/// Packs a lattice node address into the entity slot of the draw key.
///
/// Both components reach the key. A key that dropped one would give a field
/// that varies along one axis and is constant along the other, and every
/// determinism test would still pass, because the defect repeats.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(not(feature = "probe-nondeterminism"))]
const fn node_key(cell_q: i32, cell_r: i32) -> u64 {
    ((cell_q as u32 as u64) << 32) | (cell_r as u32 as u64)
}

/// The perturbed packing. It drops the row component of the node address.
///
/// This is the defect that the testing rule warns about: the world it builds
/// is identical on every run and at every thread count, so both determinism
/// tests pass over it. Only a test of the key itself sees it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
const fn node_key(cell_q: i32, _cell_r: i32) -> u64 {
    (cell_q as u32 as u64) << 32
}

/// Returns the smooth interpolation weight for a position in a cell.
///
/// The curve is the cubic whose value and whose slope both match at each end
/// of the range. A linear weight would leave a visible crease at every
/// lattice line.
fn smooth(position: Fix32) -> Fix32 {
    let doubled = sim_math::add(position, position);
    let inner = sim_math::sub(Fix32::from_int(3), doubled);
    sim_math::mul(sim_math::mul(position, position), inner)
}

/// Returns the value between two ends at a weight.
fn interpolate(low: Fix32, high: Fix32, weight: Fix32) -> Fix32 {
    sim_math::add(low, sim_math::mul(sim_math::sub(high, low), weight))
}
