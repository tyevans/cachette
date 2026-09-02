//! The choice pass.
//!
//! A unit scores a small fixed set of options and takes the highest. Each
//! score is one multiplication: how much the unit wants a thing, multiplied
//! by how much of that thing is near.[^1]
//!
//! How much is near comes from the level 1 cell that the unit stands in, so
//! the unit reads a bounded neighbourhood and never searches the world. The
//! pass reads level 1 and writes nothing to it.[^2]
//!
//! Every value here is an integer or a Q16.16 value, and every operation
//! goes through the arithmetic module.[^3] A score is transient: the pass
//! compares it and discards it, so no score enters simulated state and no
//! score reaches the state hash.
//!
//! The options are scanned in ascending option index with a strict
//! comparison, so the lowest option index wins a tie. The option indices are
//! distinct, so the order is total and the tie needs no draw.[^4]
//!
//! The pass calls no content code. A content-supplied weight is a value in a
//! table, never a function.[^5]
//!
//! # References
//!
//! [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
//! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^4]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`

use crate::cohort::NEED_FULL;
use crate::pyramid::CellSummary;
use crate::sim_math;
use crate::types::Fix32;

/// The number of options that a unit scores.
///
/// The set is fixed at compile time. A unit scores every option in it and
/// takes the highest, so the cost of the pass is the option count times the
/// population and nothing else.[^1]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
pub const OPTION_COUNT: usize = 4;

/// The intent value that means a unit holds what it was doing.
///
/// The value sits at the top of the byte range, which no option index
/// reaches, because the option count is far below it. It is a property of
/// the column layout and not a budget.
pub const NO_INTENT: u8 = u8::MAX;

/// The value that a unit reads from its level 1 cell.
///
/// The engine owns this list. A content author supplies a weight against
/// each option, never a function that computes one.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellField {
    /// The share of the tiles of the cell that admit a unit.
    OpenShare,
    /// The mean height of the tiles of the cell.
    MeanHeight,
    /// The mean stub value of the tiles of the cell.
    ///
    /// **No option row reads this.** The tile value is a random walk that no
    /// other system reads or writes, and the `forage` row read it until the
    /// summary carried a resource.[^1] The viewer still paints the tile value
    /// at level 0, and the item that repairs the viewer holds the question of
    /// whether the pass that computes it should stay at all.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-181. `docs/FINDINGS.md`
    /// [^2]: Backlog item 0188, show the food of a tile and the reason a unit chose. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
    MeanValue,
    /// The number of units for each open tile of the cell.
    UnitsForEachOpenTile,
    /// The food that each tile of the cell still holds.
    ///
    /// The ground generates the stock, the gather resolve takes from it, and
    /// the recovery pass gives part of it back. A unit that scores this reads
    /// a quantity that another system writes.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decisions D1 and D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    MeanFood,
}

/// What the unit brings to an option.
///
/// A unit carries one need, and the need is what a consumption pass fills
/// and a rate empties.[^1] An option either answers the need or does not.
/// An option that answers it is driven by what the unit lacks. An option
/// that does not is driven by what the unit has, because a starving unit
/// has no appetite for anything else.
///
/// # References
///
/// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drive {
    /// The need that the unit still holds.
    Met,
    /// The need that the unit has lost.
    Unmet,
}

/// One option of the fixed set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionRow {
    /// The name that an explanation reports.
    pub name: &'static str,
    /// What the unit brings to the option.
    pub drive: Drive,
    /// What the option reads from the level 1 cell.
    pub field: CellField,
}

/// The fixed option set, in option index order.
///
/// The order is the tie-break order. A change to it changes which option
/// wins a tie, so the order is part of the behaviour and not a listing.[^1]
///
/// A change to the field that one row reads is not a change to the order.
/// The `forage` row keeps its index, so the tie-break is what it was.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub const OPTIONS: [OptionRow; OPTION_COUNT] = [
    OptionRow {
        name: "roam",
        drive: Drive::Met,
        field: CellField::OpenShare,
    },
    OptionRow {
        name: "forage",
        drive: Drive::Unmet,
        field: CellField::MeanFood,
    },
    OptionRow {
        name: "climb",
        drive: Drive::Met,
        field: CellField::MeanHeight,
    },
    OptionRow {
        name: "join",
        drive: Drive::Met,
        field: CellField::UnitsForEachOpenTile,
    },
];

/// The score that an option must reach before a unit acts on it.
///
/// **The floor is a frame-budget parameter. It is not a design knob.**
/// Without it, a world where every option scores near zero lets the
/// tie-break decide. Every unit then takes option zero, the whole
/// population walks one way, and every unit becomes a mover. The movement
/// subsystem is sized for a fraction of the population, so that failure
/// multiplies what movement costs.[^1] [^2]
///
/// A change to this value changes the mover count, and therefore the frame
/// budget. The reference table holds the reasoning and the derivation.[^3]
///
/// # References
///
/// [^1]: Findings register, FND-014. `docs/FINDINGS.md`
/// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^3]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
pub const SCORE_FLOOR: Fix32 = Fix32(1 << 14);

/// The largest interval that a schedule may state.
///
/// The interval is a power of two, so the phase test is a mask and not a
/// division. The limit is the width of the phase, which is a property of
/// the arithmetic and not a budget.
pub const PERIOD_LOG2_CEILING: u32 = 16;

/// The reason that the choice pass refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceError {
    /// The interval is above the ceiling that the phase width allows.
    PeriodAboveCeiling(u32),
    /// The option index is at or above the option count.
    OptionOutsideSet(u8),
}

impl core::fmt::Display for ChoiceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PeriodAboveCeiling(period) => write!(
                formatter,
                "the interval exponent {period} is above the ceiling {PERIOD_LOG2_CEILING}"
            ),
            Self::OptionOutsideSet(option) => write!(
                formatter,
                "the option {option} is at or above the option count {OPTION_COUNT}"
            ),
        }
    }
}

impl std::error::Error for ChoiceError {}

/// When a unit re-reads the world.
///
/// The choice does not run on every tick. It runs at an interval, and the
/// interval is staggered so that the whole population does not choose at
/// once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChoiceSchedule {
    period_log2: u32,
}

impl ChoiceSchedule {
    /// The interval that a world starts with.
    ///
    /// The value is a parameter of the world. The reference table states
    /// what it costs and why the project recommends it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    pub const DEFAULT: Self = Self { period_log2: 5 };

    /// Builds a schedule from the exponent of the interval.
    ///
    /// An exponent of zero makes every unit choose on every tick.
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is above the ceiling.
    pub const fn new(period_log2: u32) -> Result<Self, ChoiceError> {
        if period_log2 > PERIOD_LOG2_CEILING {
            return Err(ChoiceError::PeriodAboveCeiling(period_log2));
        }
        Ok(Self { period_log2 })
    }

    /// Returns the exponent of the interval.
    #[must_use]
    pub const fn period_log2(self) -> u32 {
        self.period_log2
    }

    /// Returns the number of ticks between two choices.
    #[must_use]
    pub const fn period(self) -> u64 {
        1u64 << self.period_log2
    }

    /// Reports whether a unit in one cell chooses on one frame.
    ///
    /// **The key is the level 1 cell, not the identity of the unit.** A cell
    /// key gives one frame to a whole cell: every unit that shares a cell
    /// chooses on the same frame, whatever order the array holds. An identity
    /// key gives one frame to a scattered set instead, and reads a whole cache
    /// line to use a few bytes of it.[^1] [^2]
    ///
    /// The contiguity this buys depends on the order of the array, and the
    /// arena is not ordered by tile. It is a slot array in spawn order that
    /// reuses a freed slot, and it never compacts, because compaction would
    /// invalidate every identity that names a slot. A cell key gives long runs
    /// only where spawn order happens to follow tile order.
    ///
    /// The phase mixes the cell index. A bare mask of the cell index would
    /// choose a regular spatial stripe of the world on each tick, which ties
    /// the decision phase to the geography.
    ///
    /// The test is a pure function of the cell and the frame. It reads no
    /// counter and no accumulator, so it gives one answer at any thread
    /// count.[^3]
    ///
    /// A unit that crosses a cell boundary may choose twice inside one
    /// interval, or skip one interval. That is accepted behaviour. A unit
    /// that arrives in a new region should re-read it, and a skipped
    /// interval delays a choice rather than losing it.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-023. `docs/FINDINGS.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub const fn chooses_now(self, cell: u32, frame: u64) -> bool {
        let mask = self.period() - 1;
        stagger_phase(cell, self.period_log2) as u64 == (frame & mask)
    }
}

/// Returns the phase of one level 1 cell.
///
/// The mix is a fixed multiply and one exclusive-or shift. It is exact
/// integer arithmetic on an index, so it holds the same value on every
/// machine.
#[must_use]
pub const fn stagger_phase(cell: u32, period_log2: u32) -> u32 {
    let mut state = cell.wrapping_mul(0x9e37_79b9);
    state ^= state >> 16;
    state & ((1u32 << period_log2) - 1)
}

/// The weight that a unit puts on each option.
///
/// The profile is content. It is a table of values, and the engine reads it
/// rather than calling into it.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeightProfile {
    weights: [Fix32; OPTION_COUNT],
}

impl WeightProfile {
    /// The profile that puts one unit of weight on every option.
    pub const EVEN: Self = Self {
        weights: [Fix32::ONE; OPTION_COUNT],
    };

    /// The profile that puts no weight on any option.
    ///
    /// A unit with this profile wants nothing, so every score is zero, every
    /// score is below the floor, and the unit holds.
    pub const ZEROED: Self = Self {
        weights: [Fix32::ZERO; OPTION_COUNT],
    };

    /// Returns the weight of one option, or `None` when the index is outside
    /// the set.
    #[must_use]
    pub const fn weight(&self, option: u8) -> Option<Fix32> {
        if option as usize >= OPTION_COUNT {
            return None;
        }
        Some(self.weights[option as usize])
    }

    /// Writes the weight of one option.
    ///
    /// # Errors
    ///
    /// Returns an error when the index is outside the set.
    pub const fn set(&mut self, option: u8, weight: Fix32) -> Result<(), ChoiceError> {
        if option as usize >= OPTION_COUNT {
            return Err(ChoiceError::OptionOutsideSet(option));
        }
        self.weights[option as usize] = weight;
        Ok(())
    }
}

impl Default for WeightProfile {
    fn default() -> Self {
        Self::EVEN
    }
}

/// Returns the value that one option reads from one cell.
///
/// A cell that covers nothing gives zero. A fold over an empty set is the
/// identity, and the identity of every field here is zero.[^1]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[must_use]
pub fn field_value(summary: CellSummary, field: CellField) -> Fix32 {
    match field {
        CellField::OpenShare => summary.open_share(),
        CellField::MeanHeight => summary.mean_height(),
        CellField::MeanValue => summary.mean_value(),
        CellField::UnitsForEachOpenTile => summary.units_for_each_open_tile(),
        CellField::MeanFood => summary.mean_food(),
    }
    .unwrap_or(Fix32::ZERO)
}

/// Returns what a unit brings to one option.
#[must_use]
pub fn drive_value(need: Fix32, drive: Drive) -> Fix32 {
    match drive {
        Drive::Met => need,
        Drive::Unmet => sim_math::sub(NEED_FULL, need),
    }
}

/// Returns the score of one option.
///
/// The score is one multiplication of the want by what is near, and the
/// want is itself the drive scaled by the weight. Every operation goes
/// through the arithmetic module and saturates, so the result is exact and
/// total.[^1]
///
/// The score is transient. Nothing stores it, so it enters no state hash.
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub fn score(need: Fix32, weight: Fix32, summary: CellSummary, option: OptionRow) -> Fix32 {
    let want = sim_math::mul(drive_value(need, option.drive), weight);
    sim_math::mul(want, field_value(summary, option.field))
}

/// Returns the order in which the choice scans the options.
///
/// The order is ascending option index. The comparison is strict, so the
/// lowest option index wins a tie, and the option indices are distinct, so
/// the order is total.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[cfg(not(feature = "probe-nondeterminism"))]
#[must_use]
pub const fn option_order() -> [u8; OPTION_COUNT] {
    [0, 1, 2, 3]
}

/// Returns the options in descending index order, which is a defect.
///
/// This is the perturbed build. The scan reads the options from the top of
/// the set, so the strict comparison now gives a tie to the **highest**
/// option index. The choice is still deterministic and still gives one
/// answer at any thread count, so neither determinism test can see it. Only
/// a test that constructs a tie and names the winner can.[^1]
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^2]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
/// [^2]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
#[must_use]
pub const fn option_order() -> [u8; OPTION_COUNT] {
    [3, 2, 1, 0]
}

/// The scores of one unit, and what it chose from them.
///
/// A watcher asks why a unit chose what it chose, and this is the answer.
/// The engine recomputes it on demand from the world as it stands. It
/// stores no score, so the explanation costs nothing when nobody asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChoiceExplanation {
    /// The level 1 cell that the unit read.
    pub cell: u32,
    /// What the unit still needs.
    pub need: Fix32,
    /// The score of each option, in option index order.
    pub scores: [Fix32; OPTION_COUNT],
    /// The value each option read from the cell, in option index order.
    pub fields: [Fix32; OPTION_COUNT],
    /// The weight each option carried, in option index order.
    pub weights: [Fix32; OPTION_COUNT],
    /// The score that an option had to reach.
    pub floor: Fix32,
    /// The option that the scores select, or `NO_INTENT` for a hold.
    pub best: u8,
    /// The intent that the unit carries now.
    pub intent: u8,
    /// Whether the unit re-reads the world on the next frame.
    pub chooses_next_frame: bool,
}

impl ChoiceExplanation {
    /// Returns the name of the option that the scores select.
    ///
    /// Returns `None` when every score is below the floor.
    #[must_use]
    pub fn best_name(&self) -> Option<&'static str> {
        if self.best == NO_INTENT {
            return None;
        }
        Some(OPTIONS[self.best as usize].name)
    }
}

/// Returns the option that a unit takes, or `NO_INTENT` for a hold.
///
/// The scan starts at the floor and uses a strict comparison, so an option
/// that only equals the running best never displaces it and an option that
/// only equals the floor never wins. The lowest option index therefore wins
/// a tie, and a unit whose every option is below the floor holds what it was
/// doing.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: Findings register, FND-014. `docs/FINDINGS.md`
#[must_use]
pub fn best_option(need: Fix32, summary: CellSummary, profile: &WeightProfile) -> u8 {
    let mut best = NO_INTENT;
    let mut best_score = SCORE_FLOOR;
    for option in option_order() {
        let weight = profile.weights[option as usize];
        let value = score(need, weight, summary, OPTIONS[option as usize]);
        if value > best_score {
            best_score = value;
            best = option;
        }
    }
    best
}

/// Returns the scores of one unit and the option they select.
#[must_use]
pub fn explain(
    cell: u32,
    need: Fix32,
    summary: CellSummary,
    profile: &WeightProfile,
    intent: u8,
    chooses_next_frame: bool,
) -> ChoiceExplanation {
    let mut scores = [Fix32::ZERO; OPTION_COUNT];
    let mut fields = [Fix32::ZERO; OPTION_COUNT];
    for (index, option) in OPTIONS.iter().enumerate() {
        fields[index] = field_value(summary, option.field);
        scores[index] = score(need, profile.weights[index], summary, *option);
    }
    ChoiceExplanation {
        cell,
        need,
        scores,
        fields,
        weights: profile.weights,
        floor: SCORE_FLOOR,
        best: best_option(need, summary, profile),
        intent,
        chooses_next_frame,
    }
}
