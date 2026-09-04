//! Contractual trade between two factions, and the negotiation that makes it.
//!
//! A trade has two halves. The negotiation is a conversation: an offer, a
//! counteroffer, an acceptance, a refusal, and a terminal refusal that closes
//! the exchange. The contract is an obligation that the world enforces: a
//! quantity of a resource moves because a contract says so.
//!
//! **Both halves are simulated state.** The terms and the status enter the
//! state hash, so two worlds that hold the same tiles and different contracts
//! give different hashes.[^1] The words that a player writes are not here. The
//! engine holds no text, no channel and no delivery between players.[^2]
//!
//! **The table is a plane over ordered pairs of factions.** A faction is one
//! bit in a mask, and a relation between factions is a plane of such
//! values.[^3] The row for the pair `(A, B)` holds the negotiation that A
//! opened toward B. The plane never follows the population, so a world with
//! one unit and a world at the target population hold the same table.
//!
//! **The plane holds nothing until somebody speaks.** A world in which nobody
//! traded allocates no row and writes no byte into the state hash. This is the
//! shape the sparse upgrade store already uses.[^4]
//!
//! **Nothing in this module draws a random number.** Every outcome follows
//! from the terms, the tick and what the carriers delivered. There is no draw,
//! so there is no key.[^5]
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0120, a trade negotiation is engine state and the words are not, decision D2. `docs/adrs/draft/adr-0120-a-trade-negotiation-is-engine-state.md`
//! [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^4]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::resource::ResourceKind;
use crate::types::{FactionId, Tick};

/// The pair holds no negotiation. Either side may open one.
pub const TRADE_IDLE: u8 = 0;
/// The party that opened the pair spoke last. The other party answers.
pub const TRADE_OFFERED: u8 = 1;
/// The party that answered spoke last. The party that opened answers.
pub const TRADE_COUNTERED: u8 = 2;
/// Both parties agreed. The terms are a contract and the world enforces it.
pub const TRADE_BOUND: u8 = 3;
/// Both parties delivered in full.
pub const TRADE_SETTLED: u8 = 4;
/// The deadline passed and one party still owed.
pub const TRADE_DEFAULTED: u8 = 5;

/// The number of status values.
pub const TRADE_STATUS_COUNT: u8 = 6;

/// One party opened a negotiation.
pub const ACT_OFFER: u8 = 0;
/// One party restated the terms.
pub const ACT_COUNTER: u8 = 1;
/// One party agreed to the terms, so a contract now binds both.
pub const ACT_ACCEPT: u8 = 2;
/// One party declined. The pair may open again at once.
pub const ACT_REFUSE: u8 = 3;
/// One party declined and closed the pair until a tick it named.
pub const ACT_CLOSE: u8 = 4;
/// The party that closed the pair opened it again before that tick.
pub const ACT_REOPEN: u8 = 5;
/// A contract reached full delivery on both sides.
pub const ACT_SETTLE: u8 = 6;
/// A contract reached its deadline with a debt on one side or on both.
pub const ACT_DEFAULT: u8 = 7;

/// The number of speech acts.
pub const ACT_COUNT: u8 = 8;

/// The reason that this module refused a caller.
///
/// Each variant is a mistake that a caller can make. The module returns the
/// variant and never panics. A god that cannot tell a refusal from a closed
/// door asks for ever, so the closure carries the tick that opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeError {
    /// The two parties are one faction. A faction does not trade with itself.
    SameFaction(FactionId),
    /// The identifier is at or above the faction count of this world.
    NoSuchFaction(FactionId),
    /// The number names no resource kind.
    NoSuchKind(u8),
    /// A side of the contract binds nothing. Both sides must bind a quantity.
    EmptyTerms,
    /// The offer named no deadline. A contract that cannot fail is not a
    /// contract.
    NoDeadline,
    /// The two parties already hold a live negotiation or a live contract.
    AlreadyOpen,
    /// The pair holds no live negotiation, so there is nothing to answer.
    NothingOpen,
    /// The other party spoke last, so this party may not speak again yet.
    NotYourTurn,
    /// The terms already bind both parties, so nobody may restate them.
    AlreadyBound,
    /// A terminal refusal closed this direction until the tick named here.
    Closed(Tick),
    /// This party closed nothing, so it opens nothing.
    NothingClosed,
    /// No unit of the speaker stands on ground that the listener holds.
    NoPresence,
    /// The closure named no duration.
    NoDuration,
}

/// One ordered pair of factions, and what stands between them.
///
/// The row holds the terms in the orientation of the pair. `give` is what the
/// party that opened the pair owes. `take` is what the other party owes. A
/// counteroffer restates both, in that same orientation, so a reader never has
/// to know who spoke last in order to read the terms.
///
/// The layout is 8 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 1 + 1 + 1 + 1 bytes, which is
/// 48 bytes at an alignment of 8. The type holds no padding byte at all, so it
/// puts no uninitialised byte into the state hash. A constant below asserts the
/// size, so a field added later fails the build rather than the hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct TradeRow {
    /// The tick at which this negotiation opened.
    pub opened: Tick,
    /// The tick at which a bound contract fails if a debt remains.
    ///
    /// The value is zero until the contract binds. The offer names a term in
    /// ticks, and the acceptance turns that term into this tick, so a long
    /// negotiation never eats the delivery window.
    pub deadline: Tick,
    /// The tick at or after which this direction may open again.
    ///
    /// The value is zero when nothing closed the direction. A terminal
    /// refusal writes a tick here, and the party that wrote it is the only
    /// party that clears it early.
    pub closed_until: Tick,
    /// What the party that opened the pair owes, as a whole number.
    pub give_amount: u32,
    /// What the other party owes, as a whole number.
    pub take_amount: u32,
    /// What the party that opened the pair has delivered.
    pub given: u32,
    /// What the other party has delivered.
    pub taken: u32,
    /// How many ticks a bound contract runs for.
    pub term: u32,
    /// The resource kind that the party that opened the pair owes.
    pub give_kind: u8,
    /// The resource kind that the other party owes.
    pub take_kind: u8,
    /// The status of the pair.
    pub status: u8,
    /// How many times somebody spoke in this negotiation. It saturates.
    pub rounds: u8,
}

/// The size of one row, in bytes.
///
/// The build fails when a field changes this, because a row with padding puts
/// an uninitialised byte into the state hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
pub const TRADE_ROW_BYTES: usize = 48;

const _: () = assert!(core::mem::size_of::<TradeRow>() == TRADE_ROW_BYTES);

impl TradeRow {
    /// Returns whether the row holds a live negotiation or a live contract.
    #[must_use]
    pub const fn is_live(&self) -> bool {
        matches!(self.status, TRADE_OFFERED | TRADE_COUNTERED | TRADE_BOUND)
    }

    /// Returns whether a contract binds both parties.
    #[must_use]
    pub const fn is_bound(&self) -> bool {
        self.status == TRADE_BOUND
    }

    /// Returns what the party that opened the pair still owes.
    #[must_use]
    pub const fn owed_by_proposer(&self) -> u32 {
        self.give_amount.saturating_sub(self.given)
    }

    /// Returns what the other party still owes.
    #[must_use]
    pub const fn owed_by_responder(&self) -> u32 {
        self.take_amount.saturating_sub(self.taken)
    }

    /// Returns whether both parties have delivered in full.
    #[must_use]
    pub const fn is_paid(&self) -> bool {
        self.given >= self.give_amount && self.taken >= self.take_amount
    }

    /// Clears the negotiation and keeps the closure.
    ///
    /// A refusal ends what the parties were discussing. It never clears a
    /// closure, because a closure belongs to the party that wrote it.
    pub const fn clear(&mut self) {
        self.opened = Tick(0);
        self.deadline = Tick(0);
        self.give_amount = 0;
        self.take_amount = 0;
        self.given = 0;
        self.taken = 0;
        self.term = 0;
        self.give_kind = 0;
        self.take_kind = 0;
        self.status = TRADE_IDLE;
        self.rounds = 0;
    }
}

/// One thing that somebody said, or one thing that a contract did.
///
/// The engine holds one append-only array of these for the last step. A caller
/// reads the array at the frame barrier and never inside a step.[^1]
///
/// The layout is 8 + 2 + 2 + 1 + 1 + 2 bytes, which is 16 bytes at an
/// alignment of 8. The trailing array declares every padding byte.[^2]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct TradeSpoken {
    /// The tick at which it happened.
    pub tick: Tick,
    /// The faction that opened the pair.
    pub proposer: u16,
    /// The other faction of the pair.
    pub responder: u16,
    /// Which speech act it was.
    pub act: u8,
    /// The status of the pair after the act.
    pub status: u8,
    /// The declared padding. Always zero.
    pub padding: [u8; 2],
}

impl TradeSpoken {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        proposer: FactionId,
        responder: FactionId,
        act: u8,
        status: u8,
    ) -> Self {
        Self {
            tick,
            proposer: proposer.0,
            responder: responder.0,
            act,
            status,
            padding: [0; 2],
        }
    }
}

/// The negotiation plane, one row for each ordered pair of factions.
///
/// **The plane allocates nothing until the first speech act.** A world that
/// never traded holds no row, folds no byte into the state hash, and costs
/// nothing. The first act sizes the plane from the faction count of the world
/// and it never grows again.
///
/// The index of a pair is the proposer times the faction count plus the
/// responder. That order is the order a walk reads the rows in, so no walk
/// over this table depends on a hash order or on a thread.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug)]
pub struct TradeTable {
    /// One row for each ordered pair, or nothing at all.
    rows: Vec<TradeRow>,
    /// How many factions the world holds.
    factions: u16,
}

impl TradeTable {
    /// Builds an empty plane for a world with this many factions.
    #[must_use]
    pub const fn new(factions: u16) -> Self {
        Self {
            rows: Vec::new(),
            factions,
        }
    }

    /// Returns whether the plane holds no row.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns how many factions the plane was built for.
    #[must_use]
    pub const fn factions(&self) -> u16 {
        self.factions
    }

    /// Returns every row, in pair order.
    ///
    /// The slice is empty until somebody speaks.
    #[must_use]
    pub fn rows(&self) -> &[TradeRow] {
        &self.rows
    }

    /// Returns the index of one ordered pair, or `None` when either faction
    /// is at or above the faction count.
    #[must_use]
    pub fn index_of(&self, proposer: FactionId, responder: FactionId) -> Option<usize> {
        if proposer.0 >= self.factions || responder.0 >= self.factions {
            return None;
        }
        Some(proposer.0 as usize * self.factions as usize + responder.0 as usize)
    }

    /// Returns the row of one ordered pair.
    ///
    /// A plane that holds no row answers the empty row, because a pair that
    /// nobody ever spoke about is idle. This keeps the lazy plane and the
    /// filled plane indistinguishable to every reader.
    #[must_use]
    pub fn row(&self, proposer: FactionId, responder: FactionId) -> Option<TradeRow> {
        let index = self.index_of(proposer, responder)?;
        Some(self.rows.get(index).copied().unwrap_or_default())
    }

    /// Returns the row of one ordered pair for writing, and fills the plane
    /// when this is the first write.
    ///
    /// Returns `None` when either faction is at or above the faction count.
    pub fn row_mut(&mut self, proposer: FactionId, responder: FactionId) -> Option<&mut TradeRow> {
        let index = self.index_of(proposer, responder)?;
        if self.rows.is_empty() {
            let count = self.factions as usize * self.factions as usize;
            self.rows = vec![TradeRow::default(); count];
        }
        self.rows.get_mut(index)
    }

    /// Returns one row by its index in the plane.
    #[must_use]
    pub fn row_at(&self, index: usize) -> Option<TradeRow> {
        self.rows.get(index).copied()
    }

    /// Returns one row by its index in the plane, for writing.
    pub fn row_at_mut(&mut self, index: usize) -> Option<&mut TradeRow> {
        self.rows.get_mut(index)
    }

    /// Folds the plane into a state hash, in pair order.
    ///
    /// A plane that holds no row writes nothing, so a world that never traded
    /// hashes as it did before this module existed.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        if self.rows.is_empty() {
            return hash;
        }
        hash.write(bytemuck::cast_slice(&self.rows))
            .write_u64(u64::from(self.factions))
    }

    /// Reports whether the plane holds its invariants.
    ///
    /// The plane is either empty or exactly one row for each ordered pair.
    /// Every status names a value this module states. What a party delivered
    /// never passes what it owes, because the transfer takes the smaller of
    /// the two and a delivery that passed the debt would move a quantity that
    /// no contract asked for.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.rows.is_empty() {
            return true;
        }
        if self.rows.len() != self.factions as usize * self.factions as usize {
            return false;
        }
        self.rows.iter().all(|row| {
            row.status < TRADE_STATUS_COUNT
                && row.given <= row.give_amount
                && row.taken <= row.take_amount
        })
    }
}

/// Returns whether a number names a resource kind, and which one.
///
/// # Errors
///
/// Returns an error when the number names no kind.
pub fn kind_of(value: u8) -> Result<ResourceKind, TradeError> {
    ResourceKind::from_u8(value).ok_or(TradeError::NoSuchKind(value))
}
