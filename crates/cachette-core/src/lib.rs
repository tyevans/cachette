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

pub mod bridge;
pub mod census;
pub mod character;
pub mod choose;
pub mod cohort;
pub mod contest;
pub mod descent;
pub mod event;
pub mod founding;
pub mod hash;
pub mod hex;
pub mod holding;
mod household;
pub mod influence;
pub mod luxury;
pub mod position;
pub mod presence;
pub mod promotion;
pub mod pyramid;
pub mod rates;
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
pub use contest::{ContestError, UnitFell};
pub use descent::{
    Descent, DescentError, DescentId, HouseId, Parents, DESCENT_CEILING, RELATION_DEPTH,
};
pub use event::{ResourceTaken, TileChanged};
pub use founding::{Founding, FoundingError, FoundingOutcome, Provision, Survey};
pub use hash::StateHash;
pub use hex::{Axial, Grid, GridError};
pub use holding::{FactionMask, Holder, Holding};
pub use influence::{Conductance, Influence, InfluenceError, InfluenceField};
pub use luxury::{
    LuxuryError, LuxuryField, LuxuryId, LuxurySet, LuxuryTile, VarietyLevel, LUXURY_CEILING,
};
pub use position::{
    release_the_dead, Position, PositionError, PositionTable, SitePreference, NO_WORK,
    POSITIONS_PER_SITE, WORK_COMMODITY,
};
pub use presence::{PresenceRelation, PRESENCE_ROWS};
pub use pyramid::{CellSummary, ExitField, Pyramid, NO_EXIT};
pub use rates::{
    RateError, RateLedger, RatePass, RateSchedule, RateTable, SiteRate, SiteShortfall,
};
pub use resource::{
    Amount, CarryLoad, DepletionLedger, LedgerEntry, RecoveryRules, ResourceField, ResourceKind,
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
    TradeError, TradeRow, TradeSpoken, TradeTable, ACT_ACCEPT, ACT_CLOSE, ACT_COUNT, ACT_COUNTER,
    ACT_DEFAULT, ACT_OFFER, ACT_REFUSE, ACT_REOPEN, ACT_SETTLE, TRADE_BOUND, TRADE_COUNTERED,
    TRADE_DEFAULTED, TRADE_IDLE, TRADE_OFFERED, TRADE_ROW_BYTES, TRADE_SETTLED, TRADE_STATUS_COUNT,
};
pub use types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx};
pub use unit_type::{
    UnitTypeError, UnitTypeId, UnitTypeRow, UnitTypeTable, DEFAULT_UNIT_TYPE, UNIT_TYPE_COUNT,
};
pub use upgrade::{UpgradeKind, UpgradeMap, UpgradeSite, UPGRADE_KIND_COUNT};
pub use weather::{
    Drops, Storm, WeatherError, WeatherField, COOLDOWN_TICKS, PLACES_CEILING, STRENGTH_CEILING,
    WET_MARK,
};
pub use world::{IdentityError, StepError, World, WorldConfig, WorldError};
