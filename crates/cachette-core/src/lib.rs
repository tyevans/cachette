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

pub mod event;
pub mod hash;
pub mod hex;
pub mod rng;
pub mod sim_math;
pub mod slots;
pub mod sort;
pub mod types;
pub mod world;

pub use event::TileChanged;
pub use hash::StateHash;
pub use hex::{Axial, Grid, GridError};
pub use slots::{Candidate, SlotError, Slots};
pub use sort::{SortError, SortKey};
pub use types::{Accum, Entity, Fix32, Tick, TileIdx};
pub use world::{StepError, World, WorldConfig, WorldError};
