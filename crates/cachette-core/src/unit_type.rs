//! The unit type, and the table it indexes.
//!
//! A unit type is an index into a shared table. The table is data that the
//! world is built with. It holds no code, and a lookup in it is not a
//! callback.[^1]
//!
//! A type parameterises a verb. It does not multiply the verbs, so no pass
//! branches on a type name and no pass holds a rule for one type.[^1]
//!
//! Every value in the table is a fixed-point number in the project scale. No
//! value here is a floating point number.[^2]
//!
//! # References
//!
//! [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::types::Fix32;

/// The number of rows that the unit type table holds.
///
/// **This is the width of the table and not a budget.** The resolution of a
/// meeting reads the table for each ordered pair of types, so its cost follows
/// the square of this number. The number is therefore a structural property of
/// the design: it is small so that the square stays small, and it is fixed so
/// that no world pays for a table it did not fill.[^1]
///
/// A row that nobody filled holds zero in every field, and a type of zero
/// attack reaches nothing.
///
/// # References
///
/// [^1]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
pub const UNIT_TYPE_COUNT: usize = 8;

/// The type that a unit carries until something gives it another.
///
/// Row zero of an unfilled table holds zero attack and zero armour, so a
/// world that never filled the table holds units that reach nothing and that
/// nothing reaches.
pub const DEFAULT_UNIT_TYPE: UnitTypeId = UnitTypeId(0);

/// The reason that the table refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitTypeError {
    /// The number names no row of the table.
    TypeAboveCeiling(u8),
    /// The attack value is below zero. An attack is a quantity of harm and a
    /// negative one would heal.
    AttackBelowZero(Fix32),
    /// The armour value is below zero. An armour is a threshold and a
    /// negative one is below every attack, including no attack at all.
    ArmourBelowZero(Fix32),
}

impl core::fmt::Display for UnitTypeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TypeAboveCeiling(value) => write!(
                formatter,
                "the unit type {value} is at or above the ceiling {UNIT_TYPE_COUNT}"
            ),
            Self::AttackBelowZero(value) => {
                write!(formatter, "the attack {} is below zero", value.0)
            }
            Self::ArmourBelowZero(value) => {
                write!(formatter, "the armour {} is below zero", value.0)
            }
        }
    }
}

impl std::error::Error for UnitTypeError {}

/// The type of a unit, as an index into the shared table.
///
/// The type is one byte, because the table is small and fixed. It is a
/// newtype, so no other one-byte value substitutes for it in silence.[^1]
///
/// # References
///
/// [^1]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitTypeId(pub u8);

impl UnitTypeId {
    /// Returns the type that a number names, or `None` when the number names
    /// no row of the table.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < UNIT_TYPE_COUNT {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the row index of the type.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// One row of the unit type table.
///
/// The row is plain data with a declared layout, so a copy of the table
/// enters the state hash byte for byte and carries no uninitialised byte.[^1]
///
/// The layout is 4 + 4 bytes, which is 8 bytes at an alignment of 4. Every
/// byte is a declared field, so the type holds no padding at all.
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitTypeRow {
    /// The harm that one unit of this type delivers in one resolution.
    ///
    /// The value is a whole number of casualties for each attacker, in the
    /// project fixed-point scale, so a value of one half means that two
    /// attackers of this type end one defender.
    pub attack: Fix32,
    /// The attack that an attacker must exceed to reach a unit of this type.
    ///
    /// The value is in the same scale as the attack, so the comparison is
    /// exact and the two never mean different things.
    pub armour: Fix32,
}

/// The shared table that a unit type indexes.
///
/// The table is dense and its length never changes. A caller fills the rows
/// it wants and leaves the rest at zero.[^1]
///
/// # References
///
/// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitTypeTable {
    rows: [UnitTypeRow; UNIT_TYPE_COUNT],
}

impl UnitTypeTable {
    /// Builds a table whose every row holds zero.
    ///
    /// A world built with this table holds no contest, because no attack
    /// exceeds any armour when both are zero.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rows: [UnitTypeRow {
                attack: Fix32::ZERO,
                armour: Fix32::ZERO,
            }; UNIT_TYPE_COUNT],
        }
    }

    /// Returns every row, in type order.
    #[must_use]
    pub const fn rows(&self) -> &[UnitTypeRow; UNIT_TYPE_COUNT] {
        &self.rows
    }

    /// Returns one row of the table.
    #[must_use]
    pub const fn row(&self, unit_type: UnitTypeId) -> UnitTypeRow {
        self.rows[unit_type.index()]
    }

    /// Writes one row of the table.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row, when the attack is
    /// below zero, or when the armour is below zero.
    pub const fn define(
        &mut self,
        unit_type: u8,
        attack: Fix32,
        armour: Fix32,
    ) -> Result<(), UnitTypeError> {
        let Some(row) = UnitTypeId::from_u8(unit_type) else {
            return Err(UnitTypeError::TypeAboveCeiling(unit_type));
        };
        if attack.0 < 0 {
            return Err(UnitTypeError::AttackBelowZero(attack));
        }
        if armour.0 < 0 {
            return Err(UnitTypeError::ArmourBelowZero(armour));
        }
        self.rows[row.index()] = UnitTypeRow { attack, armour };
        Ok(())
    }

    /// Reports whether an attacker of one type reaches a defender of another.
    ///
    /// **This is the one statement of the penetration threshold.** The attack
    /// of the attacker must exceed the armour of the defender. An attack that
    /// equals the armour does not exceed it and does not reach.[^1]
    ///
    /// The pass calls this once for each ordered pair of types, before it
    /// aggregates anything, so a pair that does not reach contributes exactly
    /// zero however many attackers stand on the tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
    #[must_use]
    pub const fn penetrates(&self, attacker: UnitTypeId, defender: UnitTypeId) -> bool {
        self.rows[attacker.index()].attack.0 > self.rows[defender.index()].armour.0
    }

    /// Absorbs the table into the state hash.
    ///
    /// The table decides what a later frame does, so the whole-world hash
    /// covers it. Two worlds that hold the same units and different tables
    /// must diverge at the next meeting.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(bytemuck::cast_slice(&self.rows))
    }

    /// Reports whether the table holds its invariants.
    ///
    /// No row holds a negative attack and no row holds a negative armour.
    /// The writer refuses both, and this is what fails when a value reaches
    /// the table by another path.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        self.rows
            .iter()
            .all(|row| row.attack.0 >= 0 && row.armour.0 >= 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unfilled_table_reaches_nothing() {
        let table = UnitTypeTable::empty();
        for attacker in 0..UNIT_TYPE_COUNT as u8 {
            for defender in 0..UNIT_TYPE_COUNT as u8 {
                let attacker = UnitTypeId::from_u8(attacker).expect("the number names a row");
                let defender = UnitTypeId::from_u8(defender).expect("the number names a row");
                assert!(!table.penetrates(attacker, defender));
            }
        }
    }

    #[test]
    fn an_attack_that_equals_the_armour_does_not_reach() {
        let mut table = UnitTypeTable::empty();
        table
            .define(0, Fix32::from_int(4), Fix32::ZERO)
            .expect("the row is inside the table");
        table
            .define(1, Fix32::ZERO, Fix32::from_int(4))
            .expect("the row is inside the table");
        let bowman = UnitTypeId(0);
        let tank = UnitTypeId(1);
        assert!(
            !table.penetrates(bowman, tank),
            "an attack that equals the armour must not exceed it"
        );
    }

    #[test]
    fn the_table_refuses_a_row_it_does_not_hold() {
        let mut table = UnitTypeTable::empty();
        let refused = table.define(UNIT_TYPE_COUNT as u8, Fix32::ONE, Fix32::ZERO);
        assert_eq!(
            refused,
            Err(UnitTypeError::TypeAboveCeiling(UNIT_TYPE_COUNT as u8))
        );
    }

    #[test]
    fn the_table_refuses_a_negative_value() {
        let mut table = UnitTypeTable::empty();
        assert_eq!(
            table.define(0, Fix32(-1), Fix32::ZERO),
            Err(UnitTypeError::AttackBelowZero(Fix32(-1)))
        );
        assert_eq!(
            table.define(0, Fix32::ZERO, Fix32(-1)),
            Err(UnitTypeError::ArmourBelowZero(Fix32(-1)))
        );
        assert!(table.check_invariants());
    }
}
