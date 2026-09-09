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
        // **This path derives the field whatever the seed sets did.** The step
        // derives the field only when a send changed a seed set, and a test
        // compares the field the step left against the field this path gives.
        // A path that read the same flag would compare the field against
        // itself.[^6]
        //
        // [^5]: Findings register, FND-664. `docs/FINDINGS.md`
        // [^6]: Testing rules, section 1. `.agents/rules/testing.md`
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
    /// **The field takes the seed set and decides for itself whether to
    /// derive.** It holds the arguments the last derivation read, and it
    /// skips the walk over the blocks when this call repeats them. The walk
    /// over the blocks is the largest cost of a frame that occupies no new
    /// block and empties no tile.[^3]
    ///
    /// **No stock plane conducts across water**, so every plane takes the
    /// land crossing. The empty slice is how the approach field states
    /// that.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^3]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    fn derive_stock_field(&mut self) {
        let seeds = {
            let _span = stage::open(Stage::RebuildStockSeeds);
            self.stock_seed_tiles()
        };
        self.stock_approaches.derive_within_when_the_inputs_changed(
            self.terrain,
            &seeds,
            &[],
            STOCK_PASSES,
        );
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
    /// **The walk asks the ground of a tile once, and then asks every
    /// kind.** The ground is generated from a noise field rather than
    /// stored, and the stock of a tile is generated from the ground, so the
    /// ground is the expensive half of the question. A walk that took the
    /// kind on the outside asked for the ground of each tile once for each
    /// kind, and the seed set is the largest cost of the stock field.[^4]
    /// [^5]
    ///
    /// The walk is over the arena in ascending identity order, then over the
    /// occupied blocks in ascending order, then over the tiles of each block
    /// in ascending offset, and then over the kinds in the order the table
    /// declares. It runs on the calling thread and it names no thread
    /// count.[^3]
    ///
    /// The derivation reads the answer as a set. It groups the seeds by the
    /// plane and the block, and each seed of a group gives its tile a reach
    /// of zero, so the order of this vector reaches no offset.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: Findings register, FND-590. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^5]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
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
        for block in &occupied {
            let first_column = (block % layout.blocks_wide()) * edge;
            let first_row = (block / layout.blocks_wide()) * edge;
            for row in first_row..first_row + edge {
                for column in first_column..first_column + edge {
                    let address = Axial::new(column as i32, row as i32);
                    let Some(tile) = self.grid.index_of(address) else {
                        continue;
                    };
                    let Some(ground) = self.terrain.kind(address) else {
                        continue;
                    };
                    for kind in ResourceKind::ALL {
                        let Some(original) =
                            self.resources.original_of_ground(address, ground, kind)
                        else {
                            continue;
                        };
                        let taken = self.depletion.taken(tile, kind).0;
                        if original.0 > taken {
                            seeds.push((u16::from(kind.to_u8()), tile));
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
    /// **The fine field takes the seed set and decides for itself whether to
    /// derive.** It holds the arguments the last derivation read, and it
    /// skips the walk over the blocks when this call repeats them. A frame
    /// that founds no site, razes none and takes none repeats them, and the
    /// walk is the largest cost of such a frame.[^4]
    ///
    /// The coarse field beside it derives at every call. It walks the level 1
    /// cells rather than the tiles of a block, and it costs a small part of
    /// what the fine one costs.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
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
        let seeds = self.site_seed_tiles();
        self.home_approaches
            .derive_when_the_inputs_changed(self.terrain, &seeds, &[]);
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

#[cfg(test)]
mod tests {
    //! The two guarded approach fields, and what each one depends on.
    //!
    //! The engine derives the home approach field and the stock approach
    //! field at every rebuild of level 1, and each one skips the walk when
    //! its arguments repeat. A skip that outlives a change to an argument is
    //! a stale offset, and a stale offset is a wrong answer that repeats on
    //! every thread count and on every machine. The two determinism tests
    //! compare a run against a run, so neither one can see it.[^1]
    //!
    //! The tests here therefore assert on what each field depends on. One
    //! test for each input drives the engine, changes that input, and reads
    //! the derivation count and the offsets the field answers with. A last
    //! test compares the guarded field against an unguarded derivation at
    //! every frame, which is the equality that makes the skip legal.[^2]
    //!
    //! The seed sets are private to this module, so the comparison against an
    //! unguarded derivation lives here rather than beside the public
    //! interface.[^3]
    //!
    //! # References
    //!
    //! [^1]: Testing Rules, section 2. `.agents/rules/testing.md`
    //! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    //! [^3]: Testing Rules, section 6. `.agents/rules/testing.md`

    use super::*;
    use crate::pyramid::ApproachField;
    use crate::types::Entity;
    use crate::world::WorldConfig;

    /// The seed of every world these tests build.
    const SEED: u64 = 20_260_908;

    /// How many planes a snapshot reads.
    ///
    /// A home plane is a faction and a stock plane is a resource kind, and
    /// the worlds here hold fewer of each than this. A snapshot that read
    /// only the planes one field uses would miss an offset that appeared in
    /// a plane nobody expected.
    const SNAPSHOT_PLANES: u16 = 8;

    /// The number of tiles that carry a unit that a fixture world needs.
    ///
    /// A world of open water seeds nothing, so every field of it is empty and
    /// every assertion below passes whatever the guard does. The fixture must
    /// supply ground, and this is the amount it must supply.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
    const GROUND_A_FIXTURE_NEEDS: u32 = 64;

    /// Builds a world of one faction count, on ground that carries units.
    ///
    /// The walk reads a world of each seed in turn and takes the first one
    /// whose ground carries enough units. It draws nothing of its own, so it
    /// adds no state to the fixture.
    fn world_of(faction_count: u16) -> World {
        for step in 0..64u64 {
            let config = WorldConfig {
                width: 48,
                height: 48,
                seed: SEED + step,
                faction_count,
                unit_capacity: 64,
            };
            let world = World::new(config).expect("the extent must describe a world");
            if passable_tile_count(&world) >= GROUND_A_FIXTURE_NEEDS {
                return world;
            }
        }
        panic!("no seed of the walk gives a world with ground");
    }

    /// Returns how many tiles of the world carry a unit.
    fn passable_tile_count(world: &World) -> u32 {
        let mut count = 0;
        for index in 0..world.grid().tile_count() {
            let Some(address) = world.grid().address_of(TileIdx(index)) else {
                continue;
            };
            if world.admits_a_unit(address) {
                count += 1;
            }
        }
        count
    }

    /// Returns the first address that carries a unit, from a starting tile.
    ///
    /// The walk asks the grid for the address of each tile, so it names no
    /// second rule for how a tile index maps to an address.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn passable_address_from(world: &World, first: u32) -> Axial {
        let count = world.grid().tile_count();
        for step in 0..count {
            let index = (first + step) % count;
            let Some(address) = world.grid().address_of(TileIdx(index)) else {
                continue;
            };
            if world.admits_a_unit(address) {
                return address;
            }
        }
        panic!("the world must hold ground that carries a unit");
    }

    /// Returns every offset the field answers with, over every plane a
    /// snapshot reads and every tile of the world.
    ///
    /// The vector is the whole observable projection of the field. Two fields
    /// derived from the same arguments give the same vector, and a field that
    /// went stale gives a different one.
    fn snapshot(world: &World, field: &ApproachField) -> Vec<Option<u8>> {
        let mut offsets = Vec::new();
        for plane in 0..SNAPSHOT_PLANES {
            for tile in 0..world.grid().tile_count() {
                offsets.push(field.offset(plane, TileIdx(tile)));
            }
        }
        offsets
    }

    /// Derives a field again from the site seed set of the world, with no
    /// guard.
    fn unguarded_home_field(world: &World) -> ApproachField {
        let mut field = ApproachField::new(world.pyramid.layout());
        field.derive(world.terrain, &world.site_seed_tiles(), &[]);
        field
    }

    /// Derives a field again from the stock seed set of the world, with no
    /// guard.
    fn unguarded_stock_field(world: &World) -> ApproachField {
        let mut field = ApproachField::new(world.pyramid.layout());
        field.derive_within(world.terrain, &world.stock_seed_tiles(), &[], STOCK_PASSES);
        field
    }

    /// Builds a world that holds one site of faction zero and one unit.
    fn world_with_one_site() -> (World, Entity) {
        let mut world = world_of(3);
        let address = passable_address_from(&world, 0);
        world
            .spawn_soldier(address, FactionId(0))
            .expect("the arena must take the soldier");
        let site = world
            .found_settlement(address, FactionId(0))
            .expect("the ground must take the settlement");
        world.step(1).expect("the step must run");
        (world, site)
    }

    #[test]
    fn a_frame_that_changes_no_site_derives_the_home_approach_field_no_further_time() {
        let (mut world, _site) = world_with_one_site();
        let derivations = world.home_approaches.derivations();
        let offsets = snapshot(&world, &world.home_approaches);
        for _ in 0..8 {
            world.step(1).expect("the step must run");
        }
        assert_eq!(
            world.home_approaches.derivations(),
            derivations,
            "eight frames that change no site must derive the field no further time"
        );
        assert_eq!(
            snapshot(&world, &world.home_approaches),
            offsets,
            "a frame that derives nothing must leave the offsets alone"
        );
    }

    #[test]
    fn founding_a_site_derives_the_home_approach_field_again() {
        let (mut world, _site) = world_with_one_site();
        let derivations = world.home_approaches.derivations();
        let offsets = snapshot(&world, &world.home_approaches);
        let far = passable_address_from(&world, world.grid().tile_count() / 2);
        world
            .spawn_soldier(far, FactionId(1))
            .expect("the arena must take the soldier");
        world
            .found_settlement(far, FactionId(1))
            .expect("the ground must take the settlement");
        world.step(1).expect("the step must run");
        assert!(
            world.home_approaches.derivations() > derivations,
            "a founded site must derive the field again"
        );
        assert_ne!(
            snapshot(&world, &world.home_approaches),
            offsets,
            "a founded site must change the offsets the field answers with"
        );
    }

    #[test]
    fn razing_a_site_derives_the_home_approach_field_again() {
        let (mut world, site) = world_with_one_site();
        let derivations = world.home_approaches.derivations();
        let offsets = snapshot(&world, &world.home_approaches);
        assert!(
            world.destroy_settlement(site),
            "the identity must resolve to a live site"
        );
        world.step(1).expect("the step must run");
        assert!(
            world.home_approaches.derivations() > derivations,
            "a razed site must derive the field again"
        );
        assert_ne!(
            snapshot(&world, &world.home_approaches),
            offsets,
            "a razed site must change the offsets the field answers with"
        );
    }

    /// A capture moves the site into the plane of the taker, and the field
    /// must answer in that plane afterwards.
    ///
    /// The test moves the faction column rather than winning a siege. A siege
    /// takes hundreds of frames of work, and the input this guard reads is
    /// the column.
    #[test]
    fn capturing_a_site_derives_the_home_approach_field_again() {
        let (mut world, site) = world_with_one_site();
        let derivations = world.home_approaches.derivations();
        let offsets = snapshot(&world, &world.home_approaches);
        assert!(
            world.settlements.set_faction(site, FactionId(2)),
            "the identity must resolve to a live site"
        );
        world.step(1).expect("the step must run");
        assert!(
            world.home_approaches.derivations() > derivations,
            "a captured site must derive the field again"
        );
        assert_ne!(
            snapshot(&world, &world.home_approaches),
            offsets,
            "a captured site must change the offsets the field answers with"
        );
    }

    #[test]
    fn a_frame_of_a_world_with_no_unit_derives_the_stock_field_no_further_time() {
        let mut world = world_of(1);
        world.step(1).expect("the step must run");
        let derivations = world.stock_approaches.derivations();
        let offsets = snapshot(&world, &world.stock_approaches);
        for _ in 0..8 {
            world.step(1).expect("the step must run");
        }
        assert_eq!(
            world.stock_approaches.derivations(),
            derivations,
            "eight frames of a world with no unit must derive the stock field no further time"
        );
        assert_eq!(
            snapshot(&world, &world.stock_approaches),
            offsets,
            "a frame that derives nothing must leave the offsets alone"
        );
    }

    #[test]
    fn a_unit_that_occupies_a_block_derives_the_stock_field_again() {
        let mut world = world_of(1);
        world.step(1).expect("the step must run");
        let derivations = world.stock_approaches.derivations();
        let offsets = snapshot(&world, &world.stock_approaches);
        let address = passable_address_from(&world, 0);
        world
            .spawn_soldier(address, FactionId(0))
            .expect("the arena must take the soldier");
        world.step(1).expect("the step must run");
        assert!(
            world.stock_approaches.derivations() > derivations,
            "a unit that occupies a block must derive the stock field again"
        );
        assert_ne!(
            snapshot(&world, &world.stock_approaches),
            offsets,
            "a unit that occupies a block must change the offsets the field answers with"
        );
    }

    /// The guarded field must answer what an unguarded derivation answers, at
    /// every frame.
    ///
    /// This is the equality that makes the skip an optimisation and not a
    /// different answer. The world holds units and sites, so the seed set of
    /// each field moves while the run goes on.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    #[test]
    fn a_guarded_field_answers_what_an_unguarded_derivation_answers() {
        let mut world = world_of(3);
        for slot in 0..12u32 {
            let address = passable_address_from(&world, slot * 37);
            let faction = FactionId(u16::try_from(slot % 3).expect("the faction must fit"));
            if world.spawn_soldier(address, faction).is_err() {
                continue;
            }
            if slot % 4 == 0 {
                let _ = world.found_settlement(address, faction);
            }
        }
        for frame in 0..40u32 {
            world.step(1).expect("the step must run");
            let home = unguarded_home_field(&world);
            assert_eq!(
                world.home_approaches.entry_count(),
                home.entry_count(),
                "the guarded home field must hold the entries of an unguarded one at frame {frame}"
            );
            assert_eq!(
                snapshot(&world, &world.home_approaches),
                snapshot(&world, &home),
                "the guarded home field must answer what an unguarded one answers at frame {frame}"
            );
            let stock = unguarded_stock_field(&world);
            assert_eq!(
                world.stock_approaches.entry_count(),
                stock.entry_count(),
                "the guarded stock field must hold the entries of an unguarded one at frame {frame}"
            );
            assert_eq!(
                snapshot(&world, &world.stock_approaches),
                snapshot(&world, &stock),
                "the guarded stock field must answer what an unguarded one answers at frame {frame}"
            );
        }
    }

    /// The stock seed set must hold a tile of an occupied block exactly when
    /// the tile holds stock of that kind.
    ///
    /// The seed walk asks the ground of a tile once and then generates the
    /// stock of each kind from it. The tile stock reader asks the ground
    /// again for each kind. Two ways of asking one question is the defect
    /// shape this project meets most often, so this test derives one from the
    /// other and compares.[^1]
    ///
    /// The comparison is over the whole world and over every kind, so a tile
    /// the walk added and a tile the walk missed both fail.
    ///
    /// **The test asserts that the wanted set is not empty.** A world whose
    /// occupied blocks hold no stock at all would compare two empty vectors,
    /// and the test would then measure the fixture.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: Testing Rules, section 2a. `.agents/rules/testing.md`
    #[test]
    fn the_stock_seed_set_agrees_with_the_tile_stock_reader() {
        let mut world = world_of(2);
        for slot in 0..8u32 {
            let address = passable_address_from(&world, slot * 53);
            let _ = world.spawn_soldier(address, FactionId(0));
        }
        for frame in 0..12u32 {
            world.step(1).expect("the step must run");
            let seeds = world.stock_seed_tiles();
            let layout = world.pyramid.layout();
            let mut occupied: Vec<u32> = Vec::new();
            for unit in world.soldiers.iter() {
                let Some(tile) = world.soldiers.tile(unit) else {
                    continue;
                };
                let Some(key) = layout.key_of(tile) else {
                    continue;
                };
                occupied.push(layout.block_of_key(key));
            }
            occupied.sort_unstable();
            occupied.dedup();
            let mut wanted: Vec<(u16, TileIdx)> = Vec::new();
            for index in 0..world.grid().tile_count() {
                let tile = TileIdx(index);
                let Some(key) = layout.key_of(tile) else {
                    continue;
                };
                if !occupied.contains(&layout.block_of_key(key)) {
                    continue;
                }
                let Some(address) = world.grid().address_of(tile) else {
                    continue;
                };
                for kind in ResourceKind::ALL {
                    if world
                        .tile_stock(address, kind)
                        .is_some_and(|amount| amount.0 > 0)
                    {
                        wanted.push((u16::from(kind.to_u8()), tile));
                    }
                }
            }
            assert!(
                !wanted.is_empty(),
                "the fixture must give an occupied block that holds stock, at frame {frame}"
            );
            let mut seen = seeds;
            seen.sort_unstable();
            wanted.sort_unstable();
            assert_eq!(
                seen, wanted,
                "the seed walk and the tile stock reader disagree at frame {frame}"
            );
        }
    }
}
