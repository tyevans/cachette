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
//! [^3]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge, BLOCK_BITS_DEFAULT};
use crate::character::{CharacterArena, CharacterError};
use crate::choose::{
    self, ChoiceError, ChoiceExplanation, ChoiceSchedule, NeedBuckets, WeightProfile,
};
use crate::cohort::{
    self, CohortError, CohortTable, DeathPlane, DrawLedger, NeedCondition, NeedRule, SiteRationed,
    UnitStarved,
};
use crate::descent::{DescentId, Parents};
use crate::event::{ResourceTaken, TileChanged, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};
use crate::founding::{self, Founding, FoundingError, FoundingOutcome, Survey};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, GridError, NEIGHBOUR_COUNT};
use crate::holding::{FactionMask, Holder, Holding};
use crate::household;
use crate::influence::{Influence, InfluenceError, InfluenceField};
use crate::position::{self, Position, PositionError, PositionTable, SitePreference};
use crate::pyramid::{CellSummary, ExitField, Pyramid};
use crate::rates::{RateError, RateLedger, RateSchedule, RateTable, SiteShortfall};
use crate::resource::{
    ledger_key, Amount, CarryLoad, DepletionLedger, RecoveryRules, ResourceField, ResourceKind,
    RESOURCE_KIND_COUNT,
};
use crate::rng;
use crate::sim_math;
use crate::site::{CommodityId, SettlementArena, SettlementError, COMMODITY_COUNT};
use crate::slots::Slots;
use crate::soldier::{SoldierArena, SoldierError};
#[cfg(not(feature = "probe-nondeterminism"))]
use crate::sort;
use crate::sort::{BoundedKey, SortError};
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TerrainTile, TileKind};
use crate::tile_value::{TileValueRange, TileValues};
use crate::types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx, FACTION_CEILING};
use crate::upgrade::{self, UpgradeKind, UpgradeMap, UpgradeSite};

/// The reason that a value did not name a live entity.
///
/// A caller outside this crate cannot build an identity, so it names one by
/// the value the engine gave it. That value can be stale, and it can be
/// nothing the engine ever gave. The variants below tell the two apart, so
/// that a boundary can report which one happened.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// The value is not an identity at all. The engine never gives out zero.
    NotAnIdentity,
    /// The value names a slot that the arena does not hold.
    NoSuchSlot {
        /// The slot that the value named.
        slot: u32,
    },
    /// The slot exists and holds a later generation, so the entity is dead.
    ///
    /// The arena may have given the slot to another entity. Resolution
    /// refuses rather than return that entity.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    Stale {
        /// The slot that the value named.
        slot: u32,
        /// The generation that the value carried.
        given: u32,
        /// The generation that the arena holds for the slot.
        held: u32,
    },
}

impl core::fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotAnIdentity => write!(formatter, "the value is not an identity"),
            Self::NoSuchSlot { slot } => {
                write!(formatter, "the arena holds no slot {slot}")
            }
            Self::Stale { slot, given, held } => write!(
                formatter,
                "the identity names slot {slot} at generation {given}, \
                 and the arena holds generation {held} there"
            ),
        }
    }
}

impl std::error::Error for IdentityError {}

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
    /// The site rate pass refused to run.
    Rates(RateError),
    /// The consumption pass refused to run.
    Consumption(CohortError),
    /// The choice pass refused to run.
    Choice(ChoiceError),
    /// The influence solve refused to run.
    Influence(InfluenceError),
    /// A pass over the positions of the sites refused to run.
    Positions(PositionError),
}

impl From<PositionError> for StepError {
    fn from(error: PositionError) -> Self {
        match error {
            PositionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Positions(other),
        }
    }
}

impl From<InfluenceError> for StepError {
    fn from(error: InfluenceError) -> Self {
        match error {
            InfluenceError::ZeroThreads => Self::ZeroThreads,
            other => Self::Influence(other),
        }
    }
}

impl From<ChoiceError> for StepError {
    fn from(error: ChoiceError) -> Self {
        Self::Choice(error)
    }
}

impl From<CohortError> for StepError {
    fn from(error: CohortError) -> Self {
        match error {
            CohortError::ZeroThreads => Self::ZeroThreads,
            other => Self::Consumption(other),
        }
    }
}

impl From<RateError> for StepError {
    fn from(error: RateError) -> Self {
        match error {
            RateError::ZeroThreads => Self::ZeroThreads,
            other => Self::Rates(other),
        }
    }
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
    /// The influence field refused to build.
    Influence(InfluenceError),
}

impl From<InfluenceError> for WorldError {
    fn from(error: InfluenceError) -> Self {
        Self::Influence(error)
    }
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
            Self::Influence(error) => {
                write!(formatter, "the world has no influence field: {error:?}")
            }
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
            Self::Rates(error) => write!(formatter, "the site rate pass refused: {error}"),
            Self::Consumption(error) => {
                write!(formatter, "the consumption pass refused: {error}")
            }
            Self::Choice(error) => write!(formatter, "the choice pass refused: {error}"),
            Self::Influence(error) => {
                write!(formatter, "the influence solve refused: {error:?}")
            }
            Self::Positions(error) => {
                write!(formatter, "the position pass refused: {error}")
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
    /// The number of unit slots that the world reserves.
    ///
    /// The world reserves this many entries in each unit column when it is
    /// built, and it opens no more. A spawn past the reservation gets a
    /// typed refusal.[^1]
    ///
    /// **This is the one place that states the reservation.** The arena
    /// takes the value from here and names no default of its own, so no
    /// second site can disagree with this one.[^2]
    ///
    /// The reservation is paid once, at construction. The cost of a tick
    /// grows with the number of units that live, not with the number the
    /// world reserved.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0084, the world reserves the unit columns at construction, decisions D1 and D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    /// [^2]: ADR-0084, the world reserves the unit columns at construction, decision D2. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    /// [^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    pub unit_capacity: u32,
}

impl WorldConfig {
    /// The population that the project targets, counted over everybody.
    ///
    /// One million is the whole population. Soldiers are a fraction of it,
    /// and civilians are not separate entities on top of the million. The
    /// project owner answered this, and the scale constants table holds the
    /// row.[^1]
    ///
    /// This is the reservation that a world takes when the caller states no
    /// other. It is a target the project chose, not a figure anybody
    /// measured.[^2]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    /// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub const TARGET_UNIT_POPULATION: u32 = 1_000_000;
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
            unit_capacity: Self::TARGET_UNIT_POPULATION,
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
    /// The tile stub value of every tile.
    ///
    /// The field generates the value of a tile from the seed and stores only
    /// what the frames changed, so building a world visits no tile and
    /// allocates nothing that grows with the tile count.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    /// [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    values: TileValues,
    log: Vec<TileChanged>,
    /// The events of the last step, from the gather resolve.
    gather_log: Vec<ResourceTaken>,
    soldiers: SoldierArena,
    settlements: SettlementArena,
    characters: CharacterArena,
    bridge: UnitTileBridge,
    terrain: Terrain,
    /// The stock that each tile started with. It stores nothing.
    resources: ResourceField,
    /// What has been taken from each tile that somebody gathered from.
    depletion: DepletionLedger,
    /// What left the world in the hands of a dead unit.
    ///
    /// A unit that dies takes its load out of the world. Conservation must
    /// still balance, so the world records where the load went rather than
    /// letting it disappear.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    departed: [u64; RESOURCE_KIND_COUNT],
    /// The upgrade that each improved tile carries.
    ///
    /// The map holds one entry for each tile that somebody built on, and it
    /// holds nothing else. A world in which nobody built holds no entry, so
    /// the memory cost follows the building and not the size of the
    /// world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    upgrades: UpgradeMap,
    /// Level 1 of the pyramid, derived from level 0 at the barrier.
    pyramid: Pyramid,
    /// The direction that each level 1 cell holds, for each option.
    ///
    /// The array is what movement steers by. It is a projection of level 1,
    /// derived again at every rebuild of it, and it sits beside the summaries
    /// rather than inside them, because two directions do not add.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D2 and D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    exits: ExitField,
    /// Who holds each tile, and what each faction holds.
    ///
    /// The holder column is level 0 and it is the truth. It is the one value
    /// that names the faction which owns a tile, and the tile event carries
    /// it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    holding: Holding,
    /// What each faction reaches, over the level 1 cell lattice.
    ///
    /// The field is a plane over the level 1 cells and it is not a summary:
    /// a cell of it is the result of a relaxation that reads the neighbours
    /// of the cell, and it carries what the last solve left. The record
    /// states the boundary that draws against the record which owns level 0,
    /// and an open choice asks a reviewer to settle it.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^2]: Decisions register, DEC-067. `docs/DECISIONS.md`
    influence: InfluenceField,
    /// When the site rates apply.
    schedule: RateSchedule,
    /// The production rate and the upkeep rate of each site.
    rates: RateTable,
    /// Every rate that has applied since the world was built.
    rate_ledger: RateLedger,
    /// The sites that could not pay at the last application.
    shortfall_log: Vec<SiteShortfall>,
    /// What a unit needs, and how fast it needs it.
    need_rule: NeedRule,
    /// The cohorts of every site, derived from the home column of the units.
    cohorts: CohortTable,
    /// Every draw that has run since the world was built.
    draw_ledger: DrawLedger,
    /// The sites that could not serve every cohort at the last draw.
    rationed_log: Vec<SiteRationed>,
    /// One bit for each unit that the last shortage ended.
    ///
    /// The plane is the batch of a structural change, and the scan of it
    /// applies the change after the barrier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
    death_plane: DeathPlane,
    /// The units that a shortage ended at the last scan, in slot order.
    starved_log: Vec<UnitStarved>,
    /// When each unit re-reads the world and chooses again.
    choice: ChoiceSchedule,
    /// How finely the choice tells two needs apart.
    ///
    /// **The width of the bucket is the mechanism of the decision and not a
    /// detail of it.** A need is a Q16.16 quantity, so unbucketed two units in
    /// one cell almost never share a need and the pass computes one answer for
    /// each unit.[^1] The width is a parameter of the world because no record
    /// sets it and no measurement chooses it, and a blocker governs every cost
    /// figure this project holds.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    /// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    /// [^3]: Decisions register, DEC-097. `docs/DECISIONS.md`
    buckets: NeedBuckets,
    /// The weight that a unit puts on each option of the choice.
    ///
    /// The profile is content, and it is a table of values. The engine reads
    /// it and never calls into it.[^1]
    ///
    /// It is an input to the world and not a fact the world holds, so it
    /// does not reach the state hash. What it decides does: the intent
    /// column carries the outcome, and that column is hashed.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    weights: WeightProfile,
    /// The positions that each site holds, and what each site wants.
    ///
    /// The table follows the slot column of the settlement arena. It is
    /// stored per site and never per tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    positions: PositionTable,
    /// When the site positions are rebalanced.
    ///
    /// The interval is a parameter of the world. The pass that reads it
    /// holds no period of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    position_schedule: RateSchedule,
    /// What the live stores hold, by the account of this world.
    ///
    /// The store column states the same total a second time, and the
    /// conservation check is what fails when the two disagree.[^1] The
    /// world adjusts this account at each place where a store changes
    /// outside the rate pass: a write from the control plane, and the loss
    /// of a settlement.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    store_account: [Accum; COMMODITY_COUNT],
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
        // The tile value field stores nothing here. It holds the seed and
        // the extent, and it generates a tile when a reader asks for one, so
        // the cost is paid per reader and never once for every tile before
        // the first frame.[^2]
        //
        // [^2]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
        let values = TileValues::new(config.seed, grid);
        let layout = BlockLayout::new(grid, BLOCK_BITS_DEFAULT)?;
        // The influence field is a plane over the level 1 cells, so its
        // lattice is the block lattice at the pitch of one block. It is a hex
        // grid for the same reason level 0 is one, and building it here means
        // the field states no geometry of its own.[^1]
        //
        // [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
        let cell_lattice = Grid::new(layout.blocks_wide(), layout.blocks_high())?;
        let soldiers = SoldierArena::new(grid, config.unit_capacity);
        let settlements = SettlementArena::new(grid);
        // The character tier states its own ceiling, and the arena checks
        // it here, once, when the world is built. Nothing checks a count on
        // a later call.[^1]
        //
        // [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
        let characters = CharacterArena::new();
        let mut bridge = UnitTileBridge::new(layout);
        bridge.rebuild(&soldiers)?;
        let terrain = Terrain::new(config.seed, grid);
        let mut world = Self {
            config,
            grid,
            tick: Tick(0),
            values,
            log: Vec::new(),
            gather_log: Vec::new(),
            soldiers,
            settlements,
            characters,
            bridge,
            terrain,
            resources: ResourceField::new(terrain),
            depletion: DepletionLedger::new(),
            departed: [0; RESOURCE_KIND_COUNT],
            upgrades: UpgradeMap::new(),
            pyramid: Pyramid::new(layout, ResourceField::new(terrain))?,
            exits: ExitField::new(cell_lattice),
            holding: Holding::new(layout),
            influence: InfluenceField::new(cell_lattice, config.faction_count)?,
            schedule: RateSchedule::DEFAULT,
            rates: RateTable::new(),
            rate_ledger: RateLedger::ZERO,
            shortfall_log: Vec::new(),
            need_rule: NeedRule::DEFAULT,
            cohorts: CohortTable::new(),
            draw_ledger: DrawLedger::ZERO,
            rationed_log: Vec::new(),
            death_plane: DeathPlane::new(),
            starved_log: Vec::new(),
            choice: ChoiceSchedule::DEFAULT,
            buckets: NeedBuckets::DEFAULT,
            weights: WeightProfile::EVEN,
            positions: PositionTable::new(),
            position_schedule: RateSchedule::DEFAULT,
            store_account: [Accum(0); COMMODITY_COUNT],
        };
        // A world that has never stepped still answers a question about a
        // region. A level that nothing rebuilt would describe an empty world
        // and would be wrong rather than absent.
        world.rebuild_level_1(1)?;
        // The conductance of a cell follows the ground it covers, and the
        // ground does not change for the life of a world, so this runs once
        // and never again. It reads the level that the rebuild above just
        // filled rather than sweeping the tiles a second time.[^1]
        //
        // [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        world.influence.read_the_ground(world.pyramid.cells())?;
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
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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

    /// Returns the resource field of the world.
    ///
    /// The field holds the ground, and nothing else. It costs the same at any
    /// tile count, because it stores no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub const fn resources(&self) -> ResourceField {
        self.resources
    }

    /// Returns the stock that one tile started with.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn original_stock(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        self.resources.original(address, kind)
    }

    /// Returns what has been taken from one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn taken_from(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        let tile = self.grid.index_of(address)?;
        Some(self.depletion.taken(tile, kind))
    }

    /// Returns the stock that one tile still holds.
    ///
    /// The answer is what the tile started with, less what has been taken.
    /// The engine stores the second term only, so a tile nobody touched costs
    /// nothing.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub fn tile_stock(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        let tile = self.grid.index_of(address)?;
        let original = self.resources.original(address, kind)?;
        Some(Amount(
            original
                .0
                .saturating_sub(self.depletion.taken(tile, kind).0),
        ))
    }

    /// Returns the depletion ledger.
    ///
    /// The ledger holds one entry for each tile and kind that somebody
    /// gathered from. A world in which nothing was gathered holds none.
    #[must_use]
    pub const fn depletion(&self) -> &DepletionLedger {
        &self.depletion
    }

    /// Returns how fast each kind of deposit recovers.
    #[must_use]
    pub const fn recovery_rules(&self) -> RecoveryRules {
        self.depletion.recovery()
    }

    /// Replaces the rules that say how fast each kind of deposit recovers.
    ///
    /// The caller replaces the whole rule set, so a period lives in one place
    /// and no two sites can hold a different value for one kind.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn set_recovery_rules(&mut self, rules: RecoveryRules) {
        self.depletion.set_recovery(rules);
    }

    /// Returns what one soldier carries.
    ///
    /// Returns `None` when the identity is dead.
    #[must_use]
    pub fn soldier_carry(&self, entity: Entity) -> Option<CarryLoad> {
        self.soldiers.carry(entity)
    }

    /// Tells one soldier to gather a kind of resource.
    ///
    /// The soldier then takes from the tile it stands on, once in each step,
    /// until the caller stops it. Returns `false` when the identity is dead.
    ///
    /// The command names a unit and a kind. It never loops over a tile, and it
    /// runs no work of its own: the step resolves every order of the frame in
    /// one pass.[^1]
    ///
    /// **The order holds until the unit next chooses.** The choice pass is the
    /// engine writer of this column, and it writes the order of a unit only on
    /// the frame that the level 1 cell of that unit chooses.[^2] An order given
    /// here therefore survives the frames until then, and the choice replaces
    /// it when it comes round.
    ///
    /// # References
    ///
    /// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    pub fn order_gather(&mut self, entity: Entity, kind: ResourceKind) -> bool {
        self.soldiers.set_gather_order(entity, Some(kind))
    }

    /// Tells one soldier to stop gathering.
    ///
    /// Returns `false` when the identity is dead. The stop holds until the
    /// unit next chooses, in the same way an order does.
    pub fn stop_gather(&mut self, entity: Entity) -> bool {
        self.soldiers.set_gather_order(entity, None)
    }

    /// Returns the gather order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier gathers.
    #[must_use]
    pub fn gather_order(&self, entity: Entity) -> Option<Option<ResourceKind>> {
        self.soldiers.gather_order(entity)
    }

    /// Returns the gather events of the last step.
    ///
    /// One event reports one grant. A watcher reads the log to see a resource
    /// being taken.
    #[must_use]
    pub fn gather_log(&self) -> &[ResourceTaken] {
        &self.gather_log
    }

    /// Returns the gather events of the last step as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn gather_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.gather_log)
    }

    /// Returns what left the world in the hands of a dead unit, by kind.
    #[must_use]
    pub const fn departed_carry(&self) -> &[u64; RESOURCE_KIND_COUNT] {
        &self.departed
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
    /// when the ground carries no unit, or when another settlement already
    /// stands on the tile.
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
        // Ground that admits no unit admits no holder, and a settlement is a
        // holder of ground.[^2] The rule reads the passability of the tile
        // and states nothing of its own about the ground, so the capacity
        // table stays the one declaration of which ground carries
        // anybody.[^3] [^4]
        //
        // The extent refusal stays with the arena, which owns the grid, so
        // this test says nothing about an address outside the world.
        //
        // [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
        // [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.grid.contains(address) && !self.admits_a_unit(address) {
            return Err(SettlementError::TileAdmitsNobody(address));
        }
        let settlement = self.settlements.found(address, faction)?;
        // The rate table follows the slot column of the arena, and a founding
        // may open a slot that the table has never held. A new row earns
        // nothing and owes nothing.
        //
        // The founding does not clear the row. The loss of a settlement does
        // that, and it is the only place that needs to: a slot the arena has
        // never handed out already holds an idle row. Clearing the row here as
        // well would state one fact in two places, and neither copy would fail
        // when the other was removed.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        self.rates.open_to(self.settlements.slot_count());
        // The position table follows the same slot column, for the same
        // reason. A new slot holds no position and the preference that a
        // site starts with.
        self.positions.open_to(self.settlements.slot_count());
        Ok(settlement)
    }

    /// Surveys a bounded sample of the world for a place to found a group.
    ///
    /// The call reads a fixed number of candidate places and a fixed number
    /// of tiles around each one. Neither number is a function of the world
    /// extent, so the cost of the call does not grow with the world.[^1] The
    /// call writes nothing. A watcher asks it why a place is good and gets
    /// the counts that made the score.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    pub fn survey_founding(&self, group: u32, faction: FactionId) -> Result<Survey, FoundingError> {
        founding::survey(self.resources, group, faction, &[])
    }

    /// Surveys a sample for a place that keeps its distance from the places
    /// taken.
    ///
    /// The faction fills the frame slot of the draw key, so two factions read
    /// two samples.[^1] A place closer than the minimum distance to a place in
    /// the list is not eligible, whatever the ground there holds.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    pub fn survey_founding_apart(
        &self,
        group: u32,
        faction: FactionId,
        taken: &[Axial],
    ) -> Result<Survey, FoundingError> {
        founding::survey(self.resources, group, faction, taken)
    }

    /// Surveys the places a caller names, against the places taken.
    ///
    /// A caller that wants to compare two places of its own choosing calls
    /// this. The call writes nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    pub fn survey_places(
        &self,
        addresses: &[Axial],
        group: u32,
        taken: &[Axial],
    ) -> Result<Survey, FoundingError> {
        founding::survey_addresses(self.resources, addresses, group, taken)
    }

    /// Founds a run: a group of people, in a place the engine chose.
    ///
    /// The size of the group is an input to the run. It is not the population
    /// the world is sized for, and the world reserves the same storage
    /// whatever it is.[^1]
    ///
    /// The founding is one of two ways to people a world. A caller that wants
    /// a unit in a place of its own choosing spawns one directly, and this
    /// call is built on that one.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, when no place in the
    /// sample admits the whole group, when the ordering refuses to run, or
    /// when a person or the settlement refuses to arrive.
    ///
    /// # References
    ///
    /// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    /// [^2]: Open decisions register, DEC-030. `docs/DECISIONS.md`
    pub fn found_run(&mut self, group: u32, faction: FactionId) -> Result<Founding, FoundingError> {
        self.found_one(group, faction, &[])
    }

    /// Founds one group for each faction the world holds.
    ///
    /// The run founds in ascending faction index. The order is a property of
    /// the run and not an input, so no caller can give one faction the better
    /// place by listing it first.[^1] Founding N keeps the minimum distance
    /// from every place a founding before it took, so the foundings are a
    /// sequence and not a set.[^2]
    ///
    /// The run reports one outcome for each faction. A faction that finds no
    /// admissible place is refused, and the foundings before it stand.[^3]
    /// The faction set comes from the world, so the loop holds no second
    /// count of its own.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    /// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn found_run_for_every_faction(&mut self, group: u32) -> Vec<FoundingOutcome> {
        let mut taken: Vec<Axial> = Vec::new();
        let mut outcomes = Vec::new();
        for index in 0..self.config.faction_count.max(1) {
            let faction = FactionId(index);
            let result = self.found_one(group, faction, &taken);
            if let Ok(founding) = &result {
                taken.push(founding.place());
            }
            outcomes.push(FoundingOutcome::new(faction, result));
        }
        outcomes
    }

    /// Founds one group, away from the places already taken.
    fn found_one(
        &mut self,
        group: u32,
        faction: FactionId,
        taken: &[Axial],
    ) -> Result<Founding, FoundingError> {
        let survey = self.survey_founding_apart(group, faction, taken)?;
        let chosen = survey
            .chosen()
            .ok_or(FoundingError::NoPlaceFound(survey.drawn()))?;
        let place = chosen.address();
        let (settlement, people) = self.settle_group(place, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        Ok(Founding::new(place, settlement, people, survey))
    }

    /// Founds a group at a place the caller names.
    ///
    /// The engine chooses the place of a run. This call exists so that a test
    /// can compare a place the engine chose against a place it did not, on a
    /// quantity the test computes for itself.
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, when the address lies
    /// outside the world, when the place does not admit the whole group, or
    /// when a person or the settlement refuses to arrive.
    pub fn found_group_at(
        &mut self,
        address: Axial,
        group: u32,
        faction: FactionId,
    ) -> Result<Founding, FoundingError> {
        if !self.grid.contains(address) {
            return Err(FoundingError::OutsideWorld(address));
        }
        let survey = founding::survey_addresses(self.resources, &[address], group, &[])?;
        let chosen = survey
            .chosen()
            .ok_or(FoundingError::NoPlaceFound(survey.drawn()))?;
        let (settlement, people) = self.settle_group(address, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        Ok(Founding::new(chosen.address(), settlement, people, survey))
    }

    /// Sets the food a founded site produces, from the ground it reaches.
    ///
    /// A founding seats a group and gives it a store. Nothing else fills
    /// that store, so a site founded without a rate feeds nobody, and every
    /// unit in it crosses the bound at the same tick.[^1] The founding
    /// therefore sets the rate, because the founding is the one call that
    /// has both the site and the survey that measured the ground.
    ///
    /// The rule is that one unit of food the place reaches feeds one person.
    /// The ration comes from the need rule and is not repeated here, so the
    /// amount a person eats has one declaration site.[^2] A place that
    /// reaches more food than the group needs therefore holds a surplus, and
    /// a place that reaches less runs the group short. The survey score
    /// weighs food, so the choice the engine makes now decides whether the
    /// group lives.[^3]
    ///
    /// The rate never reaches the upkeep table. Upkeep is a rate above zero
    /// that subtracts, and this is production.[^4]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-124. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^3]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^4]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    fn provision_site(&mut self, settlement: Entity, food: Amount) {
        // The count saturates at the range the fixed-point constructor
        // takes. A survey reads a bounded sample, so the food it reports is
        // bounded, and the saturation is a guard rather than a case.
        let reached = i16::try_from(food.0).unwrap_or(i16::MAX);
        let rate = sim_math::mul(self.need_rule.ration(), Fix32::from_int(reached));
        // The identity is live, because this call site founded it in the
        // same function. A refusal here is a programming error in the
        // founding, not a caller mistake.
        self.set_production_rate(settlement, CommodityId(0), rate)
            .expect("the rate is at or above zero and the commodity is in the set");
    }

    /// Seats a settlement at a place and spreads a group over its disc.
    ///
    /// The disc is walked in its fixed order, and each open tile takes up to
    /// the number of units that its ground holds.[^1] The order is the same
    /// on every run and at every thread count, because it is a function of
    /// the address alone.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn settle_group(
        &mut self,
        place: Axial,
        group: u32,
        faction: FactionId,
    ) -> Result<(Entity, Vec<Entity>), FoundingError> {
        let settlement = self.found_settlement(place, faction)?;
        let mut people = Vec::with_capacity(group as usize);
        let mut remaining = group;
        for address in founding::disc(self.grid, place, founding::SURVEY_RADIUS) {
            if remaining == 0 {
                break;
            }
            let Some(kind) = self.tile_kind(address) else {
                continue;
            };
            if !kind.is_passable() {
                continue;
            }
            // The founding fills each tile to the capacity of its ground and
            // reads no occupancy count. A spawn does not read the capacity
            // either, and the derived occupancy structure is stale between
            // two frames, so a read of it here would be a third call site for
            // a rebuild that the step already owns.[^3] [^4] A second
            // founding over one disc may therefore over-fill a tile, which is
            // the caller mistake that decision permits and that movement
            // corrects, because admission never raises a tile above its
            // capacity.
            //
            // [^3]: Open decisions register, DEC-020. `docs/DECISIONS.md`
            // [^4]: Open decisions register, DEC-021. `docs/DECISIONS.md`
            for _ in 0..kind.capacity().min(remaining) {
                // A refusal here leaves a settlement standing and a part of
                // the group alive. The reservation makes that refusal
                // reachable, because a world whose unit reservation is below
                // the group runs out of slots part way through this
                // loop.[^6] Undo the founding rather than report a failure
                // over a world that half changed.
                //
                // [^6]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
                match self.spawn_soldier(address, faction) {
                    Ok(person) => people.push(person),
                    Err(error) => {
                        self.abandon_founding(people, settlement);
                        return Err(FoundingError::Person(error));
                    }
                }
                remaining -= 1;
            }
        }
        if remaining > 0 {
            // The eligibility rule says the disc holds the group, so this is
            // a disagreement between the rule and the placement rather than a
            // caller mistake. Report it and leave nothing half-founded.
            self.abandon_founding(people, settlement);
            return Err(FoundingError::NoPlaceFound(1));
        }
        // The group belongs to the settlement it founded. A unit draws from
        // the store of the site it belongs to, and a founding is the one
        // place today that puts a unit and a site together.[^5]
        //
        // [^5]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        for person in &people {
            self.set_home_site(*person, Some(settlement));
        }
        Ok((settlement, people))
    }

    /// Undoes a founding that could not finish.
    ///
    /// A founding that stops part way leaves a settlement standing and a
    /// part of its group alive. Both are removed here, so a refused founding
    /// changes nothing that a caller can observe. This is the one place that
    /// undoes a founding, and every refusal after the settlement stands goes
    /// through it.
    fn abandon_founding(&mut self, people: Vec<Entity>, settlement: Entity) {
        for person in people {
            self.despawn_soldier(person);
        }
        self.destroy_settlement(settlement);
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
        // What the settlement held leaves the account here. A dead slot keeps
        // its bytes until a founding clears them, and those bytes are not a
        // holding of anybody. The account must fall by the same amount, or
        // the conservation check finds a difference that no rate made.
        //
        // The slot is read before the loss, because the identity stops
        // resolving the moment the arena frees the slot.[^1]
        //
        // [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
        let Some(slot) = self.settlements.slot_of(entity) else {
            return false;
        };
        let store = self
            .settlements
            .store(entity)
            .expect("the identity resolved to a live slot");
        if !self.settlements.destroy(entity) {
            return false;
        }
        // A rate belongs to the site that earned it. The site is gone, so the
        // rate goes with it and the slot does not pay its successor.
        self.rates.clear_slot(slot);
        // A position belongs to the site that opened it. The site is gone,
        // so its positions go with it and the settlement founded next in
        // that slot does not inherit a staff it never hired.
        self.positions.clear_slot(slot);
        // A unit that drew from the lost site now belongs to no site. A home
        // left behind would name the slot, and the settlement founded next
        // in that slot would feed a population it never took.
        for unit in self.soldiers.iter().collect::<Vec<_>>() {
            if self.soldiers.home(unit) == Some(Some(slot)) {
                self.soldiers.set_home(unit, None);
            }
        }
        for index in 0..COMMODITY_COUNT {
            let held = store
                .quantity(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            self.store_account[index] =
                sim_math::combine(self.store_account[index], Accum(-i64::from(held.0)));
        }
        true
    }

    /// Resolves the value of an identity back to the settlement it names.
    ///
    /// A caller outside this crate holds an identity as the value the engine
    /// gave it. It cannot build one, and this is the only way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds. It refuses a mismatch, and it never
    /// returns the settlement that now stands in the slot.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an identity, when the arena
    /// holds no such slot, or when the slot holds a later generation.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D2 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn resolve_settlement(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.settlements.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.settlements.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.settlements.generation_of(slot),
        })
    }

    /// Returns the positions of every site.
    #[must_use]
    pub const fn positions(&self) -> &PositionTable {
        &self.positions
    }

    /// Returns the positions that one site holds.
    ///
    /// A dead identity gives `None` rather than the row of the settlement
    /// that now stands in the slot.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn site_positions(&self, site: Entity) -> Option<&[Position]> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.row(slot)
    }

    /// Returns what one site wants of each kind of work.
    #[must_use]
    pub fn site_preference(&self, site: Entity) -> Option<SitePreference> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.preference(slot)
    }

    /// Returns the unit that holds one position of one site.
    ///
    /// The call resolves the stored identity against the unit arena. A unit
    /// that died gives `None`, and the unit that took its slot is never the
    /// answer.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn position_holder(&self, site: Entity, index: usize) -> Option<Entity> {
        let slot = self.settlements.slot_of(site)?;
        self.positions.occupant(slot, index, &self.soldiers)
    }

    /// Gives one position of one site to one unit.
    ///
    /// This is the setter. It states no rule about who should hold a
    /// position, because the rule that chooses is separate work.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no live settlement, when the
    /// identity names no live unit, when the site holds no position at that
    /// index, and when the unit already holds another position at that site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    pub fn seat_in_position(
        &mut self,
        site: Entity,
        index: usize,
        unit: Entity,
    ) -> Result<(), PositionError> {
        let slot = self
            .settlements
            .slot_of(site)
            .ok_or(PositionError::NoSuchSlot(site.index()))?;
        if !self.soldiers.contains(unit) {
            return Err(PositionError::NoSuchSlot(unit.index()));
        }
        self.positions.seat(slot, index, unit)
    }

    /// Changes what a set of sites wants of one kind of work.
    ///
    /// **The command names no unit.** It states what a place wants, and the
    /// rebalance turns that into a number of positions of each kind. A
    /// caller that wanted to name the workers would be looping over
    /// entities, which the control plane never does.[^1]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the
    /// target is checked, before anything is written.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when an identity names no live settlement, or when
    /// the target is below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    pub fn prefer_at_sites(
        &mut self,
        sites: &[Entity],
        kind: ResourceKind,
        target: Fix32,
    ) -> Result<(), PositionError> {
        if target.0 < 0 {
            return Err(PositionError::TargetBelowZero(target));
        }
        let mut slots = Vec::with_capacity(sites.len());
        for site in sites {
            slots.push(
                self.settlements
                    .slot_of(*site)
                    .ok_or(PositionError::NoSuchSlot(site.index()))?,
            );
        }
        for slot in slots {
            self.positions.set_target(slot, kind, target)?;
        }
        Ok(())
    }

    /// Returns when the site positions are rebalanced.
    #[must_use]
    pub const fn position_schedule(&self) -> RateSchedule {
        self.position_schedule
    }

    /// Sets when the site positions are rebalanced.
    ///
    /// The interval is a parameter of the world. This function holds no
    /// recommended value.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is
    /// above the range that the schedule takes.
    pub fn set_position_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.position_schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
    }

    /// Returns the rule that says what a unit needs.
    #[must_use]
    pub const fn need_rule(&self) -> NeedRule {
        self.need_rule
    }

    /// Sets the rule that says what a unit needs.
    ///
    /// The rule carries the bound at which a shortage ends a unit, so the
    /// bound is a parameter of the world and never a constant of a
    /// kernel.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    pub const fn set_need_rule(&mut self, rule: NeedRule) {
        self.need_rule = rule;
    }

    /// Returns the condition that a shortage has put a unit in.
    ///
    /// Returns `None` when the identity is dead.[^1] The condition is a
    /// name, and it is what a watcher reads. A watcher that read the
    /// accumulator would hold the bound of the rule a second time.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn unit_condition(&self, entity: Entity) -> Option<NeedCondition> {
        self.soldiers
            .deficit(entity)
            .map(|deficit| self.need_rule.condition(deficit))
    }

    /// Returns the units that a shortage ended at the last scan, in slot
    /// order.
    #[must_use]
    pub fn starved_log(&self) -> &[UnitStarved] {
        &self.starved_log
    }

    /// Returns the starved log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn starved_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.starved_log)
    }

    /// Returns the cohorts of the last consumption pass.
    ///
    /// The table is derived from the home column of the units, and the pass
    /// derives it again on every application.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[must_use]
    pub const fn cohorts(&self) -> &CohortTable {
        &self.cohorts
    }

    /// Returns every draw that has run since the world was built.
    #[must_use]
    pub const fn draw_ledger(&self) -> DrawLedger {
        self.draw_ledger
    }

    /// Returns the sites that could not serve every cohort at the last
    /// draw, in slot order.
    #[must_use]
    pub fn rationed_log(&self) -> &[SiteRationed] {
        &self.rationed_log
    }

    /// Returns the rationed log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn rationed_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.rationed_log)
    }

    /// Gives a unit the site that it draws from, and reports whether it
    /// wrote.
    ///
    /// Returns `false` when either identity is dead. A unit that belongs to
    /// no site draws from nothing, and `None` puts it in that state.
    ///
    /// The world holds this call rather than the arena, because the arena
    /// holds no settlement column and cannot tell a live site from a dead
    /// one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_home_site(&mut self, soldier: Entity, site: Option<Entity>) -> bool {
        let home = match site {
            Some(site) => match self.settlements.slot_of(site) {
                Some(slot) => Some(slot),
                None => return false,
            },
            None => None,
        };
        self.soldiers.set_home(soldier, home)
    }

    /// Returns the dwelling that one unit lives in.
    ///
    /// Returns `None` when the identity is dead. Returns `Some(None)` when
    /// the unit lives nowhere, which is a state the world represents rather
    /// than an error.[^1]
    ///
    /// A unit lives where it draws from. The record that fixes a settlement
    /// to a tile and gives it the pooled store makes those one fact, so the
    /// world holds one column for both and never two.[^2] [^3]
    ///
    /// The world invariant holds that every home names a live settlement, so
    /// the second `None` means the unit lives nowhere and never means that
    /// its dwelling was lost.[^4]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: Findings register, FND-116. `docs/FINDINGS.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn dwelling_of(&self, unit: Entity) -> Option<Option<Entity>> {
        let home = self.soldiers.home(unit)?;
        Some(home.and_then(|slot| self.settlements.entity_at(slot)))
    }

    /// Returns every unit that lives in one dwelling.
    ///
    /// A household is derived. Nothing stores one, and no rule declares
    /// one.[^1] The members are every live unit whose home column entry names
    /// this dwelling, so a unit that takes a dwelling of its own leaves the
    /// household it was in by the same write that puts it in the new one.
    ///
    /// Returns `None` when the identity is dead. A dwelling that nobody lives
    /// in returns an empty list, which is an answer and not an error.
    ///
    /// The members come back in ascending slot order of the unit arena. That
    /// key is a property of storage, so no thread order reaches it.[^2]
    ///
    /// The call passes over the unit arena. A watcher that wants the
    /// headcount of a place reads the cohort table instead, which holds it
    /// per site and per faction without a pass.[^3]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
    /// [^2]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[must_use]
    pub fn household_of(&self, dwelling: Entity) -> Option<Vec<Entity>> {
        let mut members = Vec::new();
        if self.household_into(dwelling, &mut members) {
            Some(members)
        } else {
            None
        }
    }

    /// Writes every unit that lives in one dwelling into a buffer the caller
    /// owns, and reports whether the dwelling resolved.
    ///
    /// The answer is the answer of [`Self::household_of`]. A caller that asks
    /// about many dwellings hands the same buffer to each call rather than
    /// taking a new one each time. The buffer is cleared on every call,
    /// including the call that refuses, so a stale roster never survives a
    /// dead identity.
    pub fn household_into(&self, dwelling: Entity, members: &mut Vec<Entity>) -> bool {
        members.clear();
        let Some(slot) = self.settlements.slot_of(dwelling) else {
            return false;
        };
        // The identity resolved, so the slot must be live. The check is local,
        // because a reader that trusts an argument across two structures is
        // how a dead dwelling comes back holding a roster.
        if self.settlements.live_column().get(slot as usize) != Some(&1) {
            return false;
        }
        household::residents_of(&self.soldiers, slot, members);
        true
    }

    /// Returns the schedule that the site rates apply on.
    #[must_use]
    pub const fn economy_schedule(&self) -> RateSchedule {
        self.schedule
    }

    /// Sets the schedule that the site rates apply on.
    ///
    /// The interval is a parameter of the schedule. No kernel holds it as a
    /// constant, so a caller changes how often a store moves without
    /// touching the engine.[^1]
    ///
    /// A rate is what one tick earns, so raising the period does not raise
    /// what a site earns over a span of ticks.
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is
    /// above the range that the scaling multiply takes.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    pub fn set_economy_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
    }

    /// Returns the rate table of every site.
    #[must_use]
    pub const fn rates(&self) -> &RateTable {
        &self.rates
    }

    /// Returns every rate that has applied since the world was built.
    #[must_use]
    pub const fn rate_ledger(&self) -> RateLedger {
        self.rate_ledger
    }

    /// Returns the sites that could not pay at the last application.
    ///
    /// The log holds one event for each site and commodity that fell short,
    /// in slot order. It is empty on a tick that the schedule does not name.
    #[must_use]
    pub fn shortfall_log(&self) -> &[SiteShortfall] {
        &self.shortfall_log
    }

    /// Returns the shortfall log as bytes.
    ///
    /// The event type is plain data with declared padding, so the bytes are
    /// the same on every run and the determinism test compares them
    /// directly.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn shortfall_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.shortfall_log)
    }

    /// Returns the production rate of a settlement, for one commodity.
    ///
    /// Returns `None` when the identity is dead, and `None` when the
    /// commodity is outside the set.
    #[must_use]
    pub fn production_rate(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        let slot = self.settlements.slot_of(entity)?;
        self.rates.production(slot, commodity)
    }

    /// Returns the upkeep rate of a settlement, for one commodity.
    #[must_use]
    pub fn upkeep_rate(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        let slot = self.settlements.slot_of(entity)?;
        self.rates.upkeep(slot, commodity)
    }

    /// Writes the production rate of a settlement, for one commodity.
    ///
    /// The rate is what one tick earns. The schedule scales it to the
    /// amount of one application, so raising the period does not raise what
    /// a site earns over a span of ticks.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, and when the commodity
    /// is outside the set.
    pub fn set_production_rate(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<bool, RateError> {
        let Some(slot) = self.settlements.slot_of(entity) else {
            return Ok(false);
        };
        self.rates.open_to(self.settlements.slot_count());
        self.rates.set_production(slot, commodity, rate)?;
        Ok(true)
    }

    /// Writes the upkeep rate of a settlement, for one commodity.
    ///
    /// Upkeep is a rate above zero that subtracts. It is never a production
    /// rate below zero.[^1]
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, and when the commodity
    /// is outside the set.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    pub fn set_upkeep_rate(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<bool, RateError> {
        let Some(slot) = self.settlements.slot_of(entity) else {
            return Ok(false);
        };
        self.rates.open_to(self.settlements.slot_count());
        self.rates.set_upkeep(slot, commodity, rate)?;
        Ok(true)
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
        // A write from outside the rate pass changes what the stores hold, so
        // it must change the account by the same amount. The old quantity is
        // read before the write, because after the write it is gone.
        let before = self
            .settlements
            .store(entity)
            .and_then(|store| store.quantity(commodity));
        let wrote = self.settlements.set_store(entity, commodity, quantity)?;
        if wrote {
            let before = before.expect("the write resolved the commodity");
            let index = commodity.0 as usize;
            let change = i64::from(quantity.0) - i64::from(before.0);
            self.store_account[index] = sim_math::combine(self.store_account[index], Accum(change));
        }
        Ok(wrote)
    }

    /// Returns the characters of the world.
    ///
    /// The living character is one of the four fixed entity shapes, and it
    /// has its own column set. It carries no tile position.[^1] The shape
    /// declares the character tier at the type, so a caller may walk the
    /// population.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    /// [^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[must_use]
    pub const fn characters(&self) -> &CharacterArena {
        &self.characters
    }

    /// Creates a character in the world and returns their identity.
    ///
    /// The character is born on the current tick of the world.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, or when the
    /// faction is one the world does not have.
    pub fn create_character(&mut self, faction: FactionId) -> Result<Entity, CharacterError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a character of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(CharacterError::FactionAboveCeiling(faction));
        }
        self.characters.create(self.config.seed, faction, self.tick)
    }

    /// Bears a child of two characters and returns the identity of the
    /// child.
    ///
    /// The child is born on the current tick of the world. It takes the
    /// faction of its mother, and it records both parents. The record of
    /// descent keeps those edges after either parent is gone, so a watcher
    /// reads a dead parent through a living child.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when either parent is gone, when the two parents
    /// are one character, when the arena holds no free slot, or when the
    /// record of descent is full.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    pub fn bear_character(
        &mut self,
        mother: Entity,
        father: Entity,
    ) -> Result<Entity, CharacterError> {
        self.characters
            .bear(self.config.seed, mother, father, self.tick)
    }

    /// Returns the two parents of a living character.
    ///
    /// Returns `None` when the identity is dead. Returns a pair of absent
    /// parents when the character founds a line. The world invents no
    /// parent.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    #[must_use]
    pub fn character_parents(&self, entity: Entity) -> Option<Parents> {
        self.characters.parents(entity)
    }

    /// Returns every ancestor of a living character, in ascending birth
    /// order.
    ///
    /// Returns an empty list when the identity is dead and when the
    /// character founds a line. The order is explicit and it is the same on
    /// every run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn character_ancestors(&self, entity: Entity) -> Vec<DescentId> {
        let Some(id) = self.characters.descent_id(entity) else {
            return Vec::new();
        };
        self.characters.descent().ancestors(id)
    }

    /// Returns every descendant of a living character, in ascending birth
    /// order.
    ///
    /// Returns an empty list when the identity is dead and when the
    /// character has no child.
    #[must_use]
    pub fn character_descendants(&self, entity: Entity) -> Vec<DescentId> {
        let Some(id) = self.characters.descent_id(entity) else {
            return Vec::new();
        };
        self.characters.descent().descendants(id)
    }

    /// Returns the relation between two characters.
    ///
    /// The value is Wright's coefficient of relationship. A parent and a
    /// child give one half. Two characters with no ancestor in common give
    /// zero, and a character who founds a line therefore stands at zero to
    /// everybody.[^1]
    ///
    /// The value is a Q16.16 fixed-point number and it is exact. Every step
    /// of the recursion halves a value, so no step rounds.[^2]
    ///
    /// Returns zero when either identity is dead.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^2]: The character graph and inheritance, section 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
    #[must_use]
    pub fn character_relation(&self, left: Entity, right: Entity) -> Fix32 {
        let (Some(left), Some(right)) = (
            self.characters.descent_id(left),
            self.characters.descent_id(right),
        ) else {
            return Fix32::ZERO;
        };
        self.characters.descent().relation(left, right)
    }

    /// Removes a character and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`. The identity
    /// of a character who is gone never resolves again, so the character
    /// created next in that slot does not answer to it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn remove_character(&mut self, entity: Entity) -> bool {
        self.characters.remove(entity)
    }

    /// Writes the renown of a character and reports whether it wrote.
    ///
    /// Returns `false` when the identity is dead. A renown of zero is a
    /// real state, so a write of zero is a write.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    pub fn set_character_renown(&mut self, entity: Entity, renown: Fix32) -> bool {
        self.characters.set_renown(entity, renown)
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
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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

    /// Resolves the value of an identity back to the soldier it names.
    ///
    /// A caller outside this crate holds an identity as the value that
    /// [`Entity::to_bits`] gave. It cannot build one, and this is the only
    /// way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds for the slot. It refuses a mismatch. It
    /// never returns the soldier that now occupies the slot, because that
    /// soldier is not the one the caller named.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an identity, when the arena
    /// holds no such slot, or when the slot holds a later generation.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D2 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn resolve_soldier(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.soldiers.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.soldiers.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.soldiers.generation_of(slot),
        })
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
        // Read the load before the arena clears it. What the soldier carried
        // leaves the world, and conservation still has to balance, so the
        // world records where it went.[^2]
        //
        // [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        let load = self.soldiers.carry(entity);
        if !self.soldiers.despawn(entity) {
            return false;
        }
        if let Some(load) = load {
            for kind in ResourceKind::ALL {
                self.departed[kind.index()] += u64::from(load.of(kind).0);
            }
        }
        true
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
        self.values.total()
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
            .write_u64(u64::from(self.config.faction_count));
        // The tile value field is generated from the seed and stores only
        // what the frames changed. The hash writes the value of every tile
        // and not the stored part alone: the seed and the extent above are
        // the inputs of the generator, and a change to the generator moves
        // every tile of every world while leaving both untouched.[^3]
        //
        // [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        let hash = self.values.hash_into(hash);
        // The ground is part of the world, so the whole-world hash covers
        // it. The seed and the extent are already above, but they are the
        // inputs of the generator, not its output. A change to the generator
        // moves every tile of every world, and only the tiles report it.[^1]
        //
        // [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.terrain.hash_into(hash);
        // The stock a tile started with is generated, so the same argument
        // holds for it: the seed is the input of the generator and only the
        // tiles report a change to the generator itself.
        let hash = self.resources.hash_into(hash);
        let mut hash = self.depletion.hash_into(hash);
        for amount in &self.departed {
            hash = hash.write_u64(*amount);
        }
        // An upgrade is the difference between the world the generator made
        // and the world the units made. It is simulated state, and an
        // unfinished build is state that the next frame reads, so both the
        // kind and the progress enter the hash.[^2]
        //
        // [^2]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
        let hash = self.upgrades.hash_into(hash);
        // Who holds each tile is simulated state, so the whole-world hash
        // covers it.
        let hash = self.holding.hash_into(hash);
        let hash = self.soldiers.hash_into(hash);
        let hash = self.settlements.hash_into(hash);
        let hash = self.characters.hash_into(hash);
        // A rate is state that a later frame reads, and the ledger is the
        // record of what the rates have already done. A hash that covered
        // the stores and neither of these would report the same value for
        // two worlds that must diverge on the next application.
        let hash = self
            .rates
            .hash_into(hash)
            .write_u64(u64::from(self.schedule.period()))
            .write_u64(u64::from(self.schedule.phase()));
        let hash = self.rate_ledger.hash_into(hash);
        // The need of a unit and the deficit that follows it are simulated
        // state, and the unit columns already carry them into the hash. The
        // rule, the cohorts and the draw ledger are the rest of the pass:
        // two worlds that hold the same needs and different rules must
        // diverge on the next application.
        let hash = self.need_rule.hash_into(hash);
        let hash = self.cohorts.hash_into(hash);
        // What each faction reaches is state that a later frame reads: the
        // next solve starts from the field this one left. Two worlds that
        // hold the same tiles and different fields must diverge.
        let hash = self.influence.hash_into(hash);
        // A position is state that a later frame reads: a unit that holds
        // one still holds it on the next frame, and the preference decides
        // what the next rebalance opens. Two worlds that hold the same
        // stores and different preferences must diverge.
        let hash = self
            .positions
            .hash_into(hash)
            .write_u64(u64::from(self.position_schedule.period()))
            .write_u64(u64::from(self.position_schedule.phase()));
        let mut hash = self.draw_ledger.hash_into(hash);
        for total in &self.store_account {
            hash = hash.write_u64(total.0 as u64);
        }
        hash
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
        // The field holds one entry for each tile a frame changed, in
        // ascending tile order. A lookup is a binary search, so an entry out
        // of order does not fail. It returns the wrong tile.
        if !self.values.check_invariants() {
            return false;
        }
        if self.values.grid() != self.grid {
            return false;
        }
        if self.grid.width() != self.config.width || self.grid.height() != self.config.height {
            return false;
        }
        // The influence lattice and the block layout state the shape of
        // level 1, and they state it in two places. This is what fails when
        // the two disagree.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.influence.cells().tile_count() != self.pyramid.layout().block_count()
            || self.influence.faction_count() != self.config.faction_count
        {
            return false;
        }

        // The upgrade map rises, names each tile once, names a tile inside
        // the world, and banks no progress beyond the work its kind asks
        // for.[^2]
        //
        // [^2]: Findings register, FND-011. `docs/FINDINGS.md`
        if !self.upgrades.check_invariants(self.grid.tile_count()) {
            return false;
        }
        let ceiling = self.config.faction_count.max(1);
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
        // [^4]: ADR-0023, an aggregate combines exactly, in any order, decision D5. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
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
        // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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
        // The character faction column stands under the same ceiling as
        // every other faction column.
        if self
            .characters
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        if !self.characters.check_invariants() {
            return false;
        }
        if !self.check_store_conservation() {
            return false;
        }
        if !self
            .shortfall_log
            .iter()
            .all(|event| event.padding == [0; 2] && event.amount.0 > 0)
        {
            return false;
        }
        if !self.check_cohorts() {
            return false;
        }
        if !self.check_positions() {
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
        // The holding covers the same world, no tile names a faction the
        // world does not have, and no faction holds ground that admits no
        // unit. The check derives the held list, the census and the block
        // masks again and compares them against the stored ones.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.holding.grid() != self.grid {
            return false;
        }
        if !self.holding.check_invariants(self.terrain, ceiling) {
            return false;
        }
        // Level 1 and the running census are two statements of how much
        // ground is held. The check fails when they disagree.
        let census: i64 = (0..ceiling)
            .map(|faction| self.holding.holding_of(FactionId(faction)))
            .sum();
        if census != self.holding.held_tiles() {
            return false;
        }
        if self.bridge.describes(&self.soldiers).is_ok() && total.held_tiles() != census {
            return false;
        }
        if !self.check_conservation() {
            return false;
        }
        if !self
            .gather_log
            .iter()
            .all(|event| event.padding == [0; 7] && (event.tile.0 as usize) < self.tile_count())
        {
            return false;
        }
        self.log
            .iter()
            .all(|event| event.padding == [0; 5] && (event.tile.0 as usize) < self.tile_count())
    }

    /// Reports whether the store column agrees with the account of it.
    ///
    /// What a site held, plus what production put in, minus what upkeep
    /// took, is what the site holds. This check states that equality over
    /// every live site at once.
    ///
    /// The account moves at four places: a write from the control plane,
    /// the loss of a settlement, the rate pass, and nowhere else. A fifth
    /// place that changes a store and forgets the account fails here, and it
    /// fails whatever the thread count was, because a rule that leaks the
    /// same amount on every run repeats perfectly and no determinism test
    /// can see it.[^1]
    ///
    /// The check is exact. Every term is a whole number in a 64-bit
    /// accumulator, so the sum is the same in any order and nothing
    /// rounds.[^2]
    ///
    /// The rate table also states a slot count that the arena already
    /// holds. A check must fail when the two copies disagree.[^3]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-048. `docs/FINDINGS.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    fn check_store_conservation(&self) -> bool {
        if !self.rates.check_invariants() {
            return false;
        }
        if self.rates.slot_count() < self.settlements.slot_count() {
            return false;
        }
        let mut held = [Accum(0); COMMODITY_COUNT];
        for settlement in self.settlements.iter() {
            let Some(store) = self.settlements.store(settlement) else {
                return false;
            };
            for (index, total) in held.iter_mut().enumerate() {
                let Some(quantity) = store.quantity(CommodityId(index as u16)) else {
                    return false;
                };
                *total = sim_math::accumulate(*total, quantity);
            }
        }
        held == self.store_account
    }

    /// Reports whether the positions of every site hold their rules.
    ///
    /// Three statements must hold together, and each of them is a place
    /// where one fact could be stored twice.[^1]
    ///
    /// The table and the settlement arena state the same slot count. Every
    /// position names a unit that still exists, so a holder that died leaves
    /// no stale identity behind.[^2] No site holds more positions than the
    /// ground under it admits, and both bounds come from the terrain
    /// capacity table.[^3]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    fn check_positions(&self) -> bool {
        if self.positions.slot_count() != self.settlements.slot_count() {
            return false;
        }
        if !self.positions.check_invariants() {
            return false;
        }
        if !self.positions.check_holders(&self.soldiers) {
            return false;
        }
        matches!(
            self.positions.check_capacity(
                self.settlements.tile_column(),
                self.settlements.live_column(),
                self.terrain,
            ),
            Ok(true)
        )
    }

    /// Reports whether the cohort table and the home column agree.
    ///
    /// The table is a summary of the home column of the units, and the pass
    /// derives it again on every application. Between two applications a
    /// spawn or a home write leaves it behind, in the same way that a
    /// structural change leaves the derived unit structure stale.[^1] This
    /// check therefore states what is true at every moment: the table holds
    /// its own key, every home names a live site, and every reported event
    /// says what it means.
    ///
    /// The equality between the headcounts and the population is true right
    /// after an application, and a test asserts it there.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: Testing rules, section 5. `.claude/rules/testing.md`
    #[must_use]
    fn check_cohorts(&self) -> bool {
        if !self.cohorts.check_invariants() {
            return false;
        }
        // Every home names a live site. A home left on a lost site would
        // feed the settlement founded next in that slot.
        for (slot, home) in self.soldiers.home_column().iter().enumerate() {
            if self.soldiers.live_column()[slot] != 1 || *home == crate::soldier::NO_HOME {
                continue;
            }
            if self.settlements.live_column().get(*home as usize) != Some(&1) {
                return false;
            }
        }
        if !self
            .rationed_log
            .iter()
            .all(|event| event.padding == [0; 6] && event.granted.0 < event.demanded.0)
        {
            return false;
        }
        // A starved unit is dead by the time anyone reads the log, so the
        // check states what the event itself must hold: declared padding,
        // an identity that packs, and a deficit that reached the bound.
        self.starved_log.iter().all(|event| {
            event.padding == [0; 4]
                && event.unit != 0
                && self.need_rule.condition(event.deficit) == NeedCondition::Starved
        })
    }

    /// Reports whether the cohorts describe the unit columns.
    ///
    /// The check derives the table again from the home column and compares.
    /// A summary that nothing compares against its source is a second
    /// declaration site with nothing that fails on disagreement.[^1]
    ///
    /// The answer is true right after an application of the consumption
    /// pass, and it is false after a spawn that no application has seen.
    /// The caller states which moment it means.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn cohorts_describe_the_units(&self) -> bool {
        self.cohorts.describes(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        )
    }

    /// Reports whether the world conserves every resource.
    ///
    /// What left the tiles equals what the live units carry, plus what left
    /// the world in the hands of a dead unit. The equality holds for each kind
    /// on its own, because a gather never turns one kind into another.[^1]
    ///
    /// The check is exact. Every term is a whole number in a 64-bit
    /// accumulator, so the sum is the same in any order and nothing rounds.[^2]
    ///
    /// A determinism test cannot see a broken invariant, because a rule that
    /// leaks the same amount on every run repeats perfectly.[^3] This check is
    /// what fails instead.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: Findings register, FND-048. `docs/FINDINGS.md`
    #[must_use]
    fn check_conservation(&self) -> bool {
        if !self.depletion.check_invariants() {
            return false;
        }
        let mut left_the_tiles = [0i64; RESOURCE_KIND_COUNT];
        for entry in self.depletion.entries() {
            let (key, amount) = (&entry.key, &entry.taken);
            let Some(kind) = ResourceKind::from_u8((key & 0b11) as u8) else {
                return false;
            };
            let tile = TileIdx((key >> 2) as u32);
            // Nothing takes more from a tile than the tile ever held.
            let Some(original) = self.resources.original_at(tile, kind) else {
                return false;
            };
            if *amount > original.0 {
                return false;
            }
            left_the_tiles[kind.index()] += i64::from(*amount);
        }
        let mut arrived = [0i64; RESOURCE_KIND_COUNT];
        for soldier in self.soldiers.iter() {
            let Some(load) = self.soldiers.carry(soldier) else {
                return false;
            };
            for kind in ResourceKind::ALL {
                arrived[kind.index()] += i64::from(load.of(kind).0);
            }
        }
        for kind in ResourceKind::ALL {
            let index = kind.index();
            // Recovery gives a part of the take back to the tile, so the
            // stored take alone no longer balances what the units hold. The
            // returned total is the second term, and it is what makes the
            // equality hold across a recovery.
            let returned = self.depletion.returned(kind).0;
            if left_the_tiles[index] + returned != arrived[index] + self.departed[index] as i64 {
                return false;
            }
        }
        true
    }

    /// Writes the intent of every unit whose cell chooses on this frame.
    ///
    /// The pass is one operation over all units. Nothing loops over units
    /// outside the engine.[^1]
    ///
    /// **The pass walks the lattice, and it never walks the population.** It
    /// divides the level 1 cells into contiguous ranges, and each thread takes
    /// one range. A thread skips a cell that does not choose on this frame and
    /// a cell that holds no unit, so the deciding work follows the cell count
    /// and the population cannot raise it.[^7] The earlier shape collected
    /// every live unit into one list before any thread started, and that
    /// collect was serial and grew with the population.[^8]
    ///
    /// **The engine computes one answer once for every unit that would compute
    /// the same answer.**[^9] A cell holds one answer table over the buckets of
    /// need, and a unit reads the entry for its bucket. The table fills as a
    /// unit asks, so a cell never scores more buckets than it holds units, and
    /// it never scores more than the bucket count.[^10]
    ///
    /// **The pass writes the gather order in the same write as the intent.**
    /// One pass writes both, for the same units, on the same frame. A second
    /// stage that derived the order from the option would be a second writer
    /// of one column, and nothing would fail when the two disagreed.[^4] The
    /// kind that an option gathers comes from the option row, which is the one
    /// declaration of that map.[^5]
    ///
    /// A unit whose cell does not choose on this frame keeps the intent it
    /// held, and it keeps the gather order it held. A control-plane order
    /// therefore survives until the cell of that unit next chooses, and the
    /// choice then replaces it.[^6]
    ///
    /// A unit whose every option scores below the floor holds what it
    /// was doing, which is the case the floor exists for.[^2] It holds no
    /// intent, so it takes no gather order either.
    ///
    /// Each thread reads a range of the lattice and writes its own output
    /// slot. The join reads the slots in slot order, the cells of a slot rise,
    /// and the derived unit structure orders the units of a cell. So the
    /// result takes its order from the lattice and never from the thread that
    /// finished first.[^3]
    ///
    /// The apply walks that same order. It is the one part that touches every
    /// unit that chose, and applying an answer to a unit is per-unit by
    /// necessity.[^7]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, and when the
    /// derived unit structure no longer describes the arena.
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    /// [^2]: Findings register, FND-014. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^5]: Findings register, FND-191. `docs/FINDINGS.md`
    /// [^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^7]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D3. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^8]: Findings register, FND-252. `docs/FINDINGS.md`
    /// [^9]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^10]: ADR-0098, the choice is decided for each cell and each bucket of need. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    fn choose(&mut self, threads: usize) -> Result<(), StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }
        // The occupancy bitplane and the block range are unguarded reads.
        // They answer from the last rebuild and cannot refuse a stale
        // question, so a pass that skipped a cell on a stale bitplane would
        // skip it in silence. Ask the guarded question once, here, before
        // any thread trusts the shape.
        self.bridge.describes(&self.soldiers)?;
        let layout = self.pyramid.layout();
        let cells = layout.block_count();
        if cells == 0 {
            return Ok(());
        }
        let frame = self.tick.0;
        let schedule = self.choice;
        let weights = &self.weights;
        let buckets = self.buckets;
        let pyramid = &self.pyramid;
        let bridge = &self.bridge;
        let soldiers = &self.soldiers;
        let chunk_len = (cells as usize).div_ceil(threads).max(1) as u32;
        let mut slots: Slots<Vec<(Entity, u8)>> =
            Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

        std::thread::scope(|scope| {
            let mut start = 0u32;
            for slot in slots.entries_mut() {
                if start >= cells {
                    break;
                }
                let end = start.saturating_add(chunk_len).min(cells);
                scope.spawn(move || {
                    let needs = soldiers.need_column();
                    let mut chosen: Vec<(Entity, u8)> = Vec::new();
                    for cell in start..end {
                        // The stagger key is the level 1 cell. It is never
                        // the identity of the unit.
                        if !schedule.chooses_now(cell, frame) {
                            continue;
                        }
                        if !bridge.block_is_occupied(cell) {
                            continue;
                        }
                        let Some(summary) = pyramid.cell(cell) else {
                            continue;
                        };
                        let units = bridge
                            .in_block(soldiers, cell)
                            .expect("the caller checked that the bridge describes this arena");
                        let mut answers = choose::CellAnswers::new(summary, buckets);
                        for unit in units {
                            let need = needs[unit.index() as usize];
                            chosen.push((*unit, answers.answer(need, weights)));
                        }
                    }
                    *slot = chosen;
                });
                start = end;
            }
        });

        let chosen = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        for (unit, intent) in chosen {
            self.soldiers.set_intent_at(unit.index(), intent);
            // The same write. A unit that chose the option which gathers
            // holds an order for that kind, and a unit that chose anything
            // else, or nothing, holds none.
            self.soldiers
                .set_gather_order(unit, choose::gathers(intent));
        }
        Ok(())
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
        let count = self.grid.tile_count();
        let chunk_len = (count as usize).div_ceil(threads).max(1) as u32;

        let mut slots: Slots<ChunkResult> =
            Slots::filled(threads, ChunkResult::default()).map_err(|_| StepError::ZeroThreads)?;

        // Each worker reads one contiguous range of tiles and writes to its
        // own slot. No worker writes to the field, because the merge that
        // follows the join is the one place that fixes the order.[^12]
        //
        // [^12]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/REGISTRY.md`
        {
            let _span = stage::open(Stage::TileScan);
            let values = &self.values;
            std::thread::scope(|scope| {
                let mut start = 0u32;
                for slot in slots.entries_mut() {
                    if start >= count {
                        break;
                    }
                    let end = start.saturating_add(chunk_len).min(count);
                    let range = values.range(start, end);
                    scope.spawn(move || {
                        *slot = update_range(tick, seed, start, end, range);
                    });
                    start = end;
                }
            });
        }

        {
            let _span = stage::open(Stage::LogJoin);
            let mut log = core::mem::take(&mut self.log);
            log.clear();
            self.log = slots.combine(log, |mut joined, slot| {
                joined.extend_from_slice(&slot.events);
                joined
            });
        }

        // The ranges are disjoint, so each tile appears in one run at most.
        // The merge needs one ascending run, and the sort by tile index is
        // what gives it one. Nothing here reads the order the slots joined
        // in, so the field is the same at any thread count.[^13]
        //
        // [^13]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        {
            let _span = stage::open(Stage::ChangeMerge);
            let mut run: Vec<(u32, Fix32)> = slots.combine(Vec::new(), |mut joined, slot| {
                joined.extend_from_slice(&slot.changes);
                joined
            });
            run.sort_unstable_by_key(|pair| pair.0);
            self.values.merge_ascending(&run);
        }

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
        {
            let _span = stage::open(Stage::BridgeRefreshOpening);
            self.refresh_bridge()?;
        }

        // The choice runs before movement, and it is what movement reads.
        // It reads level 1 as the last barrier left it, and it writes
        // nothing to any level above level 0.[^11]
        //
        // [^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::Choose);
            self.choose(threads)?;
        }

        let intents = {
            let _span = stage::open(Stage::MovementIntents);
            soldier_moves(
                tick,
                seed,
                self.terrain,
                &self.soldiers,
                self.pyramid.layout(),
                &self.exits,
                threads,
            )?
        };

        // Admission grants the intents. It reads the occupancy of a target
        // from the derived structure, which the last barrier rebuilt, so it
        // must run before anything moves.[^3]
        let granted = {
            let _span = stage::open(Stage::Admit);
            admit(
                &intents,
                &self.soldiers,
                &self.bridge,
                self.terrain,
                &self.upgrades,
                self.grid,
                threads,
            )?
        };
        {
            let _span = stage::open(Stage::PlaceGranted);
            for (soldier, address) in granted {
                self.soldiers
                    .place(soldier, address)
                    .expect("the granted address is inside the world and admits a unit");
            }
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
        {
            let _span = stage::open(Stage::BridgeRefreshBarrier);
            self.refresh_bridge()?;
        }

        // The gather resolve runs after the barrier of this frame. It reads
        // where each unit stands, and the movement above has just moved
        // them, so a resolve before the barrier would take from the tile the
        // unit left.[^6]
        //
        // The resolve changes no structure. It writes a load into a column
        // and an amount into the ledger, and neither moves a unit, so the
        // barrier above stays the barrier of this frame.
        //
        // [^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
        // Recovery runs before the gather resolve, so a unit takes what the
        // deposit holds at this tick. A resolve that ran first would take
        // against an amount that the world had already moved past.[^14]
        //
        // The pass walks the depleted set and no tile, so a world that
        // gathered nothing does no work here, at any tile count.[^14]
        //
        // [^14]: ADR-0080, a depleted deposit recovers by ageing the stored take, decisions D1 and D2. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
        {
            let _span = stage::open(Stage::DepletionRecover);
            self.depletion.recover(tick);
        }

        {
            let _span = stage::open(Stage::Gather);
            self.gather(threads)?;
        }

        // The build advance runs after the barrier of this frame, for the
        // same reason the gather resolve does: it reads where each unit
        // stands, and the movement above has just moved them.[^16]
        //
        // The advance writes the upgrade map and nothing else. It moves no
        // unit, so the barrier above stays the barrier of this frame.
        //
        // The pass reads the builders and the sites. It takes no grid and no
        // tile count, so a world in which nobody built does no work here, at
        // any tile count.[^16]
        //
        // [^16]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D1 and D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
        {
            let _span = stage::open(Stage::Build);
            self.build(threads)?;
        }

        // The holding spreads after the barrier of this frame, because it
        // reads where each unit stands and the movement above has just moved
        // them. It writes a tile column, and it moves no unit, so the barrier
        // above stays the barrier of this frame.[^7]
        //
        // [^7]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
        {
            let _span = stage::open(Stage::HoldingSpread);
            self.holding
                .advance(self.terrain, &self.soldiers, &self.bridge, threads)?;
        }

        // The event reports the tile as this frame left it, so the holder is
        // stamped here and not in the value pass above. The value pass runs
        // at the top of the step and the spread above is the last thing in
        // this step that writes the holder column, so a stamp taken any
        // earlier would publish the holder of the frame before. A stale read
        // is a confident wrong answer, and it is the defect this event
        // carried.[^15]
        //
        // The pass costs one write for each event, and not one for each
        // tile.
        //
        // [^15]: Findings register, FND-029 and FND-079. `docs/FINDINGS.md`
        {
            let _span = stage::open(Stage::StampHolders);
            let holders = self.holding.holders();
            for event in &mut self.log {
                event.holder = holders[event.tile.0 as usize];
            }
        }
        // The site rates apply after the barrier of this frame and after the
        // gather resolve, and before level 1 rebuilds.
        //
        // The position is stated against the barrier on purpose. The pass
        // reads no derived structure and changes no structure, so it is not a
        // barrier and it does not need one. What it needs is to run after
        // everything that moves a quantity in this frame, so that the store a
        // derived level reads is the store the frame settled on. The gather
        // resolve is that work today, and level 1 is the derived level.[^8]
        //
        // The pass is skipped on a tick the schedule does not name, and the
        // schedule is a parameter of the world rather than a constant of this
        // function.[^9]
        //
        // [^8]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        // [^9]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        {
            let _span = stage::open(Stage::ApplyRates);
            self.apply_rates(threads)?;
        }

        // Consumption runs after the rates, on the same schedule. A unit
        // draws from the store of the site it belongs to, and the rates are
        // what filled that store this frame, so a draw before them would
        // spend the store of the frame before.[^10]
        //
        // The pass reads no derived structure and changes no structure, so
        // it is not a barrier. It reads the home column and the store
        // column, and it writes the need column, the deficit column and the
        // store column.
        //
        // [^10]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        {
            let _span = stage::open(Stage::Consume);
            self.consume(threads)?;
        }

        // The scan of the death plane runs after consumption, because
        // consumption is what moves a deficit to the bound. It is a
        // structural change, so it is batched into the plane during the
        // pass and applied here, in one ascending scan, after the frame has
        // settled.[^12]
        //
        // The scan removes units, so the derived structure that the barrier
        // above rebuilt now names a dead identity. The refresh below is that
        // barrier taken again over the structural apply, and it must run
        // before the derived level reads either of them.[^13]
        //
        // [^12]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
        // [^13]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::Reap);
            self.reap(threads)?;
        }
        {
            let _span = stage::open(Stage::BridgeRefreshAfterReap);
            self.refresh_bridge()?;
        }

        // The positions of the sites settle after the deaths of this frame.
        // A position that named a unit the scan above ended would hold a
        // stale identity, and the invariant check refuses that state.[^17]
        //
        // [^17]: ADR-0065, a group is a site membership, not a region, decision D2. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
        {
            let _span = stage::open(Stage::SettlePositions);
            self.settle_positions(threads)?;
        }

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
        // [^5]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::RebuildLevel1);
            self.rebuild_level_1(threads)?;
        }

        // The influence solve runs last, after every change this frame made
        // and after the derived level it reads was rebuilt. It runs the same
        // fixed number of passes whatever the field holds and whatever the
        // sources hold, and it takes no branch on whether a source
        // exists.[^16]
        //
        // [^16]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
        {
            let _span = stage::open(Stage::InfluenceSolve);
            self.influence.solve(threads)?;
        }
        Ok(&self.log)
    }

    /// Returns when each unit re-reads the world and chooses again.
    #[must_use]
    pub const fn choice_schedule(&self) -> ChoiceSchedule {
        self.choice
    }

    /// Returns how finely the choice tells two needs apart.
    #[must_use]
    pub const fn need_buckets(&self) -> NeedBuckets {
        self.buckets
    }

    /// Sets the width of a need bucket, as a power of two.
    ///
    /// **This changes what a unit does.** Two units whose needs share a bucket
    /// receive one answer, so a wider bucket makes two units of different need
    /// act alike and a narrower one approaches one answer for each unit.[^1]
    /// The reference table holds the value a world starts with and the
    /// derivation of it, and an open decision holds the choice of a better
    /// one.[^2] [^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is outside the range that the answer
    /// table holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    /// [^2]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    /// [^3]: Decisions register, DEC-097. `docs/DECISIONS.md`
    pub const fn set_need_buckets(&mut self, shift: u32) -> Result<(), ChoiceError> {
        match NeedBuckets::new(shift) {
            Ok(buckets) => {
                self.buckets = buckets;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Sets the interval between two choices, as a power of two.
    ///
    /// An exponent of zero makes every unit choose on every tick. The
    /// interval is a parameter of the world. This function holds no
    /// recommended value, and the reference table holds the derivation.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is above the ceiling.
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    pub const fn set_choice_schedule(&mut self, period_log2: u32) -> Result<(), ChoiceError> {
        match ChoiceSchedule::new(period_log2) {
            Ok(schedule) => {
                self.choice = schedule;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the weight that a unit puts on one option.
    ///
    /// Returns `None` when the option index is outside the set.
    #[must_use]
    pub const fn option_weight(&self, option: u8) -> Option<Fix32> {
        self.weights.weight(option)
    }

    /// Sets the weight that a unit puts on one option.
    ///
    /// The weight is content: a value in a table that the engine reads. The
    /// engine never calls content code inside the choice.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the option index is outside the set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    pub const fn set_option_weight(
        &mut self,
        option: u8,
        weight: Fix32,
    ) -> Result<(), ChoiceError> {
        self.weights.set(option, weight)
    }

    /// Returns the option that one soldier last chose.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier holds an intent. A soldier that holds
    /// none found nothing above the floor, and it does not move.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    #[must_use]
    pub fn soldier_intent(&self, entity: Entity) -> Option<Option<u8>> {
        self.soldiers.intent(entity)
    }

    /// Returns the level 1 cell that covers one tile.
    #[must_use]
    fn cell_of(&self, tile: TileIdx) -> Option<u32> {
        let layout = self.pyramid.layout();
        Some(layout.block_of_key(layout.key_of(tile)?))
    }

    /// Returns why one soldier chose what it chose.
    ///
    /// The answer holds every score, the value each option read from the
    /// level 1 cell, the weight each option carried, and the floor that an
    /// option had to clear. The engine recomputes it from the world as it
    /// stands now, because it stores no score.[^1]
    ///
    /// Returns `None` when the identity is dead or names no tile of this
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    #[must_use]
    pub fn explain_choice(&self, entity: Entity) -> Option<ChoiceExplanation> {
        let slot = self.soldiers.slot_of(entity)?;
        let tile = self.soldiers.tile(entity)?;
        let cell = self.cell_of(tile)?;
        let summary = self.pyramid.cell(cell)?;
        let need = self.soldiers.need_column()[slot as usize];
        let intent = self.soldiers.intent_column()[slot as usize];
        Some(choose::explain(
            cell,
            need,
            summary,
            &self.weights,
            self.buckets,
            intent,
            self.choice.chooses_now(cell, self.tick.0.wrapping_add(1)),
        ))
    }

    /// Returns the holding of the world.
    ///
    /// The holding says who holds each tile, and how much ground each
    /// faction holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn holding(&self) -> &Holding {
        &self.holding
    }

    /// Returns every upgrade in the world, in ascending tile order.
    ///
    /// A world in which nobody built returns an empty slice. The map holds
    /// one entry for each improved tile and none for any other, so the length
    /// of this slice is the whole storage cost of the upgrades.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub fn upgrade_sites(&self) -> &[UpgradeSite] {
        self.upgrades.sites()
    }

    /// Returns the upgrade on one tile, finished or under construction.
    ///
    /// Returns `None` when the address lies outside the world, and when the
    /// tile carries no upgrade.
    #[must_use]
    pub fn upgrade_at(&self, address: Axial) -> Option<UpgradeSite> {
        self.upgrades.at(self.grid.index_of(address)?)
    }

    /// Returns the finished upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none, and when the upgrade there
    /// is still under construction. An unfinished build changes nothing about
    /// the tile.
    #[must_use]
    pub fn finished_upgrade(&self, address: Axial) -> Option<UpgradeKind> {
        self.upgrades.finished(self.grid.index_of(address)?)
    }

    /// Returns the number of units that may stand on one tile.
    ///
    /// This is the one reader of the ground table and the upgrade table
    /// together. Admission calls the same function, so no caller can read one
    /// table without the other.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub fn tile_capacity(&self, address: Axial) -> Option<u32> {
        let ground = self.terrain.kind(address)?.capacity();
        Some(upgrade::capacity_with(
            ground,
            self.finished_upgrade(address),
        ))
    }

    /// Returns the number of entries that the last build advance read.
    ///
    /// The advance reads the builders and the sites. It reads no tile, so
    /// this number does not grow with the size of the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub const fn last_build_visits(&self) -> u64 {
        self.upgrades.last_advance_visits()
    }

    /// Tells one soldier to build, or to stop building.
    ///
    /// The soldier adds to the upgrade on the tile it stands on, on every
    /// tick, until something stops it. It does not have to stay: a soldier
    /// that walks away stops adding, and the work it did stays on the
    /// tile.[^1]
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn order_build(&mut self, entity: Entity, kind: UpgradeKind) -> bool {
        self.soldiers.set_build_order(entity, Some(kind))
    }

    /// Tells one soldier to stop building.
    ///
    /// Returns `false` when the identity is dead.
    pub fn stop_build(&mut self, entity: Entity) -> bool {
        self.soldiers.set_build_order(entity, None)
    }

    /// Returns the build order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier builds.
    #[must_use]
    pub fn build_order(&self, entity: Entity) -> Option<Option<UpgradeKind>> {
        self.soldiers.build_order(entity)
    }

    /// Removes the upgrade from one tile and reports whether it removed one.
    ///
    /// The tile returns to the world the generator made. Nothing stores a
    /// property of an improved tile except this map, so removing the entry is
    /// the whole of the return and no second copy can survive it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn destroy_upgrade(&mut self, address: Axial) -> bool {
        let Some(tile) = self.grid.index_of(address) else {
            return false;
        };
        self.upgrades.remove(tile).is_some()
    }

    /// Returns who holds one tile.
    ///
    /// The answer names a faction, or nobody. It never names two factions,
    /// because a tile carries one holder.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn tile_holder(&self, address: Axial) -> Option<Holder> {
        self.holding.holder(address)
    }

    /// Returns the number of tiles one faction holds.
    ///
    /// The call reads a running total, so it costs the same whatever the
    /// size of the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn holding_of(&self, faction: FactionId) -> i64 {
        self.holding.holding_of(faction)
    }

    /// Returns the factions that hold ground in the block covering a tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn holders_near(&self, address: Axial) -> Option<FactionMask> {
        let tile = self.grid.index_of(address)?;
        let key = self.holding.layout().key_of(tile)?;
        self.holding
            .block_mask(self.holding.layout().block_of_key(key))
    }

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
        let block = self.cell_of(tile)?;
        self.influence.cells().address_of(TileIdx(block))
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
        self.rebuild_level_1(threads)?;
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
    fn rebuild_level_1(&mut self, threads: usize) -> Result<(), BridgeError> {
        self.pyramid.rebuild(
            &self.values,
            self.holding.holders(),
            &self.soldiers,
            &self.bridge,
            &self.depletion,
            threads,
        )?;
        self.exits.derive(&self.pyramid);
        Ok(())
    }

    /// Resolves every gather order of the frame in one pass.
    ///
    /// The resolve sorts the intents by the deposit they name, then by the
    /// identity of the unit. Each deposit then owns one contiguous segment,
    /// and the identity is the final key field so no two intents tie.[^1] The
    /// sort runs on one thread, so no result here takes its order from a
    /// thread that finished first.[^2]
    ///
    /// The resolve scans each segment in its sorted order and grants until the
    /// deposit is empty. A unit that reaches an empty deposit takes nothing
    /// and produces no event. One pass over the sorted intents resolves the
    /// whole set, so the cost follows the number of units that gather and not
    /// the number of deposits.[^3]
    ///
    /// **The resolve never locks a tile and never retries.** Two units that
    /// name one deposit sit in one segment, and the sort decides which of them
    /// takes the last of it.[^1]
    ///
    /// What leaves each deposit goes to the ledger, and the same amount goes
    /// into the load of the unit. The two writes come from one grant, so
    /// nothing is created and nothing is lost.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the sort refuses the keys.
    ///
    /// # References
    ///
    /// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    fn gather(&mut self, threads: usize) -> Result<(), StepError> {
        self.gather_log.clear();
        let intents = gather_intents(&self.soldiers, threads)?;
        if intents.is_empty() {
            return Ok(());
        }

        let keys: Vec<BoundedKey> = intents
            .iter()
            .map(|intent| {
                BoundedKey::new(ledger_key(intent.tile, intent.kind), intent.unit.to_bits())
            })
            .collect();
        let last = TileIdx(self.grid.tile_count().saturating_sub(1));
        let ceiling = ledger_key(last, ResourceKind::ALL[RESOURCE_KIND_COUNT - 1]);
        let order = gather_order_of(&keys, ceiling)?;

        let tick = self.tick;
        // The ascending run that the ledger merges. The sorted order is the
        // key order, so a run built while walking it is already ascending.
        let mut run: Vec<(u64, u32)> = Vec::new();
        let mut at = 0usize;
        while at < order.len() {
            let key = keys[order[at] as usize].order();
            let mut end = at;
            while end < order.len() && keys[order[end] as usize].order() == key {
                end += 1;
            }
            let first = intents[order[at] as usize];
            // The deposit is read once for the whole segment. The stock a tile
            // started with is generated, so reading it twice computes it
            // twice.
            let original = self
                .resources
                .original_at(first.tile, first.kind)
                .unwrap_or(Amount::ZERO);
            let mut left = original
                .0
                .saturating_sub(self.depletion.taken(first.tile, first.kind).0);
            // A finished upgrade raises what a unit takes in one tick. The
            // rate is read once for the whole segment, beside the deposit
            // that the segment draws from.[^1]
            //
            // [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
            let rate = upgrade::gather_rate_with(GATHER_RATE, self.upgrades.finished(first.tile));
            let mut granted = 0u32;
            for position in &order[at..end] {
                if left == 0 {
                    break;
                }
                let intent = intents[*position as usize];
                let amount = rate.min(left);
                left -= amount;
                granted += amount;
                let added = self
                    .soldiers
                    .add_carry(intent.unit, intent.kind, Amount(amount));
                debug_assert!(added, "the intent came from a live soldier");
                self.gather_log.push(ResourceTaken::new(
                    tick,
                    intent.unit.to_bits(),
                    intent.tile,
                    amount,
                    intent.kind.to_u8(),
                ));
            }
            if granted > 0 {
                run.push((key, granted));
            }
            at = end;
        }
        self.depletion.merge_ascending(&run, tick);
        Ok(())
    }

    /// Advances every upgrade that a unit is building.
    ///
    /// The pass reads the builders and the upgrade map. It reads no tile
    /// column and it takes no tile count, so a world of any size in which one
    /// unit builds costs the same.[^1]
    ///
    /// The builders of one tile are gathered into one contribution and the
    /// map is merged once, in ascending tile order. The contribution is a
    /// count of builders times a whole-number rate, so it is the same
    /// whatever order the threads produced the intents in.[^2] [^3]
    ///
    /// **A tile carries one upgrade.** When builders on one tile name
    /// different kinds, the kind already standing there wins. A tile that
    /// holds no site takes the lowest kind number present, which is the first
    /// in the sorted order, so the answer does not depend on which unit
    /// arrived first.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, or when the
    /// sort refuses the keys.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn build(&mut self, threads: usize) -> Result<(), StepError> {
        let intents = build_intents(&self.soldiers, threads)?;
        if intents.is_empty() {
            // The merge is still called, so the visit count describes this
            // tick rather than the last one that built anything.
            self.upgrades.merge_ascending(&[]);
            return Ok(());
        }

        let keys: Vec<BoundedKey> = intents
            .iter()
            .map(|intent| {
                BoundedKey::new(
                    upgrade::site_key(intent.tile, intent.kind),
                    intent.unit.to_bits(),
                )
            })
            .collect();
        let ceiling = upgrade::key_ceiling(self.grid.tile_count());
        let order = build_order_of(&keys, ceiling)?;

        // The key packs the tile above the kind, so the sorted order is tile
        // major and every builder of one tile sits in one run.
        let mut run: Vec<(TileIdx, UpgradeKind, i64)> = Vec::new();
        let mut at = 0usize;
        while at < order.len() {
            let tile = intents[order[at] as usize].tile;
            let mut end = at;
            while end < order.len() && intents[order[end] as usize].tile == tile {
                end += 1;
            }
            let held = self.upgrades.at(tile).map(|site| site.kind);
            let winner = held.unwrap_or(intents[order[at] as usize].kind);
            let builders = order[at..end]
                .iter()
                .filter(|position| intents[**position as usize].kind == winner)
                .count() as i64;
            if builders > 0 {
                run.push((tile, winner, builders * upgrade::BUILD_RATE));
            }
            at = end;
        }
        self.upgrades.merge_ascending(&run);
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
    /// Applies the production rate and the upkeep rate of every site.
    ///
    /// The pass writes the store column and nothing else. It runs on the
    /// tick that the schedule names, and it does nothing on every other
    /// tick.
    ///
    /// The account of what the stores hold moves by the net of the pass.
    /// That net is what landed minus what was taken, and both are exact
    /// integers, so the account and the column stay equal.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// Settles the positions of every site.
    ///
    /// The release runs on every frame, because a unit dies on any frame and
    /// a position that named it would hold a stale identity until the next
    /// rebalance.[^1]
    ///
    /// The rebalance runs on the interval that the schedule names. It reads
    /// the store as this frame left it, so it runs after the rates and after
    /// the consumption draw.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0065, a group is a site membership, not a region, decision D2. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    /// [^2]: ADR-0065, a group is a site membership, not a region, decision D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
    fn settle_positions(&mut self, threads: usize) -> Result<(), StepError> {
        position::release_the_dead(&mut self.positions, &self.soldiers, threads)?;
        if !self.position_schedule.due(self.tick) {
            return Ok(());
        }
        position::rebalance(
            &mut self.positions,
            self.settlements.live_column(),
            self.settlements.tile_column(),
            self.settlements.store_column(),
            self.terrain,
            threads,
        )?;
        // The resize opens the positions and seats nobody. This fills them,
        // and it runs on the same schedule because a seat cannot be taken
        // before it is opened.[^1]
        //
        // [^1]: ADR-0099, a site fills its positions by one sort and one scan, decision D2. `docs/adrs/draft/adr-0099-a-site-fills-its-positions-by-one-sort-and-one-scan.md`
        position::assign(&mut self.positions, &self.soldiers, threads)?;
        Ok(())
    }

    fn apply_rates(&mut self, threads: usize) -> Result<(), StepError> {
        self.shortfall_log.clear();
        self.rates.open_to(self.settlements.slot_count());
        let schedule = self.schedule;
        let tick = self.tick;
        let pass = crate::rates::apply(
            schedule,
            tick,
            &self.rates,
            self.settlements.store_update(),
            threads,
        )?;
        for index in 0..COMMODITY_COUNT {
            let net = pass
                .ledger
                .net(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            self.store_account[index] = sim_math::combine(self.store_account[index], net);
        }
        self.rate_ledger = self.rate_ledger.combine(pass.ledger);
        self.shortfall_log = pass.shortfalls;
        Ok(())
    }

    /// Runs the consumption pass of one frame.
    ///
    /// The pass runs when the schedule is due, and it does nothing
    /// otherwise. It has four stages, and the order between them is the
    /// order of the rule: the need falls, the cohorts draw, the draw feeds
    /// the units, and the deficit follows the need.[^1]
    ///
    /// The cohort table is derived from the home column of the units. The
    /// pass derives it again here rather than carrying it between frames,
    /// so the table cannot disagree with the column it summarises.[^2]
    ///
    /// What leaves a store is what the cohorts received, so the account of
    /// the stores falls by the same amount. A pass that moved a quantity and
    /// forgot the account would fail the conservation check on every frame,
    /// at every thread count.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, and when the
    /// columns disagree.
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decisions D1, D2 and D4. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^3]: Findings register, FND-065. `docs/FINDINGS.md`
    fn consume(&mut self, threads: usize) -> Result<(), StepError> {
        self.rationed_log.clear();
        if !self.schedule.due(self.tick) {
            return Ok(());
        }
        let rule = self.need_rule;
        let schedule = self.schedule;
        let commodity = CommodityId(0);

        // The need falls first. The subtract saturates at zero.
        cohort::decay(
            schedule.per_application(rule.decay()),
            self.soldiers.need_update(),
            threads,
        )?;

        // The cohorts are derived from the unit columns, in slot order.
        let sites = self.settlements.slot_count();
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            sites,
        );

        let pass = cohort::draw(
            self.tick,
            schedule.per_application(rule.ration()),
            commodity,
            &self.cohorts,
            self.settlements.store_update(),
            threads,
        )?;

        // What the cohorts received left the stores.
        let index = commodity.0 as usize;
        self.store_account[index] = sim_math::combine(
            self.store_account[index],
            Accum(-pass.ledger.granted[index].0),
        );
        self.draw_ledger = self.draw_ledger.combine(pass.ledger);
        self.rationed_log = pass.rationed;

        cohort::satisfy(
            rule,
            &pass.shares,
            &self.cohorts,
            self.soldiers.need_update(),
            threads,
        )?;
        Ok(())
    }

    /// Ends every unit that the shortage marked, in ascending slot order.
    ///
    /// The mark pass writes one bit for each unit into a dense plane, and
    /// each thread owns disjoint words of it, so the plane is the same at
    /// any thread count. The scan is ordered all the same: the deaths apply
    /// in the order it finds them, and a free slot returns to the queue in
    /// that order.[^1]
    ///
    /// A death advances the generation of the slot, so the identity of the
    /// dead unit never resolves to the unit spawned next in that slot.[^2]
    /// The world removes the unit through its own despawn, which accounts
    /// for what the unit carried, so conservation still balances.[^3]
    ///
    /// The cohort table summarises the home column, and a death changes that
    /// column, so the table is derived again here. A summary left stale
    /// would state a headcount that no unit backs.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    fn reap(&mut self, threads: usize) -> Result<(), StepError> {
        self.starved_log.clear();
        if !self.schedule.due(self.tick) {
            return Ok(());
        }
        cohort::mark_starved(
            self.need_rule,
            self.soldiers.deficit_column(),
            self.soldiers.live_column(),
            &mut self.death_plane,
            threads,
        )?;
        let order = cohort::starved_order(&self.death_plane, threads)?;
        if order.is_empty() {
            return Ok(());
        }
        let tick = self.tick;
        for slot in order {
            let index = slot as usize;
            let deficit = self.soldiers.deficit_column()[index];
            let generation = self.soldiers.generation_of(slot);
            let unit = Entity::new(slot, generation)
                .expect("a marked slot is live, so it holds a generation of one or more");
            self.starved_log
                .push(UnitStarved::new(tick, unit.to_bits(), deficit));
            let ended = self.despawn_soldier(unit);
            debug_assert!(ended, "a marked slot holds a live unit");
        }
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        Ok(())
    }

    fn refresh_bridge(&mut self) -> Result<(), StepError> {
        if self.bridge.describes(&self.soldiers).is_ok() {
            return Ok(());
        }
        self.bridge.rebuild(&self.soldiers)?;
        Ok(())
    }
}

/// The amount that one unit takes from one tile in one step.
///
/// The rate is content. It is declared here until content exists, and the
/// register holds the open choice of its value.[^1]
///
/// The rate is high against the stock of a tile, so a full tile of gatherers
/// always empties a deposit and never divides it evenly. That is the case the
/// resolve exists for, and a lower rate would make the contested case rare
/// instead of ordinary.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-022. `docs/DECISIONS.md`
/// [^2]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
const GATHER_RATE: u32 = 4;

/// One gather order, ready for the resolve.
#[derive(Clone, Copy, Debug)]
struct GatherIntent {
    /// The unit that gathers.
    unit: Entity,
    /// The tile that the unit stands on.
    tile: TileIdx,
    /// The kind that the unit gathers.
    kind: ResourceKind,
}

/// Returns the order in which the resolve reads the gather intents.
///
/// The order is the key vector sort: by the tile and the kind together, then
/// by the identity of the unit.[^1] It depends on the key values alone, so it
/// is the same at any thread count, and it does not follow the slot order of
/// the arena.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[cfg(not(feature = "probe-nondeterminism"))]
fn gather_order_of(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    sort::order_bounded(keys, ceiling)
}

/// Returns the gather intents in the order they arrived, which is a defect.
///
/// This is the perturbed build. The resolve reads the joined intent list
/// rather than the sorted one, so who empties a deposit depends on the order
/// the slots were joined in. The slot probe reverses that order, and the
/// reversal is visible only above one thread, so the thread-count test then
/// fails.
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
fn gather_order_of(keys: &[BoundedKey], _ceiling: u64) -> Result<Vec<u32>, SortError> {
    // A stable sort by the deposit alone. Each deposit still owns one
    // contiguous segment, which the resolve requires to scan a segment at all,
    // and within a segment the order is the order the intents arrived in.
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by_key(|position| keys[*position as usize].order());
    Ok(order)
}

/// Returns the gather intent of each live soldier that carries an order.
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the join reads the slots in slot order. The result never depends
/// on thread completion order.[^1]
///
/// A soldier with no order gathers nothing and produces no intent, so a world
/// in which nobody was told to gather costs one pass over the live set.
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn gather_intents(soldiers: &SoldierArena, threads: usize) -> Result<Vec<GatherIntent>, StepError> {
    let live: Vec<Entity> = soldiers.iter().collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<GatherIntent>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    std::thread::scope(|scope| {
        for (chunk, slot) in live.chunks(chunk_len).zip(slots.entries_mut()) {
            scope.spawn(move || {
                *slot = chunk
                    .iter()
                    .filter_map(|unit| {
                        let kind = soldiers.gather_order(*unit)??;
                        let tile = soldiers.tile(*unit)?;
                        Some(GatherIntent {
                            unit: *unit,
                            tile,
                            kind,
                        })
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

/// One unit that is building, and what it is building.
#[derive(Clone, Copy, Debug)]
struct BuildIntent {
    /// The unit that builds.
    unit: Entity,
    /// The tile that the unit stands on.
    tile: TileIdx,
    /// The kind that the unit builds.
    kind: UpgradeKind,
}

/// Returns the order in which the advance reads the build intents.
///
/// The order is the key vector sort: by the tile and the kind together, then
/// by the identity of the unit.[^1] It depends on the key values alone, so it
/// is the same at any thread count, and it does not follow the slot order of
/// the arena.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// The sound sort is used in the perturbed build as well. The advance sums a
/// count of builders, and integer addition is order-free, so a perturbed order
/// would change nothing and a probe over it would assert nothing.[^3]
///
/// [^3]: ADR-0004, iteration order is explicit, decision D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn build_order_of(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    crate::sort::order_bounded(keys, ceiling)
}

/// Returns the build intent of each live soldier that carries an order.
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the join reads the slots in slot order. The result never depends
/// on thread completion order.[^1]
///
/// A soldier with no order builds nothing and produces no intent, so a world
/// in which nobody was told to build costs one pass over the live set.
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn build_intents(soldiers: &SoldierArena, threads: usize) -> Result<Vec<BuildIntent>, StepError> {
    let live: Vec<Entity> = soldiers.iter().collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<BuildIntent>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    std::thread::scope(|scope| {
        for (chunk, slot) in live.chunks(chunk_len).zip(slots.entries_mut()) {
            scope.spawn(move || {
                *slot = chunk
                    .iter()
                    .filter_map(|unit| {
                        let kind = soldiers.build_order(*unit)??;
                        let tile = soldiers.tile(*unit)?;
                        Some(BuildIntent {
                            unit: *unit,
                            tile,
                            kind,
                        })
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
/// A soldier that holds no intent does not move at all. The choice pass
/// writes the intent, and it runs before this one.[^7]
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
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^7]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^8]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^9]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^10]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
fn soldier_moves(
    tick: Tick,
    seed: u64,
    terrain: Terrain,
    soldiers: &SoldierArena,
    layout: BlockLayout,
    exits: &ExitField,
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
                        // A unit that holds no intent does not move. The
                        // choice pass writes the intent, and a unit whose
                        // every option scored below the floor holds what it
                        // was doing.[^7]
                        let option = soldiers.intent(*soldier)??;
                        let here = soldiers.address(*soldier)?;
                        // **The option steers the step.** The unit reads the
                        // entry of its own cell and its own option, and it
                        // never scores a neighbouring cell of its own.[^8]
                        //
                        // The direction index of the cell lattice names the
                        // same offset as the direction index of the tile
                        // lattice, because both are the six neighbour offsets
                        // of a hex.[^9]
                        //
                        // A cell that no neighbour beats holds no direction,
                        // and the unit falls back to the uniform draw. The
                        // draw is keyed on the system, the frame, the entity
                        // and the draw index, so it never reads a
                        // thread-local state.[^10]
                        let cell = layout.block_of_key(layout.key_of(soldiers.tile(*soldier)?)?);
                        let direction = match exits.exit(cell, option) {
                            Some(Some(direction)) => direction as usize,
                            _ => rng::draw_below(
                                seed,
                                rng::SYSTEM_SOLDIER_MOVE,
                                tick.0,
                                soldier.to_bits(),
                                DRAW_MOVE_DIRECTION,
                                NEIGHBOUR_COUNT as u64,
                            ) as usize,
                        };
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
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^7]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
fn admit(
    intents: &[(Entity, Axial)],
    soldiers: &SoldierArena,
    bridge: &UnitTileBridge,
    terrain: Terrain,
    upgrades: &UpgradeMap,
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
                    // The ground states the capacity and a finished
                    // upgrade adds to it. One function answers the whole
                    // question, so admission cannot read the ground table
                    // without the upgrade table.[^8]
                    //
                    // [^8]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
                    let ground = terrain
                        .kind(address)
                        .map_or(0, crate::terrain::TileKind::capacity);
                    segment.capacity =
                        upgrade::capacity_with(ground, upgrades.finished(TileIdx(segment.tile)));
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

/// What one worker produces from one range of tiles.
///
/// The worker writes the events it emitted and the changes it made. It
/// writes nothing to the world, so two workers on two ranges cannot race.
#[derive(Clone, Debug, Default)]
struct ChunkResult {
    /// The events of the range, in ascending tile order.
    events: Vec<TileChanged>,
    /// The change made to each tile of the range, in ascending tile order.
    changes: Vec<(u32, Fix32)>,
}

/// Updates one contiguous range of tiles and returns what it changed.
///
/// The function is pure in the sense that the record requires: the same
/// prior values and the same key give the same result.[^1]
///
/// The range is read through a view of the field rather than through a
/// mutable slice, because the field stores no array of values to slice.
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
fn update_range(
    tick: Tick,
    seed: u64,
    start: u32,
    end: u32,
    range: TileValueRange<'_>,
) -> ChunkResult {
    let mut result = ChunkResult::default();
    let mut cursor = 0usize;
    for index in start..end {
        let raw = rng::draw_below(seed, rng::SYSTEM_TILE_STUB, tick.0, u64::from(index), 0, 8);
        if raw >= 4 {
            continue;
        }
        let delta = Fix32((raw as i32) - 2);
        if delta.0 == 0 {
            continue;
        }
        // The value is read here and not at the top of the loop, because a
        // tile the draw did not choose needs no value. The cursor only moves
        // forward, and the loop visits the tiles in ascending order, so a
        // skipped tile costs the cursor nothing.
        let value = range.value(TileIdx(index), &mut cursor);
        let updated = sim_math::add(value, delta);
        result.changes.push((index, delta));
        let kind = if delta.0 > 0 {
            CHANGE_KIND_RAISED
        } else {
            CHANGE_KIND_LOWERED
        };
        // The holder is stamped after the holding spread, at the end of the
        // step. This pass runs at the top of the step, so any holder it read
        // here would be the holder of the frame before.[^2]
        //
        // [^2]: Findings register, FND-029. `docs/FINDINGS.md`
        result.events.push(TileChanged::new(
            tick,
            TileIdx(index),
            updated,
            Holder::NOBODY,
            kind,
        ));
    }
    result
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

    /// Builds a world with a broken part.
    fn broken(change: impl FnOnce(&mut World)) -> World {
        let mut world = World::new(WorldConfig {
            width: 4,
            height: 2,
            seed: 1,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
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
    fn a_stored_change_outside_the_extent_fails_the_check() {
        assert!(!broken(|world| {
            world.values.merge_ascending(&[(64, Fix32(1))]);
        })
        .check_invariants());
    }

    #[test]
    fn an_event_with_padding_fails_the_check() {
        assert!(!broken(|world| {
            let mut event = TileChanged::new(Tick(1), TileIdx(0), Fix32::ZERO, Holder::NOBODY, 1);
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
                Holder::NOBODY,
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
                Holder::NOBODY,
                1,
            ));
        })
        .check_invariants());
    }
}
