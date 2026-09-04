//! The counter-based random number generator.
//!
//! Every draw is keyed on the tuple of system, frame, entity and draw index.
//! The generator maps that key to a value. It holds no state.
//!
//! Thread-local generator state is forbidden. Such state makes the result
//! depend on which thread served which entity, and that is exactly what the
//! schedule must not control.[^1]
//!
//! The mixer is written in the project with known-answer tests, because the
//! available crates are not maintained at the level this warrants.[^1]
//!
//! # References
//!
//! [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`

/// The identifier of the system that draws the value.
///
/// Each system owns one constant. Two systems must never share one, because
/// a shared identifier makes two draws correlate.
pub type SystemId = u32;

/// The system identifier of the stub tile system.
pub const SYSTEM_TILE_STUB: SystemId = 1;

/// The system identifier of the soldier movement system.
///
/// The movement system owns this identifier alone. It does not share the
/// identifier of the tile system, because a shared identifier makes the two
/// draws correlate.
pub const SYSTEM_SOLDIER_MOVE: SystemId = 2;

/// The system identifier of the terrain generator.
///
/// The terrain generator owns this identifier alone. It shares it with no
/// other system, because two systems that share an identifier draw the same
/// value from the same frame, entity and draw index.
pub const SYSTEM_TERRAIN: SystemId = 3;

/// The system identifier of the resource field.
///
/// The resource field owns this identifier alone. It does not share the
/// identifier of the terrain generator, because two systems that share an
/// identifier draw the same value from the same frame, entity and draw index.
/// The stock of a tile would then follow the height of that tile exactly.
pub const SYSTEM_RESOURCE: SystemId = 4;

/// Returns the value for one draw.
///
/// The arguments are the world seed, the system, the frame, the entity, and
/// the draw index within the frame. The same arguments always give the same
/// value.
#[must_use]
pub const fn draw(seed: u64, system: SystemId, frame: u64, entity: u64, index: u32) -> u64 {
    let mut state = seed;
    state = mix(state ^ (system as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    state = mix(state ^ frame.wrapping_mul(0xbf58_476d_1ce4_e5b9));
    state = mix(state ^ entity.wrapping_mul(0x94d0_49bb_1331_11eb));
    mix(state ^ (index as u64).wrapping_mul(0xd6e8_feb8_6659_fd93))
}

/// Returns a value in the half-open range from zero to the bound.
///
/// Returns zero when the bound is zero. The reduction is a widening
/// multiply, which is exact and holds no bias that matters at this scale.
#[must_use]
pub const fn draw_below(
    seed: u64,
    system: SystemId,
    frame: u64,
    entity: u64,
    index: u32,
    bound: u64,
) -> u64 {
    if bound == 0 {
        return 0;
    }
    let value = draw(seed, system, frame, entity, index);
    ((value as u128 * bound as u128) >> 64) as u64
}

/// The mixing function. It is the finaliser of SplitMix64.
const fn mix(value: u64) -> u64 {
    let mut state = value;
    state ^= state >> 30;
    state = state.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

/// The system identifier of the character tier.
///
/// The character tier owns this identifier alone. Every character draw keys
/// on it, and a birth draw keys on the mother because the child holds no
/// identity when the draw happens.
pub const SYSTEM_CHARACTER: SystemId = 6;

/// The system identifier of the consumption pass.
///
/// The pass owns this identifier alone. Two systems that share an identifier
/// draw the same value from the same frame, entity and draw index, so a unit
/// would then be fed and moved on one number.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub const SYSTEM_CONSUMPTION: SystemId = 7;

/// The system identifier of the founding.
///
/// The founding owns this identifier alone. It does not share the identifier
/// of the terrain generator or of the resource field, because two systems
/// that share an identifier draw the same value from the same frame, entity
/// and draw index. The sample would then sit on the lattice of the ground.
pub const SYSTEM_FOUNDING: SystemId = 5;

/// The system identifier of the contest that resolves a meeting.
///
/// The contest owns this identifier alone. It does not share the identifier
/// of the movement system, because two systems that share an identifier draw
/// the same value from the same frame, entity and draw index. The units that
/// fell on a tile would then follow the step that the units of that tile
/// took.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub const SYSTEM_CONTEST: SystemId = 8;

/// The system identifier of the conversion pass.
///
/// The conversion pass owns this identifier alone. It does not share the
/// identifier of the contest, because two systems that share an identifier
/// draw the same value from the same frame, entity and draw index. The units
/// that changed faction on a tile would then follow the units that fell on
/// that tile.[^1]
/// The system identifier of the weather field.
///
/// The weather field owns this identifier alone. It does not share the
/// identifier of the terrain generator, because two systems that share an
/// identifier draw the same value from the same frame, entity and draw index.
/// The sea over a cell would then lift exactly where the ground is high.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub const SYSTEM_CONVERSION: SystemId = 9;
pub const SYSTEM_WEATHER: SystemId = 10;
