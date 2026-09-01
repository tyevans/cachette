//! The tier that each entity shape declares.
//!
//! The entity storage holds four fixed shapes, and each shape gets its own
//! set of columns.[^1] The shapes do not vary at run time, so a shape that
//! is not one of the four is a compile-time error and never a row in a
//! table.[^2]
//!
//! Every shape also belongs to one of three tiers. The tier says how a
//! caller may reach the population of that shape. A mass tier holds more
//! entities than a script may walk, so a caller sends one selector and the
//! engine resolves it. A character tier holds a bounded population, so a
//! caller may walk it. A singleton tier holds one thing.
//!
//! **The tier is a property of the shape, not of the current count.** A
//! check on the count makes the same script work on a small world and fail
//! on a large one. The failure then appears far from its cause, and it
//! appears only at scale.[^3]
//!
//! The tier is an associated constant on a sealed trait. A caller outside
//! this crate cannot declare a shape or a tier, and no caller inside it can
//! choose one while the engine runs.
//!
//! A tier states the ceiling of the population it admits. The character
//! ceiling is a derived figure, and the scale constants table holds it.[^4]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^3]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D2. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
//! [^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`

use crate::character::CharacterArena;
use crate::site::SettlementArena;
use crate::soldier::SoldierArena;

/// The largest living character population that the project supports.
///
/// The figure is derived from the cost of the control-plane decision pass,
/// and the scale constants table holds it with that derivation.[^1] Every
/// cost figure in this project is derived and not measured.[^2]
///
/// # References
///
/// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const CHARACTER_CEILING: u32 = 262_144;

/// The tier that a shape belongs to.
///
/// A shape declares its tier once, at the type. The tier never changes
/// while the engine runs.[^1]
///
/// # References
///
/// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityTier {
    /// The population is larger than a caller may walk. A caller sends a
    /// selector, and the engine resolves it.
    Mass,
    /// The population is bounded, so a caller may walk it.
    Character,
    /// The shape holds one thing.
    Singleton,
}

impl EntityTier {
    /// Returns the largest population that the tier admits.
    ///
    /// Returns `None` for the mass tier. The limit of a mass shape is the
    /// range of the slot index, which is a property of the identity layout
    /// and not a budget.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub const fn population_ceiling(self) -> Option<u32> {
        match self {
            Self::Mass => None,
            Self::Character => Some(CHARACTER_CEILING),
            Self::Singleton => Some(1),
        }
    }
}

impl core::fmt::Display for EntityTier {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            Self::Mass => "mass",
            Self::Character => "character",
            Self::Singleton => "singleton",
        };
        formatter.write_str(name)
    }
}

/// The private supertrait that seals [`Shape`].
mod sealed {
    /// The marker that only this crate implements.
    pub trait Sealed {}
    impl Sealed for crate::soldier::SoldierArena {}
    impl Sealed for crate::site::SettlementArena {}
    impl Sealed for crate::character::CharacterArena {}
}

/// One of the fixed entity shapes.
///
/// The trait is sealed, so no caller outside this crate declares a shape or
/// a tier. A component set that a caller assembles is a compile-time error
/// and not a run-time refusal.[^1]
///
/// # References
///
/// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
pub trait Shape: sealed::Sealed {
    /// The tier that this shape belongs to.
    const TIER: EntityTier;
}

impl Shape for SoldierArena {
    /// A soldier is one of a million, so no caller walks the population.
    const TIER: EntityTier = EntityTier::Mass;
}

impl Shape for SettlementArena {
    /// A settlement population is smaller than a soldier population, and
    /// the mass tier is the stricter of the two answers. A shape declares
    /// the stricter tier that its population admits. Widening a tier later
    /// keeps every script that already works. Narrowing one does not.
    const TIER: EntityTier = EntityTier::Mass;
}

impl Shape for CharacterArena {
    /// A living character population is bounded, and the bound is a
    /// declared ceiling and not a count that the world happens to reach.
    const TIER: EntityTier = EntityTier::Character;
}
