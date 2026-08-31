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
pub mod character;
pub mod event;
pub mod hash;
pub mod hex;
pub mod pyramid;
pub mod resource;
pub mod rng;
pub mod sim_math;
pub mod site;
pub mod slots;
pub mod soldier;
pub mod sort;
pub mod terrain;
pub mod tier;
pub mod types;
pub mod world;

pub use bridge::{BlockLayout, BlockRange, BridgeError, UnitTileBridge};
pub use character::{CharacterArena, CharacterError};
pub use event::{ResourceTaken, TileChanged};
pub use hash::StateHash;
pub use hex::{Axial, Grid, GridError};
pub use resource::{Amount, CarryLoad, DepletionLedger, ResourceField, ResourceKind};
pub use site::{CommodityId, SettlementArena, SettlementError, Store, COMMODITY_COUNT};
pub use slots::{Candidate, SlotError, Slots};
pub use soldier::{SoldierArena, SoldierError};
pub use sort::{SortError, SortKey};
pub use terrain::{Terrain, TerrainTile, TileKind};
pub use tier::{EntityTier, Shape, CHARACTER_CEILING};
pub use types::{Accum, Entity, FactionId, Fix32, Tick, TileIdx};
pub use world::{StepError, World, WorldConfig, WorldError};
