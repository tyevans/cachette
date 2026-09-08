//! The tile readers, and the pass that ages the tile value field.
//!
//! The readers answer what one tile is: its terrain, its climate, its kind
//! and its value. The pass over a chunk of the value column sits beside them,
//! because it writes the field the readers read.

use super::World;
use crate::climate::{Climate, ClimateField};
use crate::event::{TileChanged, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};
use crate::hex::{Axial, Grid};
use crate::holding::Holder;
use crate::rng;
use crate::terrain::{Terrain, TerrainTile, TileKind};
use crate::tile_value::TileValueChunk;
use crate::types::{Accum, Fix32, Tick, TileIdx};

/// What one worker produces from one range of tiles.
///
/// The worker writes the events it emitted and the changes it made. It
/// writes nothing to the world, so two workers on two ranges cannot race.
#[derive(Clone, Debug, Default)]
pub(super) struct ChunkResult {
    /// The events of the range, in ascending tile order.
    pub(super) events: Vec<TileChanged>,
    /// The net change this range made to the count of changed tiles.
    ///
    /// The range applied its own changes to the field as it went, so nothing
    /// is carried here for a later pass to apply. Only the count comes back,
    /// because the count belongs to the whole field and not to one range.
    pub(super) changed: i64,
}

/// Updates one contiguous range of tiles, in place, and returns its events.
///
/// The function is pure in the sense that the record requires: the same prior
/// values and the same key give the same result.[^1]
///
/// **The range is a mutable chunk of the field, and it is the only writer of
/// those tiles.** The field hands out disjoint chunks, so this function
/// cannot reach a tile another worker holds, and it needs no atomic.[^2]
///
/// The draw is keyed on the system, the frame and the tile, so a tile gives
/// the same answer whichever worker holds it and at any thread count.[^3]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
/// [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub(super) fn update_range(tick: Tick, seed: u64, mut chunk: TileValueChunk<'_>) -> ChunkResult {
    let mut result = ChunkResult::default();
    let (start, end) = (chunk.start(), chunk.end());
    for index in start..end {
        let raw = rng::draw_below(seed, rng::SYSTEM_TILE_STUB, tick.0, u64::from(index), 0, 8);
        if raw >= 4 {
            continue;
        }
        let delta = Fix32((raw as i32) - 2);
        if delta.0 == 0 {
            continue;
        }
        // The tile is inside the chunk by construction, because the loop
        // walks the chunk's own range. A `None` here would mean the chunk
        // reported a range it does not hold.
        let Some(updated) = chunk.add(TileIdx(index), delta) else {
            continue;
        };
        let kind = if delta.0 > 0 {
            CHANGE_KIND_RAISED
        } else {
            CHANGE_KIND_LOWERED
        };
        // The holder is stamped after the holding spread, at the end of the
        // step. This pass runs at the top of the step, so any holder it read
        // here would be the holder of the frame before.[^4]
        //
        // [^4]: Findings register, FND-029. `docs/FINDINGS.md`
        result.events.push(TileChanged::new(
            tick,
            TileIdx(index),
            updated,
            Holder::NOBODY,
            kind,
        ));
    }
    result.changed = chunk.changed();
    result
}

impl World {
    /// Returns the shape of the world.
    ///
    /// A caller reads a tile address through the grid. The viewer needs it
    /// to place a tile on the screen, because the engine holds no screen
    /// position.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the terrain of the world.
    ///
    /// The terrain holds the seed and the extent, and nothing else. It costs
    /// the same at any tile count, because it stores no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub const fn terrain(&self) -> Terrain {
        self.terrain
    }

    /// Returns the terrain of one tile.
    ///
    /// Returns `None` when the address lies outside the world. The call
    /// computes the tile. It reads no array of tiles, so it never goes
    /// stale.[^1]
    ///
    /// **The climate of the world reaches the answer.** A world that asked for
    /// no spin holds a quiet climate, which reads temperate at every address
    /// and changes nothing, so the answer is the tile that the seed alone
    /// gives.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: The climate field. [`ClimateField`]
    #[must_use]
    pub fn tile_terrain(&self, address: Axial) -> Option<TerrainTile> {
        self.terrain.tile_under(address, self.tile_climate(address))
    }

    /// Returns the terrain kind of one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn tile_kind(&self, address: Axial) -> Option<TileKind> {
        Some(self.tile_terrain(address)?.kind)
    }

    /// Returns the climate over one tile.
    ///
    /// A world that asked for no spin answers temperate at every address.
    #[must_use]
    pub fn tile_climate(&self, address: Axial) -> Climate {
        self.climate.at_reference(address, self.climate_reference)
    }

    /// Returns the climate field of the world.
    #[must_use]
    pub const fn climate(&self) -> &ClimateField {
        &self.climate
    }

    /// Returns the value of the tile at an address.
    ///
    /// Returns `None` when the address is outside the world. The lookup is
    /// one multiply, one add, and one load. It converts no coordinate.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    #[must_use]
    pub fn tile_value(&self, address: Axial) -> Option<Fix32> {
        let index = self.grid.index_of(address)?;
        self.values.at(index)
    }

    /// Returns the value of the tile at an index.
    ///
    /// Returns `None` when the index names no tile. A caller that already
    /// holds an index uses this and converts no coordinate.
    #[must_use]
    pub fn tile_value_at(&self, index: TileIdx) -> Option<Fix32> {
        self.values.at(index)
    }

    /// Returns the number of tiles.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.grid.tile_count() as usize
    }

    /// Returns a copy of the whole tile value column.
    ///
    /// **The call visits every tile and allocates one value for each.** The
    /// world holds no array of tile values, so there is no view to hand out
    /// and the copy is the whole cost. The name says so, because what copies
    /// is declared at the call site.[^1]
    ///
    /// A caller that wants one tile calls the single-tile read instead.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn copy_tile_values(&self) -> Vec<Fix32> {
        self.values.copy_all()
    }

    /// Returns the number of tiles that hold a stored change.
    ///
    /// A world that has never stepped holds none, at any tile count. The
    /// count grows with what the frames have changed and never with the size
    /// of the world alone, which is what the product record asks of the
    /// build.[^1]
    ///
    /// # References
    ///
    /// [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    #[must_use]
    pub fn stored_tile_changes(&self) -> usize {
        self.values.stored_changes()
    }

    /// Returns the sum of the tile column.
    ///
    /// The accumulator is 64 bits wide, and the addition is exactly
    /// associative, so the answer does not depend on the fold order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`, and ADR-0004, iteration order is explicit, decision D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn tile_total(&self) -> Accum {
        self.values.total()
    }
}
