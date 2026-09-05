//! The graded relation between two factions.
//!
//! # Why this exists
//!
//! Two factions in this engine were at war whenever they touched. The contest
//! pass resolved a meeting wherever two factions stood beside each other, and
//! nothing gated it. A game needs alliance, peace, tension and war, and it
//! needs a faction to move from one to another by what happens in the
//! world.[^1]
//!
//! # What it holds
//!
//! One signed integer for each ordered pair of factions, in a dense matrix.
//! The entry for the pair (A, B) is what A feels toward B, and the pair (B, A)
//! is a separate entry. The matrix is simulated state: a later frame reads it,
//! so it enters the state hash.[^1] [^2]
//!
//! A band is a threshold. A pass that asks whether A is at war with B compares
//! the integer to the war edge. The edges, every step, the drift schedule and
//! the verb bound are rows in the balance register, and the constants here
//! carry the provisional values those rows record.[^3]
//!
//! # What moves it
//!
//! Every cause moves the relation by an integer step. A delivered contract
//! raises both directions. A failed contract lowers the party that was owed
//! toward the defaulter. A unit that falls lowers its faction toward the
//! killer. A unit that converts lowers its old faction toward the leader. A
//! drift moves each entry one step toward the peace band on a schedule. One
//! verb moves an entry by a bounded step, and it refuses a speaker without
//! command reach.[^1] [^4]
//!
//! **One function writes an entry, and it logs the crossing.** Every cause
//! comes through `write`, so a crossing of the war edge is logged exactly once
//! however it happened.[^5]
//!
//! No item in this module uses a floating-point type.[^6]
//!
//! # References
//!
//! [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
//! [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^3]: Balance register, the relation. `docs/reference/balance.md`
//! [^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^5]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::rates::RateSchedule;
use crate::sim_math;
use crate::types::{FactionId, Tick};

/// The edge at or above which a relation is an alliance.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row, marks it unset, and records how the value was
/// chosen.[^1]
///
/// # References
///
/// [^1]: Balance register, the alliance edge. `docs/reference/balance.md`
pub const ALLIANCE_EDGE_DEFAULT: i32 = 8;

/// The edge at or above which a relation is peace. Provisional, as above.[^1]
///
/// Every entry of a new world holds this value, so two factions that never
/// met are at peace and the contest resolves nothing between them.
///
/// # References
///
/// [^1]: Balance register, the peace edge. `docs/reference/balance.md`
pub const PEACE_EDGE_DEFAULT: i32 = 0;

/// The edge below which a relation is war. Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the war edge. `docs/reference/balance.md`
pub const WAR_EDGE_DEFAULT: i32 = -8;

/// The step a contract delivered in full moves both directions up by.
/// Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the step on a contract delivered in full. `docs/reference/balance.md`
pub const STEP_CONTRACT_DELIVERED_DEFAULT: i32 = 2;

/// The step a failed contract moves the owed party down by. Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the step on a contract that fails. `docs/reference/balance.md`
pub const STEP_CONTRACT_FAILED_DEFAULT: i32 = 2;

/// The step one fallen unit moves its faction down toward the killer by.
/// Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the step when a unit falls to the other side. `docs/reference/balance.md`
pub const STEP_UNIT_FELL_DEFAULT: i32 = 1;

/// The step one converted unit moves its old faction down toward the leader
/// by. Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the step when a unit converts away. `docs/reference/balance.md`
pub const STEP_UNIT_CONVERTED_DEFAULT: i32 = 1;

/// The step a storm on the ground of the other moves the holder down by.
///
/// **This row is unset and behind a blocker, and nothing reads it.** A god
/// inflicts weather only on ground its own faction holds, so no source for the
/// cause exists before the weather harm pass.[^1] [^2]
///
/// # References
///
/// [^1]: Balance register, the step when a storm falls on the ground of the other. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
pub const STEP_STORM_DEFAULT: i32 = 0;

/// The step the drift moves an entry toward the peace band by. Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the drift step toward peace. `docs/reference/balance.md`
pub const STEP_DRIFT_DEFAULT: i32 = 1;

/// The schedule the drift runs on. Provisional.[^1]
///
/// It is the shape the economy and the position schedules use: a period and a
/// phase, and the drift runs on the ticks the schedule names.
///
/// # References
///
/// [^1]: Balance register, the drift schedule. `docs/reference/balance.md`
pub const DRIFT_SCHEDULE_DEFAULT: RateSchedule = RateSchedule::DEFAULT;

/// The largest step one call of the relation verb moves an entry by, in
/// either direction. Provisional.[^1]
///
/// # References
///
/// [^1]: Balance register, the bound on one `move_relation` step. `docs/reference/balance.md`
pub const MOVE_BOUND_DEFAULT: i32 = 4;

/// The number of bands the edges cut the integer range into.
pub const BAND_COUNT: u8 = 4;

/// The edges and the steps the relation reads.
///
/// Every field is a parameter of the world and enters the hash, because the
/// step reads each one on every tick and two worlds that differ in one must
/// diverge.[^1] The conversion edge and the guest edge are the two permitted
/// bands the register names as rows of their own.[^2]
///
/// The layout is twelve four-byte fields, so it holds no padding byte.
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: Balance register, the relation. `docs/reference/balance.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct RelationRules {
    /// The edge at or above which a relation is an alliance.
    pub alliance_edge: i32,
    /// The edge at or above which a relation is peace.
    pub peace_edge: i32,
    /// The edge below which a relation is war.
    pub war_edge: i32,
    /// The edge below which the leading faction may convert a unit of the
    /// other. A leader at peace with a faction converts none of its units.
    pub conversion_edge: i32,
    /// The edge below which a holder refuses a guest onto the ground it holds.
    pub guest_edge: i32,
    /// The step on a contract delivered in full.
    pub contract_delivered: i32,
    /// The step on a contract that fails.
    pub contract_failed: i32,
    /// The step for each unit that falls.
    pub unit_fell: i32,
    /// The step for each unit that converts away.
    pub unit_converted: i32,
    /// The step the drift moves toward the peace band by.
    pub drift: i32,
    /// The largest step the verb takes in one call.
    pub move_bound: i32,
    /// The period of the drift schedule.
    pub drift_period: u32,
    /// The phase of the drift schedule.
    pub drift_phase: u32,
    /// The step a storm on the ground of the other moves the holder by.
    /// Unset and unread until the weather harm pass wires the cause.
    pub storm: i32,
}

impl RelationRules {
    /// The rules a world starts with. Every value is the provisional default
    /// the balance register records.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the relation. `docs/reference/balance.md`
    pub const DEFAULT: Self = Self {
        alliance_edge: ALLIANCE_EDGE_DEFAULT,
        peace_edge: PEACE_EDGE_DEFAULT,
        war_edge: WAR_EDGE_DEFAULT,
        conversion_edge: PEACE_EDGE_DEFAULT,
        guest_edge: PEACE_EDGE_DEFAULT,
        contract_delivered: STEP_CONTRACT_DELIVERED_DEFAULT,
        contract_failed: STEP_CONTRACT_FAILED_DEFAULT,
        unit_fell: STEP_UNIT_FELL_DEFAULT,
        unit_converted: STEP_UNIT_CONVERTED_DEFAULT,
        drift: STEP_DRIFT_DEFAULT,
        move_bound: MOVE_BOUND_DEFAULT,
        drift_period: DRIFT_SCHEDULE_DEFAULT.period(),
        drift_phase: DRIFT_SCHEDULE_DEFAULT.phase(),
        storm: STEP_STORM_DEFAULT,
    };

    /// Returns the band number of a value: how many edges lie at or below it.
    ///
    /// Zero is below the war edge, one is at or above it and below the peace
    /// edge, two is at or above the peace edge and below the alliance edge,
    /// and three is at or above the alliance edge. The number counts edges,
    /// so a caller that wants a name supplies its own.
    #[must_use]
    pub const fn band_of(&self, value: i32) -> u8 {
        (value >= self.war_edge) as u8
            + (value >= self.peace_edge) as u8
            + (value >= self.alliance_edge) as u8
    }

    /// Returns the drift schedule.
    #[must_use]
    pub fn drift_schedule(&self) -> Option<RateSchedule> {
        RateSchedule::new(self.drift_period, self.drift_phase)
    }
}

/// The reason the relation verb refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationError {
    /// The number names no faction of this world.
    NoSuchFaction(u16),
    /// The speaker addressed its own faction.
    SameFaction,
    /// The type of the speaker unit has a command reach of zero.
    NoCommandReach,
    /// The step is above the bound in one direction or the other.
    StepAboveBound { step: i32, bound: i32 },
}

impl core::fmt::Display for RelationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchFaction(faction) => {
                write!(formatter, "{faction} names no faction of this world")
            }
            Self::SameFaction => write!(formatter, "a faction holds no relation toward itself"),
            Self::NoCommandReach => write!(
                formatter,
                "the speaker's type has a command reach of zero, so it cannot move a relation"
            ),
            Self::StepAboveBound { step, bound } => {
                write!(formatter, "the step {step} is above the bound of {bound}")
            }
        }
    }
}

impl std::error::Error for RelationError {}

/// A relation crossed the war edge.
///
/// The event reports one ordered pair whose entry crossed the war edge in one
/// direction or the other on the last step. A watcher reads the log to see a
/// declaration or a peace. The bands are numbers as `RelationRules::band_of`
/// numbers them.[^1]
///
/// The layout is 8 + 2 + 2 + 1 + 1 + 2 bytes, which is 16 bytes at an
/// alignment of 8. The trailing array declares every padding byte, so the type
/// holds no uninitialised byte.[^2]
///
/// # References
///
/// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D6. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
/// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct RelationCrossed {
    /// The tick at which the entry crossed.
    pub tick: Tick,
    /// The faction whose feeling moved.
    pub from_faction: FactionId,
    /// The faction it feels toward.
    pub to_faction: FactionId,
    /// The band number before the move.
    pub band_before: u8,
    /// The band number after the move.
    pub band_after: u8,
    /// The declared padding. Always zero.
    pub padding: [u8; 2],
}

impl RelationCrossed {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        from_faction: FactionId,
        to_faction: FactionId,
        band_before: u8,
        band_after: u8,
    ) -> Self {
        Self {
            tick,
            from_faction,
            to_faction,
            band_before,
            band_after,
            padding: [0; 2],
        }
    }

    /// Reports whether the crossing went into the war band.
    #[must_use]
    pub const fn is_declaration(&self) -> bool {
        self.band_after < self.band_before
    }
}

/// The relation between every ordered pair of factions.
#[derive(Clone, Debug)]
pub struct RelationMatrix {
    factions: u16,
    /// The entry for (A, B) is at `A * factions + B`.
    entries: Vec<i32>,
    rules: RelationRules,
    log: Vec<RelationCrossed>,
}

impl RelationMatrix {
    /// Builds the matrix for a world. Every entry starts at the peace edge.
    #[must_use]
    pub fn new(faction_count: u16) -> Self {
        let factions = faction_count.max(1);
        let count = usize::from(factions) * usize::from(factions);
        Self {
            factions,
            entries: vec![RelationRules::DEFAULT.peace_edge; count],
            rules: RelationRules::DEFAULT,
            log: Vec::new(),
        }
    }

    /// Returns how many factions the matrix covers.
    #[must_use]
    pub const fn factions(&self) -> u16 {
        self.factions
    }

    /// Returns the rules.
    #[must_use]
    pub const fn rules(&self) -> RelationRules {
        self.rules
    }

    /// Replaces the rules. The entries stay where they are.
    pub const fn set_rules(&mut self, rules: RelationRules) {
        self.rules = rules;
    }

    const fn index_of(&self, from: FactionId, to: FactionId) -> Option<usize> {
        if from.0 >= self.factions || to.0 >= self.factions {
            return None;
        }
        Some(from.0 as usize * self.factions as usize + to.0 as usize)
    }

    /// Returns what one faction feels toward another, or `None` when a
    /// number names no faction.
    #[must_use]
    pub fn get(&self, from: FactionId, to: FactionId) -> Option<i32> {
        self.entries.get(self.index_of(from, to)?).copied()
    }

    /// Returns the band number of what one faction feels toward another.
    #[must_use]
    pub fn band(&self, from: FactionId, to: FactionId) -> Option<u8> {
        Some(self.rules.band_of(self.get(from, to)?))
    }

    /// Reports whether either of the pair is in the war band toward the
    /// other. The contest resolves a meeting only when this holds.[^1]
    ///
    /// A number that names no faction is at war with nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub fn war_between(&self, a: FactionId, b: FactionId) -> bool {
        if a == b {
            return false;
        }
        let war = self.rules.war_edge;
        matches!(self.get(a, b), Some(value) if value < war)
            || matches!(self.get(b, a), Some(value) if value < war)
    }

    /// Reports whether the leading faction may convert a unit of another.
    ///
    /// **A leader converts only where it is below the conversion edge toward
    /// the faction of the unit.** The edge is the peace edge by default, so a
    /// leader at peace with a faction converts none of its units, and a
    /// leader in tension or at war converts as the field says. The choice
    /// keeps a peaceful border from bleeding units, and the register row
    /// holds the edge.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the permitted bands for a conversion. `docs/reference/balance.md`
    #[must_use]
    pub fn permits_conversion(&self, leader: FactionId, target: FactionId) -> bool {
        if leader == target {
            return false;
        }
        matches!(self.get(leader, target), Some(value) if value < self.rules.conversion_edge)
    }

    /// Reports whether a holder refuses a guest onto the ground it holds.
    ///
    /// **This is the one statement of the movement rule.** A holder refuses a
    /// guest of another faction when the holder is below the guest edge toward
    /// the guest. A guest of the holder's own faction is never refused, and
    /// nobody refuses on ground nobody holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the band below which a holder refuses a guest. `docs/reference/balance.md`
    #[must_use]
    pub fn refuses_guest(&self, holder: FactionId, guest: FactionId) -> bool {
        if holder == guest {
            return false;
        }
        matches!(self.get(holder, guest), Some(value) if value < self.rules.guest_edge)
    }

    /// Reports whether two factions may open a trade. An offer is refused
    /// when either side is in the war band toward the other.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    #[must_use]
    pub fn permits_offer(&self, a: FactionId, b: FactionId) -> bool {
        !self.war_between(a, b)
    }

    /// Writes one entry and logs a crossing of the war edge.
    ///
    /// **Every cause comes through here.** Returns the value written, or
    /// `None` when a number names no faction or the pair is one faction.
    pub fn write(&mut self, tick: Tick, from: FactionId, to: FactionId, value: i32) -> Option<i32> {
        if from == to {
            return None;
        }
        let index = self.index_of(from, to)?;
        let before = self.entries[index];
        self.entries[index] = value;
        let war = self.rules.war_edge;
        if (before < war) != (value < war) {
            self.log.push(RelationCrossed::new(
                tick,
                from,
                to,
                self.rules.band_of(before),
                self.rules.band_of(value),
            ));
        }
        Some(value)
    }

    /// Moves one entry by a step, saturating at the ends of the range.
    ///
    /// Returns the value after the move, or `None` when a number names no
    /// faction or the pair is one faction.
    pub fn shift(&mut self, tick: Tick, from: FactionId, to: FactionId, step: i32) -> Option<i32> {
        let before = self.get(from, to)?;
        self.write(tick, from, to, sim_math::offset(before, step))
    }

    /// A contract between two factions delivered in full. Both directions
    /// move up by the delivered step.
    pub fn on_contract_delivered(&mut self, tick: Tick, a: FactionId, b: FactionId) {
        let step = self.rules.contract_delivered;
        self.shift(tick, a, b, step);
        self.shift(tick, b, a, step);
    }

    /// A contract failed at its deadline. The party that was owed moves down
    /// toward the defaulter by the failed step.
    pub fn on_contract_failed(&mut self, tick: Tick, owed: FactionId, defaulter: FactionId) {
        let step = self.rules.contract_failed;
        self.shift(tick, owed, defaulter, -step);
    }

    /// Units of one faction fell to another. The victim moves down toward the
    /// killer by the fallen step for each unit.
    pub fn on_units_fell(&mut self, tick: Tick, victim: FactionId, killer: FactionId, count: u32) {
        let step = sim_math::offset_by_count(self.rules.unit_fell, count);
        self.shift(tick, victim, killer, -step);
    }

    /// Units of one faction converted to another. The old faction moves down
    /// toward the leader by the converted step for each unit.
    pub fn on_units_converted(
        &mut self,
        tick: Tick,
        from: FactionId,
        leader: FactionId,
        count: u32,
    ) {
        let step = sim_math::offset_by_count(self.rules.unit_converted, count);
        self.shift(tick, from, leader, -step);
    }

    /// Moves every entry one drift step toward the peace band, on the ticks
    /// the drift schedule names.
    ///
    /// An entry below the peace edge moves up and stops at the edge. An entry
    /// at or above the alliance edge moves down and stops one below it, which
    /// is the top of the peace band. An entry inside the peace band stays.
    /// The walk is in pair order, so it names no thread.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn drift(&mut self, tick: Tick) {
        let Some(schedule) = self.rules.drift_schedule() else {
            return;
        };
        if !schedule.due(tick) {
            return;
        }
        let rules = self.rules;
        let step = rules.drift;
        for from in 0..self.factions {
            for to in 0..self.factions {
                if from == to {
                    continue;
                }
                let (from, to) = (FactionId(from), FactionId(to));
                let Some(value) = self.get(from, to) else {
                    continue;
                };
                let next = if value < rules.peace_edge {
                    sim_math::offset(value, step).min(rules.peace_edge)
                } else if value >= rules.alliance_edge {
                    sim_math::offset(value, -step).max(sim_math::offset(rules.alliance_edge, -1))
                } else {
                    continue;
                };
                self.write(tick, from, to, next);
            }
        }
    }

    /// Returns the crossings of the last step.
    #[must_use]
    pub fn log(&self) -> &[RelationCrossed] {
        &self.log
    }

    /// Returns the crossings of the last step as bytes.
    #[must_use]
    pub fn log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.log)
    }

    /// Returns how many crossings of the last step went into the war band.
    #[must_use]
    pub fn declarations(&self) -> i64 {
        self.log
            .iter()
            .filter(|event| event.is_declaration())
            .count() as i64
    }

    /// Empties the log of the last step.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    /// Folds every value that a later frame reads into the hash.
    ///
    /// The entries and the rules enter. The log does not, because it is a
    /// log of one tick and the next step empties it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(u64::from(self.factions))
            .write(bytemuck::cast_slice(&self.entries))
            .write(bytemuck::bytes_of(&self.rules))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_band_number_counts_the_edges_at_or_below_the_value() {
        let rules = RelationRules::DEFAULT;
        assert_eq!(rules.band_of(rules.war_edge - 1), 0);
        assert_eq!(rules.band_of(rules.war_edge), 1);
        assert_eq!(rules.band_of(rules.peace_edge - 1), 1);
        assert_eq!(rules.band_of(rules.peace_edge), 2);
        assert_eq!(rules.band_of(rules.alliance_edge - 1), 2);
        assert_eq!(rules.band_of(rules.alliance_edge), 3);
        assert!(rules.band_of(i32::MAX) < BAND_COUNT);
    }

    #[test]
    fn a_shift_saturates_at_the_ends_of_the_range() {
        let mut matrix = RelationMatrix::new(2);
        let (a, b) = (FactionId(0), FactionId(1));
        matrix.write(Tick(1), a, b, i32::MAX - 1);
        assert_eq!(matrix.shift(Tick(1), a, b, 5), Some(i32::MAX));
        matrix.write(Tick(1), a, b, i32::MIN + 1);
        assert_eq!(matrix.shift(Tick(1), a, b, -5), Some(i32::MIN));
    }

    #[test]
    fn the_rules_hold_no_padding_byte() {
        assert_eq!(core::mem::size_of::<RelationRules>(), 14 * 4);
        assert_eq!(core::mem::size_of::<RelationCrossed>(), 16);
    }
}
