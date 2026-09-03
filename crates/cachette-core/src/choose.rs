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
use crate::resource::ResourceKind;
use crate::sim_math;
use crate::types::Fix32;

/// The number of options that a unit scores.
///
/// The set is fixed at compile time, and a unit takes the highest option in
/// it.[^1]
///
/// **The engine does not score the set once for each unit.** It scores it once
/// for each level 1 cell and each bucket of need, and a unit reads the
/// answer.[^2] [^3] The accepted record says in its own text that the cost of
/// the pass is the option count times the population and nothing else. That
/// sentence is a consequence the record derived rather than a decision it made,
/// and it is false now. The record that makes it false says so, and a register
/// holds how the project repairs a stale consequence inside an accepted
/// record.[^4]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
/// [^3]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
/// [^4]: Decisions register, DEC-096. `docs/DECISIONS.md`
pub const OPTION_COUNT: usize = 5;

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

/// What one option ranks.
///
/// An option that ranks a level 1 summary field is worth the same to every
/// unit of a cell. An option that ranks the state of the unit itself is not,
/// and the choice key holds that state so the answer is still shared.[^1]
///
/// The two arms also decide what steers the step. A cell field is ranked by
/// the exit field, which holds one direction for each cell and each
/// option.[^2] The carry class is ranked by the return field, which holds one
/// direction for each cell and each faction.[^3]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^3]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ranked {
    /// The option ranks one summary field of the level 1 cell.
    Cell(CellField),
    /// The option ranks how much the unit itself carries.
    Carry,
}

/// The number of carry classes.
///
/// The count is fixed at compile time, and it is what bounds the choice
/// table. A class that a content author could add would put the bound in the
/// content rather than in the engine, which is the shape the choice record
/// refuses.[^1]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D2. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
pub const CARRY_CLASS_COUNT: usize = 2;

/// What a unit carries, as a class rather than as a quantity.
///
/// **The class is not the load.** A load is a whole number for each resource
/// kind, so two units of one cell almost never hold one load, and an option
/// that read the load would give one answer for each unit. The class holds
/// two values, so the choice still computes one answer for a whole group of
/// units.[^1]
///
/// A unit is laden when it holds a home site and its load reaches the carry
/// mark of the world. **A unit with no home is never laden**, because the
/// delivery moves a load into the store of a home site and a unit without one
/// has nothing to deliver to.[^2]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decisions D1 and D2. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarryClass {
    /// The unit carries less than the mark, or it holds no home site.
    Free,
    /// The unit holds a home site and carries at least the mark.
    Laden,
}

impl CarryClass {
    /// Returns the index of the class.
    ///
    /// The index is the second half of the choice key. It is stated once,
    /// here, so no reader of the table declares the order a second time.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Free => 0,
            Self::Laden => 1,
        }
    }

    /// Returns the class of one index, or `None` when the index names none.
    #[must_use]
    pub const fn of_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Free),
            1 => Some(Self::Laden),
            _ => None,
        }
    }
}

/// One option of the fixed set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionRow {
    /// The name that an explanation reports.
    pub name: &'static str,
    /// What the unit brings to the option.
    pub drive: Drive,
    /// What the option ranks, and therefore what steers the step it takes.
    pub ranked: Ranked,
    /// The resource that a unit gathers while it holds this option.
    ///
    /// **This row is the one declaration of the map from an option to a
    /// resource kind.** A second site that named the kind of a gathering
    /// option would be one value in two places, with nothing to fail when the
    /// copies disagree.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: Findings register, FND-191. `docs/FINDINGS.md`
    pub gathers: Option<ResourceKind>,
}

/// The fixed option set, in option index order.
///
/// The order is the tie-break order. A change to it changes which option
/// wins a tie, so the order is part of the behaviour and not a listing.[^1]
///
/// A change to the field that one row reads is not a change to the order.
///
/// **The row that ranks the carry takes the lowest index, so it wins a
/// tie.**[^2] The four rows that rank the ground keep the order they had
/// between themselves, so every tie that they already decide keeps the winner
/// it has.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D5. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
pub const OPTIONS: [OptionRow; OPTION_COUNT] = [
    OptionRow {
        // **The row takes the lowest option index, so it wins a tie.** The
        // four rows below it rank the ground, and the open share of a cell
        // reaches one whole unit wherever a whole block admits a unit, which
        // ties the roam row with this one exactly. A tie decided by the ground
        // would leave a laden unit walking in a world with no water in
        // it.[^1] [^3]
        //
        // The four rows below keep the order they had between themselves, so
        // every tie that they already decide keeps the winner it has.
        //
        // The row names no resource kind, so the choice pass clears the
        // gather order of a unit that takes it. **A laden unit stops
        // gathering**, and that rule lives in the row rather than in a second
        // place that could disagree with it.[^2]
        //
        // The drive is what the unit still holds, so a starving unit forages
        // rather than walks home. A unit that is fed and full goes home.
        //
        // [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
        // [^2]: Findings register, FND-191. `docs/FINDINGS.md`
        // [^3]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D5. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
        name: "deliver",
        drive: Drive::Met,
        ranked: Ranked::Carry,
        gathers: None,
    },
    OptionRow {
        name: "roam",
        drive: Drive::Met,
        ranked: Ranked::Cell(CellField::OpenShare),
        gathers: None,
    },
    OptionRow {
        name: "forage",
        drive: Drive::Unmet,
        ranked: Ranked::Cell(CellField::MeanFood),
        // The option is driven by what the unit lacks, and what a unit lacks
        // is the ration that the consumption pass draws. Food is the only
        // kind that answers it. Wood and stone answer no need today, and a
        // world that holds three kinds does not make three options.[^1]
        //
        // [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        gathers: Some(ResourceKind::Food),
    },
    OptionRow {
        name: "climb",
        drive: Drive::Met,
        ranked: Ranked::Cell(CellField::MeanHeight),
        gathers: None,
    },
    OptionRow {
        name: "join",
        drive: Drive::Met,
        ranked: Ranked::Cell(CellField::UnitsForEachOpenTile),
        gathers: None,
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
    /// The bucket exponent is outside the range that the answer table holds.
    BucketShiftOutsideRange(u32),
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
            Self::BucketShiftOutsideRange(shift) => write!(
                formatter,
                "the bucket exponent {shift} is outside the range \
                 {NEED_BUCKET_SHIFT_FLOOR} to {NEED_BUCKET_SHIFT_CEILING}"
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

/// Returns the value that one option ranks.
///
/// An option that ranks a cell field reads the summary. An option that ranks
/// the carry reads the class, and it is worth one whole unit of value to a
/// laden unit and nothing to any other. **A unit that carries nothing can
/// therefore never take such an option**, because a score of zero is below
/// the floor.[^1]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
#[must_use]
pub fn ranked_value(summary: CellSummary, carry: CarryClass, ranked: Ranked) -> Fix32 {
    match ranked {
        Ranked::Cell(field) => field_value(summary, field),
        Ranked::Carry => match carry {
            CarryClass::Free => Fix32::ZERO,
            CarryClass::Laden => Fix32::ONE,
        },
    }
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
pub fn score(
    need: Fix32,
    carry: CarryClass,
    weight: Fix32,
    summary: CellSummary,
    option: OptionRow,
) -> Fix32 {
    let want = sim_math::mul(drive_value(need, option.drive), weight);
    sim_math::mul(want, ranked_value(summary, carry, option.ranked))
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
    [0, 1, 2, 3, 4]
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
    [4, 3, 2, 1, 0]
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
    /// The need that the pass scored, which is the bucket of the need above.
    ///
    /// The pass computes one answer for each cell and each bucket of need,
    /// so the score it compared was taken at the lower bound of the
    /// bucket.[^1] An explanation that scored the exact need would report a
    /// winner that the unit did not take.
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    pub scored_need: Fix32,
    /// The carry class that the pass scored.
    ///
    /// The pass computes one answer for each cell, each bucket of need and
    /// each carry class, so an explanation that scored one class and reported
    /// another would name a winner that the unit did not take.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D4. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    pub carry: CarryClass,
    /// The score of each option, in option index order.
    pub scores: [Fix32; OPTION_COUNT],
    /// The value each option ranked, in option index order.
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
pub fn best_option(
    need: Fix32,
    carry: CarryClass,
    summary: CellSummary,
    profile: &WeightProfile,
) -> u8 {
    let mut best = NO_INTENT;
    let mut best_score = SCORE_FLOOR;
    for option in option_order() {
        let weight = profile.weights[option as usize];
        let value = score(need, carry, weight, summary, OPTIONS[option as usize]);
        if value > best_score {
            best_score = value;
            best = option;
        }
    }
    best
}

/// The finest bucket that the answer table holds.
///
/// The table is a fixed array on the stack, so its length is a property of the
/// layout and not a budget. This exponent sets that length.
///
/// A finer bucket than this buys nothing. It approaches one answer for each
/// unit, which is the shape the cost record refuses.[^1]
///
/// # References
///
/// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
pub const NEED_BUCKET_SHIFT_FLOOR: u32 = 8;

/// The coarsest bucket that a caller may state.
///
/// At this exponent the range holds two buckets. A coarser one would hold one,
/// and every unit of a cell would then take one answer whatever it needs. The
/// limit is the width of the need and not a budget.
pub const NEED_BUCKET_SHIFT_CEILING: u32 = 16;

/// The number of entries that the answer table holds.
///
/// The value is derived from the finest bucket and from the full need. It is
/// not a second declaration of either.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
pub const NEED_BUCKET_CEILING: usize = ((NEED_FULL.0 >> NEED_BUCKET_SHIFT_FLOOR) + 1) as usize;

/// The number of entries that the answer table holds.
///
/// The key of an answer is the bucket of the need and the class of the carry,
/// so the entry count is the product of the two.[^1] Both terms are derived
/// from the values that declare them, and neither is stated a second
/// time.[^2]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D2. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
pub const ANSWER_CEILING: usize = NEED_BUCKET_CEILING * CARRY_CLASS_COUNT;

/// How finely the choice tells two needs apart.
///
/// **Unbucketed, the key is the exact need, and the distinct keys in a cell are
/// bounded by the cohorts standing in it.** A cohort is one site and one
/// faction, and its units draw one ration and hold one need exactly. So units
/// of one cohort share a key, units of two cohorts almost never do, and the
/// bound belongs to the content rather than to the engine: a world that gave
/// every unit a site of its own would put one key on every unit. **The bucket
/// is the bound the engine holds**, and that is why it is the mechanism of the
/// decision and not a detail of it.[^1] [^4]
///
/// **The width is a parameter of the world, and no record sets it.** A wide
/// bucket makes two units of different need act alike. A narrow one approaches
/// one answer for each unit. One blocker governs every cost figure this
/// project holds, and no measurement of what this pass costs exists on the
/// target platform.[^2] The reference table holds the value this world starts
/// with and the derivation of it.[^3]
///
/// The width is a power of two in the fixed-point scale, so the bucket of a
/// need is a shift and never a division.
///
/// # References
///
/// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
/// [^3]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
/// [^4]: Findings register, FND-259. `docs/FINDINGS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeedBuckets {
    shift: u32,
}

impl NeedBuckets {
    /// The width that a world starts with.
    ///
    /// **The value matches the width of a bucket to the rate at which a need
    /// moves.** The default need rule takes a fixed amount off a unit on every
    /// tick, and this bucket is that amount, so a unit crosses one bucket in
    /// one tick. A finer bucket tells apart two needs that the rule cannot
    /// separate inside a tick. A coarser one lets a need change without the
    /// bucket changing, so the choice lags the need.
    ///
    /// **The decay is itself a parameter of the need rule, and a caller who
    /// changes it should reconsider this.** That coupling is why the width is
    /// a parameter of the world rather than a constant of the module.
    ///
    /// The reference table states what it costs, what it changes, and the
    /// measurement it was chosen against.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    /// [^2]: Findings register, FND-259. `docs/FINDINGS.md`
    pub const DEFAULT: Self = Self { shift: 12 };

    /// Builds a width from the exponent of the bucket.
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is outside the range the table
    /// holds.
    pub const fn new(shift: u32) -> Result<Self, ChoiceError> {
        if shift < NEED_BUCKET_SHIFT_FLOOR || shift > NEED_BUCKET_SHIFT_CEILING {
            return Err(ChoiceError::BucketShiftOutsideRange(shift));
        }
        Ok(Self { shift })
    }

    /// Returns the exponent of the bucket.
    #[must_use]
    pub const fn shift(self) -> u32 {
        self.shift
    }

    /// Returns the number of buckets that the need range holds.
    #[must_use]
    pub const fn count(self) -> usize {
        ((NEED_FULL.0 >> self.shift) + 1) as usize
    }

    /// Returns the bucket that holds one need.
    ///
    /// The arena keeps every need between zero and the full need, and this
    /// call clamps rather than trusting that. A need outside the range gives
    /// the nearest bucket, so the answer is total.
    #[must_use]
    pub const fn bucket(self, need: Fix32) -> usize {
        if need.0 <= 0 {
            return 0;
        }
        let raw = (need.0 >> self.shift) as usize;
        let last = self.count() - 1;
        if raw >= last {
            return last;
        }
        raw
    }

    /// Returns the need that stands for one bucket.
    ///
    /// The value is the lower bound of the bucket. The last bucket holds the
    /// full need alone, so the unit that needs everything scores its exact
    /// need.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D2. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    #[must_use]
    pub const fn need(self, bucket: usize) -> Fix32 {
        Fix32((bucket as i32) << self.shift)
    }
}

impl Default for NeedBuckets {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The answers that one level 1 cell holds, for each bucket of need.
///
/// **The engine computes one answer once for every unit that would compute
/// the same answer.**[^1] The inputs to a choice are the cell of the unit
/// and the need of the unit, and nothing else, because the engine holds one
/// weight profile for every unit alive.[^2] Two units that share a cell and
/// a bucket therefore share an answer, and the table holds it.
///
/// The table fills as a unit asks for a bucket. A cell that holds three
/// units scores at most three buckets, so the table never costs more than
/// the per-unit pass it replaces. A cell that holds a thousand units scores
/// at most the bucket count, so the deciding work has a ceiling that the
/// population cannot raise.[^3]
///
/// **The lazy fill changes no answer.** The answer of a bucket depends on
/// the bucket, the cell and the profile. It does not depend on which unit
/// asked first, or on how many asked. The table therefore gives one answer
/// at any thread count.[^4]
///
/// # References
///
/// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
/// [^2]: Findings register, FND-251. `docs/FINDINGS.md`
/// [^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
/// [^4]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[derive(Clone, Copy, Debug)]
pub struct CellAnswers {
    summary: CellSummary,
    buckets: NeedBuckets,
    answers: [u8; ANSWER_CEILING],
    scored: [bool; ANSWER_CEILING],
}

impl CellAnswers {
    /// Builds an empty table over one cell summary, at one bucket width.
    #[must_use]
    pub const fn new(summary: CellSummary, buckets: NeedBuckets) -> Self {
        Self {
            summary,
            buckets,
            answers: [NO_INTENT; ANSWER_CEILING],
            scored: [false; ANSWER_CEILING],
        }
    }

    /// Returns the option that a unit of this need and this carry class takes
    /// in this cell.
    ///
    /// The call scores the entry the first time a unit asks for it, and reads
    /// the stored answer every time after.
    ///
    /// **The key is the bucket of the need and the class of the carry.** Two
    /// units that share both share an answer, and the engine computes it
    /// once.[^1] The class holds a fixed number of values, so the entry count
    /// has a ceiling that the population cannot raise.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    /// [^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    pub fn answer(&mut self, need: Fix32, carry: CarryClass, profile: &WeightProfile) -> u8 {
        let bucket = self.buckets.bucket(need);
        let at = entry_of(bucket, carry);
        if !self.scored[at] {
            self.answers[at] = best_option(self.buckets.need(bucket), carry, self.summary, profile);
            self.scored[at] = true;
        }
        self.answers[at]
    }

    /// Returns the number of buckets that this table has scored.
    ///
    /// This is what makes the cost claim checkable. A reviewer reads the
    /// claim that the deciding work follows the lattice; this count lets a
    /// test read it as well. The record states that nothing enforces the
    /// claim, and this is the part of it that a test can hold.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    #[must_use]
    pub fn scored_count(&self) -> usize {
        self.scored.iter().filter(|scored| **scored).count()
    }
}

/// Returns the entry of one bucket and one carry class.
///
/// The bucket is the major term, so the two entries of one bucket sit beside
/// each other. The layout is stated once, here, and every reader of the table
/// goes through it.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[must_use]
pub const fn entry_of(bucket: usize, carry: CarryClass) -> usize {
    bucket * CARRY_CLASS_COUNT + carry.index()
}

/// What one unit brings to a choice.
///
/// The two fields are the whole of it. The engine holds one weight profile for
/// every unit alive, so the cell, the need and the carry class are the only
/// inputs a choice has.[^1] [^2]
///
/// # References
///
/// [^1]: Findings register, FND-251. `docs/FINDINGS.md`
/// [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitState {
    /// The need that the unit still holds.
    pub need: Fix32,
    /// The class of what the unit carries.
    pub carry: CarryClass,
}

/// Returns the scores of one unit and the option they select.
#[must_use]
pub fn explain(
    cell: u32,
    unit: UnitState,
    summary: CellSummary,
    profile: &WeightProfile,
    buckets: NeedBuckets,
    intent: u8,
    chooses_next_frame: bool,
) -> ChoiceExplanation {
    // The pass scores the bucket of the need and not the need, so the
    // explanation scores the same value. An explanation taken at the exact
    // need would name a winner that the unit did not take.
    let UnitState { need, carry } = unit;
    let scored_need = buckets.need(buckets.bucket(need));
    let mut scores = [Fix32::ZERO; OPTION_COUNT];
    let mut fields = [Fix32::ZERO; OPTION_COUNT];
    for (index, option) in OPTIONS.iter().enumerate() {
        fields[index] = ranked_value(summary, carry, option.ranked);
        scores[index] = score(scored_need, carry, profile.weights[index], summary, *option);
    }
    ChoiceExplanation {
        cell,
        need,
        scored_need,
        carry,
        scores,
        fields,
        weights: profile.weights,
        floor: SCORE_FLOOR,
        best: best_option(scored_need, carry, summary, profile),
        intent,
        chooses_next_frame,
    }
}

/// Returns the resource that one option gathers.
///
/// Returns `None` when the option gathers nothing, and when the value is not
/// an option index at all. A unit that holds no intent therefore gathers
/// nothing, in the same call and by the same rule.[^1]
///
/// The answer comes from the option row and from nowhere else, so the map
/// from an option to a resource kind has one declaration site.[^2]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
/// [^2]: Findings register, FND-191. `docs/FINDINGS.md`
#[must_use]
pub const fn gathers(option: u8) -> Option<ResourceKind> {
    if option as usize >= OPTION_COUNT {
        return None;
    }
    OPTIONS[option as usize].gathers
}
