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
//! [^1]: ADR-0006, The Python boundary, decision D2. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
//! [^2]: ADR-0001, Determinism as the primary constraint, decision D3. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
//! [^3]: ADR-0001, Determinism as the primary constraint, decision D2. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`

pub mod event;
pub mod hash;
pub mod rng;
pub mod sim_math;
pub mod types;
pub mod world;

pub use event::TileChanged;
pub use hash::StateHash;
pub use types::{Accum, Entity, Fix32, Tick, TileIdx};
pub use world::{StepError, World, WorldConfig};
