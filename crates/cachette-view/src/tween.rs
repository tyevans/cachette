//! Moves a painted unit between two tiles, and names the speed of the run.
//!
//! The engine moves a unit one whole tile in one tick, and the world holds
//! one tick at a time. A viewer that draws a unit between two tiles must
//! remember where the unit was, because the engine does not. That memory is
//! the viewer's and never the engine's.[^1]
//!
//! # The table
//!
//! The table maps a unit identity to the tile the unit is moving from. It
//! holds an entry for a unit painted on the last frame and for no other
//! unit. The frame rebuilds it from the units it paints, so a unit that left
//! the view, died, or fell outside the frame is gone from the table when the
//! frame ends. That is the eviction rule.
//!
//! **The table is bounded by the frame and never by the population.** The
//! caller gives the bound, and the frame command uses the pixel count of the
//! frame. A frame cannot show more units than it has pixels, so the bound
//! loses nothing a watcher could see. A unit painted past the bound is not
//! recorded, and it draws at its tile on the next frame.
//!
//! The table is read by key and never iterated, so its order reaches no
//! pixel. No pixel enters a state hash, so a fractional position here breaks
//! no determinism rule.[^2]
//!
//! # The phase
//!
//! The phase is the share of the current tick that has elapsed on the wall
//! clock, in the half-open range from zero to one. A phase of zero means the
//! tick is complete and every unit stands on its tile. A phase above zero
//! puts a unit that moved at that share of the way from the old centre to
//! the new one, and the table keeps the old tile until the phase returns to
//! zero.
//!
//! The phase is a floating point number and it stops here. Nothing formed
//! from it reaches the engine.[^3]
//!
//! # The speed
//!
//! The speed crosses the boundary as an integer, in thousandths of a tick
//! for each frame. Zero means paused. The viewer renders the word, so the
//! caller sends no text.[^4]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^4]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`

use std::collections::HashMap;

use cachette_core::{Axial, Entity};

/// The furthest a unit may move in one tick and still draw between two
/// tiles, in tiles.
///
/// A unit that moved further than this jumped: something respawned it,
/// converted it, or placed it by a verb. A tween across a jump would draw a
/// unit crossing ground it never crossed, so the unit draws at its tile.
pub const TWEEN_REACH: u32 = 2;

/// The speed of a caller that runs one tick for each frame, in thousandths
/// of a tick for each frame.
pub const ONE_TICK_EACH_FRAME: u32 = 1000;

/// How far the wall clock is through the tick, and how fast the world runs.
///
/// Both values belong to the caller. The engine holds no clock and no
/// speed.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pace {
    /// The share of the current tick that has elapsed, from zero to one.
    pub phase: f32,
    /// The ticks each frame runs, in thousandths. Zero means paused.
    pub speed_milli: u32,
}

impl Pace {
    /// The pace of a caller that runs one tick for each frame and tweens
    /// nothing.
    pub const STILL: Self = Self {
        phase: 0.0,
        speed_milli: ONE_TICK_EACH_FRAME,
    };

    /// Builds a pace, holding the phase inside its range.
    ///
    /// A phase at or above one names the same moment as a phase of zero: the
    /// tick is complete. A phase below zero, and a phase that is not a
    /// number, read as zero.
    #[must_use]
    pub fn new(phase: f32, speed_milli: u32) -> Self {
        let phase = if phase.is_finite() && (0.0..1.0).contains(&phase) {
            phase
        } else {
            0.0
        };
        Self { phase, speed_milli }
    }

    /// Returns the speed as the word the window shows.
    #[must_use]
    pub fn speed_word(self) -> String {
        speed_word(self.speed_milli)
    }
}

/// Renders a speed in thousandths of a tick for each frame as a word.
///
/// Zero is `paused`. A whole number of ticks reads `x2`. A unit fraction
/// reads `x1/2`. Any other value reads to three decimal places.
///
/// **The viewer holds the words and the caller holds the number.** The
/// engine takes no text from the caller.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
#[must_use]
pub fn speed_word(speed_milli: u32) -> String {
    if speed_milli == 0 {
        return "paused".to_string();
    }
    if speed_milli.is_multiple_of(ONE_TICK_EACH_FRAME) {
        return format!("x{}", speed_milli / ONE_TICK_EACH_FRAME);
    }
    if ONE_TICK_EACH_FRAME.is_multiple_of(speed_milli) {
        return format!("x1/{}", ONE_TICK_EACH_FRAME / speed_milli);
    }
    format!(
        "x{}.{:03}",
        speed_milli / ONE_TICK_EACH_FRAME,
        speed_milli % ONE_TICK_EACH_FRAME
    )
}

/// Returns the distance between two tiles, in tiles.
///
/// The arithmetic is the viewer's own, and the engine is not asked.
#[must_use]
pub fn hex_distance(from: Axial, to: Axial) -> u32 {
    let across = i64::from(to.q) - i64::from(from.q);
    let down = i64::from(to.r) - i64::from(from.r);
    let third = -(across + down);
    let far = across.abs().max(down.abs()).max(third.abs());
    u32::try_from(far).unwrap_or(u32::MAX)
}

/// Where each painted unit stood on the last frame, by identity.
///
/// The module documentation states the bound and the eviction rule.
#[derive(Debug)]
pub struct Motion {
    /// The tile each unit painted on the last frame moves from.
    last: HashMap<Entity, Axial>,
    /// The entries the current frame has recorded.
    next: Vec<(Entity, Axial)>,
    /// The most entries the table holds.
    bound: usize,
}

impl Motion {
    /// Builds a table that holds at most this many units.
    #[must_use]
    pub fn bounded(bound: usize) -> Self {
        Self {
            last: HashMap::new(),
            next: Vec::new(),
            bound,
        }
    }

    /// Builds a table that records nothing, for a caller that tweens nothing.
    #[must_use]
    pub fn none() -> Self {
        Self::bounded(0)
    }

    /// Builds a table bounded by the pixels of a frame.
    ///
    /// A frame cannot show more units than it has pixels, so this bound
    /// loses nothing a watcher could see.
    #[must_use]
    pub fn for_frame(width: usize, height: usize) -> Self {
        Self::bounded(width.saturating_mul(height))
    }

    /// Returns the most entries the table holds.
    #[must_use]
    pub const fn bound(&self) -> usize {
        self.bound
    }

    /// Returns how many units the table remembers from the last frame.
    #[must_use]
    pub fn len(&self) -> usize {
        self.last.len()
    }

    /// Reports whether the table remembers no unit.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.last.is_empty()
    }

    /// Returns the tile a unit moves from, when the last frame painted it.
    #[must_use]
    pub fn moving_from(&self, entity: Entity) -> Option<Axial> {
        self.last.get(&entity).copied()
    }

    /// Opens a frame. Every entry recorded before this belongs to the frame
    /// before it.
    pub(crate) fn begin(&mut self) {
        self.next.clear();
    }

    /// Places one painted unit, and returns the tile to draw it from.
    ///
    /// Returns the tile the unit moves from when the unit draws between that
    /// tile and its address at the phase. Returns nothing when the unit
    /// draws at its tile: the phase is zero, the table does not hold the
    /// unit, the unit did not move, or it moved further than the reach.
    ///
    /// The entry for the next frame is the tile the unit moves from while
    /// the tween runs, and the address of the unit otherwise. A unit past
    /// the bound is not recorded.
    pub(crate) fn place(&mut self, entity: Entity, address: Axial, phase: f32) -> Option<Axial> {
        let from = self.last.get(&entity).copied().filter(|&from| {
            phase > 0.0 && from != address && hex_distance(from, address) <= TWEEN_REACH
        });
        if self.next.len() < self.bound {
            self.next.push((entity, from.unwrap_or(address)));
        }
        from
    }

    /// Closes a frame. The entries it recorded become the last frame's, and
    /// a unit the frame did not record is evicted here.
    pub(crate) fn end(&mut self) {
        self.last.clear();
        self.last.extend(self.next.drain(..));
    }
}

/// Returns the point at this share of the way from one point to another.
#[must_use]
pub fn between(from: (f32, f32), to: (f32, f32), phase: f32) -> (f32, f32) {
    (
        from.0 + (to.0 - from.0) * phase,
        from.1 + (to.1 - from.1) * phase,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speed_word_names_each_speed() {
        assert_eq!(speed_word(0), "paused");
        assert_eq!(speed_word(250), "x1/4");
        assert_eq!(speed_word(500), "x1/2");
        assert_eq!(speed_word(1000), "x1");
        assert_eq!(speed_word(2000), "x2");
        assert_eq!(speed_word(8000), "x8");
        assert_eq!(speed_word(1500), "x1.500");
    }

    #[test]
    fn a_phase_outside_the_range_reads_as_zero() {
        assert_eq!(Pace::new(1.0, 1000).phase, 0.0);
        assert_eq!(Pace::new(-0.5, 1000).phase, 0.0);
        assert_eq!(Pace::new(f32::NAN, 1000).phase, 0.0);
        assert_eq!(Pace::new(0.5, 1000).phase, 0.5);
    }

    #[test]
    fn the_distance_counts_tiles() {
        assert_eq!(hex_distance(Axial::new(0, 0), Axial::new(0, 0)), 0);
        assert_eq!(hex_distance(Axial::new(0, 0), Axial::new(1, 0)), 1);
        assert_eq!(hex_distance(Axial::new(0, 0), Axial::new(1, -1)), 1);
        assert_eq!(hex_distance(Axial::new(0, 0), Axial::new(2, 1)), 3);
    }
}
