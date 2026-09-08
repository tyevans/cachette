//! The relation matrix between factions, and the moves that change it.
//!
//! A relation is one band between two factions. A move steps the band, and
//! the rules say which step is allowed. The readers, the move and the log sit
//! together, because they describe one matrix.

use super::errors::MoveRelationError;
use super::World;
use crate::relation::{RelationCrossed, RelationError, RelationRules};
use crate::types::{Entity, FactionId};

impl World {
    /// Returns what one faction feels toward another, or `None` when a
    /// number names no faction of this world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub fn relation(&self, from: FactionId, to: FactionId) -> Option<i32> {
        self.relations.get(from, to)
    }

    /// Returns the band number of what one faction feels toward another: how
    /// many of the edges lie at or below the value. Zero is the war band.
    #[must_use]
    pub fn relation_band(&self, from: FactionId, to: FactionId) -> Option<u8> {
        self.relations.band(from, to)
    }

    /// Reports whether either of two factions is in the war band toward the
    /// other.
    #[must_use]
    pub fn at_war(&self, a: FactionId, b: FactionId) -> bool {
        self.relations.war_between(a, b)
    }

    /// Writes what one faction feels toward another, outright.
    ///
    /// This is the caller's own path and it holds no gate. A crossing of the
    /// war edge is logged as any other cause logs it. Returns `false` when a
    /// number names no faction or the pair is one faction.
    pub fn set_relation(&mut self, from: FactionId, to: FactionId, value: i32) -> bool {
        self.relations.write(self.tick, from, to, value).is_some()
    }

    /// Returns the edges and the steps the relation reads.
    #[must_use]
    pub const fn relation_rules(&self) -> RelationRules {
        self.relations.rules()
    }

    /// Replaces the edges and the steps the relation reads.
    pub const fn set_relation_rules(&mut self, rules: RelationRules) {
        self.relations.set_rules(rules);
    }

    /// Moves what the faction of a speaker unit feels toward another faction
    /// by a bounded step.
    ///
    /// **The verb refuses a speaker whose type has a command reach of zero.**
    /// The gate reads the type column of the unit and no per-faction
    /// flag.[^1] It refuses a step above the bound in either direction, and
    /// the bound is a register row.[^2] A leader may always declare, so the
    /// verb reads no band before it moves.
    ///
    /// Returns the value after the move.
    ///
    /// # Errors
    ///
    /// Returns an error when the speaker is dead, when the other number names
    /// no faction, when the other faction is the speaker's own, when the
    /// speaker's type has no command reach, and when the step is above the
    /// bound.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    pub fn move_relation(
        &mut self,
        speaker: Entity,
        other: FactionId,
        step: i32,
    ) -> Result<i32, MoveRelationError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let faction = self.move_relation_refusal(speaker, other, step)?;
        self.relations
            .shift(self.tick, faction, other, step)
            .ok_or(MoveRelationError::Relation(RelationError::SameFaction))
    }

    /// Reports whether the relation verb would refuse one move, without
    /// moving anything.
    ///
    /// **This is the one statement of the rule.** The relation verb calls it
    /// before it shifts a row, and the legality answer calls it to fill one
    /// row of the action table.[^1] Returns the faction of the speaker, so
    /// that the verb does not read it twice.
    ///
    /// # Errors
    ///
    /// Returns the refusal the relation verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn move_relation_refusal(
        &self,
        speaker: Entity,
        other: FactionId,
        step: i32,
    ) -> Result<FactionId, MoveRelationError> {
        let (Some(faction), Some(unit_type)) = (
            self.soldiers.faction(speaker),
            self.soldiers.unit_type(speaker),
        ) else {
            return Err(MoveRelationError::DeadUnit(speaker));
        };
        if other.0 >= self.config.faction_count.max(1) {
            return Err(RelationError::NoSuchFaction(other.0).into());
        }
        if other == faction {
            return Err(RelationError::SameFaction.into());
        }
        if self.unit_types.row(unit_type).command_reach == 0 {
            return Err(RelationError::NoCommandReach.into());
        }
        let bound = self.relations.rules().move_bound;
        if step > bound || step < -bound {
            return Err(RelationError::StepAboveBound { step, bound }.into());
        }
        Ok(faction)
    }

    /// Returns the crossings of the war edge on the last step, in the order
    /// they happened.
    #[must_use]
    pub fn relation_log(&self) -> &[RelationCrossed] {
        self.relations.log()
    }

    /// Returns the relation log as bytes. The thread-count equivalence test
    /// compares this slice byte for byte.
    #[must_use]
    pub fn relation_log_bytes(&self) -> &[u8] {
        self.relations.log_bytes()
    }
}
