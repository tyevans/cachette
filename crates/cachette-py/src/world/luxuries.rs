//! The luxury deposits on the ground, and the variety they give.
//!
//! This module holds the verb that seeds the deposits, the ceiling on the
//! luxury identifiers, the deposits at one tile, and the variety of a tile, a
//! cell, a faction and the whole world.
//!
//! The grouping is by the subject. A luxury is a deposit on a tile and a
//! variety count over a set of tiles, and nothing else reads it.

use super::PyWorld;
use crate::errors::{ConfigError, ViewError};
use cachette_core::luxury::{LuxuryId, LUXURY_CEILING};
use cachette_core::FactionId;
use cachette_core::TileIdx;
use pyo3::prelude::*;

#[pymethods]
impl PyWorld {
    /// Seeds the luxuries of the world.
    ///
    /// A luxury is a presence and not a quantity. A tile carries a luxury or
    /// it does not, and no unit gathers one. The three gatherable kinds are a
    /// separate and fixed catalogue, and this call does not touch them.
    ///
    /// `placements` is a sequence of `(tile, luxury)` pairs of integers. The
    /// tile is the index of a tile in the world. The luxury is a number below
    /// the ceiling that `luxury_ceiling` reports. **The caller gives the
    /// whole set in one call**, in any order, and the engine sorts it. Python
    /// never loops over tiles.[^1]
    ///
    /// One tile may carry more than one luxury. A pair that repeats a
    /// luxury on a tile adds nothing, because a tile carries a luxury or it
    /// does not.
    ///
    /// **The world takes one seed.** A second call raises. The field is not a
    /// fact of a frame, so a reader of it never asks which frame it read.
    ///
    /// Read the result with `luxuries_at`, `variety_at`, `world_variety`,
    /// `cell_variety` and `faction_variety`.
    ///
    /// **Nothing in the engine reads the variety.** It is a score for the
    /// control plane, and no simulation pass consumes it.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the world already took a seed, when a pair
    /// names a luxury at or above the ceiling, and when a pair names a tile
    /// outside the world. A refusal changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: Decisions register, DEC-200. `docs/DECISIONS.md`
    fn seed_luxuries(&self, placements: Vec<(u32, u8)>) -> PyResult<()> {
        let seeds: Vec<(TileIdx, LuxuryId)> = placements
            .into_iter()
            .map(|(tile, luxury)| (TileIdx(tile), LuxuryId(luxury)))
            .collect();
        let mut world = self.lock();
        world
            .seed_luxuries(&seeds)
            .map_err(|error| ConfigError::new_err(error.to_string()))
    }

    /// The number of luxuries the world addresses, as an integer.
    ///
    /// A set of luxuries is one 64-bit word, so the catalogue holds this many
    /// and no more. A number at or above it is refused by `seed_luxuries`,
    /// and it is never folded onto another luxury: two luxuries on one bit
    /// would report the variety as one less than it is.
    #[staticmethod]
    fn luxury_ceiling() -> u8 {
        LUXURY_CEILING
    }

    /// The luxuries that one tile carries, as an integer bit set.
    ///
    /// Bit `n` of the answer stands for the luxury numbered `n`. A tile that
    /// carries nothing gives zero, and so does a tile outside the world,
    /// because such a tile carries nothing.
    ///
    /// The answer is one fixed-width integer, so this read costs the same
    /// whatever the number of luxuries on the tile.
    fn luxuries_at(&self, tile: u32) -> u64 {
        self.lock().luxuries_at(TileIdx(tile)).to_bits()
    }

    /// The number of different luxuries on one tile, as an integer.
    ///
    /// A tile that carries nothing gives zero.
    fn variety_at(&self, tile: u32) -> u32 {
        self.lock().luxuries_at(TileIdx(tile)).variety()
    }

    /// The number of different luxuries in the whole world, as an integer.
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no simulation pass consumes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[getter]
    fn world_variety(&self) -> u32 {
        self.lock().world_variety()
    }

    /// The number of luxury deposits in the whole world, as an integer.
    ///
    /// A deposit is one luxury on one tile. A tile that carries three
    /// luxuries holds three deposits. This counts deposits.
    /// `world_variety` counts different luxuries. The two answers differ
    /// whenever one luxury stands on more than one tile.
    #[getter]
    fn luxury_deposits(&self) -> i64 {
        self.lock().luxuries().deposits().0
    }

    /// The number of tiles that carry a luxury, as an integer.
    ///
    /// The engine stores one entry for each such tile and nothing else, so
    /// this is also the size of what the world stores for luxuries.
    #[getter]
    fn luxury_tile_count(&self) -> usize {
        self.lock().luxuries().len()
    }

    /// The number of different luxuries in one level 1 cell, as an integer.
    ///
    /// A cell summarises one block of tiles. The answer is the number of
    /// different luxuries on the tiles of the block. It is exact, because
    /// the engine combines the tiles by a set union and not by an average.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the world holds no such cell.
    fn cell_variety(&self, cell: u32) -> PyResult<u32> {
        self.lock()
            .variety_level()
            .variety(cell)
            .ok_or_else(|| ViewError::new_err(format!("the world holds no cell {cell}")))
    }

    /// The number of different luxuries on the ground one faction holds, as
    /// an integer.
    ///
    /// The answer counts a luxury once, whatever the number of held tiles
    /// that carry it. A faction that holds no ground gives zero.
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no simulation pass consumes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    fn faction_variety(&self, faction: u16) -> u32 {
        self.lock().faction_variety(FactionId(faction))
    }
}
