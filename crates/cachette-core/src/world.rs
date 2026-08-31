//! The world, and the frame step.
//!
//! The world holds one tile field, one event type, and two systems. The
//! tile system is a stub, and it exists so that the determinism harnesses
//! have a subject before the first solver is written.[^1] The movement
//! system gives each soldier one neighbour tile each frame. Replace the body
//! of a system. Do not replace the shape of the step.
//!
//! The step shows three rules that every later system must follow.
//!
//! The step writes each parallel result to an indexed output slot. It never
//! uses thread completion order and it never uses work-stealing order.[^2]
//! The target has a weak memory model, so an atomic costs a real barrier
//! where a strong model would absorb it. Disjoint outputs are therefore a
//! requirement and not a preference.[^3]
//!
//! The step draws every random value from a counter, keyed on the tuple of
//! system, frame, entity and draw index.[^4]
//!
//! The step routes all arithmetic through the arithmetic module.[^5]
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^3]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak, a draft record. `docs/adrs/draft/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge, BLOCK_BITS_DEFAULT};
use crate::event::{TileChanged, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, GridError, NEIGHBOUR_COUNT};
use crate::pyramid::{CellSummary, Pyramid};
use crate::rng;
use crate::sim_math;
use crate::site::{CommodityId, SettlementArena, SettlementError};
use crate::slots::Slots;
use crate::soldier::{SoldierArena, SoldierError};
#[cfg(not(feature = "probe-nondeterminism"))]
use crate::sort;
use crate::sort::{BoundedKey, SortError};
use crate::terrain::{Terrain, TerrainTile, TileKind};
use crate::types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx, FACTION_CEILING};

/// The reason that a step refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    /// The caller asked for zero threads. A step needs at least one.
    ZeroThreads,
    /// The rebuild of the unit-to-tile bridge refused to run.
    Bridge(BridgeError),
    /// The admission sort refused the intent keys.
    Sort(SortError),
    /// An intent named a tile outside the world.
    ///
    /// The intent half already drops a target outside the extent, so this
    /// says that the two halves disagree rather than that a caller erred.
    TargetOutsideWorld,
}

impl From<SortError> for StepError {
    fn from(error: SortError) -> Self {
        Self::Sort(error)
    }
}

impl From<BridgeError> for StepError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

/// The reason that a world refused to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// The configured extent does not describe a grid.
    Grid(GridError),
    /// The block partition of the world refused to build.
    Bridge(BridgeError),
    /// The configured faction count is above the storage ceiling.
    ///
    /// A faction is one bit in a 64-bit mask, so the project holds a fixed
    /// ceiling. A world may hold fewer factions than the ceiling. It may
    /// never hold more.[^1]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    FactionCountAboveCeiling(u16),
}

impl From<BridgeError> for WorldError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

impl core::fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Grid(error) => write!(formatter, "the world extent is not a grid: {error}"),
            Self::Bridge(error) => write!(formatter, "the world has no block partition: {error}"),
            Self::FactionCountAboveCeiling(count) => write!(
                formatter,
                "the world asks for {count} factions, and the ceiling is {FACTION_CEILING}"
            ),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<GridError> for WorldError {
    fn from(error: GridError) -> Self {
        Self::Grid(error)
    }
}

impl core::fmt::Display for StepError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a step needs at least one thread"),
            Self::Bridge(error) => write!(formatter, "the bridge rebuild refused: {error}"),
            Self::Sort(error) => write!(formatter, "the admission sort refused: {error}"),
            Self::TargetOutsideWorld => {
                write!(formatter, "an intent named a tile outside the world")
            }
        }
    }
}

impl std::error::Error for StepError {}

/// The settings that build a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    /// The number of columns in the world.
    ///
    /// The world is a rhombus, so the extent is a width and a height and the
    /// tile count follows from them.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    pub width: u32,
    /// The number of rows in the world.
    pub height: u32,
    /// The world seed. Every random draw takes it.
    pub seed: u64,
    /// The number of factions.
    ///
    /// The ceiling is 63, because a faction is one bit in a 64-bit mask and
    /// one value is reserved for no faction. The scale constants table holds
    /// the value.[^1]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    pub faction_count: u16,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        }
    }
}

/// A simulated world.
///
/// The world holds no global mutable state, so one process may hold many
/// worlds and step them in parallel.[^1]
///
/// # References
///
/// [^1]: ADR-0047, many worlds live in one interpreter. `docs/adrs/REGISTRY.md`
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    grid: Grid,
    tick: Tick,
    values: Vec<Fix32>,
    factions: Vec<FactionId>,
    log: Vec<TileChanged>,
    soldiers: SoldierArena,
    settlements: SettlementArena,
    bridge: UnitTileBridge,
    terrain: Terrain,
    /// Level 1 of the pyramid, derived from level 0 at the barrier.
    pyramid: Pyramid,
}

impl World {
    /// Builds a world from the settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid.
    pub fn new(config: WorldConfig) -> Result<Self, WorldError> {
        if config.faction_count > FACTION_CEILING {
            return Err(WorldError::FactionCountAboveCeiling(config.faction_count));
        }
        let grid = Grid::new(config.width, config.height)?;
        let count = grid.tile_count() as usize;
        let divisor = u64::from(config.faction_count.max(1));
        let mut values = Vec::with_capacity(count);
        let mut factions = Vec::with_capacity(count);
        for index in 0..count {
            let raw = rng::draw(config.seed, rng::SYSTEM_TILE_STUB, 0, index as u64, 0);
            values.push(Fix32((raw >> 40) as i32));
            factions.push(FactionId((index as u64 % divisor) as u16));
        }
        let layout = BlockLayout::new(grid, BLOCK_BITS_DEFAULT)?;
        let soldiers = SoldierArena::new(grid);
        let settlements = SettlementArena::new(grid);
        let mut bridge = UnitTileBridge::new(layout);
        bridge.rebuild(&soldiers)?;
        let terrain = Terrain::new(config.seed, grid);
        let mut world = Self {
            config,
            grid,
            tick: Tick(0),
            values,
            factions,
            log: Vec::new(),
            soldiers,
            settlements,
            bridge,
            terrain,
            pyramid: Pyramid::new(layout, terrain)?,
        };
        // A world that has never stepped still answers a question about a
        // region. A level that nothing rebuilt would describe an empty world
        // and would be wrong rather than absent.
        world
            .pyramid
            .rebuild(&world.values, &world.soldiers, &world.bridge, 1)?;
        Ok(world)
    }

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
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub const fn terrain(&self) -> Terrain {
        self.terrain
    }

    /// Returns the terrain of one tile.
    ///
    /// Returns `None` when the address lies outside the world. The call
    /// computes the tile. It reads no array, so it never goes stale.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub fn tile_terrain(&self, address: Axial) -> Option<TerrainTile> {
        self.terrain.tile(address)
    }

    /// Returns the terrain kind of one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn tile_kind(&self, address: Axial) -> Option<TileKind> {
        self.terrain.kind(address)
    }

    /// Returns the soldiers of the world.
    ///
    /// The soldier is one of the four fixed entity shapes, and it has its
    /// own column set.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    #[must_use]
    pub const fn soldiers(&self) -> &SoldierArena {
        &self.soldiers
    }

    /// Returns the settlements of the world.
    ///
    /// The settlement is one of the four fixed entity shapes, and it has
    /// its own column set. It is fixed to a tile and it holds pooled
    /// stores.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    #[must_use]
    pub const fn settlements(&self) -> &SettlementArena {
        &self.settlements
    }

    /// Founds a settlement in the world and returns its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, when the faction is at or above the ceiling,
    /// or when another settlement already stands on the tile.
    pub fn found_settlement(
        &mut self,
        address: Axial,
        faction: FactionId,
    ) -> Result<Entity, SettlementError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a settlement of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SettlementError::FactionAboveCeiling(faction));
        }
        self.settlements.found(address, faction)
    }

    /// Destroys a settlement and reports whether it destroyed one.
    ///
    /// A stale identity destroys nothing and returns `false`. The identity
    /// of a destroyed settlement never resolves again, so the settlement
    /// founded next in that slot does not answer to it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn destroy_settlement(&mut self, entity: Entity) -> bool {
        self.settlements.destroy(entity)
    }

    /// Returns the settlement that stands on an address.
    ///
    /// Returns `None` when the address is outside the world, and `None`
    /// when no settlement stands there.
    #[must_use]
    pub fn settlement_on(&self, address: Axial) -> Option<Entity> {
        self.settlements.on_tile(address)
    }

    /// Writes the quantity of one commodity into the store of a settlement.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the commodity is outside the commodity set.
    pub fn set_settlement_store(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        quantity: Fix32,
    ) -> Result<bool, SettlementError> {
        self.settlements.set_store(entity, commodity, quantity)
    }

    /// Reports whether the ground at an address admits a unit.
    ///
    /// The answer is a property of the ground alone.[^1] It does not depend
    /// on the tick, on the faction, or on what already stands there. An
    /// address outside the world gives `false`, and the caller reports that
    /// refusal under its own name.
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub fn admits_a_unit(&self, address: Axial) -> bool {
        self.terrain
            .kind(address)
            .is_some_and(TileKind::is_passable)
    }

    /// Refuses an address that lies inside the world on ground that admits
    /// no unit.
    ///
    /// The extent refusal stays with the arena, which owns the grid. This
    /// call therefore says nothing about an address outside the world.
    fn refuse_impassable(&self, address: Axial) -> Result<(), SoldierError> {
        match self.terrain.kind(address) {
            Some(kind) if !kind.is_passable() => Err(SoldierError::TileImpassable(address)),
            _ => Ok(()),
        }
    }

    /// Adds a soldier to the world and returns its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, when the ground at the address admits no
    /// unit, or when the faction is at or above the ceiling.
    pub fn spawn_soldier(
        &mut self,
        address: Axial,
        faction: FactionId,
    ) -> Result<Entity, SoldierError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a soldier of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SoldierError::FactionAboveCeiling(faction));
        }
        self.refuse_impassable(address)?;
        self.soldiers.spawn(address, faction)
    }

    /// Removes a soldier and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn despawn_soldier(&mut self, entity: Entity) -> bool {
        self.soldiers.despawn(entity)
    }

    /// Moves a soldier to another tile.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the address is outside the world, or when the
    /// ground at the address admits no unit.
    pub fn place_soldier(&mut self, entity: Entity, address: Axial) -> Result<bool, SoldierError> {
        self.refuse_impassable(address)?;
        self.soldiers.place(entity, address)
    }

    /// Returns the unit-to-tile bridge.
    ///
    /// The bridge is derived from the soldier columns, and it rebuilds at the
    /// frame barrier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub const fn bridge(&self) -> &UnitTileBridge {
        &self.bridge
    }

    /// Returns the soldiers that stand on one tile.
    ///
    /// The call reads the block range, then searches inside it. It scans no
    /// population.[^1]
    ///
    /// The answer is the occupancy as it stood at the last barrier. A spawn,
    /// a despawn or a move since then makes the bridge stale, and the call
    /// then returns an error rather than a wrong answer.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge is stale, or when the address is
    /// outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    pub fn soldiers_on(&self, address: Axial) -> Result<&[Entity], BridgeError> {
        self.bridge.on_tile(&self.soldiers, address)
    }

    /// Returns the number of soldiers that stand on one tile.
    ///
    /// # Errors
    ///
    /// Returns an error for the same reasons that [`Self::soldiers_on`] does.
    pub fn soldier_count_on(&self, address: Axial) -> Result<usize, BridgeError> {
        self.bridge.count_on_tile(&self.soldiers, address)
    }

    /// Rebuilds the unit-to-tile bridge from the soldier columns.
    ///
    /// The step calls this at the barrier. A caller that changes the
    /// population outside a step calls it to make the bridge readable
    /// again.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, or when the
    /// rebuild refuses.
    pub fn rebuild_bridge(&mut self, threads: usize) -> Result<(), StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }
        self.bridge.rebuild(&self.soldiers)?;
        Ok(())
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
        self.values.get(index.0 as usize).copied()
    }

    /// Returns the settings that built the world.
    #[must_use]
    pub const fn config(&self) -> WorldConfig {
        self.config
    }

    /// Returns the current tick.
    #[must_use]
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    /// Returns the number of tiles.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.values.len()
    }

    /// Returns the whole tile value column.
    ///
    /// The column is one flat array, so the view costs no copy.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn tile_values(&self) -> &[Fix32] {
        &self.values
    }

    /// Returns the events of the last step.
    #[must_use]
    pub fn event_log(&self) -> &[TileChanged] {
        &self.log
    }

    /// Returns the events of the last step as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn event_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.log)
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
        let mut total = Accum(0);
        for value in &self.values {
            total = sim_math::accumulate(total, *value);
        }
        total
    }

    /// Returns the hash of the whole state.
    ///
    /// The golden test compares this value against a stored file.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let hash = StateHash::new()
            .write_u64(self.tick.0)
            .write_u64(self.config.seed)
            .write_u64(u64::from(self.config.width))
            .write_u64(u64::from(self.config.height))
            .write_u64(u64::from(self.config.faction_count))
            .write(bytemuck::cast_slice(&self.values))
            .write(bytemuck::cast_slice(&self.factions));
        // The ground is part of the world, so the whole-world hash covers
        // it. The seed and the extent are already above, but they are the
        // inputs of the generator, not its output. A change to the generator
        // moves every tile of every world, and only the tiles report it.[^1]
        //
        // [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.terrain.hash_into(hash);
        let hash = self.soldiers.hash_into(hash);
        self.settlements.hash_into(hash)
    }

    /// Reports whether the world holds its invariants.
    ///
    /// The Python state machine calls this method after every rule.[^1]
    ///
    /// # References
    ///
    /// [^1]: The testing rule, drive the real caller. `.claude/rules/testing.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.values.len() != self.factions.len() {
            return false;
        }
        if self.values.len() != self.grid.tile_count() as usize {
            return false;
        }
        if self.grid.width() != self.config.width || self.grid.height() != self.config.height {
            return false;
        }
        let ceiling = self.config.faction_count.max(1);
        if self.factions.iter().any(|faction| faction.0 >= ceiling) {
            return false;
        }
        // The soldier faction column is a second population under the same
        // ceiling. Checking one and not the other let the test suite spawn
        // soldiers of factions the world did not have, and pass.
        if self
            .soldiers
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        // The arena holds a copy of the grid. A check must fail when the two
        // copies disagree.
        if self.soldiers.grid() != self.grid {
            return false;
        }
        // Level 1 covers the world once, and at a barrier it counts the
        // population the arena holds.
        //
        // The full equality between a level and the level below is a sweep of
        // every tile, and this check runs after every rule the control plane
        // applies, so it reads the totals instead. The equality itself is a
        // test.[^4]
        //
        // The tile total holds at every moment, because the ground does not
        // change. The unit total holds at a barrier only: a spawn made between
        // two frames leaves the level as stale as the structure it was built
        // from, which is the documented state and not a defect. The freshness
        // of the derived structure is what says which moment this is.
        //
        // [^4]: ADR-0023, an aggregate combines exactly, in any order, decision D5, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
        let total = self.pyramid.total();
        if total.tiles() != i64::from(self.grid.tile_count()) {
            return false;
        }
        if self.bridge.describes(&self.soldiers).is_ok()
            && total.units() != i64::from(self.soldiers.len())
        {
            return false;
        }

        // No soldier stands on ground that admits no unit. The spawn, the
        // placement and the movement each refuse such a tile, and this check
        // is what fails when a later path forgets to.[^1]
        //
        // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        if self
            .soldiers
            .iter()
            .filter_map(|soldier| self.soldiers.address(soldier))
            .any(|address| !self.admits_a_unit(address))
        {
            return false;
        }
        // The terrain holds a second copy of the seed and of the extent. One
        // value declared twice needs a check that fails when the copies
        // disagree, because a silently wrong copy reads back correctly and
        // changes the whole world.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.terrain.seed() != self.config.seed || self.terrain.grid() != self.grid {
            return false;
        }
        if !self.soldiers.check_invariants() {
            return false;
        }
        // The settlement arena holds a copy of the grid, and its faction
        // column stands under the same ceiling as every other faction
        // column. A check must fail when a copy disagrees.[^1]
        //
        // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
        if self.settlements.grid() != self.grid {
            return false;
        }
        if self
            .settlements
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        if !self.settlements.check_invariants() {
            return false;
        }
        // The bridge is a second declaration of where a soldier stands, and
        // the tile column is the first. The check fails when the two
        // disagree.[^1] A stale bridge cannot be compared against columns it
        // was not derived from, so the structure check stands alone there.
        //
        // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
        if !self.bridge.check_structure() {
            return false;
        }
        if self.bridge.layout().grid() != self.grid {
            return false;
        }
        match self.bridge.check_invariants(&self.soldiers) {
            Ok(held) => {
                if !held {
                    return false;
                }
            }
            Err(BridgeError::Stale { .. }) => {}
            Err(_) => return false,
        }
        self.log
            .iter()
            .all(|event| event.padding == [0; 5] && (event.tile.0 as usize) < self.values.len())
    }

    /// Runs one frame on the given number of threads.
    ///
    /// The result does not depend on the thread count. Every thread writes
    /// to its own output slot, and the step joins the slots in slot order.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    pub fn step(&mut self, threads: usize) -> Result<&[TileChanged], StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }

        self.tick = Tick(self.tick.0.wrapping_add(1));

        let tick = self.tick;
        let seed = self.config.seed;
        let values = &mut self.values;
        let factions = &self.factions[..];
        let count = values.len();
        let chunk_len = count.div_ceil(threads).max(1);

        let mut slots: Slots<Vec<TileChanged>> =
            Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

        std::thread::scope(|scope| {
            let mut base = 0usize;
            for (chunk, slot) in values.chunks_mut(chunk_len).zip(slots.entries_mut()) {
                let start = base;
                base += chunk.len();
                let owned_factions = &factions[start..base];
                scope.spawn(move || {
                    *slot = update_chunk(tick, seed, start, chunk, owned_factions);
                });
            }
        });

        let mut log = core::mem::take(&mut self.log);
        log.clear();
        self.log = slots.combine(log, |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });

        // Every soldier chooses a neighbour, then the step applies the
        // choices. The choice is a pure read of the world, so the two halves
        // never interleave and no soldier sees a half-applied world.[^1]
        // A spawn or a despawn made between two frames is a structural change
        // that has not passed a barrier, and it leaves the derived structure
        // stale. Admission reads the occupancy of a target from that
        // structure, so the step opens by giving those changes their
        // barrier.[^4]
        //
        // This is not a second barrier. The rebuild at the end of this
        // function is the barrier of this frame, and it stays last.
        self.refresh_bridge()?;

        let intents = soldier_moves(tick, seed, self.terrain, &self.soldiers, threads)?;

        // Admission grants the intents. It reads the occupancy of a target
        // from the derived structure, which the last barrier rebuilt, so it
        // must run before anything moves.[^3]
        let granted = admit(
            &intents,
            &self.soldiers,
            &self.bridge,
            self.terrain,
            self.grid,
            threads,
        )?;
        for (soldier, address) in granted {
            self.soldiers
                .place(soldier, address)
                .expect("the granted address is inside the world and admits a unit");
        }

        // The bridge rebuilds here, at the barrier, and after the structural
        // apply. Rebuilding before the apply would leave a dead identity in
        // the unit array for the whole frame.[^2] The movement above is the
        // structural apply, so this call stays last in the step.
        //
        // [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        self.refresh_bridge()?;

        // Level 1 rebuilds after the structure it reads, and after every
        // change to level 0 that this frame made. It is derived, so it is
        // last.[^5]
        //
        // The rebuild is called here rather than through the public wrapper,
        // because the wrapper refreshes the structure first and the barrier
        // above has already done that. Two refreshes would be one decision in
        // two places, and the second would hide a rebuild that ran in the
        // wrong order: a structure left stale by a barrier out of order would
        // be quietly repaired instead of refused.
        //
        // [^5]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        self.pyramid
            .rebuild(&self.values, &self.soldiers, &self.bridge, threads)?;
        Ok(&self.log)
    }

    /// Returns level 1 of the pyramid.
    ///
    /// The level is derived from level 0 and holds no fact of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    #[must_use]
    pub const fn pyramid(&self) -> &Pyramid {
        &self.pyramid
    }

    /// Returns the level 1 summary of the cell that covers one tile.
    #[must_use]
    pub fn summary_covering(&self, address: Axial) -> Option<CellSummary> {
        self.pyramid.cell_covering(address)
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
        self.pyramid
            .rebuild(&self.values, &self.soldiers, &self.bridge, threads)?;
        Ok(())
    }

    /// Rebuilds the derived structure when it no longer describes the arena.
    ///
    /// One rule governs both rebuild sites in the step: rebuild when the
    /// arena has moved since the last rebuild, and not otherwise. The
    /// structure holds the revision it was built from, so the test is one
    /// comparison and it reads no unit.
    ///
    /// **A frame in which no unit moved rebuilds nothing.** A structure that
    /// already describes the arena is the structure a rebuild would produce,
    /// so skipping it is not an optimisation that trades a guarantee. The
    /// record sanctions a rebuild each frame and argues from the merge order
    /// of incremental writes rather than from frequency, so it neither
    /// requires the rebuild nor forbids the test.[^1]
    ///
    /// A crowd is where this pays. A unit whose every target is full is
    /// refused every frame, and a world in which nothing was admitted leaves
    /// the arena untouched.
    ///
    /// # Errors
    ///
    /// Returns an error when the rebuild refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    fn refresh_bridge(&mut self) -> Result<(), StepError> {
        if self.bridge.describes(&self.soldiers).is_ok() {
            return Ok(());
        }
        self.bridge.rebuild(&self.soldiers)?;
        Ok(())
    }
}

/// The draw index of the movement direction.
///
/// The movement system takes one draw for each soldier in each frame. A
/// second draw in the same system and frame must take the next index.
const DRAW_MOVE_DIRECTION: u32 = 0;

/// Returns the move that each live soldier chose, in slot order.
///
/// Each soldier draws one direction from the counter-based generator. The
/// key is the tuple of the system, the frame, the entity and the draw
/// index, so the same soldier in the same frame gets the same direction
/// however the work was scheduled.[^1] The key holds the entity identity,
/// which pairs the slot index with the generation, and not the slot index
/// alone.[^2]
///
/// A soldier whose chosen neighbour falls outside the world stays put. The
/// world is a rhombus and it does not wrap, so an address outside the
/// extent names no tile.[^3]
///
/// A soldier whose chosen neighbour holds ground that admits no unit also
/// stays put.[^6] This refusal belongs to the intent half. The ground refuses
/// every unit on every frame, whatever else stands there, so the intent never
/// reaches admission and the soldier takes no lateral step. A tile that is
/// full is a different refusal, and admission owns it.[^5]
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the step joins the slots in slot order. The result never
/// depends on thread completion order.[^4]
///
/// This is the intent half of movement only. A separate step admits the
/// intents against the capacity of each target, and it may refuse any of
/// them.[^5]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
fn soldier_moves(
    tick: Tick,
    seed: u64,
    terrain: Terrain,
    soldiers: &SoldierArena,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    let live: Vec<Entity> = soldiers.iter().collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let grid = soldiers.grid();
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<(Entity, Axial)>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    std::thread::scope(|scope| {
        for (chunk, slot) in live.chunks(chunk_len).zip(slots.entries_mut()) {
            scope.spawn(move || {
                *slot = chunk
                    .iter()
                    .filter_map(|soldier| {
                        let here = soldiers.address(*soldier)?;
                        let direction = rng::draw_below(
                            seed,
                            rng::SYSTEM_SOLDIER_MOVE,
                            tick.0,
                            soldier.to_bits(),
                            DRAW_MOVE_DIRECTION,
                            NEIGHBOUR_COUNT as u64,
                        ) as usize;
                        // A neighbour outside the world gives `None`. The
                        // soldier then stays put, because the world does not
                        // wrap.
                        let target = grid.neighbour(here, direction)?;
                        // The ground refuses the soldier outright. The
                        // soldier stays put, and nothing later in the frame
                        // sees the intent.
                        if !terrain.kind(target)?.is_passable() {
                            return None;
                        }
                        Some((*soldier, target))
                    })
                    .collect();
            });
        }
    });

    Ok(slots.combine(Vec::new(), |mut joined, slot| {
        joined.extend_from_slice(slot);
        joined
    }))
}

/// The number of admission passes that one frame runs.
///
/// Each pass admits what it can against the room the previous pass
/// confirmed. The engine never runs to a fixpoint, because a fixpoint needs
/// a convergence test and a solver in this project runs a fixed count.[^1]
///
/// The count is content. It is declared here until content exists, and the
/// register holds the open choice of its value.[^2]
///
/// One pass admits no chain: a unit cannot follow another out of a full
/// tile in the same frame. Two passes admit a chain of two. A longer chain
/// waits for the next frame, which is a delay and never a wrong answer.
///
/// # References
///
/// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
/// [^2]: Decisions register, DEC-019. `docs/DECISIONS.md`
const ADMISSION_PASSES: usize = 2;

/// A running count for each tile that the admission touched.
///
/// The tiles are held sorted by index, so a lookup is a binary search and the
/// order never depends on how the counts were gathered. A dense array over
/// every tile would be faster to update and would cost the whole world in
/// memory for a frame that touches a handful of tiles.[^1]
///
/// A count is merged in ascending runs, never inserted one at a time.
/// Inserting into the middle of a vector moves every later entry, which is
/// quadratic in the number of tiles the frame touches, and the target scale
/// is a million units.[^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
#[derive(Debug, Default)]
struct TileCounts {
    entries: Vec<(u32, u32)>,
    scratch: Vec<(u32, u32)>,
}

impl TileCounts {
    /// Returns the count that one tile carries.
    fn get(&self, tile: u32) -> u32 {
        match self.entries.binary_search_by_key(&tile, |(key, _)| *key) {
            Ok(at) => self.entries[at].1,
            Err(_) => 0,
        }
    }

    /// Adds a run of counts, given in ascending tile order.
    ///
    /// The caller states the order and the merge relies on it. A run out of
    /// order would silently produce an unsorted result, and every later
    /// lookup would then read the wrong tile, so the merge asserts it.
    fn merge_ascending(&mut self, run: &[(u32, u32)]) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "a merged run must be sorted by tile and hold each tile once"
        );
        if run.is_empty() {
            return;
        }
        self.scratch.clear();
        self.scratch.reserve(self.entries.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.entries.len() && there < run.len() {
            let (mine, theirs) = (self.entries[here], run[there]);
            if mine.0 < theirs.0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 < mine.0 {
                self.scratch.push(theirs);
                there += 1;
            } else {
                self.scratch.push((mine.0, mine.1 + theirs.1));
                here += 1;
                there += 1;
            }
        }
        self.scratch.extend_from_slice(&self.entries[here..]);
        self.scratch.extend_from_slice(&run[there..]);
        core::mem::swap(&mut self.entries, &mut self.scratch);
    }
}

/// Returns the order in which admission reads the intents.
///
/// The order is the key vector sort: by target tile, then by the identity of
/// the unit.[^1] It depends on the key values alone, so it is the same at any
/// thread count.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[cfg(not(feature = "probe-nondeterminism"))]
fn admission_order(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    sort::order_bounded(keys, ceiling)
}

/// Returns the intents in the order they arrived, which is a defect.
///
/// This is the perturbed build. Admission reads the joined intent list rather
/// than the sorted one, so who enters a full tile depends on the order the
/// slots were joined in. The slot probe reverses that order, and the reversal
/// is visible only above one thread, so the thread-count test then fails.
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^1]
///
/// # Errors
///
/// Never. The signature matches the sound build so that the caller does not
/// change.
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
fn admission_order(keys: &[BoundedKey], _ceiling: u64) -> Result<Vec<u32>, SortError> {
    // A stable sort by the target alone. Each target still owns one
    // contiguous segment, which admission requires to scan a segment at all,
    // and within a segment the order is the order the intents arrived in.
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by_key(|position| keys[*position as usize].order());
    Ok(order)
}

/// One target tile and the run of intents that name it.
///
/// The table is built once for a frame. The capacity and the occupancy are
/// read once for each target rather than once for each pass, because the
/// ground is computed on demand and reading it twice computes it twice.[^1]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
#[derive(Clone, Copy, Debug)]
struct Segment {
    /// The target tile that owns the segment.
    tile: u32,
    /// The first sorted position of the segment.
    start: usize,
    /// One past the last sorted position of the segment.
    end: usize,
    /// The units the ground of the target admits.
    capacity: u32,
    /// The units that stood on the target at the last barrier.
    standing: u32,
}

/// Adds one to the last entry of an ascending run, or starts a new one.
///
/// The caller visits the tiles in ascending order, so a repeat is always the
/// last entry.
fn bump(run: &mut Vec<(u32, u32)>, tile: u32) {
    match run.last_mut() {
        Some(last) if last.0 == tile => last.1 += 1,
        _ => run.push((tile, 1)),
    }
}

/// Returns the intents that admission granted, in the sorted admission order.
///
/// Admission sorts the intents by target tile, then by the identity of the
/// unit. Each target tile then owns one contiguous segment, and the identity
/// is the final key field so no two intents tie.[^1] The sort is the engine's
/// key vector sort, and it runs on one thread, so no result here takes its
/// order from a thread that finished first.[^2]
///
/// Admission scans each segment in its sorted order and admits until the
/// target reaches the capacity of its ground. The capacity comes from the
/// terrain table. This function holds no capacity value of its own.[^3]
///
/// **The occupancy of a target comes from the derived structure**, which the
/// barrier rebuilt before the intents were drawn.[^4] Admission carries no
/// dense array over every tile.
///
/// **Only an admitted departure releases room.** An intent is not a
/// departure. A unit that intends to leave and is then rejected at its own
/// target has not left, and the room it appeared to release was never
/// released. Take three tiles in a line, with the middle and the far tile
/// both full. The unit in the middle is rejected at the far tile. A rule that
/// counted its intent would admit the unit behind it into the middle tile,
/// and the middle tile would end the tick above its capacity.[^1]
///
/// That failure is deterministic, so neither determinism test can see it.
/// Only a test that asserts the capacity invariant can.[^5]
///
/// **A departure is applied after the scan, not inside it.** The segments are
/// disjoint by target, so the room of a target is read once for each segment.
/// The units leaving one tile are scattered across many segments, because
/// they chose different targets, so the departures are a separate reduction
/// over the admitted set, keyed on the source tile.[^1]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys, or when the derived
/// structure cannot answer for a tile.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^7]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak, a draft record. `docs/adrs/draft/adr-0009-parallel-stages-write-disjoint-outputs.md`
fn admit(
    intents: &[(Entity, Axial)],
    soldiers: &SoldierArena,
    bridge: &UnitTileBridge,
    terrain: Terrain,
    grid: Grid,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    if intents.is_empty() {
        return Ok(Vec::new());
    }

    // The ordering field is the target tile index and the identifier is the
    // entity. One unit writes one intent, so no two identifiers collide.
    let mut keys = Vec::with_capacity(intents.len());
    for (entity, target) in intents {
        let index = grid
            .index_of(*target)
            .ok_or(StepError::TargetOutsideWorld)?;
        keys.push(BoundedKey::new(u64::from(index.0), entity.to_bits()));
    }
    let ceiling = u64::from(grid.tile_count().saturating_sub(1));
    let order = admission_order(&keys, ceiling)?;

    // The sorted intents, as a tile beside the intent it belongs to. The
    // passes walk this rather than following the permutation into the keys,
    // so a pass reads its tiles in order rather than at random.
    let sorted: Vec<(u32, u32)> = order
        .iter()
        .map(|position| (keys[*position as usize].order() as u32, *position))
        .collect();

    // The segment table, built once. Each target owns one contiguous segment,
    // and the segments are disjoint.[^1]
    //
    // The capacity and the occupancy are read here and not inside the passes.
    // The ground is a pure function of the seed and the address, so reading
    // it twice computes it twice, and the record calls a repeated sweep of
    // the ground a design mistake.[^6] The occupancy comes from the structure
    // the last barrier built, and that answer does not change during the
    // frame either: what changes is the arrivals and the departures this
    // admission grants, and those are counted separately.
    let mut segments: Vec<Segment> = Vec::new();
    let mut at = 0usize;
    while at < sorted.len() {
        let tile = sorted[at].0;
        let mut end = at;
        while end < sorted.len() && sorted[end].0 == tile {
            end += 1;
        }
        segments.push(Segment {
            tile,
            start: at,
            end,
            capacity: 0,
            standing: 0,
        });
        at = end;
    }

    // The capacity and the occupancy of each target are read in parallel.
    // Each thread writes its own chunk of the table, and a chunk is named by
    // its position in the table rather than by the thread that filled it, so
    // the result never depends on which thread finished first.[^7]
    let chunk_len = segments.len().div_ceil(threads).max(1);
    let mut refusal: Option<BridgeError> = None;
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in segments.chunks_mut(chunk_len) {
            handles.push(scope.spawn(move || {
                for segment in chunk.iter_mut() {
                    let Some(address) = grid.address_of(TileIdx(segment.tile)) else {
                        // The tile came from a key the sort built out of a
                        // grid index, so it names a tile. An address that
                        // does not resolve is a defect in the caller and the
                        // whole step refuses below.
                        continue;
                    };
                    segment.capacity = terrain
                        .kind(address)
                        .map_or(0, crate::terrain::TileKind::capacity);
                    match bridge.count_on_tile(soldiers, address) {
                        Ok(standing) => segment.standing = standing as u32,
                        Err(error) => return Err(error),
                    }
                }
                Ok(())
            }));
        }
        for handle in handles {
            if let Ok(Err(error)) = handle.join() {
                refusal.get_or_insert(error);
            }
        }
    });
    if let Some(error) = refusal {
        return Err(error.into());
    }

    let mut granted = vec![false; intents.len()];
    let mut arrived = TileCounts::default();
    let mut departed = TileCounts::default();
    let mut admitted: Vec<(Entity, Axial)> = Vec::new();

    for _ in 0..ADMISSION_PASSES {
        let first = admitted.len();
        // The segments come in ascending tile order, so the arrivals of one
        // pass are already an ascending run.
        let mut arrivals: Vec<(u32, u32)> = Vec::new();

        for segment in &segments {
            // A departure only ever leaves a tile a unit stood on, so the
            // subtraction cannot go below zero. It saturates rather than
            // wrapping, because a wrap here would read as a full tile and
            // reject every unit in silence.
            let occupancy = segment.standing.saturating_sub(departed.get(segment.tile))
                + arrived.get(segment.tile);
            let mut room = segment.capacity.saturating_sub(occupancy);
            if room == 0 {
                continue;
            }

            for (_, position) in &sorted[segment.start..segment.end] {
                if room == 0 {
                    break;
                }
                let position = *position as usize;
                if granted[position] {
                    continue;
                }
                granted[position] = true;
                admitted.push(intents[position]);
                bump(&mut arrivals, segment.tile);
                room -= 1;
            }
        }
        arrived.merge_ascending(&arrivals);

        // The scan is over. Only now does a departure release room, and only
        // an admitted one. The reduction is keyed on the source tile.
        let mut sources: Vec<u32> = admitted[first..]
            .iter()
            .filter_map(|(entity, _)| soldiers.tile(*entity))
            .map(|tile| tile.0)
            .collect();
        if sources.is_empty() {
            // Nothing moved in this pass, so no later pass can move anything.
            break;
        }
        sources.sort_unstable();
        let mut departures: Vec<(u32, u32)> = Vec::new();
        for tile in sources {
            bump(&mut departures, tile);
        }
        departed.merge_ascending(&departures);
    }

    Ok(admitted)
}

/// Updates one contiguous chunk of tiles and returns the events it emitted.
///
/// The function is pure in the sense that the record requires: the same
/// prior values and the same key give the same result.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
fn update_chunk(
    tick: Tick,
    seed: u64,
    start: usize,
    values: &mut [Fix32],
    factions: &[FactionId],
) -> Vec<TileChanged> {
    let mut events = Vec::new();
    for (offset, value) in values.iter_mut().enumerate() {
        let index = start + offset;
        let raw = rng::draw_below(seed, rng::SYSTEM_TILE_STUB, tick.0, index as u64, 0, 8);
        if raw >= 4 {
            continue;
        }
        let delta = Fix32((raw as i32) - 2);
        if delta.0 == 0 {
            continue;
        }
        let updated = sim_math::add(*value, delta);
        *value = updated;
        let kind = if delta.0 > 0 {
            CHANGE_KIND_RAISED
        } else {
            CHANGE_KIND_LOWERED
        };
        events.push(TileChanged::new(
            tick,
            TileIdx(index as u32),
            updated,
            factions[offset],
            kind,
        ));
    }
    events
}

#[cfg(test)]
mod tests {
    //! Unit tests for the state that the public API cannot reach.
    //!
    //! The testing policy allows a unit test where a test cannot observe
    //! the case through the public interface. The public API cannot build
    //! a world that breaks its own invariants, so a test of the invariant
    //! check must build one here.[^1]
    //!
    //! # References
    //!
    //! [^1]: Testing policy, section 2. `docs/TESTING.md`

    use super::*;

    /// Builds a world with a mismatched column length.
    fn broken(change: impl FnOnce(&mut World)) -> World {
        let mut world = World::new(WorldConfig {
            width: 4,
            height: 2,
            seed: 1,
            faction_count: 2,
        })
        .expect("the extent must describe a world");
        change(&mut world);
        world
    }

    #[test]
    fn a_sound_world_holds_its_invariants() {
        assert!(broken(|_| {}).check_invariants());
    }

    #[test]
    fn a_short_faction_column_fails_the_check() {
        assert!(!broken(|world| {
            world.factions.pop();
        })
        .check_invariants());
    }

    #[test]
    fn a_short_value_column_fails_the_check() {
        assert!(!broken(|world| {
            world.values.pop();
            world.factions.pop();
        })
        .check_invariants());
    }

    #[test]
    fn a_faction_above_the_ceiling_fails_the_check() {
        assert!(!broken(|world| {
            world.factions[0] = FactionId(2);
        })
        .check_invariants());
        // The ceiling is exclusive. The highest valid identifier passes.
        assert!(broken(|world| {
            world.factions[0] = FactionId(1);
        })
        .check_invariants());
    }

    #[test]
    fn an_event_with_padding_fails_the_check() {
        assert!(!broken(|world| {
            let mut event = TileChanged::new(Tick(1), TileIdx(0), Fix32::ZERO, FactionId(0), 1);
            event.padding[0] = 1;
            world.log.push(event);
        })
        .check_invariants());
    }

    #[test]
    fn an_event_that_names_no_tile_fails_the_check() {
        assert!(!broken(|world| {
            world.log.push(TileChanged::new(
                Tick(1),
                TileIdx(8),
                Fix32::ZERO,
                FactionId(0),
                1,
            ));
        })
        .check_invariants());
        // The bound is exclusive. The highest valid index passes.
        assert!(broken(|world| {
            world.log.push(TileChanged::new(
                Tick(1),
                TileIdx(7),
                Fix32::ZERO,
                FactionId(0),
                1,
            ));
        })
        .check_invariants());
    }
}
