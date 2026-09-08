//! The household of a unit, and the dwelling that holds it.
//!
//! A household ties a unit to a home site. The readers answer where a unit
//! lives and who lives with it. The subject is small and self-contained.

use super::World;
use crate::household;
use crate::types::Entity;

impl World {
    /// Gives a unit the site that it draws from, and reports whether it
    /// wrote.
    ///
    /// Returns `false` when either identity is dead. A unit that belongs to
    /// no site draws from nothing, and `None` puts it in that state.
    ///
    /// The world holds this call rather than the arena, because the arena
    /// holds no settlement column and cannot tell a live site from a dead
    /// one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_home_site(&mut self, soldier: Entity, site: Option<Entity>) -> bool {
        let home = match site {
            Some(site) => match self.settlements.slot_of(site) {
                Some(slot) => Some(slot),
                None => return false,
            },
            None => None,
        };
        self.soldiers.set_home(soldier, home)
    }

    /// Returns the dwelling that one unit lives in.
    ///
    /// Returns `None` when the identity is dead. Returns `Some(None)` when
    /// the unit lives nowhere, which is a state the world represents rather
    /// than an error.[^1]
    ///
    /// A unit lives where it draws from. The record that fixes a settlement
    /// to a tile and gives it the pooled store makes those one fact, so the
    /// world holds one column for both and never two.[^2] [^3]
    ///
    /// The world invariant holds that every home names a live settlement, so
    /// the second `None` means the unit lives nowhere and never means that
    /// its dwelling was lost.[^4]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: Findings register, FND-116. `docs/FINDINGS.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn dwelling_of(&self, unit: Entity) -> Option<Option<Entity>> {
        let home = self.soldiers.home(unit)?;
        Some(home.and_then(|slot| self.settlements.entity_at(slot)))
    }

    /// Returns every unit that lives in one dwelling.
    ///
    /// A household is derived. Nothing stores one, and no rule declares
    /// one.[^1] The members are every live unit whose home column entry names
    /// this dwelling, so a unit that takes a dwelling of its own leaves the
    /// household it was in by the same write that puts it in the new one.
    ///
    /// Returns `None` when the identity is dead. A dwelling that nobody lives
    /// in returns an empty list, which is an answer and not an error.
    ///
    /// The members come back in ascending slot order of the unit arena. That
    /// key is a property of storage, so no thread order reaches it.[^2]
    ///
    /// The call passes over the unit arena. A watcher that wants the
    /// headcount of a place reads the cohort table instead, which holds it
    /// per site and per faction without a pass.[^3]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
    /// [^2]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[must_use]
    pub fn household_of(&self, dwelling: Entity) -> Option<Vec<Entity>> {
        let mut members = Vec::new();
        if self.household_into(dwelling, &mut members) {
            Some(members)
        } else {
            None
        }
    }

    /// Writes every unit that lives in one dwelling into a buffer the caller
    /// owns, and reports whether the dwelling resolved.
    ///
    /// The answer is the answer of [`Self::household_of`]. A caller that asks
    /// about many dwellings hands the same buffer to each call rather than
    /// taking a new one each time. The buffer is cleared on every call,
    /// including the call that refuses, so a stale roster never survives a
    /// dead identity.
    pub fn household_into(&self, dwelling: Entity, members: &mut Vec<Entity>) -> bool {
        members.clear();
        let Some(slot) = self.settlements.slot_of(dwelling) else {
            return false;
        };
        // The identity resolved, so the slot must be live. The check is local,
        // because a reader that trusts an argument across two structures is
        // how a dead dwelling comes back holding a roster.
        if self.settlements.live_column().get(slot as usize) != Some(&1) {
            return false;
        }
        household::residents_of(&self.soldiers, slot, members);
        true
    }
}
