//! A decayed memory of recent events, per faction, and who caused each one.
//!
//! An observation of one frame is a snapshot. A snapshot cannot tell a
//! faction that holds two hundred tiles and is gaining from one that holds two
//! hundred and is losing. It cannot say that a rival is taking a city now, and
//! it cannot say that one rival keeps killing the people of the reader. This
//! module holds the state that answers those three questions.
//!
//! # Why the engine holds it and the control plane cannot
//!
//! **Every event log of this engine holds one step and no more.** The step
//! empties each log before any system runs, so a reader that samples the logs
//! at its own pace sees the steps it sampled and misses the rest. A learner
//! that decides once in ten ticks would read one step in ten and would
//! under-report by nine parts in ten, while every value it read stayed
//! plausible. The accumulator therefore advances inside the step, once for
//! each step, and no caller can skip a step.
//!
//! # The decay
//!
//! One counter holds one kind of event for one faction. The advance moves it
//! by one recursion:
//!
//! ```text
//! next = current - (current >> bits) + arrivals
//! ```
//!
//! The recursion needs one shift and two additions. It holds no division, no
//! multiplication and no floating point number, so it obeys the rule that
//! aggregated state carries no floating point number.[^1]
//!
//! **The fixed point of the recursion is the arrival rate shifted left by the
//! same bits.** Take a constant arrival of `a` on every step. The counter
//! grows while `current >> bits` is below `a`, and it stops at the first value
//! whose shift equals `a`, which is `a << bits` exactly. The counter therefore
//! measures the arrival rate of the recent past at a fixed scale, and the
//! bound of the counter is the largest arrival one step can hold, shifted left
//! by the same bits. That largest arrival is a world parameter: the reserved
//! unit count, the tile count, the site slot count or the seated faction
//! count, by the kind. No counter bound follows the population of a live
//! world.[^2]
//!
//! The half life of the recursion is about `bits` times the natural logarithm
//! of two steps, so the shift is the one dial that sets how long the memory
//! lasts.
//!
//! # Two lengths, because a spike is not a trend
//!
//! The module keeps two counters for each kind: a recent one at a small shift
//! and a lasting one at a large shift. Both reach the same steady value for
//! the same constant arrival rate, once each is divided by its own shift. A
//! reader that compares the two therefore reads zero while the rate holds, a
//! positive value while the rate rises, and a negative value while it falls.
//! One rival that takes a city this minute is a spike. One rival that keeps
//! killing the people of the reader is a trend. The pair separates them, and a
//! single counter cannot.
//!
//! # Attribution without a seat number
//!
//! A faction needs to know which rival keeps acting against it. This module
//! therefore holds a second set of counters, keyed on the ordered pair of the
//! subject and the other faction, for the kinds that name an aggressor. The
//! iteration over that set is over ascending seat numbers, so the order is
//! stated and nothing reads a hash order.[^3]
//!
//! **No slot of the observation is indexed by a seat number.** A league seats
//! one policy in one seat for one game and in another seat for the next, so a
//! policy that learned a seat number reads another faction's quantities under
//! the same weight.[^4] The reader of this state therefore publishes order
//! statistics over the rivals, which carry no identity at all, and a separate
//! block of rival tokens carries the joint detail in an order that a
//! permutation-invariant encoder consumes.[^5]
//!
//! # What the state costs
//!
//! The state is one row for each faction, and one row for each ordered pair of
//! factions. **Nothing here follows the population and nothing follows the
//! world size.** The pair term follows the square of the seated faction count,
//! which the faction relation matrix already follows.[^6]
//!
//! # Determinism
//!
//! One advance runs on the calling thread, at the end of the step. It reads
//! each log in the order the log holds, and it visits the factions in
//! ascending seat order. Nothing reads a thread completion order and nothing
//! reads a hash order.[^3]
//!
//! # References
//!
//! [^1]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
//! [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^4]: Findings register, FND-647. `docs/FINDINGS.md`
//! [^5]: Research report 42, what a policy should be able to see, sections 6.3 and 6.4. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
//! [^6]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`

use crate::sim_math;
use crate::types::FactionId;

/// How long one counter remembers.
///
/// The two lengths exist so that a reader can tell a spike from a trend. A
/// single length cannot: one arrival burst and a steady rate that reaches the
/// same counter value are the same number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Decay {
    /// The short memory. It follows the last few steps.
    Recent,
    /// The long memory. It follows the trend over many steps.
    Lasting,
}

impl Decay {
    /// Both lengths, in the order the observation holds them.
    pub const ALL: [Self; 2] = [Self::Recent, Self::Lasting];

    /// Returns the shift that sets how long the memory lasts.
    ///
    /// The shift is a structural choice of this module and not a balance
    /// value. It states the scale at which a counter measures an arrival
    /// rate, and every published value divides the counter by the same shift,
    /// so the published value does not depend on it.
    #[must_use]
    pub const fn bits(self) -> u32 {
        match self {
            Self::Recent => 4,
            Self::Lasting => 8,
        }
    }

    /// Returns the position this length holds inside one counter pair.
    #[must_use]
    pub const fn position(self) -> usize {
        match self {
            Self::Recent => 0,
            Self::Lasting => 1,
        }
    }

    /// Returns the name of the length.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Lasting => "lasting",
        }
    }
}

/// The quantity that a published share of one kind divides by.
///
/// **A raw count is not comparable between two worlds.** Twenty units lost is
/// a rout on a small world and a skirmish on a large one. Each kind therefore
/// names the stock of the reader that the loss or the gain came out of, and
/// the reader publishes the count as a share of that stock. Two worlds of
/// different size and the same event rate then publish the same value.[^1]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stock {
    /// The units of the faction that are alive.
    LiveUnits,
    /// The tiles the faction holds.
    HeldTiles,
    /// The sites the faction holds.
    Sites,
    /// The factions seated beside the reader.
    Rivals,
    /// The load every live unit of the faction could carry at once.
    CarryRoom,
}

/// One kind of event that this module remembers.
///
/// **The list answers what a decision depends on, and it is not the list of
/// every log the engine holds.** A kind is here when a policy would act
/// differently on reading it: it must lose or gain the reader something, and
/// the reader must be able to change it. A log that only reports a fact of the
/// world, such as a fire starting or a trade being spoken, has no entry,
/// because the loss it causes is already here under the kind that names the
/// loss.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryKind {
    /// Units of the faction that fell in a meeting with another faction.
    ///
    /// This is the cost of being fought. It names the faction that delivered
    /// the most harm to each group that lost units.
    OwnUnitsFelled,
    /// Units of another faction that this faction felled in a meeting.
    ///
    /// This is the only cumulative count of aggression the engine keeps, so
    /// it is the one signal a trainer can reward an aggressive policy on.
    RivalUnitsFelled,
    /// Units of the faction that a shortage ended.
    ///
    /// A famine and a defeat both remove units. A policy that could not tell
    /// them apart would answer a famine by building soldiers.
    OwnUnitsStarved,
    /// Units of the faction that a fire ended.
    ///
    /// A fire has no aggressor, so a policy that read this under the felled
    /// kind would learn to fear ground that nobody threatens.
    OwnUnitsBurned,
    /// Tiles that the faction stopped holding.
    ///
    /// Held ground follows the reach of a city, so ground moves without any
    /// event naming a mover. The delta of the held count is the only reader
    /// of it.
    OwnGroundLost,
    /// Tiles that the faction started holding.
    GroundGained,
    /// Sites that the faction lost, whether the taker kept one or destroyed
    /// it.
    ///
    /// A city is the unit of the game that a policy cannot replace quickly,
    /// so losing one is the loss a policy must see soonest.
    OwnSitesLost,
    /// Sites that the faction took, whether it kept one or destroyed it.
    SitesTaken,
    /// Upgrades that went from ground the faction held.
    ///
    /// Reach grows with finished upgrades, so an upgrade lost is reach lost.
    OwnUpgradesLost,
    /// Crossings where another faction's feeling toward this one fell a band.
    ///
    /// This is the signal that a war is coming, and it arrives before the
    /// first unit falls.
    RelationsFellAgainstMe,
    /// Resource amounts that the units of the faction took from tiles.
    ResourceGathered,
}

impl MemoryKind {
    /// Every kind, in the order the observation holds them.
    pub const ALL: [Self; 11] = [
        Self::OwnUnitsFelled,
        Self::RivalUnitsFelled,
        Self::OwnUnitsStarved,
        Self::OwnUnitsBurned,
        Self::OwnGroundLost,
        Self::GroundGained,
        Self::OwnSitesLost,
        Self::SitesTaken,
        Self::OwnUpgradesLost,
        Self::RelationsFellAgainstMe,
        Self::ResourceGathered,
    ];

    /// Returns the name of the kind.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OwnUnitsFelled => "own_units_felled",
            Self::RivalUnitsFelled => "rival_units_felled",
            Self::OwnUnitsStarved => "own_units_starved",
            Self::OwnUnitsBurned => "own_units_burned",
            Self::OwnGroundLost => "own_ground_lost",
            Self::GroundGained => "ground_gained",
            Self::OwnSitesLost => "own_sites_lost",
            Self::SitesTaken => "sites_taken",
            Self::OwnUpgradesLost => "own_upgrades_lost",
            Self::RelationsFellAgainstMe => "relations_fell_against_me",
            Self::ResourceGathered => "resource_gathered",
        }
    }

    /// Returns the position the kind holds in one row.
    #[must_use]
    pub const fn position(self) -> usize {
        match self {
            Self::OwnUnitsFelled => 0,
            Self::RivalUnitsFelled => 1,
            Self::OwnUnitsStarved => 2,
            Self::OwnUnitsBurned => 3,
            Self::OwnGroundLost => 4,
            Self::GroundGained => 5,
            Self::OwnSitesLost => 6,
            Self::SitesTaken => 7,
            Self::OwnUpgradesLost => 8,
            Self::RelationsFellAgainstMe => 9,
            Self::ResourceGathered => 10,
        }
    }

    /// Returns the stock of the reader that a published share divides by.
    #[must_use]
    pub const fn stock(self) -> Stock {
        match self {
            Self::OwnUnitsFelled
            | Self::RivalUnitsFelled
            | Self::OwnUnitsStarved
            | Self::OwnUnitsBurned => Stock::LiveUnits,
            Self::OwnGroundLost | Self::GroundGained | Self::OwnUpgradesLost => Stock::HeldTiles,
            Self::OwnSitesLost | Self::SitesTaken => Stock::Sites,
            Self::RelationsFellAgainstMe => Stock::Rivals,
            Self::ResourceGathered => Stock::CarryRoom,
        }
    }

    /// Returns the position the kind holds in one pair row, or `None` when no
    /// faction causes the kind.
    ///
    /// **This match is the only declaration of which kinds carry an
    /// aggressor.** The count below counts the kinds this match answers for,
    /// so a kind added here reaches the pair row without a second edit.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn blame_position(self) -> Option<usize> {
        match self {
            Self::OwnUnitsFelled => Some(0),
            Self::RivalUnitsFelled => Some(1),
            Self::OwnSitesLost => Some(2),
            Self::SitesTaken => Some(3),
            Self::RelationsFellAgainstMe => Some(4),
            Self::OwnUnitsStarved
            | Self::OwnUnitsBurned
            | Self::OwnGroundLost
            | Self::GroundGained
            | Self::OwnUpgradesLost
            | Self::ResourceGathered => None,
        }
    }
}

/// How many kinds one row of counters holds.
pub const KIND_COUNT: usize = MemoryKind::ALL.len();

/// How many kinds one pair row of counters holds.
///
/// The count is derived from the blame position of every kind, so it cannot
/// disagree with that match.
pub const BLAMED_KIND_COUNT: usize = {
    let mut found = 0;
    let mut at = 0;
    while at < KIND_COUNT {
        if MemoryKind::ALL[at].blame_position().is_some() {
            found += 1;
        }
        at += 1;
    }
    found
};

/// How many lengths one counter set holds.
pub const DECAY_COUNT: usize = Decay::ALL.len();

/// The decayed event history of every faction of one world.
///
/// The type holds the counters, the arrivals of the step in progress, and the
/// held tile count that the last advance read. It holds no world and no
/// reference, so the world owns one and hands it the arrivals of each step.
///
/// The default value holds no faction at all. It exists so that the advance
/// can take the history out of the world, fold the logs beside it, and put it
/// back. Every reader of a default value answers zero, because no number names
/// a faction of it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventMemory {
    factions: usize,
    totals: Vec<i64>,
    blamed: Vec<i64>,
    arrivals: Vec<i64>,
    blamed_arrivals: Vec<i64>,
    previous_held: Vec<i64>,
}

impl EventMemory {
    /// Builds an empty history for a world of the given seated faction count.
    ///
    /// A faction count of zero still allocates one row, because every reader
    /// of this type names a faction and a world holds at least one seat.
    #[must_use]
    pub fn new(factions: u16) -> Self {
        let factions = usize::from(factions.max(1));
        Self {
            factions,
            totals: vec![0; factions * KIND_COUNT * DECAY_COUNT],
            blamed: vec![0; factions * factions * BLAMED_KIND_COUNT * DECAY_COUNT],
            arrivals: vec![0; factions * KIND_COUNT],
            blamed_arrivals: vec![0; factions * factions * BLAMED_KIND_COUNT],
            previous_held: vec![0; factions],
        }
    }

    /// Returns how many factions the history holds a row for.
    ///
    /// **The seated faction count is declared in the world settings and again
    /// here, so the world invariant check compares the two.** A history built
    /// for the wrong count would answer zero for a faction that exists, and
    /// nothing else would fail.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn factions(&self) -> usize {
        self.factions
    }

    /// Adds arrivals of one kind to one faction, for the step in progress.
    ///
    /// The caller is a pass of the step, or the advance itself where the
    /// arrival comes from a log. A negative amount is refused, because a
    /// counter of arrivals never falls.
    pub fn record(&mut self, faction: FactionId, kind: MemoryKind, amount: i64) {
        if amount <= 0 {
            return;
        }
        let Some(place) = self.arrival_at(faction, kind) else {
            return;
        };
        self.arrivals[place] = self.arrivals[place].saturating_add(amount);
    }

    /// Adds arrivals of one kind to one faction, and names the faction that
    /// caused them.
    ///
    /// **The call writes both rows, so the pair row and the total row cannot
    /// disagree.** A caller that wrote one and forgot the other would leave a
    /// share of the damage that no rival accounts for, and nothing would
    /// fail.[^1]
    ///
    /// A kind that names no aggressor writes the total row alone. A subject
    /// that names itself as the other faction writes the total row alone,
    /// because a faction is not its own rival.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub fn record_blamed(
        &mut self,
        faction: FactionId,
        other: FactionId,
        kind: MemoryKind,
        amount: i64,
    ) {
        self.record(faction, kind, amount);
        if amount <= 0 || faction == other {
            return;
        }
        let Some(place) = self.blamed_arrival_at(faction, other, kind) else {
            return;
        };
        self.blamed_arrivals[place] = self.blamed_arrivals[place].saturating_add(amount);
    }

    /// Records the ground one faction gained and lost since the last advance.
    ///
    /// **Held ground carries no event.** A tile changes hands because the
    /// reach of a city moved, and the pass that rewrites the holder column
    /// writes one column and no log. The delta of the held count is therefore
    /// the only reader of it, and this call is where the previous count lives.
    pub fn note_held(&mut self, faction: FactionId, held: i64) {
        let before = {
            let Some(previous) = self.previous_held.get_mut(usize::from(faction.0)) else {
                return;
            };
            let before = *previous;
            *previous = held;
            before
        };
        self.record(faction, MemoryKind::OwnGroundLost, before - held);
        self.record(faction, MemoryKind::GroundGained, held - before);
    }

    /// Moves every counter one step, then empties the arrivals.
    ///
    /// **This runs once for each step and it never runs twice.** A counter
    /// that advanced on a decision step alone would measure one step in as
    /// many as the caller skipped.
    pub fn advance(&mut self) {
        for (counters, arrivals) in self
            .totals
            .chunks_exact_mut(DECAY_COUNT)
            .zip(self.arrivals.iter())
        {
            for length in Decay::ALL {
                let place = &mut counters[length.position()];
                *place = sim_math::decay(*place, *arrivals, length.bits());
            }
        }
        for (counters, arrivals) in self
            .blamed
            .chunks_exact_mut(DECAY_COUNT)
            .zip(self.blamed_arrivals.iter())
        {
            for length in Decay::ALL {
                let place = &mut counters[length.position()];
                *place = sim_math::decay(*place, *arrivals, length.bits());
            }
        }
        self.arrivals.fill(0);
        self.blamed_arrivals.fill(0);
    }

    /// Returns the counter of one kind of one faction at one length.
    ///
    /// Returns zero when the number names no faction of this history.
    #[must_use]
    pub fn total(&self, faction: FactionId, kind: MemoryKind, length: Decay) -> i64 {
        self.arrival_at(faction, kind).map_or(0, |place| {
            self.totals[place * DECAY_COUNT + length.position()]
        })
    }

    /// Returns the counter of one kind that one other faction caused, at one
    /// length.
    ///
    /// Returns zero when the kind names no aggressor, when either number
    /// names no faction, and when the two numbers name one faction.
    #[must_use]
    pub fn blamed_total(
        &self,
        faction: FactionId,
        other: FactionId,
        kind: MemoryKind,
        length: Decay,
    ) -> i64 {
        self.blamed_arrival_at(faction, other, kind)
            .map_or(0, |place| {
                self.blamed[place * DECAY_COUNT + length.position()]
            })
    }

    /// Returns the position of one arrival counter, or `None` when the number
    /// names no faction.
    fn arrival_at(&self, faction: FactionId, kind: MemoryKind) -> Option<usize> {
        let faction = usize::from(faction.0);
        if faction >= self.factions {
            return None;
        }
        Some(faction * KIND_COUNT + kind.position())
    }

    /// Returns the position of one pair arrival counter, or `None` when the
    /// kind names no aggressor and when either number names no faction.
    fn blamed_arrival_at(
        &self,
        faction: FactionId,
        other: FactionId,
        kind: MemoryKind,
    ) -> Option<usize> {
        let slot = kind.blame_position()?;
        let faction = usize::from(faction.0);
        let other = usize::from(other.0);
        if faction >= self.factions || other >= self.factions {
            return None;
        }
        Some((faction * self.factions + other) * BLAMED_KIND_COUNT + slot)
    }
}
