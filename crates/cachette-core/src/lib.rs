//! The Cachette simulation core.
//!
//! This crate holds the simulation. It has no dependency on PyO3, so a
//! Python callback inside a simulation step is a compile error and not a
//! review comment.[^1] The absence of PyO3 also lets Miri run over the
//! unsafe storage code.[^1]
//!
//! All arithmetic on simulated state goes through [`sim_math`].[^2] No item
//! in this crate uses a floating-point type.[^3]
//!
//! # References
//!
//! [^1]: ADR-0041, a crate split enforces the boundary at compile time. `docs/adrs/REGISTRY.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

pub mod balance;
pub mod bridge;
pub mod campaign;
pub mod census;
pub mod character;
pub mod choose;
pub mod climate;
pub mod cohort;
pub mod contest;
pub mod controller;
pub mod conversion;
pub mod descent;
pub mod effective;
pub mod event;
pub mod event_layout;
pub mod founding;
pub mod growth;
pub mod hash;
pub mod hex;
pub mod holding;
mod household;
pub mod influence;
pub mod luxury;
pub mod observation;
pub mod padded;
pub mod plan;
pub mod position;
pub mod presence;
pub mod production;
pub mod promotion;
pub mod pyramid;
pub mod rates;
pub mod relation;
pub mod resource;
pub mod rng;
pub mod sim_math;
pub mod site;
pub mod slots;
pub mod soldier;
pub mod sort;
pub mod stage;
mod stock;
pub mod terrain;
pub mod tier;
pub mod tile_value;
pub mod trade;
pub mod types;
pub mod unit_type;
pub mod upgrade;
pub mod weather;
pub mod world;

pub use balance::{Balance, RENOWN_TARGET};
pub use bridge::{BlockLayout, BlockRange, BridgeError, UnitTileBridge};
pub use census::{census, Census, CensusError};
pub use character::{CharacterArena, CharacterError, Sex};
pub use choose::{
    CellField, ChoiceError, ChoiceExplanation, ChoiceSchedule, Drive, NeedBuckets, OptionRow,
    WeightProfile, NO_INTENT, OPTIONS, OPTION_COUNT,
};
pub use cohort::{
    CohortError, CohortRow, CohortTable, DeathPlane, DrawLedger, DrawPass, NeedCondition, NeedRule,
    SiteRationed, UnitStarved, COHORTS_PER_SITE,
};
pub use contest::{ContestError, Grievance, UnitFell};
pub use controller::{
    CarrierAssignment, Choice, ControllerCommand, FactionRow, FactionState, FactionWeights,
    GameEnd, Terms, WinPath, ADVERT_PERIOD_DEFAULT, ADVERT_PHASE_DEFAULT, CARRIER_ASSIGNMENT_BYTES,
    COMMAND_ADVERTISE, COMMAND_BUILD, COMMAND_CAMPAIGN, COMMAND_CARRY, COMMAND_GATHER,
    COMMAND_PROJECT, COMMAND_QUEUE, COMMAND_RELATION, COMMAND_TRADE, CONTRACT_CARRIERS_DEFAULT,
    CONTRACT_TERM_DEFAULT, EVALUATIONS_DEFAULT, SURPLUS_MARK_DEFAULT, TICK_LIMIT_DEFAULT,
    WEIGHT_HIGH, WEIGHT_LOW,
};
pub use conversion::{ConversionError, Convert, UnitConverted};
pub use descent::{
    Descent, DescentError, DescentId, HouseId, Parents, DESCENT_CEILING, RELATION_DEPTH,
};
pub use event::{ResourceTaken, TileChanged};
pub use founding::{
    Founding, FoundingError, FoundingOutcome, Provision, SettleError, SettleOutcome, Survey,
};
pub use hash::StateHash;
pub use hex::{Axial, Grid, GridError};
pub use holding::{FactionMask, Holder, Holding};
pub use influence::{Conductance, Influence, InfluenceError, InfluenceField};
pub use luxury::{
    LuxuryError, LuxuryField, LuxuryId, LuxurySet, LuxuryTile, VarietyLevel, LUXURY_CEILING,
};
pub use observation::{
    array_threshold, BlockForm, Observation, SightRules, TileLayer, SIGHT_BLOCKERS_DEFAULT,
    SIGHT_CEILING, SIGHT_RADIUS_DEFAULT, SIGHT_STEP_DEFAULT,
};
pub use plan::{PlanRefusal, PlanRegister, PlanRules, Project};
pub use position::{
    release_the_dead, Position, PositionError, PositionTable, SitePreference, NO_WORK,
    POSITIONS_PER_SITE, WORK_COMMODITY,
};
pub use presence::{PresenceRelation, PRESENCE_ROWS};
pub use production::{
    BuildCostRow, BuildCostTable, QueueEntry, QueueError, QueueOrder, QueueTable,
    DEFAULT_BUILD_COST_TABLE, QUEUE_BOUND, QUEUE_ENTRY_BYTES, QUEUE_PERIOD_DEFAULT,
    QUEUE_PHASE_DEFAULT, WORK_PER_ADVANCE,
};
pub use pyramid::{CellSummary, ExitField, Pyramid, NO_EXIT};
pub use rates::{
    RateError, RateLedger, RatePass, RateSchedule, RateTable, SiteRate, SiteShortfall,
};
pub use relation::{RelationCrossed, RelationError, RelationMatrix, RelationRules};
pub use resource::{
    Amount, CarryLoad, DepletionLedger, LedgerEntry, RecoveryRules, ResourceField, ResourceKind,
    TileGround,
};
pub use site::{
    CommodityId, SettlementArena, SettlementError, Store, StoreUpdate, COMMODITY_COUNT,
};
pub use slots::{Candidate, SlotError, Slots};
pub use soldier::{SoldierArena, SoldierError};
pub use sort::{SortError, SortKey};
pub use stage::{FrameCosts, Stage, StageCost, STAGES, STAGE_COUNT};
pub use terrain::{Terrain, TerrainTile, TileKind};
pub use tier::{EntityTier, Shape, CHARACTER_CEILING};
pub use tile_value::{TileValueChunk, TileValueRange, TileValues};
pub use trade::{
    Advert, Consideration, MarketTable, TradeError, TradeRow, TradeSpoken, TradeTable, ACT_ACCEPT,
    ACT_CLOSE, ACT_COUNT, ACT_COUNTER, ACT_DEFAULT, ACT_OFFER, ACT_REFUSE, ACT_REOPEN, ACT_SETTLE,
    ACT_STEP_RELATION, ACT_TRANSFER_LAND, ADVERT_BYTES, ADVERT_OFFERS, ADVERT_WANTS,
    CONSIDERATION_KIND_COUNT, DEFAULT_BOARD_ROWS, DEFAULT_LAND_LIST_BOUND, KIND_LAND,
    KIND_RELATION, KIND_RESOURCE, TRADE_BOUND, TRADE_COUNTERED, TRADE_DEFAULTED, TRADE_IDLE,
    TRADE_OFFERED, TRADE_ROW_BYTES, TRADE_SETTLED, TRADE_STATUS_COUNT,
};
pub use types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx};
pub use unit_type::{
    UnitTypeError, UnitTypeId, UnitTypeRow, UnitTypeTable, DEFAULT_UNIT_TYPE,
    DEFAULT_UNIT_TYPE_TABLE, UNIT_TYPE_COLUMN_COUNT, UNIT_TYPE_COUNT,
};
pub use upgrade::{
    UpgradeCategory, UpgradeMap, UpgradeRow, UpgradeSite, UpgradeTable, DEFAULT_UPGRADE_TABLE,
    UPGRADE_CATEGORY_COUNT, UPGRADE_LEVEL_COUNT,
};
pub use weather::{
    CellGround, Drops, Latitudes, Storm, WeatherError, WeatherField, WeatherScale, Wind,
    AIR_SATURATION, COOLDOWN_TICKS, HEAT_CEILING, LATITUDE_FINE, LATITUDE_POLE, PASS_CEILING,
    PLACES_CEILING, SPEED_CEILING, STRENGTH_CEILING, WARMTH_FINE, WARMTH_FLOOR, WET_MARK,
};
pub use world::{
    CampaignError, CensusBasis, CensusRow, ConvertError, IdentityError, MoveRelationError,
    SeedError, Standing, StepError, World, WorldConfig, WorldError, FOUNDING_GROUP_DEFAULT,
    LUXURY_DEPOSITS_DEFAULT, STOCK_CEILING_OF_ONE_SETTLEMENT, SUBSYSTEM_CENSUS,
};
