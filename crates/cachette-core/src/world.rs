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

use crate::action::{ActionSchema, ActionShape, Verb};
use crate::balance::Balance;
use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge, BLOCK_BITS_DEFAULT};
use crate::campaign::{self, CampaignEvent, CampaignRegister, CampaignRow};
use crate::character::{CharacterArena, CharacterError};
use crate::choose::{
    self, CarryClass, ChoiceError, ChoiceExplanation, ChoiceSchedule, NeedBuckets, Ranked,
    WeightProfile, OPTIONS,
};
use crate::climate::{Climate, ClimateField};
use crate::cohort::{
    self, CohortError, CohortTable, DeathPlane, DrawLedger, NeedCondition, NeedRule, SiteRationed,
    UnitStarved,
};
use crate::contest::{self, ContestError, Grievance, UnitFell};
use crate::controller::{
    self, CarrierAssignment, Choice, Controller, ControllerCommand, FactionRow, FactionState,
    FactionWeights, GameEnd, WinPath,
};
use crate::conversion::{self, ConversionError, Convert, UnitConverted};
use crate::descent::{DescentId, Parents};
use crate::event::{
    FactionEliminated, ResourceTaken, SettlementFounded, SiteTaken, TileChanged, UpgradeCollapsed,
    UpgradeFinished, CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED, TAKE_KIND_CAPTURED, TAKE_KIND_RAZED,
    WEAR_CAUSE_ARMY, WEAR_CAUSE_BOTH, WEAR_CAUSE_ORDERED, WEAR_CAUSE_WEATHER,
};
use crate::founding::{
    self, Founding, FoundingError, FoundingOutcome, SettleError, SettleOutcome, Survey,
};
use crate::growth;
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, GridError, NEIGHBOUR_COUNT};
use crate::holding::{FactionMask, Holder, Holding, LeaseRules, ReachRules};
use crate::household;
use crate::influence::{Influence, InfluenceError, InfluenceField};
use crate::luxury::{LuxuryError, LuxuryField, LuxuryId, LuxurySet, VarietyLevel};
use crate::observation::{Observation, SightRules};
use crate::padded::PaddedLattice;
use crate::plan::{self, Needs, PlanRefusal, PlanRegister, PlanRules, Project};
use crate::position::{
    self, Position, PositionError, PositionTable, SitePreference, WORK_COMMODITY,
};
use crate::presence::PresenceRelation;
use crate::production::{
    BuildCostRow, BuildCostTable, QueueEntry, QueueError, QueueOrder, QueueTable,
    QUEUE_PERIOD_DEFAULT, QUEUE_PHASE_DEFAULT, WORK_PER_ADVANCE,
};
use crate::promotion::{self, PromotionError, UnitPromoted};
use crate::pyramid::{
    ApproachField, CellSummary, ExitField, Pyramid, ReturnField, SeededField, AT_SEED, STOCK_PASSES,
};
use crate::rates::{RateError, RateLedger, RateSchedule, RateTable, SiteShortfall};
use crate::relation::{RelationCrossed, RelationError, RelationMatrix, RelationRules};
use crate::resource::{
    ledger_key, Amount, CarryLoad, DepletionLedger, RecoveryRules, ResourceField, ResourceKind,
    TileGround, RESOURCE_KIND_COUNT,
};
use crate::rng;
use crate::sim_math;
use crate::site::{CommodityId, SettlementArena, SettlementError, SiegeRules, COMMODITY_COUNT};
use crate::slots::Slots;
use crate::soldier::{SoldierArena, SoldierError, NO_HOME};
#[cfg(not(feature = "probe-nondeterminism"))]
use crate::sort;
use crate::sort::{BoundedKey, SortError};
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TerrainTile, TileKind};
use crate::tile_value::{TileValueChunk, TileValues};
use crate::trade::{
    self, Advert, Consideration, MarketTable, TradeError, TradeRow, TradeSpoken, TradeTable,
    ACT_ACCEPT, ACT_CLOSE, ACT_COUNTER, ACT_DEFAULT, ACT_OFFER, ACT_REFUSE, ACT_REOPEN, ACT_SETTLE,
    ACT_STEP_RELATION, ACT_TRANSFER_LAND, DEFAULT_BOARD_ROWS, DEFAULT_LAND_LIST_BOUND, KIND_LAND,
    KIND_RELATION, KIND_RESOURCE, TRADE_BOUND, TRADE_COUNTERED, TRADE_DEFAULTED, TRADE_IDLE,
    TRADE_OFFERED, TRADE_SETTLED,
};
use crate::types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx, FACTION_CEILING};
use crate::unit_type::{
    UnitTypeError, UnitTypeId, UnitTypeRow, UnitTypeTable, DEFAULT_UNIT_TYPE_TABLE, LEADER,
    SOLDIER, UNIT_TYPE_COUNT,
};
use crate::upgrade::{
    self, BuildRefusal, UpgradeCategory, UpgradeMap, UpgradeRow, UpgradeSite, UpgradeTable,
    UpgradeTableError,
};
use crate::weather::{
    ground_over_lattice, CellGround, Ground, Latitudes, Storm, WeatherError, WeatherField,
    WeatherScale, Wind,
};

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

/// The reason that the world refused an order to raze a site.
///
/// Each value names the thing that refused, so a caller repairs the call
/// without guessing which part of it was wrong.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RazeError {
    /// The identity names no live site.
    NoSuchSite,
    /// The razer owns the site. A faction does not raze its own city.
    OwnSite,
    /// No siege of the razer stands against the site.
    ///
    /// A raze is an order against a siege, and a siege stands only while the
    /// razer holds the site tile against no defender.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D11. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    NotBesieging,
}

impl core::fmt::Display for RazeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchSite => write!(formatter, "the identity names no live site"),
            Self::OwnSite => write!(formatter, "a faction does not raze its own site"),
            Self::NotBesieging => {
                write!(formatter, "no siege of the razer stands against the site")
            }
        }
    }
}

impl std::error::Error for RazeError {}

/// What the siege pass does to one site on one tick.
///
/// A site falls to work and never to a moment. The pass reads the trigger
/// for every site first, and it writes afterwards, so the answer for one
/// site is fixed before any write moves the world.[^1]
///
/// # References
///
/// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SiegeStep {
    /// A unit of the owning faction stands on the site tile, so any siege
    /// the site carried ends and its work is gone.
    Ends,
    /// Nobody presses the site this tick, and no unit of the owning faction
    /// stands on its tile. A siege the site carries waits, unchanged.
    Waits,
    /// A siege stands and has not reached the work a capture costs.
    Presses {
        /// The faction that stands on the site tile.
        besieger: FactionId,
        /// The work the siege has done, including this tick.
        work: i64,
    },
    /// A siege stands and has reached the work a capture costs.
    Falls {
        /// The faction that stands on the site tile.
        besieger: FactionId,
        /// The work the siege has done, including this tick.
        work: i64,
        /// The work a raze of this site costs.
        raze_work: i64,
        /// One when the besieging faction ordered a raze, zero otherwise.
        ordered: u8,
    },
}

/// The reason that the world refused to send a set of units somewhere.
///
/// Each value names the thing that refused, so a caller repairs the call
/// without guessing which part of the set was wrong.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendError {
    /// The number names no destination plane of this world.
    NoSuchDestination(u16),
    /// An identity names no live soldier.
    DeadUnit(Entity),
    /// A seed address is outside the world.
    AddressOutsideWorld(Axial),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchDestination(destination) => {
                write!(formatter, "{destination} names no destination")
            }
            Self::DeadUnit(unit) => {
                write!(formatter, "the identity {unit:?} names no live soldier")
            }
            Self::AddressOutsideWorld(address) => {
                write!(formatter, "the address {address:?} is outside the world")
            }
        }
    }
}

impl std::error::Error for SendError {}

/// The reason that a conversion verb refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvertError {
    /// The number names no faction of this world.
    NoSuchFaction(u16),
    /// An identity names no live soldier.
    DeadUnit(Entity),
}

/// The reason that the relation verb refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveRelationError {
    /// The speaker identity names no live soldier.
    DeadUnit(Entity),
    /// The relation refused the move.
    Relation(RelationError),
}

impl From<RelationError> for MoveRelationError {
    fn from(error: RelationError) -> Self {
        Self::Relation(error)
    }
}

impl core::fmt::Display for MoveRelationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DeadUnit(unit) => write!(formatter, "{unit:?} names no live soldier"),
            Self::Relation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for MoveRelationError {}

/// Why a campaign was not raised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignError {
    /// The number names no faction of this world.
    NoSuchFaction(u16),
    /// The objective address is outside the world.
    OutsideWorld(Axial),
    /// The cohort size is zero.
    EmptyCohort,
    /// The faction holds a live campaign.
    LiveCampaign,
    /// The faction has no idle unit to take.
    NoIdleUnit,
    /// The world holds no destination plane for the faction.
    NoPlane(u16),
    /// The send verb refused.
    Send(SendError),
}

impl core::fmt::Display for CampaignError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{faction} names no faction of this world")
            }
            Self::OutsideWorld(address) => {
                write!(formatter, "the objective {address:?} is outside the world")
            }
            Self::EmptyCohort => write!(formatter, "a cohort of zero raises nothing"),
            Self::LiveCampaign => write!(formatter, "the faction holds a live campaign"),
            Self::NoIdleUnit => write!(formatter, "the faction has no idle unit"),
            Self::NoPlane(plane) => write!(
                formatter,
                "the world holds no destination plane {plane} for the faction"
            ),
            Self::Send(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CampaignError {}

impl From<SendError> for CampaignError {
    fn from(error: SendError) -> Self {
        Self::Send(error)
    }
}

impl core::fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{faction} names no faction of this world")
            }
            Self::DeadUnit(unit) => {
                write!(formatter, "the identity {unit:?} names no live soldier")
            }
        }
    }
}

impl std::error::Error for ConvertError {}

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
    /// The promotion pass refused to run.
    Promotion(PromotionError),
    /// The resolution of a meeting refused to run.
    Contest(ContestError),
    /// The conversion pass refused to run.
    Conversion(ConversionError),
    /// The weather solve refused to run.
    Weather(WeatherError),
}

impl From<ConversionError> for StepError {
    fn from(error: ConversionError) -> Self {
        match error {
            ConversionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Conversion(other),
        }
    }
}

impl From<PromotionError> for StepError {
    fn from(error: PromotionError) -> Self {
        match error {
            PromotionError::ZeroThreads => Self::ZeroThreads,
            other => Self::Promotion(other),
        }
    }
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

impl From<WeatherError> for StepError {
    fn from(error: WeatherError) -> Self {
        match error {
            WeatherError::ZeroThreads => Self::ZeroThreads,
            other => Self::Weather(other),
        }
    }
}

impl From<ContestError> for StepError {
    fn from(error: ContestError) -> Self {
        match error {
            ContestError::ZeroThreads => Self::ZeroThreads,
            other => Self::Contest(other),
        }
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
    /// The weather field refused to build.
    Weather(WeatherError),
}

impl From<WeatherError> for WorldError {
    fn from(error: WeatherError) -> Self {
        Self::Weather(error)
    }
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
            Self::Weather(error) => {
                write!(formatter, "the world has no weather field: {error}")
            }
        }
    }
}

impl std::error::Error for WorldError {}

/// Why the seeding of a world refused.
///
/// The seeding founds a run for every faction, and it places the luxuries.
/// It then leaves the world readable, and that needs a rebuild of the
/// derived unit structure. Either half can refuse, so the error says which
/// one did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedError {
    /// The luxury field refused the placements.
    Luxury(LuxuryError),
    /// The rebuild of the derived unit structure refused to run.
    Bridge(BridgeError),
}

impl From<LuxuryError> for SeedError {
    fn from(error: LuxuryError) -> Self {
        Self::Luxury(error)
    }
}

impl From<BridgeError> for SeedError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

impl core::fmt::Display for SeedError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Luxury(error) => write!(formatter, "the world took no luxuries: {error}"),
            Self::Bridge(error) => write!(
                formatter,
                "the seeded world holds no readable unit structure: {error}"
            ),
        }
    }
}

impl std::error::Error for SeedError {}

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
            Self::Promotion(error) => {
                write!(formatter, "the promotion pass refused: {error}")
            }
            Self::Contest(error) => {
                write!(formatter, "the resolution of a meeting refused: {error}")
            }
            Self::Conversion(error) => {
                write!(formatter, "the conversion pass refused: {error}")
            }
            Self::Weather(error) => {
                write!(formatter, "the weather solve refused: {error}")
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

    /// The number of destination planes that a world holds when the caller
    /// states no other.
    ///
    /// **This is a fixture-facing parameter and not a budget.** It says how
    /// many places a control plane may send units to at one time, before it
    /// re-aims a plane it already used. No record holds the value and no
    /// measurement chooses it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub const DEFAULT_DESTINATION_COUNT: u16 = 4;

    /// The destination planes that one faction climbs at the same time.
    ///
    /// A faction climbs three planes, and none of the three may yield to
    /// another. The first carries its campaign, its carriers and its project
    /// order, which already take turns among themselves. The second carries a
    /// crossing, because a faction on an island that waits for its war to end
    /// waits for a war it cannot reach. The third carries a settling, because
    /// a faction fights for most of a run and a settler that waits for the
    /// war to end never founds anything.
    ///
    /// **This is a structural property of the controller and not a budget.**
    /// It counts the purposes the controller sends units for, and each of the
    /// three is a purpose that stands in the code.
    pub const PLANES_FOR_ONE_FACTION: u16 = 3;

    /// Returns the destination planes that a world of this shape holds.
    ///
    /// The count gives each faction the planes it climbs at the same time,
    /// and it never falls below the count a caller that states no faction
    /// gets. A caller may raise it or lower it afterwards.
    #[must_use]
    pub const fn destination_plane_count(&self) -> u16 {
        let wanted = self
            .faction_count
            .saturating_mul(Self::PLANES_FOR_ONE_FACTION);
        if wanted > Self::DEFAULT_DESTINATION_COUNT {
            wanted
        } else {
            Self::DEFAULT_DESTINATION_COUNT
        }
    }
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
    /// Which factions stand on the ground of which other factions.
    ///
    /// The relation is derived at the end of a step and never stored as a
    /// fact.[^3]
    ///
    /// [^3]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    presence: PresenceRelation,
    /// What each faction sees now, and what each faction has ever seen.
    ///
    /// The layer of what a faction sees now is derived, and the step rebuilds
    /// it. The layer of what a faction has ever seen is state, and the step
    /// carries it forward.[^4]
    ///
    /// [^4]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    observation: Observation,
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
    /// What every delivery has moved out of a carry and into a store, for
    /// each kind.
    ///
    /// A delivery takes a quantity out of the account that the conservation
    /// check balances against the carries, and puts it into the account that
    /// the store check balances against the stores. This total is the term
    /// that links the two, and without it the first check fails on the frame
    /// of the first delivery.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    delivered: [u64; RESOURCE_KIND_COUNT],
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
    /// The direction of the nearest site of a faction, for each level 1 cell.
    ///
    /// It steers a unit that carries a load home. The field is derived at
    /// every rebuild of level 1, beside the exit field.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    returns: ReturnField,
    /// The direction of the nearest site tile of a faction, for each tile of
    /// a seeded block and each faction plane.
    ///
    /// **The return field above steers a laden unit to the cell that holds a
    /// site, and no further.** A delivery reads the tile the unit stands on,
    /// so a carrier that reached the cell has delivered nothing. This field
    /// resolves that last cell at the pitch of one tile.[^1] [^2]
    ///
    /// It is the same mechanism the destination planes use, keyed on the
    /// faction instead of the destination. It is seeded from the same set as
    /// the return field, so it answers wherever the return field led.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    /// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
    home_approaches: ApproachField,
    /// The direction of the nearest tile that holds stock, for each tile of a
    /// seeded block and each resource kind.
    ///
    /// **The exit field answers at the pitch of a level 1 cell, and the stock
    /// of a tile is a level 0 property.** A unit that stands on barren ground
    /// inside the cell with the most food reads that its own cell is the best
    /// one, and nothing tells it to step two tiles sideways. It then strips
    /// the ground it stands on and stands there.[^1]
    ///
    /// This field resolves the cell at the pitch of one tile. It is the same
    /// mechanism the home approach above uses, keyed on the resource kind
    /// instead of the faction.[^2]
    ///
    /// **The engine seeds it only over the blocks that a gatherer stands
    /// in.** The set of gatherers is the whole set the field serves, so the
    /// derivation follows that set and never the world.[^3]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-589. `docs/FINDINGS.md`
    /// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    stock_approaches: ApproachField,
    /// The direction of the nearest tile of a named destination, for each
    /// level 1 cell and each destination plane.
    ///
    /// It steers a unit that the control plane sent somewhere. The field is
    /// derived at every rebuild of level 1, beside the exit field and the
    /// return field, and it is derived again when the seeds change.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    destinations: SeededField,
    /// The seed tiles of each destination plane, in ascending order.
    ///
    /// **The control plane names these, and nothing else writes them.** The
    /// entry of one plane holds each tile once and in ascending order, so the
    /// derivation reads one set whatever order the caller named the tiles
    /// in.[^1]
    ///
    /// **The set holds tiles, and the cell of each is derived from it.** The
    /// coarse field is at block pitch and it reads the cells. The approach
    /// field is at tile pitch and it reads the tiles. Storing the cell beside
    /// the tile would be one fact in two places, and the two would then
    /// disagree with nothing to fail.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^3]: Findings register, FND-315. `docs/FINDINGS.md`
    destination_seeds: Vec<Vec<TileIdx>>,
    /// The direction of the nearest seed tile, for each tile of a seeded
    /// block and each destination plane.
    ///
    /// **The coarse field steers a unit to the cell that holds its target,
    /// and no further.** This one resolves that last cell at the pitch of one
    /// tile, so a unit sent at a tile arrives at the tile.[^1]
    ///
    /// It is derived beside the coarse field, from the same seed set.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    approaches: ApproachField,
    /// Whether the reach of each destination plane spreads through open
    /// water.
    ///
    /// The send verb writes this, and it writes what the set it was given
    /// says: a plane conducts through water when every unit sent to it
    /// crosses water. A plane that carries one unit the water refuses does
    /// not, because the field would then steer that unit at a coast.[^1]
    ///
    /// The entry is simulated state. A later frame derives the field from it,
    /// so two worlds that hold the same seeds and different conduction must
    /// diverge.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    destination_crossings: Vec<u8>,
    /// The load at which a unit counts as laden.
    ///
    /// A laden unit takes the option that carries its load home, and a unit
    /// below the mark does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    carry_mark: Amount,
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
    /// The base production rate and the upkeep rate of each site.
    ///
    /// This is stored state and it enters the state hash. The founding writes
    /// the production column of a site once, from the ground the survey
    /// measured.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    rates: RateTable,
    /// The effective rate that the last application read, as scratch.
    ///
    /// **This is derived, not stored.** The rate pass fills it again from the
    /// ground, the moisture, the upgrades and the residents on every
    /// application, so it holds no fact of its own and it stays out of the
    /// state hash. Its four inputs are stored and they enter the hash.[^1]
    ///
    /// The field exists so that the pass allocates once rather than on every
    /// application. Nothing outside the pass reads it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    effective_rates: RateTable,
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
    /// The shared table that a unit type indexes.
    ///
    /// **The table is data that the world is built with.** It holds no code,
    /// and a lookup in it is not a callback. A caller fills the rows it
    /// wants, and a world that filled none holds no contest at all.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    unit_types: UnitTypeTable,
    /// The table that a category and a level index.
    ///
    /// **The table is data that the world is built with.** A row is one
    /// category at one level. It names the ground it fits, the work it takes
    /// and the columns a pass reads. No pass branches on a category.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    upgrade_table: UpgradeTable,
    /// The build queue of every site, indexed by the slot of the site.
    ///
    /// **A site holds a bounded, ordered queue of plain-data entries.** The
    /// order is the order the entries were queued, and nothing reorders
    /// them. The whole table enters the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    queues: QueueTable,
    /// The shared table that a unit type indexes for its build cost.
    ///
    /// **The table is data that the world is built with**, in the way the
    /// unit type table and the upgrade table are. The work that finishes an
    /// entry is a value of it, and it is never a constant of the kernel.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    build_costs: BuildCostTable,
    /// When the queue advance acts.
    ///
    /// The interval is a parameter of the world and never a constant of the
    /// stage.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D5. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    queue_schedule: RateSchedule,
    /// When the growth stage acts.
    ///
    /// The interval is a parameter of the world and never a constant of the
    /// stage.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    growth_schedule: RateSchedule,
    /// The housing that one person takes.
    ///
    /// The value is a parameter of the world. The balance register holds the
    /// row and this field states no value of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    housing_per_person: u32,
    /// The store that one birth costs.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    food_per_birth: [Fix32; COMMODITY_COUNT],
    /// The chance that one proposal of one site becomes a birth.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    birth_chance: Fix32,
    /// The housing that a founded site starts with.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population. `docs/reference/balance.md`
    founding_housing: u32,
    /// How many people the growth stage added on the last tick.
    ///
    /// The count is a census of one tick, and the growth stage clears it
    /// before it acts. A zero is therefore visible: a world that cannot grow
    /// and a world that chose not to both read zero, and the free places
    /// beside it say which.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, the consequences. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    births: u32,
    /// What growth took out of the stores, for each commodity.
    ///
    /// The total is cumulative over the life of the world. It is the growth
    /// term of the conservation statement: what the stores held, plus what
    /// production put in, less what upkeep spent, less what the cohorts drew,
    /// less this, is what the stores hold.
    growth_ledger: [Accum; COMMODITY_COUNT],
    /// One bit for each unit that the last meeting ended.
    ///
    /// The plane is the batch of a structural change, in the way the plane of
    /// the shortage is, and the scan of it applies the change in ascending
    /// slot order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fell_plane: DeathPlane,
    /// The units that a meeting ended at the last resolution, in slot order.
    fell_log: Vec<UnitFell>,
    /// Which faction lost units to which at the last resolution, summed by
    /// pair. The relation pass reads it after the contest applies.
    grievances: Vec<Grievance>,
    /// What each faction feels toward each other faction.
    ///
    /// The matrix is simulated state and enters the hash. The contest, the
    /// conversion and the admission read it, and the settle path, the
    /// contest apply, the conversion apply, the drift and the verb write
    /// it.[^rel]
    ///
    /// [^rel]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    relations: RelationMatrix,
    /// The units that the conversion pass marked this frame, in slot order.
    ///
    /// The buffer is reused across frames, so the pass allocates once rather
    /// than once for each frame.
    convert_marks: Vec<Convert>,
    /// The units that changed faction this frame.
    converted_log: Vec<UnitConverted>,
    /// The promotions of the last frame that ran the pass.
    ///
    /// The log is cleared at the start of each pass, in the way every other
    /// per-frame log is, so a reader sees the promotions of one frame and
    /// never an accumulation.
    promoted_log: Vec<UnitPromoted>,
    /// The schedule that the promotion pass runs on.
    ///
    /// **The scan runs at a barrier of its own and not on every tick.** A
    /// pass over every unit on every frame spends the frame for an answer
    /// that changes rarely, and the eligibility of a unit is a level that
    /// only rises.[^1]
    ///
    /// The interval is a parameter of the world, so a caller changes how
    /// often the world looks for somebody to promote without touching the
    /// engine.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decisions D2 and D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    character_schedule: RateSchedule,
    /// The most characters that one promotion pass may create.
    ///
    /// **This is the cut at the rank, and it is not the ceiling.** The
    /// character arena is built at the ceiling of its declared tier and
    /// refuses a create beyond it, so the population bound has one
    /// enforcement site whatever this says.[^1] This says how fast the world
    /// is willing to approach it.
    ///
    /// The default admits as many as the arena has room for, so a caller who
    /// sets nothing gets the ceiling and no second bound.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-004. `docs/BLOCKERS.md`
    promotion_budget: u32,
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
    /// The profile reaches the state hash. The pass reads it on every tick,
    /// so it is a value the world holds and not only an input to it. The
    /// intent column carries the outcome, and an outcome states the effect
    /// and never the cause: two worlds that hold the same intents and
    /// different weights hash the same and diverge at the next choice.[^2]
    /// [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    /// [^3]: Findings register, FND-537. `docs/FINDINGS.md`
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
    /// The luxuries that each tile carries.
    ///
    /// The field holds one entry for each tile that carries a luxury, and it
    /// holds nothing else. A world in which nobody seeded a luxury holds no
    /// entry.[^1]
    ///
    /// The control plane seeds it once, and nothing writes it after that. It
    /// is simulated state all the same, because two worlds that carry
    /// different luxuries are different worlds, so it enters the state
    /// hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    luxuries: LuxuryField,
    /// The luxuries of each level 1 cell.
    ///
    /// The level is derived from the field above, and level 0 is the truth.
    /// It sits beside the cell summaries rather than inside them, because a
    /// union of luxuries is idempotent and two idempotent values do not
    /// add.[^1] It does not enter the state hash, because it holds no fact of
    /// its own.
    ///
    /// The field never changes after the world is seeded, so the level is
    /// derived when the seed lands and never again.
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    variety: VarietyLevel,
    /// Whether the world has taken a luxury seed.
    ///
    /// The flag is not a second copy of the field. A seed that places nothing
    /// leaves the field empty, and an unseeded world leaves it empty as well,
    /// so the field cannot answer the question on its own.[^1]
    ///
    /// The flag reaches the state hash. Two worlds that hold the same
    /// luxuries are not the same world when one took a seed and the other did
    /// not, because the next seed call is accepted by one and refused by the
    /// other.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    luxuries_seeded: bool,
    store_account: [Accum; COMMODITY_COUNT],
    /// What each ordered pair of factions has agreed, and what it still owes.
    ///
    /// The plane holds one row for each ordered pair, and it holds nothing
    /// until somebody speaks. It never follows the population.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    trade: TradeTable,
    /// What the last step said about trade.
    trade_log: Vec<TradeSpoken>,
    /// The upgrades the wear pass removed, since the last step began.
    ///
    /// **A removed upgrade leaves nothing behind.** The entry is dropped and
    /// the tile returns to the world the generator made, so this log is the
    /// only record that anything stood there. The step clears it before any
    /// system runs.
    ///
    /// The log is written and never read by a pass, so it enters no state
    /// hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    collapsed_log: Vec<UpgradeCollapsed>,
    /// The upgrade levels that finished, since the last step began.
    ///
    /// The step clears it before any system runs, and it enters no state
    /// hash.
    finished_log: Vec<UpgradeFinished>,
    /// The settlements that were founded, since the last step began.
    ///
    /// A settlement founded by a caller between two steps lands here and
    /// stays until the next step clears it. The log enters no state hash.
    founded_log: Vec<SettlementFounded>,
    /// The sites that changed hands or were razed, since the last step began.
    ///
    /// A raze by a caller between two steps lands here and stays until the
    /// next step clears it. The log enters no state hash, in the same way
    /// that every other log does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    taken_log: Vec<SiteTaken>,
    /// What a site resists, and what a raze costs over a capture.
    ///
    /// A site falls to work and never to a moment, and these are the two
    /// balance rows that price the work.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: Balance register, the siege. `docs/reference/balance.md`
    siege_rules: SiegeRules,
    /// The sites that changed hands over the run, and the sites that burned.
    ///
    /// **These two count the acts and never the sites that stand.** A run
    /// that keeps every city it takes and a run that burns every one differ
    /// here and nowhere else, because the site count reports the outcome of
    /// both. The census reports them, and no rule of the simulation reads
    /// them, so they enter no state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    sites_captured: i64,
    /// The sites that burned over the run. See the field above.
    sites_razed: i64,
    /// The site ticks that a siege pressed over the run.
    ///
    /// One site under siege for one tick adds one. A run in which no army
    /// ever holds a rival site reads zero here, and that is the reading
    /// that tells a keeping run from a run with no conquest in it.
    sieges_pressed: i64,
    /// The sieges that a relief ended before the site fell, over the run.
    ///
    /// A unit of the owning faction on the site tile ends a siege and takes
    /// its work away. A besieger that leaves ends nothing, so it adds
    /// nothing here.
    sieges_relieved: i64,
    /// The factions that left the game, since the last step began.
    ///
    /// The log enters no state hash. The column below is the stored fact,
    /// and the log is what a reader sees of it for one tick.
    eliminated_log: Vec<FactionEliminated>,
    /// One for a faction that has left the game, zero otherwise.
    ///
    /// **This is stored state and the step reads it, so it enters the state
    /// hash.**[^1] The elimination pass reads it to leave a faction that has
    /// already gone alone, and every game end reader reads it to refuse a
    /// faction that is out.
    ///
    /// The column is indexed by the faction identifier, and it holds one
    /// entry for each faction the world was built with. It is a one-byte
    /// integer and never a boolean, because it crosses into a hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    eliminated: Vec<u8>,
    /// What each faction offers and wants, one fixed block of rows for each.
    ///
    /// The table holds nothing until a faction advertises. It is simulated
    /// state and it enters the hash, because a controller of a later pass
    /// reads it inside the step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    market: MarketTable,
    /// The most tiles one land consideration names.
    ///
    /// The bound is a balance value and the register calls it unset.[^1] The
    /// value here is a stand-in that a caller replaces.
    ///
    /// # References
    ///
    /// [^1]: Balance register, land list bound. `docs/reference/balance.md`
    land_list_bound: u32,
    /// The water in the air and on the ground, over the level 1 cells.
    ///
    /// The field is a plane over the level 1 cell lattice and it is not a
    /// summary: a cell of it holds what the last solve left there, and that
    /// value appears nowhere at level 0. It is simulated state, so it enters
    /// the state hash.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    weather: WeatherField,
    /// The block geometry of the weather lattice.
    ///
    /// **The weather has a pitch of its own, and it is a parameter of the
    /// world.** The layout says which weather cell covers a tile. It is the
    /// same layout as the level 1 layout when the world takes the level 1
    /// pitch, and a different one at every other pitch.[^1]
    ///
    /// # References
    ///
    /// [^1]: The weather scale. [`WeatherScale`]
    weather_layout: BlockLayout,
    /// The weather lattice, and the margin of cells around the world that the
    /// weather solve steps and no reader sees.
    ///
    /// **This is the one site that turns a cell of the world into a cell of
    /// the weather plane.** Every reader of the weather names a tile, the
    /// reader below turns that tile into a cell, and this adds the margin
    /// offset. A second site that added the offset could disagree with this
    /// one, so there is not one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    weather_lattice: PaddedLattice,
    /// The ground under each weather cell, in cell index order.
    ///
    /// **The weather reads the ground, and the ground does not change.** The
    /// three numbers it holds are the tiles the cell covers, the tiles of it
    /// that admit a unit, and the sum of their heights. Every one of them is
    /// a pure function of the world seed and the address, so the world folds
    /// this once and never rebuilds it.[^1]
    ///
    /// It is derived from level 0 and it is not simulated state, so it enters
    /// no state hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    weather_ground: Vec<CellGround>,
    /// The climate that the weather left over each weather cell.
    ///
    /// **The field is quiet unless the caller asked for a spin.** A quiet
    /// field reads temperate at every address, so a world that asked for no
    /// spin generates the ground that the seed alone gives.[^1]
    ///
    /// The field is stored, and the terrain readers of this world read it, so
    /// it enters the state hash.[^2]
    ///
    /// # References
    ///
    /// [^1]: The climate field. [`ClimateField`]
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    climate: ClimateField,
    /// The mean standing water of the climate field, folded once.
    ///
    /// A terrain reader asks the climate over one address, and the climate of
    /// a cell is a share against the mean over the whole field. Folding the
    /// field for each tile would make a tile read cost the size of the
    /// lattice, so the world folds it once here.
    ///
    /// **This is derived, and the field is the declaration.** It is refreshed
    /// only when the field is built, and the field never changes after that.
    climate_reference: i64,
    /// The faction controller: one row for each faction, the two parameters
    /// the step reads on every tick, and the game end record.
    ///
    /// Every value in it is state that a later frame reads, so the whole of
    /// it enters the hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    controller: Controller,
    /// The campaign register: what each faction marches on, and with how
    /// many. A later frame reads it, so it enters the hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    campaigns: CampaignRegister,
    /// The projects each faction has zoned.
    ///
    /// The plan is simulated state that the step reads, so it enters the
    /// whole-world hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    plan: PlanRegister,
    /// The win-path balance values: the value each game end reader compares,
    /// the rate that feeds one of them, and whether the readers run.
    ///
    /// **A caller sets these at run time, and each holds a constant as its
    /// default.** The step reads them, so they enter the state hash.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decisions D1 and D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    balance: Balance,
    /// What the run has produced, for the subsystems that keep a per-tick
    /// count.
    ///
    /// The controller, the campaign register, the relation matrix and the
    /// queue table each empty their counts when the next tick starts. The
    /// world folds each count into this total at the one site that empties
    /// it, so the census reads a total for the whole run and never a reading
    /// of one tick.[^1]
    ///
    /// **This is not simulated state and it does not enter the hash.** No
    /// frame reads it. It is a derived count of what already happened, in
    /// the way the per-tick counts it folds are.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-498. `docs/FINDINGS.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    census: CensusTotals,
}

/// What the run has produced, for each subsystem that counts one tick at a
/// time.
///
/// Every field is a total since the world was built. A total never falls.
///
/// **The world folds a per-tick count into a field here at the one site that
/// empties that count.** The per-tick counter stays the only place that
/// counts the act, so this is a fold of one number and not a second place
/// that counts the same thing.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CensusTotals {
    /// The commands a verb took, over the run.
    controller_commands: i64,
    /// The commands a verb refused, over the run.
    controller_refused: i64,
    /// The relation moves the controller made through the verb, over the run.
    relation_moves: i64,
    /// The boards the trading stage wrote, over the run.
    boards_written: i64,
    /// The offers the trading stage opened, over the run.
    offers_made: i64,
    /// The contracts the trading stage bound, over the run.
    contracts_bound: i64,
    /// The carriers the trading stage assigned, over the run.
    carriers_assigned: i64,
    /// The crossings into the war band, over the run, from any cause.
    wars_declared: i64,
    /// The campaigns raised, over the run.
    campaigns_raised: i64,
    /// The campaigns whose objective passed to the campaigner, over the run.
    campaigns_won: i64,
    /// The units the queues produced, over the run.
    queue_produced: i64,
    /// The finished entries an advance refused for want of a resident, over
    /// the run.
    queue_refused_without_a_person: i64,
    /// The finished entries an advance refused for want of goods, over the
    /// run.
    queue_refused_without_goods: i64,
    /// The orders the queue verb refused, over the run.
    queue_refused_at_the_verb: i64,
    /// The people the growth stage added, over the run.
    births: i64,
}

impl World {
    /// Builds a world from the settings.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid.
    pub fn new(config: WorldConfig) -> Result<Self, WorldError> {
        Self::with_weather_scale(config, WeatherScale::DEFAULT)
    }

    /// Builds a world from the settings, at a stated weather resolution.
    ///
    /// **The resolution of the weather is a parameter of the world.** The
    /// scale states the tiles along one side of a weather cell, and one of
    /// the values it takes gives each tile a cell of its own.[^1]
    ///
    /// The cost of the field follows the cell count, so a fine pitch costs
    /// the world on every tick and a coarse one costs a small fraction of
    /// it. The default is the level 1 pitch.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// and when the scale does not describe a lattice over that extent.
    ///
    /// # References
    ///
    /// [^1]: The weather scale. [`WeatherScale`]
    /// Builds a world whose ground the weather of the world shaped.
    ///
    /// **The engine runs the weather forward over an empty world before the
    /// world starts.** It accumulates the temperature and the standing water
    /// of every weather cell over a fixed tick count, stores that small field,
    /// and the terrain readers of the world then read it. A cell that stands
    /// wetter than the world grows more forest. A cell that stands drier, or
    /// colder, grows less.
    ///
    /// **The spin is not a loop.** The weather reads the mean height of a cell
    /// and its open water share, and the climate changes neither. The spin
    /// hands the weather the ground folded from the terrain that the seed
    /// alone gives, the classification runs once afterwards, and nothing feeds
    /// back. The tick count is fixed, so there is no convergence test.[^1]
    ///
    /// A tick count of zero builds the world that the seed alone gives.
    ///
    /// # Errors
    ///
    /// Returns an error when the world refuses to build, and when the weather
    /// refuses the spin.
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    pub fn with_climate(
        config: WorldConfig,
        weather_scale: WeatherScale,
        spin_ticks: u64,
        threads: usize,
    ) -> Result<Self, WorldError> {
        let mut world = Self::with_weather_scale(config, weather_scale)?;
        if spin_ticks == 0 {
            return Ok(world);
        }
        let climate = ClimateField::spin(world.terrain, weather_scale, spin_ticks, threads)?;
        world.climate_reference = climate.wetness_reference();
        world.climate = climate;
        Ok(world)
    }

    pub fn with_weather_scale(
        config: WorldConfig,
        weather_scale: WeatherScale,
    ) -> Result<Self, WorldError> {
        Self::with_weather_margin(config, weather_scale, weather_scale.margin_cells())
    }

    /// Builds a world at a stated weather resolution and a stated margin.
    ///
    /// **The margin is the ring of weather cells around the world that the
    /// solve steps and no reader sees.** It exists so that the border of the
    /// world has real upwind. Without it, air leaves through one edge of the
    /// lattice and nothing arrives through the other, so the cells beside an
    /// edge stay starved and a mass can only be born inside the frame.
    ///
    /// The scale derives the margin that a world takes when the caller states
    /// none, from the distance the transport carries air while a mass
    /// forms.[^1]
    ///
    /// **A margin of zero gives the lattice that covers the world and nothing
    /// more**, which is the field the engine held before the margin existed.
    /// A test states that the two agree.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// when the scale does not describe a lattice over that extent, and when
    /// the margin makes the lattice too large to index.
    ///
    /// # References
    ///
    /// [^1]: The margin derivation. [`WeatherScale::margin_cells`]
    pub fn with_weather_margin(
        config: WorldConfig,
        weather_scale: WeatherScale,
        weather_margin: u32,
    ) -> Result<Self, WorldError> {
        Self::with_weather_latitudes(config, weather_scale, weather_margin, Latitudes::DEFAULT)
    }

    /// Builds a world at a stated weather resolution, margin and latitude
    /// span.
    ///
    /// **The span says what the row axis of the world means.** A span from
    /// pole to pole makes the world a planet: it has poles, a banded
    /// circulation, subtropical deserts and an equatorial rain belt. A narrow
    /// span makes the world one region of a planet, and the latitude term
    /// then goes flat and the climate comes from the ground and the sea
    /// alone.[^1]
    ///
    /// A world that states no span is a planet.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the configured extent does not describe a grid,
    /// when the scale does not describe a lattice over that extent, and when
    /// the margin makes the lattice too large to index.
    ///
    /// # References
    ///
    /// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D1. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    pub fn with_weather_latitudes(
        config: WorldConfig,
        weather_scale: WeatherScale,
        weather_margin: u32,
        latitudes: Latitudes,
    ) -> Result<Self, WorldError> {
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
        // **The weather lattice states its own pitch.** It is the level 1
        // lattice when the world takes the level 1 pitch, and a lattice of
        // its own at any other. The ground under it is folded once here,
        // because every field the weather reads from it is a pure function
        // of the seed and the address.[^3]
        //
        // [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        let weather_layout = BlockLayout::new(grid, weather_scale.bits())?;
        // **The weather lattice is larger than the world.** The margin is the
        // ring of cells around it that the solve steps and no reader sees. It
        // gives the border of the world real upwind: air that arrives at an
        // edge has crossed ground of the right character rather than arriving
        // from nothing.[^4]
        //
        // [^4]: The margin derivation. [`WeatherScale::margin_cells`]
        let weather_cells = Grid::new(weather_layout.blocks_wide(), weather_layout.blocks_high())?;
        let weather_lattice = PaddedLattice::new(weather_cells, weather_margin)?;
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
            presence: PresenceRelation::new(),
            observation: Observation::new(layout),
            terrain,
            resources: ResourceField::new(terrain),
            depletion: DepletionLedger::new(),
            departed: [0; RESOURCE_KIND_COUNT],
            delivered: [0; RESOURCE_KIND_COUNT],
            upgrades: UpgradeMap::new(),
            pyramid: Pyramid::new(layout, ResourceField::new(terrain))?,
            exits: ExitField::new(cell_lattice),
            returns: ReturnField::new(cell_lattice, config.faction_count),
            home_approaches: ApproachField::new(layout),
            stock_approaches: ApproachField::new(layout),
            destinations: SeededField::new(cell_lattice, config.destination_plane_count()),
            approaches: ApproachField::new(layout),
            destination_seeds: vec![Vec::new(); config.destination_plane_count() as usize],
            destination_crossings: vec![0; config.destination_plane_count() as usize],
            carry_mark: CARRY_MARK_DEFAULT,
            holding: Holding::new(layout),
            luxuries: LuxuryField::new(),
            variety: VarietyLevel::derive(layout, &LuxuryField::new()),
            luxuries_seeded: false,
            influence: InfluenceField::new(cell_lattice, config.faction_count)?,
            weather: WeatherField::with_latitudes(
                weather_lattice,
                weather_scale,
                latitudes,
                config.faction_count,
            )?,
            weather_layout,
            weather_lattice,
            weather_ground: ground_over_lattice(weather_lattice, weather_layout, terrain),
            climate: ClimateField::quiet(weather_layout),
            climate_reference: 0,
            controller: Controller::new(config.seed, config.faction_count),
            balance: Balance::default(),
            campaigns: CampaignRegister::new(config.faction_count),
            plan: PlanRegister::new(config.faction_count, PlanRules::DEFAULT),
            census: CensusTotals::default(),
            schedule: RateSchedule::DEFAULT,
            rates: RateTable::new(),
            effective_rates: RateTable::new(),
            rate_ledger: RateLedger::ZERO,
            shortfall_log: Vec::new(),
            need_rule: NeedRule::DEFAULT,
            cohorts: CohortTable::new(),
            draw_ledger: DrawLedger::ZERO,
            rationed_log: Vec::new(),
            death_plane: DeathPlane::new(),
            starved_log: Vec::new(),
            // The world is built with the default table, so a unit that
            // nothing typed is a worker and gathers, builds and carries.[^1]
            //
            // [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
            unit_types: DEFAULT_UNIT_TYPE_TABLE,
            queues: QueueTable::new(),
            build_costs: BuildCostTable::default(),
            queue_schedule: RateSchedule::new(QUEUE_PERIOD_DEFAULT, QUEUE_PHASE_DEFAULT)
                .expect("the default period is inside the range"),
            growth_schedule: RateSchedule::new(
                growth::GROWTH_PERIOD_DEFAULT,
                growth::GROWTH_PHASE_DEFAULT,
            )
            .expect("the default period is inside the range"),
            housing_per_person: growth::HOUSING_PER_PERSON_DEFAULT,
            food_per_birth: growth::FOOD_PER_BIRTH_DEFAULT,
            birth_chance: growth::BIRTH_CHANCE_DEFAULT,
            founding_housing: growth::FOUNDING_HOUSING_DEFAULT,
            births: 0,
            growth_ledger: [Accum(0); COMMODITY_COUNT],
            // The world is built with the default upgrade table, so a build
            // order names one of the categories that table holds.[^2]
            //
            // [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
            upgrade_table: upgrade::DEFAULT_UPGRADE_TABLE,
            fell_plane: DeathPlane::new(),
            fell_log: Vec::new(),
            grievances: Vec::new(),
            relations: RelationMatrix::new(config.faction_count),
            convert_marks: Vec::new(),
            converted_log: Vec::new(),
            promoted_log: Vec::new(),
            character_schedule: RateSchedule::DEFAULT,
            promotion_budget: u32::MAX,
            choice: ChoiceSchedule::DEFAULT,
            buckets: NeedBuckets::DEFAULT,
            weights: WeightProfile::EVEN,
            positions: PositionTable::new(),
            position_schedule: RateSchedule::DEFAULT,
            store_account: [Accum(0); COMMODITY_COUNT],
            trade: TradeTable::new(config.faction_count),
            trade_log: Vec::new(),
            collapsed_log: Vec::new(),
            finished_log: Vec::new(),
            founded_log: Vec::new(),
            taken_log: Vec::new(),
            siege_rules: SiegeRules::DEFAULT,
            sites_captured: 0,
            sites_razed: 0,
            sieges_pressed: 0,
            sieges_relieved: 0,
            eliminated_log: Vec::new(),
            eliminated: vec![0u8; FACTION_CEILING as usize],
            market: MarketTable::new(config.faction_count, DEFAULT_BOARD_ROWS),
            land_list_bound: DEFAULT_LAND_LIST_BOUND,
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
        // A world that has never stepped still answers the presence
        // question. A relation that nothing derived would refuse the read,
        // and a caller cannot tell that refusal from a stale one.[^2]
        //
        // [^2]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        world.presence.rebuild(&world.soldiers, &world.holding, 1)?;
        // A world that has never stepped still answers what a faction sees.
        // A layer that nothing rebuilt would say that every faction is
        // blind, and a caller cannot tell that from a faction with no
        // unit.[^3]
        //
        // [^3]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
        world
            .observation
            .rebuild(&world.soldiers, world.terrain, 1)?;
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
    /// computes the tile. It reads no array of tiles, so it never goes
    /// stale.[^1]
    ///
    /// **The climate of the world reaches the answer.** A world that asked for
    /// no spin holds a quiet climate, which reads temperate at every address
    /// and changes nothing, so the answer is the tile that the seed alone
    /// gives.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^2]: The climate field. [`ClimateField`]
    #[must_use]
    pub fn tile_terrain(&self, address: Axial) -> Option<TerrainTile> {
        self.terrain.tile_under(address, self.tile_climate(address))
    }

    /// Returns the terrain kind of one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn tile_kind(&self, address: Axial) -> Option<TileKind> {
        Some(self.tile_terrain(address)?.kind)
    }

    /// Returns the climate over one tile.
    ///
    /// A world that asked for no spin answers temperate at every address.
    #[must_use]
    pub fn tile_climate(&self, address: Axial) -> Climate {
        self.climate.at_reference(address, self.climate_reference)
    }

    /// Returns the climate field of the world.
    #[must_use]
    pub const fn climate(&self) -> &ClimateField {
        &self.climate
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

    /// Returns the faction of one soldier.
    ///
    /// Returns `None` when the identity names no live soldier.
    ///
    /// **This is a point read.** A caller that wants the units of a faction
    /// asks for the whole set instead, because the control plane never walks
    /// the population one unit at a time.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn soldier_faction(&self, entity: Entity) -> Option<FactionId> {
        self.soldiers.faction(entity)
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

    /// Sends a set of units to a set of tiles, through one destination plane.
    ///
    /// **The control plane names the seed set, and the engine builds one
    /// field.** The seeds are the tiles the caller wants the units at. The
    /// engine takes the level 1 cell of each of them, seeds the plane at every
    /// one, and relaxes a reach outward from the whole set at once. Every unit
    /// the call names then reads one entry of that plane and steps.[^1] [^2]
    ///
    /// **No unit gains a search.** A unit reads the entry of its own cell and
    /// its own plane. It reads no neighbouring cell, it scores no neighbour,
    /// and it computes nothing from its own address toward a destination.[^3]
    ///
    /// **The set is all or nothing.** Every identity resolves and every
    /// address is inside the world before anything changes.
    ///
    /// The order of the seeds does not reach the field. The engine sorts the
    /// cells and holds each of them once, so two calls that name one set in
    /// two orders derive one field.[^4]
    ///
    /// A caller that names the same plane again replaces the seed set of that
    /// plane. Every unit already sent to it then climbs the new one.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no destination plane, when an
    /// identity names no live soldier, or when an address is outside the
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
    /// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn send_units_to(
        &mut self,
        units: &[Entity],
        seeds: &[Axial],
        destination: u16,
    ) -> Result<(), SendError> {
        if destination >= self.destinations.plane_count() {
            return Err(SendError::NoSuchDestination(destination));
        }
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(SendError::DeadUnit(*unit));
            }
        }
        let mut tiles = Vec::with_capacity(seeds.len());
        for address in seeds {
            let tile = self
                .grid
                .index_of(*address)
                .ok_or(SendError::AddressOutsideWorld(*address))?;
            // The cell is derived from the tile at every read, and it is
            // stored nowhere, so the two cannot disagree.
            self.cell_of(tile)
                .ok_or(SendError::AddressOutsideWorld(*address))?;
            tiles.push(tile);
        }
        // The set is a set. The sort and the dedup make the stored order a
        // property of the tiles and never of the order the caller named them
        // in.[^4]
        tiles.sort_unstable_by_key(|tile: &TileIdx| tile.0);
        tiles.dedup();
        self.destination_seeds[destination as usize] = tiles;
        // **The plane conducts through water when every unit sent to it
        // crosses water.** The caller states no flag. It states a set, and
        // the crossing of that set follows from the type of each unit in it,
        // which is a column of the shared table.[^6]
        //
        // A mixed set does not cross. The field is one direction for each
        // cell, so a plane that crossed for the whole set would steer the
        // units that the water refuses at a coast, which is the failure the
        // land rule was written against.[^7]
        //
        // An empty set does not cross. A send of no unit steers nobody, and
        // the safe answer costs nothing.
        //
        // [^6]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        // [^7]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
        let crosses = !units.is_empty()
            && units.iter().all(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            });
        self.destination_crossings[destination as usize] = u8::from(crosses);
        for unit in units {
            assert!(
                self.soldiers.set_sent(*unit, Some(destination)),
                "a resolved identity must name a soldier the arena can send"
            );
        }
        // The field is derived here as well as at the barrier. A caller reads
        // the direction between two steps, and a derived value that one path
        // leaves stale is a confident wrong answer.[^5]
        //
        // [^5]: Findings register, FND-029. `docs/FINDINGS.md`
        self.derive_destination_fields();
        Ok(())
    }

    /// Stops sending a set of units.
    ///
    /// Every unit the call names goes back to the option it chose for itself.
    /// **The set is all or nothing**: every identity resolves before anything
    /// changes.
    ///
    /// # Errors
    ///
    /// Returns an error when an identity names no live soldier.
    pub fn stop_sending(&mut self, units: &[Entity]) -> Result<(), SendError> {
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(SendError::DeadUnit(*unit));
            }
        }
        for unit in units {
            assert!(
                self.soldiers.set_sent(*unit, None),
                "a resolved identity must name a soldier the arena can stop"
            );
        }
        Ok(())
    }

    /// Returns the destination that the control plane sent one unit to.
    ///
    /// The outer option reports whether the identity names a live soldier.
    /// The inner one reports whether anybody sent it anywhere.
    #[must_use]
    pub fn sent_to(&self, entity: Entity) -> Option<Option<u16>> {
        self.soldiers.sent(entity)
    }

    /// Returns the number of destination planes that the world holds.
    #[must_use]
    pub const fn destination_count(&self) -> u16 {
        self.destinations.plane_count()
    }

    /// Sets the number of destination planes that the world holds.
    ///
    /// **The caller names the plane that carries an order, and the engine
    /// allocates none.** The count says how many places a control plane may
    /// send units to at one time, before it re-aims a plane it already
    /// used.[^1]
    ///
    /// The call clears the seed set of every plane, so no order steers
    /// anything until the caller sends a set again. A unit that was sent to a
    /// plane the world no longer holds reads no direction, and the next step
    /// releases it rather than leaving it sent for ever.[^2]
    ///
    /// The cost is the cell count times the number of planes, in bytes. No
    /// figure appears here, because one blocker governs every cost figure this
    /// project holds.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub fn set_destination_count(&mut self, count: u16) {
        self.destinations = SeededField::new(self.destinations.cells(), count);
        self.destination_seeds = vec![Vec::new(); count as usize];
        self.destination_crossings = vec![0; count as usize];
        self.derive_destination_fields();
    }

    /// Returns the direction that a unit sent to one destination takes from
    /// one address.
    ///
    /// The outer option reports whether the address and the destination name
    /// an entry. The inner one reports whether the cell holds a direction at
    /// all. **A cell that holds a seed, and a cell the reach never arrived
    /// at, both hold none.** The fine field answers the first case at the
    /// pitch of one tile, and the step releases a sent unit that neither
    /// field steers.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub fn destination_direction(&self, destination: u16, address: Axial) -> Option<Option<u8>> {
        let tile = self.grid.index_of(address)?;
        self.destinations
            .direction(destination, self.cell_of(tile)?)
    }

    /// Returns the destination field of the world.
    ///
    /// The field holds one direction for each level 1 cell and each
    /// destination plane.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub const fn destination_field(&self) -> &SeededField {
        &self.destinations
    }

    /// Returns one seed for each destination plane and each cell of its set.
    ///
    /// The walk is over the planes in ascending order and over the cells of
    /// each plane in ascending order, so the set does not depend on a thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn destination_seed_pairs(&self) -> Vec<(u16, u32)> {
        let mut seeds = Vec::new();
        for (plane, tiles) in self.destination_seeds.iter().enumerate() {
            for tile in tiles {
                let Some(cell) = self.cell_of(*tile) else {
                    continue;
                };
                seeds.push((plane as u16, cell));
            }
        }
        seeds
    }

    /// Returns one seed for each destination plane and each tile of its set.
    ///
    /// The walk is over the planes in ascending order and over the tiles of
    /// each plane in ascending order, so the set does not depend on a thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn destination_seed_tiles(&self) -> Vec<(u16, TileIdx)> {
        let mut seeds = Vec::new();
        for (plane, tiles) in self.destination_seeds.iter().enumerate() {
            for tile in tiles {
                seeds.push((plane as u16, *tile));
            }
        }
        seeds
    }

    /// Derives the coarse and the fine field of every destination plane.
    ///
    /// **This is the one place that derives either of them.** Both come from
    /// one seed set, and a path that wrote one without the other would leave
    /// a stale value that nothing fails on.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn derive_destination_fields(&mut self) {
        self.destinations.derive(
            &self.pyramid,
            &self.destination_seed_pairs(),
            &self.destination_crossings,
        );
        self.approaches.derive(
            self.terrain,
            &self.destination_seed_tiles(),
            &self.destination_crossings,
        );
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

    /// Returns what every delivery has moved into a store, for each kind.
    #[must_use]
    pub const fn delivered_carry(&self) -> &[u64; RESOURCE_KIND_COUNT] {
        &self.delivered
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
        // The queue table follows the same slot column, for the same reason.
        // A new slot holds an empty queue.
        self.queues.open_to(self.settlements.slot_count());
        // **A founded site starts with the housing of the world parameter.**
        // A founded site must house the group that founds it, or a run starts
        // crowded. The housing is stored and the ground never sets it, so the
        // founding writes it here and no pass derives it later.[^5]
        //
        // [^5]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        self.settlements
            .set_housing(settlement, self.founding_housing);
        // **This is the one place a settlement comes into existence, so it is
        // the one place that says so.** Every caller path and the settle
        // system pass through here. The census row is a count of what stands,
        // so it falls when a settlement is lost and it hides a founding in
        // the same tick.
        //
        // A founding between two steps stays in the log until the next step
        // clears it.
        if let Some(tile) = self.grid.index_of(address) {
            self.founded_log.push(SettlementFounded::new(
                self.tick,
                settlement.to_bits(),
                tile,
                faction,
            ));
        }
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
        self.record_seat(faction, place);
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
        self.record_seat(faction, address);
        Ok(Founding::new(chosen.address(), settlement, people, survey))
    }

    /// Returns the place of every settlement that stands, in slot order.
    ///
    /// This is the list a founding keeps its distance from. It is derived
    /// from the arena on every call, so no second copy of it can disagree
    /// with the settlements the world holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn standing_places(&self) -> Vec<Axial> {
        self.settlements
            .iter()
            .filter_map(|site| self.settlements.address(site))
            .collect()
    }

    /// Founds a city from each settler of a set, and spends the settler.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The verb reads that column and never a type index, so no rule here
    /// names a type.[^1] It founds a settlement on the tile the unit stands
    /// on, for the faction of the unit, and it seats the group the column
    /// names.[^2]
    ///
    /// **The founding keeps the distance that the seeding keeps.** The place
    /// of every settlement that stands is the list the survey compares
    /// against, and the comparison is the one the seeding uses. A place
    /// inside that distance is refused.[^3]
    ///
    /// The verb refuses a unit that no live unit answers to, a unit whose
    /// settle column is zero, a faction the world does not hold, a tile any
    /// faction holds, a tile that carries a settlement, ground that admits no
    /// unit, and a place inside the founding distance. A refused unit changes
    /// nothing, and a set in which the verb refuses every unit changes
    /// nothing at all.[^2]
    ///
    /// **The founding spends the settler.** The unit leaves the world after
    /// the settlement stands, and the group the column names takes its
    /// place.[^2]
    ///
    /// The units are answered in the order the caller gave. A founding by an
    /// earlier unit of the set enters the distance list of the units after
    /// it, so a set of two settlers on one tile founds one city.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    #[must_use]
    pub fn settle_set(&mut self, units: &[Entity]) -> Vec<SettleOutcome> {
        units
            .iter()
            .map(|unit| {
                let result = self.settle_one(*unit);
                SettleOutcome::new(*unit, result)
            })
            .collect()
    }

    /// Founds a city from one settler, and spends it.
    fn settle_one(&mut self, unit: Entity) -> Result<Founding, SettleError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let (address, group, faction, survey) = self.settle_refusal(unit)?;
        let chosen = survey
            .candidates()
            .first()
            .copied()
            .ok_or(SettleError::OutsideWorld(address))?;
        let (settlement, people) = self.settle_group(address, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        // The seat of a faction is the tile of its first founding, and a
        // settler founds after that one. The call leaves a seat that stands.
        self.record_seat(faction, address);
        // The settler is spent. It leaves after the settlement stands, so a
        // refusal above never costs the unit.[^1]
        //
        // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        self.despawn_soldier(unit);
        Ok(Founding::new(address, settlement, people, survey))
    }

    /// Reports whether the settle verb would refuse one settler, without
    /// founding anything.
    ///
    /// **This is the one statement of the rule.** The settle verb calls it
    /// before it seats a group, and the legality answer calls it to fill one
    /// row of the action table.[^1] Nothing here mutates. It returns the
    /// place, the group size, the faction and the survey, so that the verb
    /// does not read them twice.
    ///
    /// # Errors
    ///
    /// Returns the refusal the settle verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn settle_refusal(
        &self,
        unit: Entity,
    ) -> Result<(Axial, u32, FactionId, Survey), SettleError> {
        let (Some(unit_type), Some(address), Some(faction)) = (
            self.soldiers.unit_type(unit),
            self.soldiers.address(unit),
            self.soldiers.faction(unit),
        ) else {
            return Err(SettleError::NoSuchUnit(unit));
        };
        let group = self.unit_types.row(unit_type).settle_group;
        if group == 0 {
            return Err(SettleError::NotASettler(unit));
        }
        if !self.grid.contains(address) {
            return Err(SettleError::OutsideWorld(address));
        }
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SettleError::FactionMayNotFound(faction));
        }
        // A tile belongs to the faction of the nearest city within reach, and
        // a settler founds only where nobody holds.[^1]
        //
        // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        if self
            .holding
            .holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            return Err(SettleError::GroundIsHeld(address));
        }
        if self.settlement_on(address).is_some() {
            return Err(SettleError::SettlementStands(address));
        }
        // The distance list and the eligibility both come from the survey the
        // seeding uses, so the distance rule has one statement in the tree.
        let taken = self.standing_places();
        let survey = founding::survey_addresses(self.resources, &[address], group, &taken)?;
        let chosen = survey
            .candidates()
            .first()
            .copied()
            .ok_or(SettleError::OutsideWorld(address))?;
        if !chosen.is_separated() {
            return Err(SettleError::TooCloseToACity(address));
        }
        if !chosen.is_eligible() {
            return Err(SettleError::GroundAdmitsNobody(address));
        }
        Ok((address, group, faction, survey))
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
        // A lost settlement takes its queue with it. A block left as it was
        // would give the settlement founded next in that slot the orders of
        // the one before it.
        self.queues.clear_slot(slot);
        // A unit that drew from the lost site now belongs to no site. A home
        // left behind would name the slot, and the settlement founded next
        // in that slot would feed a population it never took.
        for unit in self.soldiers.iter().collect::<Vec<_>>() {
            if self.soldiers.home(unit) == Some(Some(slot)) {
                self.soldiers.set_home(unit, None);
            }
        }
        for (index, account) in self.store_account.iter_mut().enumerate() {
            let held = store
                .quantity(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            *account = sim_math::combine(*account, Accum(-i64::from(held.0)));
        }
        true
    }

    /// Presses the siege against every site that a rival occupies
    /// undefended, and takes or burns the ones that fall.
    ///
    /// **A site falls to work and never to a moment.** A faction that stands
    /// on a site tile with no unit of the owning faction on it besieges the
    /// site. The siege does one work for each besieging unit on the tile, on
    /// each tick it stands. The site changes hands when the work reaches what
    /// the site resists, and the resistance is the residents the site
    /// holds.[^6]
    ///
    /// **A garrison of one refuses the siege, and a relief force ends one.**
    /// The trigger is read again on every tick. The tick the owner puts a
    /// unit back on the tile, or the tick the besieger leaves, the siege ends
    /// and its work is gone. A besieger that returns starts at nothing. A
    /// besieged city therefore has two answers: keep a unit at home, or send
    /// one back before the work is done.[^7]
    ///
    /// **The occupier is the faction the occupancy list names, and that list
    /// is built once for the tick.** The lease pass reads it to move a lease
    /// and this pass reads it to decide a siege, so who stands on a tile is
    /// stated once. The list names the faction with the most units on the
    /// tile, and a tie goes to the lowest faction identifier. Two factions
    /// that could besiege one site therefore resolve by a rule and never by
    /// an iteration order, and the work of the faction that loses the tile is
    /// gone.[^2] [^3]
    ///
    /// **What stands at a kept site passes to the taker whole.** The store,
    /// the housing, the rates, the staff and the upgrades on the ground are
    /// untouched, and every unit that draws from the site changes faction
    /// with it. Only the queue is cleared, because a queue holds orders that
    /// the taker never gave.[^4]
    ///
    /// **The taker keeps a city it can supply and burns one it cannot, and
    /// burning costs more.** A city of the taker supplies the captured site
    /// when the site stands inside the reach of that city. The reach is the
    /// quantity the ground rule already computes for every city on every
    /// tick, and the upgrades a faction finishes inside its own ground extend
    /// it to a bound.[^5] A supplied site changes hands at the capture work.
    /// A site no city of the taker reaches does not fall there: the siege
    /// presses on to the raze work, which is a multiple of the capture work,
    /// and the site burns when the siege reaches that.[^6]
    ///
    /// The pass walks the settlements in slot order, and the occupancy list
    /// is in ascending tile order. Both are stable keys.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D3. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0180, a site changes hands or the taker destroys it, decisions D1 and D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^5]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^6]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^7]: ADR-0180, a site changes hands or the taker destroys it, decision D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    fn capture_sites(&mut self, occupancy: &[(TileIdx, FactionId)]) {
        // **The pass reads every site before it writes one.** A write moves
        // the faction of a site and the faction of its residents, and a later
        // read would then see a world that the earlier sites of this same
        // tick had changed. The plan for each site is fixed first.
        let mut plans: Vec<(Entity, SiegeStep)> = Vec::new();
        let mut any_fell = false;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            let step = self.siege_step(site, occupancy);
            if matches!(step, SiegeStep::Falls { .. }) {
                any_fell = true;
            }
            plans.push((site, step));
        }
        // **The taker keeps a city it can supply and burns one it cannot.**
        // The reach of a city is the quantity the ground rule already
        // computes for every city on every tick, and the upgrades a faction
        // finishes inside its own ground are what extend it.[^8] A road
        // between two cities therefore decides which conquests a faction can
        // keep, and nothing here states a distance of its own.
        //
        // The list is built once for the tick, and only on a tick that has a
        // site at its capture work. A tick with none pays nothing for it.
        //
        // [^8]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        let cities = if any_fell {
            self.holding.cities(&self.settlements, &self.upgrades)
        } else {
            Vec::new()
        };
        let mut moved = false;
        for (site, step) in plans {
            match step {
                SiegeStep::Ends => {
                    if self.settlements.siege(site).is_some() {
                        self.sieges_relieved =
                            sim_math::combine(Accum(self.sieges_relieved), Accum(1)).0;
                    }
                    let _ = self.settlements.clear_siege(site);
                }
                SiegeStep::Waits => {}
                SiegeStep::Presses { besieger, work } => {
                    self.sieges_pressed = sim_math::combine(Accum(self.sieges_pressed), Accum(1)).0;
                    let _ = self.settlements.set_siege(site, besieger, work);
                }
                SiegeStep::Falls {
                    besieger,
                    work,
                    raze_work,
                    ordered,
                } => {
                    self.sieges_pressed = sim_math::combine(Accum(self.sieges_pressed), Accum(1)).0;
                    let Some(address) = self.settlements.address(site) else {
                        continue;
                    };
                    // **A faction that ordered a raze burns the site,
                    // whatever the reach says.** The order is the one way a
                    // caller states an intent the engine would not have
                    // chosen, and it costs the raze work like every other
                    // raze.
                    if ordered == 1 {
                        if work >= raze_work {
                            moved |= self.burn_site(site, besieger);
                        } else {
                            let _ = self.settlements.set_siege(site, besieger, work);
                        }
                        continue;
                    }
                    // The site being taken still belongs to the faction that
                    // is losing it, so the taker's own cities are the only
                    // ones this walk sees.
                    let mut holds_one = false;
                    let mut supplied = false;
                    for city in &cities {
                        if city.faction != besieger {
                            continue;
                        }
                        holds_one = true;
                        if address.distance(city.address) <= city.reach {
                            supplied = true;
                            break;
                        }
                    }
                    // **A taker that holds no city keeps what it takes.** The
                    // rule asks which of the taker's cities supplies this
                    // one, and a faction with none is not a faction that
                    // failed to reach it. A captured city is then the only
                    // city that faction has, and it supplies itself. Without
                    // this the last army of a beaten faction could never take
                    // a capital, and a faction that lost every city could
                    // never return.
                    if supplied || !holds_one {
                        moved |= self.take_site(site, besieger);
                    } else if work >= raze_work {
                        moved |= self.burn_site(site, besieger);
                    } else {
                        // The taker cannot supply the site, so the siege
                        // presses on to the raze work. The city stands, and
                        // its owner has every tick until then to relieve it.
                        let _ = self.settlements.set_siege(site, besieger, work);
                    }
                }
            }
        }
        let took = moved;
        // **A capture writes the faction of a unit, and that moves the arena
        // past the derived structure.** The unit stands where it stood, so
        // the rebuild gives the same structure back and only restamps the
        // revision. A pass after this one reads the structure and refuses a
        // stale one, so the restamp cannot wait for the next barrier.
        if took {
            debug_assert!(
                self.refresh_bridge().is_ok(),
                "the arena and the structure describe one world"
            );
            let _ = self.refresh_bridge();
        }
    }

    /// Returns what the siege pass does to one site this tick.
    ///
    /// The call writes nothing. It reads the trigger, adds this tick's work
    /// to the work the siege carried, and compares the total against what the
    /// site resists.[^1]
    ///
    /// **The work the siege carried counts only when the same faction did
    /// it.** The occupancy names one faction for a tile, so a site whose
    /// besieger loses the tile starts again at nothing under whoever takes
    /// it. Two factions that could besiege one site therefore resolve by the
    /// rule that decides the occupier, and never by an iteration order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    fn siege_step(&self, site: Entity, occupancy: &[(TileIdx, FactionId)]) -> SiegeStep {
        let Some(tile) = self.settlements.tile(site) else {
            return SiegeStep::Waits;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return SiegeStep::Waits;
        };
        // **A garrison of one refuses the siege, and it takes the work with
        // it.** The defence is read from the units that stand on the tile and
        // never from the occupancy, because the occupancy names only the
        // largest faction and a garrison of one is rarely that.
        let factions = self.soldiers.faction_column();
        let standing = self.bridge.on_tile_unguarded(tile);
        if standing
            .iter()
            .any(|unit| factions[unit.index() as usize] == owner)
        {
            return SiegeStep::Ends;
        }
        let Ok(entry) = occupancy.binary_search_by_key(&tile.0, |(tile, _)| tile.0) else {
            return SiegeStep::Waits;
        };
        let besieger = occupancy[entry].1;
        if besieger == owner {
            return SiegeStep::Waits;
        }
        // **The force is the units of the besieging faction on the tile.** A
        // besieger does one work a tick, which is what a builder does, so a
        // larger army takes a city sooner and one unit takes a long time.
        let force = standing
            .iter()
            .filter(|unit| factions[unit.index() as usize] == besieger)
            .count();
        if force == 0 {
            return SiegeStep::Waits;
        }
        let carried = match self.settlements.siege(site) {
            Some((who, work)) if who == besieger => work,
            _ => 0,
        };
        let work = sim_math::combine(Accum(carried), Accum(force as i64)).0;
        let residents = self.site_residents(site).unwrap_or(0);
        if work < self.siege_rules.capture_work(residents) {
            return SiegeStep::Presses { besieger, work };
        }
        SiegeStep::Falls {
            besieger,
            work,
            raze_work: self.siege_rules.raze_work(residents),
            ordered: u8::from(
                self.settlements.siege_intent(site) == Some(crate::site::SIEGE_INTENT_RAZE),
            ),
        }
    }

    /// Moves one standing site to a new faction, with everything it holds.
    ///
    /// **This is the one place a capture happens.** The site keeps its
    /// identity, so a stored handle to a captured site still resolves.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    fn take_site(&mut self, site: Entity, taker: FactionId) -> bool {
        let Some(slot) = self.settlements.slot_of(site) else {
            return false;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return false;
        };
        let Some(tile) = self.settlements.tile(site) else {
            return false;
        };
        if owner == taker || !self.settlements.set_faction(site, taker) {
            return false;
        }
        // The siege ends with the thing it stood against. A site that changed
        // hands is no longer besieged, and the work that took it is spent.
        let _ = self.settlements.clear_siege(site);
        // **The residents change hands with the site.** A resident is a live
        // unit whose home names this slot, and the household is the home
        // column read backwards. Conquest makes the taker larger, and a
        // resident left with its old faction would be a person inside a city
        // that is no longer its own.
        //
        // The walk is over the live units in slot order, which is a stable
        // key.
        for unit in self.soldiers.iter().collect::<Vec<_>>() {
            if self.soldiers.home(unit) != Some(Some(slot)) {
                continue;
            }
            self.soldiers.set_faction(unit, taker);
            // **A character changes hands with the unit that carries it.** A
            // unit names a character, and the character carries a faction of
            // its own. A resident that changed faction while its character
            // did not would leave the two disagreeing, and no pass reads both
            // to notice.[^1]
            //
            // [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
            let bits = self.soldiers.character_column()[unit.index() as usize];
            if let Some(character) = Entity::from_bits(bits) {
                self.characters.set_faction(character, taker);
            }
        }
        // A queue holds orders the previous faction gave. The taker gave none
        // of them, so the block is cleared by the same call a loss uses. Work
        // that stands on the ground is an upgrade on a tile and it is
        // untouched, so a part-built upgrade changes hands with the ground
        // and a part-built order does not.
        self.queues.clear_slot(slot);
        // The cohort table indexes the residents by site and by faction, and
        // the faction of every resident has just changed.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        self.sites_captured = sim_math::combine(Accum(self.sites_captured), Accum(1)).0;
        self.taken_log.push(SiteTaken::new(
            self.tick,
            site.to_bits(),
            tile,
            owner,
            taker,
            TAKE_KIND_CAPTURED,
        ));
        true
    }

    /// Orders the siege of a faction to destroy the site rather than take
    /// it.
    ///
    /// **A raze is an order and never a moment.** The order writes the
    /// intent of a siege that already stands. The siege then presses to the
    /// work a raze costs, and the site burns when the work reaches it. The
    /// order therefore costs the razing faction the same time and the same
    /// force that the engine charges every other raze.[^1]
    ///
    /// **This is the one thing a caller states that the engine would not.**
    /// The engine keeps a site its own reach supplies and burns one it does
    /// not.[^2] A caller that wants a supplied city burned orders this, and
    /// the reach then decides nothing.
    ///
    /// The order lasts as long as the siege. A relief force that ends the
    /// siege ends the order with it, and a besieger that comes back must
    /// order again.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no live site, when the razer
    /// owns the site, and when no siege of the razer stands against it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D9 and D11. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^3]: ADR-0180, a site changes hands or the taker destroys it, decision D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    pub fn order_raze(&mut self, site: Entity, razer: FactionId) -> Result<(), RazeError> {
        let owner = self
            .settlements
            .faction(site)
            .ok_or(RazeError::NoSuchSite)?;
        if owner == razer {
            return Err(RazeError::OwnSite);
        }
        match self.settlements.siege(site) {
            Some((besieger, _)) if besieger == razer => {}
            _ => return Err(RazeError::NotBesieging),
        }
        if !self
            .settlements
            .set_siege_intent(site, crate::site::SIEGE_INTENT_RAZE)
        {
            return Err(RazeError::NotBesieging);
        }
        Ok(())
    }

    /// Destroys a site and pays its store to the faction that took it.
    ///
    /// **This is the one place a raze happens.** The caller's verb and the
    /// capture pass both go through it, and neither repeats the work. The
    /// caller checks the trigger and refreshes the derived unit structure
    /// afterwards, because the two callers reach this from different points
    /// of a step.
    ///
    /// Returns `false` when the identity names no live site.
    fn burn_site(&mut self, site: Entity, razer: FactionId) -> bool {
        let Some(slot) = self.settlements.slot_of(site) else {
            return false;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return false;
        };
        let Some(tile) = self.settlements.tile(site) else {
            return false;
        };
        let store = self
            .settlements
            .store(site)
            .expect("the identity resolved to a live slot");
        // The plunder moves before the loss, because the loss subtracts what
        // the site still holds from the account. Moving it first and clearing
        // it leaves the account exactly as it was.
        if let Some(receiver) = self.nearest_site_of(razer, tile) {
            for index in 0..COMMODITY_COUNT {
                let commodity = CommodityId(index as u16);
                let carried = store
                    .quantity(commodity)
                    .expect("the index came from the commodity count");
                let held = self
                    .settlements
                    .store(receiver)
                    .expect("the nearest site is live")
                    .quantity(commodity)
                    .expect("the index came from the commodity count");
                let _ =
                    self.settlements
                        .set_store(receiver, commodity, sim_math::add(held, carried));
                let _ = self.settlements.set_store(site, commodity, Fix32::ZERO);
            }
        }
        // The upgrades on the tile go with the site. An upgrade changes hands
        // with the ground, so a razer that wanted the roads and the walls
        // should have captured instead.[^3]
        //
        // [^3]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
        self.upgrades.remove(tile);
        // The residents die with the site. The despawn is the one path a unit
        // leaves the world by, so a razed resident leaves the arena, the
        // structure and the cohorts by the route a killed one takes.
        let residents: Vec<Entity> = self
            .soldiers
            .iter()
            .filter(|unit| self.soldiers.home(*unit) == Some(Some(slot)))
            .collect();
        for unit in residents {
            self.despawn_soldier(unit);
        }
        if !self.destroy_settlement(site) {
            return false;
        }
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        self.sites_razed = sim_math::combine(Accum(self.sites_razed), Accum(1)).0;
        self.taken_log.push(SiteTaken::new(
            self.tick,
            site.to_bits(),
            tile,
            owner,
            razer,
            TAKE_KIND_RAZED,
        ));
        true
    }

    /// Returns the live site of a faction that stands nearest a tile.
    ///
    /// A tie between two sites at one distance goes to the lower settlement
    /// slot, because the walk is in ascending slot order and the comparison
    /// is strict.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn nearest_site_of(&self, faction: FactionId, tile: TileIdx) -> Option<Entity> {
        let address = self.grid.address_of(tile)?;
        let mut best: Option<(u32, Entity)> = None;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            let Some(seat) = self.settlements.address(site) else {
                continue;
            };
            let distance = address.distance(seat);
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, site));
            }
        }
        best.map(|(_, site)| site)
    }

    /// Reports whether a faction has left the game.
    ///
    /// A faction outside the ceiling has never been in the game, and this
    /// answers `false` for it.
    #[must_use]
    pub fn is_eliminated(&self, faction: FactionId) -> bool {
        self.eliminated
            .get(faction.0 as usize)
            .is_some_and(|state| *state == 1)
    }

    /// Removes from the game every faction that holds no site and no unit.
    ///
    /// **A faction leaves the game when nothing of it can act and nothing of
    /// it can grow.** A site grows people and a unit acts, so a faction with
    /// neither has no way back into the world. It cannot found, because a
    /// founding needs a unit. It cannot build, gather, fight or take ground.
    /// A rule that waited for more would wait for ever.[^1]
    ///
    /// **A character alone does not keep a faction in play.** A character
    /// carries no tile, so it stands nowhere. It holds no ground, it takes no
    /// step and it fights nothing.[^2] The pass removes the characters of a
    /// faction that leaves, so nothing outlives the faction and no stored row
    /// names a faction that is out.
    ///
    /// **The ground it held is released to nobody.** The spread of the next
    /// tick then gives each released tile to the nearest city that reaches
    /// it, by the rule that already decides every holder, and a tile that no
    /// city reaches stays with nobody.[^3] Nothing here states a second rule
    /// for who inherits.
    ///
    /// **The pass runs once for each tick and it does not cascade.** Ground
    /// released by one elimination reaches another faction on the next tick,
    /// through the spread. A pass that settled until quiet would run a
    /// variable number of times, and no pass in this engine does that.[^4]
    ///
    /// The walk is over the factions in ascending identifier order, which is
    /// a stable key. Two factions that leave on one tick are recorded in that
    /// order.[^5]
    ///
    /// # References
    ///
    /// [^1]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D1. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^2]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    /// [^3]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D3. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^4]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn eliminate_factions(&mut self, threads: usize) {
        let population = *self.soldiers.population_by_faction();
        let mut sites = [0u32; FACTION_CEILING as usize];
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            if let Some(faction) = self.settlements.faction(site) {
                sites[faction.0 as usize] += 1;
            }
        }
        let mut leaving: Vec<FactionId> = Vec::new();
        for number in 0..self.config.faction_count.max(1) {
            let faction = FactionId(number);
            let index = usize::from(number);
            if self.eliminated.get(index).copied() != Some(0) {
                continue;
            }
            // A faction that never founded and never spawned has not left the
            // game. It has not entered it. The seat is what says it entered,
            // and the controller keeps one for the first founding.
            if self
                .controller
                .row(faction)
                .and_then(FactionRow::seat)
                .is_none()
            {
                continue;
            }
            if sites[index] > 0 || population[index] > 0 {
                continue;
            }
            leaving.push(faction);
        }
        for faction in leaving {
            self.eliminated[faction.0 as usize] = 1;
            // The release ends the holder and the lease together. A lease
            // left behind would take the tile back on the next spread, from a
            // faction that can no longer raise it.
            let released = self.holding.release(faction, threads);
            let leftover: Vec<Entity> = self
                .characters
                .iter()
                .filter(|character| {
                    self.characters.faction_column()[character.index() as usize] == faction
                })
                .collect();
            for character in leftover {
                self.characters.remove(character);
            }
            self.eliminated_log
                .push(FactionEliminated::new(self.tick, released, faction));
        }
    }

    /// Returns the siege that stands against a site.
    ///
    /// The answer is the besieging faction and the work that faction has
    /// done. It is `None` when the identity names no live site, and `None`
    /// when no siege stands.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn siege_of(&self, site: Entity) -> Option<(FactionId, i64)> {
        self.settlements.siege(site)
    }

    /// Returns the work a siege must do before this site changes hands.
    ///
    /// The resistance is the residents the site holds, so the answer moves
    /// as the city grows and as a war empties it.[^1] Returns `None` when
    /// the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn capture_work_of(&self, site: Entity) -> Option<i64> {
        let residents = self.site_residents(site)?;
        Some(self.siege_rules.capture_work(residents))
    }

    /// Returns the work a siege must do before this site is destroyed.
    ///
    /// Returns `None` when the identity names no live site.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn raze_work_of(&self, site: Entity) -> Option<i64> {
        let residents = self.site_residents(site)?;
        Some(self.siege_rules.raze_work(residents))
    }

    /// Returns the sites that changed hands since the last step began.
    #[must_use]
    pub fn taken_log(&self) -> &[SiteTaken] {
        &self.taken_log
    }

    /// Returns the raw bytes of the taken log.
    #[must_use]
    pub fn taken_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.taken_log)
    }

    /// Returns the factions that left the game since the last step began.
    #[must_use]
    pub fn eliminated_log(&self) -> &[FactionEliminated] {
        &self.eliminated_log
    }

    /// Returns the raw bytes of the eliminated log.
    #[must_use]
    pub fn eliminated_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.eliminated_log)
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

    /// Returns the promotions of the last frame that ran the pass.
    ///
    /// The slice holds one row for each unit the pass promoted. It is empty
    /// on a frame the schedule does not name, and empty on a frame that
    /// promoted nobody. A reader cannot tell the two apart from the slice,
    /// and nothing needs to.
    #[must_use]
    pub fn promoted_log(&self) -> &[UnitPromoted] {
        &self.promoted_log
    }

    /// Returns the promotion log as bytes, for a caller across the boundary.
    ///
    /// The event is plain data with declared padding, so the bytes are the
    /// same on every run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn promoted_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.promoted_log)
    }

    /// Returns the character that a unit was promoted into.
    ///
    /// The outer option reports whether the unit is live. The inner one
    /// reports whether the unit carries a character. The answer names a
    /// living character, because a character that the arena no longer holds
    /// resolves to nothing.
    #[must_use]
    pub fn unit_character(&self, unit: Entity) -> Option<Option<Entity>> {
        let named = self.soldiers.character_of(unit)?;
        Some(named.filter(|character| self.characters.contains(*character)))
    }

    /// Returns what a unit has ever gathered, summed over every kind.
    ///
    /// Returns `None` when the identity is dead. The value never falls while
    /// the unit lives.
    #[must_use]
    pub fn unit_deeds(&self, unit: Entity) -> Option<u64> {
        self.soldiers.deeds(unit)
    }

    /// Returns the deeds at which a unit becomes eligible for promotion.
    #[must_use]
    pub fn deed_threshold(&self) -> u64 {
        self.soldiers.deed_threshold()
    }

    /// Sets the deeds at which a unit becomes eligible for promotion.
    ///
    /// The threshold is a content parameter and not a budget. A caller raises
    /// it to make a person rarer and lowers it to make one common.
    pub fn set_deed_threshold(&mut self, threshold: u64) {
        self.soldiers.set_deed_threshold(threshold);
    }

    /// Returns the most characters that one promotion pass may create.
    #[must_use]
    pub const fn promotion_budget(&self) -> u32 {
        self.promotion_budget
    }

    /// Sets the most characters that one promotion pass may create.
    ///
    /// A budget of zero promotes nobody. The arena ceiling still binds above
    /// this, so a budget larger than the headroom takes the headroom.
    pub const fn set_promotion_budget(&mut self, budget: u32) {
        self.promotion_budget = budget;
    }

    /// Returns the schedule that the promotion pass runs on.
    #[must_use]
    pub const fn character_schedule(&self) -> RateSchedule {
        self.character_schedule
    }

    /// Sets the schedule that the promotion pass runs on.
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero or above the range.
    pub fn set_character_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.character_schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
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

    /// Returns the shared table that a unit type indexes.
    ///
    /// The table holds one row for each type. A row is a set of capability
    /// columns, each a whole number or a fixed-point value, and a zero in a
    /// column means that the type cannot do what the column names.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    #[must_use]
    pub const fn unit_types(&self) -> &UnitTypeTable {
        &self.unit_types
    }

    /// Writes one row of the unit type table.
    ///
    /// The caller gives the whole row. There is no two-column form, because
    /// a caller that gave two columns would leave the rest at zero and would
    /// define a unit that fights and does nothing else without knowing
    /// it.[^1]
    ///
    /// **The values are content and not a budget.** No record holds one,
    /// because a record may hold no number that a content choice can
    /// move.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row of the table, or when a
    /// fixed-point column of the row is below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
    pub fn define_unit_type(
        &mut self,
        unit_type: u8,
        row: UnitTypeRow,
    ) -> Result<(), UnitTypeError> {
        self.unit_types.define(unit_type, row)
    }

    /// Returns the type of one unit, or `None` when the identity is dead.
    #[must_use]
    pub fn unit_type(&self, entity: Entity) -> Option<UnitTypeId> {
        self.soldiers.unit_type(entity)
    }

    /// Returns the row of the table that one unit indexes, or `None` when the
    /// identity is dead.
    ///
    /// The unit carries the index and never a copy of the row, so this is two
    /// indexed reads and it returns what the table holds now.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    #[must_use]
    pub fn unit_type_row(&self, entity: Entity) -> Option<UnitTypeRow> {
        let unit_type = self.soldiers.unit_type(entity)?;
        Some(self.unit_types.row(unit_type))
    }

    /// Sets the type of one unit, and reports whether it wrote.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent unit or skips it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_unit_type(&mut self, entity: Entity, unit_type: UnitTypeId) -> bool {
        self.soldiers.set_unit_type(entity, unit_type)
    }

    /// Returns the units that a meeting ended at the last resolution, in slot
    /// order.
    #[must_use]
    pub fn fell_log(&self) -> &[UnitFell] {
        &self.fell_log
    }

    /// Returns the fallen log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn fell_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.fell_log)
    }

    /// Returns what one faction feels toward another, or `None` when a
    /// number names no faction of this world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub fn relation(&self, from: FactionId, to: FactionId) -> Option<i32> {
        self.relations.get(from, to)
    }

    /// Returns the band number of what one faction feels toward another: how
    /// many of the edges lie at or below the value. Zero is the war band.
    #[must_use]
    pub fn relation_band(&self, from: FactionId, to: FactionId) -> Option<u8> {
        self.relations.band(from, to)
    }

    /// Reports whether either of two factions is in the war band toward the
    /// other.
    #[must_use]
    pub fn at_war(&self, a: FactionId, b: FactionId) -> bool {
        self.relations.war_between(a, b)
    }

    /// Writes what one faction feels toward another, outright.
    ///
    /// This is the caller's own path and it holds no gate. A crossing of the
    /// war edge is logged as any other cause logs it. Returns `false` when a
    /// number names no faction or the pair is one faction.
    pub fn set_relation(&mut self, from: FactionId, to: FactionId, value: i32) -> bool {
        self.relations.write(self.tick, from, to, value).is_some()
    }

    /// Returns the edges and the steps the relation reads.
    #[must_use]
    pub const fn relation_rules(&self) -> RelationRules {
        self.relations.rules()
    }

    /// Replaces the edges and the steps the relation reads.
    pub const fn set_relation_rules(&mut self, rules: RelationRules) {
        self.relations.set_rules(rules);
    }

    /// Moves what the faction of a speaker unit feels toward another faction
    /// by a bounded step.
    ///
    /// **The verb refuses a speaker whose type has a command reach of zero.**
    /// The gate reads the type column of the unit and no per-faction
    /// flag.[^1] It refuses a step above the bound in either direction, and
    /// the bound is a register row.[^2] A leader may always declare, so the
    /// verb reads no band before it moves.
    ///
    /// Returns the value after the move.
    ///
    /// # Errors
    ///
    /// Returns an error when the speaker is dead, when the other number names
    /// no faction, when the other faction is the speaker's own, when the
    /// speaker's type has no command reach, and when the step is above the
    /// bound.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    pub fn move_relation(
        &mut self,
        speaker: Entity,
        other: FactionId,
        step: i32,
    ) -> Result<i32, MoveRelationError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let faction = self.move_relation_refusal(speaker, other, step)?;
        self.relations
            .shift(self.tick, faction, other, step)
            .ok_or(MoveRelationError::Relation(RelationError::SameFaction))
    }

    /// Reports whether the relation verb would refuse one move, without
    /// moving anything.
    ///
    /// **This is the one statement of the rule.** The relation verb calls it
    /// before it shifts a row, and the legality answer calls it to fill one
    /// row of the action table.[^1] Returns the faction of the speaker, so
    /// that the verb does not read it twice.
    ///
    /// # Errors
    ///
    /// Returns the refusal the relation verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn move_relation_refusal(
        &self,
        speaker: Entity,
        other: FactionId,
        step: i32,
    ) -> Result<FactionId, MoveRelationError> {
        let (Some(faction), Some(unit_type)) = (
            self.soldiers.faction(speaker),
            self.soldiers.unit_type(speaker),
        ) else {
            return Err(MoveRelationError::DeadUnit(speaker));
        };
        if other.0 >= self.config.faction_count.max(1) {
            return Err(RelationError::NoSuchFaction(other.0).into());
        }
        if other == faction {
            return Err(RelationError::SameFaction.into());
        }
        if self.unit_types.row(unit_type).command_reach == 0 {
            return Err(RelationError::NoCommandReach.into());
        }
        let bound = self.relations.rules().move_bound;
        if step > bound || step < -bound {
            return Err(RelationError::StepAboveBound { step, bound }.into());
        }
        Ok(faction)
    }

    /// Returns the crossings of the war edge on the last step, in the order
    /// they happened.
    #[must_use]
    pub fn relation_log(&self) -> &[RelationCrossed] {
        self.relations.log()
    }

    /// Returns the relation log as bytes. The thread-count equivalence test
    /// compares this slice byte for byte.
    #[must_use]
    pub fn relation_log_bytes(&self) -> &[u8] {
        self.relations.log_bytes()
    }

    /// Returns the units that changed faction in the last step.
    ///
    /// The log covers the last step alone. It holds one entry for each unit
    /// that changed hands, whether the field converted it or the control
    /// plane did. **This is what a god reads to see conversion happen.** A
    /// mechanic that a player cannot observe is a mechanic that a player
    /// cannot play.[^1]
    ///
    /// The entries lie in ascending arena slot order, which is a stable key
    /// and never a thread completion order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0134, a god reads conversion as an event log and as the faction counts it already reads, decision D1. `docs/adrs/draft/adr-0134-a-god-reads-conversion-as-an-event-log.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn converted_log(&self) -> &[UnitConverted] {
        &self.converted_log
    }

    /// Returns the conversion log as bytes.
    ///
    /// The event type is plain data with an explicit layout, so the bytes are
    /// the events.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn converted_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.converted_log)
    }

    /// Changes the faction of every unit that the identities name.
    ///
    /// **The set is all or nothing.** Every identity resolves and the faction
    /// is checked before anything changes. A partly applied set would leave
    /// the caller with no way to say what happened.[^1]
    ///
    /// A unit that already belongs to the faction is left alone, and it emits
    /// no event. The verb is therefore idempotent over one set.
    ///
    /// The engine holds no rule that decides when a control plane may call
    /// this. It is the deliberate route, beside the field that converts a
    /// unit where another faction leads.[^2]
    ///
    /// The call clears the orders of every unit it converts, and it changes
    /// the faction of the character that a converted unit carries. The
    /// reasoning is in the record.[^3]
    ///
    /// # Errors
    ///
    /// Returns [`ConvertError::NoSuchFaction`] when the number names no
    /// faction of this world, and [`ConvertError::DeadUnit`] when an identity
    /// names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    /// [^2]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D4. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
    /// [^3]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decisions D2, D3 and D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    pub fn convert_units(
        &mut self,
        units: &[Entity],
        faction: FactionId,
    ) -> Result<(), ConvertError> {
        if faction.0 >= self.config.faction_count {
            return Err(ConvertError::NoSuchFaction(faction.0));
        }
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(ConvertError::DeadUnit(*unit));
            }
        }
        let marks = conversion::marks_for_set(&self.soldiers, units, faction);
        self.apply_converts(&marks);
        Ok(())
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

    /// Returns the faction that holds a settlement.
    ///
    /// Returns `None` when the identity names no live settlement. A site
    /// changes hands, so this answer is not fixed at the founding and a
    /// caller must read it again.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D1. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn settlement_faction(&self, entity: Entity) -> Option<FactionId> {
        self.settlements.faction(entity)
    }

    /// Returns the quantity of one commodity in the store of a settlement.
    ///
    /// Returns `None` when the identity is dead, and `None` when the
    /// commodity is outside the set.
    #[must_use]
    pub fn settlement_store(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        self.settlements
            .store(entity)
            .and_then(|store| store.quantity(commodity))
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

    /// Resolves the value of an identity back to the character it names.
    ///
    /// A caller outside this crate holds an identity as the value the engine
    /// gave. It cannot build one, and this is the only way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds for the slot. It refuses a mismatch, and it
    /// never returns the character who now holds the slot.[^2]
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
    pub fn resolve_character(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.characters.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.characters.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.characters.generation_of(slot),
        })
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

    /// Returns the water crossing column of the type of one unit.
    ///
    /// **This is the one place that reads the column for a unit.** A unit
    /// whose type the arena cannot answer for crosses nothing, which is the
    /// answer every type gave before the column existed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn crossing_of(&self, unit: Entity) -> u32 {
        self.soldiers
            .unit_type(unit)
            .map_or(0, |unit_type| self.unit_types.row(unit_type).water_crossing)
    }

    /// Reports whether the ground at an address admits one named unit.
    ///
    /// The ground states a capacity for the crossing the type of the unit
    /// holds, and a capacity above zero admits it. A type with no crossing
    /// therefore gets the answer the ground alone gives, and a type that
    /// crosses open water may stand on it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn admits_this_unit(&self, unit: Entity, address: Axial) -> bool {
        self.terrain
            .kind(address)
            .is_some_and(|kind| kind.is_passable_for(self.crossing_of(unit)))
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

    /// Returns the number of live soldiers of one faction.
    ///
    /// **This is one read, not a pass over the population.** The arena
    /// maintains the count where a slot becomes live and where it stops being
    /// live, so a caller that asks how many people a faction has left never
    /// reads a unit.[^1]
    ///
    /// A faction whose last unit ends reads zero here, and nothing else in
    /// the engine says so.
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn population_of(&self, faction: FactionId) -> u32 {
        self.soldiers.population_of(faction)
    }

    /// Returns the live soldier count of every faction, by faction number.
    #[must_use]
    pub const fn population_by_faction(&self) -> &[u32; FACTION_CEILING as usize] {
        self.soldiers.population_by_faction()
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

    /// Returns how many factions this world holds.
    ///
    /// A faction identifier below this number names a faction. The ceiling is
    /// a property of the mask that holds a relation between factions.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn faction_count(&self) -> u16 {
        self.config.faction_count
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

    /// Seeds the luxuries of the world.
    ///
    /// Each placement names a tile and a luxury. The caller gives the whole
    /// set in one call, so the control plane crosses the boundary once and
    /// never loops over tiles.[^1] The caller states the placements in any
    /// order, and the engine sorts them.
    ///
    /// **The world takes a seed once.** The field is not a fact of a frame,
    /// and a reader of it never has to ask which frame it read. A second call
    /// is refused, whether or not the first one placed anything.
    ///
    /// The call derives level 1 from the field, so nothing derives it again
    /// on a later frame.
    ///
    /// # Errors
    ///
    /// Returns [`LuxuryError::AlreadySeeded`] when the world already took a
    /// seed. Returns [`LuxuryError::IdAboveCeiling`] when a placement names a
    /// luxury above the catalogue, and [`LuxuryError::NoSuchTile`] when a
    /// placement names a tile outside the world. A refusal changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    pub fn seed_luxuries(&mut self, placements: &[(TileIdx, LuxuryId)]) -> Result<(), LuxuryError> {
        if self.luxuries_seeded {
            return Err(LuxuryError::AlreadySeeded);
        }
        let field = LuxuryField::seed(self.grid, placements)?;
        self.variety = VarietyLevel::derive(self.bridge.layout(), &field);
        self.luxuries = field;
        self.luxuries_seeded = true;
        Ok(())
    }

    /// Reports whether the world has taken a luxury seed.
    #[must_use]
    pub const fn luxuries_seeded(&self) -> bool {
        self.luxuries_seeded
    }

    /// Returns the luxuries of the world.
    #[must_use]
    pub const fn luxuries(&self) -> &LuxuryField {
        &self.luxuries
    }

    /// Returns the luxuries of every level 1 cell.
    #[must_use]
    pub const fn variety_level(&self) -> &VarietyLevel {
        &self.variety
    }

    /// Returns the luxuries that one tile carries.
    ///
    /// A tile that carries none gives the empty set. A tile outside the world
    /// gives the empty set as well, because it carries nothing.
    #[must_use]
    pub fn luxuries_at(&self, tile: TileIdx) -> LuxurySet {
        self.luxuries.at(tile)
    }

    /// Returns the variety of the whole world.
    ///
    /// The variety is the number of different luxuries that stand anywhere in
    /// the world. **Nothing in the engine reads this. It is a score for the
    /// control plane, and no pass consumes it.**[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[must_use]
    pub fn world_variety(&self) -> u32 {
        self.luxuries.set().variety()
    }

    /// Returns the variety of the ground that one faction holds.
    ///
    /// The answer is the number of different luxuries on the tiles of that
    /// faction. The fold runs over the luxury entries in ascending tile
    /// order, and it asks the holder column for each one, so its cost follows
    /// the number of placements and not the size of the world.[^1]
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no pass consumes it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[must_use]
    pub fn faction_variety(&self, faction: FactionId) -> u32 {
        let mut total = LuxurySet::EMPTY;
        for row in self.luxuries.tiles() {
            let Some(address) = self.grid.address_of(row.tile) else {
                continue;
            };
            let Some(holder) = self.holding.holder(address) else {
                continue;
            };
            if holder.faction() == Some(faction) {
                total = total.union(row.set);
            }
        }
        total.variety()
    }

    /// Returns the hash of the whole state.
    ///
    /// The golden test compares this value against a stored file.[^1]
    ///
    /// **Every stored value that a pass or a verb reads enters this hash.** A
    /// derived projection stays out, and the inputs of the derivation enter
    /// instead. A parameter that decides what a pass writes enters, even when
    /// the column the pass writes is already here: a hash of the effect
    /// reports a changed parameter one or more ticks after the change.[^2]
    ///
    /// A new value needs a test that changes it through the public interface
    /// and asserts that the hash differs. The golden file cannot do that work,
    /// because it cannot say which input the hash stopped depending on.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 to D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^3]: ADR-0164, every stored value the step reads enters the state hash, decision D4. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
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
        // The ledger writes the recovery rules with its entries. The pass
        // reads a period on every tick, and a hash that wrote the takes and
        // not the periods reports the effect one or more ticks after the
        // cause.[^20]
        //
        // [^20]: Findings register, FND-480. `docs/FINDINGS.md`
        let mut hash = self.depletion.hash_into(hash);
        for amount in &self.departed {
            hash = hash.write_u64(*amount);
        }
        // What has been delivered is a stored total that a later frame reads,
        // in the way the departed total is. It is the term that links the
        // carry account to the store account.
        for amount in &self.delivered {
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
        // The remembered layer of each faction is state that a later frame
        // reads, and the sight rules decide what the next rebuild writes, so
        // both enter. The layer of what a faction sees now is derived from
        // the units and the rules, so it stays out and its inputs enter
        // instead.[^21]
        //
        // [^21]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 to D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
        let hash = self.observation.hash_into(hash);
        // The unit type table decides what the next meeting does, so the
        // whole-world hash covers it. Two worlds that hold the same units and
        // different tables must diverge at the next meeting.
        let hash = self.unit_types.hash_into(hash);
        // The queue of every site is state that a later frame reads, and the
        // cost table decides what a later frame does, so both enter. The
        // schedule decides which ticks act, so it enters too.[^17]
        //
        // [^17]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        let hash = self.queues.hash_into(hash);
        let hash = self.build_costs.hash_into(hash);
        let hash = hash
            .write_u64(u64::from(self.queue_schedule.period()))
            .write_u64(u64::from(self.queue_schedule.phase()));
        // The growth parameters decide what a later frame does, so the
        // whole-world hash covers them. The housing of each site is a column
        // of the settlement arena, and that arena hashes its own columns.[^18]
        //
        // [^18]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D6. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        let hash = hash
            .write_u64(u64::from(self.growth_schedule.period()))
            .write_u64(u64::from(self.growth_schedule.phase()))
            .write_u64(u64::from(self.housing_per_person))
            .write_u64(u64::from(self.founding_housing))
            .write(&self.birth_chance.0.to_le_bytes());
        let hash = self
            .food_per_birth
            .iter()
            .fold(hash, |hash, cost| hash.write(&cost.0.to_le_bytes()));
        // The upgrade table decides what a build order does and what an
        // upgrade changes, so the whole-world hash covers it. Two worlds
        // built with different tables never hash the same.[^5]
        //
        // [^5]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        let hash = self.upgrade_table.hash_into(hash);
        // A luxury is authored rather than generated, so no input above
        // produces it. Two worlds that carry different luxuries are
        // different worlds, and only the field says so.[^4]
        //
        // [^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.luxuries.hash_into(hash);
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
        // What growth took is a term of the world conservation statement, in
        // the way the rate ledger is, so the whole-world hash covers it.
        let hash = self
            .growth_ledger
            .iter()
            .fold(hash, |hash, total| hash.write(&total.0.to_le_bytes()));
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
        let mut hash = self.influence.hash_into(hash);
        // The seed set of each destination is state that a later frame reads.
        // The field itself is derived from it and from level 1, and a derived
        // value states no fact of its own, so the seeds enter the hash and the
        // field does not. The send column of each unit is already in the
        // arena hash above.[^4]
        //
        // [^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        for tiles in &self.destination_seeds {
            hash = hash.write_u64(tiles.len() as u64);
            for tile in tiles {
                hash = hash.write(&tile.0.to_le_bytes());
            }
        }
        // Whether a plane conducts through water is stored beside its seeds,
        // and the derivation reads it, so it enters the hash for the reason
        // the seeds do.
        for crossing in &self.destination_crossings {
            hash = hash.write(&crossing.to_le_bytes());
        }
        // A position is state that a later frame reads: a unit that holds
        // one still holds it on the next frame, and the preference decides
        // what the next rebalance opens. Two worlds that hold the same
        // stores and different preferences must diverge.
        let hash = self
            .positions
            .hash_into(hash)
            .write_u64(u64::from(self.position_schedule.period()))
            .write_u64(u64::from(self.position_schedule.phase()))
            // The promotion schedule decides which frames promote, so two
            // worlds that differ in it diverge and the hash must say so. The
            // deed threshold and the columns it governs are folded by the
            // unit arena, which holds them.
            .write_u64(u64::from(self.character_schedule.period()))
            .write_u64(u64::from(self.character_schedule.phase()))
            .write_u64(u64::from(self.promotion_budget));
        // The four parameters of the choice are read on every tick, and each
        // one decides what a unit does next. The intent column carries the
        // outcome, and an outcome states the effect and never the cause: two
        // worlds that hold the same intents and different parameters hash the
        // same and diverge at the next frame that chooses.[^21]
        //
        // [^21]: Findings register, FND-537. `docs/FINDINGS.md`
        let hash = self
            .choice
            .hash_into(hash)
            .write_u64(u64::from(self.carry_mark.0));
        let hash = self.buckets.hash_into(hash);
        let hash = self.weights.hash_into(hash);
        // The bound on a land side is a parameter that a verb reads. Two
        // worlds that differ in it answer the same offer differently.
        let hash = hash.write_u64(u64::from(self.land_list_bound));
        // Whether the world took a luxury seed is a fact the field cannot
        // state: a seed that placed nothing leaves the field as an unseeded
        // world leaves it, and the two worlds refuse and accept the next seed
        // differently.[^22]
        //
        // [^22]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        let hash = hash.write_u64(u64::from(self.luxuries_seeded));
        let mut hash = self.draw_ledger.hash_into(hash);
        for total in &self.store_account {
            hash = hash.write_u64(total.0 as u64);
        }
        // Whether a faction has left the game is stored state that the step
        // reads. The elimination pass reads it to leave a faction that has
        // gone alone, and every game end reader reads it to refuse a faction
        // that is out, so it enters the hash.[^24]
        //
        // [^24]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
        let hash = hash.write(&self.eliminated);
        // What two factions agreed is state that a later frame reads: the
        // settlement pass moves a quantity because a contract says so. The
        // plane holds no row until somebody speaks, and it then folds nothing,
        // so a world that never traded hashes as it did before trade
        // existed.[^16]
        //
        // [^16]: ADR-0126, a trade negotiation is engine state and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
        let hash = self.trade.hash_into(hash);
        // The controller rows, its two parameters and the game end record are
        // each read by a later frame. The evaluation count and the tick limit
        // enter for the reason the recovery rules do: a value the step reads
        // on every tick and that the hash does not cover lets two worlds hash
        // the same and diverge on the next tick.[^18]
        //
        // [^18]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
        let hash = self.controller.hash_into(hash);
        // The win-path balance values are read by a later frame: a reader
        // compares one on every tick, and the renown pass adds another. Two
        // worlds that differ only in a balance value must diverge, and the
        // hash must say so.[^20]
        //
        // [^20]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D1. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
        let hash = self.balance.hash_into(hash);
        // What each faction marches on is state that a later frame reads: the
        // stage closes a campaign against the holder it recorded at the
        // raise, and the raise refuses while one is live.
        let hash = self.campaigns.hash_into(hash);
        // The plan of each faction decides where a unit may build, and the
        // step reads it, so two worlds that differ only in a plan must
        // diverge and the hash must say so.[^7]
        //
        // [^7]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        let hash = self.plan.hash_into(hash);
        // What each faction advertises is state that a controller reads. The
        // table holds no row until somebody advertises, and it then folds
        // nothing, so a world with no board hashes as it did before.
        let hash = self.market.hash_into(hash);
        // What each faction feels toward each other is state that a later
        // frame reads: the next contest fires or not because of it. The edges
        // and the steps enter with the entries, for the reason the controller
        // parameters do.[^19]
        //
        // [^19]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        let hash = self.relations.hash_into(hash);
        // The water in the air and on the ground is state that a later frame
        // reads: the next solve starts from the field this one left. Two
        // worlds that hold the same tiles and different weather must
        // diverge.[^17]
        //
        // [^17]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        let hash = self.weather.hash_into(hash);
        // The climate is stored, and the terrain readers of this world read
        // it, so two worlds that hold the same seed and different climates
        // must diverge.[^17]
        self.climate.hash_into(hash)
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
        // the world, and banks no progress beyond the work that the row
        // above each entry asks for.[^2] The table holds a row for every
        // level an entry stands at.
        //
        // [^2]: Findings register, FND-011. `docs/FINDINGS.md`
        if !self
            .upgrades
            .check_invariants(self.grid.tile_count(), &self.upgrade_table)
        {
            return false;
        }
        if !self.upgrade_table.check_invariants() {
            return false;
        }
        if !self.plan.check_invariants() {
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

        // No soldier stands on ground that admits no unit of its own type.
        // The spawn, the placement and the movement each refuse such a tile,
        // and this check is what fails when a later path forgets to.[^1]
        //
        // **The gate is the capacity table, and the crossing column of the
        // unit is an argument to it.** The movement pass admits a step by
        // that table, so a check that read the ground alone would state a
        // second, stricter rule and fail on a mariner that crossed open
        // water exactly as its row permits.[^11]
        //
        // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        // [^11]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        if self.soldiers.iter().any(|soldier| {
            self.soldiers
                .address(soldier)
                .is_some_and(|address| !self.admits_this_unit(soldier, address))
        }) {
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
        // A unit that names a character must name one the arena still holds.
        // A link to a removed character is the stale identity that the
        // generation exists to catch.[^1]
        //
        // [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
        for (slot, bits) in self.soldiers.character_column().iter().enumerate() {
            let Some(character) = Entity::from_bits(*bits) else {
                continue;
            };
            if self.soldiers.live_column()[slot] != 1 {
                return false;
            }
            if !self.characters.contains(character) {
                return false;
            }
        }
        if !self.characters.check_invariants() {
            return false;
        }
        // The plane is either empty or one row for each ordered pair, and
        // no party ever delivered more than it owed.
        if !self.trade.check_invariants() {
            return false;
        }
        if self.trade.factions() != self.config.faction_count {
            return false;
        }
        // The board is either empty or one block for each faction.
        if !self.market.check_invariants() {
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
        if !self.check_contest() {
            return false;
        }
        // Every drop that entered the air is in the air, on the ground, or
        // counted as evaporated. A pass that scaled the water rather than
        // moving it would break this, and nothing else reports it.[^9]
        //
        // [^9]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
        if !self.weather.check_account() {
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
        // The luxury entries rise, they name each tile once, and no entry is
        // empty. A field that broke any of those answers a lookup with the
        // wrong tile, and nothing else notices.
        if !self.luxuries.check_invariants(self.grid.tile_count()) {
            return false;
        }
        // Level 1 of the variety states the same fact a second time, and it
        // states it over the cells. A check must fail when the two copies
        // disagree, because a derived level that drifts reads back correctly
        // and answers the wrong question.[^5]
        //
        // The check compares the union of every cell against the union of
        // every tile. It is a fold over the cells and a fold over the
        // entries, so it visits no tile that carries nothing.
        //
        // [^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.variety.total() != self.luxuries.set() {
            return false;
        }
        if self.variety.deposit_total() != self.luxuries.deposits() {
            return false;
        }
        if self.variety.layout() != self.bridge.layout() {
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

    /// Reports whether the unit type table and the fallen log hold their
    /// invariants.
    ///
    /// The table holds no negative value, because an attack is a quantity of
    /// harm and an armour is a threshold. The log holds declared padding, an
    /// identity that packs, a tile inside the world, and a type the table
    /// holds. A fallen unit is dead by the time anyone reads the log, so the
    /// check states what the event itself must hold.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    fn check_contest(&self) -> bool {
        if !self.queues.check_invariants(self.settlements.slot_count()) {
            return false;
        }
        if !self.unit_types.check_invariants() {
            return false;
        }
        let tiles = self.grid.tile_count();
        if !self.fell_log.iter().all(|event| {
            event.padding == [0; 1]
                && event.unit != 0
                && event.tile.0 < tiles
                && event.faction.0 < FACTION_CEILING
                && event.unit_type.index() < crate::unit_type::UNIT_TYPE_COUNT
        }) {
            return false;
        }
        // A conversion event names two different factions of this world, and
        // a live tile. An event that named one faction twice would report a
        // change that did not happen.
        self.converted_log.iter().all(|event| {
            event.unit != 0
                && event.tile.0 < tiles
                && event.from.0 < FACTION_CEILING
                && event.to.0 < FACTION_CEILING
                && event.from != event.to
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
            // A delivered quantity has left the carries and reached a store,
            // so it is neither held nor departed. It is the term that links
            // this check to the store check.
            let delivered = self.delivered[index] as i64;
            if left_the_tiles[index] + returned
                != arrived[index] + self.departed[index] as i64 + delivered
            {
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
        let mark = self.carry_mark;
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
                    let carries = soldiers.carry_column();
                    let homes = soldiers.home_column();
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
                            let slot = unit.index() as usize;
                            let need = needs[slot];
                            // The carry class is the third term of the key. It
                            // is a bounded class of the state of the unit
                            // itself, and it is not the load.[^17]
                            //
                            // [^17]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
                            let carry = carry_class_of(carries[slot], homes[slot], mark);
                            chosen.push((*unit, answers.answer(need, carry, weights)));
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

        self.trade_log.clear();
        // **These three logs are cleared here, before any system runs.** The
        // rule for every log in this engine is the same: a log holds what
        // happened since the last step began, and a reader that misses a step
        // misses the events. A settlement founded by a caller between two
        // steps therefore survives to the next read.
        self.collapsed_log.clear();
        self.finished_log.clear();
        self.founded_log.clear();
        self.taken_log.clear();
        self.eliminated_log.clear();
        self.fold_relations_into_the_census();
        self.relations.clear_log();
        self.tick = Tick(self.tick.0.wrapping_add(1));

        let tick = self.tick;
        let seed = self.config.seed;
        let count = self.grid.tile_count();
        let chunk_len = (count as usize).div_ceil(threads).max(1) as u32;

        let mut slots: Slots<ChunkResult> =
            Slots::filled(threads, ChunkResult::default()).map_err(|_| StepError::ZeroThreads)?;

        // **Each worker writes its own contiguous range of the field.** The
        // field hands out disjoint chunks, so no two workers touch one tile
        // and no worker needs an atomic. The requirement on a parallel stage
        // is met by the type and not by a rule a reviewer has to check.[^12]
        //
        // The field used to be read here and written afterwards, by a merge
        // that took the joined changes, sorted them and rebuilt the stored
        // list. That merge is gone. It cost a share of the frame that grew
        // with the tiles that had ever changed, and the dense array it now
        // writes needs no run, no sort and no join.[^14] [^15]
        //
        // [^12]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
        // [^14]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, the consequences. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
        // [^15]: Findings register, FND-292. `docs/FINDINGS.md`
        {
            let _span = stage::open(Stage::TileScan);
            // The array is allocated here, on the first frame that runs, and
            // never when the world is built.
            self.values.prepare();
            std::thread::scope(|scope| {
                for (slot, chunk) in slots
                    .entries_mut()
                    .iter_mut()
                    .zip(self.values.chunks_mut(chunk_len))
                {
                    scope.spawn(move || {
                        *slot = update_range(tick, seed, chunk);
                    });
                }
            });
        }

        // The count of changed tiles is a sum over the chunks. Addition of
        // integers does not depend on the order of the terms, so the count is
        // the same at any thread count.[^13]
        //
        // [^13]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        self.values
            .absorb_changed(slots.combine(0i64, |total, slot| total + slot.changed));

        {
            let _span = stage::open(Stage::LogJoin);
            let mut log = core::mem::take(&mut self.log);
            log.clear();
            self.log = slots.combine(log, |mut joined, slot| {
                joined.extend_from_slice(&slot.events);
                joined
            });
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

        // The walk order of the movement pass. The bridge holds every live
        // unit in block-major tile order, and it sorted them at the barrier,
        // so this order costs the frame nothing more than it already paid.
        let live = self.bridge.units(&self.soldiers)?.to_vec();
        let intents = {
            let _span = stage::open(Stage::MovementIntents);
            soldier_moves(
                tick,
                seed,
                self.terrain,
                &UnitWalk {
                    soldiers: &self.soldiers,
                    live: &live,
                },
                &Steering {
                    layout: self.pyramid.layout(),
                    exits: &self.exits,
                    approaches: &self.approaches,
                    home_approaches: &self.home_approaches,
                    stock_approaches: &self.stock_approaches,
                    site_tiles: self.settlements.tile_column(),
                    returns: &self.returns,
                    destinations: &self.destinations,
                },
                &Building {
                    holding: &self.holding,
                    plan: &self.plan,
                    unit_types: &self.unit_types,
                    upgrades: &self.upgrades,
                    table: &self.upgrade_table,
                },
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
                &self.upgrade_table,
                self.grid,
                Guests {
                    holders: self.holding.holders(),
                    relations: &self.relations,
                },
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
            // **The release runs here, in the stage that applies the step,
            // and it reads the tile the unit now stands on.** A unit that
            // reached the tile it was sent to is free from this line on, so
            // it reads its option row again and it gathers and delivers.
            // Nothing released a sent unit before, and the defect was
            // invisible for as long as arrival was impossible.[^31]
            //
            // The release takes no frame from the unit. The choice pass
            // already wrote an intent for it, and the gather and the delivery
            // of this frame run below and read no send, so a unit released
            // here acts on the frame it is released.
            //
            // [^31]: Findings register, FND-572. `docs/FINDINGS.md`
            self.release_sent_units();
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
            // The ledger comes out of the world for the pass, so that the
            // pass can read the weather and the upgrade map beside it. It
            // goes back in the same block, and nothing between the two lines
            // reads it.
            let mut depletion = core::mem::take(&mut self.depletion);
            depletion.recover(tick, &|tile| self.tile_ground(tile));
            self.depletion = depletion;
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

        // The wear runs after the build of this frame, so a worker mends its
        // site and the weather then takes from what the worker left. The
        // other order would let a site collapse in the tick a worker filled
        // it, and the worker would have bought nothing.
        //
        // The pass reads the bridge, which the barrier above rebuilt, so it
        // reads where each unit stands after the movement of this frame. It
        // writes the upgrade map and moves no unit, so the barrier above
        // stays the barrier of this frame.
        //
        // The weather solve runs later in this step, so the pass reads the
        // field the previous step left. That is a fixed order and not a stale
        // read: every tick reads the field of the tick before it.[^17]
        //
        // [^17]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
        {
            let _span = stage::open(Stage::UpgradeWear);
            self.wear_upgrades();
        }

        // The cities rewrite the holder column here, after the barrier of
        // this frame and after the build above. The reach of a city counts
        // the finished upgrades on the ground it held at the end of the
        // previous step, so the count reads the column this stage is about to
        // overwrite.[^7]
        //
        // The pass reads no unit position, so a unit gives its faction no
        // claim on the ground it stands on. It writes a tile column and moves
        // no unit, so the barrier above stays the barrier of this frame.
        //
        // [^7]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1, D2 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        // **The lease moves before the holder is decided.** A tile that
        // carries a unit moves its lease one step toward the faction present,
        // and the decay then pulls every live lease toward zero on a fixed
        // schedule. The rewrite below reads the lease this pass wrote.[^21]
        //
        // The pass reads the bridge, which the barrier above rebuilt, so it
        // reads where each unit stands after the movement of this frame. It
        // writes two tile columns and moves no unit, so the barrier above
        // stays the barrier of this frame.
        //
        // [^21]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2, D3, D4 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
        // The occupancy outlives this block, because the capture pass below
        // reads the same list. Who stands on a tile is stated once, and the
        // lease and the capture both read that one statement.[^22]
        let occupancy = {
            let _span = stage::open(Stage::HoldingLease);
            let occupancy = self.tile_occupancy(threads)?;
            self.holding.advance_leases(&occupancy, tick.0);
            occupancy
        };

        // **A site changes hands here, between the lease and the spread.**
        // The occupancy the lease pass built is the one statement of who
        // stands on a tile, and this pass reads the same list rather than
        // counting again.[^22] The spread below then reads the settlement
        // faction column that this pass has just written, so a city taken on
        // this tick reaches its new ground on this tick and not the next.
        //
        // The pass is serial and it walks the settlements in slot order,
        // which is a stable key.[^23]
        //
        // [^22]: ADR-0180, a site changes hands or the taker destroys it, decision D4. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
        // [^23]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        {
            let _span = stage::open(Stage::SiteCapture);
            self.capture_sites(&occupancy);
        }

        {
            let _span = stage::open(Stage::HoldingSpread);
            self.holding
                .rewrite(self.terrain, &self.settlements, &self.upgrades, threads)?;
        }

        // **A faction leaves the game here, after the spread.** The spread
        // above has just written the holder column, so the ground this pass
        // releases is the ground the faction holds now and not the ground it
        // held at the last barrier.[^24]
        //
        // The pass runs once for each tick. An elimination that releases
        // ground gives that ground to a rival city through the spread of the
        // next tick, by the rule that already decides every holder. Nothing
        // loops here, and no elimination cascades inside one tick.[^24]
        //
        // [^24]: ADR-0181, a faction that holds no site and no unit leaves the game, decisions D2, D3 and D4. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
        {
            let _span = stage::open(Stage::FactionEliminate);
            self.eliminate_factions(threads);
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
        // The delivery runs after the gather resolve and before the rate
        // pass. It reads where each unit stands, so it runs after the barrier
        // that the movement of this frame passed. It moves a quantity, and
        // two records say that the rate pass and the consumption pass run
        // after every stage that moves one.[^16] [^17] It changes no
        // structure, so it is not a barrier and it needs none.
        //
        // [^16]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^17]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        self.deliver(threads)?;

        // The meeting resolves here, after the barrier of this frame and
        // after the holding spread. It reads where each unit stands, and the
        // movement above has just moved them, so a resolution before the
        // barrier would fight on the tile a unit left.[^19]
        //
        // **It resolves at the tile, and never at a level 1 cell.** A cell
        // summarises a whole block of tiles, and a fight resolved there kills
        // units spread over all of them.[^19]
        //
        // It removes units, so it is a structural change. Nothing between
        // here and the refresh below reads the derived unit structure, and
        // that refresh is the barrier of the change.[^20]
        //
        // [^19]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
        // [^20]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::Contest);
            self.contest(threads)?;
        }
        // The contract settlement runs directly after the ordinary delivery,
        // for the same reason that one runs where it does. It reads where each
        // unit stands, so it runs after the barrier that the movement of this
        // frame passed. It moves a quantity, so it runs before the rate pass
        // and before the consumption pass.[^18] [^19] It changes no structure,
        // so it is not a barrier and it needs none.
        //
        // [^18]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^19]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D3. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
        self.settle_trades(threads)?;

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

        // The queue advance runs after the shortage scan and before the
        // barrier below it. It reads the store, so it runs after the rate
        // pass and after the consumption pass, which are what move a
        // quantity in this frame.[^25] It removes a resident and adds a
        // typed unit, so it is a structural change, and the refresh below is
        // the barrier of that change.[^26]
        //
        // **It runs after the scan and not before it.** The scan holds a
        // plane of the slots it ends. A stage that freed a slot and filled it
        // again before the scan applied would give the scan a live unit that
        // it never marked.
        //
        // The stage takes no thread count. It visits the sites and their
        // entries, and it walks the units once on a tick where an entry
        // finishes.[^27]
        //
        // [^25]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
        // [^26]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        // [^27]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D5. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        // Growth runs after the pass that ends a starved unit, in the same
        // frame. A place that a death freed this frame is free this frame,
        // and the occupancy that the admission reads is the settled one.[^28]
        // The reap above applied its plane already, so no slot this stage
        // fills can carry a death that the scan had not yet marked.
        //
        // It runs before the queue advance, because growth is the only source
        // of people and the queue is the only consumer of them.[^29] A queue
        // that spent a person before growth added one would decide against
        // the population of the frame before.
        //
        // The stage takes no thread count. It visits the sites, and it walks
        // no unit.[^30]
        //
        // [^28]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
        // [^29]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        // [^30]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        {
            let _span = stage::open(Stage::Grow);
            self.grow();
        }
        {
            let _span = stage::open(Stage::QueueAdvance);
            self.advance_queues();
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

        // The promotion scan runs after the deaths of this frame, so it never
        // promotes a unit that the shortage ended in the same frame. It reads
        // no derived structure and changes none, so it needs no barrier of
        // its own.[^18]
        //
        // [^18]: ADR-0104, a soldier is promoted from a level that never falls, decision D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
        {
            let _span = stage::open(Stage::Promote);
            self.promote(threads)?;
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

        // The presence relation is derived last, after every structural
        // change this frame made and after the holding spread that decides
        // who holds each tile. A fold before the reap would name a unit the
        // frame ended, and a fold before the spread would answer against the
        // holders of the previous frame.[^19]
        //
        // The relation is derived and never stored, so it reaches no state
        // hash and no event.[^19]
        //
        // [^19]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        // The weather solve runs after level 1 rebuilds, because it reads
        // the height and the water share of each cell from the summaries
        // that rebuild produced. It runs the same fixed number of spread
        // passes whatever the field holds, and it takes no branch on whether
        // a storm exists.[^21]
        //
        // The gather resolve above reads the ground of the cell as the
        // previous frame left it, in the way movement reads the exit field of
        // the previous barrier. A solve placed before the gather would answer
        // from a level 1 that this frame had not yet rebuilt.[^22]
        //
        // [^21]: ADR-0141, a weather pass moves water and never scales it, decision D3. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
        // [^22]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
        {
            let _span = stage::open(Stage::WeatherSolve);
            self.weather
                .solve(tick, seed, &self.weather_ground, threads)?;
        }

        // Conversion runs after the influence solve, because it reads the
        // field that solve produced, and before the presence fold, because
        // the fold reads the faction of every unit. A conversion after the
        // fold would leave the relation answering for the factions of the
        // frame before, and the freshness check would pass, because the fold
        // records the arena revision it read.[^20] [^21]
        //
        // It changes no unit structurally, so no barrier stands between it
        // and the fold. It does raise the arena revision, because the
        // relation below is derived from the faction column.[^22]
        //
        // [^20]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
        // [^21]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D5. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
        // [^22]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
        {
            let _span = stage::open(Stage::Convert);
            self.convert(threads)?;
        }
        {
            let _span = stage::open(Stage::PresenceFold);
            self.presence
                .rebuild(&self.soldiers, &self.holding, threads)?;
        }
        // The observation pass runs after every change to a unit position
        // and after every change to a unit faction, because it reads both.
        // A pass placed before the reap would give sight to a unit the frame
        // ended, and a pass placed before the conversion would give the
        // tiles of a converted unit to the faction it left.[^26]
        //
        // The layer of what a faction sees now is derived here. The layer of
        // what a faction has ever seen is state, and this pass is the only
        // writer of it.[^26]
        //
        // [^26]: ADR-0059, fog storage grows with observed area, not with world area, decisions D1 and D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
        {
            let _span = stage::open(Stage::Observe);
            self.observation
                .rebuild(&self.soldiers, self.terrain, threads)?;
        }
        // The drift runs after every cause of this frame has written the
        // relation and before the controller reads it, so the controller
        // plans against the relation this frame settled on. It takes no
        // thread count: the matrix follows the square of the faction ceiling
        // and no term follows the population.[^25]
        //
        // [^25]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        {
            let _span = stage::open(Stage::RelationDrift);
            self.relations.drift(tick);
        }
        // The controller runs last, after every derived structure of the
        // frame describes the frame. It takes no thread count. It checks the
        // game end readers first, and it emits nothing once the record is
        // written.[^23] [^24]
        //
        // [^23]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
        // [^24]: ADR-0148, a game end is recorded once and stops the controllers, decision D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
        {
            let _span = stage::open(Stage::Controller);
            self.run_controller();
        }
        // **A step leaves the world readable.** The controller founds cities,
        // and a founding seats a group and spends the settler. Both change
        // the unit arena, so the refresh above no longer describes it. The
        // derived unit structure counts the changes of the arena and refuses
        // every answer once the counts differ, so a reader between two steps
        // met a refusal on each tick a faction founded.[^25]
        //
        // The opening refresh of the next step would repair it, so the
        // simulation carried on and only a reader saw the fault. A reader is
        // not obliged to check, and the drawing is not the only one, so the
        // step repairs it here instead.
        //
        // **The refresh costs nothing on a tick that founded nothing.** It
        // compares the two counts and returns, and it rebuilds only when the
        // controller changed the arena. A tick that changed the arena pays
        // for one rebuild, and no tick pays for two.
        //
        // [^25]: ADR-0018, the unit to tile bridge is derived and rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        {
            let _span = stage::open(Stage::BridgeRefreshClosing);
            self.refresh_bridge()?;
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
    /// Returns what one gather takes in addition, because the ground is wet.
    ///
    /// The bonus is a whole number of resource units and it is never a
    /// fraction of the ordinary rate. A rate multiplied by a fraction would
    /// be a second scale beside the one the resource ledger counts in.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    fn wet_bonus(&self, tile: TileIdx) -> u32 {
        // The weather lattice has a pitch of its own, so the cell of a tile
        // comes from the weather reader and not from the level 1 reader.
        match self.weather_cell_of(tile) {
            Some(cell) if self.weather.cell_is_wet(cell) => WET_GATHER_BONUS,
            _ => 0,
        }
    }

    /// Returns the ticks that one deposit here takes to regain one unit.
    ///
    /// **This is the answer the recovery pass acts on.** The reader calls the
    /// same rule the pass calls, over the same ground, so a caller that shows
    /// the number and a pass that uses it cannot disagree.[^1]
    ///
    /// The answer moves with the weather and with what stands on the tile. A
    /// caller that asks twice at different ticks gets two answers, and both
    /// are correct at the tick they were asked at.
    ///
    /// Returns `None` when the address lies outside the world, and when the
    /// kind does not recover at all.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn recovery_period_at(&self, address: Axial, kind: ResourceKind) -> Option<u32> {
        let tile = self.grid.index_of(address)?;
        self.depletion
            .recovery()
            .period_for(kind, self.tile_ground(tile))
    }

    /// Returns the weather field of the world.
    ///
    /// The field is a plane over the level 1 cell lattice. It holds the water
    /// in the air above each cell and the water on the ground of each
    /// cell.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    pub const fn weather(&self) -> &WeatherField {
        &self.weather
    }

    /// Returns the water in the air above the cell that covers one tile.
    ///
    /// The unit is drops, and a drop is a whole number. Returns `None` when
    /// the address lies outside the world.
    #[must_use]
    pub fn air_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.air_at(self.weather_cell_of(tile)?).0)
    }

    /// Returns the share of the sky over one tile that a watcher sees as
    /// cloud.
    ///
    /// **Cloud is the air held against what the air of that cell can hold**,
    /// and not the air held against a mark that every cell shares. The
    /// published saturation curve puts the capacity of a tropical cell about
    /// thirty times above the capacity of a polar one, so a ramp against one
    /// mark paints a cold sky black. Every reader that paints cloud takes
    /// this one, so the rule has one declaration site.[^1]
    ///
    /// The share runs from none to a whole sky. Returns `None` when the
    /// address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn cloud_share_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.cloud_share_at(self.weather_cell_of(tile)?))
    }

    /// Returns the water on the ground of the cell that covers one tile.
    ///
    /// The unit is drops, and a drop is a whole number. Returns `None` when
    /// the address lies outside the world.
    #[must_use]
    pub fn ground_water_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.ground_at(self.weather_cell_of(tile)?).0)
    }

    /// Returns the wind over the cell that covers one tile.
    ///
    /// The wind is a bounded integer vector over the two axes of the cell
    /// lattice. It is carried state, so a watcher who reads it reads what the
    /// next step will read.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    #[must_use]
    pub fn wind_at(&self, address: Axial) -> Option<Wind> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.wind_at(self.weather_cell_of(tile)?))
    }

    /// Returns the temperature of the cell that covers one tile.
    ///
    /// The unit is a whole degree on the scale of the weather field, from
    /// zero to the heat ceiling. It is carried state, so a watcher who reads
    /// it reads what the next step will read.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    #[must_use]
    pub fn temperature_at(&self, address: Axial) -> Option<i32> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.warmth_at(self.weather_cell_of(tile)?))
    }

    /// Reports whether the ground under one tile is wet.
    ///
    /// A unit that gathers on wet ground takes more in one tick than a unit
    /// on dry ground.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    #[must_use]
    pub fn ground_is_wet(&self, address: Axial) -> Option<bool> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.cell_is_wet(self.weather_cell_of(tile)?))
    }

    /// Puts weather over a set of places, at the command of a god.
    ///
    /// The faction is the congregation the god directs. Each place names a
    /// tile, and the water lands on the level 1 cell that covers it, so two
    /// places in one cell are one place.
    ///
    /// **A god acts only where its own people hold the ground.** The cell of
    /// every place must hold at least one tile of the faction.[^1]
    ///
    /// **The call is all or nothing.** Every place is resolved and every gate
    /// is checked before anything changes.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the faction is outside the set this world holds,
    /// when the caller names more places than one call carries, when the
    /// strength is outside its range, when a place lies outside the world,
    /// when the faction holds no ground in the cell of a place, and when the
    /// faction inflicted weather too recently.
    ///
    /// # References
    ///
    /// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    /// [^2]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D3. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    pub fn inflict_weather(
        &mut self,
        faction: FactionId,
        places: &[Axial],
        strength: u8,
    ) -> Result<Storm, WeatherError> {
        // The water lands on the weather cell that covers the place, and the
        // weather lattice has a pitch of its own. The holder gate below still
        // asks the level 1 holding, because that is where a holder lives.
        let layout = self.weather_layout;
        let lattice = self.weather_lattice;
        let holding = &self.holding;
        let grid = self.grid;
        let ground = Ground {
            grid,
            cell_of: &move |tile: TileIdx| weather_cell_of(layout, lattice, tile),
            holders_near: &move |address: Axial| {
                let tile = grid.index_of(address)?;
                let key = holding.layout().key_of(tile)?;
                holding.block_mask(holding.layout().block_of_key(key))
            },
        };
        self.weather
            .inflict(faction, places, strength, self.tick, &ground)
    }

    fn cell_of(&self, tile: TileIdx) -> Option<u32> {
        let layout = self.pyramid.layout();
        Some(layout.block_of_key(layout.key_of(tile)?))
    }

    /// Returns the weather cell that covers a tile.
    ///
    /// **This is not the level 1 cell.** The weather lattice states its own
    /// pitch, and the two agree only when the world takes the level 1 pitch.
    /// Every reader of the weather field asks this rather than the level 1
    /// reader, so one fact has one declaration site.[^1]
    ///
    /// **The weather lattice is larger than the world, and this adds the
    /// margin offset.** The margin is a ring of cells that the solve steps
    /// and no reader sees, and the answer here names a cell of the whole
    /// lattice. So a reader that goes through this reads the world wherever
    /// the margin stands, and a reader that indexed a weather plane by a
    /// level 1 cell would read the margin instead.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn weather_cell_of(&self, tile: TileIdx) -> Option<u32> {
        weather_cell_of(self.weather_layout, self.weather_lattice, tile)
    }

    /// Returns the weather lattice, with the margin it carries.
    #[must_use]
    pub const fn weather_lattice(&self) -> PaddedLattice {
        self.weather_lattice
    }

    /// Returns the block geometry of the weather lattice.
    ///
    /// The drawing reads it, because the pitch of the lattice decides whether
    /// a cell value paints as a field or as a tile.
    #[must_use]
    pub const fn weather_layout(&self) -> BlockLayout {
        self.weather_layout
    }

    /// Returns what the recovery rule needs to know about one tile.
    ///
    /// The reader gathers the moisture over the tile and the upgrade that
    /// stands on it. It applies neither. The recovery rule holds the whole of
    /// the arithmetic, so the moisture cannot be read one way here and
    /// another way there.[^1]
    ///
    /// The moisture is the water on the ground of the weather cell that
    /// covers the tile, which is not the level 1 cell unless the world takes
    /// the level 1 pitch.[^2] The
    /// value is the one that the solve of the previous frame left, in the
    /// same way the gather resolve reads it.
    ///
    /// A tile that carries no finished upgrade answers with bare ground.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    fn tile_ground(&self, tile: TileIdx) -> TileGround {
        let moisture = self
            .weather_cell_of(tile)
            .map_or(0, |cell| self.weather.ground_at(cell).0);
        let Some(site) = self.upgrades.at(tile) else {
            return TileGround {
                moisture,
                ..TileGround::BARE
            };
        };
        let improvement = self
            .upgrade_table
            .row(site.category, site.level)
            .map_or(0, |row| row.recovery_change);
        TileGround {
            moisture,
            improvement,
            condition: site.condition.0,
        }
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
        let carry = carry_class_of(
            self.soldiers.carry_column()[slot as usize],
            self.soldiers.home_column()[slot as usize],
            self.carry_mark,
        );
        Some(choose::explain(
            cell,
            choose::UnitState { need, carry },
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

    /// Returns which factions stand on the ground of which other factions.
    ///
    /// Row `host` names every faction that has a live unit standing on a tile
    /// that `host` holds. A unit on ground its own faction holds sets no bit,
    /// so the diagonal is always empty.[^1]
    ///
    /// **The answer is 63 words whatever the population.** The relation is a
    /// mask row for each faction, which is the shape every relation between
    /// factions takes in this project.[^2]
    ///
    /// The relation is exact. The fold reads the holder of the exact tile
    /// each unit stands on, so a clear bit means that no unit is there.
    ///
    /// # Errors
    ///
    /// Returns an error when the population changed since the last step, so
    /// that a caller meets a refusal rather than a stale answer.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^3]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    pub fn presence_rows(&self) -> Result<&[FactionMask], BridgeError> {
        self.presence.rows(&self.soldiers)
    }

    /// Returns one entry for each tile that carries a unit: the tile and the
    /// faction that has the most units on it.
    ///
    /// **The tie goes to the lowest faction identifier.** The units of one
    /// tile reach this pass in the order the unit index packs them, and that
    /// order changes when a unit dies, moves or is promoted. A rule that took
    /// the first faction it found would take the packing, and a packing is
    /// not a stable key.[^1] [^2]
    ///
    /// The tally holds only the factions that stand on one tile, and the
    /// capacity of the ground bounds how many units that is. It is therefore
    /// local to one tile, and no stored field is indexed by the faction.[^1]
    ///
    /// Each thread reads a contiguous run of blocks and writes its own slot.
    /// The join reads the slots in slot order, and a block holds a
    /// contiguous run of tiles in ascending order, so the joined list is in
    /// ascending tile order at every thread count. Nothing reads which thread
    /// finished first.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the thread count is zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^2]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn tile_occupancy(&self, threads: usize) -> Result<Vec<(TileIdx, FactionId)>, StepError> {
        let threads = threads.max(1);
        let blocks = self.bridge.layout().block_count();
        if blocks == 0 {
            return Ok(Vec::new());
        }
        let block_chunk = (blocks as usize).div_ceil(threads).max(1);
        let slot_count = (blocks as usize).div_ceil(block_chunk);
        let mut slots: Slots<Vec<(TileIdx, FactionId)>> =
            Slots::filled(slot_count, Vec::new()).map_err(|_| StepError::ZeroThreads)?;
        let bridge = &self.bridge;
        let arena = &self.soldiers;
        std::thread::scope(|scope| {
            let mut first = 0u32;
            for slot in slots.entries_mut() {
                let start = first;
                let stop = (start as usize + block_chunk).min(blocks as usize) as u32;
                first = stop;
                scope.spawn(move || {
                    let factions = arena.faction_column();
                    let tiles = arena.tile_column();
                    let mut tally: Vec<(FactionId, u32)> = Vec::new();
                    for block in start..stop {
                        let (keys, units) = bridge.block_window(block);
                        let mut position = 0usize;
                        while position < keys.len() {
                            let mut end = position + 1;
                            while end < keys.len() && keys[end] == keys[position] {
                                end += 1;
                            }
                            let tile_units = &units[position..end];
                            position = end;
                            tally.clear();
                            for unit in tile_units {
                                let faction = factions[unit.index() as usize];
                                match tally.iter_mut().find(|(named, _)| *named == faction) {
                                    Some((_, count)) => *count += 1,
                                    None => tally.push((faction, 1)),
                                }
                            }
                            // The most units takes the tick, and a tie goes
                            // to the lowest faction identifier. Neither test
                            // reads the packing of the units.
                            let mut best: Option<(FactionId, u32)> = None;
                            for (faction, count) in &tally {
                                let better = match best {
                                    None => true,
                                    Some((named, most)) => {
                                        *count > most || (*count == most && faction.0 < named.0)
                                    }
                                };
                                if better {
                                    best = Some((*faction, *count));
                                }
                            }
                            if let Some((faction, _)) = best {
                                let tile = tiles[tile_units[0].index() as usize];
                                slot.push((tile, faction));
                            }
                        }
                    }
                });
            }
        });
        let mut occupancy = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        // A block holds a contiguous run of tiles, and the slots join in
        // block order, so the list is already in tile order. The sort is what
        // makes that a property of the data rather than of the layout.
        occupancy.sort_unstable_by_key(|(tile, _)| tile.0);
        Ok(occupancy)
    }

    /// Reports whether a unit of `guest` stands on ground that `host` holds.
    ///
    /// Returns `false` when `guest` and `host` are the same faction, because
    /// the relation holds no diagonal.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the population changed since the last step.
    ///
    /// # References
    ///
    /// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    pub fn stands_in_territory(
        &self,
        guest: FactionId,
        host: FactionId,
    ) -> Result<bool, BridgeError> {
        self.presence.stands_in(&self.soldiers, guest, host)
    }

    /// Returns how far a unit sees, and what stops it seeing.
    #[must_use]
    pub const fn sight_rules(&self) -> SightRules {
        self.observation.rules()
    }

    /// Sets how far a unit sees, and what stops it seeing.
    ///
    /// The next step reads the new rules. The rules are a stored value that
    /// the step reads, so they enter the state hash and two worlds that
    /// differ in them diverge at once.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 and D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    pub const fn set_sight_rules(&mut self, rules: SightRules) {
        self.observation.set_rules(rules);
    }

    /// Reports whether a faction sees a tile now.
    ///
    /// A faction sees a tile when one of its own live units observes it. A
    /// settlement gives no sight, held ground gives no sight, and an upgrade
    /// gives no sight.[^1]
    ///
    /// The answer is a reading of this frame. A faction that marched away
    /// stops seeing the tile at once.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_sees_now(&self, faction: FactionId, address: Axial) -> bool {
        self.grid
            .index_of(address)
            .is_some_and(|tile| self.observation.sees_now(faction, tile))
    }

    /// Reports whether a faction has ever seen a tile.
    ///
    /// The remembered layer only grows. A faction that saw a tile once holds
    /// that fact for the life of the world, and the step never removes
    /// one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_has_seen(&self, faction: FactionId, address: Axial) -> bool {
        self.grid
            .index_of(address)
            .is_some_and(|tile| self.observation.has_seen(faction, tile))
    }

    /// Returns how many tiles a faction sees now.
    ///
    /// The count is widened, so it does not depend on the margin of a narrow
    /// type at the target extent.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn faction_seen_now(&self, faction: FactionId) -> i64 {
        self.observation.seen_now(faction)
    }

    /// Returns how many tiles a faction has ever seen.
    #[must_use]
    pub fn faction_seen_ever(&self, faction: FactionId) -> i64 {
        self.observation.seen_ever(faction)
    }

    /// Returns which factions see one level 1 cell now.
    ///
    /// The mask is derived from the layers of every faction, and only the
    /// observation pass writes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn cell_seen_now(&self, cell: u32) -> FactionMask {
        self.observation.block_seen_now(cell)
    }

    /// Returns which factions have ever seen one level 1 cell.
    #[must_use]
    pub fn cell_seen_ever(&self, cell: u32) -> FactionMask {
        self.observation.block_seen_ever(cell)
    }

    /// Returns what each faction sees and what each faction remembers.
    ///
    /// A caller that walks the tiles of one faction reads the layers here
    /// rather than asking about each tile in turn.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub const fn observation(&self) -> &Observation {
        &self.observation
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

    /// Returns how sound the upgrade on one tile is.
    ///
    /// The value runs from nothing to the full condition. A level that has
    /// just been finished stands at the full condition, the weather and a
    /// hostile army take from it, and a worker on the tile puts it back.
    ///
    /// Returns `None` when the tile carries no upgrade. A site under
    /// construction reports the full condition, because nothing stands on its
    /// tile for anything to wear.
    #[must_use]
    pub fn upgrade_condition(&self, address: Axial) -> Option<i64> {
        self.upgrade_at(address).map(|site| site.condition.0)
    }

    /// Returns the upgrades the wear pass removed since the last step began.
    ///
    /// **A removed upgrade leaves nothing behind.** The tile returns to the
    /// world the generator made, so this log is the only record that anything
    /// stood there. A reader that misses a step misses the events.
    ///
    /// The log is ordered by tile within the tick.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn collapsed_log(&self) -> &[UpgradeCollapsed] {
        &self.collapsed_log
    }

    /// Returns the upgrade levels that finished since the last step began.
    ///
    /// The log is ordered by tile within the tick. A reader that misses a
    /// step misses the events.
    #[must_use]
    pub fn finished_log(&self) -> &[UpgradeFinished] {
        &self.finished_log
    }

    /// Returns the settlements founded since the last step began.
    ///
    /// A founding by a caller between two steps lands here and stays until
    /// the next step clears it. The log is ordered by the order the foundings
    /// were made in, which no thread fixes: the settle path and every caller
    /// path are serial.
    #[must_use]
    pub fn founded_log(&self) -> &[SettlementFounded] {
        &self.founded_log
    }

    /// Returns how many upgrades the last wear collapsed.
    ///
    /// The count describes one tick, in the same way the visit count of the
    /// last build advance does. No pass reads it and it enters no state hash.
    #[must_use]
    pub fn upgrade_collapses(&self) -> u64 {
        self.upgrades.last_wear_collapses()
    }

    /// Returns the finished upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none, and when the upgrade there
    /// is still under construction. An unfinished build changes nothing about
    /// the tile.
    #[must_use]
    pub fn finished_upgrade(&self, address: Axial) -> Option<UpgradeCategory> {
        let site = self.upgrade_at(address)?;
        if site.is_complete() {
            Some(site.category)
        } else {
            None
        }
    }

    /// Returns the level that stands on one tile.
    ///
    /// Returns zero when the tile carries no upgrade, and when the upgrade
    /// there has not reached its first level.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub fn upgrade_level(&self, address: Axial) -> u8 {
        self.upgrade_at(address)
            .map_or(upgrade::NO_LEVEL, |site| site.level)
    }

    /// Returns the row that stands on one tile.
    ///
    /// Returns `None` when the tile carries no upgrade, and when the upgrade
    /// there has not reached its first level. This is how a pass asks what an
    /// upgrade does: it reads a column of the row, and it names no
    /// category.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub fn standing_upgrade_row(&self, address: Axial) -> Option<UpgradeRow> {
        let tile = self.grid.index_of(address)?;
        self.upgrades.standing(tile, &self.upgrade_table)
    }

    /// Returns every entry that stands at a level, with the row it indexes.
    ///
    /// An entry whose first level is still under construction is not here,
    /// because it changes nothing about its tile. The walk is over the sparse
    /// map in tile order, so it is not a walk over the world.[^1]
    ///
    /// A pass that asks what the upgrades of a world do reads this and then
    /// reads a column. It names no category.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn standing_rows(&self) -> impl Iterator<Item = (UpgradeSite, UpgradeRow)> + '_ {
        self.upgrades.sites().iter().filter_map(|site| {
            self.upgrade_table
                .row(site.category, site.level)
                .map(|row| (*site, row))
        })
    }

    /// Returns the table that a category and a level index.
    ///
    /// The table holds one row for each pair of a category and a level. A row
    /// names the ground it fits, the work it takes and the columns a pass
    /// reads.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub const fn upgrade_table(&self) -> &UpgradeTable {
        &self.upgrade_table
    }

    /// Writes one row of the upgrade table.
    ///
    /// The caller gives the whole row. There is no partial form, because a
    /// caller that gave two columns would leave the rest at zero and would
    /// define an upgrade that changes nothing else without knowing it.[^1]
    ///
    /// **The values are content and not a budget.** No record holds one,
    /// because a record may hold no number that a content choice can
    /// move.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no category, and when the level
    /// is not one the table holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
    pub fn define_upgrade_row(
        &mut self,
        category: u8,
        level: u8,
        row: UpgradeRow,
    ) -> Result<(), UpgradeTableError> {
        self.upgrade_table.define(category, level, row)
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
        self.tile_capacity_for(address, crate::terrain::NO_WATER_CROSSING)
    }

    /// Returns the number of units of a given water crossing that may stand
    /// on one tile.
    ///
    /// The argument is the water crossing column of a unit type row, and zero
    /// means cannot.[^2] The reader still reads the ground table and the
    /// upgrade table together, and it still states no rule of its own about
    /// which ground admits whom.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    #[must_use]
    pub fn tile_capacity_for(&self, address: Axial, water_crossing: u32) -> Option<u32> {
        let ground = self.terrain.kind(address)?.capacity_for(water_crossing);
        Some(upgrade::capacity_with(
            ground,
            self.standing_upgrade_row(address),
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
    /// **The order names a category and never a level.** The engine resolves
    /// the row from the ground under the tile and the level that stands
    /// there, and it refuses when no row fits.[^3]
    ///
    /// # Errors
    ///
    /// Returns a refusal when the identity is dead, when the tile carries
    /// another category, when the category is at its top, when the row does
    /// not fit the ground, and when the row asks for ground that the
    /// builder's own faction does not hold.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub fn order_build(
        &mut self,
        entity: Entity,
        category: UpgradeCategory,
    ) -> Result<(), BuildRefusal> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check, so the answer and the verb
        // cannot state two different rules.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        if let Err(refusal) = self.build_refusal(entity, category) {
            // The plan counts the refusals the permission rule makes, and
            // not the ones the table or the identity make. The split is the
            // one the verb kept before the check moved out of it.
            if matches!(
                refusal,
                BuildRefusal::ProjectHoldsAnother { .. }
                    | BuildRefusal::NoProject { .. }
                    | BuildRefusal::GroundNotHeld { .. }
            ) {
                self.plan.count_refusal();
            }
            return Err(refusal);
        }
        if self.soldiers.set_build_order(entity, Some(category)) {
            Ok(())
        } else {
            Err(BuildRefusal::NoSuchBuilder)
        }
    }

    /// Reports whether the build verb would refuse one builder, without
    /// ordering anything.
    ///
    /// **This is the one statement of the rule.** The build verb calls it
    /// before it writes an order, and the legality answer calls it to fill
    /// one row of the action table.[^1] Nothing here mutates, so a caller
    /// may ask about every candidate and change nothing.
    ///
    /// # Errors
    ///
    /// Returns the refusal the build verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn build_refusal(
        &self,
        entity: Entity,
        category: UpgradeCategory,
    ) -> Result<(), BuildRefusal> {
        // The verb refuses at the moment of the order, so a caller learns at
        // once. The pass below applies the same test on every step, so a
        // build whose ground changed hands stops.[^2]
        //
        // [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        let (Some(tile), Some(faction)) =
            (self.soldiers.tile(entity), self.soldiers.faction(entity))
        else {
            return Err(BuildRefusal::NoSuchBuilder);
        };
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        let ground = self
            .grid
            .address_of(tile)
            .and_then(|address| self.terrain.kind(address))
            .ok_or(BuildRefusal::NoSuchBuilder)?;
        let row = resolve_build_row(
            &self.upgrade_table,
            ground,
            self.upgrades.at(tile),
            category,
        )?;
        let zoned = self.plan.zones(faction, tile);
        if !build_is_permitted(holder, faction, row, category, zoned) {
            // The three refusals answer three rules. A project of another
            // category met the plan first, because it refuses whatever the
            // row asks for. A row that asks for held ground then met the
            // ground rule. A row that asks for none met the plan.
            return Err(match zoned {
                Some(held) if held != category => BuildRefusal::ProjectHoldsAnother {
                    zoned: held,
                    asked: category,
                },
                _ if row.own_ground_required == 0 => BuildRefusal::NoProject { category },
                _ => BuildRefusal::GroundNotHeld { category },
            });
        }
        if self.soldiers.slot_of(entity).is_some() {
            Ok(())
        } else {
            Err(BuildRefusal::NoSuchBuilder)
        }
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
    pub fn build_order(&self, entity: Entity) -> Option<Option<UpgradeCategory>> {
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
        let Some(site) = self.upgrades.remove(tile) else {
            return false;
        };
        // **An upgrade has two sinks, and both say so.** The wear pass writes
        // the same event with a cause of its own. A destruction that wrote
        // nothing would leave a watcher with a tile that changed and no
        // reason for it.
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        self.collapsed_log.push(UpgradeCollapsed::new(
            self.tick,
            tile,
            holder,
            site.category.0,
            site.level,
            WEAR_CAUSE_ORDERED,
        ));
        true
    }

    /// Writes one project into the plan of one faction.
    ///
    /// **The solver calls this verb and a Python caller calls it.** No path
    /// exists for the solver alone, so a god that zones a project by hand
    /// puts it in the same list the solver writes to, and a unit cannot tell
    /// the two apart.[^1] [^2]
    ///
    /// The verb refuses a tile past the plan bound, a category that no row of
    /// the table fits, and a tile the faction does not hold when the row asks
    /// for held ground. A row that asks for no held ground is permitted
    /// anywhere, because that row is how a faction reaches ground it does not
    /// yet hold.[^3]
    ///
    /// # Errors
    ///
    /// Returns a refusal when the number names no faction, when the address
    /// lies outside the world, when no row fits, when the row asks for held
    /// ground that the faction does not hold, and when the plan is full.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    pub fn zone_project(
        &mut self,
        faction: FactionId,
        address: Axial,
        category: UpgradeCategory,
    ) -> Result<(), PlanRefusal> {
        if usize::from(faction.0) >= self.plan.faction_count() {
            self.plan.count_refusal();
            return Err(PlanRefusal::NoSuchFaction(faction));
        }
        let (Some(tile), Some(ground)) = (self.grid.index_of(address), self.terrain.kind(address))
        else {
            self.plan.count_refusal();
            return Err(PlanRefusal::AddressOutsideWorld(address));
        };
        let row = resolve_build_row(
            &self.upgrade_table,
            ground,
            self.upgrades.at(tile),
            category,
        )
        .map_err(|_| {
            self.plan.count_refusal();
            PlanRefusal::NoRowFits { category }
        })?;
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        if row.own_ground_required != 0 && holder.faction().map(|held| held.0) != Some(faction.0) {
            self.plan.count_refusal();
            return Err(PlanRefusal::GroundNotHeld { category });
        }
        self.plan.write(faction, Project::new(tile, category))
    }

    /// Removes the project one faction zoned on one tile.
    ///
    /// Reports whether it removed one. A caller and the solver both reach
    /// this verb.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    pub fn clear_project(&mut self, faction: FactionId, address: Axial) -> bool {
        let Some(tile) = self.grid.index_of(address) else {
            return false;
        };
        self.plan.clear(faction, tile)
    }

    /// Returns the projects one faction has zoned, in ascending tile order.
    #[must_use]
    pub fn plan_of(&self, faction: FactionId) -> &[Project] {
        self.plan.projects_of(faction)
    }

    /// Returns the category one faction zoned on one tile.
    #[must_use]
    pub fn project_at(&self, faction: FactionId, address: Axial) -> Option<UpgradeCategory> {
        let tile = self.grid.index_of(address)?;
        self.plan.zones(faction, tile)
    }

    /// Returns the project one unit takes: the nearest by hex distance.
    ///
    /// **This is the assignment rule, and the controller calls this one
    /// function.** When two projects tie on distance, the lower tile index
    /// wins. A unit of another faction, a dead unit and a faction with an
    /// empty plan each give nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    #[must_use]
    pub fn project_for(&self, faction: FactionId, unit: Entity) -> Option<Project> {
        if self.soldiers.faction(unit) != Some(faction) {
            return None;
        }
        let here = self.grid.address_of(self.soldiers.tile(unit)?)?;
        // The comparison is the tie rule, and it is written out rather than
        // left to the order of the list. The plan is in tile order, so a
        // comparison that took the last of several equals would take the
        // highest tile index.
        let mut best: Option<(u32, u32, Project)> = None;
        for project in self.plan.projects_of(faction) {
            let Some(there) = self.grid.address_of(project.tile) else {
                continue;
            };
            let key = (here.distance(there), project.tile.0, *project);
            if best.is_none_or(|held| key < held) {
                best = Some(key);
            }
        }
        best.map(|(_, _, project)| project)
    }

    /// Returns the way the solver would lay between two places.
    ///
    /// **This is the path search of the plan, and the solver walks the same
    /// window.** The search relaxes the tiles inside the radius a fixed
    /// number of times and then walks back from the far end, taking the
    /// neighbour with the lowest pair of cost and tile index at every step.
    /// It never runs until the frontier settles.[^1] [^2]
    ///
    /// Returns an empty list when either address lies outside the world, when
    /// the far end lies past the search radius, and when no path of the pass
    /// budget reaches it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
    #[must_use]
    pub fn planned_path(&self, from: Axial, to: Axial) -> Vec<Axial> {
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let Some(start) = self.grid.index_of(from) else {
            return Vec::new();
        };
        let Some(window) = plan::PathWindow::build(&ground, start, self.plan.rules()) else {
            return Vec::new();
        };
        window
            .path_to(self.grid, to)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|tile| self.grid.address_of(tile))
            .collect()
    }

    /// Returns the cost the path search charged to reach one place from
    /// another.
    ///
    /// The cost is the whole number the ground charges along the way. It is
    /// the value the tie rule of the path compares first, so a reader can see
    /// which two ways tie.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    #[must_use]
    pub fn planned_path_cost(&self, from: Axial, to: Axial) -> Option<i64> {
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let start = self.grid.index_of(from)?;
        let window = plan::PathWindow::build(&ground, start, self.plan.rules())?;
        window.cost_of(to)
    }

    /// Returns the values the plan and its solver read.
    #[must_use]
    pub const fn plan_rules(&self) -> PlanRules {
        self.plan.rules()
    }

    /// Sets the values the plan and its solver read.
    ///
    /// **The call clears every plan.** The bound decides the size of the
    /// register, so a register built with another bound holds its rows
    /// elsewhere, and carrying them over would put a project of one faction
    /// into the plan of another. A caller that changes the rules zones again.
    ///
    /// Every value is a balance row, and a blocker governs each of them.[^1]
    /// [^2]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the plan. `docs/reference/balance.md`
    /// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    pub fn set_plan_rules(&mut self, rules: PlanRules) {
        self.plan = PlanRegister::new(self.config.faction_count, rules);
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

    /// Returns how far one city reaches, in hex steps.
    ///
    /// The reach is the base plus one step for each block of finished
    /// upgrades on the ground the city held at the end of the previous step,
    /// capped at the bound.[^1] Returns `None` when the identity names no
    /// live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn city_reach(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        self.holding
            .cities(&self.settlements, &self.upgrades)
            .into_iter()
            .find(|city| city.slot == slot)
            .map(|city| city.reach)
    }

    /// Reports whether one faction holds one tile.
    ///
    /// Returns `None` when the address lies outside the world. The call reads
    /// the holder column, which the cities rewrite on every step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn holds(&self, faction: FactionId, address: Axial) -> Option<bool> {
        Some(self.holding.holder(address)?.faction() == Some(faction))
    }

    /// Returns how far a city reaches, and what extends the reach.
    ///
    /// The three values are balance rows, and every value in that register is
    /// unset until the balance pass measures it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the holding. `docs/reference/balance.md`
    #[must_use]
    pub const fn reach_rules(&self) -> ReachRules {
        self.holding.rules()
    }

    /// Sets how far a city reaches, and what extends the reach.
    ///
    /// The three values are balance rows.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the holding. `docs/reference/balance.md`
    pub const fn set_reach_rules(&mut self, rules: ReachRules) {
        self.holding.set_rules(rules);
    }

    /// Returns what a site resists, and what a raze costs over a capture.
    #[must_use]
    pub const fn siege_rules(&self) -> SiegeRules {
        self.siege_rules
    }

    /// Sets what a site resists, and what a raze costs over a capture.
    ///
    /// The two values are balance rows.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the siege. `docs/reference/balance.md`
    pub const fn set_siege_rules(&mut self, rules: SiegeRules) {
        self.siege_rules = rules;
    }

    /// Returns how a lease rises, falls and claims.
    ///
    /// A lease is one faction and one count for each tile. The count follows
    /// the units that stand on the tile, and a lease at or above the claim
    /// threshold holds the tile whatever city reaches it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decisions D1 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    #[must_use]
    pub const fn lease_rules(&self) -> LeaseRules {
        self.holding.lease_rules()
    }

    /// Sets how a lease rises, falls and claims.
    ///
    /// The seven values are balance rows, and one blocker governs each of
    /// them.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the lease. `docs/reference/balance.md`
    /// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    pub const fn set_lease_rules(&mut self, rules: LeaseRules) {
        self.holding.set_lease_rules(rules);
    }

    /// Returns the lease of one tile: the faction it names, and the count.
    ///
    /// Returns `None` when the address lies outside the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D1. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    #[must_use]
    pub fn tile_lease(&self, address: Axial) -> Option<(Holder, i32)> {
        self.holding.lease(address)
    }

    /// Returns how many ticks a campaign runs before it expires.
    ///
    /// Zero means that no campaign expires. The value is a balance row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the campaign deadline. `docs/reference/balance.md`
    #[must_use]
    pub const fn campaign_deadline(&self) -> Tick {
        self.campaigns.deadline()
    }

    /// Sets how many ticks a campaign runs before it expires.
    ///
    /// Zero means that no campaign expires.
    pub const fn set_campaign_deadline(&mut self, deadline: Tick) {
        self.campaigns.set_deadline(deadline);
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
        self.derive_return_fields();
        self.derive_stock_field();
        self.derive_destination_fields();
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
        self.returns.derive(&self.pyramid, &self.site_seeds());
        // **No return plane conducts across water**, so every plane takes the
        // land crossing. The empty slice is how the approach field states
        // that, in the way the return field states it to the coarse
        // derivation.[^3]
        //
        // [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
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

    /// Returns the load at which a unit counts as laden.
    #[must_use]
    pub const fn carry_mark(&self) -> Amount {
        self.carry_mark
    }

    /// Sets the load at which a unit counts as laden.
    ///
    /// A mark of zero makes every unit that holds a home laden, whatever it
    /// carries.
    pub const fn set_carry_mark(&mut self, mark: Amount) {
        self.carry_mark = mark;
    }

    /// Returns the carry class of one unit.
    ///
    /// A unit is laden when it holds a home site and its load reaches the
    /// carry mark. **A unit with no home is never laden**, because the
    /// delivery moves a load into the store of a home site, and a unit that
    /// has none can deliver to nothing.[^1]
    ///
    /// Returns `None` when the identity is dead or names no unit of this
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    #[must_use]
    pub fn carry_class(&self, entity: Entity) -> Option<CarryClass> {
        let slot = self.soldiers.slot_of(entity)? as usize;
        Some(carry_class_of(
            self.soldiers.carry_column()[slot],
            self.soldiers.home_column()[slot],
            self.carry_mark,
        ))
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
    /// Moves the load of every unit that stands on the tile of its home site
    /// into the store of that site.
    ///
    /// **The resource loop had no sink.** A unit gathered into a carry column
    /// and no verb moved that load into a store, so the store of a site rose
    /// only by the fixed rate the founding set from the survey, and the ground
    /// the units stood on did not change it. The whole economy was a constant
    /// decided before the first frame.[^1]
    ///
    /// **A delivery is admitted by sort, then by transfer.** Two units of one
    /// site deliver into one store and the store saturates at its ceiling, so
    /// a saturating add is not order-free.[^2] The pass therefore orders the
    /// deliveries by the site and then by the identity of the unit, and it
    /// transfers in that order. That is the shape the gather resolve already
    /// uses against a deposit.[^3] [^4]
    ///
    /// **A load the store cannot hold stays in the carry.** A quantity that
    /// vanished without a record would break the conservation equality, and
    /// nothing would fail.[^2] The unit keeps the remainder and delivers it on
    /// a later tick.
    ///
    /// **The transfer moves whole units only.** A carry holds a whole number
    /// and a store holds a fixed-point quantity, so the room a store has may
    /// end between two whole numbers. The pass takes the whole part of that
    /// room, which converts exactly in both directions. A conversion that
    /// rounded would create or destroy a quantity.[^5]
    ///
    /// **The commodity comes from the declared map and never from a literal.**
    /// The engine already writes the number of a commodity at two sites, and a
    /// third literal would be one value in three places with nothing to fail
    /// when the copies disagree.[^6] [^7]
    ///
    /// The pass runs on the calling thread. It writes one store at a time in a
    /// stated order, so it names no thread and depends on no thread count.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the ordering refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0004, iteration order is explicit, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^6]: Findings register, FND-191. `docs/FINDINGS.md`
    /// [^7]: Decisions register, DEC-073. `docs/DECISIONS.md`
    fn deliver(&mut self, threads: usize) -> Result<(), StepError> {
        let carriers = self.carriers_at_home(threads);
        if carriers.is_empty() {
            return Ok(());
        }
        let keys: Vec<BoundedKey> = carriers
            .iter()
            .map(|(unit, site)| BoundedKey::new(u64::from(*site), unit.to_bits()))
            .collect();
        let ceiling = u64::from(self.settlements.slot_count().saturating_sub(1));
        let order = gather_order_of(&keys, ceiling)?;

        for index in order {
            let (unit, site) = carriers[index as usize];
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            for kind in ResourceKind::ALL {
                let held_by_unit = load.of(kind);
                if held_by_unit.0 == 0 {
                    continue;
                }
                let commodity = WORK_COMMODITY[kind.index()];
                let Some(held) = self
                    .settlements
                    .store_column()
                    .get(site as usize)
                    .and_then(|store| store.quantity(commodity))
                else {
                    continue;
                };
                // The room of the store, in whole units. The subtract cannot
                // go below zero because the ceiling is the largest value the
                // scale holds.
                let room = sim_math::sub(Fix32::MAX, held).to_int_floor();
                let moved = held_by_unit.0.min(u32::try_from(room).unwrap_or(0));
                if moved == 0 {
                    continue;
                }
                // The conversion is exact in both directions: a whole number
                // that the room admits fits the scale, and the scale holds it
                // with no fractional part.
                let quantity = Fix32::from_int(i16::try_from(moved).unwrap_or(i16::MAX));
                let moved = u32::try_from(quantity.to_int_floor()).unwrap_or(0);
                if moved == 0 {
                    continue;
                }
                let after = sim_math::add(held, quantity);
                if !self.set_store_quantity(site, commodity, after) {
                    continue;
                }
                self.soldiers.take_carry(unit, kind, Amount(moved));
                self.delivered[kind.index()] += u64::from(moved);
            }
        }
        Ok(())
    }

    /// Returns the negotiation and the contract between one ordered pair.
    ///
    /// The pair is ordered. The row for the proposer and the responder, in
    /// that order, holds the negotiation that the proposer opened toward the
    /// responder. A pair that nobody ever spoke about answers an idle row.
    ///
    /// Returns `None` when either identifier is at or above the faction count
    /// of this world.
    #[must_use]
    pub fn trade_row(&self, proposer: FactionId, responder: FactionId) -> Option<TradeRow> {
        self.trade.row(proposer, responder)
    }

    /// Returns every row of the negotiation plane, in pair order.
    ///
    /// The slice is empty until somebody speaks. The index of a pair is the
    /// proposer times the faction count plus the responder.
    #[must_use]
    pub fn trade_book(&self) -> &[TradeRow] {
        self.trade.rows()
    }

    /// Returns what the last step said about trade.
    ///
    /// The log holds one entry for each speech act and for each settlement or
    /// default that the step resolved. A control plane reads it at the frame
    /// barrier and never inside a step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn trade_log(&self) -> &[TradeSpoken] {
        &self.trade_log
    }

    /// Returns the trade log as raw bytes.
    ///
    /// The event type is plain data with declared padding, so the bytes are
    /// the log and nothing else.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn trade_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.trade_log)
    }

    /// Reports whether any live unit of one faction stands on ground that
    /// another faction holds.
    ///
    /// **This is the gate that every speech act passes.** A player speaks to
    /// another player only while one of its own units stands in that player's
    /// territory, and a trade is a thing two players say to each other.[^1]
    ///
    /// The read walks the unit column once and reads the holder of the tile
    /// each unit stands on. It reads primary state only, so it holds no copy
    /// of an answer that another structure also holds.[^2] It costs one column
    /// read for each live unit, once for each speech act, and a speech act
    /// happens between frames.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn stands_in_territory_of(&self, speaker: FactionId, listener: FactionId) -> bool {
        let holders = self.holding.holders();
        let target = Holder::of(listener);
        self.soldiers.iter().any(|unit| {
            self.soldiers.faction(unit) == Some(speaker)
                && self
                    .soldiers
                    .tile(unit)
                    .and_then(|tile| holders.get(tile.0 as usize).copied())
                    == Some(target)
        })
    }

    /// Refuses a faction identifier that this world does not hold.
    fn check_faction(&self, faction: FactionId) -> Result<(), TradeError> {
        if faction.0 >= self.config.faction_count {
            return Err(TradeError::NoSuchFaction(faction));
        }
        Ok(())
    }

    /// Refuses a pair that this world cannot hold a negotiation for.
    fn check_pair(&self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_faction(speaker)?;
        self.check_faction(other)?;
        if speaker == other {
            return Err(TradeError::SameFaction(speaker));
        }
        Ok(())
    }

    /// Returns the orientation of the live row of an unordered pair.
    ///
    /// **One unordered pair holds at most one live negotiation.** Two players
    /// discuss one thing at a time, so the answer names the row rather than
    /// leaving the caller to guess which of the two orientations is live.
    fn live_orientation(
        &self,
        speaker: FactionId,
        other: FactionId,
    ) -> Result<(FactionId, FactionId), TradeError> {
        if self
            .trade
            .row(speaker, other)
            .is_some_and(|row| row.is_live())
        {
            return Ok((speaker, other));
        }
        if self
            .trade
            .row(other, speaker)
            .is_some_and(|row| row.is_live())
        {
            return Ok((other, speaker));
        }
        Err(TradeError::NothingOpen)
    }

    /// Returns the faction whose turn it is to answer a live row.
    fn turn_of(row: TradeRow, proposer: FactionId, responder: FactionId) -> Option<FactionId> {
        match row.status {
            TRADE_OFFERED => Some(responder),
            TRADE_COUNTERED => Some(proposer),
            _ => None,
        }
    }

    /// Writes one entry into the trade log.
    fn say(&mut self, proposer: FactionId, responder: FactionId, act: u8, status: u8) {
        self.trade_log.push(TradeSpoken::new(
            self.tick, proposer, responder, act, status,
        ));
    }

    /// Opens a negotiation from one faction toward another.
    ///
    /// The terms bind both parties. The give side is what the proposer owes
    /// and the take side is what the responder owes. Each is a whole quantity
    /// of one resource kind, so no term of a contract is a floating point
    /// number.[^1]
    ///
    /// The term is how many ticks the contract runs for once it binds. The
    /// acceptance turns it into a deadline. A contract that cannot fail is not
    /// a contract, so a term of zero is refused.
    ///
    /// **The offer passes the presence gate.** A unit of the proposer must
    /// stand on ground that the responder holds.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when a party names no faction of this world, when the
    /// two parties are one faction, when either kind names no resource, when
    /// either quantity is zero, when the term is zero, when the unordered pair
    /// already holds a live negotiation, when a terminal refusal closed this
    /// direction, or when the proposer has no presence.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    // The terms of a contract are six values and the parties are two more.
    // A structure that held them would be a second name for the row this
    // function writes, and the control plane would then state the terms twice:
    // once to build it and once to read the row back.
    #[allow(clippy::too_many_arguments)]
    pub fn offer_trade(
        &mut self,
        proposer: FactionId,
        responder: FactionId,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        term: u32,
    ) -> Result<(), TradeError> {
        self.offer_consideration(
            proposer,
            responder,
            Consideration::resource(give_kind, give_amount),
            Consideration::resource(take_kind, take_amount),
            term,
        )
    }

    /// Opens a negotiation from one faction toward another, with a tagged
    /// consideration on each side.
    ///
    /// **Each side is one tag and the content the tag names.**[^1] A resource
    /// side is a kind and a quantity, and a unit carries it. A land side is a
    /// list of tiles that the debtor holds, and the holder changes when the
    /// other side is delivered in full. A relation side is stored and applies
    /// as a logged no-op until the relation matrix exists.
    ///
    /// The give side is what the proposer owes and the take side is what the
    /// responder owes. A land side is checked against its debtor: every tile
    /// of the give side must be held by the proposer, and every tile of the
    /// take side by the responder.[^2] A land side whose tiles carry an
    /// upgrade is refused while the question of what happens to the upgrade
    /// is open.[^3]
    ///
    /// # Errors
    ///
    /// Returns every error the resource verb returns, and also an error when
    /// a tag names no kind, when a land side is empty or names more tiles than
    /// the bound, when a tile lies outside the world, when the debtor does not
    /// hold a tile, or when a tile carries an upgrade.
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^2]: ADR-0147, a contract consideration is a tagged kind, decision D4. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^3]: Blockers register, BLK-036. `docs/BLOCKERS.md`
    pub fn offer_consideration(
        &mut self,
        proposer: FactionId,
        responder: FactionId,
        give: Consideration,
        take: Consideration,
        term: u32,
    ) -> Result<(), TradeError> {
        self.check_pair(proposer, responder)?;
        // An offer across a pair at war is refused before anything else is
        // read. The predicate is the relation module's, so the trade verbs
        // and the contest read one statement of the war band.[^war]
        //
        // [^war]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        if !self.relations.permits_offer(proposer, responder) {
            return Err(TradeError::AtWar);
        }
        let give = self.check_consideration(proposer, give)?;
        let take = self.check_consideration(responder, take)?;
        if term == 0 {
            return Err(TradeError::NoDeadline);
        }
        if self.live_orientation(proposer, responder).is_ok() {
            return Err(TradeError::AlreadyOpen);
        }
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        if row.closed_until.0 > self.tick.0 {
            return Err(TradeError::Closed(row.closed_until));
        }
        if !self.stands_in_territory_of(proposer, responder) {
            return Err(TradeError::NoPresence);
        }
        let tick = self.tick;
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        entry.clear();
        entry.opened = tick;
        entry.give_tag = give.tag;
        entry.give_kind = give.kind;
        entry.give_amount = give.amount;
        entry.take_tag = take.tag;
        entry.take_kind = take.kind;
        entry.take_amount = take.amount;
        entry.term = term;
        entry.status = TRADE_OFFERED;
        entry.rounds = 1;
        self.trade.set_land(index, false, give.tiles);
        self.trade.set_land(index, true, take.tiles);
        self.say(proposer, responder, ACT_OFFER, TRADE_OFFERED);
        Ok(())
    }

    /// Checks one side of a contract against its debtor and returns it in
    /// the form the plane stores.
    ///
    /// A land side comes back sorted and without a repeated tile, and its
    /// amount is the tile count. The other kinds come back as given.
    fn check_consideration(
        &self,
        debtor: FactionId,
        mut side: Consideration,
    ) -> Result<Consideration, TradeError> {
        match side.tag {
            KIND_RESOURCE => {
                trade::kind_of(side.kind)?;
                if side.amount == 0 {
                    return Err(TradeError::EmptyTerms);
                }
                side.tiles.clear();
            }
            KIND_LAND => {
                side.tiles.sort_unstable();
                side.tiles.dedup();
                if side.tiles.is_empty() {
                    return Err(TradeError::EmptyTerms);
                }
                let count = u32::try_from(side.tiles.len()).unwrap_or(u32::MAX);
                if count > self.land_list_bound {
                    return Err(TradeError::TooMuchLand(count, self.land_list_bound));
                }
                let holders = self.holding.holders();
                let wanted = Holder::of(debtor);
                for tile in &side.tiles {
                    let Some(holder) = holders.get(tile.0 as usize) else {
                        return Err(TradeError::NoSuchTile);
                    };
                    if *holder != wanted {
                        return Err(TradeError::LandNotHeld(*tile));
                    }
                    // **An upgrade changes hands with the ground**, so a
                    // land side that carries one needs no refusal and no
                    // arithmetic. The upgrade is stored against the tile and
                    // no owner stands beside it, so the ground carries it.[^32]
                    //
                    // [^32]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
                }
                side.kind = 0;
                side.amount = count;
            }
            KIND_RELATION => {
                if side.amount == 0 {
                    return Err(TradeError::EmptyTerms);
                }
                side.tiles.clear();
            }
            other => return Err(TradeError::NoSuchTag(other)),
        }
        Ok(side)
    }

    /// Returns every tile of the level 1 cell that covers one address, in
    /// ascending tile index.
    ///
    /// A cell on the world edge is partial, and the call returns the tiles
    /// that exist. This is the set a land side names when it names a cell.
    ///
    /// # Errors
    ///
    /// Returns an error when the address lies outside the world.
    pub fn cell_tiles(&self, address: Axial) -> Result<Vec<TileIdx>, TradeError> {
        let layout = self.pyramid.layout();
        let tile = self.grid.index_of(address).ok_or(TradeError::NoSuchCell)?;
        let key = layout.key_of(tile).ok_or(TradeError::NoSuchCell)?;
        let block = layout.block_of_key(key);
        let edge = layout.block_edge();
        let across = block % layout.blocks_wide();
        let down = block / layout.blocks_wide();
        let first_q = across * edge;
        let last_q = (first_q + edge).min(self.grid.width());
        let first_r = down * edge;
        let last_r = (first_r + edge).min(self.grid.height());
        let mut tiles = Vec::with_capacity((edge * edge) as usize);
        for r in first_r..last_r {
            for q in first_q..last_q {
                let here = Axial::new(
                    i32::try_from(q).map_err(|_| TradeError::NoSuchCell)?,
                    i32::try_from(r).map_err(|_| TradeError::NoSuchCell)?,
                );
                tiles.push(self.grid.index_of(here).ok_or(TradeError::NoSuchCell)?);
            }
        }
        Ok(tiles)
    }

    /// Returns the tiles of one side of one ordered pair. The give side is
    /// `false` and the take side is `true`.
    ///
    /// A side that is not land answers an empty slice, and so does a pair
    /// that names no faction.
    #[must_use]
    pub fn trade_land(
        &self,
        proposer: FactionId,
        responder: FactionId,
        take_side: bool,
    ) -> &[TileIdx] {
        match self.trade.index_of(proposer, responder) {
            Some(index) => self.trade.land_of(index, take_side),
            None => &[],
        }
    }

    /// Returns the most tiles one land consideration may name.
    #[must_use]
    pub const fn land_list_bound(&self) -> u32 {
        self.land_list_bound
    }

    /// Sets the most tiles one land consideration may name.
    ///
    /// The bound is a balance value and not a budget.[^1] A bound of zero
    /// refuses every land side.
    ///
    /// # References
    ///
    /// [^1]: Balance register, land list bound. `docs/reference/balance.md`
    pub fn set_land_list_bound(&mut self, bound: u32) {
        self.land_list_bound = bound;
    }

    /// Returns how many advertisement rows one faction's board holds.
    #[must_use]
    pub const fn board_rows(&self) -> u16 {
        self.market.bound()
    }

    /// Sets how many advertisement rows one faction's board holds.
    ///
    /// The row count is a balance value and not a budget.[^1] A change
    /// empties every board, because the table is laid out by the bound.
    ///
    /// # References
    ///
    /// [^1]: Balance register, board size. `docs/reference/balance.md`
    pub fn set_board_rows(&mut self, rows: u16) {
        self.market.set_bound(rows);
    }

    /// Replaces the whole board of one faction.
    ///
    /// A board says what a faction offers and wants. It is a statement and
    /// not a speech act, so it passes no presence gate and costs no standing.
    /// A reader of another faction's board learns what that faction posted
    /// and nothing else.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction names no faction of this world, when
    /// the rows outnumber the bound, when a good names no resource kind, or
    /// when a row names neither offers nor wants. No row changes on an error.
    pub fn advertise(&mut self, faction: FactionId, rows: &[Advert]) -> Result<(), TradeError> {
        self.check_faction(faction)?;
        self.market.advertise(faction, rows)
    }

    /// Returns the board of one faction, empty rows included.
    ///
    /// The slice is empty for a faction that never advertised and for a
    /// number that names no faction.
    #[must_use]
    pub fn market(&self, faction: FactionId) -> &[Advert] {
        self.market.board(faction)
    }

    /// Restates the terms of a live negotiation.
    ///
    /// The speaker is the party that did not speak last. The terms are always
    /// stated in the orientation of the row, so the give side is what the
    /// party that opened the pair owes, whoever is speaking now.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, when either kind names no resource, when either quantity is zero,
    /// or when the speaker has no presence.
    pub fn counter_trade(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
    ) -> Result<(), TradeError> {
        self.counter_consideration(
            speaker,
            other,
            Consideration::resource(give_kind, give_amount),
            Consideration::resource(take_kind, take_amount),
        )
    }

    /// Restates the terms of a live negotiation, with a tagged consideration
    /// on each side.
    ///
    /// The terms are stated in the orientation of the row, whoever speaks.
    /// The give side is what the party that opened the pair owes, and a land
    /// side is checked against that party. The take side is what the other
    /// party owes, and a land side is checked against it.
    ///
    /// # Errors
    ///
    /// Returns every error the resource verb returns, and every error the
    /// tagged offer returns for a side.
    pub fn counter_consideration(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        give: Consideration,
        take: Consideration,
    ) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        // A counter across a pair at war is refused, as an offer is. A war
        // declared during a negotiation therefore ends the talking.
        if !self.relations.permits_offer(speaker, other) {
            return Err(TradeError::AtWar);
        }
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let give = self.check_consideration(proposer, give)?;
        let take = self.check_consideration(responder, take)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let status = if row.status == TRADE_OFFERED {
            TRADE_COUNTERED
        } else {
            TRADE_OFFERED
        };
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.give_tag = give.tag;
        entry.give_kind = give.kind;
        entry.give_amount = give.amount;
        entry.take_tag = take.tag;
        entry.take_kind = take.kind;
        entry.take_amount = take.amount;
        entry.status = status;
        entry.rounds = entry.rounds.saturating_add(1);
        self.trade.set_land(index, false, give.tiles);
        self.trade.set_land(index, true, take.tiles);
        self.say(proposer, responder, ACT_COUNTER, status);
        Ok(())
    }

    /// Agrees to the terms of a live negotiation, so a contract binds both.
    ///
    /// The speaker is the party that did not speak last. The deadline is this
    /// tick plus the term the offer named.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, or when the speaker has no presence.
    pub fn accept_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let deadline = Tick(self.tick.0.saturating_add(u64::from(row.term)));
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.status = TRADE_BOUND;
        entry.deadline = deadline;
        entry.rounds = entry.rounds.saturating_add(1);
        self.say(proposer, responder, ACT_ACCEPT, TRADE_BOUND);
        Ok(())
    }

    /// Declines the terms of a live negotiation.
    ///
    /// **This is a refusal and not a closed door.** The pair is idle after it,
    /// and either party may open a new negotiation on the next call. A player
    /// that wants the other to stop asking calls the closing verb instead.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, or when the speaker has no presence.
    pub fn refuse_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.clear();
        self.trade.clear_land(index);
        self.say(proposer, responder, ACT_REFUSE, TRADE_IDLE);
        Ok(())
    }

    /// Declines the terms and closes the direction for a stated number of
    /// ticks.
    ///
    /// **This is the terminal refusal.** It ends the negotiation, and it also
    /// stops the other party from opening a new one toward the speaker until
    /// the tick it names. The closure is directional: the speaker may still
    /// open a negotiation toward the other party, because the speaker closed
    /// its own door and promised no silence of its own.
    ///
    /// The tick that opens the direction again is readable. A caller reads the
    /// closure from the row for the other party and the speaker, in that
    /// order, which is the row the other party would open. A player that
    /// cannot tell a refusal from a closed door asks for ever.
    ///
    /// Only the speaker opens the direction early, through the opening verb.
    /// Nothing the other party does shortens the closure. That is what makes
    /// it terminal.
    ///
    /// # Errors
    ///
    /// Returns an error when the duration is zero, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no presence.
    pub fn close_trade(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        ticks: u32,
    ) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        if ticks == 0 {
            return Err(TradeError::NoDuration);
        }
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let until = Tick(self.tick.0.saturating_add(u64::from(ticks)));
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.clear();
        self.trade.clear_land(index);
        // The closure sits on the row the other party would open, which is
        // the pair with the other party first. Writing it on the live row
        // would close the speaker's own door and leave the other party free
        // to ask again, which is the opposite of what the verb promises.
        let door = self
            .trade
            .row_mut(other, speaker)
            .ok_or(TradeError::NothingOpen)?;
        door.closed_until = until;
        self.say(proposer, responder, ACT_CLOSE, TRADE_IDLE);
        Ok(())
    }

    /// Opens a direction that this faction closed, before the closure ends.
    ///
    /// Only the faction that closed the direction opens it again.
    ///
    /// # Errors
    ///
    /// Returns an error when the speaker closed nothing toward this party.
    pub fn reopen_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let row = self
            .trade
            .row(other, speaker)
            .ok_or(TradeError::NoSuchFaction(other))?;
        if row.closed_until.0 <= self.tick.0 {
            return Err(TradeError::NothingClosed);
        }
        let door = self
            .trade
            .row_mut(other, speaker)
            .ok_or(TradeError::NoSuchFaction(other))?;
        door.closed_until = Tick(0);
        self.say(other, speaker, ACT_REOPEN, row.status);
        Ok(())
    }

    /// Moves the load of every unit that stands on the site of a faction that
    /// a contract obliges it to deliver to, and then fails every contract that
    /// reached its deadline with a debt.
    ///
    /// **A contract moves nothing on its own.** A quantity reaches the other
    /// party because a unit carried it onto the tile of a settlement that
    /// party holds.[^1] The engine already moves a load this way when a unit
    /// stands on its own site, and this pass is the same transfer against
    /// another faction's site.
    ///
    /// **A delivery is admitted by sort, then by transfer.** Two units of one
    /// faction may deliver into one store, and a store saturates at its
    /// ceiling, so a saturating add is not order-free.[^2] The pass orders the
    /// deliveries by the site and then by the identity of the unit, which is
    /// the order the ordinary delivery already uses.[^3] [^4]
    ///
    /// **A load the store cannot hold stays in the carry.** A quantity that
    /// vanished without a record would break the conservation equality.[^2]
    ///
    /// **A delivery never passes the debt.** The transfer takes the smallest
    /// of what the unit carries, what the party owes, and what the store can
    /// hold. A contract therefore moves the quantity it named and no more.
    ///
    /// **The deadline is checked after the delivery of this tick.** A contract
    /// whose deadline is this tick gets this tick's delivery, and it fails
    /// only when a debt survives it.
    ///
    /// **A default costs the defaulting party the direction it would ask on
    /// again, for as long as the contract ran.** The duration is the term of
    /// the contract itself, so no balance figure decides it. The quantities
    /// that already moved stay where they arrived, because taking them back
    /// would need a transfer that no unit carried.[^1]
    ///
    /// The pass runs on the calling thread. It writes one store at a time in a
    /// stated order, so it names no thread and depends on no thread
    /// count.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the ordering refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decisions D1 and D4. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0004, iteration order is explicit, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn settle_trades(&mut self, threads: usize) -> Result<(), StepError> {
        if self.trade.is_empty() {
            return Ok(());
        }
        self.carry_contract_loads(threads)?;
        self.apply_priced_sides(threads);
        self.fail_overdue_contracts();
        Ok(())
    }

    /// Applies every land set and every relation step whose price has
    /// arrived.
    ///
    /// **A side that no unit carries applies when the other side is delivered
    /// in full.**[^1] A land set applies by giving every tile in it to the
    /// creditor, in ascending tile index. A relation step applies as a logged
    /// no-op, because the relation matrix arrives with a later pass; the log
    /// entry is the record that the side was delivered, and nothing else
    /// moves. When neither side is carried, both apply on the first pass
    /// after the contract binds.
    ///
    /// The walk is over the plane in pair order, after the carriers of this
    /// tick have delivered and before the deadline check, so a contract whose
    /// resource side settles this tick delivers its land this tick. The order
    /// between two land sets that name one tile is the pair order, and no
    /// thread and no hash order enters it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn apply_priced_sides(&mut self, threads: usize) {
        let count = self.trade.rows().len();
        for index in 0..count {
            let Some(row) = self.trade.row_at(index) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let (proposer, responder) = self.pair_of(index);
            let responder_done = row.owed_by_responder() == 0 || !row.responder_side_is_carried();
            let proposer_done = row.owed_by_proposer() == 0 || !row.proposer_side_is_carried();
            let mut applied = false;
            if !row.proposer_side_is_carried() && row.owed_by_proposer() > 0 && responder_done {
                self.apply_side(index, false, responder, row.give_tag, threads);
                applied = true;
            }
            if !row.responder_side_is_carried() && row.owed_by_responder() > 0 && proposer_done {
                self.apply_side(index, true, proposer, row.take_tag, threads);
                applied = true;
            }
            if !applied {
                continue;
            }
            let paid = self.trade.row_at_mut(index).is_some_and(|entry| {
                if entry.is_paid() {
                    entry.status = TRADE_SETTLED;
                    true
                } else {
                    false
                }
            });
            if paid {
                self.say(proposer, responder, ACT_SETTLE, TRADE_SETTLED);
                // A contract delivered in full warms both directions, from
                // this settle site as from the carried one.
                self.relations
                    .on_contract_delivered(self.tick, proposer, responder);
            }
        }
    }

    /// Applies one side that no unit carries, and marks it delivered.
    fn apply_side(
        &mut self,
        index: usize,
        take_side: bool,
        creditor: FactionId,
        tag: u8,
        threads: usize,
    ) {
        let act = match tag {
            KIND_LAND => {
                let tiles = self.trade.land_of(index, take_side).to_vec();
                self.holding.transfer(&tiles, Holder::of(creditor), threads);
                ACT_TRANSFER_LAND
            }
            KIND_RELATION => {
                // The relation matrix does not exist yet. The step is stored
                // and logged, and it moves nothing. A later pass replaces this
                // arm with the move, and until then this side is inert on
                // purpose.
                ACT_STEP_RELATION
            }
            _ => return,
        };
        if let Some(entry) = self.trade.row_at_mut(index) {
            if take_side {
                entry.taken = entry.take_amount;
            } else {
                entry.given = entry.give_amount;
            }
        }
        let (proposer, responder) = self.pair_of(index);
        self.say(proposer, responder, act, TRADE_BOUND);
    }

    /// Returns every delivery a bound contract admits this tick, in slot
    /// order.
    ///
    /// The walk is over the unit slots and it reads no derived structure. The
    /// order is the slot order, which does not depend on the thread count. The
    /// caller sorts it on a total key before it transfers anything.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn contract_carriers(&self, threads: usize) -> Vec<ContractDelivery> {
        let _ = threads;
        let mut found = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            if load == CarryLoad::EMPTY {
                continue;
            }
            let Some(faction) = self.soldiers.faction(unit) else {
                continue;
            };
            let Some(address) = self.soldiers.address(unit) else {
                continue;
            };
            let Some(site) = self.settlements.on_tile(address) else {
                continue;
            };
            let Some(host) = self.settlements.faction(site) else {
                continue;
            };
            if host == faction {
                continue;
            }
            let Some(slot) = self.settlements.slot_of(site) else {
                continue;
            };
            let Ok((proposer, responder)) = self.live_orientation(faction, host) else {
                continue;
            };
            let (Some(index), Some(row)) = (
                self.trade.index_of(proposer, responder),
                self.trade.row(proposer, responder),
            ) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let owes_as_proposer = faction == proposer;
            // Only a resource is carried. A land set and a relation step
            // apply in the pass that follows the carriers, and a load
            // delivered against either would pay a debt that no unit can
            // pay.
            let carried = if owes_as_proposer {
                row.proposer_side_is_carried()
            } else {
                row.responder_side_is_carried()
            };
            if !carried {
                continue;
            }
            let (kind, owed) = if owes_as_proposer {
                (row.give_kind, row.owed_by_proposer())
            } else {
                (row.take_kind, row.owed_by_responder())
            };
            if owed == 0 {
                continue;
            }
            let Some(kind) = ResourceKind::from_u8(kind) else {
                continue;
            };
            if load.of(kind).0 == 0 {
                continue;
            }
            found.push(ContractDelivery {
                unit,
                site: slot,
                kind,
                row: index,
                owes_as_proposer,
            });
        }
        found
    }

    /// Transfers what every contract carrier may deliver this tick.
    fn carry_contract_loads(&mut self, threads: usize) -> Result<(), StepError> {
        let carriers = self.contract_carriers(threads);
        if carriers.is_empty() {
            return Ok(());
        }
        let keys: Vec<BoundedKey> = carriers
            .iter()
            .map(|delivery| BoundedKey::new(u64::from(delivery.site), delivery.unit.to_bits()))
            .collect();
        let ceiling = u64::from(self.settlements.slot_count().saturating_sub(1));
        let order = gather_order_of(&keys, ceiling)?;

        for position in order {
            let delivery = carriers[position as usize];
            let Some(row) = self.trade.row_at(delivery.row) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let owed = if delivery.owes_as_proposer {
                row.owed_by_proposer()
            } else {
                row.owed_by_responder()
            };
            if owed == 0 {
                continue;
            }
            let Some(load) = self.soldiers.carry(delivery.unit) else {
                continue;
            };
            let carried = load.of(delivery.kind).0;
            if carried == 0 {
                continue;
            }
            let commodity = WORK_COMMODITY[delivery.kind.index()];
            let Some(held) = self
                .settlements
                .store_column()
                .get(delivery.site as usize)
                .and_then(|store| store.quantity(commodity))
            else {
                continue;
            };
            // The room of the store, in whole units. The subtract cannot go
            // below zero because the ceiling is the largest value the scale
            // holds.
            let room = sim_math::sub(Fix32::MAX, held).to_int_floor();
            let moved = carried.min(owed).min(u32::try_from(room).unwrap_or(0));
            if moved == 0 {
                continue;
            }
            // The conversion is exact in both directions: a whole number that
            // the room admits fits the scale, and the scale holds it with no
            // fractional part.
            let quantity = Fix32::from_int(i16::try_from(moved).unwrap_or(i16::MAX));
            let moved = u32::try_from(quantity.to_int_floor()).unwrap_or(0);
            if moved == 0 {
                continue;
            }
            let after = sim_math::add(held, quantity);
            if !self.set_store_quantity(delivery.site, commodity, after) {
                continue;
            }
            self.soldiers
                .take_carry(delivery.unit, delivery.kind, Amount(moved));
            // The delivered account links the carry account to the store
            // account. A transfer that forgot it would break the conservation
            // check on the frame of the first contract delivery.
            self.delivered[delivery.kind.index()] += u64::from(moved);
            let paid = match self.trade.row_at_mut(delivery.row) {
                Some(entry) => {
                    if delivery.owes_as_proposer {
                        entry.given = entry.given.saturating_add(moved);
                    } else {
                        entry.taken = entry.taken.saturating_add(moved);
                    }
                    if entry.is_paid() {
                        entry.status = TRADE_SETTLED;
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
            if paid {
                let (proposer, responder) = self.pair_of(delivery.row);
                self.say(proposer, responder, ACT_SETTLE, TRADE_SETTLED);
                // A contract delivered in full warms both directions.
                self.relations
                    .on_contract_delivered(self.tick, proposer, responder);
            }
        }
        Ok(())
    }

    /// Returns the ordered pair that one row index names.
    fn pair_of(&self, index: usize) -> (FactionId, FactionId) {
        let width = (self.trade.factions() as usize).max(1);
        let proposer = (index / width) as u16;
        let responder = (index % width) as u16;
        (FactionId(proposer), FactionId(responder))
    }

    /// Fails every contract that reached its deadline with a debt.
    ///
    /// The walk is over the plane in pair order, so it names no thread and it
    /// reads no hash order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn fail_overdue_contracts(&mut self) {
        let tick = self.tick;
        let mut failed: Vec<(usize, bool, bool, u32)> = Vec::new();
        for (index, row) in self.trade.rows().iter().enumerate() {
            if !row.is_bound() || row.deadline.0 > tick.0 {
                continue;
            }
            // A side that no unit carries cannot be short on its own. It
            // waits on the other side, so only a carried side owes at the
            // deadline.
            failed.push((
                index,
                row.owed_by_proposer() > 0 && row.proposer_side_is_carried(),
                row.owed_by_responder() > 0 && row.responder_side_is_carried(),
                row.term,
            ));
        }
        for (index, proposer_owes, responder_owes, term) in failed {
            let (proposer, responder) = self.pair_of(index);
            if let Some(entry) = self.trade.row_at_mut(index) {
                entry.status = TRADE_DEFAULTED;
            }
            // The party that was owed cools toward the party that defaulted.
            if proposer_owes {
                self.relations.on_contract_failed(tick, responder, proposer);
            }
            if responder_owes {
                self.relations.on_contract_failed(tick, proposer, responder);
            }
            let until = Tick(tick.0.saturating_add(u64::from(term)));
            // The defaulting party loses the direction it would ask on again,
            // for as long as the contract ran. The duration comes from the
            // contract, so no balance figure decides it.
            if proposer_owes {
                if let Some(door) = self.trade.row_mut(proposer, responder) {
                    door.closed_until = until;
                }
            }
            if responder_owes {
                if let Some(door) = self.trade.row_mut(responder, proposer) {
                    door.closed_until = until;
                }
            }
            self.say(proposer, responder, ACT_DEFAULT, TRADE_DEFAULTED);
        }
    }

    /// Returns every live unit that stands on the tile of its home site, with
    /// the slot of that site.
    ///
    /// The walk is over the unit slots and it reads no derived structure. A
    /// unit with no home, a unit whose home is gone, and a unit that stands
    /// somewhere else all give nothing.
    ///
    /// The order is the slot order, which does not depend on the thread
    /// count. The caller sorts it on a total key before it transfers
    /// anything, so nothing downstream reads this order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn carriers_at_home(&self, threads: usize) -> Vec<(Entity, u32)> {
        let _ = threads;
        let mut found = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(Some(home)) = self.soldiers.home(unit) else {
                continue;
            };
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            if load == CarryLoad::EMPTY {
                continue;
            }
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            let stands_at_home = self
                .settlements
                .tile_column()
                .get(home as usize)
                .is_some_and(|site_tile| *site_tile == tile);
            if stands_at_home {
                found.push((unit, home));
            }
        }
        found
    }

    /// Writes a quantity into the store of one site slot and keeps the
    /// account.
    ///
    /// **The account and the store are written in one call.** They are two
    /// copies of one total, and the conservation check is what fails when they
    /// disagree, so a path that wrote one without the other would break that
    /// check on every later frame.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    fn set_store_quantity(&mut self, slot: u32, commodity: CommodityId, after: Fix32) -> bool {
        let index = commodity.0 as usize;
        let Some(store) = self
            .settlements
            .store_update()
            .stores
            .get_mut(slot as usize)
        else {
            return false;
        };
        let Some(before) = store.quantity(commodity) else {
            return false;
        };
        if !store.set_quantity(commodity, after) {
            return false;
        }
        let change = sim_math::sub(after, before);
        self.store_account[index] = sim_math::accumulate(self.store_account[index], change);
        true
    }

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
            let rate = upgrade::gather_rate_with(
                GATHER_RATE,
                self.upgrades.standing(first.tile, &self.upgrade_table),
            );
            // Wet ground yields more. The weather field is read once for the
            // whole segment, beside the deposit and the upgrade rate, and it
            // is read at the level 1 cell that covers the tile because that
            // is where the weather lives.[^2]
            //
            // The reader takes the ground as the solve of the previous frame
            // left it. The solve of this frame runs at the end of the step,
            // after level 1 rebuilds.[^3]
            //
            // [^2]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
            // [^3]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
            let rate = rate.saturating_add(self.wet_bonus(first.tile));
            let mut granted = 0u32;
            for position in &order[at..end] {
                if left == 0 {
                    break;
                }
                let intent = intents[*position as usize];
                // The type of the unit scales the tile rate and caps the
                // load. A gather rate of zero takes nothing, and a load at
                // the carry capacity takes nothing more.[^4]
                //
                // **A unit that cannot gather keeps its order and takes
                // nothing.** The order is not refused here, because the
                // choice pass writes the same column from inside the step,
                // and a type can change after an order is given. The boundary
                // verb refuses the order for such a unit, so a caller learns
                // at once; the pass is what holds when the caller is the
                // engine.
                //
                // [^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
                let row = self.unit_types.row(intent.unit_type);
                let unit_rate = sim_math::scale_amount(rate, row.gather_rate);
                let held = self
                    .soldiers
                    .carry(intent.unit)
                    .map_or(0i64, |load| load.total().0);
                let room = i64::from(row.carry_capacity).saturating_sub(held);
                let room = u32::try_from(room).unwrap_or(0);
                let amount = unit_rate.min(left).min(room);
                if amount == 0 {
                    continue;
                }
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
        // The ledger comes out of the world for the merge, for the reason the
        // recovery pass takes it out: the merge ages each entry to this tick
        // first, and ageing reads the weather and the upgrade map.
        let mut depletion = core::mem::take(&mut self.depletion);
        depletion.merge_ascending(&run, tick, &|tile| self.tile_ground(tile));
        self.depletion = depletion;
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
    /// sum over the builders of a whole-number rate scaled by the build rate
    /// of each builder's type, so it is the same whatever order the threads
    /// produced the intents in.[^2] [^3]
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
        let intents = build_intents(
            &self.soldiers,
            &self.holding,
            plan::Ground {
                grid: self.grid,
                terrain: self.terrain,
                upgrades: &self.upgrades,
                table: &self.upgrade_table,
            },
            &self.plan,
            threads,
        )?;
        if intents.is_empty() {
            // The merge is still called, so the visit count describes this
            // tick rather than the last one that built anything.
            self.upgrades.merge_ascending(&[], &self.upgrade_table);
            return Ok(());
        }

        let keys: Vec<BoundedKey> = intents
            .iter()
            .map(|intent| {
                BoundedKey::new(
                    upgrade::site_key(intent.tile, intent.category),
                    intent.unit.to_bits(),
                )
            })
            .collect();
        let ceiling = upgrade::key_ceiling(self.grid.tile_count());
        let order = build_order_of(&keys, ceiling)?;

        // The key packs the tile above the kind, so the sorted order is tile
        // major and every builder of one tile sits in one run.
        let mut run: Vec<(TileIdx, UpgradeCategory, i64)> = Vec::new();
        let mut at = 0usize;
        while at < order.len() {
            let tile = intents[order[at] as usize].tile;
            let mut end = at;
            while end < order.len() && intents[order[end] as usize].tile == tile {
                end += 1;
            }
            let held = self.upgrades.at(tile).map(|site| site.category);
            let winner = held.unwrap_or(intents[order[at] as usize].category);
            // Each builder adds the builder rate scaled by the build rate of
            // its type. A build rate of zero adds nothing, so a unit that
            // cannot build keeps its order and moves no site. The sum is
            // integer addition, so it is the same in any order.[^5]
            //
            // [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
            let work = order[at..end]
                .iter()
                .map(|position| intents[*position as usize])
                .filter(|intent| intent.category == winner)
                .fold(0i64, |total, intent| {
                    total.saturating_add(build_contribution(self.unit_types.row(intent.unit_type)))
                });
            if work > 0 {
                run.push((tile, winner, work));
            }
            at = end;
        }
        // The level that stands at each tile of the run, read before the
        // merge. The raise below compares it against the level the merge
        // produced, so nothing else states which build finished.
        let before: Vec<u8> = run
            .iter()
            .map(|(tile, _, _)| {
                self.upgrades
                    .at(*tile)
                    .map_or(upgrade::NO_LEVEL, |site| site.level)
            })
            .collect();
        self.upgrades.merge_ascending(&run, &self.upgrade_table);
        self.publish_the_finished_levels();
        self.lodge_the_finished_levels(&run, &before);
        Ok(())
    }

    /// Writes one event for each level that the merge just raised.
    ///
    /// **A level that finished is a moment, and this is the one place that
    /// says so.** The census counts how many upgrades stand and how many of
    /// them claim a wonder. A count says that a thing happened and it does
    /// not say where, whose it is, or which category rose.
    ///
    /// The merge gathers the raises in ascending tile order, so the log is
    /// ordered by tile within the tick. Nothing here reads a thread
    /// completion order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn publish_the_finished_levels(&mut self) {
        if self.upgrades.last_merge_raised().is_empty() {
            return;
        }
        let tick = self.tick;
        let holders = self.holding.holders();
        let written: Vec<UpgradeFinished> = self
            .upgrades
            .last_merge_raised()
            .iter()
            .map(|site| {
                let holder = holders
                    .get(site.tile.0 as usize)
                    .copied()
                    .unwrap_or(Holder::NOBODY);
                UpgradeFinished::new(tick, site.tile, holder, site.category.0, site.level)
            })
            .collect();
        self.finished_log.extend_from_slice(&written);
    }

    /// Takes condition from every upgrade that the weather or an army wears.
    ///
    /// **This is the only sink an upgrade has that a caller does not drive by
    /// hand.** Before it existed the engine removed no upgrade and damaged
    /// none, so every road, terrace, store and wall ever built stood for the
    /// rest of the run and the built world only accumulated.
    ///
    /// Two causes take condition, and the pass sums them.
    ///
    /// A storm over the tile takes a fixed amount for each tick. The pass
    /// reads the wetness of the level 1 cell that holds the tile, which is
    /// the reader the gather resolve already uses, so the weather is read in
    /// one way and not two.[^1] [^2] The weather solve runs later in the
    /// step, so this pass reads the field the previous step left.
    ///
    /// A hostile unit on the tile takes a fixed amount for each unit. A unit
    /// is hostile when the faction that holds the tile is at war with the
    /// faction of the unit. An upgrade on ground nobody holds therefore wears
    /// only from the weather, because no faction owns it to be at war with.
    ///
    /// The pass walks the sites and the units on their tiles. It takes no
    /// grid and no tile count, so a world in which nobody built does no work
    /// here, at any tile count.[^3]
    ///
    /// The walk is serial and it runs in ascending tile order, which the map
    /// holds its entries in. Nothing here reads a thread completion order,
    /// and the sum over the units of one tile is integer addition, so it is
    /// the same in any order.[^4] [^5] The pass makes no random draw: wear is
    /// a rate and not a chance.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    fn wear_upgrades(&mut self) {
        if self.upgrades.is_empty() {
            return;
        }
        let holders = self.holding.holders();
        let mut cursor = self.bridge.tile_cursor();
        // The sites are held in ascending tile order, so the run this builds
        // is in ascending tile order and the tile reader walks forward.
        let worn: Vec<(TileIdx, i64, u8, Holder)> = self
            .upgrades
            .sites()
            .iter()
            .filter(|site| site.is_complete())
            .map(|site| {
                let tile = site.tile;
                let storm = match self.weather_cell_of(tile) {
                    Some(cell) if self.weather.cell_is_wet(cell) => {
                        upgrade::WEATHER_WEAR_FOR_EACH_TICK
                    }
                    _ => 0,
                };
                let holder = holders
                    .get(tile.0 as usize)
                    .copied()
                    .unwrap_or(Holder::NOBODY);
                let army = match holder.faction() {
                    Some(owner) => {
                        let hostile = self
                            .bridge
                            .units_on_tile(&mut cursor, tile)
                            .iter()
                            .filter(|unit| {
                                self.soldiers
                                    .faction(**unit)
                                    .is_some_and(|guest| self.relations.war_between(owner, guest))
                            })
                            .count() as i64;
                        hostile.saturating_mul(upgrade::ARMY_WEAR_FOR_EACH_UNIT)
                    }
                    None => 0,
                };
                let cause = match (storm > 0, army > 0) {
                    (true, true) => WEAR_CAUSE_BOTH,
                    (true, false) => WEAR_CAUSE_WEATHER,
                    (false, true) => WEAR_CAUSE_ARMY,
                    (false, false) => 0,
                };
                (tile, storm.saturating_add(army), cause, holder)
            })
            .filter(|(_, taken, _, _)| *taken > 0)
            .collect();
        let run: Vec<(TileIdx, i64)> = worn
            .iter()
            .map(|(tile, taken, _, _)| (*tile, *taken))
            .collect();
        self.upgrades.wear_ascending(&run);
        // **An upgrade that collapsed leaves nothing behind, so the event is
        // the only record of it.** The removed sites come back in ascending
        // tile order and the run is in ascending tile order, so the two walk
        // together and the log is ordered by tile within the tick. Nothing
        // here reads a thread completion order.[^6]
        //
        // [^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        let removed = self.upgrades.last_wear_removed();
        if !removed.is_empty() {
            let tick = self.tick;
            let mut there = 0usize;
            let mut written = Vec::with_capacity(removed.len());
            for site in removed {
                while there < worn.len() && worn[there].0 .0 < site.tile.0 {
                    there += 1;
                }
                let (cause, holder) = worn
                    .get(there)
                    .map_or((0, Holder::NOBODY), |entry| (entry.2, entry.3));
                written.push(UpgradeCollapsed::new(
                    tick,
                    site.tile,
                    holder,
                    site.category.0,
                    site.level,
                    cause,
                ));
            }
            self.collapsed_log.extend_from_slice(&written);
        }
    }

    /// Raises the housing of a settlement for each level that the merge
    /// finished.
    ///
    /// **This is the one place that composes an upgrade row and the housing
    /// of a site.** The row states the housing that one level adds, the
    /// settlement column stores it, and no second declaration of the number
    /// exists.[^1] The record asks for a stored field rather than a
    /// composition derived on read, so the raise is written once when the
    /// level rises.[^2]
    ///
    /// A finished level raises the settlement on its own tile, or on one of
    /// the six tiles beside it. A level that stands beside no settlement
    /// houses nobody, which is the same rule the store raise applies.
    ///
    /// The pass names no category. It reads the housing column of the row
    /// the entry indexes.[^3]
    ///
    /// The run arrives in ascending tile order and this loop is serial, so
    /// two lodgings beside one settlement add in tile order and in no other.[^4]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn lodge_the_finished_levels(
        &mut self,
        run: &[(TileIdx, UpgradeCategory, i64)],
        before: &[u8],
    ) {
        for ((tile, _, _), stood) in run.iter().zip(before.iter()) {
            let Some(site) = self.upgrades.at(*tile) else {
                continue;
            };
            if site.level <= *stood {
                continue;
            }
            // Every level the merge crossed adds the housing of its own row.
            let mut housing = 0u32;
            let mut level = *stood + 1;
            while level <= site.level {
                if let Some(row) = self.upgrade_table.row(site.category, level) {
                    housing = housing.saturating_add(row.housing_change);
                }
                level += 1;
            }
            if housing == 0 {
                continue;
            }
            let Some(address) = self.grid.address_of(*tile) else {
                continue;
            };
            let Some(settlement) = core::iter::once(Some(address))
                .chain(self.grid.neighbours(address))
                .find_map(|place| place.and_then(|near| self.settlements.on_tile(near)))
            else {
                continue;
            };
            let held = self.settlements.housing(settlement).unwrap_or(0);
            self.settlements
                .set_housing(settlement, held.saturating_add(housing));
        }
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

    /// Promotes the units that earned it, on the character schedule.
    ///
    /// The budget is the headroom of the character arena: the ceiling of the
    /// declared tier less the characters alive now. The arena refuses a
    /// create beyond its capacity whatever this says, so the ceiling has one
    /// enforcement site and this is the cut at the rank.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D4. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    fn promote(&mut self, threads: usize) -> Result<(), StepError> {
        self.promoted_log.clear();
        if !self.character_schedule.due(self.tick) {
            return Ok(());
        }
        let headroom = CharacterArena::ceiling().saturating_sub(self.characters.len());
        let budget = self.promotion_budget.min(headroom);
        self.promoted_log = promotion::promote(
            &mut self.soldiers,
            &mut self.characters,
            self.config.seed,
            self.tick,
            budget,
            threads,
        )?;
        Ok(())
    }

    /// Runs the rate pass of one frame.
    ///
    /// The stored table holds the base rate of each site. The pass derives an
    /// effective rate from it and from the world, and hands the derived table
    /// to the apply function. The stored table is not written.[^1]
    ///
    /// The derived table is scratch. It holds no fact of its own, so it stays
    /// out of the state hash, and its four inputs are stored and already
    /// enter it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decisions D1 and D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    fn apply_rates(&mut self, threads: usize) -> Result<(), StepError> {
        self.shortfall_log.clear();
        self.rates.open_to(self.settlements.slot_count());
        let schedule = self.schedule;
        let tick = self.tick;
        // The table is taken out of the world so that the fill may read the
        // world, and it is put back below. The take leaves the field empty
        // for the length of this call and nothing else reads it.
        let mut effective = core::mem::take(&mut self.effective_rates);
        self.fill_effective_rates(&mut effective);
        let outcome = crate::rates::apply(
            schedule,
            tick,
            &effective,
            self.settlements.store_update(),
            threads,
        );
        self.effective_rates = effective;
        let pass = outcome?;
        for (index, account) in self.store_account.iter_mut().enumerate() {
            let net = pass
                .ledger
                .net(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            *account = sim_math::combine(*account, net);
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
            schedule.per_application(rule.ration()),
            cohort::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &pass.shares,
            &self.cohorts,
            self.soldiers.need_update(),
            threads,
        )?;
        Ok(())
    }

    /// Returns the queue of one site, in queue position order.
    ///
    /// Returns `None` when the identity names no site that stands. The order
    /// is the order the entries were queued, and nothing reorders them.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub fn site_queue(&self, site: Entity) -> Option<&[QueueEntry]> {
        let slot = self.settlements.slot_of(site)?;
        Some(self.queues.entries_of(slot))
    }

    /// Orders the queue of one site.
    ///
    /// **This is the one verb that reaches a queue.** A Python caller, the
    /// built-in controller and a learner all call it, and no other path
    /// writes an entry.[^1] The engine holds the mechanism, the bound and the
    /// refusals, and it holds no rule about what to queue.
    ///
    /// A push puts one entry of the named type at the back. A clear takes the
    /// entry at one position out and closes the gap, and the work the store
    /// already paid for is lost.
    ///
    /// **Every refusal is counted.** A refused order changes nothing, and the
    /// count of the refusals sits beside the count of what the queue
    /// produced, so a watcher reading a queue that never moves can tell the
    /// two apart.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no site that stands, when the
    /// site belongs to another faction, when the number names no row of the
    /// unit type table, when the queue already holds its bound, and when the
    /// position holds no entry.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    pub fn order_site_queue(
        &mut self,
        faction: FactionId,
        site: Entity,
        order: QueueOrder,
    ) -> Result<(), QueueError> {
        let outcome = self.take_queue_order(faction, site, order);
        if outcome.is_err() {
            self.queues.count_refused_at_the_verb();
        }
        outcome
    }

    /// Takes one queue order, and states every refusal.
    ///
    /// The verb above counts what this refuses. The two are apart so that the
    /// count sits at one place and no path can refuse without counting.
    fn take_queue_order(
        &mut self,
        faction: FactionId,
        site: Entity,
        order: QueueOrder,
    ) -> Result<(), QueueError> {
        let (Some(slot), Some(owner)) = (
            self.settlements.slot_of(site),
            self.settlements.faction(site),
        ) else {
            return Err(QueueError::NoSuchSite(site));
        };
        if owner != faction {
            return Err(QueueError::SiteBelongsToAnother {
                owner,
                asked: faction,
            });
        }
        self.queues.open_to(self.settlements.slot_count());
        match order {
            QueueOrder::Push(unit_type) => {
                if UnitTypeId::from_u8(unit_type.0).is_none() {
                    return Err(QueueError::TypeAboveCeiling(unit_type.0));
                }
                self.queues.push(slot, unit_type)
            }
            QueueOrder::Clear(position) => self.queues.remove(slot, position).map(|_| ()),
        }
    }

    /// Returns the build cost row of one unit type.
    #[must_use]
    pub const fn build_cost(&self, unit_type: UnitTypeId) -> BuildCostRow {
        self.build_costs.row(unit_type)
    }

    /// Writes the build cost row of one unit type.
    ///
    /// The costs are data that the world is built with, in the way the unit
    /// type table and the upgrade table are.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row of the unit type table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    pub const fn define_build_cost(
        &mut self,
        unit_type: u8,
        row: BuildCostRow,
    ) -> Result<(), QueueError> {
        self.build_costs.define(unit_type, row)
    }

    /// Returns the entries one site may hold in its queue.
    ///
    /// The bound is a parameter of the world and never a function of the
    /// population.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub const fn queue_bound(&self) -> usize {
        self.queues.bound()
    }

    /// Sets the entries one site may hold in its queue.
    ///
    /// **A bound of zero turns the queue off.** The verb then refuses every
    /// push, no site holds an entry, and no unit is built. A test that wants
    /// the engine to leave its units alone sets it.
    ///
    /// Returns `false` when the bound is above the width of the stored block,
    /// which the balance register holds as a row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the production queue, the queue bound row. `docs/reference/balance.md`
    pub const fn set_queue_bound(&mut self, bound: usize) -> bool {
        self.queues.set_bound(bound)
    }

    /// Returns the housing that stands at a site.
    ///
    /// The answer is a quantity of housing and not a count of people. It
    /// follows from what has been built at the site, and the ground never
    /// sets it.[^1]
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_housing(&self, site: Entity) -> Option<u32> {
        self.settlements.housing(site)
    }

    /// Writes the housing that stands at a site.
    ///
    /// Returns `false` when the identity names no live site.
    pub fn set_site_housing(&mut self, site: Entity, housing: u32) -> bool {
        self.settlements.set_housing(site, housing)
    }

    /// Returns how many units live at a site.
    ///
    /// **Nothing stores this count.** The answer is the sum of the cohort
    /// rows of the site, and the cohort table derives every one of those
    /// rows from the home column of the unit arena.[^1]
    ///
    /// The call costs the faction ceiling, which is a structural constant of
    /// the project. It never walks the population.
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_residents(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        // A slot the derived table has not covered holds no counted
        // resident. That is the answer the table gives, and it is the answer
        // the growth stage reads. A caller that spawned a unit and did not
        // step reads it before the frame settled, in the way every derived
        // structure of this engine behaves, and one public check says so.[^1]
        //
        // [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        Some(self.cohorts.residents(slot).unwrap_or(0))
    }

    /// Returns the free places of a site.
    ///
    /// The free places are the people the housing holds, less the residents
    /// the site has. A site above its housing has no free place, and the
    /// answer is zero rather than a value below zero.[^1]
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_free_places(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        Some(self.free_places_of(slot))
    }

    /// Returns the free places of one settlement slot.
    fn free_places_of(&self, slot: u32) -> u32 {
        let housing = self
            .settlements
            .housing_column()
            .get(slot as usize)
            .copied()
            .unwrap_or(0);
        let residents = self.cohorts.residents(slot).unwrap_or(0);
        growth::free_places(housing, self.housing_per_person, residents)
    }

    /// Returns the housing that one person takes.
    #[must_use]
    pub const fn housing_per_person(&self) -> u32 {
        self.housing_per_person
    }

    /// Sets the housing that one person takes.
    ///
    /// A value of zero leaves no site with a free place, so no site grows.
    pub const fn set_housing_per_person(&mut self, housing: u32) {
        self.housing_per_person = housing;
    }

    /// Returns the housing that a founded site starts with.
    #[must_use]
    pub const fn founding_housing(&self) -> u32 {
        self.founding_housing
    }

    /// Sets the housing that a founded site starts with.
    ///
    /// The value reaches the sites founded after the call. It changes no site
    /// that already stands.
    pub const fn set_founding_housing(&mut self, housing: u32) {
        self.founding_housing = housing;
    }

    /// Returns the store that one birth costs.
    #[must_use]
    pub const fn food_per_birth(&self) -> [Fix32; COMMODITY_COUNT] {
        self.food_per_birth
    }

    /// Sets the store that one birth costs.
    pub const fn set_food_per_birth(&mut self, cost: [Fix32; COMMODITY_COUNT]) {
        self.food_per_birth = cost;
    }

    /// Returns the chance that one proposal becomes a birth.
    #[must_use]
    pub const fn birth_chance(&self) -> Fix32 {
        self.birth_chance
    }

    /// Sets the chance that one proposal becomes a birth.
    ///
    /// A chance at or above one makes every proposal a birth. A chance at or
    /// below zero makes none.
    pub const fn set_birth_chance(&mut self, chance: Fix32) {
        self.birth_chance = chance;
    }

    /// Returns when the growth stage acts.
    #[must_use]
    pub const fn growth_schedule(&self) -> RateSchedule {
        self.growth_schedule
    }

    /// Sets when the growth stage acts.
    pub const fn set_growth_schedule(&mut self, schedule: RateSchedule) {
        self.growth_schedule = schedule;
    }

    /// Returns what growth took out of the stores, for each commodity.
    ///
    /// The total is cumulative over the life of the world. A caller that
    /// states the conservation of the stores reads it, because growth is a
    /// sink that neither the rate ledger nor the draw ledger holds.
    #[must_use]
    pub const fn growth_ledger(&self) -> [Accum; COMMODITY_COUNT] {
        self.growth_ledger
    }

    /// Returns how many people the growth stage added on the last tick.
    ///
    /// The count is a census of one tick. The growth stage clears it before
    /// it acts, so a zero says that the world grew nobody and not that the
    /// world never grew.
    #[must_use]
    pub const fn births(&self) -> u32 {
        self.births
    }

    /// Returns when the queue advance acts.
    #[must_use]
    pub const fn queue_schedule(&self) -> RateSchedule {
        self.queue_schedule
    }

    /// Sets when the queue advance acts.
    pub const fn set_queue_schedule(&mut self, schedule: RateSchedule) {
        self.queue_schedule = schedule;
    }

    /// Sets the quantity of one good that one advance of a queue costs.
    ///
    /// Returns `false` when the commodity is outside the set.
    pub fn set_queue_charge(&mut self, commodity: CommodityId, quantity: Fix32) -> bool {
        self.queues.set_charge(commodity, quantity)
    }

    /// Returns how many units the queues produced on the last step.
    #[must_use]
    pub const fn queue_produced(&self) -> u32 {
        self.queues.produced()
    }

    /// Returns how many finished entries the last advance refused, because
    /// the site held no resident to spend.
    #[must_use]
    pub const fn queue_refused_without_a_person(&self) -> u32 {
        self.queues.refused_without_a_person()
    }

    /// Returns how many finished entries the last advance refused, because
    /// the store could not pay the goods.
    ///
    /// The two refusals are counted apart, because they mean different things
    /// to a watcher and to a learner.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub const fn queue_refused_without_goods(&self) -> u32 {
        self.queues.refused_without_goods()
    }

    /// Returns how many queue orders the verb refused since the last advance.
    #[must_use]
    pub const fn queue_refused_at_the_verb(&self) -> u32 {
        self.queues.refused_at_the_verb()
    }

    /// Reports whether the store of one slot holds every good of a cost.
    fn store_holds(&self, slot: u32, goods: &[Fix32; COMMODITY_COUNT]) -> bool {
        let Some(store) = self.settlements.store_column().get(slot as usize) else {
            return false;
        };
        goods.iter().enumerate().all(|(index, wanted)| {
            store
                .quantity(CommodityId(index as u16))
                .is_some_and(|held| held.0 >= wanted.0)
        })
    }

    /// Takes every good of a cost out of the store of one slot.
    ///
    /// The caller reads the store first, so no subtract here goes below zero.
    /// The write goes through the one path that keeps the account of the
    /// stores, so the conservation check still balances.
    fn take_from_store(&mut self, slot: u32, goods: &[Fix32; COMMODITY_COUNT]) {
        for (index, wanted) in goods.iter().enumerate() {
            let commodity = CommodityId(index as u16);
            let Some(held) = self
                .settlements
                .store_column()
                .get(slot as usize)
                .and_then(|store| store.quantity(commodity))
            else {
                continue;
            };
            self.set_store_quantity(slot, commodity, sim_math::sub(held, *wanted));
        }
    }

    /// Grows the population of every site, and returns nothing.
    ///
    /// # What it does
    ///
    /// The store of a site sets a rate, and the rate proposes a birth. The
    /// free places of the site admit the proposals, in ordinal order, until
    /// no place is free. A refused proposal is discarded and it is not
    /// carried to the next application.[^1] [^2]
    ///
    /// **The housing is a bound and not a factor.** A site at its housing
    /// grows nobody, however much food it holds. A site with free places
    /// grows at the rate its store sets, and the free places never scale that
    /// rate.[^2]
    ///
    /// An admitted proposal costs the store, spawns one unit at the address
    /// of the site, and writes the residence of that unit. The unit carries
    /// the type the spawn path gives, which is the worker row. This stage
    /// names no unit type.[^3]
    ///
    /// # Where it runs, and why
    ///
    /// It runs after the shortage scan and before the queue advance. The
    /// step states both positions.
    ///
    /// # What it costs
    ///
    /// The stage visits the settlements in slot order and reads a fixed
    /// number of proposals at each one. It walks no unit and it searches
    /// nothing, so its cost follows the settlements and the faction ceiling
    /// and never the population.[^4]
    ///
    /// # Determinism
    ///
    /// The sites apply in slot order, and the proposals of one site apply in
    /// ordinal order. Every draw is keyed on the system, the tick, the site
    /// and the ordinal of the proposal.[^5] No thread takes part.
    ///
    /// # References
    ///
    /// [^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    /// [^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    /// [^3]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^4]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    fn grow(&mut self) {
        // The count is a census of one tick, and this stage is where the
        // tick starts for it. A world that cannot grow and a world that
        // chose not to both read zero here, and the free places say which.
        //
        // The run total is folded here, immediately before the per-tick
        // count is emptied, because this is the one site that empties
        // it.[^7]
        //
        // [^7]: Findings register, FND-498. `docs/FINDINGS.md`
        self.census.births += i64::from(self.births);
        self.births = 0;
        if !self.growth_schedule.due(self.tick) {
            return;
        }
        let tick = self.tick.0;
        let seed = self.config.seed;
        let cost = self.food_per_birth;
        let chance = self.birth_chance;
        let mut grew = false;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            // **The housing admits, and it never scales the rate.** The free
            // places are read once for the site, and each birth takes one of
            // them. A site with no free place is done here, whatever its
            // store holds.
            let mut free = self.free_places_of(slot);
            if free == 0 {
                continue;
            }
            let Some(store) = self.settlements.store(site) else {
                continue;
            };
            let mut held = [Fix32::ZERO; COMMODITY_COUNT];
            for (index, quantity) in held.iter_mut().enumerate() {
                *quantity = store
                    .quantity(CommodityId(index as u16))
                    .expect("the index is inside the commodity set");
            }
            let proposals = growth::proposals(&held, &cost);
            let (Some(address), Some(faction)) = (
                self.settlements.address(site),
                self.settlements.faction(site),
            ) else {
                continue;
            };
            for index in 0..proposals {
                if free == 0 {
                    break;
                }
                if !growth::proposal_takes(seed, tick, slot, index, chance) {
                    continue;
                }
                // The rate counted what the store could pay when the stage
                // read it. Each birth spends, so the check runs again for
                // each one rather than once for the site.
                if !self.store_holds(slot, &cost) {
                    break;
                }
                let Ok(person) = self.spawn_soldier(address, faction) else {
                    break;
                };
                self.set_home_site(person, Some(site));
                self.take_from_store(slot, &cost);
                for (index, quantity) in cost.iter().enumerate() {
                    self.growth_ledger[index] =
                        sim_math::accumulate(self.growth_ledger[index], *quantity);
                }
                free -= 1;
                self.births += 1;
                grew = true;
            }
        }
        if !grew {
            return;
        }
        // The cohort table summarises the home column, and this stage changed
        // that column. A table left stale would state a headcount that no
        // unit backs, and the invariant check refuses that state.[^6]
        //
        // [^6]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }

    /// Advances the front entry of every queue, and applies what finished.
    ///
    /// # Where it runs, and why
    ///
    /// The stage runs after the shortage scan of this frame and before the
    /// barrier that follows it. It reads the store, so it runs after the rate
    /// pass and after the consumption pass, which are what move a quantity
    /// into and out of a store in this frame.[^1] [^2] It removes units and
    /// adds units, so it is a structural change, and the barrier below it is
    /// the barrier of that change.[^3]
    ///
    /// **It runs after the shortage scan and not before it.** The scan holds
    /// a plane of the slots it ends. A stage that freed a slot and filled it
    /// again before the scan applied would give the scan a live unit that it
    /// never marked.
    ///
    /// # The cost
    ///
    /// The first pass visits the sites and their front entries, so its cost
    /// follows the site count and never the population.[^4] **The second pass
    /// walks the unit arena once, and only on a tick where an entry
    /// finishes.** A finished entry must name the residents it takes, the
    /// residence of a unit is the home column it carries, and the engine
    /// stores no list of the residents of a site.[^5] One walk for the whole
    /// tick is the shape the controller stage already uses when it buckets
    /// the units of a faction for one command.[^6]
    ///
    /// # The order
    ///
    /// The first pass runs in site slot order. The second walks the units in
    /// ascending slot order and keeps the lowest slots of each site, so the
    /// residents an entry takes are fixed by the arena and not by a thread.
    /// The third applies in site slot order and then in queue position
    /// order.[^7]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    /// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^4]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^5]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D3. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn advance_queues(&mut self) {
        // The counts are a census of one tick, and this stage is where the
        // tick starts for them. The run total takes what they hold before
        // they are emptied.
        self.fold_queues_into_the_census();
        self.queues.clear_counts();
        if !self.queue_schedule.due(self.tick) {
            return;
        }
        self.queues.open_to(self.settlements.slot_count());
        let charge = *self.queues.charge();

        // Pass one. The sites, in slot order, and one front entry each.
        let mut finished: Vec<(u32, QueueEntry)> = Vec::new();
        for slot in 0..self.settlements.slot_count() {
            if self.settlements.entity_at(slot).is_none() {
                continue;
            }
            let Some(mut entry) = self.queues.front(slot) else {
                continue;
            };
            let work = self.build_costs.row(entry.unit_type).work;
            if entry.work < work {
                // **A queue is never free, and the store pays as the entry
                // advances.** A site whose store cannot pay makes no
                // progress, and its entry stays where it is. An entry that
                // has already reached its work costs nothing further,
                // because only an advance charges.[^8]
                //
                // [^8]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
                if !self.store_holds(slot, &charge) {
                    continue;
                }
                self.take_from_store(slot, &charge);
                // The accumulator is a whole number and it is clamped at the
                // work the entry needs, so a long build carries no credit
                // into the next entry.
                entry.work = entry.work.saturating_add(WORK_PER_ADVANCE).min(work);
                self.queues.set_front(slot, entry);
            }
            if entry.work >= work {
                finished.push((slot, entry));
            }
        }
        if finished.is_empty() {
            return;
        }

        // Pass two. One walk of the unit arena for the whole tick, which
        // buckets the residents of every site that finished an entry.
        let mut place = vec![usize::MAX; self.settlements.slot_count() as usize];
        for (index, (slot, _)) in finished.iter().enumerate() {
            place[*slot as usize] = index;
        }
        let mut residents: Vec<Vec<Entity>> = vec![Vec::new(); finished.len()];
        {
            let homes = self.soldiers.home_column();
            let owners = self.soldiers.faction_column();
            let live = self.soldiers.live_column();
            let sites = self.settlements.faction_column();
            for slot in 0..homes.len() {
                if live[slot] != 1 || homes[slot] == NO_HOME {
                    continue;
                }
                let Some(index) = place.get(homes[slot] as usize).copied() else {
                    continue;
                };
                if index == usize::MAX {
                    continue;
                }
                let (site_slot, entry) = finished[index];
                if sites[site_slot as usize] != owners[slot] {
                    continue;
                }
                let people = self.build_costs.row(entry.unit_type).people as usize;
                if residents[index].len() >= people {
                    continue;
                }
                let generation = self.soldiers.generation_of(slot as u32);
                if let Some(unit) = Entity::new(slot as u32, generation) {
                    residents[index].push(unit);
                }
            }
        }

        // Pass three. The completions apply in site slot order.
        for (index, (slot, entry)) in finished.iter().copied().enumerate() {
            let row = self.build_costs.row(entry.unit_type);
            // **A finished entry is refused when the site holds no spare
            // person or cannot pay the goods.** It is refused and not
            // discarded: the entry stays at the front of the queue. The two
            // reasons are counted apart.[^9]
            //
            // [^9]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D4 and D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
            if residents[index].len() < row.people as usize {
                self.queues.count_without_a_person();
                continue;
            }
            if !self.store_holds(slot, &row.goods) {
                self.queues.count_without_goods();
                continue;
            }
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            let (Some(address), Some(faction)) = (
                self.settlements.address(site),
                self.settlements.faction(site),
            ) else {
                continue;
            };
            let taken = std::mem::take(&mut residents[index]);
            // **The unit that leaves and the unit that arrives hold distinct
            // identities**, so no reader confuses the two. The world removes
            // each resident through its own despawn, which accounts for what
            // the unit carried, so conservation still balances.[^10]
            //
            // The despawns run before the spawn, so the arena holds a free
            // slot whatever its capacity. The spawn can therefore refuse only
            // when the row takes no person at all, and the balance register
            // fixes the people of every row above zero.[^11]
            //
            // [^10]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
            // [^11]: Balance register, the production queue, the people row. `docs/reference/balance.md`
            for unit in &taken {
                self.despawn_soldier(*unit);
            }
            let Ok(made) = self.spawn_soldier(address, faction) else {
                debug_assert!(
                    taken.is_empty(),
                    "a despawn frees a slot, so a spawn after one cannot refuse"
                );
                continue;
            };
            self.soldiers.set_unit_type(made, entry.unit_type);
            self.set_home_site(made, Some(site));
            self.take_from_store(slot, &row.goods);
            let popped = self.queues.remove(slot, 0);
            debug_assert!(popped.is_ok(), "the front entry was read above");
            self.queues.count_produced();
        }

        // The cohort table summarises the home column, and this stage changed
        // that column. A table left stale would state a headcount that no
        // unit backs, and the invariant check refuses that state.[^12]
        //
        // [^12]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }

    /// Reports whether a faction already has a leader on order.
    ///
    /// A leader is a unit whose type row carries a command reach above zero.
    /// The scan walks the queue of every site of the faction, so its cost
    /// follows the site count and the queue bound, and never the
    /// population.[^1]
    ///
    /// The gate reads the type column and no per-faction flag, in the way
    /// every other reader of the capability does.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn leader_is_on_order(&self, faction: FactionId) -> bool {
        let sites = self.settlements.faction_column();
        (0..self.settlements.slot_count()).any(|slot| {
            self.settlements.entity_at(slot).is_some()
                && sites[slot as usize] == faction
                && self
                    .queues
                    .entries_of(slot)
                    .iter()
                    .any(|entry| self.unit_types.row(entry.unit_type).command_reach > 0)
        })
    }

    /// Returns the lowest-slot site of one faction whose queue has room.
    ///
    /// The scan walks the settlements in slot order and no unit, so its cost
    /// follows the site count.
    fn controller_queue_site(&self, faction: FactionId) -> Option<Entity> {
        (0..self.settlements.slot_count())
            .find(|slot| {
                self.settlements.entity_at(*slot).is_some()
                    && self.settlements.faction_column()[*slot as usize] == faction
                    && self.queues.len_of(*slot) < self.queues.bound()
            })
            .and_then(|slot| self.settlements.entity_at(slot))
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

    /// Gives the champion of each killer faction the renown its units earned.
    ///
    /// # Why this exists
    ///
    /// **This is the one source of renown in the engine.** A win path reads
    /// the renown column, and the standing reading reports it, and no pass
    /// wrote it. The path could therefore never fire and the reading never
    /// moved. A quantity that only a reader touches states a capability the
    /// engine does not have.[^1]
    ///
    /// # What it does
    ///
    /// The contest of this frame states, for each pair, which faction felled
    /// how many units of which other faction. The killer of each pair earns
    /// one share of renown for each unit it felled, and the share is a
    /// balance value.[^2]
    ///
    /// **The renown goes to one character and not to the faction.** The
    /// reader takes the highest renown among the live characters of a
    /// faction, so renown spread over every character would never reach the
    /// target and the source would stay inert. The champion of a faction is
    /// its live character with the highest renown, and a tie goes to the
    /// lowest identity.
    ///
    /// A faction with no live character earns nothing. Renown is a property
    /// of a person, and a faction that has promoted nobody has no person to
    /// carry it.
    ///
    /// # Determinism
    ///
    /// The scan visits the character arena in slot order and replaces the
    /// champion only on a strictly greater key, so the answer is a property
    /// of the arena and never of a thread.[^3] The gains are summed into
    /// 64-bit accumulators, which combine in any order, and every value is a
    /// fixed-point value or a whole number.[^4] The renown column already
    /// enters the state hash.
    ///
    /// # Cost
    ///
    /// The scan walks the character arena, whose ceiling the character tier
    /// declares, and never the unit population.[^5]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
    /// [^2]: Balance register, the renown target. `docs/reference/balance.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^5]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    fn award_renown(&mut self) {
        let factions = usize::from(self.config.faction_count.max(1));
        let mut earned = vec![Accum(0); factions];
        let mut any = false;
        for grievance in &self.grievances {
            let Some(slot) = earned.get_mut(usize::from(grievance.killer.0)) else {
                continue;
            };
            *slot = sim_math::combine(
                *slot,
                sim_math::scale_by_count(self.balance.renown_per_fell(), grievance.count),
            );
            any = true;
        }
        if !any {
            return;
        }
        // The champion of each faction: the live character with the highest
        // renown, and the lowest identity among equals.
        let mut champions: Vec<Option<(Fix32, u64, Entity)>> = vec![None; factions];
        for character in self.characters.iter() {
            let (Some(faction), Some(renown)) = (
                self.characters.faction(character),
                self.characters.renown(character),
            ) else {
                continue;
            };
            let Some(slot) = champions.get_mut(usize::from(faction.0)) else {
                continue;
            };
            let bits = character.to_bits();
            let better = match slot {
                Some((best, best_bits, _)) => (renown.0, *best_bits) > (best.0, bits),
                None => true,
            };
            if better {
                *slot = Some((renown, bits, character));
            }
        }
        for (index, gain) in earned.iter().copied().enumerate() {
            if gain.0 == 0 {
                continue;
            }
            let Some(Some((renown, _, champion))) = champions.get(index).copied() else {
                continue;
            };
            let raised = sim_math::add(renown, sim_math::narrow(gain));
            let wrote = self.characters.set_renown(champion, raised);
            debug_assert!(wrote, "the scan above found the character live");
        }
    }

    /// Resolves every meeting of this frame and ends the units that fell.
    ///
    /// The pass marks in parallel and applies in one ascending scan of the
    /// slots, so the deaths never follow a thread completion order.[^1]
    ///
    /// It runs a fixed amount of work for the world it is given. It holds no
    /// convergence test and no time budget.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn contest(&mut self, threads: usize) -> Result<(), StepError> {
        self.fell_log.clear();
        // The structure the pass reads must describe the world it reads. The
        // movement of this frame has already passed its barrier, and nothing
        // between that barrier and here moves a unit.
        self.bridge.describes(&self.soldiers)?;
        contest::resolve(
            &self.unit_types,
            &self.relations,
            contest::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &self.soldiers,
            &self.bridge,
            &mut self.fell_plane,
            &mut self.grievances,
            threads,
        )?;
        let order = cohort::starved_order(&self.fell_plane, threads)?;
        if order.is_empty() {
            return Ok(());
        }
        let tick = self.tick;
        // A unit that fell lowers its faction toward the faction that killed
        // it. The list is summed by pair and sorted on it, so the writes
        // happen in pair order.[^3]
        //
        // [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        for grievance in &self.grievances {
            self.relations
                .on_units_fell(tick, grievance.victim, grievance.killer, grievance.count);
        }
        self.award_renown();
        for slot in order {
            let index = slot as usize;
            let tile = self.soldiers.tile_column()[index];
            let faction = self.soldiers.faction_column()[index];
            let unit_type = self.soldiers.type_column()[index];
            let generation = self.soldiers.generation_of(slot);
            let unit = Entity::new(slot, generation)
                .expect("a marked slot is live, so it holds a generation of one or more");
            self.fell_log.push(UnitFell::new(
                tick,
                unit.to_bits(),
                tile,
                faction,
                unit_type,
            ));
            let ended = self.despawn_soldier(unit);
            debug_assert!(ended, "a marked slot holds a live unit");
        }
        // The cohorts are derived from the home column of the units, and this
        // pass has just removed some of them. A table left as it was would
        // hold a headcount that no unit answers to, and the invariant check
        // refuses that state.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        Ok(())
    }

    /// Converts every unit that the influence field takes this frame.
    ///
    /// The pass marks in parallel and applies in one ascending scan of the
    /// slots, so the changes never follow a thread completion order.[^1]
    ///
    /// It runs a fixed amount of work for the world it is given. It holds no
    /// convergence test and no time budget.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn convert(&mut self, threads: usize) -> Result<(), StepError> {
        self.converted_log.clear();
        // The structure the pass walks must describe the world it reads. The
        // reap of this frame has already passed its barrier, and nothing
        // between that barrier and here moves or removes a unit.
        self.bridge.describes(&self.soldiers)?;
        let mut marks = core::mem::take(&mut self.convert_marks);
        let outcome = conversion::resolve(
            conversion::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &self.relations,
            &self.soldiers,
            &self.bridge,
            &self.influence,
            self.pyramid.layout(),
            &mut marks,
            threads,
        );
        if outcome.is_ok() {
            self.apply_converts(&marks);
        }
        self.convert_marks = marks;
        outcome?;
        // A faction change raises the arena revision, so the derived unit
        // structure no longer describes the arena. The refresh here is what
        // keeps the whole-world invariant check strong on a frame that
        // converted somebody. It returns at once on every other frame.[^3]
        //
        // [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        self.refresh_bridge()?;
        Ok(())
    }

    /// Applies a list of marks in ascending slot order, and logs each one.
    ///
    /// **This is the one place that changes the faction of a unit.** The
    /// field pass and the control plane verb both come through here, so the
    /// two cannot disagree about what a conversion does.[^1]
    ///
    /// The arena moves its own per-faction count, and this call rebuilds the
    /// cohorts, which are the other total that follows the faction of a
    /// unit.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D5. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    fn apply_converts(&mut self, marks: &[Convert]) {
        if marks.is_empty() {
            return;
        }
        let tick = self.tick;
        for mark in marks {
            let index = mark.slot as usize;
            let from = self.soldiers.faction_column()[index];
            let tile = self.soldiers.tile_column()[index];
            let generation = self.soldiers.generation_of(mark.slot);
            let Some(unit) = Entity::new(mark.slot, generation) else {
                continue;
            };
            if !self.soldiers.set_faction(unit, mark.faction) {
                continue;
            }
            // The orders end with the old faction. An order is a standing
            // instruction from the control plane that owned the unit, and a
            // unit that kept one would let a god steer the units of another
            // god.[^1]
            //
            // [^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D3. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
            self.soldiers.set_gather_order(unit, None);
            self.soldiers.set_build_order(unit, None);
            self.soldiers.set_sent(unit, None);
            // The person goes with the body. A body of one faction carrying a
            // person of another would be one allegiance in two places.[^2]
            //
            // [^2]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
            if let Some(Some(character)) = self.soldiers.character_of(unit) {
                self.characters.set_faction(character, mark.faction);
            }
            self.converted_log.push(UnitConverted::new(
                tick,
                unit.to_bits(),
                tile,
                from,
                mark.faction,
            ));
            // The faction that lost the unit lowers toward the faction that
            // took it. The marks are in slot order, so the writes are
            // too.[^3]
            //
            // [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
            self.relations
                .on_units_converted(tick, from, mark.faction, 1);
        }
        // The cohorts are the units of one faction at one site, so a faction
        // change moves a unit from one row to another. A table left as it was
        // would hold a headcount that no unit answers to, and the invariant
        // check refuses that state.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }

    /// Frees every sent unit that its destination plane no longer steers.
    ///
    /// **A verb that sends a unit must state what releases it.** A sent unit
    /// climbs its destination plane and reads no option row, so a unit that
    /// arrives and stays sent neither gathers nor delivers. Nothing released
    /// such a unit, and the defect was invisible for as long as arrival was
    /// impossible.[^1]
    ///
    /// The pass frees two kinds of unit, and one rule names both. A unit that
    /// stands on a tile of the seed set has arrived. A unit that reads no
    /// direction from the fine field and none from the coarse one is somewhere
    /// its plane leads nowhere: the plane holds no seed, the seed it held is
    /// gone, or the ground between the two refuses the unit. Neither kind may
    /// stay sent, because neither takes another step.
    ///
    /// **The release is the destination the caller set, and nothing else.**
    /// The pass clears the home site of no unit and the intent of no unit.
    ///
    /// The walk is over the arena in slot order, and it writes the send column
    /// of one unit for each entry. No entry reads what another wrote, so the
    /// order decides nothing and the result is the same at any thread
    /// count.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-572. `docs/FINDINGS.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn release_sent_units(&mut self) {
        let layout = self.pyramid.layout();
        let mut released: Vec<Entity> = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(Some(destination)) = self.soldiers.sent(unit) else {
                continue;
            };
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            let Some(key) = layout.key_of(tile) else {
                continue;
            };
            let state = send_state(
                destination,
                tile,
                layout.block_of_key(key),
                &self.approaches,
                &self.destinations,
            );
            if matches!(state, SendState::Steer(_)) {
                continue;
            }
            released.push(unit);
        }
        for unit in released {
            self.soldiers.set_sent(unit, None);
        }
    }

    /// Rebuilds the derived unit structure when the arena has moved past it.
    ///
    /// **This is the one place that makes the world readable again.** The
    /// step calls it at each of its barriers, and a verb that changes the
    /// arena outside a step calls it before it returns. A second body that
    /// compared the revisions for itself would be one rule in two places.[^1]
    ///
    /// The call returns a bridge error and not a step error, because a verb
    /// outside a step calls it too and a verb makes no step.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn refresh_bridge(&mut self) -> Result<(), BridgeError> {
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

/// What a unit takes in addition, in one tick, from wet ground.
///
/// The value sits here beside the ordinary rate, because both describe what
/// one gather takes and a second declaration elsewhere would be one fact in
/// two places.[^1] No measurement chose it, and a blocker holds the question
/// of what weather should be worth.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
const WET_GATHER_BONUS: u32 = 2;

/// One gather order, ready for the resolve.
#[derive(Clone, Copy, Debug)]
struct GatherIntent {
    /// The unit that gathers.
    unit: Entity,
    /// The tile that the unit stands on.
    tile: TileIdx,
    /// The kind that the unit gathers.
    kind: ResourceKind,
    /// The type of the unit. The resolve reads the gather rate and the carry
    /// capacity of the row it indexes.
    unit_type: UnitTypeId,
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
                        let unit_type = soldiers.unit_type(*unit)?;
                        Some(GatherIntent {
                            unit: *unit,
                            tile,
                            kind,
                            unit_type,
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
    /// The category that the unit builds.
    category: UpgradeCategory,
    /// The type of the unit. The advance reads the build rate of the row it
    /// indexes.
    unit_type: UnitTypeId,
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

/// Reports whether a builder may build one row on the ground it stands on.
///
/// **One function states the rule, and two paths call it.** The verb that
/// gives a build order calls it at the moment of the order. The pass that
/// collects the build intents calls it on every step. Two tests that drifted
/// apart would let a build the verb refused finish anyway.[^1] [^2]
///
/// The holder of the tile must be the builder's own faction when the row asks
/// for it. A row whose own ground column is zero crosses ground nobody holds,
/// because that is how a faction reaches ground it does not yet hold.[^1] That
/// row is permitted only where the builder's own faction zoned a project of
/// the same category, so the plan is the bound on the reach.[^4]
///
/// **The plan binds every category and not only the reaching one.** An order
/// that names one category on a tile the builder's own faction zones for
/// another is refused, whatever the row asks for. Without that clause a
/// faction plants an upgrade of one category on a tile its own plan zones for
/// another, and the tile then carries one category and no order can ever
/// raise the other.[^4] [^5]
///
/// The rule reads a column of the resolved row. It names no category, so a
/// row a caller wrote at run time obeys the same rule as a row of the default
/// table.[^3]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
/// [^4]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^5]: Findings register, FND-496. `docs/FINDINGS.md`
#[must_use]
const fn build_is_permitted(
    holder: Holder,
    faction: FactionId,
    row: UpgradeRow,
    category: UpgradeCategory,
    zoned: Option<UpgradeCategory>,
) -> bool {
    // The plan is read once, and both clauses below read that one answer. A
    // second lookup would be a second declaration of the same fact.[^6]
    //
    // [^6]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    if let Some(held) = zoned {
        if held.to_u8() != category.to_u8() {
            return false;
        }
    }
    if row.own_ground_required == 0 {
        // A row that asks for no held ground is how a faction reaches ground
        // it does not hold. The plan is the bound that stops it: a unit lays
        // one only inside a project of its own faction.[^4]
        //
        // [^4]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        return zoned.is_some();
    }
    match holder.faction() {
        Some(held) => held.0 == faction.0,
        None => false,
    }
}

/// Resolves the row that a build order names on one tile.
///
/// **This is the one statement of the resolution, and two paths call it.**
/// The verb that gives a build order calls it at the moment of the order. The
/// pass that collects the build intents calls it on every step. Two tests
/// that drifted apart would let a build the verb refused finish anyway.[^2]
///
/// The next level is one when the tile carries no upgrade of the category,
/// and one above the level that stands there otherwise.[^1]
///
/// **A level that owes a repair resolves to the row that stands there.** The
/// work of a worker on such a site buys condition first, so the row the order
/// reads is the row it mends. Without this arm a worker on a crumbling
/// top-level upgrade would be refused for the category being at its top, and
/// nothing could ever mend one.
///
/// A level whose gap in condition is worth less than one unit of work owes no
/// repair, and it resolves to the row above in the usual way. That is what
/// lets a level under light wear still rise.
///
/// # Errors
///
/// Returns a refusal when the tile carries another category, when the
/// category holds no row above the level that stands there, and when the row
/// does not fit the ground.
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn resolve_build_row(
    table: &UpgradeTable,
    ground: crate::terrain::TileKind,
    standing: Option<UpgradeSite>,
    category: UpgradeCategory,
) -> Result<UpgradeRow, BuildRefusal> {
    let level = match standing {
        Some(site) if site.category != category => {
            return Err(BuildRefusal::TileHoldsAnother {
                standing: site.category,
                asked: category,
            })
        }
        Some(site) => site.level,
        None => upgrade::NO_LEVEL,
    };
    if let Some(site) = standing {
        // The price is the one statement of whether a repair is due, and the
        // build pass spends the same value. A gap worth less than one unit of
        // work is priced at zero, and the order then reads the row above in
        // the usual way.[^3]
        //
        // [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
        if site.repair_price(table) > 0 {
            if let Some(row) = table.row(category, site.level) {
                return Ok(row);
            }
        }
    }
    let row = table
        .row(category, level + 1)
        .ok_or(BuildRefusal::CategoryAtTop { category, level })?;
    if !row.fits(ground) {
        return Err(BuildRefusal::GroundDoesNotFit { category, ground });
    }
    Ok(row)
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
/// Returns the build intent of one unit, or `None` when it builds nothing now.
///
/// **One function states what a unit builds, and three paths call it.** The
/// build pass collects the intents. The movement pass asks whether the unit
/// stands on the work it was ordered to do. The verb that gives the order
/// calls the two rules inside it directly, at the moment of the order.[^1]
///
/// The read is gated on the build order, which is one byte of one column. A
/// unit that carries no order costs that byte and nothing else, so a
/// population that does not build pays one column and no lookup.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
fn build_intent_of(
    unit: Entity,
    soldiers: &SoldierArena,
    holding: &Holding,
    ground: plan::Ground<'_>,
    plan: &PlanRegister,
) -> Option<BuildIntent> {
    let plan::Ground {
        grid,
        terrain,
        upgrades,
        table,
    } = ground;
    let category = soldiers.build_order(unit)??;
    let tile = soldiers.tile(unit)?;
    let unit_type = soldiers.unit_type(unit)?;
    // One function resolves the row, and the verb that gives the order calls
    // the same one. A build the table no longer holds, and a build whose tile
    // gained another category since the order, stop here.[^3]
    //
    // [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    let kind = terrain.kind(grid.address_of(tile)?)?;
    let row = resolve_build_row(table, kind, upgrades.at(tile), category).ok()?;
    // One function states the ground rule, and the verb that gives the order
    // calls the same one. A build whose ground changed hands since the order
    // stops here.[^2]
    //
    // [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    let holder = holding
        .holders()
        .get(tile.0 as usize)
        .copied()
        .unwrap_or(Holder::NOBODY);
    let faction = soldiers.faction(unit)?;
    if !build_is_permitted(holder, faction, row, category, plan.zones(faction, tile)) {
        return None;
    }
    Some(BuildIntent {
        unit,
        tile,
        category,
        unit_type,
    })
}

/// Returns the work that one unit of this type adds to a site in one tick.
///
/// **This is the one statement of what a builder contributes.** The build
/// advance sums it, and the movement hold reads it to decide whether the unit
/// is doing anything by standing still. A second expression of the same
/// product would let a unit be held for work it never adds.[^1]
///
/// A row whose build rate is zero contributes zero, and so does a row whose
/// rate scales the base work below one. Zero means cannot.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
#[must_use]
const fn build_contribution(row: crate::unit_type::UnitTypeRow) -> i64 {
    sim_math::scale_work(upgrade::BUILD_RATE, row.build_rate)
}

/// Reports whether the tile a unit stands on is work it must stay for.
///
/// **A build order is per unit and per tile, and movement takes its direction
/// from a field over cells.** A hold cannot be written into that field
/// without pinning every unit of the cell, so the movement pass asks this
/// question of each unit instead. The record states the rule and the
/// reasoning.[^1] [^2]
///
/// **Nothing stores the answer.** The pass derives it from the order, the
/// ground, the plan and the type on the tick it reads it, so there is no hold
/// to clear when the work finishes, when the plan drops the project, or when
/// the unit dies. A stored hold that outlived one of those three would freeze
/// a unit for the rest of the run.[^1]
///
/// **A unit that adds no work is never held.** The soldier row, the merchant
/// row and the leader row all build at zero, and the controller orders every
/// unit of its faction that stands on a zoned tile, whatever its type.[^3] A
/// hold without this clause would freeze one of them for ever, because no
/// work would ever finish the site it stands on.
///
/// # References
///
/// [^1]: ADR-0168, a build order holds a unit on its tile, and the hold is derived, decisions D1, D2 and D3. `docs/adrs/draft/adr-0168-a-build-order-holds-a-unit-on-its-tile.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^3]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
fn build_holds_unit(
    unit: Entity,
    soldiers: &SoldierArena,
    building: &Building<'_>,
    ground: plan::Ground<'_>,
) -> bool {
    let Building {
        holding,
        plan,
        unit_types,
        upgrades: _,
        table: _,
    } = *building;
    build_intent_of(unit, soldiers, holding, ground, plan)
        .is_some_and(|intent| build_contribution(unit_types.row(intent.unit_type)) > 0)
}

fn build_intents(
    soldiers: &SoldierArena,
    holding: &Holding,
    ground: plan::Ground<'_>,
    plan: &PlanRegister,
    threads: usize,
) -> Result<Vec<BuildIntent>, StepError> {
    let plan::Ground {
        grid,
        terrain,
        upgrades,
        table,
    } = ground;
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
                        build_intent_of(
                            *unit,
                            soldiers,
                            holding,
                            plan::Ground {
                                grid,
                                terrain,
                                upgrades,
                                table,
                            },
                            plan,
                        )
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

/// Returns the tile one step away, or `None` when nothing may stand there.
///
/// The call answers two refusals with one value, because a caller that wanted
/// to tell them apart would have to ask the grid and the terrain separately
/// and would then hold the rule twice.[^1]
///
/// A neighbour outside the world is no tile. The world is a rhombus and it
/// does not wrap.[^2]
///
/// Ground that admits no unit is no target. The capacity table is the one
/// statement of which ground admits a unit.[^3]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// One delivery that a bound contract admits.
///
/// The record is built in unit slot order and then sorted on a total key
/// before anything moves, so nothing downstream reads the order it was
/// collected in.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Copy, Debug)]
struct ContractDelivery {
    /// The unit that carries the load.
    unit: Entity,
    /// The slot of the settlement that receives it.
    site: u32,
    /// The kind of resource that the contract names for this party.
    kind: ResourceKind,
    /// The index of the row in the negotiation plane.
    row: usize,
    /// Whether this party is the one that opened the pair.
    owes_as_proposer: bool,
}

/// The group that the crossing order surveys for.
///
/// The survey scores a place against a group it must feed, and it refuses a
/// place that cannot.[^1] The crossing order founds nothing and seats nobody.
/// It names a place to walk to, so it asks for the smallest group the survey
/// admits and takes the best place that group could live at.
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
const CROSSING_SURVEY_GROUP: u32 = 1;

/// The group the settling survey asks a place to hold.
///
/// The survey scores a place against the group that would live there, and the
/// settle verb seats the group the settle column of the type names. The
/// target choice asks for the smallest group the survey admits, because a
/// place the smallest group cannot take is a place no settler can found on.
const SETTLING_SURVEY_GROUP: u32 = 1;

/// The ticks a settler lives away from a site that feeds it.
///
/// A settler that walks carries no home, so nothing feeds it and its deficit
/// grows by a fixed step each tick until it reaches the bound. The value is
/// measured and not derived: a probe sends one settler at distant ground and
/// reads the arena after every tick, and the starved log names the unit on
/// the same tick on every world the probe ran.[^1]
///
/// The figure is a property of the need rule and of nothing else. Standing on
/// water does not change it. The probe measured the same lifetime for a
/// settler that spent a third of it at sea.
///
/// # References
///
/// [^1]: The settler range probe. `crates/cachette-core/examples/settler_range_probe.rs`
const SETTLER_LIFETIME: u32 = 89;

/// How far a settler may be sent from the sites of its faction.
///
/// **A settler walks to its target and it eats on the way.** A place on the
/// far side of the world is a place the settler starves before it reaches, so
/// the target choice takes the best place inside this reach rather than the
/// best place in the world.
///
/// The value is half the measured lifetime, because a settler walks one tile
/// in one tick and it does not walk in a straight line. Ground that refuses a
/// step sends it round, and each detour costs a tick it cannot get back. Half
/// the lifetime leaves it as many ticks in hand as it spent walking.
///
/// **The last cell is no longer part of that margin.** The approach field
/// resolves the block that holds the target at the pitch of one tile, so a
/// settler that reaches that block walks at the target rather than wandering
/// inside it.[^2]
///
/// The two assertions below are floors and not knobs. A reach at or under the
/// founding distance would leave no place eligible, because the survey
/// refuses every place nearer than that distance. A reach under the edge of a
/// level 1 cell would name a target in the cell the settler already stands
/// in, and the coarse field steers nobody inside one cell.
///
/// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
const SETTLER_REACH: u32 = SETTLER_LIFETIME / 2;

const _: () = assert!(
    SETTLER_REACH > founding::MINIMUM_FOUNDING_DISTANCE,
    "a reach inside the founding distance leaves no place eligible"
);

const _: () = assert!(
    SETTLER_REACH > 1 << BLOCK_BITS_DEFAULT,
    "a reach inside one level 1 cell names a target that steers nobody"
);

/// Returns the neighbour a unit steps onto, or nothing when the ground there
/// refuses it.
///
/// **The gate is the capacity table, and this states no rule of its own.**
/// The `water_crossing` argument is the column of the type of the unit that
/// is stepping, and zero means cannot. The table takes it and answers the
/// capacity of the target, and passability is what it always was: a capacity
/// above zero.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
fn step_target(
    grid: Grid,
    terrain: Terrain,
    here: Axial,
    direction: usize,
    water_crossing: u32,
) -> Option<Axial> {
    let target = grid.neighbour(here, direction)?;
    if terrain.kind(target)?.is_passable_for(water_crossing) {
        Some(target)
    } else {
        None
    }
}

/// The load at which a unit counts as laden, when the caller states none.
///
/// A unit that reaches the mark takes the option that carries its load home,
/// and a unit below it does not. **The value is a parameter of the world, and
/// no record sets it.** A low mark sends a unit home for almost nothing and
/// spends its whole life walking. A high mark keeps a unit in the field until
/// a deposit near it runs dry. The reference table holds the value and the
/// derivation of it.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^2]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
pub const CARRY_MARK_DEFAULT: Amount = Amount(32);

/// Returns the carry class of one unit, from its load, its home and the mark.
///
/// **This is the one place that states the rule.** The choice pass, the
/// explanation and the public read all come through it, so no second site can
/// hold a different rule.[^1]
///
/// A unit with no home is never laden. The delivery moves a load into the
/// store of a home site, so an option that sent a homeless unit home would be
/// a capability that nothing can act on.[^2] [^3]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^3]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
#[must_use]
/// Folds the terrain into the ground under each weather cell.
///
/// **The weather reads three numbers about the ground, and none of them
/// changes.** The tiles a cell covers, the tiles of it that admit a unit, and
/// the sum of their heights are each a pure function of the world seed and
/// the address, so the fold runs once when the world is built.[^1]
///
/// **It does not read the level 1 summary.** That summary describes a block
/// thirty-two tiles a side, and it is the wrong source at any other weather
/// pitch. The two agree exactly at the level 1 pitch, because both fold the
/// same three fields over the same tiles.[^2]
///
/// The walk is row by row and then column by column, and the combine is
/// integer addition, so the answer does not depend on the order.[^3]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// Returns the weather cell that covers a tile, margin offset included.
///
/// **This is the one site that adds the margin offset.** The block layout
/// gives the cell of the world, and the lattice turns that into the cell of
/// the whole plane that the solve steps. Two callers ask it: the reader that
/// every tile reader goes through, and the god that inflicts weather on a
/// place.[^1]
///
/// Returns `None` when the tile is outside the world.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
fn weather_cell_of(layout: BlockLayout, lattice: PaddedLattice, tile: TileIdx) -> Option<u32> {
    lattice.whole_of_inner(layout.block_of_key(layout.key_of(tile)?))
}

fn carry_class_of(load: CarryLoad, home: u32, mark: Amount) -> CarryClass {
    if home == NO_HOME {
        return CarryClass::Free;
    }
    if load.total().0 < i64::from(mark.0) {
        return CarryClass::Free;
    }
    CarryClass::Laden
}

/// The draw index of the movement direction.
///
/// The movement system takes one draw for each soldier in each frame. A
/// second draw in the same system and frame must take the next index.
const DRAW_MOVE_DIRECTION: u32 = 0;

/// The draw index of the direction a unit takes when the ground refuses the
/// exit of its cell.
///
/// The fall-back is a second draw in the same system and the same frame, so
/// it takes the next index. A fall-back that reused the first index would
/// give the refused unit the direction that was just refused.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const DRAW_MOVE_FALLBACK: u32 = 1;

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
/// The live units of one frame, and the arena that holds their columns.
///
/// The two travel together. The order is a property of the walk and the
/// columns are a property of the arena, and a caller that could pass one
/// without the other could pass an order taken from a different arena.
/// The per-cell fields that steer a step, and the lattice they are indexed by.
///
/// The three travel together. Every option takes its direction from one of the
/// two fields, and both are indexed by the level 1 cell that the lattice
/// names, so a caller that could pass one without the others could pass a
/// field taken from a different lattice.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^2]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
struct Steering<'a> {
    /// The block lattice that both fields below are indexed by.
    layout: BlockLayout,
    /// One direction for each cell and each option that ranks a cell field.
    exits: &'a ExitField,
    /// One direction for each cell and each faction.
    returns: &'a ReturnField,
    /// One direction for each cell and each destination plane.
    ///
    /// The control plane names the seed set of a plane. A unit it sent to
    /// that plane reads one entry of it, in place of the entry that its own
    /// option would have read.[^3]
    ///
    /// [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    destinations: &'a SeededField,
    /// One direction for each destination plane and each tile of a seeded
    /// block.
    ///
    /// **The coarse field above steers a unit to the cell that holds its
    /// target and no further.** This one resolves that last cell at the pitch
    /// of one tile. A unit reads one entry of it, keyed on its own tile, and
    /// it reads no neighbouring tile, so it still searches nothing.[^4]
    ///
    /// [^4]: Findings register, FND-315. `docs/FINDINGS.md`
    approaches: &'a ApproachField,
    /// One direction for each faction plane and each tile of a block that
    /// holds a site of that faction.
    ///
    /// **The return field steers a laden unit to the cell that holds a site
    /// and no further**, and a delivery reads the tile. This one resolves
    /// that last cell at the pitch of one tile. It is the same mechanism as
    /// the approach field above, keyed on the faction.[^4]
    ///
    /// [^4]: Findings register, FND-315. `docs/FINDINGS.md`
    home_approaches: &'a ApproachField,
    /// One direction for each resource kind plane and each tile of a block
    /// that a gatherer stands in.
    ///
    /// **The exit field steers a unit to the cell that holds the most stock
    /// and no further**, and a gather resolve reads the tile. A unit that
    /// reached the cell stood on barren ground and took nothing, which is the
    /// defect the sent unit and the carrier both had.[^6] [^7]
    ///
    /// [^6]: Findings register, FND-315. `docs/FINDINGS.md`
    /// [^7]: Findings register, FND-589. `docs/FINDINGS.md`
    stock_approaches: &'a ApproachField,
    /// The tile of every settlement slot.
    ///
    /// **A unit stops on the tile of its own home and on no other.** The
    /// field above is seeded at every site of the faction, in the way the
    /// return field is, so its seed offset says only that the unit stands on
    /// some site of its faction. A unit that stopped on a site that is not
    /// its home would hold its load for ever, because a delivery reads the
    /// home tile.[^5]
    ///
    /// [^5]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    site_tiles: &'a [TileIdx],
}

/// What the movement pass reads to answer whether a unit is building here.
///
/// The three references are the three the build rule needs and that the
/// steering does not: who holds the ground, what the faction's plan zones,
/// and what the unit's type can do.
struct Building<'a> {
    /// The held-ground column that the build rule reads.
    holding: &'a Holding,
    /// The plan register that bounds where a unit may build.
    plan: &'a PlanRegister,
    /// The shared type table that states what a unit contributes.
    unit_types: &'a UnitTypeTable,
    /// The sparse upgrade map that holds what already stands on a tile.
    upgrades: &'a UpgradeMap,
    /// The shared upgrade table that resolves the row a build order names.
    table: &'a UpgradeTable,
}

struct UnitWalk<'a> {
    /// The arena that every identity below resolves against.
    soldiers: &'a SoldierArena,
    /// Every live unit, in the order the pass walks them.
    live: &'a [Entity],
}

/// What the destination plane of a sent unit says about the tile it stands on.
///
/// A plane answers one of three things, and the movement pass and the release
/// pass both read this answer. **The rule is one function, so no reader
/// declares a second time what arrival means.**[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SendState {
    /// The plane names the neighbour that the unit steps onto.
    Steer(u8),
    /// The unit stands on a tile of the seed set. It has arrived.
    Arrived,
    /// The plane names no step and no arrival. It leads the unit nowhere.
    Lost,
}

/// Returns what the destination plane says to a unit on one tile.
///
/// **The fine field wins over the coarse one.** The coarse field answers for
/// a block of tiles, and the fine one answers for a tile of a block that
/// holds a seed. A unit outside every seeded block reads no fine entry and
/// takes the coarse answer, which is the answer it read before the fine field
/// existed.[^1]
///
/// The lost answer covers three cases with one rule: the plane holds no seed,
/// the seed it held is gone, and the unit stands where neither field reaches.
/// A unit that reads it is released rather than steered, because a plane that
/// leads nowhere would otherwise hold the unit for ever.[^2]
///
/// The function reads one entry of each field, keyed on the tile and the cell
/// of the unit. It reads no neighbour and it scores none, so a unit still
/// searches nothing.[^3]
///
/// # References
///
/// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-572. `docs/FINDINGS.md`
/// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
fn send_state(
    destination: u16,
    tile: TileIdx,
    cell: u32,
    approaches: &ApproachField,
    destinations: &SeededField,
) -> SendState {
    match approaches.offset(destination, tile) {
        Some(AT_SEED) => SendState::Arrived,
        Some(direction) => SendState::Steer(direction),
        None => match destinations.direction(destination, cell) {
            Some(Some(direction)) => SendState::Steer(direction),
            _ => SendState::Lost,
        },
    }
}

fn soldier_moves(
    tick: Tick,
    seed: u64,
    terrain: Terrain,
    walk: &UnitWalk<'_>,
    steering: &Steering<'_>,
    building: &Building<'_>,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    let UnitWalk { soldiers, live } = *walk;
    let Steering {
        layout,
        exits,
        returns,
        destinations,
        approaches,
        home_approaches,
        stock_approaches,
        site_tiles,
    } = *steering;
    // **The walk is in cell order, not in slot order.** The two hold the same
    // units and differ only in the order. Every read below the filter is a
    // read of the tile side of the world at the tile the unit stands on: the
    // exit of its cell, the ground of its target, and the address of both.
    // The tile side is the larger of the two footprints, so the order that
    // makes it ascending is the order to walk.
    //
    // The arena cannot supply that order. A slot is half of an identity, so a
    // slot never moves, and an arena filled in tile order drifts away from it
    // as units die and slots return.[^13] The bridge already sorts every live
    // unit on the tile key once for each frame, at the barrier, so this order
    // costs the frame nothing more than it already paid.[^14]
    //
    // Nothing downstream reads this order. Admission sorts what it receives
    // on a total key of the target tile and the whole identity, so the same
    // set in another order gives the same result.[^15] The golden state hash
    // is the check that this claim holds, and it was checked by reversing the
    // walk rather than by asserting that reversing it would be safe.
    //
    // The caller passes the order in, and the caller is the one place that
    // takes it from the bridge. This function never chooses it.
    //
    // [^13]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    // [^14]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    // [^15]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
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
                        // **A unit that stands on the work it was ordered to
                        // do does not move.** The build advance adds the work
                        // of a builder to the tile the builder stands on, and
                        // one level of one upgrade asks for more work than one
                        // builder adds in one tick. A unit that walked away
                        // between two ticks therefore spread its labour over
                        // many tiles and finished none of them.[^22] [^23]
                        //
                        // **The hold is derived here and stored nowhere.** The
                        // question is asked again on every tick, from the
                        // order, the ground, the plan and the type. It stops
                        // being true on the tick the work finishes, on the
                        // tick the plan drops the project, and on the tick the
                        // ground changes hands. There is no hold to clear, so
                        // no unit can carry a stale one.[^22]
                        //
                        // **The clause sits above the send and above the
                        // intent**, because a unit the controller sent to a
                        // project still carries that send when it arrives. A
                        // send that outranked the hold would walk the builder
                        // off the tile it was sent to.[^22]
                        //
                        // [^22]: ADR-0168, a build order holds a unit on its tile, and the hold is derived, decisions D1, D2 and D3. `docs/adrs/draft/adr-0168-a-build-order-holds-a-unit-on-its-tile.md`
                        // [^23]: Findings register, FND-545. `docs/FINDINGS.md`
                        if build_holds_unit(
                            *soldier,
                            soldiers,
                            building,
                            plan::Ground {
                                grid,
                                terrain,
                                upgrades: building.upgrades,
                                table: building.table,
                            },
                        ) {
                            return None;
                        }
                        // **A unit the control plane sent somewhere climbs
                        // the plane it was sent to, and it reads no intent.**
                        // An order from the control plane is not a score, so
                        // it does not join the option set and it does not
                        // compete with one. It replaces the field that steers
                        // the step, and it replaces nothing else.[^20]
                        //
                        // The test sits above the intent filter on purpose. A
                        // unit that has chosen nothing yet holds no intent,
                        // and a sent unit that waited for one would stand
                        // still until its cell next chose.[^21]
                        //
                        // [^20]: ADR-0125, the control plane names the seed set of a destination field, decision D2. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
                        // [^21]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
                        let sent = soldiers.sent(*soldier)?;
                        // A unit that holds no intent does not move. The
                        // choice pass writes the intent, and a unit whose
                        // every option scored below the floor holds what it
                        // was doing.[^7]
                        let option = match sent {
                            Some(_) => None,
                            None => Some(soldiers.intent(*soldier)??),
                        };
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
                        // **Which field steers the step comes from the option
                        // row.** A row that ranks a summary field of the cell
                        // is steered by the exit field. A row that ranks the
                        // state of the unit is steered by the return field,
                        // which holds one direction for each cell and each
                        // faction.[^18]
                        //
                        // The unit still reads one entry. It reads no
                        // neighbouring cell, it scores no neighbour, and it
                        // computes nothing from its own address toward its own
                        // site, so no unit searches anything.[^19]
                        //
                        // [^18]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
                        // [^19]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
                        // **A unit that stands on the tile it was sent to
                        // takes no step.** The coarse field holds one
                        // direction for a block of tiles, so it says nothing
                        // once the unit is inside the block that holds its
                        // target, and the unit fell back to the keyed draw
                        // and walked away from the tile it had reached. The
                        // approach field answers at tile pitch, and the seed
                        // offset is how it says the unit has arrived.[^25]
                        //
                        // The clause sits above the steering, because a unit
                        // that arrived has nowhere further to be steered.
                        //
                        // **A sent unit never draws a direction.** It steps
                        // where its plane says, or it stands still and the
                        // step releases it once it has applied the moves of
                        // this frame. The fall-back to the keyed draw made a
                        // sent unit wander, and that wandering was the only
                        // thing that ever carried a load home.[^26]
                        //
                        // [^25]: Findings register, FND-315. `docs/FINDINGS.md`
                        // [^26]: Findings register, FND-572. `docs/FINDINGS.md`
                        let send = match sent {
                            Some(destination) => Some(send_state(
                                destination,
                                soldiers.tile(*soldier)?,
                                cell,
                                approaches,
                                destinations,
                            )),
                            None => None,
                        };
                        let steered = match send {
                            Some(SendState::Steer(direction)) => Some(direction),
                            // A unit that arrived, and a unit whose plane
                            // leads nowhere, both take no step.
                            Some(_) => return None,
                            None => None,
                        };
                        // **The homeward leg reads the same mechanism, keyed
                        // on the faction.** The return field holds one
                        // direction for a block of tiles, so it says nothing
                        // once a laden unit is inside the block that holds a
                        // site, and the unit fell back to the keyed draw. A
                        // delivery reads the tile the unit stands on, so a
                        // carrier that reached the cell delivered nothing,
                        // which is the defect the sent unit had.[^25]
                        let homeward = match (sent, option) {
                            (None, Some(option))
                                if matches!(OPTIONS[option as usize].ranked, Ranked::Carry) =>
                            {
                                home_approaches
                                    .offset(soldiers.faction(*soldier)?.0, soldiers.tile(*soldier)?)
                            }
                            _ => None,
                        };
                        // **The unit stops on the tile of its own home and on
                        // no other.** The field is seeded at every site of
                        // the faction, so the seed offset says only that the
                        // unit stands on some site of its faction. A unit
                        // that stopped on a site that is not its home would
                        // hold its load for ever. Such a unit keeps the
                        // answer it had before this field existed.
                        if homeward == Some(AT_SEED) {
                            let home = soldiers.home(*soldier)?;
                            let at_home = home.and_then(|slot| site_tiles.get(slot as usize))
                                == Some(&soldiers.tile(*soldier)?);
                            if at_home {
                                return None;
                            }
                        }
                        // **The gathering leg reads the same mechanism, keyed
                        // on the resource kind the unit was ordered to
                        // gather.** The exit field holds one direction for a
                        // block of tiles, and the stock of a tile is a level 0
                        // property. A unit inside the cell with the most food
                        // read that its own cell was the best one, stripped
                        // the ground under it, and stood there while the world
                        // still held food two tiles away.[^27]
                        //
                        // The plane is the order column, because the gather
                        // resolve reads that column. A field keyed on the
                        // option row instead would walk a unit to food and let
                        // it take stone.[^28]
                        //
                        // A unit that a caller sent somewhere reads its
                        // destination plane instead, and a laden unit reads
                        // the home field above, so this answers only for a
                        // unit that is steering itself by the ground.
                        //
                        // [^27]: Findings register, FND-589. `docs/FINDINGS.md`
                        // [^28]: Findings register, FND-590. `docs/FINDINGS.md`
                        let toward_stock = match (sent, option) {
                            (None, Some(option))
                                if matches!(OPTIONS[option as usize].ranked, Ranked::Cell(_)) =>
                            {
                                soldiers.gather_order(*soldier).flatten().and_then(|kind| {
                                    stock_approaches
                                        .offset(u16::from(kind.to_u8()), soldiers.tile(*soldier)?)
                                })
                            }
                            _ => None,
                        };
                        // **A unit that stands on stock of the kind it wants
                        // takes no step.** The seed offset is how the field
                        // says so. A unit that stepped away would strip one
                        // tile of one unit each frame and walk on.
                        if toward_stock == Some(AT_SEED) {
                            return None;
                        }
                        let steer = match (steered, option) {
                            // **The destination plane wins over the option
                            // row.** A caller that sends a unit somewhere has
                            // said where it goes, and the option the unit
                            // scored for itself says only what it wants.[^20]
                            // Which field answered is decided above, and the
                            // fine field wins over the coarse one there.[^25]
                            (Some(direction), _) => Some(Some(direction)),
                            (None, Some(option)) => match OPTIONS[option as usize].ranked {
                                // **The fine field wins over the coarse
                                // one**, in the way it does for a sent unit
                                // and for a laden one. A unit whose block
                                // holds no stock of its kind reads no fine
                                // entry and takes the coarse answer, which is
                                // the answer it read before this field
                                // existed. The seed offset never reaches
                                // here, because a unit that stands on stock
                                // already left the walk.[^27]
                                Ranked::Cell(_) => match toward_stock {
                                    Some(direction) => Some(Some(direction)),
                                    None => exits.exit(cell, option),
                                },
                                // **The fine field wins over the coarse
                                // one**, in the way it does for a sent unit.
                                // A unit outside every seeded block reads no
                                // fine entry and takes the coarse answer.
                                // The seed offset falls through to the coarse
                                // answer too, because the unit that reached
                                // its own home already left the walk.[^25]
                                Ranked::Carry => match homeward {
                                    Some(direction) if direction != AT_SEED => {
                                        Some(Some(direction))
                                    }
                                    _ => returns.direction(soldiers.faction(*soldier)?, cell),
                                },
                            },
                            // A unit that holds no intent and no destination
                            // left the walk at the filter above.
                            (None, None) => None,
                        };
                        let direction = match steer {
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
                        // **A refused direction falls back to a draw, and it
                        // never freezes the unit.** The exit of a cell is one
                        // direction for a block of tiles, and the ground under
                        // one unit of that block may refuse it. That refusal
                        // repeats every frame, because the cell, the option
                        // and the direction all hold, so a unit that only
                        // stayed put would stay put for ever. A unit against
                        // a shoreline is the case that showed it.[^16]
                        //
                        // The fall-back is the same keyed draw that a cell
                        // with no exit already takes, at the next draw index.
                        // It is keyed on the frame, so a unit the draw refuses
                        // again draws a different direction on the next
                        // frame.[^10]
                        //
                        // The fall-back reads the six neighbours of no cell.
                        // It reads one tile, which is the tile the unit would
                        // step onto, so the rule that a unit never searches
                        // its neighbourhood holds.[^17]
                        //
                        // [^16]: Findings register, FND-315. `docs/FINDINGS.md`
                        // [^17]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
                        // **The crossing of the unit is a column of its own
                        // type row.** The row is data and the movement pass
                        // reads it, so a type that crosses water and a type
                        // that does not take the same code path and differ
                        // only in the number they hand the capacity
                        // table.[^24]
                        //
                        // A unit whose type the arena cannot answer for
                        // crosses nothing, which is the answer every type
                        // gave before the column existed.
                        //
                        // [^24]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
                        let water_crossing = soldiers.unit_type(*soldier).map_or(0, |unit_type| {
                            building.unit_types.row(unit_type).water_crossing
                        });
                        let target = step_target(grid, terrain, here, direction, water_crossing);
                        let target = match target {
                            Some(target) => target,
                            None => {
                                let again = rng::draw_below(
                                    seed,
                                    rng::SYSTEM_SOLDIER_MOVE,
                                    tick.0,
                                    soldier.to_bits(),
                                    DRAW_MOVE_FALLBACK,
                                    NEIGHBOUR_COUNT as u64,
                                ) as usize;
                                step_target(grid, terrain, here, again, water_crossing)?
                            }
                        };
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
    /// Returns the count that one tile carries, for a caller that asks in
    /// ascending tile order.
    ///
    /// **The caller must ask for tiles in ascending order and must not change
    /// the table between two asks.** The position only moves forward, so a
    /// tile below the last one asked for reads zero rather than its count. The
    /// debug assertion below is what states that, and
    /// `a_forward_reader_agrees_with_the_search` is what proves it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    fn read_ascending(&self, at: &mut usize, tile: u32) -> u32 {
        while *at < self.entries.len() && self.entries[*at].0 < tile {
            *at += 1;
        }
        if *at < self.entries.len() && self.entries[*at].0 == tile {
            self.entries[*at].1
        } else {
            0
        }
    }

    /// Returns the count that one tile carries, by searching for it.
    ///
    /// **Nothing in the engine calls this.** It is the independent answer that
    /// `a_forward_reader_agrees_with_the_search` compares the forward reader
    /// against, and it is kept because a reader with nothing to disagree with
    /// proves nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2. `.claude/rules/testing.md`
    #[cfg(test)]
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
#[allow(clippy::too_many_arguments)]
fn admit(
    intents: &[(Entity, Axial)],
    soldiers: &SoldierArena,
    bridge: &UnitTileBridge,
    terrain: Terrain,
    upgrades: &UpgradeMap,
    table: &UpgradeTable,
    grid: Grid,
    guests: Guests<'_>,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    if intents.is_empty() {
        return Ok(Vec::new());
    }

    // **A holder refuses a guest it is below the guest edge toward.** The
    // rule is stated once, in the relation module, and this pass asks it for
    // each intent. The walk is in intent order and it reads the holder column
    // as the last spread left it, so the answer is the same at any thread
    // count.[^11]
    //
    // [^11]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    let refused: Vec<bool> = intents
        .iter()
        .map(|(entity, target)| {
            let Some(index) = grid.index_of(*target) else {
                return false;
            };
            let Some(holder) = guests
                .holders
                .get(index.0 as usize)
                .and_then(|holder| holder.faction())
            else {
                return false;
            };
            let Some(guest) = soldiers.faction(*entity) else {
                return false;
            };
            guests.relations.refuses_guest(holder, guest)
        })
        .collect();

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
    // The freshness of the derived structure is established once, here, rather
    // than on every tile the fill asks about. The borrow of the arena is what
    // stops the world changing while the fill walks it.
    bridge.describes(soldiers)?;
    let chunk_len = segments.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in segments.chunks_mut(chunk_len) {
            handles.push(scope.spawn(move || {
                // The segments of a chunk are in ascending tile order, so one
                // reader walks the block rather than searching it for each
                // segment. A reader is per-thread, because it carries the
                // position of its own walk.[^9]
                //
                // [^9]: Findings register, FND-295. `docs/FINDINGS.md`
                let mut cursor = bridge.tile_cursor();
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
                    // **The room a target holds is the room it holds for
                    // the units that asked for it.** Every intent in a
                    // segment came through the step gate above, so a segment
                    // on open water holds crossing units alone and a segment
                    // on any other ground answers the same capacity either
                    // way. The pass therefore asks the table for the crossing
                    // capacity, and it states no rule of its own about which
                    // ground admits whom.[^12]
                    //
                    // [^12]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
                    let ground = terrain.kind(address).map_or(0, |kind| {
                        kind.capacity_for(crate::terrain::SOME_WATER_CROSSING)
                    });
                    segment.capacity = upgrade::capacity_with(
                        ground,
                        upgrades.standing(TileIdx(segment.tile), table),
                    );
                    segment.standing = bridge
                        .units_on_tile(&mut cursor, TileIdx(segment.tile))
                        .len() as u32;
                }
            }));
        }
        for handle in handles {
            // A thread here reads shared memory and writes its own chunk of
            // the table, so it has no failure of its own.
            handle.join().expect("an admission thread cannot fail");
        }
    });

    let mut granted = vec![false; intents.len()];
    let mut arrived = TileCounts::default();
    let mut departed = TileCounts::default();
    let mut admitted: Vec<(Entity, Axial)> = Vec::new();

    for _ in 0..ADMISSION_PASSES {
        let first = admitted.len();
        // The segments come in ascending tile order, so the arrivals of one
        // pass are already an ascending run.
        let mut arrivals: Vec<(u32, u32)> = Vec::new();

        // The segments are in ascending tile order and both count tables are
        // too, and neither table changes while this loop runs. One forward
        // reader for each therefore replaces a search for each segment.[^10]
        //
        // [^10]: Findings register, FND-300. `docs/FINDINGS.md`
        let mut standing_at = 0usize;
        let mut arrived_at = 0usize;
        for segment in &segments {
            // A departure only ever leaves a tile a unit stood on, so the
            // subtraction cannot go below zero. It saturates rather than
            // wrapping, because a wrap here would read as a full tile and
            // reject every unit in silence.
            let occupancy = segment
                .standing
                .saturating_sub(departed.read_ascending(&mut standing_at, segment.tile))
                + arrived.read_ascending(&mut arrived_at, segment.tile);
            let mut room = segment.capacity.saturating_sub(occupancy);
            if room == 0 {
                continue;
            }

            for (_, position) in &sorted[segment.start..segment.end] {
                if room == 0 {
                    break;
                }
                let position = *position as usize;
                if granted[position] || refused[position] {
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

/// What admission reads to refuse a guest: who holds each tile, and what the
/// holder feels toward the guest.
#[derive(Clone, Copy)]
struct Guests<'a> {
    holders: &'a [Holder],
    relations: &'a RelationMatrix,
}

/// What one worker produces from one range of tiles.
///
/// The worker writes the events it emitted and the changes it made. It
/// writes nothing to the world, so two workers on two ranges cannot race.
#[derive(Clone, Debug, Default)]
struct ChunkResult {
    /// The events of the range, in ascending tile order.
    events: Vec<TileChanged>,
    /// The net change this range made to the count of changed tiles.
    ///
    /// The range applied its own changes to the field as it went, so nothing
    /// is carried here for a later pass to apply. Only the count comes back,
    /// because the count belongs to the whole field and not to one range.
    changed: i64,
}

/// Updates one contiguous range of tiles, in place, and returns its events.
///
/// The function is pure in the sense that the record requires: the same prior
/// values and the same key give the same result.[^1]
///
/// **The range is a mutable chunk of the field, and it is the only writer of
/// those tiles.** The field hands out disjoint chunks, so this function
/// cannot reach a tile another worker holds, and it needs no atomic.[^2]
///
/// The draw is keyed on the system, the frame and the tile, so a tile gives
/// the same answer whichever worker holds it and at any thread count.[^3]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
/// [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
fn update_range(tick: Tick, seed: u64, mut chunk: TileValueChunk<'_>) -> ChunkResult {
    let mut result = ChunkResult::default();
    let (start, end) = (chunk.start(), chunk.end());
    for index in start..end {
        let raw = rng::draw_below(seed, rng::SYSTEM_TILE_STUB, tick.0, u64::from(index), 0, 8);
        if raw >= 4 {
            continue;
        }
        let delta = Fix32((raw as i32) - 2);
        if delta.0 == 0 {
            continue;
        }
        // The tile is inside the chunk by construction, because the loop
        // walks the chunk's own range. A `None` here would mean the chunk
        // reported a range it does not hold.
        let Some(updated) = chunk.add(TileIdx(index), delta) else {
            continue;
        };
        let kind = if delta.0 > 0 {
            CHANGE_KIND_RAISED
        } else {
            CHANGE_KIND_LOWERED
        };
        // The holder is stamped after the holding spread, at the end of the
        // step. This pass runs at the top of the step, so any holder it read
        // here would be the holder of the frame before.[^4]
        //
        // [^4]: Findings register, FND-029. `docs/FINDINGS.md`
        result.events.push(TileChanged::new(
            tick,
            TileIdx(index),
            updated,
            Holder::NOBODY,
            kind,
        ));
    }
    result.changed = chunk.changed();
    result
}

/// The number of people each faction founds with when the seeding layer
/// founds the run.
///
/// **This is a set value and not a measured one.** The project owner set it.
/// The balance register holds the row, marks it unset, and records who set
/// this value and when.[^1]
///
/// **The Python binding reads this constant for its own default.** The
/// binding declares no number of its own, so the two cannot disagree.
///
/// # References
///
/// [^1]: Balance register, the founding group. `docs/reference/balance.md`
pub const FOUNDING_GROUP_DEFAULT: u32 = 2;

/// The number of luxury deposits the seeding layer places.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row, marks it unset, and records how this value was
/// chosen.[^1]
///
/// # References
///
/// [^1]: Balance register, the luxury deposits. `docs/reference/balance.md`
pub const LUXURY_DEPOSITS_DEFAULT: u32 = 8;

/// One row of the subsystem census: a name and the reader that counts it.
///
/// **This table is the only declaration of the list.** A caller that prints
/// the census, and a test that checks it, both read this table. A list
/// written anywhere else would be a second declaration site, and nothing
/// would fail when the two disagreed.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy)]
pub struct CensusRow {
    /// The name of the subsystem, as the census prints it.
    pub name: &'static str,
    /// What the count measures: what the world holds, or what the run made.
    pub basis: CensusBasis,
    /// The reader that counts what the subsystem produced.
    pub read: fn(&World) -> i64,
}

/// What a census count measures.
///
/// **Every row states its basis, so a reader never has to know one.** A table
/// that held a per-tick row beside a run total would give one zero two
/// meanings, and a reader could not tell them apart.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-498. `docs/FINDINGS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CensusBasis {
    /// What the world holds at the tick the reader runs. The count falls when
    /// the world loses what it counts.
    Held,
    /// What the run has made since the world was built. The count never
    /// falls, so a zero means that the thing never happened.
    Total,
}

impl std::fmt::Debug for CensusRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CensusRow")
            .field("name", &self.name)
            .finish()
    }
}

/// The subsystem census, one row for each subsystem.
///
/// The order is the order the census prints. Every reader is a whole count
/// in a 64-bit accumulator, so no count depends on the margin of a narrower
/// type.[^1]
///
/// **No row reports one tick.** A row states its basis. A held row counts
/// what the world holds now. A total row counts what the run made since the
/// world was built, and a zero in it means that the thing never happened.
/// Two rows once reported the last tick beside rows that reported the run,
/// and a reader could not tell which was which.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: Findings register, FND-498. `docs/FINDINGS.md`
pub const SUBSYSTEM_CENSUS: &[CensusRow] = &[
    CensusRow {
        name: "units",
        basis: CensusBasis::Held,
        read: |world| i64::from(world.soldiers.len()),
    },
    CensusRow {
        name: "settlements",
        basis: CensusBasis::Held,
        read: |world| i64::from(world.settlements.len()),
    },
    // The people the growth stage added over the run. The stage counts one
    // tick and clears that count before it acts, so the world folds the tick
    // into a run total at the site that clears it.[^6]
    //
    // [^6]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    CensusRow {
        name: "births",
        basis: CensusBasis::Total,
        read: |world| world.census.births + i64::from(world.births),
    },
    // What the build queue of every site has made, and what it has refused.
    // The two refusals of a finished entry stay apart, because they mean
    // different things to a watcher. A watcher then tells a site with no
    // resident to spend from a site whose store cannot pay.[^4]
    //
    // [^4]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    CensusRow {
        name: "queue_produced",
        basis: CensusBasis::Total,
        read: |world| world.census.queue_produced + i64::from(world.queues.produced()),
    },
    CensusRow {
        name: "queue_refused_without_a_person",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_without_a_person
                + i64::from(world.queues.refused_without_a_person())
        },
    },
    CensusRow {
        name: "queue_refused_without_goods",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_without_goods
                + i64::from(world.queues.refused_without_goods())
        },
    },
    CensusRow {
        name: "queue_refused_at_the_verb",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.queue_refused_at_the_verb + i64::from(world.queues.refused_at_the_verb())
        },
    },
    CensusRow {
        name: "seats_filled",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .positions
                .rows()
                .iter()
                .filter(|seat| seat.exists() && seat.holder_bits() != 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "characters",
        basis: CensusBasis::Held,
        read: |world| world.characters.iter().count() as i64,
    },
    CensusRow {
        name: "upgrades_complete",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .upgrades
                .sites()
                .iter()
                .filter(|site| site.is_complete())
                .count() as i64
        },
    },
    CensusRow {
        name: "wonders_complete",
        basis: CensusBasis::Held,
        // The count reads the victory claim column of the row that stands on
        // each tile. It names no category.[^1]
        //
        // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        read: |world| {
            world
                .standing_rows()
                .filter(|(_, row)| row.victory_claim > 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "stores_built",
        basis: CensusBasis::Held,
        // The count reads the store capacity column of the row that stands on
        // each tile. It names no category.[^1]
        //
        // [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        read: |world| {
            world
                .standing_rows()
                .filter(|(_, row)| row.capacity_of_store_change > 0)
                .count() as i64
        },
    },
    CensusRow {
        name: "luxury_tiles",
        basis: CensusBasis::Held,
        read: |world| world.luxuries.len() as i64,
    },
    // The storms a god raised over the run. One call of the divine power
    // that put water into the air is one storm. The row once read whether
    // the raised total stood above zero, which counted no storm and fell
    // back to zero when the water dried.[^5]
    //
    // [^5]: Findings register, FND-498. `docs/FINDINGS.md`
    CensusRow {
        name: "storms_raised",
        basis: CensusBasis::Total,
        read: |world| world.weather.storms(),
    },
    CensusRow {
        name: "contracts",
        basis: CensusBasis::Held,
        read: |world| {
            world
                .trade
                .rows()
                .iter()
                .filter(|row| row.is_bound())
                .count() as i64
        },
    },
    // The four rows below are what the trading controller has done over the
    // run. Each one counts an act of the stage and not a state of the world,
    // in the way the controller command row does.
    CensusRow {
        name: "boards_written",
        basis: CensusBasis::Total,
        read: |world| world.census.boards_written + i64::from(world.controller.boards_written()),
    },
    CensusRow {
        name: "offers_made",
        basis: CensusBasis::Total,
        read: |world| world.census.offers_made + i64::from(world.controller.offers_made()),
    },
    CensusRow {
        name: "contracts_bound",
        basis: CensusBasis::Total,
        read: |world| world.census.contracts_bound + i64::from(world.controller.contracts_bound()),
    },
    CensusRow {
        name: "carriers_assigned",
        basis: CensusBasis::Total,
        read: |world| {
            world.census.carriers_assigned + i64::from(world.controller.carriers_assigned())
        },
    },
    CensusRow {
        name: "controller_commands",
        basis: CensusBasis::Total,
        read: |world| world.census.controller_commands + i64::from(world.controller.applied()),
    },
    CensusRow {
        name: "controller_refused",
        basis: CensusBasis::Total,
        read: |world| world.census.controller_refused + i64::from(world.controller.refused()),
    },
    // What the plans of every faction have taken, finished, dropped and
    // refused. The record asks that a drop and a refusal each be counted.[^3]
    //
    // **The drop row and the refusal row are disjoint.** A write the plan
    // turned away at its bound is a drop and nothing else, and every other
    // refusal of a write or a build is a refusal and nothing else. A reader
    // adds the two rows and counts each act once. The two once overlapped,
    // and the sum double-counted a full plan.[^7]
    //
    // [^7]: Findings register, FND-496. `docs/FINDINGS.md`
    // [^3]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1, D4 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    CensusRow {
        name: "projects_zoned",
        basis: CensusBasis::Total,
        read: |world| world.plan.zoned_count(),
    },
    CensusRow {
        name: "projects_finished",
        basis: CensusBasis::Total,
        read: |world| world.plan.finished_count(),
    },
    CensusRow {
        name: "projects_dropped",
        basis: CensusBasis::Total,
        read: |world| world.plan.dropped_count(),
    },
    CensusRow {
        name: "projects_refused",
        basis: CensusBasis::Total,
        read: |world| world.plan.refused_count(),
    },
    CensusRow {
        name: "plan_passes",
        basis: CensusBasis::Total,
        read: |world| world.plan.pass_count(),
    },
    CensusRow {
        name: "game_ended",
        basis: CensusBasis::Total,
        read: |world| i64::from(world.controller.game_end().is_set()),
    },
    // The relation moves the controller made through the verb over the run,
    // and the crossings into the war band over the run, from any cause.
    CensusRow {
        name: "relation_moves",
        basis: CensusBasis::Total,
        read: |world| world.census.relation_moves + world.relation_moves_of_the_log(),
    },
    CensusRow {
        name: "wars_declared",
        basis: CensusBasis::Total,
        read: |world| world.census.wars_declared + world.relations.declarations(),
    },
    // The campaigns raised over the run, by the controller or by a caller,
    // and the campaigns whose objective passed to the campaigner.
    CensusRow {
        name: "campaigns_raised",
        basis: CensusBasis::Total,
        read: |world| world.census.campaigns_raised + world.campaigns.count(campaign::EVENT_RAISED),
    },
    CensusRow {
        name: "campaigns_won",
        basis: CensusBasis::Total,
        read: |world| world.census.campaigns_won + world.campaigns.count(campaign::EVENT_WON),
    },
    // The two acts of a conquest over the run. A taker keeps a city its own
    // reach supplies and burns one it does not, and these two rows are the
    // only place a reader sees which of the two a run reached.[^8]
    //
    // [^8]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    CensusRow {
        name: "sites_captured",
        basis: CensusBasis::Total,
        read: |world| world.sites_captured,
    },
    CensusRow {
        name: "sites_razed",
        basis: CensusBasis::Total,
        read: |world| world.sites_razed,
    },
    CensusRow {
        name: "sieges_pressed",
        basis: CensusBasis::Total,
        read: |world| world.sieges_pressed,
    },
    CensusRow {
        name: "sieges_relieved",
        basis: CensusBasis::Total,
        read: |world| world.sieges_relieved,
    },
];

impl World {
    /// Seeds the world: founds one run for every faction and places the
    /// luxuries, both from the seed.
    ///
    /// **This takes no parameter.** The founding group and the deposit count
    /// are provisional values that the balance register holds, so a caller
    /// that builds a world from a seed calls this once and names nothing.[^1]
    /// The founding draws as the founding verb draws, and the luxuries are
    /// placed by a keyed draw on the deposit index at frame zero.[^2]
    ///
    /// Returns what each faction got, in faction order.
    ///
    /// **The call leaves the world readable.** A founding seats a group and
    /// spends nothing, and a seated group is a change of the unit arena. The
    /// derived unit structure counts the changes of the arena and refuses
    /// every answer once the counts differ, so a reader that drew before the
    /// first step met a refusal.[^3] The seeding therefore refreshes the
    /// structure before it returns, in the way a step does at its end.
    ///
    /// **The refresh costs one rebuild.** The rebuild runs on one thread, and
    /// it runs once here, whatever the faction count is.[^4] A world is
    /// seeded once, so the world pays this at creation and never again.
    ///
    /// # Errors
    ///
    /// Returns an error when the world was seeded before, and when the
    /// rebuild of the derived unit structure refuses. A world is seeded once.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the founding group and the luxury deposits. `docs/reference/balance.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^4]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
    pub fn seed_world(&mut self) -> Result<Vec<FoundingOutcome>, SeedError> {
        if self.luxuries_seeded {
            return Err(SeedError::Luxury(LuxuryError::AlreadySeeded));
        }
        let outcomes = self.found_run_for_every_faction(FOUNDING_GROUP_DEFAULT);
        let tiles = u64::from(self.grid.tile_count());
        let mut placements = Vec::with_capacity(LUXURY_DEPOSITS_DEFAULT as usize);
        for deposit in 0..LUXURY_DEPOSITS_DEFAULT {
            let tile = rng::draw_below(
                self.config.seed,
                rng::SYSTEM_LUXURY,
                0,
                u64::from(deposit),
                0,
                tiles,
            );
            let luxury = LuxuryId((deposit % u32::from(crate::luxury::LUXURY_CEILING)) as u8);
            placements.push((TileIdx(tile as u32), luxury));
        }
        self.seed_luxuries(&placements)?;
        // **A verb that changes the structure leaves the world readable.**
        // The foundings above seated a group for every faction, and each
        // seating changed the unit arena. A caller that drew here before the
        // first step met a refusal, and the opening refresh of that step was
        // what repaired it. A reader is not obliged to step first, and the
        // drawing is not the only reader, so the verb repairs it here.
        self.refresh_bridge()?;
        Ok(outcomes)
    }

    /// Gives every soldier in the set the order to gather one kind.
    ///
    /// **This is the set form, and it is the one path a caller and the
    /// controller share.** The Python binding resolves its identities and
    /// calls this. The controller calls this. A verb that only one of them
    /// reached would be a capability that nothing tests from the
    /// boundary.[^1]
    ///
    /// Returns how many entities the arena refused, because they name no
    /// live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn order_gather_set(&mut self, units: &[Entity], kind: ResourceKind) -> usize {
        let mut refused = 0usize;
        for entity in units {
            if !self.order_gather(*entity, kind) {
                refused += 1;
            }
        }
        refused
    }

    /// Gives every soldier in the set the order to build one category.
    ///
    /// The set form, shared by the binding and the controller, as the gather
    /// order is.[^1] Returns how many entities the engine refused, and the
    /// first refusal it gave. A caller that reports the reason reads the
    /// second value, and a caller that counts reads the first.
    ///
    /// One loop serves both, so no second loop can apply another rule.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub fn order_build_set_reporting(
        &mut self,
        units: &[Entity],
        category: UpgradeCategory,
    ) -> (usize, Option<BuildRefusal>) {
        let mut refused = 0usize;
        let mut first = None;
        for entity in units {
            if let Err(refusal) = self.order_build(*entity, category) {
                refused += 1;
                first = first.or(Some(refusal));
            }
        }
        (refused, first)
    }

    /// Gives every soldier in the set the order to build one category, and
    /// returns how many the engine refused.
    pub fn order_build_set(&mut self, units: &[Entity], category: UpgradeCategory) -> usize {
        self.order_build_set_reporting(units, category).0
    }

    /// Gives every soldier in the set one unit type.
    ///
    /// The set form, shared by the binding and any engine caller, as the
    /// gather order is.[^1] Returns how many entities the arena refused.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn set_unit_type_set(&mut self, units: &[Entity], unit_type: UnitTypeId) -> usize {
        let mut refused = 0usize;
        for entity in units {
            if !self.set_unit_type(*entity, unit_type) {
                refused += 1;
            }
        }
        refused
    }

    /// Raises a campaign: takes the idle units of a faction, makes them
    /// soldiers and sends them at an objective tile.
    ///
    /// **This is the one path the controller and a Python caller share.**
    /// The raise acts through the set form of the type verb and through the
    /// send verb, and it writes one row of the campaign register.[^1]
    ///
    /// An idle unit is a live unit of the faction that nobody has sent
    /// anywhere. The cohort is the lowest identities among them, up to the
    /// count asked for, so two runs over one arena take one cohort. The scan
    /// that finds them follows the population, as the verbs it feeds do.
    ///
    /// The cohort is sent on the destination plane whose number is the
    /// faction number. A caller that sends its own set on that plane re-aims
    /// the cohort.
    ///
    /// The objective kind is read from the tile: a settlement of the faction
    /// itself makes a relief, and anything else makes a take. The holder of
    /// the tile at the raise is recorded, and the campaign closes when the
    /// holder changes.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no faction, when the address is
    /// outside the world, when the cohort size is zero, when the faction holds
    /// a live campaign, when it has no idle unit, when the world holds no
    /// destination plane for it, and when the send verb refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn raise_campaign(
        &mut self,
        faction: FactionId,
        objective: Axial,
        cohort: u32,
    ) -> Result<CampaignRow, CampaignError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let idle = self.campaign_cohort(faction, objective, cohort)?;
        let tile = self
            .grid
            .index_of(objective)
            .ok_or(CampaignError::OutsideWorld(objective))?;
        let plane = faction.0;
        let objective_kind = if self
            .settlements
            .on_tile(objective)
            .and_then(|site| self.settlements.faction(site))
            == Some(faction)
        {
            campaign::OBJECTIVE_RELIEVE_SITE
        } else {
            campaign::OBJECTIVE_TAKE_SITE
        };
        let holder_at_raise = self
            .holding
            .holder(objective)
            .and_then(Holder::faction)
            .map_or(campaign::NO_HOLDER, |holder| holder.0);
        self.set_unit_type_set(&idle, SOLDIER);
        self.send_units_to(&idle, &[objective], plane)?;
        let row = CampaignRow {
            raised_at: self.tick,
            objective_tile: tile.0,
            cohort_size: idle.len() as u32,
            faction,
            holder_at_raise,
            objective_kind,
            state: campaign::STATE_LIVE,
            padding: [0; 2],
        };
        assert!(
            self.campaigns.open(row),
            "the faction exists and holds no live campaign, so the register takes the row"
        );
        self.campaigns.push(CampaignEvent {
            tick: self.tick,
            objective_tile: tile.0,
            cohort_size: row.cohort_size,
            faction,
            kind: campaign::EVENT_RAISED,
            objective_kind,
            padding: [0; 4],
        });
        Ok(row)
    }

    /// Returns the cohort that a raise would march, or the refusal it would
    /// give.
    ///
    /// **This is the one statement of the rule.** The raise verb calls it
    /// before it writes a register row, and the legality answer calls it to
    /// fill one row of the action table.[^1] Nothing here mutates.
    ///
    /// # Errors
    ///
    /// Returns the refusal the raise verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn campaign_cohort(
        &self,
        faction: FactionId,
        objective: Axial,
        cohort: u32,
    ) -> Result<Vec<Entity>, CampaignError> {
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(CampaignError::NoSuchFaction(faction.0));
        }
        self.grid
            .index_of(objective)
            .ok_or(CampaignError::OutsideWorld(objective))?;
        if cohort == 0 {
            return Err(CampaignError::EmptyCohort);
        }
        if self.campaigns.live(faction).is_some() {
            return Err(CampaignError::LiveCampaign);
        }
        let plane = faction.0;
        if plane >= self.destinations.plane_count() {
            return Err(CampaignError::NoPlane(plane));
        }
        // The lowest identities among the idle units. The arena walks in
        // slot order, and the sort puts the generation above the slot, so
        // the choice is a property of the identities and not of the slots.
        //
        // **A unit that carries command reach is never taken.** The raise
        // retypes the cohort to the soldier row, and the soldier row carries
        // no command reach, so a raise that swept up the one leader of a
        // faction spent the very unit that lets the faction declare a war.
        // The faction would then march once and never again.
        //
        // **A unit that carries a water crossing is never taken either**, for
        // the same reason. The soldier row carries no crossing, so a raise
        // that swept up the mariners of an island faction spent the very
        // units that let it leave its island, and it would do so on the tick
        // each one was built.
        let leads = |unit: &Entity| {
            self.soldiers.unit_type(*unit).is_some_and(|unit_type| {
                let row = self.unit_types.row(unit_type);
                row.command_reach > 0 || row.water_crossing > 0
            })
        };
        let mut idle: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| self.soldiers.sent(*unit) == Some(None) && !leads(unit))
            .collect();
        idle.sort_unstable_by_key(|unit| unit.to_bits());
        // **The raise takes the number of units it asks for.** The project
        // order sends the idle units of a faction on the same destination
        // plane, so a raise that took the idle units alone found almost none
        // left and marched a cohort of one.[^2] A faction holds no live
        // campaign here, so every unit already on its own plane is a project
        // walker, and the raise re-aims it. The pool is therefore the idle
        // units first, then the walkers, and both in identity order.
        //
        // [^2]: Findings register, FND-542. `docs/FINDINGS.md`
        if idle.len() < cohort as usize {
            let mut walking: Vec<Entity> = self
                .soldiers
                .iter_faction(faction)
                .filter(|unit| self.soldiers.sent(*unit) == Some(Some(plane)) && !leads(unit))
                .collect();
            walking.sort_unstable_by_key(|unit| unit.to_bits());
            idle.extend_from_slice(&walking);
        }
        if idle.is_empty() {
            return Err(CampaignError::NoIdleUnit);
        }
        idle.truncate(cohort as usize);
        Ok(idle)
    }

    /// Returns the campaign rows of one faction, in slot order. Empty when the
    /// world has no such faction.
    #[must_use]
    pub fn campaigns_of(&self, faction: FactionId) -> &[CampaignRow] {
        self.campaigns.rows_of(faction)
    }

    /// Returns what happened to the campaigns on the last step, in the order
    /// it happened.
    #[must_use]
    pub fn campaign_log(&self) -> &[CampaignEvent] {
        self.campaigns.log()
    }

    /// Returns the campaign log as bytes, for the byte comparison the
    /// thread-count test makes.
    #[must_use]
    pub fn campaign_log_bytes(&self) -> &[u8] {
        self.campaigns.log_bytes()
    }

    /// Returns how many units the controller takes when it raises a
    /// campaign.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the campaign cohort size. `docs/reference/balance.md`
    #[must_use]
    pub const fn campaign_cohort_size(&self) -> u32 {
        self.campaigns.cohort_size()
    }

    /// Sets how many units the controller takes when it raises a campaign.
    pub const fn set_campaign_cohort_size(&mut self, cohort: u32) {
        self.campaigns.set_cohort_size(cohort);
    }

    /// Returns the weight vector of one faction, or `None` when the world
    /// has no such faction.
    #[must_use]
    pub fn faction_weights(&self, faction: FactionId) -> Option<FactionWeights> {
        self.controller.row(faction).map(|row| row.weights)
    }

    /// Writes the whole weight vector of one faction.
    ///
    /// The weight vector of a faction is that faction's policy, and this is
    /// the one verb that writes it.[^1] The vector is simulated state and it
    /// enters the state hash, so a write parts two worlds on the next
    /// tick.[^2]
    ///
    /// Returns `false` and changes nothing when the world has no such
    /// faction, or when a weight lies outside the range the balance register
    /// holds.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0156, a faction's option weights are policy, set through one verb, decision D3. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^2]: ADR-0156, a faction's option weights are policy, set through one verb, decision D1. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^3]: Balance register, the weight vector range. `docs/reference/balance.md`
    pub fn set_faction_weights(&mut self, faction: FactionId, weights: FactionWeights) -> bool {
        self.controller.set_weights(faction, weights)
    }

    /// Sets the flag that says an external caller controls a faction.
    ///
    /// A faction under external control receives no evaluation from the
    /// controller.[^1] Returns `false` when the world has no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn set_externally_controlled(&mut self, faction: FactionId, controlled: bool) -> bool {
        self.controller
            .set_externally_controlled(faction, controlled)
    }

    /// Returns whether an external caller controls a faction.
    ///
    /// Returns `None` when the world has no such faction.
    #[must_use]
    pub fn is_externally_controlled(&self, faction: FactionId) -> Option<bool> {
        self.controller
            .row(faction)
            .map(|row| row.externally_controlled != 0)
    }

    /// Returns how many evaluations the controller makes for one faction on
    /// one tick.
    #[must_use]
    pub const fn controller_evaluations(&self) -> u32 {
        self.controller.evaluations()
    }

    /// Sets how many evaluations the controller makes for one faction on one
    /// tick.
    ///
    /// The count is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the controller evaluations per faction per tick. `docs/reference/balance.md`
    pub const fn set_controller_evaluations(&mut self, evaluations: u32) {
        self.controller.set_evaluations(evaluations);
    }

    /// Returns the advertisement schedule: how many ticks lie between two
    /// board writes, and the offset inside that period.
    #[must_use]
    pub fn advertisement_schedule(&self) -> (u32, u32) {
        let schedule = self.controller.advert_schedule();
        (schedule.period(), schedule.phase())
    }

    /// Sets the advertisement schedule.
    ///
    /// The period and the phase are balance values, and the register holds
    /// the row.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is above
    /// the range that the scaling multiply takes.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the advertisement schedule. `docs/reference/balance.md`
    pub fn set_advertisement_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        let schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        self.controller.set_advert_schedule(schedule);
        Ok(())
    }

    /// Returns the store above which a faction offers a good, and below which
    /// it wants one.
    #[must_use]
    pub const fn surplus_mark(&self) -> u32 {
        self.controller.surplus_mark()
    }

    /// Sets the surplus mark.
    ///
    /// The mark is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the surplus mark. `docs/reference/balance.md`
    pub const fn set_surplus_mark(&mut self, mark: u32) {
        self.controller.set_surplus_mark(mark);
    }

    /// Returns how many carriers one faction assigns to one contract.
    #[must_use]
    pub const fn carriers_per_contract(&self) -> u32 {
        self.controller.contract_carriers()
    }

    /// Sets how many carriers one faction assigns to one contract.
    ///
    /// The count is a balance value, and the register holds the row.[^1] A
    /// count of zero assigns no carrier.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
    pub const fn set_carriers_per_contract(&mut self, carriers: u32) {
        self.controller.set_contract_carriers(carriers);
    }

    /// Returns how many ticks a contract that the controller opens runs for.
    #[must_use]
    pub const fn contract_term(&self) -> u32 {
        self.controller.contract_term()
    }

    /// Sets how many ticks a contract that the controller opens runs for.
    ///
    /// The term is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the contract term. `docs/reference/balance.md`
    pub const fn set_contract_term(&mut self, term: u32) {
        self.controller.set_contract_term(term);
    }

    /// Returns every carrier the controller has assigned, in faction order
    /// and then in contract order and then in identity order.
    ///
    /// The list is simulated state and not a log of one tick. A carrier stays
    /// in it until the contract settles or fails.
    #[must_use]
    pub fn carrier_assignments(&self) -> &[CarrierAssignment] {
        self.controller.carriers()
    }

    /// Returns the identity and the contract of every carrier the controller
    /// has assigned, in the order the carrier list holds.
    ///
    /// The list holds the identity as one integer, because it is plain data.
    /// This call resolves each one against the arena, so a caller never
    /// builds an identity of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    #[must_use]
    pub fn carrier_units(&self) -> Vec<(Entity, u32, FactionId)> {
        self.controller
            .carriers()
            .iter()
            .filter_map(|entry| {
                let unit = Entity::from_bits(entry.unit)?;
                self.soldiers.slot_of(unit)?;
                Some((unit, entry.row, entry.faction))
            })
            .collect()
    }

    /// Returns the tick at which the territory reader fires.
    #[must_use]
    pub const fn tick_limit(&self) -> u64 {
        self.controller.tick_limit()
    }

    /// Sets the tick at which the territory reader fires.
    ///
    /// The limit is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the tick limit. `docs/reference/balance.md`
    pub const fn set_tick_limit(&mut self, tick_limit: u64) {
        self.controller.set_tick_limit(tick_limit);
    }

    /// Returns the game end record. It is empty until a reader fires.
    #[must_use]
    pub const fn game_end(&self) -> GameEnd {
        self.controller.game_end()
    }

    /// Returns the score of one faction on the territory path: the tiles it
    /// holds.
    ///
    /// The count is the running total the holding keeps, so this starts no
    /// pass.[^1] Returns `None` when the world has no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn score(&self, faction: FactionId) -> Option<i64> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        Some(self.holding.holding_of(faction))
    }

    /// Returns the commands the controller emitted on the last step, in the
    /// order they applied.
    #[must_use]
    pub fn controller_log(&self) -> &[ControllerCommand] {
        self.controller.log()
    }

    /// Returns how many relation moves the controller made through the verb
    /// on the tick the log holds.
    fn relation_moves_of_the_log(&self) -> i64 {
        let schema = self.action_schema();
        self.controller
            .log()
            .iter()
            .filter(|command| {
                schema.verb_of(command.action) == Some(Verb::Relation) && command.applied != 0
            })
            .count() as i64
    }

    /// Adds what the controller log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    fn fold_the_controller_into_the_census(&mut self) {
        let moves = self.relation_moves_of_the_log();
        let totals = &mut self.census;
        totals.controller_commands += i64::from(self.controller.applied());
        totals.controller_refused += i64::from(self.controller.refused());
        totals.relation_moves += moves;
        totals.boards_written += i64::from(self.controller.boards_written());
        totals.offers_made += i64::from(self.controller.offers_made());
        totals.contracts_bound += i64::from(self.controller.contracts_bound());
        totals.carriers_assigned += i64::from(self.controller.carriers_assigned());
    }

    /// Adds what the campaign log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    fn fold_campaigns_into_the_census(&mut self) {
        let raised = self.campaigns.count(campaign::EVENT_RAISED);
        let won = self.campaigns.count(campaign::EVENT_WON);
        self.census.campaigns_raised += raised;
        self.census.campaigns_won += won;
    }

    /// Adds what the relation log holds to the run total.
    ///
    /// The caller calls this immediately before it empties the log.
    fn fold_relations_into_the_census(&mut self) {
        let declarations = self.relations.declarations();
        self.census.wars_declared += declarations;
    }

    /// Adds what the queue counts hold to the run total.
    ///
    /// The caller calls this immediately before it empties the counts.
    fn fold_queues_into_the_census(&mut self) {
        let totals = &mut self.census;
        totals.queue_produced += i64::from(self.queues.produced());
        totals.queue_refused_without_a_person += i64::from(self.queues.refused_without_a_person());
        totals.queue_refused_without_goods += i64::from(self.queues.refused_without_goods());
        totals.queue_refused_at_the_verb += i64::from(self.queues.refused_at_the_verb());
    }

    /// Returns the subsystem census: one count for each row of the one
    /// table, in table order.
    #[must_use]
    pub fn subsystem_census(&self) -> Vec<(&'static str, i64)> {
        SUBSYSTEM_CENSUS
            .iter()
            .map(|row| (row.name, (row.read)(self)))
            .collect()
    }

    /// Records the seat of a faction: the tile of its first founding.
    ///
    /// A later founding of the same faction leaves the seat where it is. The
    /// controller plans around the seat, so a faction with no seat receives
    /// no evaluation.
    fn record_seat(&mut self, faction: FactionId, place: Axial) {
        if let Some(tile) = self.grid.index_of(place) {
            self.controller.set_seat(faction, tile);
        }
    }

    /// Returns the seat of a faction: the tile of its first founding, or
    /// `None` when the faction founded nothing.
    #[must_use]
    pub fn seat(&self, faction: FactionId) -> Option<TileIdx> {
        self.controller.row(faction).and_then(FactionRow::seat)
    }

    /// Returns the site that one faction trades from: its live settlement in
    /// the lowest slot.
    ///
    /// The walk is over the settlement slots in slot order, so the answer is
    /// a property of the storage and of no thread.[^1]
    ///
    /// Returns `None` when the faction holds no settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn trading_site_of(&self, faction: FactionId) -> Option<Entity> {
        self.settlements
            .iter()
            .find(|site| self.settlements.faction(*site) == Some(faction))
    }

    /// Returns what the sites of one faction hold of each good, as whole
    /// numbers.
    ///
    /// **The accumulator is wide.** A store at the ceiling of the scale,
    /// summed over every site of a faction, passes the range of a narrower
    /// integer, and an accumulator must not depend on that margin.[^1]
    ///
    /// The sites are visited in slot order, and the goods in index order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn faction_stores(&self, faction: FactionId) -> [i64; RESOURCE_KIND_COUNT] {
        let mut totals = [0i64; RESOURCE_KIND_COUNT];
        for site in self.settlements.iter() {
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            let Some(store) = self.settlements.store(site) else {
                continue;
            };
            for kind in ResourceKind::ALL {
                let index = kind.index();
                let Some(held) = store.quantity(WORK_COMMODITY[index]) else {
                    continue;
                };
                totals[index] = totals[index].saturating_add(i64::from(held.to_int_floor()));
            }
        }
        totals
    }

    /// Rewrites the whole board of one faction from its site economies.
    ///
    /// The rows go through the write verb a Python caller calls, and the same
    /// refusals apply.[^1] The write replaces the whole board.[^2]
    ///
    /// Returns whether the verb took the write.
    ///
    /// # References
    ///
    /// [^1]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D5. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
    /// [^2]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D3. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
    fn controller_write_board(&mut self, faction: FactionId, draw: u32) -> bool {
        let stores = self.faction_stores(faction);
        let mark = i64::from(self.controller.surplus_mark());
        let bound = usize::from(self.market.bound());
        let rows = controller::board_of(
            self.config.seed,
            self.tick,
            faction,
            draw,
            &stores,
            mark,
            bound,
        );
        if self.advertise(faction, &rows).is_err() {
            return false;
        }
        self.controller.count_board();
        true
    }

    /// Returns the pair that one faction answers this tick, and the row of
    /// that pair.
    ///
    /// The pair is the live negotiation with the lowest other faction in
    /// which this faction speaks next. The walk is over the faction
    /// identifiers in ascending order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn controller_answer_due(&self, faction: FactionId) -> Option<(FactionId, TradeRow)> {
        for index in 0..self.config.faction_count.max(1) {
            let other = FactionId(index);
            if other == faction {
                continue;
            }
            let Ok((proposer, responder)) = self.live_orientation(faction, other) else {
                continue;
            };
            let Some(row) = self.trade.row(proposer, responder) else {
                continue;
            };
            if row.is_bound() || Self::turn_of(row, proposer, responder) != Some(faction) {
                continue;
            }
            return Some((other, row));
        }
        None
    }

    /// Returns the faction that one faction opens a negotiation with this
    /// tick, and the terms it opens with.
    ///
    /// The other boards are read in faction order. A pair in the war band and
    /// a pair that already holds a live negotiation are both passed over,
    /// because the verb refuses the first and this stage never opens a second
    /// negotiation with one pair.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn controller_match_due(&self, faction: FactionId) -> Option<(FactionId, controller::Terms)> {
        let mine = self.market.board(faction);
        if mine.is_empty() {
            return None;
        }
        for index in 0..self.config.faction_count.max(1) {
            let other = FactionId(index);
            if other == faction || !self.relations.permits_offer(faction, other) {
                continue;
            }
            if self.live_orientation(faction, other).is_ok() {
                continue;
            }
            let Some(terms) = controller::match_boards(mine, self.market.board(other)) else {
                continue;
            };
            if terms.give_amount == 0 || terms.take_amount == 0 {
                continue;
            }
            return Some((other, terms));
        }
        None
    }

    /// Takes the one negotiation step of one faction on one tick.
    ///
    /// An answer to a live negotiation comes before a new offer, so a faction
    /// that owes an answer never opens a second pair while it owes one. Every
    /// act passes the verb a Python caller calls.[^1]
    ///
    /// **The price is the integer midpoint of the two asking quantities.** No
    /// draw decides it. The faction accepts when the counteroffer asks no
    /// more than its own board asked, and refuses otherwise.[^2]
    ///
    /// Returns whether a verb took the step.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: Balance register, the surplus mark. `docs/reference/balance.md`
    fn controller_trade_step(&mut self, faction: FactionId) -> bool {
        if let Some((other, row)) = self.controller_answer_due(faction) {
            let own_ask = controller::asking_quantity_of(self.market.board(faction), row.take_kind);
            let Some(own_ask) = own_ask else {
                return self.refuse_trade(faction, other).is_ok();
            };
            if row.status == TRADE_OFFERED {
                // This faction answered the offer, so it restates the terms
                // at the midpoint of the two asks. The take side is restated
                // as it stands, because a counteroffer names both sides.
                let amount = controller::midpoint(row.give_amount, own_ask).max(1);
                let give = Consideration::resource(row.give_kind, amount);
                let take = Consideration::resource(row.take_kind, row.take_amount);
                return self
                    .counter_consideration(faction, other, give, take)
                    .is_ok();
            }
            if controller::accepts(row.give_amount, own_ask) {
                if self.accept_trade(faction, other).is_ok() {
                    self.controller.count_bound();
                    return true;
                }
                return false;
            }
            return self.refuse_trade(faction, other).is_ok();
        }
        let Some((other, terms)) = self.controller_match_due(faction) else {
            return false;
        };
        let term = self.controller.contract_term();
        let give = Consideration::resource(terms.give_kind, terms.give_amount);
        let take = Consideration::resource(terms.take_kind, terms.take_amount);
        if self
            .offer_consideration(faction, other, give, take, term)
            .is_err()
        {
            return false;
        }
        self.controller.count_offer();
        true
    }

    /// Returns every contract that one faction owes a carried quantity on,
    /// and the other party of each.
    ///
    /// The walk is over the negotiation plane in pair order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn controller_debts_of(&self, faction: FactionId) -> Vec<(u32, FactionId)> {
        let mut found = Vec::new();
        for (index, row) in self.trade.rows().iter().enumerate() {
            if !row.is_bound() {
                continue;
            }
            let (proposer, responder) = self.pair_of(index);
            let (owes, other) = if proposer == faction {
                (
                    row.proposer_side_is_carried() && row.owed_by_proposer() > 0,
                    responder,
                )
            } else if responder == faction {
                (
                    row.responder_side_is_carried() && row.owed_by_responder() > 0,
                    proposer,
                )
            } else {
                continue;
            };
            if owes {
                found.push((index as u32, other));
            }
        }
        found
    }

    /// Returns how many carriers one faction has on one contract.
    fn controller_carrier_count(&self, faction: FactionId, row: u32) -> u32 {
        self.controller
            .carriers()
            .iter()
            .filter(|entry| entry.faction == faction && entry.row == row)
            .count() as u32
    }

    /// Reports whether one faction has carrier work this tick.
    ///
    /// The answer is yes when it holds a carrier of a contract that ended,
    /// and yes when it owes a carried quantity on a contract that has fewer
    /// carriers than the balance row asks for.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
    fn controller_carry_work(&self, faction: FactionId) -> bool {
        let want = self.controller.contract_carriers();
        let ended = self.controller.carriers().iter().any(|entry| {
            entry.faction == faction
                && !self
                    .trade
                    .row_at(entry.row as usize)
                    .is_some_and(|row| row.is_bound())
        });
        if ended {
            return true;
        }
        if want == 0 || self.campaigns.live(faction).is_some() {
            return false;
        }
        self.controller_debts_of(faction)
            .into_iter()
            .any(|(row, _)| self.controller_carrier_count(faction, row) < want)
    }

    /// Assigns the carriers of one faction, and releases the ones it no
    /// longer needs.
    ///
    /// **A carrier is an idle unit whose type carries.** A unit type column
    /// of zero means that the unit cannot carry, so such a unit is never
    /// assigned.[^1] The lowest identities are taken, so two runs over one
    /// arena take one set.
    ///
    /// The unit takes the site of its own faction as its home and is sent to
    /// the site of the other party, through the two verbs a Python caller
    /// calls.[^2] The delivery pass then moves the quantity when the unit
    /// stands there with a load.[^3]
    ///
    /// **A carrier arrives only while its load stays below the carry mark.**
    /// A unit that holds a home and a load at the mark is laden, and a laden
    /// unit walks home rather than where it was sent.[^4] The mark is a world
    /// parameter that a caller sets, and a world whose mark is below what a
    /// carrier picks up on the way sends its carriers home instead.
    ///
    /// Returns whether the stage assigned or released anything.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: ADR-0147, a contract consideration is a tagged kind, decision D2. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^4]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    /// Writes the plan of one faction, in a fixed pass count.
    ///
    /// **The solver reads three things of one faction**: the tiles of its
    /// settlements, the ground it holds, and whether its stores fall short of
    /// the mark above which a site offers. It reads no unit, and it reads no
    /// tile outside the window it builds around the seat.[^1]
    ///
    /// The pass count and the window are balance values, and a blocker
    /// governs each of them.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: Balance register, the plan. `docs/reference/balance.md`
    /// [^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    fn solve_plan(&mut self, faction: FactionId) -> u32 {
        let Some(seat) = self.seat(faction) else {
            return 0;
        };
        // The settlements of one faction, in ascending tile order. The scan
        // follows the settlements and never the tile count.
        let mut sites: Vec<TileIdx> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.tile(site))
            .collect();
        sites.sort_unstable();
        sites.dedup();
        let stores = self.faction_stores(faction);
        let mark = i64::from(self.controller.surplus_mark());
        let short_of_stores = stores.iter().any(|held| *held < mark);
        // Whether every site of the faction is full. A faction with no free
        // place anywhere grows nobody, so the plan puts a lodging first
        // until one site has room again.[^1]
        //
        // The scan follows the settlements of the faction and never the
        // population.[^2]
        //
        // [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        // [^2]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
        let mut holds_a_site = false;
        let mut every_site_full = true;
        for site in self.settlements.iter() {
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            holds_a_site = true;
            if self
                .settlements
                .slot_of(site)
                .is_some_and(|slot| self.free_places_of(slot) > 0)
            {
                every_site_full = false;
            }
        }
        let short_of_places = holds_a_site && every_site_full;
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let needs = Needs {
            seat,
            sites: &sites,
            holders: self.holding.holders(),
            short_of_stores,
            short_of_places,
        };
        plan::solve(&ground, faction, &needs, &mut self.plan)
    }

    /// Sends the idle units of one faction to the projects its plan zones.
    ///
    /// **Each unit takes the project nearest to it by hex distance, and a tie
    /// takes the lower tile index.** A unit that is not idle is not moved,
    /// and the order goes through the send verb and the build verb that a
    /// Python caller also calls.[^1] [^2]
    ///
    /// The faction climbs one destination plane, and the plane of a faction
    /// is its number. A faction that holds a live campaign or a carrier is
    /// already climbing that plane, so it takes no project order and the
    /// command is refused. That rule is the one the campaign already applies
    /// to a faction that holds a carrier.
    ///
    /// The cost follows the idle units multiplied by the plan bound. The
    /// bound is fixed, so the cost follows the population and not the
    /// world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// Returns the destination plane that one faction crosses water on.
    ///
    /// **The crossing takes a plane of its own, and it never shares one.** A
    /// campaign, a carrier and a project order all climb the plane whose
    /// number is the faction number, so one of those three must yield to
    /// another. A crossing cannot yield: a faction on an island that waits
    /// for its war to end waits for a war it cannot reach, and the capability
    /// then ships inert.
    ///
    /// The crossing plane of a faction is its number raised by the faction
    /// count, so no faction takes the plane of another and no crossing takes
    /// the plane of a march. A world with too few planes for that answers
    /// nothing, and the faction takes no crossing order until a caller raises
    /// the plane count.
    ///
    /// Returns `None` when the world holds no such plane.
    fn crossing_plane_of(&self, faction: FactionId) -> Option<u16> {
        let plane = faction.0.checked_add(self.config.faction_count.max(1))?;
        (plane < self.destinations.plane_count()).then_some(plane)
    }

    /// Returns the water-crossing units of one faction that nobody has sent
    /// anywhere, in ascending identity order.
    ///
    /// The crossing is a column of the shared type table, and zero means
    /// cannot.[^1] The scan follows the population of the faction and reads
    /// no tile, so its cost does not grow with the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn idle_crossers_of(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| self.soldiers.sent(*unit) == Some(None))
            .filter(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            })
            .collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the tile that one faction would send its water-crossing units
    /// at, or nothing.
    ///
    /// **The choice reads the same bounded sample the founding choice
    /// reads.** It draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each, so its cost does not grow with the
    /// world.[^1] The draw is keyed on the faction and on nothing that
    /// changes, so the target of a faction is the same tile on every tick and
    /// the plane does not thrash.[^2]
    ///
    /// The places the faction already holds are the places taken, so the
    /// sample offers ground at least the founding distance away from every
    /// site the faction owns.[^3] That is ground the faction has not
    /// settled, on its own landmass or on another.
    ///
    /// The answer is nothing when the faction holds no idle unit that crosses
    /// water, and when a campaign or a carrier already climbs its plane. One
    /// plane serves one purpose at a time, which is the rule the campaign and
    /// the carriers already keep.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    fn controller_crossing_target(&self, faction: FactionId) -> Option<TileIdx> {
        if self.idle_crossers_of(faction).is_empty() {
            return None;
        }
        self.crossing_plane_of(faction)?;
        // The walk is over the settlement slots in ascending order, so the
        // list of taken places is a property of the arena and not of a visit
        // order.
        let taken: Vec<Axial> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let survey = self
            .survey_founding_apart(CROSSING_SURVEY_GROUP, faction, &taken)
            .ok()?;
        // **The order takes the best eligible candidate, and not the best
        // one.** The founding takes the best one and refuses the whole sample
        // when that place is too near a place already taken, because a
        // founding that walked down the list would seat a group somewhere
        // nobody chose. A crossing order names a place to walk to, so it
        // takes the next place down instead of refusing.
        //
        // The candidates are ordered on a total key, so the first eligible
        // one is a property of the sample and not of the order it was drawn
        // in.[^4]
        //
        // [^4]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        survey
            .candidates()
            .iter()
            .find(|candidate| candidate.is_eligible())
            .map(|candidate| candidate.tile())
    }

    /// Sends the idle water-crossing units of one faction at one tile.
    ///
    /// **The order goes through the send verb a Python caller calls.**[^1]
    /// The set holds only units that cross water, so the plane it climbs
    /// conducts across water and the field steers the set over a strait
    /// rather than at the near shore of one.[^2]
    ///
    /// The faction climbs the destination plane whose number is its own, in
    /// the way a campaign and a project order do.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    fn controller_cross(&mut self, faction: FactionId, tile: TileIdx) -> bool {
        let Some(plane) = self.crossing_plane_of(faction) else {
            return false;
        };
        let Some(address) = self.grid.address_of(tile) else {
            return false;
        };
        let set = self.idle_crossers_of(faction);
        if set.is_empty() {
            return false;
        }
        self.send_units_to(&set, &[address], plane).is_ok()
    }

    /// Returns the destination plane that one faction settles on.
    ///
    /// **The settling takes a plane of its own, and it never shares one.** A
    /// campaign, a carrier and a project order all climb the plane whose
    /// number is the faction number, and a crossing climbs the plane above
    /// those. A settling cannot yield to a campaign: a faction fights for
    /// most of a run, so a settler that waits for the war to end never founds
    /// anything, and the capability then ships inert.
    ///
    /// The settling plane of a faction is its number raised by twice the
    /// faction count, so no faction takes the plane of another, no settling
    /// takes the plane of a march, and no settling takes the plane of a
    /// crossing. A world with too few planes for that answers nothing, and
    /// the faction founds only where a settler already stands.
    ///
    /// Returns `None` when the world holds no such plane.
    fn settling_plane_of(&self, faction: FactionId) -> Option<u16> {
        let above = self.config.faction_count.max(1).checked_mul(2)?;
        let plane = faction.0.checked_add(above)?;
        (plane < self.destinations.plane_count()).then_some(plane)
    }

    /// Returns the settlers of one faction, in a fixed order.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The gate reads that column and never a type index, so no rule here
    /// names a type.[^1]
    ///
    /// The walk is over the units of the faction, and the answer is sorted on
    /// the identity bits, so the order is the same at every thread count.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn settlers_of(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).settle_group > 0)
            })
            .collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the tile that one faction would send its settlers at, or
    /// nothing.
    ///
    /// **The choice reads the same bounded sample the founding choice
    /// reads.** It draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each, so its cost does not grow with the
    /// world.[^1]
    ///
    /// The places the faction already holds are the places taken, so the
    /// sample offers ground at least the founding distance away from every
    /// site the faction owns. The settle verb applies the same distance rule
    /// again when the settler arrives, so a place that became too near while
    /// the settler walked is still refused.[^2]
    ///
    /// **The order takes the best eligible candidate inside the reach of a
    /// settler.** The rule has three parts, and each part is a filter or an
    /// order over the sample.
    ///
    /// **Beyond a distance.** A candidate must keep the founding distance
    /// from every site the faction holds. The survey applies that rule, with
    /// the sites of the faction as the places taken, and the settle verb
    /// applies the same rule again when the settler arrives.[^2] The distance
    /// is the founding distance and not a second one. A second constant would
    /// be one fact in two places, and the verb would then admit a place the
    /// target choice refused.[^4]
    ///
    /// **Eligible.** A candidate must be what the survey already calls
    /// eligible: ground that admits a settlement, ground that keeps the
    /// distance, and ground with room for the group.
    ///
    /// **Best.** Among what passes those two, the order takes the highest
    /// score. A tie takes the lower tile index, so the answer is a property
    /// of the sample and not of the draw order.[^3]
    ///
    /// **The reach is what keeps the settler alive.** A settler carries no
    /// home, so nothing feeds it and it starves after a measured number of
    /// ticks. The best place in a world-wide sample is usually a place the
    /// settler dies before reaching, which is why the order takes the best
    /// place inside the reach rather than the best place in the world.
    ///
    /// **A faction with nothing in reach takes the nearest eligible place
    /// instead.** The sample is drawn over the whole world, so a faction
    /// hemmed in by held ground may draw no eligible place inside the reach.
    /// The order sends the settler at the nearest such place rather than
    /// keeping it at home, because a settler that never walks founds nothing
    /// and the walk may still end at a place the settle verb admits.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn settling_target_of(&self, faction: FactionId) -> Option<Axial> {
        self.settling_target(faction)
    }

    /// How far a settler may be sent from the sites of its faction.
    ///
    /// The reader states the one value the target choice applies. A caller
    /// that checks the rule reads it here rather than holding a second copy
    /// of the number.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn settler_reach() -> u32 {
        SETTLER_REACH
    }

    fn settling_target(&self, faction: FactionId) -> Option<Axial> {
        // The walk is over the settlement slots in ascending order, so the
        // list of taken places is a property of the arena and not of a visit
        // order.
        let taken: Vec<Axial> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let survey = self
            .survey_founding_apart(SETTLING_SURVEY_GROUP, faction, &taken)
            .ok()?;
        // The eligibility and the founding distance both come from the
        // survey, so this walk states no rule of its own. It measures the
        // distance from the site of the faction nearest to the candidate,
        // because that is the site the settler leaves from.
        let in_reach: Vec<(u32, Accum, TileIdx, Axial)> = survey
            .candidates()
            .iter()
            .filter(|candidate| candidate.is_eligible())
            .filter_map(|candidate| {
                let tile = candidate.tile();
                let address = self.grid.address_of(tile)?;
                let near = taken
                    .iter()
                    .map(|seat| seat.distance(address))
                    .min()
                    .unwrap_or(0);
                Some((near, candidate.score(), tile, address))
            })
            .collect();
        // **The best place inside the reach.** The score is the key and the
        // tile index breaks a tie, so no two keys are equal and the answer
        // does not depend on the order the candidates were drawn in.
        let best = in_reach
            .iter()
            .filter(|(near, _, _, _)| *near <= SETTLER_REACH)
            .max_by_key(|(_, score, tile, _)| (score.0, core::cmp::Reverse(tile.0)));
        if let Some((_, _, _, address)) = best {
            return Some(*address);
        }
        // Nothing eligible lies inside the reach. The nearest eligible place
        // is the fallback, and the tile index breaks a tie for the same
        // reason.
        in_reach
            .iter()
            .min_by_key(|(near, _, tile, _)| (*near, tile.0))
            .map(|(_, _, _, address)| *address)
    }

    /// Founds a city from every settler of one faction that stands on ground
    /// a city may take, and sends the rest at ground worth founding on.
    ///
    /// **A settler that never walks is inert.** A site builds a settler
    /// inside the ground its own faction holds, and the settle verb refuses
    /// held ground, so a settler that stays where it was built never founds
    /// anything.[^1] The order therefore does two things, and which one a
    /// settler gets depends on where it stands.
    ///
    /// The founding is tried first, over every settler of the faction, so a
    /// settler that has arrived founds on the tick it arrives. The order
    /// sends the settlers only when no founding was made, so a settler that
    /// stands on a place a city may take is never walked away from it.
    ///
    /// **The send goes through the send verb a Python caller calls**, and it
    /// climbs the destination plane whose number is the faction number, in
    /// the way a campaign and a project order do.[^2] The send is refused
    /// when a campaign or a carrier already climbs that plane, which is the
    /// rule the project order keeps.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn controller_settle(&mut self, faction: FactionId) -> bool {
        let settlers = self.settlers_of(faction);
        if settlers.is_empty() {
            return false;
        }
        // The founding is tried over every settler, sent or free, because a
        // settler that has walked to its target is still climbing the plane
        // on the tick it arrives.
        if self
            .settle_set(&settlers)
            .iter()
            .any(super::founding::SettleOutcome::founded)
        {
            return true;
        }
        // Nothing founded, so every settler stands on ground a city may not
        // take. The order walks them at ground that a city may.
        let Some(plane) = self.settling_plane_of(faction) else {
            return false;
        };
        let Some(address) = self.settling_target(faction) else {
            return false;
        };
        self.send_units_to(&settlers, &[address], plane).is_ok()
    }

    fn controller_take_projects(&mut self, faction: FactionId) -> bool {
        // **The rule lives in the partition, and this verb reads it.** The
        // legality answer reads the same partition.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let Some((standing, walking)) = self.project_partition(faction) else {
            return false;
        };
        let mut applied = false;
        if !walking.is_empty() {
            let plane = faction.0;
            if plane < self.destinations.plane_count() {
                // The seeds are the projects the units took. The send verb
                // sorts and deduplicates the set itself, so the order of this
                // list decides nothing.
                let seeds: Vec<Axial> = walking
                    .iter()
                    .filter_map(|(_, project)| self.grid.address_of(project.tile))
                    .collect();
                let set: Vec<Entity> = walking.iter().map(|(unit, _)| *unit).collect();
                applied |= self.send_units_to(&set, &seeds, plane).is_ok();
            }
        }
        // The build order names the category of the project the unit stands
        // on. The units are grouped by category, in category order, so each
        // call is the set form the boundary already exposes.
        for category in UpgradeCategory::ALL {
            let group: Vec<Entity> = standing
                .iter()
                .filter(|(_, held)| *held == category)
                .map(|(unit, _)| *unit)
                .collect();
            if group.is_empty() {
                continue;
            }
            let refused = self.order_build_set(&group, category);
            // The plan counts a project refusal beside the one the build
            // verb counts. The two counts were here before the check moved
            // out of the build verb, and this call keeps them.
            for _ in 0..refused {
                self.plan.count_refusal();
            }
            applied |= refused < group.len();
        }
        applied
    }

    /// Splits the units of one faction into the ones that stand on a project
    /// and the ones that would walk to one.
    ///
    /// **This is the one statement of the rule.** The project verb calls it
    /// before it moves a unit, and the legality answer calls it to fill one
    /// row of the action table.[^1] Nothing here mutates.
    ///
    /// Returns `None` when the faction has no project, when a campaign holds
    /// the destination plane, or when the carriers hold it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    #[allow(clippy::type_complexity)]
    fn project_partition(
        &self,
        faction: FactionId,
    ) -> Option<(Vec<(Entity, UpgradeCategory)>, Vec<(Entity, Project)>)> {
        if self.plan.projects_of(faction).is_empty() {
            return None;
        }
        if self.campaigns.live(faction).is_some()
            || self
                .controller
                .carriers()
                .iter()
                .any(|entry| entry.faction == faction)
        {
            return None;
        }
        let mut units: Vec<Entity> = self.soldiers.iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        // **The order does two things, and which one a unit gets depends on
        // where it stands.** A unit that already stands on a project of its
        // faction takes the build order, because the build verb refuses a
        // tile no project zones. Every other idle unit is sent toward the
        // project nearest to it. A unit that is neither is left alone.
        let mut standing: Vec<(Entity, UpgradeCategory)> = Vec::new();
        let mut walking: Vec<(Entity, Project)> = Vec::new();
        for unit in units {
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            if let Some(category) = self.plan.zones(faction, tile) {
                standing.push((unit, category));
                continue;
            }
            if self.soldiers.sent(unit) != Some(None) {
                continue;
            }
            // **A unit that carries a water crossing takes no project.** The
            // crossing order is the one order that spends such a unit well,
            // and it applies after this one, so a project order that swept up
            // the mariners of an island faction would take them on the tick
            // each one was built and the faction would never leave its
            // island. The rule has the shape of the one the campaign keeps
            // for a unit that carries command reach.
            //
            // The unit is not idle in the sense of doing nothing. A mariner
            // that stands on a project of its faction still takes the build
            // order above, because that branch reads where the unit stands
            // and not what it is.
            if self
                .soldiers
                .unit_type(unit)
                .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            {
                continue;
            }
            if let Some(project) = self.project_for(faction, unit) {
                walking.push((unit, project));
            }
        }
        Some((standing, walking))
    }

    fn controller_carriers(&mut self, faction: FactionId) -> bool {
        let mut kept: Vec<CarrierAssignment> = Vec::new();
        let mut released: Vec<Entity> = Vec::new();
        for entry in self.controller.carriers() {
            let unit = Entity::from_bits(entry.unit);
            let live = unit.is_some_and(|unit| self.soldiers.slot_of(unit).is_some());
            let bound = self
                .trade
                .row_at(entry.row as usize)
                .is_some_and(|row| row.is_bound());
            if entry.faction != faction || (bound && live) {
                if live {
                    kept.push(*entry);
                }
                continue;
            }
            if let (true, Some(unit)) = (live, unit) {
                released.push(unit);
            }
        }
        let mut acted = !released.is_empty();
        if !released.is_empty() {
            // Every unit in the list is live, because the scan above kept
            // only the live ones, so the stop verb refuses nothing.
            let _ = self.stop_sending(&released);
        }
        let want = self.controller.contract_carriers();
        let mut assigned = 0u32;
        // The destination plane of a faction is the faction number, and a
        // campaign takes the same plane. A faction that holds a live campaign
        // therefore assigns no carrier, and a faction that holds a carrier
        // raises no campaign. One plane serves one purpose at a time.
        if want > 0 && self.campaigns.live(faction).is_none() {
            let plane = faction.0;
            for (row, other) in self.controller_debts_of(faction) {
                let held = kept
                    .iter()
                    .filter(|entry| entry.faction == faction && entry.row == row)
                    .count() as u32;
                if held >= want {
                    continue;
                }
                let (Some(home), Some(theirs)) =
                    (self.trading_site_of(faction), self.trading_site_of(other))
                else {
                    continue;
                };
                let Some(address) = self
                    .settlements
                    .tile(theirs)
                    .and_then(|tile| self.grid.address_of(tile))
                else {
                    continue;
                };
                let mut idle: Vec<Entity> = self
                    .soldiers
                    .iter_faction(faction)
                    .filter(|unit| self.soldiers.sent(*unit) == Some(None))
                    .filter(|unit| {
                        self.soldiers.unit_type(*unit).is_some_and(|unit_type| {
                            self.unit_types.row(unit_type).carry_capacity > 0
                        })
                    })
                    .collect();
                idle.sort_unstable_by_key(|unit| unit.to_bits());
                idle.truncate((want - held) as usize);
                if idle.is_empty() {
                    continue;
                }
                for unit in &idle {
                    self.set_home_site(*unit, Some(home));
                }
                if self.send_units_to(&idle, &[address], plane).is_err() {
                    continue;
                }
                for unit in &idle {
                    kept.push(CarrierAssignment::new(*unit, row, faction));
                }
                assigned = assigned.saturating_add(idle.len() as u32);
                acted = true;
            }
        }
        self.controller.set_carriers(kept);
        self.controller.count_carriers(assigned);
        acted
    }

    /// Runs the controller stage.
    ///
    /// The readers run first, while the record is empty. Then the controller
    /// plans the commands of the tick, the plan is sorted by faction and
    /// sequence, and each command applies through the set form of the verb
    /// a Python caller uses.[^1] [^2]
    ///
    /// The unit set of a faction is read once for each faction that emitted
    /// a command, in one scan of the arena in slot order. That scan follows
    /// the population, as the verb it feeds does when a Python caller names
    /// the same set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2, D4 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn run_controller(&mut self) {
        // The run total takes what the two logs hold before they are
        // emptied, so a census row says what the run did and not what the
        // last tick did.[^5]
        //
        // [^5]: Findings register, FND-498. `docs/FINDINGS.md`
        self.fold_the_controller_into_the_census();
        self.fold_campaigns_into_the_census();
        self.controller.clear_log();
        self.campaigns.clear_log();
        self.check_game_end();
        let tick = self.tick;
        let factions = usize::from(self.config.faction_count.max(1));
        // One scan of the arena, in slot order, finds the speaker of each
        // faction: its lowest-slot live unit whose type has command reach.
        // The gate reads the type column of the units and no flag.[^3]
        //
        // The same scan gathers the cohort of each faction: the live units
        // sent on the plane whose number is the faction number. No second
        // scan is made for it.
        //
        // [^3]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        let mut speakers: Vec<Option<Entity>> = vec![None; factions];
        let mut cohorts: Vec<Vec<Entity>> = vec![Vec::new(); factions];
        // The same scan reports which factions hold a settler. The gate reads
        // the settle column of the type of each unit and no flag.[^3]
        let mut settlers: Vec<bool> = vec![false; factions];
        for entity in self.soldiers.iter() {
            let (Some(faction), Some(unit_type)) = (
                self.soldiers.faction(entity),
                self.soldiers.unit_type(entity),
            ) else {
                continue;
            };
            let index = usize::from(faction.0);
            if index >= factions {
                continue;
            }
            if speakers[index].is_none() && self.unit_types.row(unit_type).command_reach > 0 {
                speakers[index] = Some(entity);
            }
            if self.unit_types.row(unit_type).settle_group > 0 {
                settlers[index] = true;
            }
            if self.soldiers.sent(entity) == Some(Some(faction.0)) {
                cohorts[index].push(entity);
            }
        }
        self.close_campaigns(&cohorts);
        let objectives = self.campaign_objectives();
        // The rival of a faction is the other faction with the most held
        // tiles. A faction with no speaker has no rival, because the verb
        // would refuse it.
        let held: Vec<(FactionId, i64)> = (0..factions as u16)
            .map(|index| (FactionId(index), self.holding.holding_of(FactionId(index))))
            .collect();
        let rivals: Vec<Option<FactionId>> = (0..factions)
            .map(|index| {
                speakers[index]?;
                controller::rival_of(FactionId(index as u16), held.iter().copied())
            })
            .collect();
        // A faction that holds a carrier raises no campaign, because the
        // campaign takes the destination plane the carriers climb.
        let objectives: Vec<Option<(u8, TileIdx)>> = objectives
            .into_iter()
            .enumerate()
            .map(|(index, objective)| {
                let faction = FactionId(index as u16);
                if self
                    .controller
                    .carriers()
                    .iter()
                    .any(|entry| entry.faction == faction)
                {
                    None
                } else {
                    objective
                }
            })
            .collect();
        // The three trade commands are pushed only when there is work. A
        // faction that has nothing to advertise, nothing to say and no
        // carrier to move emits nothing, so an idle world costs no command.
        // **The solver runs before the commands are planned.** It writes the
        // plan of every faction the controller evaluates, in faction order,
        // and the project order below then reads what it wrote.[^7]
        //
        // [^7]: ADR-0152, a faction plans its roads and zones with one solver, decisions D2 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        for index in 0..factions {
            let faction = FactionId(index as u16);
            // A faction under external control and a faction with no seat
            // receive no evaluation, so neither gets a plan.[^8]
            //
            // [^8]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D6 and D7. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
            let evaluated = self
                .controller
                .row(faction)
                .is_some_and(|row| row.externally_controlled == 0 && row.seat().is_some());
            if !evaluated {
                continue;
            }
            self.solve_plan(faction);
        }
        // The rows the unit type table fills. A row whose every column is
        // zero can do nothing, so the controller does not offer it. The list
        // is read from the table, and no rule here names a type.[^10]
        //
        // [^10]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        let offered: Vec<UnitTypeId> = (0..UNIT_TYPE_COUNT)
            .map(|index| UnitTypeId(index as u8))
            .filter(|unit_type| self.unit_types.row(*unit_type) != UnitTypeRow::NONE)
            .collect();
        let queue_draw = self.controller.queue_draw_index();
        let due = self.controller.board_due(tick);
        let states: Vec<FactionState> = (0..factions)
            .map(|index| {
                let faction = FactionId(index as u16);
                FactionState {
                    rival: rivals.get(index).copied().flatten(),
                    objective: objectives.get(index).copied().flatten(),
                    board_due: due && self.trading_site_of(faction).is_some(),
                    trade_due: self.controller_answer_due(faction).is_some()
                        || self.controller_match_due(faction).is_some(),
                    carry_due: self.controller_carry_work(faction),
                    // A faction with a march to make marches. The campaign
                    // and the project order take the same idle units and the
                    // same destination plane, so one of the two must yield,
                    // and the campaign is the one the war weight asked
                    // for.[^9]
                    //
                    // [^9]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
                    project_due: !self.plan.projects_of(faction).is_empty()
                        && objectives.get(index).copied().flatten().is_none(),
                    // A faction that owns no site with room in its queue
                    // queues nothing. The type comes from one keyed draw over
                    // the rows the table fills.[^10]
                    //
                    // **A faction with no speaker queues a leader instead of
                    // the draw.** Command reach sits on the leader row alone,
                    // and a faction with no unit that carries it moves no
                    // relation. It therefore never reaches the war band, never
                    // gets an objective, and never marches. The draw offers a
                    // leader one time in four, so a faction could run a whole
                    // game without one.
                    //
                    // The want is not a standing rule. It falls away as soon
                    // as a leader stands or a leader is on order, so a faction
                    // that has one queues by the draw again.
                    // **A faction crosses water only when it holds a unit
                    // that can.** The world asks before it plans, so a
                    // faction with no mariner reads no sample and the
                    // bounded survey costs an idle world nothing.
                    cross_to: self.controller_crossing_target(faction),
                    queue_type: self.controller_queue_site(faction).and_then(|_| {
                        if speakers.get(index).copied().flatten().is_none()
                            && !self.leader_is_on_order(faction)
                        {
                            return Some(LEADER);
                        }
                        controller::queued_type_of(
                            self.config.seed,
                            tick,
                            faction,
                            queue_draw,
                            &offered,
                        )
                    }),
                    // A faction that holds no settler founds nothing, so it
                    // emits no settle command and draws nothing for one.[^12]
                    //
                    // [^12]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
                    settle_due: settlers.get(index).copied().unwrap_or(false),
                }
            })
            .collect();
        let plan = self.controller.plan(self.config.seed, tick, &states);
        if plan.is_empty() {
            return;
        }
        // One scan of the arena, in slot order, buckets the live units by
        // faction. Only a faction that emitted a command gets a bucket.
        let mut wanted = vec![false; factions];
        for (faction, _, _) in &plan {
            wanted[usize::from(faction.0)] = true;
        }
        let mut sets: Vec<Vec<Entity>> = vec![Vec::new(); factions];
        for entity in self.soldiers.iter() {
            let Some(faction) = self.soldiers.faction(entity) else {
                continue;
            };
            let index = usize::from(faction.0);
            if index < factions && wanted[index] {
                sets[index].push(entity);
            }
        }
        let schema = self.action_schema();
        for (faction, sequence, choice) in plan {
            let set = std::mem::take(&mut sets[usize::from(faction.0)]);
            let applied = match choice {
                Choice::Gather(kind) => self.order_gather_set(&set, kind) < set.len(),
                Choice::Build(kind) => self.order_build_set(&set, kind) < set.len(),
                // The relation move goes through the same verb a caller
                // uses, with the speaker the scan above found. A faction
                // with no speaker planned no move, so the refusal here is
                // the verb's own.[^4]
                //
                // [^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Relation(other) => {
                    speakers[usize::from(faction.0)].is_some_and(|speaker| {
                        self.move_relation(speaker, other, controller::RELATION_STEP)
                            .is_ok()
                    })
                }
                // The raise goes through the one core function a caller
                // uses. The cohort size is a balance value.[^5]
                //
                // [^5]: Balance register, the campaign cohort size. `docs/reference/balance.md`
                Choice::Campaign { tile, .. } => {
                    let cohort = self.campaigns.cohort_size();
                    self.grid.address_of(tile).is_some_and(|address| {
                        self.raise_campaign(faction, address, cohort).is_ok()
                    })
                }
                // The board write, the negotiation step and the carriers all
                // pass the verbs a Python caller calls, and each reads the
                // world as the commands before it in this plan left it.[^6]
                //
                // [^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Advertise => self.controller_write_board(faction, sequence),
                Choice::Trade => self.controller_trade_step(faction),
                Choice::Carry => self.controller_carriers(faction),
                Choice::Project => self.controller_take_projects(faction),
                // The queue order goes through the one verb a Python caller
                // calls. The site is the lowest-slot site of the faction
                // whose queue has room, and the verb counts a refusal.[^11]
                //
                // [^11]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D2 and D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
                Choice::Queue(unit_type) => {
                    let site = self.controller_queue_site(faction);
                    site.is_some_and(|site| {
                        self.order_site_queue(faction, site, QueueOrder::Push(unit_type))
                            .is_ok()
                    })
                }
                // The crossing order goes through the send verb a Python
                // caller calls, with the tile the world chose before it
                // planned.[^12]
                //
                // [^12]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Cross(tile) => self.controller_cross(faction, tile),
                // **The settle order reads the arena and not the set above.**
                // The set is taken by the first command a faction emits, and
                // the settle order draws last, so it would read an empty set
                // on every tick a faction gathered or built. The order takes
                // the settlers of the faction from the arena instead.[^13]
                //
                // [^13]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
                Choice::Settle => self.controller_settle(faction),
            };
            let applied = u8::from(applied);
            sets[usize::from(faction.0)] = set;
            // **The row carries the whole action.** A controller choice and
            // a learner action reach one column in one encoding, so no
            // field of either lives in a second log.[^14]
            //
            // [^14]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
            let action = choice.action(&schema).unwrap_or(0);
            self.controller.push(ControllerCommand {
                tick,
                action,
                sequence,
                faction,
                applied,
                padding: [0; 5],
            });
        }
    }

    /// Closes every live campaign whose objective changed holder or whose
    /// cohort fell, and stops sending the survivors.
    ///
    /// The cohorts are the units sent on the plane of each faction, as the
    /// one scan of the stage found them. A campaign whose objective passed to
    /// the campaigner is won. One whose objective passed to anyone else has
    /// ended. One whose cohort is empty is lost. The survivors go back to the
    /// option they chose for themselves, through the stop verb a caller has.
    fn close_campaigns(&mut self, cohorts: &[Vec<Entity>]) {
        let tick = self.tick;
        for (index, cohort) in cohorts.iter().enumerate() {
            let faction = FactionId(index as u16);
            let Some(row) = self.campaigns.live(faction) else {
                continue;
            };
            let holder = self
                .grid
                .address_of(TileIdx(row.objective_tile))
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction)
                .map_or(campaign::NO_HOLDER, |holder| holder.0);
            // **A campaign that reaches nothing closes at its deadline.** A
            // faction with a live campaign raises no other one, so a campaign
            // that never closes takes the whole run and the faction marches
            // once. The deadline is a balance value.[^1] [^2]
            //
            // [^1]: Findings register, FND-542. `docs/FINDINGS.md`
            // [^2]: Balance register, the campaign deadline. `docs/reference/balance.md`
            let deadline = self.campaigns.deadline();
            let expired = deadline.0 > 0 && tick.0.saturating_sub(row.raised_at.0) >= deadline.0;
            let (state, kind) = if holder != row.holder_at_raise {
                if holder == faction.0 {
                    (campaign::STATE_WON, campaign::EVENT_WON)
                } else {
                    (campaign::STATE_ENDED, campaign::EVENT_ENDED)
                }
            } else if cohort.is_empty() {
                (campaign::STATE_LOST, campaign::EVENT_LOST)
            } else if expired {
                (campaign::STATE_EXPIRED, campaign::EVENT_EXPIRED)
            } else {
                continue;
            };
            self.campaigns.close(faction, state);
            // Every survivor is live, because the scan found it live on this
            // tick, so the stop verb refuses nothing.
            let _ = self.stop_sending(cohort);
            self.campaigns.push(CampaignEvent {
                tick,
                objective_tile: row.objective_tile,
                cohort_size: row.cohort_size,
                faction,
                kind,
                objective_kind: row.objective_kind,
                padding: [0; 4],
            });
        }
    }

    /// Chooses, for each faction, the objective it would march on.
    ///
    /// A faction with no seat, with a live campaign, or with no pair in the
    /// war band gets none.[^1] Otherwise an own settlement whose ground a
    /// faction at war holds is a relief, and the nearest enemy settlement is
    /// a take. The relief comes first. Nearest is the hex distance from the
    /// seat, and a tie goes to the lowest settlement slot. The scan walks the
    /// settlements and no unit, so it follows the site count.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn campaign_objectives(&self) -> Vec<Option<(u8, TileIdx)>> {
        let count = self.config.faction_count.max(1);
        let sites: Vec<(u32, FactionId, TileIdx)> = self
            .settlements
            .iter()
            .filter_map(|site| {
                Some((
                    self.settlements.slot_of(site)?,
                    self.settlements.faction(site)?,
                    self.settlements.tile(site)?,
                ))
            })
            .collect();
        (0..count)
            .map(|index| {
                let faction = FactionId(index);
                let seat = self.seat(faction)?;
                if self.campaigns.live(faction).is_some() {
                    return None;
                }
                let at_war = |other: FactionId| {
                    other != faction && self.relations.war_between(faction, other)
                };
                if !(0..count).any(|other| at_war(FactionId(other))) {
                    return None;
                }
                let seat = self.grid.address_of(seat)?;
                let distance = |tile: TileIdx| {
                    self.grid
                        .address_of(tile)
                        .map_or(u32::MAX, |address| seat.distance(address))
                };
                let holder_at_war = |tile: TileIdx| {
                    self.grid
                        .address_of(tile)
                        .and_then(|address| self.holding.holder(address))
                        .and_then(Holder::faction)
                        .is_some_and(at_war)
                };
                let relief = campaign::nearest_site(
                    sites
                        .iter()
                        .filter(|(_, owner, tile)| *owner == faction && holder_at_war(*tile))
                        .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
                );
                if let Some(tile) = relief {
                    return Some((campaign::OBJECTIVE_RELIEVE_SITE, tile));
                }
                campaign::nearest_site(
                    sites
                        .iter()
                        .filter(|(_, owner, _)| at_war(*owner))
                        .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
                )
                .map(|tile| (campaign::OBJECTIVE_TAKE_SITE, tile))
            })
            .collect()
    }

    /// Runs the game end readers, while the record is empty.
    ///
    /// The readers run in the fixed order domination, territory, wonder,
    /// renown. The first that fires writes the record, and the record is
    /// written once.[^1] Each reader is a pure function of the world, and
    /// each resolves a tie by the lowest faction identifier, because it
    /// visits the factions in ascending order and stops at the first that
    /// fires.[^2]
    ///
    /// **A caller may turn the readers off.** While they are off this
    /// function records nothing and the world runs to the tick limit. The
    /// readers decide when the step stops watching, and they change nothing
    /// else, so a run with the readers off holds the same event log as a run
    /// with the readers on that never fires.[^4]
    ///
    /// **A stock total wins no game.** The wealth path is gone, and the
    /// wonder is a path of its own with a reader in this table.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    /// [^4]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn check_game_end(&mut self) {
        if !self.balance.win_readers_enabled() {
            return;
        }
        if self.controller.game_end().is_set() {
            return;
        }
        // The order of this table is a rule of the game and not a balance
        // value. A path that has no reader is absent from it.
        let readers: [(GameEndReader, WinPath); 4] = [
            (Self::domination_winner, WinPath::Domination),
            (Self::territory_winner, WinPath::Territory),
            (Self::wonder_winner, WinPath::Wonder),
            (Self::renown_winner, WinPath::Renown),
        ];
        for (reader, path) in readers {
            if let Some(winner) = reader(self) {
                self.controller.record_end(self.tick, winner, path);
                return;
            }
        }
    }

    /// The factions of the world, in ascending identifier order.
    fn factions(&self) -> impl Iterator<Item = FactionId> {
        (0..self.config.faction_count.max(1)).map(FactionId)
    }

    /// The factions that may still win, in ascending identifier order.
    ///
    /// **A faction that has left the game wins nothing.** This is the one
    /// statement of that rule, and every game end reader walks this list
    /// rather than the faction list. A reader that walked the factions would
    /// let a faction with nothing alive take the world on held ground it can
    /// no longer defend.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D5. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^2]: Findings register, FND-579. `docs/FINDINGS.md`
    fn contenders(&self) -> impl Iterator<Item = FactionId> + '_ {
        self.factions()
            .filter(move |faction| !self.is_eliminated(*faction))
    }

    /// Returns the faction that holds the seat of a faction, or `None` when
    /// the faction has no seat or nobody holds it.
    fn seat_holder(&self, faction: FactionId) -> Option<FactionId> {
        let seat = self.controller.row(faction).and_then(FactionRow::seat)?;
        self.grid
            .address_of(seat)
            .and_then(|address| self.holding.holder(address))
            .and_then(Holder::faction)
    }

    /// The domination reader: one faction holds every seat, or every other
    /// faction has no units.
    ///
    /// A seat is the tile of the first founding of a faction, and the
    /// controller keeps it as one tile for each faction. The reader reads the
    /// holder of each seat tile, one lookup for each faction, and reads the
    /// live count of each faction, which the soldier arena keeps as a running
    /// total. It walks no unit and no tile.[^1]
    ///
    /// **A faction alone has dominated nothing.** The seat clause needs a
    /// rival seat to hold, so it fires only when the winner holds the seat of
    /// at least one other faction. The unit clause needs a rival to have
    /// lost, so it fires only in a world of two or more factions and only
    /// for a faction that still has a unit. Without both guards an empty
    /// world, or a world of one faction, would end on the first tick.
    ///
    /// A tie resolves by the lowest faction identifier: the walk is in
    /// ascending order and stops at the first faction that fires.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    fn domination_winner(&self) -> Option<FactionId> {
        if self.config.faction_count < 2 {
            return None;
        }
        let population = self.soldiers.population_by_faction();
        self.contenders().find(|candidate| {
            let mut rival_seats = 0u32;
            let mut holds_every_seat = true;
            let mut every_rival_is_empty = true;
            for other in self.factions() {
                // A faction that has left the game is no rival. Its seat is
                // not a seat to hold and its population is already zero.
                if other != *candidate && self.is_eliminated(other) {
                    continue;
                }
                if self
                    .controller
                    .row(other)
                    .and_then(FactionRow::seat)
                    .is_some()
                {
                    if other != *candidate {
                        rival_seats += 1;
                    }
                    holds_every_seat &= self.seat_holder(other) == Some(*candidate);
                }
                if other != *candidate && population[usize::from(other.0)] > 0 {
                    every_rival_is_empty = false;
                }
            }
            let by_seats = rival_seats > 0 && holds_every_seat;
            let by_units = every_rival_is_empty && population[usize::from(candidate.0)] > 0;
            by_seats || by_units
        })
    }

    /// The territory reader: at the tick limit, the faction with the most
    /// held tiles. The held count is a running total the holding keeps.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn territory_winner(&self) -> Option<FactionId> {
        if self.tick.0 < self.controller.tick_limit() {
            return None;
        }
        let held = self
            .contenders()
            .map(|faction| (faction, self.holding.holding_of(faction)))
            .collect::<Vec<_>>();
        controller::territory_winner(held.into_iter())
    }

    /// Returns the seats a faction holds: the seat tiles, its own and every
    /// rival's, whose holder is the faction.
    fn seats_held_by(&self, faction: FactionId) -> i64 {
        self.factions()
            .filter(|other| self.seat_holder(*other) == Some(faction))
            .count() as i64
    }

    /// Returns the stock total of every faction, by faction number.
    ///
    /// The total sums every commodity of every live settlement of the
    /// faction, as raw Q16.16 quantities, in a 64-bit accumulator. The
    /// walk is over the settlement arena in slot order, and it is not a walk
    /// over the population or the tiles.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    fn stock_totals(&self) -> Vec<Accum> {
        let mut totals = vec![Accum(0); usize::from(self.config.faction_count.max(1))];
        let live = self.settlements.live_column();
        let factions = self.settlements.faction_column();
        for (slot, store) in self.settlements.store_column().iter().enumerate() {
            if live[slot] == 0 {
                continue;
            }
            let Some(total) = totals.get_mut(usize::from(factions[slot].0)) else {
                continue;
            };
            for commodity in 0..COMMODITY_COUNT {
                let quantity = store
                    .quantity(CommodityId(commodity as u16))
                    .expect("the commodity index is below the count");
                *total = sim_math::combine(*total, Accum(i64::from(quantity.0)));
            }
        }
        totals
    }

    /// Returns the largest victory claim that stands on ground a faction
    /// holds, for every faction by faction number.
    ///
    /// The reader walks the entries that stand at a level and reads the
    /// victory claim column of each row. It names no category.[^2]
    ///
    /// The walk is over the sparse upgrade map, which holds one entry for
    /// each improved tile and nothing else, so it is not a walk over the
    /// tiles.[^1] A claim on ground nobody holds counts for nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn wonder_progress(&self) -> Vec<i64> {
        self.victory_claims()
            .into_iter()
            .map(|pair| pair.1)
            .collect()
    }

    /// Returns the largest victory claim, and the work toward it, on the
    /// ground of every faction, by faction number.
    ///
    /// The reader walks the sparse upgrade map and reads the victory claim
    /// column of two rows for each entry: the row that stands there, and the
    /// row above it. An entry that stands at a row with a claim reports the
    /// work of that row, and an entry that builds toward one reports the work
    /// done. It names no category.[^2]
    ///
    /// The walk is over one entry for each improved tile, so it is not a walk
    /// over the tiles.[^1] A claim on ground nobody holds counts for nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub(crate) fn victory_claims(&self) -> Vec<(i64, i64)> {
        let mut best = vec![(0i64, 0i64); usize::from(self.config.faction_count.max(1))];
        for site in self.upgrades.sites() {
            let standing = self.upgrade_table.row(site.category, site.level);
            let next = self.upgrade_table.row(site.category, site.level + 1);
            let claimed = standing.map_or(0, |row| i64::from(row.victory_claim));
            let (claim, work) = if claimed > 0 {
                (claimed, standing.map_or(0, |row| i64::from(row.work)))
            } else if next.is_some_and(|row| row.victory_claim > 0) {
                (0, site.progress.0)
            } else {
                continue;
            };
            let holder = self
                .grid
                .address_of(site.tile)
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction);
            if let Some(slot) = holder.and_then(|faction| best.get_mut(usize::from(faction.0))) {
                slot.0 = slot.0.max(claim);
                slot.1 = slot.1.max(work);
            }
        }
        best
    }

    /// The wonder reader: a finished wonder stands on ground the faction
    /// holds.
    ///
    /// A wonder is an upgrade row that carries a victory claim above zero.
    /// The reader walks the sparse upgrade map, reads the claim of the row
    /// that stands at each entry, and finds the first faction that holds a
    /// standing claim. A tie resolves by the lowest faction identifier.
    ///
    /// **A wonder costs work and stands on held ground, so it is an
    /// achievement.** A stock total is not, and the wealth clause that
    /// compared one is gone.[^1] The work a wonder costs and the claim its
    /// row carries are both columns of the upgrade table, and a caller sets
    /// them.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn wonder_winner(&self) -> Option<FactionId> {
        let claims = self.victory_claims();
        self.contenders()
            .find(|faction| claims[usize::from(faction.0)].0 > 0)
    }

    /// Returns the highest renown of any live character of every faction, by
    /// faction number, as raw Q16.16 values.
    ///
    /// The walk is over the character arena in slot order. It is not a walk
    /// over the units or the tiles.
    fn best_renown(&self) -> Vec<i64> {
        let mut best = vec![0i64; usize::from(self.config.faction_count.max(1))];
        for entity in self.characters.iter() {
            let (Some(faction), Some(renown)) = (
                self.characters.faction(entity),
                self.characters.renown(entity),
            ) else {
                continue;
            };
            if let Some(slot) = best.get_mut(usize::from(faction.0)) {
                *slot = (*slot).max(i64::from(renown.0));
            }
        }
        best
    }

    /// The renown reader: a character of the faction reaches the renown
    /// target.
    ///
    /// **The contest writes renown, and this reader fires in a seeded
    /// run.** The killer of each pair earns a share for each unit it felled,
    /// and the share goes to the champion of the faction. The control plane
    /// may also write the column. The blocker that governs the wider renown
    /// rule is open, and the target is a balance value under it.[^1] [^2] A
    /// tie resolves by the lowest faction identifier.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-150. `docs/BLOCKERS.md`
    /// [^2]: Balance register, the renown target. `docs/reference/balance.md`
    fn renown_winner(&self) -> Option<FactionId> {
        let best = self.best_renown();
        self.contenders()
            .find(|faction| best[usize::from(faction.0)] >= i64::from(self.balance.renown_target()))
    }

    /// Returns the running value of one faction on each win path.
    ///
    /// Returns `None` when the world has no such faction. The values are the
    /// ones the readers compare, so a caller can watch a path approach its
    /// end.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    #[must_use]
    pub fn standing(&self, faction: FactionId) -> Option<Standing> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        let at = usize::from(faction.0);
        Some(Standing {
            held_tiles: self.holding.holding_of(faction),
            seats_held: self.seats_held_by(faction),
            live_units: i64::from(self.soldiers.population_by_faction()[at]),
            store_total: self.stock_totals()[at].0,
            best_renown: self.best_renown()[at],
            wonder_progress: self.wonder_progress()[at],
        })
    }

    /// Returns how much the finished stores on or beside the tile of a
    /// settlement raise its store capacity, as a raw Q16.16 quantity.
    ///
    /// **This is the one place that states the "on or beside" rule.** A
    /// finished store on the tile of the settlement, or on one of its six
    /// neighbours, adds its raise. The raise of one kind is a catalogue
    /// row.[^1]
    ///
    /// **Nothing in the engine reads this.** The engine holds no store
    /// capacity, so the sum is a reading for the control plane and it changes
    /// no pass. Returns `None` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the store capacity raise. `docs/reference/balance.md`
    #[must_use]
    pub fn store_capacity_raise(&self, settlement: Entity) -> Option<i64> {
        let address = self.settlements.address(settlement)?;
        let mut raise = Accum(0);
        for place in core::iter::once(Some(address)).chain(self.grid.neighbours(address)) {
            let Some(row) = place.and_then(|near| self.standing_upgrade_row(near)) else {
                continue;
            };
            raise = sim_math::combine(raise, Accum(i64::from(row.capacity_of_store_change)));
        }
        Some(raise.0)
    }

    /// Returns the win-path balance values of the world.
    ///
    /// Each value holds a constant as its default, so a world that nobody
    /// configures behaves as it did before the table existed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D1. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    #[must_use]
    pub const fn balance(&self) -> Balance {
        self.balance
    }

    /// Sets the renown at which the renown reader fires, as a raw Q16.16
    /// value.
    ///
    /// This is a threshold. It decides when the renown reader fires and
    /// changes nothing else.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_renown_target(&mut self, raw: i32) {
        self.balance.set_renown_target(raw);
    }

    /// Sets the renown that one felled unit gives the champion of the faction
    /// that felled it, as a raw Q16.16 value.
    ///
    /// This is a rate. It changes what the simulation does, because the
    /// renown column is state that a later frame reads.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_renown_per_fell(&mut self, raw: i32) {
        self.balance.set_renown_per_fell(raw);
    }

    /// Sets whether the game end readers run.
    ///
    /// While they do not run, no reader records a game end and the world runs
    /// to the tick limit. A run with the readers off holds the same event log
    /// as a run with the readers on that never fires.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_win_readers_enabled(&mut self, enabled: bool) {
        self.balance.set_win_readers_enabled(enabled);
    }

    /// Sets the work that finishes a wonder.
    ///
    /// The work is a column of the upgrade table row that holds the wonder,
    /// and this writes that column and leaves the other columns of the row
    /// where they are. The table is the one declaration site of the value.
    ///
    /// This is a rate. A wonder that costs more work takes longer to build,
    /// so the value changes what the simulation does.[^1]
    ///
    /// Returns `false` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub fn set_wonder_work(&mut self, work: u32) -> bool {
        let Some(row) = self
            .upgrade_table
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        else {
            return false;
        };
        self.upgrade_table
            .define(
                UpgradeCategory::WONDER.to_u8(),
                upgrade::WONDER_LEVEL,
                UpgradeRow { work, ..row },
            )
            .is_ok()
    }

    /// Sets the victory claim that the wonder row carries.
    ///
    /// The claim is a column of the upgrade table row that holds the wonder,
    /// and this writes that column and leaves the other columns of the row
    /// where they are. The table is the one declaration site of the value.
    ///
    /// This is a threshold. The wonder reader fires for the faction that
    /// holds a standing claim above zero, so a claim of zero takes the wonder
    /// path out of the game and changes nothing else.[^1]
    ///
    /// Returns `false` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub fn set_wonder_victory_claim(&mut self, claim: u32) -> bool {
        let Some(row) = self
            .upgrade_table
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        else {
            return false;
        };
        self.upgrade_table
            .define(
                UpgradeCategory::WONDER.to_u8(),
                upgrade::WONDER_LEVEL,
                UpgradeRow {
                    victory_claim: claim,
                    ..row
                },
            )
            .is_ok()
    }
}

/// One game end reader: a pure function of the world that names the faction
/// that wins on its path, or nobody.
type GameEndReader = fn(&World) -> Option<FactionId>;

/// The stock one settlement can hold, as a raw Q16.16 quantity summed over
/// every commodity.
///
/// A store holds one `Fix32` for each commodity, and a `Fix32` saturates at
/// `i32::MAX`. The product of the two is therefore the most stock one
/// settlement can ever report, whatever it produces and however long it
/// runs.
///
/// **This ceiling is the reason the wealth bar is not a free value.** A bar
/// below it is crossed by one settlement that only waits, because the store
/// rises and does not fall. A bar above it asks the faction for a second
/// settlement.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-543. `docs/FINDINGS.md`
pub const STOCK_CEILING_OF_ONE_SETTLEMENT: i64 = (i32::MAX as i64) * (COMMODITY_COUNT as i64);

/// The running value of one faction on each win path.
///
/// A caller that reads this watches every path the readers watch: the held
/// tiles for territory, the seats and the live units for domination, the
/// best renown for renown, and the wonder work for the wonder. The stock
/// total feeds no reader, and it is reported because a caller may want to
/// see a faction grow rich.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
/// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decisions D2 and D4. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Standing {
    /// The tiles the faction holds. The territory reader compares it.
    pub held_tiles: i64,
    /// The seats the faction holds, its own and every rival's. The
    /// domination reader compares it against the seat count.
    pub seats_held: i64,
    /// The units of the faction that are alive. The domination reader
    /// compares it: the clause fires for a faction that still has a unit
    /// while every rival has none.
    pub live_units: i64,
    /// The sum of every store of every settlement of the faction, as a raw
    /// Q16.16 quantity. **No reader compares it**, because a stock total
    /// wins no game. It is reported and not read.[^2]
    ///
    /// # References
    ///
    /// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decision D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    pub store_total: i64,
    /// The highest renown of any live character of the faction, as a raw
    /// Q16.16 value. The renown reader compares it.
    pub best_renown: i64,
    /// The most work any wonder on ground the faction holds has reached.
    /// The wonder reader does not compare this. It compares whether a
    /// finished wonder stands, and this value is how far the furthest
    /// unfinished one has come.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0173, the wealth or wonder path has no reader, decision D1. `docs/adrs/draft/adr-0173-the-wealth-or-wonder-path-has-no-reader.md`
    pub wonder_progress: i64,
}

/// The action table, the legality answer, and the verb that takes one
/// action integer.
///
/// # One integer, one action
///
/// A learner plays one faction, and it acts by one integer that indexes a
/// bounded table the engine declares.[^1] The schema of that table is a
/// mixed radix over the argument positions each verb declares, and the
/// action module holds it.[^2] This block joins the table to the verbs.
///
/// # The answer and the verb read one rule
///
/// The engine answers one byte for each row of the table, and the byte says
/// whether the verb would refuse that row at this tick.[^3] **The answer
/// restates no refusal rule.** Each verb of the enumeration carries a check
/// that reports its refusal without acting, and the verb itself calls that
/// check before it changes anything. The answer calls the same check.
///
/// **The verb that takes an action does not read the answer first.** It
/// runs the verb and reports what the verb did. A row the answer allows and
/// the verb then refuses is therefore a defect a test can find, rather than
/// a disagreement the engine hides from itself.[^3]
///
/// # The answer names no target the faction cannot see
///
/// Two verbs act on a place: a campaign marches at an objective, and a
/// crossing sends at a tile. The engine resolves both, so neither carries an
/// argument position.[^2] **The engine resolves both through the readers
/// that answer for one faction.** A campaign objective is a settlement the
/// faction has observed, and a crossing target is a place the faction has
/// observed. A learner therefore cannot learn from a legality byte that a
/// settlement it has never seen stands somewhere.[^4]
///
/// # Determinism
///
/// The answer walks the rows in ascending action order on the calling
/// thread. Every candidate list it resolves is already sorted by a stable
/// key. Nothing here draws, reads a thread, or reads a completion
/// order.[^5]
///
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
/// [^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decisions D1 and D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
/// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
/// [^4]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
/// [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
impl World {
    /// Returns the declared layout of the action table of this world.
    ///
    /// The schema names each verb, the argument positions it declares, the
    /// bound of each position, and the stride that position moves the action
    /// integer by. A caller decodes an integer by arithmetic over this
    /// schema, and never by a table it holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
    #[must_use]
    pub fn action_schema(&self) -> ActionSchema {
        ActionSchema::of(ActionShape {
            faction_count: u32::from(self.config.faction_count.max(1)),
        })
    }

    /// Returns one byte for each row of the action table of this world.
    ///
    /// The byte is one when the verb would take that row at this tick, and
    /// zero when it would refuse it. Row zero is the no-op, and it is always
    /// one, so the answer is never empty.[^1]
    ///
    /// The answer holds only what the faction observes. **No argument
    /// widens it.**[^2]
    ///
    /// Returns `None` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    #[must_use]
    pub fn legal_actions(&self, faction: FactionId) -> Option<Vec<u8>> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        let schema = self.action_schema();
        let mut answer = vec![0u8; schema.length() as usize];
        // A verb is asked once whether it could act at all, and then once
        // for each row it holds. A verb with no argument position therefore
        // costs one question.
        for row in schema.rows() {
            let allowed = self.verb_is_legal(faction, row.verb);
            for offset in 0..row.rows {
                let action = row.first + offset;
                let Some((_, arguments)) = schema.decode(action) else {
                    continue;
                };
                let legal = allowed && self.arguments_are_legal(faction, row.verb, &arguments);
                answer[action as usize] = u8::from(legal);
            }
        }
        Some(answer)
    }

    /// Applies one action of one faction, through the same verbs a caller
    /// and the built-in controller use.[^1]
    ///
    /// Returns whether the verb took the action. **This runs the verb and
    /// reports what the verb did.** It does not read the legality answer
    /// first, so a disagreement between the two is a defect a test can
    /// find.[^2]
    ///
    /// The engine writes one row of the command log for the action, whether
    /// the verb took it or refused it.[^3]
    ///
    /// Returns `false` when the number names no faction, and when the
    /// integer is at or above the length of the table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn act(&mut self, faction: FactionId, action: u32) -> bool {
        if faction.0 >= self.config.faction_count.max(1) {
            return false;
        }
        let schema = self.action_schema();
        let Some((verb, arguments)) = schema.decode(action) else {
            return false;
        };
        let applied = self.apply_verb(faction, verb, &arguments);
        let tick = self.tick;
        self.controller.push(ControllerCommand {
            tick,
            faction,
            action,
            sequence: 0,
            applied: u8::from(applied),
            padding: [0; 5],
        });
        applied
    }

    /// Reports whether one verb could act at all for one faction this tick,
    /// without reading its arguments.
    ///
    /// The verbs that declare no argument position answer here alone.
    fn verb_is_legal(&self, faction: FactionId, verb: Verb) -> bool {
        match verb {
            // The no-op changes nothing, so nothing can refuse it.
            Verb::NoOp => true,
            // A gather order and a build order reach the units of the
            // faction. The order verb refuses a unit that is not live, and
            // the set below holds live units only.
            Verb::Gather | Verb::Build => !self.faction_units(faction).is_empty(),
            Verb::Relation => self.speaker_of(faction).is_some(),
            Verb::Campaign => self
                .observed_campaign_objective(faction)
                .and_then(|tile| self.grid.address_of(tile))
                .is_some_and(|address| {
                    self.campaign_cohort(faction, address, self.campaigns.cohort_size())
                        .is_ok()
                }),
            Verb::Advertise => self.check_faction(faction).is_ok(),
            Verb::Trade => self.trade_step_is_due(faction),
            Verb::Carry => self.controller_carry_work(faction),
            Verb::Project => self.project_work(faction),
            Verb::Queue => self.controller_queue_site(faction).is_some(),
            Verb::Cross => self.observed_crossing_target(faction).is_some(),
            Verb::Settle => self.settle_work(faction),
        }
    }

    /// Reports whether one verb would take the arguments of one row.
    ///
    /// A verb with no argument position answers yes here, because the check
    /// above already read its whole refusal.
    fn arguments_are_legal(&self, faction: FactionId, verb: Verb, arguments: &[u32]) -> bool {
        let first = arguments.first().copied().unwrap_or(u32::MAX);
        match verb {
            Verb::Gather => u8::try_from(first)
                .ok()
                .and_then(ResourceKind::from_u8)
                .is_some(),
            Verb::Build => {
                let Some(category) = u8::try_from(first).ok().and_then(UpgradeCategory::from_u8)
                else {
                    return false;
                };
                self.faction_units(faction)
                    .iter()
                    .any(|unit| self.build_refusal(*unit, category).is_ok())
            }
            Verb::Relation => {
                let (Some(speaker), Ok(other)) = (self.speaker_of(faction), u16::try_from(first))
                else {
                    return false;
                };
                self.move_relation_refusal(speaker, FactionId(other), controller::RELATION_STEP)
                    .is_ok()
            }
            // The queue verb refuses a type the table does not hold. The
            // site with room is already read above.
            Verb::Queue => u8::try_from(first)
                .ok()
                .and_then(UnitTypeId::from_u8)
                .is_some(),
            Verb::NoOp
            | Verb::Campaign
            | Verb::Advertise
            | Verb::Trade
            | Verb::Carry
            | Verb::Project
            | Verb::Cross
            | Verb::Settle => true,
        }
    }

    /// Runs one verb of the action table for one faction.
    ///
    /// Every arm goes through the verb the built-in controller goes
    /// through, so a learner reaches no store the controller cannot
    /// reach.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn apply_verb(&mut self, faction: FactionId, verb: Verb, arguments: &[u32]) -> bool {
        let first = arguments.first().copied().unwrap_or(0);
        match verb {
            Verb::NoOp => true,
            Verb::Gather => {
                let Some(kind) = u8::try_from(first).ok().and_then(ResourceKind::from_u8) else {
                    return false;
                };
                let set = self.faction_units(faction);
                !set.is_empty() && self.order_gather_set(&set, kind) < set.len()
            }
            Verb::Build => {
                let Some(category) = u8::try_from(first).ok().and_then(UpgradeCategory::from_u8)
                else {
                    return false;
                };
                let set = self.faction_units(faction);
                !set.is_empty() && self.order_build_set(&set, category) < set.len()
            }
            Verb::Relation => {
                let (Some(speaker), Ok(other)) = (self.speaker_of(faction), u16::try_from(first))
                else {
                    return false;
                };
                self.move_relation(speaker, FactionId(other), controller::RELATION_STEP)
                    .is_ok()
            }
            Verb::Campaign => {
                let cohort = self.campaigns.cohort_size();
                let Some(address) = self
                    .observed_campaign_objective(faction)
                    .and_then(|tile| self.grid.address_of(tile))
                else {
                    return false;
                };
                self.raise_campaign(faction, address, cohort).is_ok()
            }
            // The learner writes its board from the same site economies the
            // controller writes from. The draw index is zero, because a
            // learner emits one action for one faction on one tick.
            Verb::Advertise => self.controller_write_board(faction, 0),
            Verb::Trade => self.controller_trade_step(faction),
            Verb::Carry => self.controller_carriers(faction),
            Verb::Project => self.controller_take_projects(faction),
            Verb::Queue => {
                let (Some(unit_type), Some(site)) = (
                    u8::try_from(first).ok().and_then(UnitTypeId::from_u8),
                    self.controller_queue_site(faction),
                ) else {
                    return false;
                };
                self.order_site_queue(faction, site, QueueOrder::Push(unit_type))
                    .is_ok()
            }
            Verb::Cross => {
                let Some(tile) = self.observed_crossing_target(faction) else {
                    return false;
                };
                self.controller_cross(faction, tile)
            }
            Verb::Settle => self.controller_settle(faction),
        }
    }

    /// Returns the live units of one faction, in identity order.
    ///
    /// The arena yields live identities only, so every unit of the answer
    /// passes the liveness test that the order verbs apply.
    fn faction_units(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self.soldiers.iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the unit of one faction that carries command reach, by the
    /// lowest identity.
    ///
    /// The relation verb reads the command reach column, and a faction with
    /// no such unit moves no relation.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn speaker_of(&self, faction: FactionId) -> Option<Entity> {
        self.faction_units(faction).into_iter().find(|unit| {
            self.soldiers
                .unit_type(*unit)
                .is_some_and(|unit_type| self.unit_types.row(unit_type).command_reach > 0)
        })
    }

    /// Returns the objective that one faction would march on, over the
    /// settlements it has observed.
    ///
    /// **This reads the tiles the faction has seen, and never the world.**
    /// The controller reads the whole world for its own factions, and a
    /// learner may not, because a legality byte that answered from the whole
    /// world would tell the learner that a settlement stands somewhere it
    /// has never looked.[^1]
    ///
    /// The rule is the controller's own: a relief comes before a take, and
    /// the nearest site wins with a tie to the lowest slot.
    ///
    /// # References
    ///
    /// [^1]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn observed_campaign_objective(&self, faction: FactionId) -> Option<TileIdx> {
        let count = self.config.faction_count.max(1);
        let seat = self.grid.address_of(self.seat(faction)?)?;
        if self.campaigns.live(faction).is_some() {
            return None;
        }
        let at_war =
            |other: FactionId| other != faction && self.relations.war_between(faction, other);
        if !(0..count).any(|other| at_war(FactionId(other))) {
            return None;
        }
        let observed = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .and_then(|address| self.grid.index_of(address))
                .is_some_and(|index| self.observation.has_seen(faction, index))
        };
        let sites: Vec<(u32, FactionId, TileIdx)> = self
            .settlements
            .iter()
            .filter_map(|site| {
                Some((
                    self.settlements.slot_of(site)?,
                    self.settlements.faction(site)?,
                    self.settlements.tile(site)?,
                ))
            })
            .filter(|(_, _, tile)| observed(*tile))
            .collect();
        let distance = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .map_or(u32::MAX, |address| seat.distance(address))
        };
        let holder_at_war = |tile: TileIdx| {
            self.grid
                .address_of(tile)
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction)
                .is_some_and(at_war)
        };
        let relief = campaign::nearest_site(
            sites
                .iter()
                .filter(|(_, owner, tile)| *owner == faction && holder_at_war(*tile))
                .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
        );
        if relief.is_some() {
            return relief;
        }
        campaign::nearest_site(
            sites
                .iter()
                .filter(|(_, owner, _)| at_war(*owner))
                .map(|(slot, _, tile)| (distance(*tile), *slot, *tile)),
        )
    }

    /// Returns the tile one faction would send its water-crossing units at,
    /// when the faction has observed that tile.
    ///
    /// **The engine surveys the target, and a learner may only reach one it
    /// has seen.** The survey itself is the controller's, so the rule has
    /// one statement. This reader drops a target the faction has never
    /// observed, so a legality byte states nothing about unseen ground.[^1]
    ///
    /// # References
    ///
    /// [^1]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn observed_crossing_target(&self, faction: FactionId) -> Option<TileIdx> {
        let tile = self.controller_crossing_target(faction)?;
        let index = self.grid.index_of(self.grid.address_of(tile)?)?;
        if self.observation.has_seen(faction, index) {
            Some(tile)
        } else {
            None
        }
    }

    /// Reports whether one faction has a negotiation step to take this tick.
    ///
    /// The two readers below are the controller's own, and the step verb
    /// takes one branch or the other from them. A faction that neither
    /// reader names takes no step.
    fn trade_step_is_due(&self, faction: FactionId) -> bool {
        self.controller_answer_due(faction).is_some()
            || self.controller_match_due(faction).is_some()
    }

    /// Reports whether one faction has project work this tick.
    ///
    /// The plan holds the projects, and the plane the send needs is busy
    /// while a campaign runs or while the carriers hold it. Those are the
    /// three gates the project verb reads before it moves a unit.
    fn project_work(&self, faction: FactionId) -> bool {
        let Some((standing, walking)) = self.project_partition(faction) else {
            return false;
        };
        if !walking.is_empty() && faction.0 < self.destinations.plane_count() {
            return true;
        }
        standing
            .iter()
            .any(|(unit, category)| self.build_refusal(*unit, *category).is_ok())
    }

    /// Reports whether one faction would found a city or walk a settler this
    /// tick.
    ///
    /// The settle verb founds from the settlers that stand on ground a city
    /// may take, and it walks the rest at the place the survey names. A
    /// faction with no settler does neither.
    fn settle_work(&self, faction: FactionId) -> bool {
        let settlers = self.settlers_of(faction);
        if settlers.is_empty() {
            return false;
        }
        if settlers
            .iter()
            .any(|unit| self.settle_refusal(*unit).is_ok())
        {
            return true;
        }
        self.settling_plane_of(faction).is_some() && self.settling_target(faction).is_some()
    }
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

    /// The forward reader is a second way to answer what the search already
    /// answers. This drives both over a wide range of tiles, in the ascending
    /// order the reader requires, and compares them.[^1]
    ///
    /// The table is deliberately gappy, so the reader must step over tiles it
    /// holds no count for, and it holds the first and the last tile of the
    /// range so neither end is a special case.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[test]
    fn a_forward_reader_agrees_with_the_search() {
        let mut counts = TileCounts::default();
        counts.merge_ascending(&[(0, 3), (1, 1), (7, 9), (8, 2), (40, 5), (99, 7)]);
        let mut at = 0usize;
        for tile in 0..100u32 {
            assert_eq!(
                counts.read_ascending(&mut at, tile),
                counts.get(tile),
                "the two answers disagree at tile {tile}"
            );
        }
        // The reader stays on its answer, so asking twice repeats it.
        let mut again = 0usize;
        assert_eq!(counts.read_ascending(&mut again, 7), 9);
        assert_eq!(counts.read_ascending(&mut again, 7), 9);
    }

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
