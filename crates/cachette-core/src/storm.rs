//! What a storm takes from the world it passes over.
//!
//! # What this holds
//!
//! The weather field carries a pressure deficit over each cell, and a storm
//! is the cone that puts one there.[^1] This module holds the arithmetic that
//! turns that deficit into harm, and the one event type that records a unit
//! the storm took. It holds no state of its own, because the deficit is the
//! whole of the state a storm needs.
//!
//! # The three harms, and the one destruction
//!
//! A storm takes condition from a finished upgrade, it takes the food a tile
//! carries, and it takes units that stand in the open. Each of the three is a
//! rate against the deficit at the cell, so a deep eye takes more than a
//! shallow edge and a cell outside every storm takes nothing.
//!
//! **A storm destroys a site by driving its condition to nothing, and by no
//! other route.** There is no severity at which a site falls whatever its
//! condition. A faction that keeps a site in repair therefore survives a
//! storm that breaks a neglected one, and the repair loop is the counter-play
//! for the whole of this module.[^2]
//!
//! # What this module does not do
//!
//! **Nothing here creates water.** The ground under a storm gets wet because
//! the deficit cuts the capacity of the air above it, so the air pours what
//! it holds onto the ground. That is an exact move of water the weather field
//! already holds, and a second site that added drops would put water into the
//! world that no cell gave up.[^3]
//!
//! **Nothing here removes an upgrade.** The wear pass owns every removal, and
//! it reads the rate below. A second removal site would leave two statements
//! of when a site falls.[^4]
//!
//! # Determinism
//!
//! Every draw is keyed on the system, the frame, the whole identity of a unit
//! and a draw index, and nothing here holds generator state.[^5] The rates
//! are integer arithmetic through the arithmetic module, so no float reaches
//! simulated state.[^6]
//!
//! # References
//!
//! [^1]: The deficit cone. [`crate::weather::Cyclone::deficit_at`]
//! [^2]: The repair loop. [`crate::upgrade::repair_gain`]
//! [^3]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
//! [^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::rng;
use crate::sim_math;
use crate::types::{Accum, FactionId, Tick, TileIdx};
use crate::unit_type::UnitTypeId;
use crate::upgrade;
use crate::weather::CYCLONE_DEPTH_CEILING;

/// The whole that a storm chance is stated out of.
///
/// A chance is an integer out of this value, and a draw below this bound
/// decides it. Nothing here is a fraction and nothing here is a float.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub const CHANCE_WHOLE: i64 = 1024;

/// The deepest deficit that a rate here is stated against.
///
/// **This is the depth ceiling of one storm, and it has one declaration
/// site.** Two storms may overlap, and the stamp pass sums their deficits, so
/// a cell can carry more than this. Each rate clamps to it, so the harm of a
/// cell under two eyes is the harm of one eye and never twice it.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const DEPTH_WHOLE: i64 = CYCLONE_DEPTH_CEILING as i64;

/// The condition that one tick under the deepest eye takes off an upgrade.
///
/// The upgrade module states the full condition of a level, and this is a
/// share of it, so a site left in a deep storm falls after some tens of
/// ticks. A site under a shallow edge loses the same share of this, so the
/// rate falls to nothing one cell beyond the reach of the storm.
///
/// **This is a balance value and not a measurement.** The blocker that asks
/// what weather should be worth governs it, and the balance register holds
/// the row.[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: Balance register, the weather rows. `docs/reference/balance.md`
pub const WEAR_AT_FULL_DEPTH: i64 = 25_000;

/// The chance, out of [`CHANCE_WHOLE`], that one tick under the deepest eye
/// ends one unit that stands in the open.
///
/// **This is what being caught out in a storm costs.** A unit that stands on
/// ground carrying a finished upgrade is not in the open, so the shelter a
/// faction built is what answers this.
///
/// **This is a balance value and not a measurement.**[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: Balance register, the weather rows. `docs/reference/balance.md`
pub const LOSS_CHANCE_AT_FULL_DEPTH: i64 = 8;

/// The numerator of the food that one tick under the deepest eye flattens.
///
/// The share is of the food the tile still carries, so a tile that a storm
/// has already stripped loses less than a full one and the stock never falls
/// below nothing.
///
/// **This is a balance value and not a measurement.**[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^2]: Balance register, the weather rows. `docs/reference/balance.md`
pub const FOOD_NUMERATOR_AT_FULL_DEPTH: i64 = 1;

/// The denominator that the food numerator is stated over.
pub const FOOD_DENOMINATOR: i64 = 8;

/// The food that a storm takes off a tile that carries any, whatever the
/// share truncates to.
///
/// **A share alone takes nothing off a small stock.** Most tiles of a world
/// carry a few whole units of food, and a share of a few units floors to
/// nothing, so a storm over ordinary ground would flatten nothing at all. The
/// weather solve met the same shape when it dried the ground, and it answered
/// it the same way.[^1]
///
/// The floor never takes more than the tile carries.
///
/// **This is a balance value and not a measurement.**[^2] [^3]
///
/// # References
///
/// [^1]: The ground dries by a share and one whole drop more. [`crate::weather::WeatherField`]
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
/// [^3]: Balance register, the weather rows. `docs/reference/balance.md`
pub const FOOD_FLOOR: i64 = 1;

/// The draw index of the loss draw of one unit.
pub const DRAW_LOSS: u32 = 0;

/// Returns the deficit that a rate reads, from the deficit a cell carries.
///
/// **This is the one place that clamps.** A cell under two eyes carries the
/// sum of both, and the rates are stated against one eye at the ceiling.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[must_use]
pub const fn rated_depth(deficit: i32) -> i64 {
    let depth = deficit as i64;
    if depth < 0 {
        return 0;
    }
    if depth > DEPTH_WHOLE {
        return DEPTH_WHOLE;
    }
    depth
}

/// Returns the condition that one tick of this deficit takes off an upgrade.
///
/// The answer is zero for a cell that no storm reaches, so a caller needs no
/// second test.
#[must_use]
pub fn wear_at(deficit: i32) -> i64 {
    let depth = rated_depth(deficit);
    if depth == 0 {
        return 0;
    }
    match sim_math::share(Accum(WEAR_AT_FULL_DEPTH), Accum(depth), Accum(DEPTH_WHOLE)) {
        Some(taken) => taken.0.clamp(0, upgrade::CONDITION_FULL),
        None => 0,
    }
}

/// Returns the food that one tick of this deficit takes off a tile.
///
/// The `carried` is the food the tile still holds. The answer never stands
/// above it, so a tile can be stripped and never owe. It is at least the
/// floor for any tile that carries food and stands under any deficit, because
/// the share alone truncates to nothing on a small stock.
#[must_use]
pub fn food_lost_at(deficit: i32, carried: i64) -> i64 {
    let depth = rated_depth(deficit);
    if depth == 0 || carried <= 0 {
        return 0;
    }
    let Some(full) = sim_math::share(
        Accum(carried),
        Accum(FOOD_NUMERATOR_AT_FULL_DEPTH),
        Accum(FOOD_DENOMINATOR),
    ) else {
        return FOOD_FLOOR.min(carried);
    };
    let scaled = match sim_math::share(full, Accum(depth), Accum(DEPTH_WHOLE)) {
        Some(taken) => taken.0,
        None => 0,
    };
    scaled.max(FOOD_FLOOR).clamp(0, carried)
}

/// Returns the chance, out of [`CHANCE_WHOLE`], that one tick of this deficit
/// ends one unit standing in the open.
#[must_use]
pub fn loss_chance_at(deficit: i32) -> i64 {
    let depth = rated_depth(deficit);
    if depth == 0 {
        return 0;
    }
    match sim_math::share(
        Accum(LOSS_CHANCE_AT_FULL_DEPTH),
        Accum(depth),
        Accum(DEPTH_WHOLE),
    ) {
        Some(chance) => chance.0.clamp(0, CHANCE_WHOLE),
        None => 0,
    }
}

/// Reports whether the storm ends one unit this tick.
///
/// The draw is keyed on the storm system, the frame and the whole identity of
/// the unit. It is keyed on the identity and not on the slot, so a unit
/// spawned into a slot that a lost unit left draws its own answer. It is
/// keyed on the frame as well, so a unit that stands in one storm for two
/// ticks draws twice.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[must_use]
pub fn takes_unit(seed: u64, frame: u64, unit: u64, deficit: i32) -> bool {
    let chance = loss_chance_at(deficit);
    if chance <= 0 {
        return false;
    }
    let roll = rng::draw_below(
        seed,
        rng::SYSTEM_STORM,
        frame,
        unit,
        DRAW_LOSS,
        CHANCE_WHOLE as u64,
    );
    (roll as i64) < chance
}

/// A storm ended one unit.
///
/// The type carries the columns that the fire casualty carries, because a
/// reader that counts the dead of a faction wants one shape. It is a separate
/// type all the same: a unit a storm took fell to no faction, and no
/// grievance follows it.
///
/// The layout is 8 + 8 + 4 + 2 + 1 + 1 bytes, which is 24 bytes at an
/// alignment of 8. The trailing array declares the padding byte, and no field
/// is a `bool`.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitLostToAStorm {
    /// The tick on which the unit went.
    pub tick: Tick,
    /// The whole identity of the unit.
    pub unit: u64,
    /// The tile it stood on.
    pub tile: TileIdx,
    /// The faction it served.
    pub faction: FactionId,
    /// The type it held.
    pub unit_type: UnitTypeId,
    /// The declared padding. Always zero.
    pub padding: [u8; 1],
}

impl UnitLostToAStorm {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        unit: u64,
        tile: TileIdx,
        faction: FactionId,
        unit_type: UnitTypeId,
    ) -> Self {
        Self {
            tick,
            unit,
            tile,
            faction,
            unit_type,
            padding: [0; 1],
        }
    }
}
