//! Turning an identity that Python handed back into an entity.
//!
//! An entity crosses to Python as one opaque identity. Python cannot take the
//! identity apart and cannot build one, so the value it gives back is a value
//! the engine gave it.[^1] That value can still be stale, and the engine
//! compares the generation.
//!
//! The resolvers sit in one module because every domain calls them. A second
//! copy of the refusal message in each domain would be one rule in many
//! places, with nothing that fails when the copies disagree.[^2]
//!
//! # References
//!
//! [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
//! [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`

use crate::errors::ViewError;
use cachette_core::{Entity, World as CoreWorld};
use pyo3::prelude::*;

/// Resolves an identity that Python handed back, or raises.
///
/// Python cannot build an identity, so the value it gives is one the engine
/// gave it. That value can still be stale. The engine compares the
/// generation, and this function turns a refusal into the typed error for a
/// stale view.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
pub(crate) fn resolve(world: &CoreWorld, unit: u64) -> PyResult<Entity> {
    world
        .resolve_soldier(unit)
        .map_err(|error| ViewError::new_err(error.to_string()))
}

/// Resolves a settlement identity that Python handed back, or raises.
///
/// The engine compares the generation, so a settlement that was lost never
/// answers for the settlement founded next in its slot.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
pub(crate) fn resolve_site(world: &CoreWorld, site: u64) -> PyResult<Entity> {
    world
        .resolve_settlement(site)
        .map_err(|error| ViewError::new_err(error.to_string()))
}

/// Resolves every settlement identity of a set, or raises on the first stale
/// one.
///
/// The whole set resolves before a caller writes anything, so one stale
/// identity leaves the world unchanged.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
pub(crate) fn resolve_sites(world: &CoreWorld, sites: &[u64]) -> PyResult<Vec<Entity>> {
    let mut resolved = Vec::with_capacity(sites.len());
    for site in sites {
        resolved.push(resolve_site(world, *site)?);
    }
    Ok(resolved)
}
