//! The named stages of one frame, and what each one costs.
//!
//! # Why this exists
//!
//! A frame is a sequence of passes. Until this module existed, a pass was a
//! private method on the world, so a cost belonged to the frame and not to
//! the pass. The only way to price a pass was to run a whole frame with the
//! pass switched off and take the difference, and only three switches exist.
//! A measurement taken that way left 62 percent of the cost of a unit in one
//! residual that nothing on the public interface could divide.[^1]
//!
//! This module names every stage the step runs, and records what each one
//! costs. The residual then divides.
//!
//! # What it does not do
//!
//! It does not declare what a stage reads or what a stage writes. That is the
//! other half of the item this module comes from.[^2] The declaration is an
//! architectural claim rather than an instrument. A declaration that nothing
//! checks against the code is also the defect shape this project meets most
//! often.[^3]
//!
//! # Determinism
//!
//! Nothing here reaches the simulation. The stage cost table reads a clock
//! and writes two integers to a static, and no pass reads either. The step takes the
//! same branches whether the table is compiled in or not, because a span is
//! an empty type without the feature. The table therefore cannot change a
//! result at any thread count.[^4]
//!
//! # How to switch it on
//!
//! The table is behind the `stage-cost` feature and it is off by default.
//! Without the feature this module compiles to the enumeration and nothing
//! else: `open` returns an empty value, `costs` reports zero, and no static
//! exists.
//!
//! ```text
//! cargo bench --bench target_cost --features stage-cost -- stage-cost 4096x4096 1000000 12
//! ```
//!
//! # The table is a total, not a sample
//!
//! `nanos` holds the sum over every frame since the last `reset`, and
//! `entries` holds how many times the step opened that stage. A caller that
//! wants the cost of one frame resets, steps once, and reads. A caller that
//! wants an average resets, steps many times, and divides.
//!
//! # References
//!
//! [^1]: Target platform costs, where the unit cost goes. `docs/reference/graviton-costs.md`
//! [^2]: Backlog item 0237, declare what each stage reads and writes. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
//! [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`

/// Declares the stages once. The enumeration, the count, the list, the name
/// and the two declarations all come from that one list.
///
/// One list is the point. A stage named in two places would be the shape this
/// project meets most often, and the two copies would disagree the first time
/// somebody added a pass.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
macro_rules! declare_stages {
    ($($variant:ident, $name:literal, $threaded:literal, $frame_entries:literal, $nested:literal;)+) => {
        /// One named pass of a frame.
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
        #[repr(u8)]
        pub enum Stage {
            $(
                #[doc = $name]
                $variant,
            )+
        }

        /// Every stage, in the order the step runs them.
        pub const STAGES: &[Stage] = &[$(Stage::$variant),+];

        /// How many stages the step runs.
        pub const STAGE_COUNT: usize = STAGES.len();

        impl Stage {
            /// Returns the name of the stage, as the register writes it.
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Stage::$variant => $name),+
                }
            }

            /// Returns whether the stage takes a thread count.
            ///
            /// This is a declaration and not a measurement. A stage that
            /// takes no thread count cannot improve with one, so a measured
            /// speedup far from 1.0 on a stage declared `false` means the
            /// declaration is wrong. The benchmark prints this column beside
            /// the measured cost so that the two can be compared.
            #[must_use]
            pub const fn takes_threads(self) -> bool {
                match self {
                    $(Stage::$variant => $threaded),+
                }
            }

            /// Returns how many times one frame opens this stage.
            ///
            /// The step calls the bridge refresh three times, at three
            /// separate positions, and each position is its own stage. Every
            /// other stage runs once. A test drives one frame and compares
            /// the table against this column, so a stage that loses its span
            /// fails rather than reporting zero in silence.
            ///
            /// **The two inner stages of the holding spread need a world in
            /// which some tile can change hands.** The spread returns early
            /// when no tile can, and then it decides nothing and applies
            /// nothing. The fixture that checks this column places units, so
            /// it reaches the case. A world with no unit would open them
            /// zero times, and that is the pass doing no work rather than a
            /// span that went missing.
            #[must_use]
            pub const fn entries_for_each_frame(self) -> u64 {
                match self {
                    $(Stage::$variant => $frame_entries),+
                }
            }

            /// Returns whether this stage sits inside another one.
            ///
            /// A nested stage divides a stage above it, so its time is
            /// already counted there. The sum over the stages skips a nested
            /// row, and the benchmark prints one so that a reader can see
            /// where a large stage spends its cost.
            #[must_use]
            pub const fn is_nested(self) -> bool {
                match self {
                    $(Stage::$variant => $nested),+
                }
            }

            /// Returns the index of the stage in the table.
            #[must_use]
            pub const fn index(self) -> usize {
                self as usize
            }
        }
    };
}

declare_stages! {
    TileScan,                 "tile_scan",                  true,  1, false;
    LogJoin,                  "log_join",                   false, 1, false;
    ChangeMerge,              "change_merge",               false, 1, false;
    BridgeRefreshOpening,     "bridge_refresh_opening",     true,  1, false;
    Choose,                   "choose",                     true,  1, false;
    MovementIntents,          "movement_intents",           true,  1, false;
    Admit,                    "admit",                      true,  1, false;
    PlaceGranted,             "place_granted",              false, 1, false;
    BridgeRefreshBarrier,     "bridge_refresh_barrier",     true,  1, false;
    DepletionRecover,         "depletion_recover",          false, 1, false;
    Gather,                   "gather",                     true,  1, false;
    Build,                    "build",                      true,  1, false;
    HoldingSpread,            "holding_spread",             true,  1, false;
    HoldingCandidates,        "holding_candidates",         true,  1, true;
    HoldingDecide,            "holding_decide",             true,  1, true;
    HoldingApply,             "holding_apply",              false, 1, true;
    StampHolders,             "stamp_holders",              false, 1, false;
    ApplyRates,               "apply_rates",                true,  1, false;
    Consume,                  "consume",                    true,  1, false;
    Reap,                     "reap",                       true,  1, false;
    BridgeRefreshAfterReap,   "bridge_refresh_after_reap",  true,  1, false;
    SettlePositions,          "settle_positions",           true,  1, false;
    RebuildLevel1,            "rebuild_level_1",            true,  1, false;
    InfluenceSolve,           "influence_solve",            true,  1, false;
}

/// What one stage cost, and how many times the step ran it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct StageCost {
    /// The total time in the stage since the last reset, in nanoseconds.
    pub nanos: u64,
    /// How many times the step opened the stage since the last reset.
    pub entries: u64,
}

/// What every stage cost since the last reset.
#[derive(Clone, Copy, Debug)]
pub struct FrameCosts {
    costs: [StageCost; STAGE_COUNT],
}

impl FrameCosts {
    /// Returns what one stage cost.
    #[must_use]
    pub const fn cost(&self, stage: Stage) -> StageCost {
        self.costs[stage.index()]
    }

    /// Returns the total across every stage, in nanoseconds.
    ///
    /// A nested stage divides a stage above it, so its time is already in
    /// this sum through that stage. Adding it again would count it twice.
    #[must_use]
    pub fn total_nanos(&self) -> u64 {
        let mut total = 0u64;
        for stage in STAGES {
            if stage.is_nested() {
                continue;
            }
            total = total.saturating_add(self.costs[stage.index()].nanos);
        }
        total
    }
}

#[cfg(feature = "stage-cost")]
mod enabled {
    use core::sync::atomic::{AtomicU64, Ordering};

    use super::{FrameCosts, Stage, StageCost, STAGE_COUNT};

    /// The time spent in each stage, in nanoseconds.
    static NANOS: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];

    /// How many times the step opened each stage.
    static ENTRIES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];

    /// Reads the clock.
    ///
    /// One lint forbids the clock across this workspace, because a simulation
    /// that reads a clock gives an answer that depends on the load of the
    /// machine.[^1] The table is the one caller inside this crate that must
    /// read it. It produces no simulated state, it enters no state hash, and
    /// it is compiled out unless the caller asks for it. The allowance sits
    /// on this function alone.
    ///
    /// # References
    ///
    /// [^1]: ADR-0005, decision D1. `docs/adrs/REGISTRY.md`
    #[allow(clippy::disallowed_methods)]
    fn now() -> std::time::Instant {
        std::time::Instant::now()
    }

    /// Times one stage. The span closes when it is dropped.
    #[derive(Debug)]
    pub struct Span {
        stage: Stage,
        start: std::time::Instant,
    }

    impl Drop for Span {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed().as_nanos();
            let elapsed = u64::try_from(elapsed).unwrap_or(u64::MAX);
            let index = self.stage.index();
            NANOS[index].fetch_add(elapsed, Ordering::Relaxed);
            ENTRIES[index].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Opens a span over one stage.
    #[must_use]
    pub fn open(stage: Stage) -> Span {
        Span {
            stage,
            start: now(),
        }
    }

    /// Clears the table.
    pub fn reset() {
        for index in 0..STAGE_COUNT {
            NANOS[index].store(0, Ordering::Relaxed);
            ENTRIES[index].store(0, Ordering::Relaxed);
        }
    }

    /// Reads the table.
    #[must_use]
    pub fn costs() -> FrameCosts {
        let mut costs = [StageCost::default(); STAGE_COUNT];
        for (index, cost) in costs.iter_mut().enumerate() {
            *cost = StageCost {
                nanos: NANOS[index].load(Ordering::Relaxed),
                entries: ENTRIES[index].load(Ordering::Relaxed),
            };
        }
        FrameCosts { costs }
    }
}

#[cfg(not(feature = "stage-cost"))]
mod enabled {
    use super::{FrameCosts, Stage, StageCost, STAGE_COUNT};

    /// An empty span. Without the feature the table records nothing, and
    /// this type holds nothing, so the step pays no branch and no store.
    #[derive(Debug)]
    pub struct Span;

    /// Closes the span, and does nothing.
    ///
    /// The implementation is empty and the compiler removes it. It exists so
    /// that a span behaves the same way in both builds: a caller that ends a
    /// span early by dropping it writes one line, and that line does not
    /// change meaning when the feature goes off.
    impl Drop for Span {
        fn drop(&mut self) {}
    }

    /// Opens a span over one stage. Without the feature this does nothing.
    #[must_use]
    pub fn open(_stage: Stage) -> Span {
        Span
    }

    /// Clears the table. Without the feature there is nothing to clear.
    pub const fn reset() {}

    /// Reads the table. Without the feature every stage reports zero.
    #[must_use]
    pub const fn costs() -> FrameCosts {
        FrameCosts {
            costs: [StageCost {
                nanos: 0,
                entries: 0,
            }; STAGE_COUNT],
        }
    }
}

pub use enabled::{costs, open, reset, Span};

/// Returns whether this build records a cost.
///
/// A caller that reads a table of zeros needs to tell "the frame cost
/// nothing" from "this build does not measure". This function is the
/// difference, and the benchmark prints it.
#[must_use]
pub const fn is_recording() -> bool {
    cfg!(feature = "stage-cost")
}
