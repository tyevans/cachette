//! The world, and the frame step.
//!
//! This is a stub. It holds one tile field, one event type, and one system.
//! It exists so that the determinism harnesses have a subject before the
//! first solver is written.[^1] Replace the body of the step. Do not replace
//! the shape of it.
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
//! [^3]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/REGISTRY.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::event::{TileChanged, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, GridError};
use crate::rng;
use crate::sim_math;
use crate::slots::Slots;
use crate::soldier::{SoldierArena, SoldierError};
use crate::types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx};

/// The reason that a step refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    /// The caller asked for zero threads. A step needs at least one.
    ZeroThreads,
}

/// The reason that a world refused to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// The configured extent does not describe a grid.
    Grid(GridError),
}

impl core::fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Grid(error) => write!(formatter, "the world extent is not a grid: {error}"),
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
}

impl World {
    /// Builds a world from the settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid.
    pub fn new(config: WorldConfig) -> Result<Self, WorldError> {
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
        Ok(Self {
            config,
            grid,
            tick: Tick(0),
            values,
            factions,
            log: Vec::new(),
            soldiers: SoldierArena::new(grid),
        })
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

    /// Adds a soldier to the world and returns its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, or when the faction is at or above the
    /// ceiling.
    pub fn spawn_soldier(
        &mut self,
        address: Axial,
        faction: FactionId,
    ) -> Result<Entity, SoldierError> {
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
    /// Returns an error when the address is outside the world.
    pub fn place_soldier(&mut self, entity: Entity, address: Axial) -> Result<bool, SoldierError> {
        self.soldiers.place(entity, address)
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
        self.soldiers.hash_into(hash)
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
        // The arena holds a copy of the grid. A check must fail when the two
        // copies disagree.
        if self.soldiers.grid() != self.grid {
            return false;
        }
        if !self.soldiers.check_invariants() {
            return false;
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
        Ok(&self.log)
    }
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
