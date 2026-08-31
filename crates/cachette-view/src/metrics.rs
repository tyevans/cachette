//! What the run cost, measured rather than derived.
//!
//! Every cost figure in this project is derived, and the blocker that holds
//! them asks for measurement on the target platform.[^1] This module is the
//! first thing that measures anything. It measures a development machine,
//! which is not the target, so a figure it prints answers a smaller question
//! than the blocker asks: it says what this binary did on this machine, on
//! this run.
//!
//! # Why a clock is allowed here
//!
//! The workspace bans reading a clock, because a solver that stops on a time
//! budget gives a different answer on a loaded machine.[^2] That ban protects
//! the simulation from a measurement that *decides* something.
//!
//! Nothing here decides anything. The engine runs the same number of steps
//! whatever these numbers say, no branch reads them, and the totals are
//! printed after the loop has ended. A measurement that no code reads cannot
//! change a result.
//!
//! The distinction is the whole reason the ban exists, so it is stated rather
//! than assumed: **the ban is on a clock that steers work, not on a clock
//! that reports it.**
//!
//! # References
//!
//! [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
//! [^2]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decision D2. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`

// The clock is banned for the simulation. See the module comment: this
// module reports, and never steers.
#![allow(clippy::disallowed_methods)]

use std::time::{Duration, Instant};

/// Returns a span in microseconds.
fn micros(span: Duration) -> f64 {
    span.as_secs_f64() * 1_000_000.0
}

/// A point in time, for measuring one span.
///
/// The type exists so that the clock is named in one module. A caller starts
/// a lap and hands it back; it never reads a clock itself. Keeping the clock
/// in one place is what makes "nothing here decides anything" checkable by
/// reading one file.
#[derive(Clone, Copy, Debug)]
pub struct Lap(Instant);

impl Lap {
    /// Starts a lap.
    #[must_use]
    pub fn start() -> Self {
        Self(Instant::now())
    }

    /// Returns the time since the lap started.
    #[must_use]
    pub fn elapsed(self) -> Duration {
        self.0.elapsed()
    }
}

/// What one run cost.
///
/// The viewer keeps the totals and the extremes. It keeps no history, so the
/// cost of measuring does not grow with the length of the run.
#[derive(Debug)]
pub struct Metrics {
    started: Instant,
    ticks: u64,
    frames: u64,
    step_total: Duration,
    step_worst: Duration,
    draw_total: Duration,
    draw_worst: Duration,
    show_total: Duration,
}

impl Metrics {
    /// Starts measuring.
    #[must_use]
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
            ticks: 0,
            frames: 0,
            step_total: Duration::ZERO,
            step_worst: Duration::ZERO,
            draw_total: Duration::ZERO,
            draw_worst: Duration::ZERO,
            show_total: Duration::ZERO,
        }
    }

    /// Records one engine step.
    pub fn step(&mut self, elapsed: Duration) {
        self.ticks += 1;
        self.step_total += elapsed;
        self.step_worst = self.step_worst.max(elapsed);
    }

    /// Records one painting.
    pub fn draw(&mut self, elapsed: Duration) {
        self.frames += 1;
        self.draw_total += elapsed;
        self.draw_worst = self.draw_worst.max(elapsed);
    }

    /// Records one hand-off to the window.
    ///
    /// The window sleeps here to hold its own frame rate, so this is mostly
    /// waiting. It is measured separately for that reason: adding it to the
    /// drawing time would make the viewer look far more expensive than it is.
    pub fn show(&mut self, elapsed: Duration) {
        self.show_total += elapsed;
    }

    /// Returns the number of steps.
    #[must_use]
    pub const fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Returns the number of paintings.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.frames
    }

    /// Returns the mean cost of one step, in microseconds.
    #[must_use]
    pub fn step_mean_micros(&self) -> f64 {
        Self::mean_micros(self.step_total, self.ticks)
    }

    /// Returns the worst cost of one step, in microseconds.
    #[must_use]
    pub fn step_worst_micros(&self) -> f64 {
        micros(self.step_worst)
    }

    /// Returns the mean cost of one painting, in microseconds.
    #[must_use]
    pub fn draw_mean_micros(&self) -> f64 {
        Self::mean_micros(self.draw_total, self.frames)
    }

    /// Returns the worst cost of one painting, in microseconds.
    #[must_use]
    pub fn draw_worst_micros(&self) -> f64 {
        micros(self.draw_worst)
    }

    /// Returns the steps each second, over the whole run so far.
    ///
    /// The window holds the loop to its own frame rate, so this reports the
    /// rate the person sees rather than the rate the engine could reach.
    #[must_use]
    pub fn ticks_each_second(&self) -> f64 {
        let wall = self.started.elapsed().as_secs_f64();
        if wall > 0.0 {
            self.ticks as f64 / wall
        } else {
            0.0
        }
    }

    /// Returns the share of the wall clock that the engine and the viewer
    /// spent working, as a number from 0 to 100.
    #[must_use]
    pub fn busy_percent(&self) -> f64 {
        let wall = self.started.elapsed().as_secs_f64();
        if wall <= 0.0 {
            return 0.0;
        }
        (self.step_total + self.draw_total).as_secs_f64() / wall * 100.0
    }

    /// Returns the mean of a total over a count, in microseconds.
    fn mean_micros(total: Duration, count: u64) -> f64 {
        if count == 0 {
            return 0.0;
        }
        micros(total) / count as f64
    }

    /// Writes the report.
    ///
    /// The report says what was measured and what was not. A figure without
    /// its conditions is the kind of number that becomes a budget by being
    /// quoted, and this project keeps budgets out of its records for exactly
    /// that reason.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
    pub fn report(&self, tiles: u32, soldiers: usize, threads: usize) {
        // Every figure comes from the accessor that the head-up display also
        // reads. One derivation, one answer: a report that recomputed a mean
        // its own way could disagree with the panel and nothing would fail.
        let wall = self.started.elapsed();
        let step_mean = self.step_mean_micros();
        let draw_mean = self.draw_mean_micros();
        let ticks_per_second = self.ticks_each_second();

        println!();
        println!("measured on this machine, on this run:");
        println!("  world        {tiles} tiles, {soldiers} soldiers, {threads} threads");
        println!(
            "  ran          {} ticks and {} frames in {:.2} s",
            self.ticks,
            self.frames,
            wall.as_secs_f64()
        );
        println!("  rate         {ticks_per_second:.1} ticks each second");
        println!(
            "  step         {step_mean:.0} us mean, {:.0} us worst",
            self.step_worst_micros()
        );
        println!(
            "  draw         {draw_mean:.0} us mean, {:.0} us worst",
            self.draw_worst_micros()
        );
        println!(
            "  busy         {:.1} percent of the wall clock, the rest waits for the window",
            self.busy_percent()
        );

        if self.draw_total > self.step_total {
            println!("  the drawing cost more than the simulation");
        } else {
            println!("  the simulation cost more than the drawing");
        }

        println!();
        println!("this is not a measurement of the target platform, which is what");
        println!("BLK-007 asks for. It is a development machine, one run, one world");
        println!("size, with the window holding the loop to its own frame rate.");
    }
}
