//! The win-path balance values that a world holds.
//!
//! A win path ends a game when a reader fires. Each reader compares a running
//! quantity of a faction against a value. This module holds the values that
//! the readers compare, and the one rate that feeds a reader, so that a
//! caller sets them at run time and needs no rebuild.[^1]
//!
//! **A value here is state.** The step reads it, so it enters the state
//! hash.[^2] No value here is a floating point number. Every value is a whole
//! number or a raw Q16.16 fixed-point value.[^3]
//!
//! **A threshold is not a rate.** A threshold decides when a reader fires and
//! changes nothing else. A rate changes what the simulation does. The renown
//! target is a threshold. The renown a felled unit gives is a rate.[^1]
//!
//! # References
//!
//! [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decisions D1 and D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
//! [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::contest::RENOWN_PER_FELL;
use crate::hash::StateHash;
use crate::types::Fix32;

/// The renown at which the renown reader fires, as a raw Q16.16 value.
///
/// A provisional value of 50 whole units, under the blocker that asks what
/// raises renown.[^1] [^2] **This is the default of a world that nobody
/// configures.** A caller changes it, and a world that nobody configures
/// holds this.
///
/// **The renown path fired in none of 252 measured games.** One source raises
/// renown, and it gives a quarter of a point for each unit the faction fells,
/// to one champion while that champion lives. A target of 1000 therefore asks
/// one character to outlive 4000 deaths, and the progress a policy reads sat
/// flat near zero for a whole game. A target of 50 asks for 200, which is a
/// feat a champion can reach and a policy can steer toward.
///
/// # References
///
/// [^1]: Balance register, the renown target. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-150. `docs/BLOCKERS.md`
pub const RENOWN_TARGET: i32 = 50 << 16;

/// The win-path balance values of one world.
///
/// Every field carries the constant that names its default, so a world that
/// nobody configures behaves as it did before this table existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Balance {
    /// The renown at which the renown reader fires, as a raw Q16.16 value.
    renown_target: i32,
    /// The renown that one felled unit gives the champion of the faction
    /// that felled it.
    renown_per_fell: Fix32,
    /// One while the readers run, zero while they record nothing.
    win_readers_enabled: u8,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            renown_target: RENOWN_TARGET,
            renown_per_fell: RENOWN_PER_FELL,
            win_readers_enabled: 1,
        }
    }
}

impl Balance {
    /// Returns the renown at which the renown reader fires, as a raw Q16.16
    /// value.
    #[must_use]
    pub const fn renown_target(&self) -> i32 {
        self.renown_target
    }

    /// Sets the renown at which the renown reader fires, as a raw Q16.16
    /// value.
    pub const fn set_renown_target(&mut self, raw: i32) {
        self.renown_target = raw;
    }

    /// Returns the renown that one felled unit gives the champion of the
    /// faction that felled it, as a raw Q16.16 value.
    #[must_use]
    pub const fn renown_per_fell(&self) -> Fix32 {
        self.renown_per_fell
    }

    /// Sets the renown that one felled unit gives the champion of the faction
    /// that felled it, as a raw Q16.16 value.
    pub const fn set_renown_per_fell(&mut self, raw: i32) {
        self.renown_per_fell = Fix32(raw);
    }

    /// Reports whether the game end readers run.
    #[must_use]
    pub const fn win_readers_enabled(&self) -> bool {
        self.win_readers_enabled != 0
    }

    /// Sets whether the game end readers run.
    ///
    /// While they do not run, no reader records a game end and the world runs
    /// to the tick limit. The readers decide when the step stops watching,
    /// and they change nothing else, so a run with the readers off holds the
    /// same event log as a run with the readers on that never fires.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_win_readers_enabled(&mut self, enabled: bool) {
        self.win_readers_enabled = enabled as u8;
    }

    /// Folds every value into the hash.
    ///
    /// Each value is one a later frame reads, so each enters.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(self.renown_target as u64)
            .write_u64(self.renown_per_fell.0 as u64)
            .write_u64(u64::from(self.win_readers_enabled))
    }
}
