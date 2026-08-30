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
//! [^1]: ADR-0001, Determinism as the primary constraint, decision D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
//! [^2]: ADR-0001, Determinism as the primary constraint, decision D6. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
//! [^3]: ADR-0002, Target platform and value types, decision D5. `docs/adrs/draft/adr-0002-value-types-are-exact-and-sized-for-one-target.md`
//! [^4]: ADR-0001, Determinism as the primary constraint, decision D5. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
//! [^5]: ADR-0001, Determinism as the primary constraint, decision D3. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`

use crate::event::{TileChanged, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};
use crate::hash::StateHash;
use crate::rng;
use crate::sim_math;
use crate::types::{Accum, FactionId, Fix32, Tick, TileIdx};

/// The reason that a step refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    /// The caller asked for zero threads. A step needs at least one.
    ZeroThreads,
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
    /// The number of tiles in the world.
    pub tile_count: u32,
    /// The world seed. Every random draw takes it.
    pub seed: u64,
    /// The number of factions.
    pub faction_count: u16,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            tile_count: 4096,
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
/// [^1]: ADR-0006, The Python boundary, decision D10. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tick: Tick,
    values: Vec<Fix32>,
    factions: Vec<FactionId>,
    log: Vec<TileChanged>,
}

impl World {
    /// Builds a world from the settings.
    #[must_use]
    pub fn new(config: WorldConfig) -> Self {
        let count = config.tile_count as usize;
        let divisor = u64::from(config.faction_count.max(1));
        let mut values = Vec::with_capacity(count);
        let mut factions = Vec::with_capacity(count);
        for index in 0..count {
            let raw = rng::draw(config.seed, rng::SYSTEM_TILE_STUB, 0, index as u64, 0);
            values.push(Fix32((raw >> 40) as i32));
            factions.push(FactionId((index as u64 % divisor) as u16));
        }
        Self {
            config,
            tick: Tick(0),
            values,
            factions,
            log: Vec::new(),
        }
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
    /// [^1]: ADR-0006, The Python boundary, decision D5. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
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
    /// [^1]: ADR-0001, Determinism as the primary constraint, decision D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
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
    /// [^1]: ADR-0001, Determinism as the primary constraint, decisions D4 and D7. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
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
    /// [^1]: ADR-0001, Determinism as the primary constraint, decision D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        StateHash::new()
            .write_u64(self.tick.0)
            .write_u64(self.config.seed)
            .write_u64(u64::from(self.config.tile_count))
            .write_u64(u64::from(self.config.faction_count))
            .write(bytemuck::cast_slice(&self.values))
            .write(bytemuck::cast_slice(&self.factions))
    }

    /// Reports whether the world holds its invariants.
    ///
    /// The Python state machine calls this method after every rule.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, The Python boundary, decision D11. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.values.len() != self.factions.len() {
            return false;
        }
        if self.values.len() != self.config.tile_count as usize {
            return false;
        }
        let ceiling = self.config.faction_count.max(1);
        if self.factions.iter().any(|faction| faction.0 >= ceiling) {
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

        let mut slots: Vec<Vec<TileChanged>> = vec![Vec::new(); threads];

        std::thread::scope(|scope| {
            let mut base = 0usize;
            for (chunk, slot) in values.chunks_mut(chunk_len).zip(slots.iter_mut()) {
                let start = base;
                base += chunk.len();
                let owned_factions = &factions[start..base];
                scope.spawn(move || {
                    *slot = update_chunk(tick, seed, start, chunk, owned_factions);
                });
            }
        });

        self.log.clear();
        for slot in ordered_slots(&slots) {
            self.log.extend_from_slice(slot);
        }
        Ok(&self.log)
    }
}

/// Returns the output slots in the order that the log joins them.
///
/// The order is slot order. Slot order is the order of the tiles, and it
/// does not depend on which thread finished first.[^1]
///
/// # References
///
/// [^1]: ADR-0001, Determinism as the primary constraint, decision D6. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
#[cfg(not(feature = "probe-nondeterminism"))]
fn ordered_slots(slots: &[Vec<TileChanged>]) -> impl Iterator<Item = &Vec<TileChanged>> {
    slots.iter()
}

/// Returns the output slots in reverse order.
///
/// This is the test-only switch. It breaks the ordering rule on purpose,
/// so that the determinism tests have a proven failure mode. Never build a
/// shipped artefact with this feature.
#[cfg(feature = "probe-nondeterminism")]
fn ordered_slots(slots: &[Vec<TileChanged>]) -> impl Iterator<Item = &Vec<TileChanged>> {
    slots.iter().rev()
}

/// Updates one contiguous chunk of tiles and returns the events it emitted.
///
/// The function is pure in the sense that the record requires: the same
/// prior values and the same key give the same result.[^1]
///
/// # References
///
/// [^1]: ADR-0001, Determinism as the primary constraint, decision D9. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
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
            tile_count: 8,
            seed: 1,
            faction_count: 2,
        });
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
