//! A household is derived from the dwelling slot. Nothing stores one.
//!
//! A **dwelling** is a settlement that stands on a tile. A unit carries the
//! slot of the dwelling it lives in, in the home column of the unit arena. A
//! **household** is every live unit that carries one slot.[^1]
//!
//! This module reads that column backwards. It writes nothing, it holds no
//! array of its own, and no structure anywhere stores the members of a
//! household. A stored roster would be a second declaration of where a
//! person lives, and nothing would fail when the roster and the slot
//! disagreed.[^2] The household is therefore a pure function of level 0, in
//! the way that every level above level 0 is.[^3]
//!
//! **A unit leaves a household by moving, not by a rule.** One column holds
//! one slot, so writing a new dwelling into it removes the unit from the old
//! household in the same write that adds it to the new one. No rule splits a
//! household, no rule merges one, and no rule can leave the two sides of a
//! move disagreeing, because there is only one side.
//!
//! **A household reads no descent.** Two strangers under one roof are one
//! household, and a parent and a child who live apart are two.[^1]
//!
//! The members come back in ascending slot order of the unit arena. That key
//! is a property of storage, so no thread order and no completion order can
//! reach it.[^4]
//!
//! # References
//!
//! [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
//! [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^3]: ADR-0022, level 0 is the only truth and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^4]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use crate::soldier::{SoldierArena, NO_HOME};
use crate::types::Entity;

/// Writes every live unit that lives in one dwelling into a buffer.
///
/// The dwelling is a slot of the settlement arena. The buffer is cleared
/// first, so an empty household is an empty buffer and never an error.
///
/// A unit that lives nowhere carries the value that means no home, and it
/// belongs to no household. Asking for that value gives an empty buffer
/// rather than every unit that lives nowhere, because units that live
/// nowhere do not live together.[^1]
///
/// The pass reads the arena in slot order, which is explicit and stable.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub(crate) fn residents_of(units: &SoldierArena, dwelling: u32, members: &mut Vec<Entity>) {
    members.clear();
    if dwelling == NO_HOME {
        return;
    }
    let homes = units.home_column();
    // The walk is over the live units, not over every slot. A walk over every
    // slot would give the same answer today, because the arena clears the home
    // of a slot it frees and its invariant states that a dead slot names no
    // dwelling. That makes this filter redundant and untestable through the
    // engine, and the register holds why it stays.[^1]
    //
    // [^1]: Findings register, FND-156. `docs/FINDINGS.md`
    for unit in units.iter() {
        if homes[unit.index() as usize] == dwelling {
            members.push(unit);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the case the public interface cannot reach.
    //!
    //! The world resolves a live settlement before it calls this function, so
    //! it never passes the value that means no home. A caller inside the
    //! crate could, and the answer must still be an empty household.

    use super::*;
    use crate::hex::Grid;
    use crate::types::FactionId;

    #[test]
    fn no_home_names_no_household() {
        let grid = Grid::new(8, 8).expect("the grid is inside the world");
        let mut units = SoldierArena::new(grid, 16);
        let address = grid.address_of(crate::types::TileIdx(0)).expect("tile zero");
        for _ in 0..3 {
            units
                .spawn(address, FactionId(0))
                .expect("the arena holds three units");
        }
        let mut members = vec![Entity::new(1, 1).expect("the identity is not zero")];
        residents_of(&units, NO_HOME, &mut members);
        assert!(
            members.is_empty(),
            "units that live nowhere do not live together"
        );
    }
}
