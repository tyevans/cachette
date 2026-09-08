//! The unit type table, and the calls that define and change a row.
//!
//! A unit type is data. It is an index into a shared table, and the table
//! parameterises the verbs. The readers and the writers of the table sit
//! together, and nothing else belongs beside them.

use super::World;
use crate::types::Entity;
use crate::unit_type::{UnitTypeError, UnitTypeId, UnitTypeRow, UnitTypeTable};

impl World {
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
}
