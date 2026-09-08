//! The summary pyramid, the fields derived over it, and the influence field.
//!
//! Level 0 is the only source of truth, and a summarised level is a derived
//! projection of it. The rebuild, the seeded fields and the direction readers
//! sit together, because each one reads the level below.

use super::errors::StepError;
use super::movement::Destinations;
use super::World;
use crate::bridge::BridgeError;
use crate::hex::Axial;
use crate::influence::{Influence, InfluenceField};
use crate::pyramid::{CellSummary, ExitField, Pyramid, ReturnField, STOCK_PASSES};
use crate::resource::ResourceKind;
use crate::stage::{self, Stage};
use crate::types::{FactionId, TileIdx};

impl World {
    /// Returns the exit direction of every cell and every option.
    ///
    /// The array is a projection of level 1. The engine derives it again at
    /// every rebuild of that level, and it holds no fact of its own.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    #[must_use]
    pub const fn exit_field(&self) -> &ExitField {
        &self.exits
    }

    /// Returns the exit direction that one option holds at one address.
    ///
    /// The direction is the index of one of the six neighbour offsets. A unit
    /// that stands at this address and holds this option steps to the
    /// neighbouring tile in that direction.[^1]
    ///
    /// The outer option reports whether the address and the option name an
    /// entry. The inner one reports whether the cell holds a direction. A cell
    /// that no neighbour beats holds none, and a unit there takes the uniform
    /// draw instead.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// Returns the direction of the nearest tile that holds stock of one
    /// kind, from one address.
    ///
    /// The answer is the seed offset when the address itself holds stock, a
    /// direction when the block that holds the address holds stock
    /// elsewhere, and nothing when the block holds none. A unit that reads
    /// nothing takes the direction of its level 1 cell instead.[^1]
    ///
    /// The engine seeds the field only over the blocks that hold a unit, so
    /// this reports nothing for an empty quarter of the world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-589. `docs/FINDINGS.md`
    #[must_use]
    pub fn stock_direction(&self, address: Axial, kind: ResourceKind) -> Option<u8> {
        let tile = self.grid.index_of(address)?;
        self.stock_approaches.offset(u16::from(kind.to_u8()), tile)
    }

    #[must_use]
    pub fn exit_direction(&self, address: Axial, option: u8) -> Option<Option<u8>> {
        let tile = self.grid.index_of(address)?;
        self.exits.exit(self.cell_of(tile)?, option)
    }

    /// Returns level 1 of the pyramid.
    ///
    /// The level is derived from level 0 and holds no fact of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    #[must_use]
    pub const fn pyramid(&self) -> &Pyramid {
        &self.pyramid
    }

    /// Returns the level 1 summary of the cell that covers one tile.
    #[must_use]
    pub fn summary_covering(&self, address: Axial) -> Option<CellSummary> {
        self.pyramid.cell_covering(address)
    }

    /// Returns what one faction reaches at the cell that covers one tile.
    ///
    /// This is the whole of the read side, and it is one gather from the
    /// level the caller already reads. Nothing walks from a unit to its
    /// faction and nothing asks who rules a tile.[^1]
    ///
    /// Returns `None` when the faction is outside the set the world holds, or
    /// when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-040. `docs/DECISIONS.md`
    #[must_use]
    pub fn influence(&self, faction: FactionId, address: Axial) -> Option<Influence> {
        self.influence.at(faction, self.influence_cell(address)?)
    }

    /// Returns the influence field of the world.
    ///
    /// A caller that reads more than one cell reads the field rather than
    /// calling the point query in a loop.
    #[must_use]
    pub const fn influence_field(&self) -> &InfluenceField {
        &self.influence
    }

    /// Sets what one faction injects at the cell that covers one tile.
    ///
    /// The world holds no rule that decides this value. A rule that writes a
    /// source term lives above the engine, and its absence is not a case: a
    /// source of zero is the ordinary value and no pass branches on it.[^1]
    ///
    /// Returns `false` when the faction or the address is outside the world.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-041. `docs/DECISIONS.md`
    pub fn set_influence_source(
        &mut self,
        faction: FactionId,
        address: Axial,
        source: Influence,
    ) -> bool {
        let Some(cell) = self.influence_cell(address) else {
            return false;
        };
        self.influence.set_source(faction, cell, source)
    }

    /// Returns the address, on the level 1 cell lattice, of the cell that
    /// covers one tile.
    ///
    /// The lattice is the block lattice at the pitch of one block, so the
    /// conversion is the block of the tile read as an address. It goes
    /// through the reader that already names the cell of a tile, so the world
    /// states that conversion once.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    fn influence_cell(&self, address: Axial) -> Option<Axial> {
        let tile = self.grid.index_of(address)?;
        crate::influence::cell_of_tile(self.pyramid.layout(), self.influence.cells(), tile)
    }

    /// Rebuilds level 1 from level 0.
    ///
    /// The engine calls this at the barrier. A caller that changed level 0
    /// outside a frame calls it too, in the same way it rebuilds the derived
    /// unit structure.
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// arena.
    pub fn rebuild_pyramid(&mut self, threads: usize) -> Result<(), StepError> {
        self.refresh_bridge()?;
        self.rebuild_level_1(threads, Destinations::Derive)?;
        Ok(())
    }

    /// Rebuilds level 1 and derives the exit field from it.
    ///
    /// **This is the one place that derives the field.** Every path that
    /// rebuilds level 1 comes through here: building a world, the barrier of a
    /// step, and the public rebuild that a caller runs outside a frame. A field
    /// left behind by one of those paths would be a stale value that nothing
    /// fails on, and a stale read is a confident wrong answer.[^1] [^2]
    ///
    /// The field is derived from the summaries this call just produced, so the
    /// choice, the summary and the field that a unit reads in one frame all
    /// come from one barrier.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// arena.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, the consequences. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    pub(super) fn rebuild_level_1(
        &mut self,
        threads: usize,
        destinations: Destinations,
    ) -> Result<(), BridgeError> {
        {
            let _span = stage::open(Stage::RebuildPyramid);
            self.pyramid.rebuild(
                &self.values,
                self.holding.holders(),
                &self.soldiers,
                &self.bridge,
                &self.depletion,
                threads,
            )?;
        }
        // Level 1 counted the units of this arena, so it states the
        // revision it read. Nothing else says which moment the level
        // describes.[^4]
        //
        // [^4]: Findings register, FND-648. `docs/FINDINGS.md`
        self.level_1_arena = Some(self.soldiers.revision());
        {
            let _span = stage::open(Stage::RebuildExits);
            self.exits.derive(&self.pyramid);
        }
        self.derive_return_fields();
        {
            let _span = stage::open(Stage::RebuildStock);
            self.derive_stock_field();
        }
        // **The step derives the destination field after the controller and
        // not here.** The controller sends several times in one frame, and
        // each send changes the seed set of a plane, so a field derived here
        // is overwritten before anything reads it. Nothing between this
        // barrier and the end of the step reads the field.[^5]
        //
        // Every other path through this function leaves the field derived,
        // because a caller outside a frame reads it as soon as the call
        // returns.[^1]
        //
        // [^5]: Findings register, FND-664. `docs/FINDINGS.md`
        if destinations == Destinations::Derive {
            let _span = stage::open(Stage::RebuildDestinations);
            self.derive_destination_fields();
        }
        Ok(())
    }

    /// Derives the fine field that steers a gatherer to stock.
    ///
    /// **This is the one place that derives it.** The field comes from the
    /// gather orders and the ground, and a path that rebuilt level 1 without
    /// it would leave a stale value that nothing fails on.[^1]
    ///
    /// **No stock plane conducts across water**, so every plane takes the
    /// land crossing. The empty slice is how the approach field states
    /// that.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    fn derive_stock_field(&mut self) {
        let seeds = self.stock_seed_tiles();
        self.stock_approaches
            .derive_within(self.terrain, &seeds, &[], STOCK_PASSES);
    }

    /// Returns one seed for each tile that holds stock, in a block that a
    /// unit stands in, as a resource kind plane and the tile.
    ///
    /// **The seed set follows the units, not the world.** A block that holds
    /// no unit seeds nothing, so the derivation costs an empty quarter of the
    /// world nothing at all. This is the cheaper algorithm that a set-valued
    /// question permits, and the set is the whole population.[^1]
    ///
    /// **Each occupied block seeds every kind, and the set reads no order
    /// column.** The field then states a fact about the ground alone: from
    /// this tile, this is the way to the nearest food, wood or stone inside
    /// the block. A set keyed on the order a unit holds now would go stale
    /// the moment anything wrote that column, and two passes of one step
    /// write it.[^2]
    ///
    /// A block that holds no stock of a kind seeds that kind nowhere, so the
    /// derivation builds no entry for it and the unit there reads the coarse
    /// field.
    ///
    /// The walk is over the arena in ascending identity order, and then over
    /// the tiles of each occupied block in ascending offset. It runs on the
    /// calling thread and it names no thread count.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: Findings register, FND-590. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn stock_seed_tiles(&self) -> Vec<(u16, TileIdx)> {
        let layout = self.pyramid.layout();
        let mut occupied: Vec<u32> = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            let Some(key) = layout.key_of(tile) else {
                continue;
            };
            occupied.push(layout.block_of_key(key));
        }
        occupied.sort_unstable();
        occupied.dedup();
        let edge = layout.block_edge();
        let mut seeds: Vec<(u16, TileIdx)> = Vec::new();
        for kind in ResourceKind::ALL {
            let plane = u16::from(kind.to_u8());
            for block in &occupied {
                let first_column = (block % layout.blocks_wide()) * edge;
                let first_row = (block / layout.blocks_wide()) * edge;
                for row in first_row..first_row + edge {
                    for column in first_column..first_column + edge {
                        let address = Axial::new(column as i32, row as i32);
                        let Some(tile) = self.grid.index_of(address) else {
                            continue;
                        };
                        if self
                            .tile_stock(address, kind)
                            .is_some_and(|amount| amount.0 > 0)
                        {
                            seeds.push((plane, tile));
                        }
                    }
                }
            }
        }
        seeds
    }

    /// Derives the coarse and the fine field that steer a unit home.
    ///
    /// **This is the one place that derives either of them.** Both come from
    /// the live sites, and a path that wrote one without the other would
    /// leave a stale value that nothing fails on.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn derive_return_fields(&mut self) {
        let _span = stage::open(Stage::RebuildReturns);
        self.returns.derive(&self.pyramid, &self.site_seeds());
        // **No return plane conducts across water**, so every plane takes the
        // land crossing. The empty slice is how the approach field states
        // that, in the way the return field states it to the coarse
        // derivation.[^3]
        //
        // [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
        drop(_span);
        let _span = stage::open(Stage::RebuildHomeApproaches);
        self.home_approaches
            .derive(self.terrain, &self.site_seed_tiles(), &[]);
    }

    /// Returns one seed for each live site, as a faction plane and the tile
    /// that holds the site.
    ///
    /// The walk is over the settlement slots in ascending order, so the set
    /// does not depend on a thread count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn site_seed_tiles(&self) -> Vec<(u16, TileIdx)> {
        let live = self.settlements.live_column();
        let tiles = self.settlements.tile_column();
        let factions = self.settlements.faction_column();
        let mut seeds = Vec::new();
        for (slot, alive) in live.iter().enumerate() {
            if *alive == 0 {
                continue;
            }
            seeds.push((factions[slot].0, tiles[slot]));
        }
        seeds
    }

    /// Returns one seed for each live site, as a faction and the level 1 cell
    /// that holds it.
    ///
    /// The walk is over the settlement slots in ascending order, so the set
    /// does not depend on a thread count.[^1] The derivation reads the set as
    /// a set: a seed gives a cell a reach of zero, and two seeds in one cell
    /// give the same answer as one.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn site_seeds(&self) -> Vec<(FactionId, u32)> {
        let live = self.settlements.live_column();
        let tiles = self.settlements.tile_column();
        let factions = self.settlements.faction_column();
        let mut seeds = Vec::new();
        for (slot, alive) in live.iter().enumerate() {
            if *alive == 0 {
                continue;
            }
            let Some(cell) = self.cell_of(tiles[slot]) else {
                continue;
            };
            seeds.push((factions[slot], cell));
        }
        seeds
    }

    /// Returns the return field of the world.
    ///
    /// The field holds the direction of the nearest site of a faction, for
    /// each level 1 cell.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    #[must_use]
    pub const fn return_field(&self) -> &ReturnField {
        &self.returns
    }

    /// Returns the direction that a unit of one faction takes to go home from
    /// one tile.
    ///
    /// The outer option reports whether the address and the faction name an
    /// entry. The inner one reports whether the cell holds a direction at
    /// all.
    #[must_use]
    pub fn return_direction(&self, faction: FactionId, address: Axial) -> Option<Option<u8>> {
        let tile = self.grid.index_of(address)?;
        self.returns.direction(faction, self.cell_of(tile)?)
    }
}
