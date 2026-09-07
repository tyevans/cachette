//! A wildfire: the ground burns, the fire spreads, and the fire ends.
//!
//! # What this holds
//!
//! A fire is a sparse set of burning tiles. Each burning tile carries the
//! fuel it has left and the intensity it burns at. The world holds one
//! [`FireField`], and the cost of it follows the fire and never the world: a
//! world that nobody set alight holds no entry at all.[^1]
//!
//! # Why the fire ends
//!
//! A fire must burn to a conclusion on its own. No pass here asks whether
//! anything changed, and no loop repeats until a set is stable. The record
//! forbids a convergence test, so the end must be a property of the
//! arithmetic instead.[^2]
//!
//! Three rules give it:
//!
//! 1. A burning tile loses [`BURN_FOR_EACH_TICK`] fuel on every tick it
//!    burns. The constant is positive, so the fuel of a tile reaches zero
//!    after a bounded number of ticks.
//! 2. A tile that stops burning enters the spent set, and a spent tile never
//!    ignites again.
//! 3. A world holds a finite number of tiles.
//!
//! The burning set is therefore empty after a bounded number of ticks,
//! whatever the wind does and whatever anybody builds. The bound is the tile
//! count multiplied by the largest fuel divided by the burn rate.
//!
//! # What stops it
//!
//! Water and high rock carry no fuel, so neither ever ignites. A cell whose
//! ground the weather has wetted refuses ignition. Ground that already burned
//! is spent. A faction may also put a fire out with people, and that is the
//! third way a tile stops burning.
//!
//! # Determinism
//!
//! Every draw is keyed on the system, the frame, an entity and a draw index,
//! and nothing here holds generator state.[^3] The pass walks the burning set
//! in ascending tile order and the six directions in index order, so the
//! result is a property of the set and never of a schedule.[^4] The pass
//! takes no thread count at all, which is the cheapest way to hold that
//! property rather than the strongest.
//!
//! The entity slot of a spread draw is a tile index, which is below `2^32`.
//! The entity slot of a casualty draw is a whole unit identity, whose
//! generation part is one or more, so it is at or above `2^32`. The two
//! cannot name one key.[^5]
//!
//! # References
//!
//! [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
//! [^2]: ADR-0005, a solver runs a fixed iteration count. `docs/adrs/REGISTRY.md`
//! [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOURS, NEIGHBOUR_COUNT};
use crate::rng;
use crate::terrain::{TileKind, KIND_COUNT};
use crate::types::{FactionId, Tick, TileIdx};
use crate::unit_type::UnitTypeId;

/// The whole that a chance is stated out of.
///
/// A chance is an integer out of this value, and a draw below this bound
/// decides it. Nothing here is a fraction and nothing here is a float.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub const CHANCE_WHOLE: i64 = 1024;

/// The whole that the lightning chance is stated out of.
///
/// The lightning chance is far smaller than a spread chance, so it takes a
/// finer whole. A caller that wants one strike in a thousand ticks states
/// one thousand.
pub const LIGHTNING_WHOLE: u64 = 1_000_000;

/// The fuel that one tile of each ground carries.
///
/// The index is the ground kind as its number, so the table cannot fall out
/// of step with the kinds.[^1] Water and high rock carry nothing, and that is
/// what stops a fire that reaches a lake or a summit.
///
/// **These are balance values and not measurements.** A blocker governs every
/// cost figure this project holds.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const FUEL_FOR_EACH_KIND: [i64; KIND_COUNT] = [0, 200, 900, 120, 0];

/// The fuel that a finished upgrade adds to the tile it stands on.
///
/// A road, a terrace, a lodging or a wall is built of what grew there, so
/// ground that carries one burns longer than bare ground.
pub const FUEL_FOR_AN_UPGRADE: i64 = 400;

/// The fuel that one tick of burning consumes.
///
/// **The value is positive, and that is what makes a fire end.** A burning
/// tile loses this much on every tick, so it stops after a bounded number of
/// them. Nothing anywhere adds fuel to a burning tile.
pub const BURN_FOR_EACH_TICK: i64 = 100;

/// The intensity that a tile ignites at.
///
/// **A fresh fire is already hot enough to hurt.** A value at which one
/// crew put a tile out on the first tick and lost nobody made fighting a
/// fire free, and a thing that costs nothing is not a choice.
pub const INTENSITY_AT_IGNITION: i64 = 200;

/// The intensity that one tick adds to a fire that nobody fights.
pub const INTENSITY_RISE: i64 = 40;

/// The largest intensity a fire reaches.
pub const INTENSITY_CEILING: i64 = 400;

/// The intensity that one person fighting the fire takes off a tile in one
/// tick.
///
/// **Three people put a tile out in one tick, and one person takes several.**
/// One tick adds intensity to a fire and a fire has a ceiling, so the work of
/// three people beats the worst a tile can reach. One person still makes
/// progress, and that is what makes a crowd worth sending. The value is a
/// balance value and no measurement chooses it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const DOUSE_WORK_FOR_EACH_UNIT: i64 = 150;

/// The share of the spread chance that the fuel of the target ground gives.
///
/// The chance of a forest is the fuel of a forest divided by this, so ground
/// that holds more to burn catches more easily.
pub const FUEL_CHANCE_DIVISOR: i64 = 4;

/// The most that a following wind adds to a spread chance, and the most that
/// a head wind takes off it.
pub const WIND_CHANCE: i64 = 256;

/// The most that a clear sky adds to a spread chance.
///
/// The term reads the cloud share of the cell, which is the water the air
/// holds against what it could hold. A clear sky adds the whole of this and
/// a covered sky adds nothing.
pub const DRY_CHANCE: i64 = 128;

/// The chance, out of [`CHANCE_WHOLE`], that a fire at full intensity ends
/// one unit standing on it in one tick.
///
/// **This is what fighting a fire costs.** A faction that sends people into
/// the fire loses some of them, so fighting a fire competes with every other
/// thing those people could do.
pub const CASUALTY_CHANCE: i64 = 96;

/// The draw index of the casualty draw of one unit.
///
/// The six draw indices below it are the six directions of a spread draw, so
/// this is the first index that no direction takes.
pub const DRAW_CASUALTY: u32 = NEIGHBOUR_COUNT as u32;

/// The draw index that asks whether lightning strikes this tick.
pub const DRAW_LIGHTNING: u32 = DRAW_CASUALTY + 1;

/// The draw index that picks the tile that lightning strikes.
pub const DRAW_LIGHTNING_TILE: u32 = DRAW_LIGHTNING + 1;

/// The entity slot that a lightning draw keys on.
///
/// No tile index reaches it, because a tile index is a `u32`. No unit
/// identity reaches it either, because that would need both the largest slot
/// and the largest generation.
pub const LIGHTNING_ENTITY: u64 = u64::MAX;

/// Why a tile caught fire.
pub type StartCause = u8;

/// A caller set the tile alight.
pub const START_CAUSE_ORDERED: StartCause = 1;

/// The fire on a neighbouring tile spread to this one.
pub const START_CAUSE_SPREAD: StartCause = 2;

/// Lightning struck the tile.
pub const START_CAUSE_LIGHTNING: StartCause = 3;

/// How a tile stopped burning.
pub type EndCause = u8;

/// The tile ran out of fuel.
pub const END_CAUSE_BURNT_OUT: EndCause = 1;

/// People put the fire on the tile out.
pub const END_CAUSE_DOUSED: EndCause = 2;

/// A tile caught fire.
///
/// The layout is 8 + 4 + 1 + 3 bytes, which is 16 bytes at an alignment of 8.
/// The trailing array declares every padding byte, so the type holds no
/// uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct FireStarted {
    /// The tick on which the tile caught.
    pub tick: Tick,
    /// The tile that caught.
    pub tile: TileIdx,
    /// What set it alight.
    pub cause: StartCause,
    /// The declared padding. Always zero.
    pub padding: [u8; 3],
}

impl FireStarted {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, tile: TileIdx, cause: StartCause) -> Self {
        Self {
            tick,
            tile,
            cause,
            padding: [0; 3],
        }
    }
}

/// A tile stopped burning.
///
/// The layout is 8 + 4 + 1 + 3 bytes, which is 16 bytes at an alignment of 8.
/// The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct FireEnded {
    /// The tick on which the tile stopped burning.
    pub tick: Tick,
    /// The tile that stopped burning.
    pub tile: TileIdx,
    /// How it stopped.
    pub cause: EndCause,
    /// The declared padding. Always zero.
    pub padding: [u8; 3],
}

impl FireEnded {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, tile: TileIdx, cause: EndCause) -> Self {
        Self {
            tick,
            tile,
            cause,
            padding: [0; 3],
        }
    }
}

/// A fire ended one unit.
///
/// The type carries the same columns that the meeting event carries, because
/// a reader that counts the dead of a faction wants one shape.[^1] It is a
/// separate type all the same: a unit the fire took fell to no faction, and
/// no grievance follows it.
///
/// The layout is 8 + 8 + 4 + 2 + 1 + 1 bytes, which is 24 bytes at an
/// alignment of 8. The trailing array declares the padding byte.[^2]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitBurned {
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

impl UnitBurned {
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

/// One burning tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FireTile {
    /// The tile that burns.
    pub tile: TileIdx,
    /// The fuel it has left. It never rises.
    pub fuel: i64,
    /// How hard it burns now.
    pub intensity: i64,
}

/// What the world tells the fire about one tile.
///
/// The fire reads the ground, the weather and the buildings through this, so
/// the pass names no field of the world and the world states each fact
/// once.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroundReading {
    /// The ground of the tile.
    pub kind: TileKind,
    /// Whether a finished upgrade stands on it.
    pub carries_upgrade: bool,
    /// Whether the weather has wetted the cell that covers it.
    pub is_wet: bool,
    /// The share of the sky over the cell that carries cloud, out of the
    /// whole that the weather states.
    pub cloud_share: i64,
    /// The whole that the cloud share is stated out of.
    pub cloud_whole: i64,
    /// The wind over the cell that covers the tile, as a lattice vector
    /// twice the component along one direction.
    pub wind_along: [i32; NEIGHBOUR_COUNT],
    /// The largest value that a wind component reaches, in the same units.
    pub wind_whole: i32,
}

impl GroundReading {
    /// The fuel that this ground carries when it ignites.
    #[must_use]
    pub fn fuel(self) -> i64 {
        let base = FUEL_FOR_EACH_KIND[self.kind.to_u8() as usize];
        if base == 0 {
            return 0;
        }
        if self.carries_upgrade {
            base.saturating_add(FUEL_FOR_AN_UPGRADE)
        } else {
            base
        }
    }

    /// Reports whether this ground can catch at all.
    ///
    /// Ground with no fuel never catches, and wet ground never catches. The
    /// two together are what a fire cannot cross.
    #[must_use]
    pub fn admits_fire(self) -> bool {
        self.fuel() > 0 && !self.is_wet
    }
}

/// Returns the chance, out of [`CHANCE_WHOLE`], that a burning tile sets one
/// neighbour alight in one tick.
///
/// The `direction` is the index of the step from the burning tile to the
/// neighbour, and the reading describes the neighbour. A fire that runs with
/// the wind reaches further than a fire that runs into it, and a fire under a
/// clear sky reaches further than a fire under cloud.
///
/// The answer is zero for ground that admits no fire, so a caller needs no
/// second test.
#[must_use]
pub fn spread_chance(intensity: i64, direction: usize, target: GroundReading) -> i64 {
    if !target.admits_fire() || direction >= NEIGHBOUR_COUNT {
        return 0;
    }
    let fuel = target.fuel() / FUEL_CHANCE_DIVISOR;
    let wind = if target.wind_whole > 0 {
        i64::from(target.wind_along[direction]).saturating_mul(WIND_CHANCE)
            / i64::from(target.wind_whole)
    } else {
        0
    };
    let dry = if target.cloud_whole > 0 {
        DRY_CHANCE.saturating_mul(target.cloud_whole - target.cloud_share.min(target.cloud_whole))
            / target.cloud_whole
    } else {
        DRY_CHANCE
    };
    let raw = fuel.saturating_add(wind).saturating_add(dry);
    if raw <= 0 {
        return 0;
    }
    // A fire that people have beaten down reaches less far than one at its
    // height. The term is the last one applied, so it scales the whole.
    let scaled = raw.saturating_mul(intensity.clamp(0, INTENSITY_CEILING)) / INTENSITY_CEILING;
    scaled.clamp(0, CHANCE_WHOLE)
}

/// Returns the chance, out of [`CHANCE_WHOLE`], that a fire ends one unit
/// standing on it in one tick.
#[must_use]
pub fn casualty_chance(intensity: i64) -> i64 {
    CASUALTY_CHANCE.saturating_mul(intensity.clamp(0, INTENSITY_CEILING)) / INTENSITY_CEILING
}

/// Every tile that burns, and every tile that has burned.
///
/// Both sets are held in ascending tile order, and a lookup is a binary
/// search. Neither grows with the world: a world that nobody set alight holds
/// nothing here at all.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FireField {
    /// The burning tiles, in ascending tile order.
    burning: Vec<FireTile>,
    /// Every tile that has stopped burning, in ascending tile order.
    ///
    /// **A spent tile never ignites again, and that is why a fire ends.** The
    /// set only grows, so the set of tiles that can still catch only shrinks.
    spent: Vec<TileIdx>,
    /// How many tiles have caught over the life of the world.
    started: i64,
    /// How many tiles ran out of fuel over the life of the world.
    burnt_out: i64,
    /// How many tiles people put out over the life of the world.
    doused: i64,
    /// How many units the fire has ended over the life of the world.
    burned_units: i64,
}

impl FireField {
    /// Builds a field that holds nothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            burning: Vec::new(),
            spent: Vec::new(),
            started: 0,
            burnt_out: 0,
            doused: 0,
            burned_units: 0,
        }
    }

    /// Reports whether a tile burns now.
    #[must_use]
    pub fn is_burning(&self, tile: TileIdx) -> bool {
        self.burning
            .binary_search_by_key(&tile.0, |entry| entry.tile.0)
            .is_ok()
    }

    /// Reports whether a tile has burned already.
    #[must_use]
    pub fn is_spent(&self, tile: TileIdx) -> bool {
        self.spent.binary_search_by_key(&tile.0, |t| t.0).is_ok()
    }

    /// Returns the state of one burning tile, or `None` when it does not
    /// burn.
    #[must_use]
    pub fn burning_tile(&self, tile: TileIdx) -> Option<FireTile> {
        let at = self
            .burning
            .binary_search_by_key(&tile.0, |entry| entry.tile.0)
            .ok()?;
        Some(self.burning[at])
    }

    /// Returns every burning tile, in ascending tile order.
    #[must_use]
    pub fn burning(&self) -> &[FireTile] {
        &self.burning
    }

    /// Returns every burning tile as an index, in ascending tile order.
    ///
    /// **This is the set a control plane sends people to.** The caller names
    /// no tile of its own and loops over nothing: it takes the whole set and
    /// hands it back as the seed set of one field.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub fn burning_tiles(&self) -> Vec<TileIdx> {
        self.burning.iter().map(|entry| entry.tile).collect()
    }

    /// Returns how many tiles burn now.
    #[must_use]
    pub fn burning_count(&self) -> usize {
        self.burning.len()
    }

    /// Returns every tile that has stopped burning, in ascending tile order.
    #[must_use]
    pub fn spent(&self) -> &[TileIdx] {
        &self.spent
    }

    /// Returns how many tiles have caught over the life of the world.
    #[must_use]
    pub const fn started_total(&self) -> i64 {
        self.started
    }

    /// Returns how many tiles ran out of fuel over the life of the world.
    #[must_use]
    pub const fn burnt_out_total(&self) -> i64 {
        self.burnt_out
    }

    /// Returns how many tiles people put out over the life of the world.
    #[must_use]
    pub const fn doused_total(&self) -> i64 {
        self.doused
    }

    /// Returns how many units the fire has ended over the life of the world.
    #[must_use]
    pub const fn burned_units_total(&self) -> i64 {
        self.burned_units
    }

    /// Records that the fire ended one unit. The world despawns it.
    pub fn count_burned_unit(&mut self) {
        self.burned_units = self.burned_units.saturating_add(1);
    }

    /// Sets one tile alight, and returns whether anything caught.
    ///
    /// Returns `false` when the tile already burns, when it has burned
    /// already, and when the ground admits no fire. A caller that names a
    /// lake gets `false` and changes nothing.
    pub fn ignite(&mut self, tile: TileIdx, ground: GroundReading) -> bool {
        if !ground.admits_fire() || self.is_spent(tile) {
            return false;
        }
        let Err(at) = self
            .burning
            .binary_search_by_key(&tile.0, |entry| entry.tile.0)
        else {
            return false;
        };
        self.burning.insert(
            at,
            FireTile {
                tile,
                fuel: ground.fuel(),
                intensity: INTENSITY_AT_IGNITION,
            },
        );
        self.started = self.started.saturating_add(1);
        true
    }

    /// Advances every burning tile by one tick, and returns the tiles that
    /// stopped.
    ///
    /// The `suppression` gives, for each burning tile in ascending tile
    /// order, the work that people standing on it took off. The caller counts
    /// them, because the fire holds no unit.
    ///
    /// **A tile that stops enters the spent set.** It cannot ignite again,
    /// and that is what makes the fire end.
    pub fn advance(&mut self, suppression: &[i64]) -> Vec<(TileIdx, EndCause)> {
        let mut ended = Vec::new();
        let mut kept = Vec::with_capacity(self.burning.len());
        for (at, entry) in self.burning.iter().enumerate() {
            let mut next = *entry;
            let taken = suppression.get(at).copied().unwrap_or(0);
            next.intensity = next
                .intensity
                .saturating_add(INTENSITY_RISE)
                .min(INTENSITY_CEILING)
                .saturating_sub(taken);
            // **The fuel falls by a positive constant on every tick.** This
            // line is the whole reason a fire ends without a convergence
            // test, so nothing may make it conditional.
            next.fuel = next.fuel.saturating_sub(BURN_FOR_EACH_TICK);
            if next.fuel <= 0 {
                ended.push((next.tile, END_CAUSE_BURNT_OUT));
                self.burnt_out = self.burnt_out.saturating_add(1);
            } else if next.intensity <= 0 {
                ended.push((next.tile, END_CAUSE_DOUSED));
                self.doused = self.doused.saturating_add(1);
            } else {
                kept.push(next);
            }
        }
        self.burning = kept;
        for (tile, _) in &ended {
            if let Err(at) = self.spent.binary_search_by_key(&tile.0, |t| t.0) {
                self.spent.insert(at, *tile);
            }
        }
        ended
    }

    /// Folds the field into the state hash.
    ///
    /// Every value here is stored state that a later step reads, so all of it
    /// enters the hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = hash.write_u64(self.burning.len() as u64);
        for entry in &self.burning {
            hash = hash
                .write_u64(u64::from(entry.tile.0))
                .write_u64(entry.fuel as u64)
                .write_u64(entry.intensity as u64);
        }
        hash = hash.write_u64(self.spent.len() as u64);
        for tile in &self.spent {
            hash = hash.write_u64(u64::from(tile.0));
        }
        hash.write_u64(self.started as u64)
            .write_u64(self.burnt_out as u64)
            .write_u64(self.doused as u64)
            .write_u64(self.burned_units as u64)
    }

    /// Reports whether the field holds what it claims to hold.
    ///
    /// Both sets are ascending and hold each tile once, no tile is in both,
    /// and every burning tile holds fuel above zero.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        for pair in self.burning.windows(2) {
            if pair[0].tile.0 >= pair[1].tile.0 {
                return false;
            }
        }
        for pair in self.spent.windows(2) {
            if pair[0].0 >= pair[1].0 {
                return false;
            }
        }
        for entry in &self.burning {
            if entry.fuel <= 0 || entry.intensity <= 0 {
                return false;
            }
            if self.is_spent(entry.tile) {
                return false;
            }
        }
        self.started >= self.burnt_out.saturating_add(self.doused)
    }
}

/// Returns the twelve-part wind reading of one cell, as the component along
/// each of the six directions.
///
/// The caller passes the wind of the cell that covers the tile. The reading
/// carries the components rather than the vector, so the spread function
/// names no weather type and the weather module stays a reader.
#[must_use]
pub fn wind_components(wind: crate::weather::Wind) -> [i32; NEIGHBOUR_COUNT] {
    let mut along = [0i32; NEIGHBOUR_COUNT];
    for (direction, entry) in along.iter_mut().enumerate() {
        *entry = wind.along(NEIGHBOURS[direction]);
    }
    along
}

/// Reports whether one burning tile sets one neighbour alight this tick.
///
/// The draw is keyed on the fire system, the frame, the burning tile and the
/// direction. Four things therefore decide it, and changing any one of them
/// changes the answer.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[must_use]
pub fn spreads(
    seed: u64,
    frame: u64,
    source: TileIdx,
    direction: usize,
    intensity: i64,
    target: GroundReading,
) -> bool {
    let chance = spread_chance(intensity, direction, target);
    if chance <= 0 {
        return false;
    }
    let roll = rng::draw_below(
        seed,
        rng::SYSTEM_FIRE,
        frame,
        u64::from(source.0),
        direction as u32,
        CHANCE_WHOLE as u64,
    );
    (roll as i64) < chance
}

/// Reports whether the fire ends one unit this tick.
///
/// The draw is keyed on the fire system, the frame and the whole identity of
/// the unit. It is keyed on the identity and not on the slot, so a unit
/// spawned into a slot that a burned unit left draws its own answer.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[must_use]
pub fn burns_unit(seed: u64, frame: u64, unit: u64, intensity: i64) -> bool {
    let chance = casualty_chance(intensity);
    if chance <= 0 {
        return false;
    }
    let roll = rng::draw_below(
        seed,
        rng::SYSTEM_FIRE,
        frame,
        unit,
        DRAW_CASUALTY,
        CHANCE_WHOLE as u64,
    );
    (roll as i64) < chance
}

/// Returns the tile that lightning strikes this tick, or `None` when no
/// strike happens.
///
/// The chance is stated out of [`LIGHTNING_WHOLE`], and a caller that states
/// zero gets no strike ever. The pass costs the same whatever the world
/// holds: it takes two draws and reads nothing over the lattice.
#[must_use]
pub fn lightning_strike(seed: u64, frame: u64, chance: u64, grid: Grid) -> Option<TileIdx> {
    if chance == 0 {
        return None;
    }
    let roll = rng::draw_below(
        seed,
        rng::SYSTEM_FIRE,
        frame,
        LIGHTNING_ENTITY,
        DRAW_LIGHTNING,
        LIGHTNING_WHOLE,
    );
    if roll >= chance {
        return None;
    }
    let count = u64::from(grid.tile_count());
    if count == 0 {
        return None;
    }
    let index = rng::draw_below(
        seed,
        rng::SYSTEM_FIRE,
        frame,
        LIGHTNING_ENTITY,
        DRAW_LIGHTNING_TILE,
        count,
    );
    let tile = TileIdx(index as u32);
    grid.address_of(tile).map(|_: Axial| tile)
}

/// The condition that one tick of fire takes off an upgrade.
///
/// The upgrade module states the full condition of a level, and this is a
/// share of it, so a building that stands in a fire falls after a few ticks.
/// The wear pass applies it, because that pass owns every removal of an
/// upgrade and a second site would be one rule in two places.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const WEAR_FOR_EACH_TICK: i64 = 250_000;
