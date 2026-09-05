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
//! [^2]: ADR-0126, a trade negotiation is engine state and the words are not, decision D2. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
//! [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^4]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::resource::ResourceKind;
use crate::types::{FactionId, Tick, TileIdx};

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
/// A land set changed holder, because the other side delivered in full.
pub const ACT_TRANSFER_LAND: u8 = 8;
/// A relation step fell due, because the other side delivered in full.
///
/// **The step moves nothing yet.** The relation matrix arrives with a later
/// pass, so the engine logs the act and applies no change. The event is the
/// record that the side was delivered.
pub const ACT_STEP_RELATION: u8 = 9;

/// The number of speech acts.
pub const ACT_COUNT: u8 = 10;

/// The side is a quantity of a resource. A unit carries it.
pub const KIND_RESOURCE: u8 = 0;
/// The side is a bounded set of tiles the debtor holds. The holder changes
/// when the other side is delivered in full.
pub const KIND_LAND: u8 = 1;
/// The side is a step on the relation between the pair. It applies when the
/// other side is delivered in full.
pub const KIND_RELATION: u8 = 2;

/// The number of consideration kinds. The set is closed.[^1]
///
/// # References
///
/// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/draft/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
pub const CONSIDERATION_KIND_COUNT: u8 = 3;

/// The advertisement rows one faction holds when the caller names no other.
///
/// **This is a stand-in and not a decision.** The balance register holds the
/// row and calls it unset.[^1] A caller sets the value through the world.
///
/// # References
///
/// [^1]: Balance register, board size. `docs/reference/balance.md`
pub const DEFAULT_BOARD_ROWS: u16 = 8;

/// The most tiles one land consideration names when the caller sets no other
/// bound.
///
/// **This is a stand-in and not a decision.** The balance register holds the
/// row and calls it unset.[^1] A caller sets the value through the world.
///
/// # References
///
/// [^1]: Balance register, land list bound. `docs/reference/balance.md`
pub const DEFAULT_LAND_LIST_BOUND: u32 = 64;

/// A board row that offers the good.
pub const ADVERT_OFFERS: u8 = 0;
/// A board row that wants the good.
pub const ADVERT_WANTS: u8 = 1;

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
    /// The number names no consideration kind.
    NoSuchTag(u8),
    /// A land side names a tile that lies outside the world.
    NoSuchTile,
    /// A land side names a level 1 cell that lies outside the world.
    NoSuchCell,
    /// A land side names a tile that its debtor does not hold.
    LandNotHeld(TileIdx),
    /// A land side names a tile that carries an upgrade.
    ///
    /// Whether an upgrade goes with the ground is an open question, and the
    /// engine refuses the offer until it is answered.[^1] One commit removes
    /// this variant when the blocker closes.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-036. `docs/BLOCKERS.md`
    UpgradeOnLand(TileIdx),
    /// A land side names more tiles than the bound. The fields are the count
    /// and the bound.
    TooMuchLand(u32, u32),
    /// An advertisement names neither offers nor wants.
    NoSuchSide(u8),
    /// A board write names more rows than the board holds. The fields are the
    /// count and the bound.
    BoardOverfull(u32, u32),
}

/// One ordered pair of factions, and what stands between them.
///
/// The row holds the terms in the orientation of the pair. `give` is what the
/// party that opened the pair owes. `take` is what the other party owes. A
/// counteroffer restates both, in that same orientation, so a reader never has
/// to know who spoke last in order to read the terms.
///
/// **Each side is a tagged consideration.** The tag names the kind, and the
/// kind says what the other fields mean. For a resource the kind byte is the
/// resource and the amount is the quantity. For a land set the amount is the
/// tile count, and the tiles sit beside the row in the plane. For a relation
/// step the kind byte and the amount are stored and nothing reads them yet.[^2]
///
/// The layout is 8 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 1 + 1 + 1 + 1 + 1 + 1 + 6
/// bytes, which is 56 bytes at an alignment of 8. The trailing array declares
/// every padding byte, so the type puts no uninitialised byte into the state
/// hash. A constant below asserts the size, so a field added later fails the
/// build rather than the hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/draft/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
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
    /// The kind of the consideration that the party that opened the pair
    /// owes.
    pub give_tag: u8,
    /// The kind of the consideration that the other party owes.
    pub take_tag: u8,
    /// The declared padding. Always zero.
    pub padding: [u8; 6],
}

/// The size of one row, in bytes.
///
/// The build fails when a field changes this, because a row with padding puts
/// an uninitialised byte into the state hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
pub const TRADE_ROW_BYTES: usize = 56;

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
        self.give_tag = KIND_RESOURCE;
        self.take_tag = KIND_RESOURCE;
        self.padding = [0; 6];
    }

    /// Returns whether the side of the party that opened the pair is a
    /// quantity that a unit carries.
    #[must_use]
    pub const fn proposer_side_is_carried(&self) -> bool {
        self.give_tag == KIND_RESOURCE
    }

    /// Returns whether the side of the other party is a quantity that a unit
    /// carries.
    #[must_use]
    pub const fn responder_side_is_carried(&self) -> bool {
        self.take_tag == KIND_RESOURCE
    }
}

/// One side of a contract, as a caller states it.
///
/// The tag names the kind, and the content is what the tag says.[^1] A
/// resource is a kind and a quantity. A land set is a list of tiles, and its
/// amount is the tile count. A relation step is a kind byte and an amount,
/// which the engine stores and does not yet read.
///
/// The type is not plain data, because a tile list has no fixed size. The row
/// that the plane stores is plain data, and the tiles sit beside it.
///
/// # References
///
/// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/draft/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Consideration {
    /// Which kind this is. One of the kind constants of this module.
    pub tag: u8,
    /// The resource kind for a resource. The step kind for a relation. Zero
    /// for land.
    pub kind: u8,
    /// The quantity for a resource or a relation. The tile count for land.
    pub amount: u32,
    /// The tiles of a land set. Empty for the other kinds.
    pub tiles: Vec<TileIdx>,
}

impl Consideration {
    /// A quantity of one resource.
    #[must_use]
    pub const fn resource(kind: u8, amount: u32) -> Self {
        Self {
            tag: KIND_RESOURCE,
            kind,
            amount,
            tiles: Vec::new(),
        }
    }

    /// A set of tiles. The amount is the tile count.
    #[must_use]
    pub fn land(tiles: Vec<TileIdx>) -> Self {
        Self {
            tag: KIND_LAND,
            kind: 0,
            amount: u32::try_from(tiles.len()).unwrap_or(u32::MAX),
            tiles,
        }
    }

    /// A step on the relation of the pair.
    #[must_use]
    pub const fn relation(kind: u8, amount: u32) -> Self {
        Self {
            tag: KIND_RELATION,
            kind,
            amount,
            tiles: Vec::new(),
        }
    }
}

/// One row of a faction's board.
///
/// The row says that the faction offers or wants a quantity of one good, and
/// what it asks in return. A row whose quantity is zero is empty.
///
/// The layout is 1 + 1 + 1 + 1 + 4 + 4 bytes, which is 12 bytes at an
/// alignment of 4. The padding byte is declared.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Advert {
    /// The resource kind the row is about.
    pub good: u8,
    /// Whether the faction offers the good or wants it.
    pub wants: u8,
    /// The resource kind the faction asks in return.
    pub asking_good: u8,
    /// The declared padding. Always zero.
    pub padding: u8,
    /// How much of the good. Zero means the row is empty.
    pub quantity: u32,
    /// How much of the asking good.
    pub asking_quantity: u32,
}

/// The size of one board row, in bytes.
pub const ADVERT_BYTES: usize = 12;

const _: () = assert!(core::mem::size_of::<Advert>() == ADVERT_BYTES);

impl Advert {
    /// Builds a row with zero padding.
    #[must_use]
    pub const fn new(
        good: u8,
        quantity: u32,
        wants: u8,
        asking_good: u8,
        asking_quantity: u32,
    ) -> Self {
        Self {
            good,
            wants,
            asking_good,
            padding: 0,
            quantity,
            asking_quantity,
        }
    }

    /// Returns whether the row says nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.quantity == 0
    }
}

/// The boards of every faction, one fixed block of rows for each.
///
/// **The table allocates nothing until the first write.** A world in which no
/// faction advertised holds no row and folds no byte into the state hash. The
/// first write sizes the table from the faction count and the row bound, and
/// it never grows again.
///
/// The block of a faction starts at the faction number times the row bound.
/// A walk reads the blocks in faction order.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug)]
pub struct MarketTable {
    /// One block of rows for each faction, or nothing at all.
    rows: Vec<Advert>,
    /// How many factions the world holds.
    factions: u16,
    /// How many rows one faction holds.
    bound: u16,
}

impl MarketTable {
    /// Builds an empty table for a world with this many factions.
    #[must_use]
    pub const fn new(factions: u16, bound: u16) -> Self {
        Self {
            rows: Vec::new(),
            factions,
            bound,
        }
    }

    /// Returns whether no faction has advertised.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns how many rows one faction holds.
    #[must_use]
    pub const fn bound(&self) -> u16 {
        self.bound
    }

    /// Sets how many rows one faction holds.
    ///
    /// The bound is a balance value.[^1] A change discards every board,
    /// because the blocks are laid out by the bound.
    ///
    /// # References
    ///
    /// [^1]: Balance register, board size. `docs/reference/balance.md`
    pub fn set_bound(&mut self, bound: u16) {
        self.bound = bound;
        self.rows.clear();
    }

    /// Returns the board of one faction, empty rows included.
    ///
    /// A table that holds no row answers an empty slice, because a faction
    /// that never advertised has an empty board.
    #[must_use]
    pub fn board(&self, faction: FactionId) -> &[Advert] {
        if faction.0 >= self.factions || self.rows.is_empty() {
            return &[];
        }
        let start = faction.0 as usize * self.bound as usize;
        &self.rows[start..start + self.bound as usize]
    }

    /// Replaces the whole board of one faction.
    ///
    /// The rows are written in the order given, and the rest of the block is
    /// emptied. A write with more rows than the bound changes nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction is at or above the faction count,
    /// when the write names more rows than the bound, when a good names no
    /// resource kind, or when a row names neither offers nor wants.
    pub fn advertise(&mut self, faction: FactionId, rows: &[Advert]) -> Result<(), TradeError> {
        if faction.0 >= self.factions {
            return Err(TradeError::NoSuchFaction(faction));
        }
        let count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        if count > u32::from(self.bound) {
            return Err(TradeError::BoardOverfull(count, u32::from(self.bound)));
        }
        for row in rows {
            kind_of(row.good)?;
            kind_of(row.asking_good)?;
            if row.wants != ADVERT_OFFERS && row.wants != ADVERT_WANTS {
                return Err(TradeError::NoSuchSide(row.wants));
            }
        }
        if self.rows.is_empty() {
            let count = self.factions as usize * self.bound as usize;
            self.rows = vec![Advert::default(); count];
        }
        let start = faction.0 as usize * self.bound as usize;
        let block = &mut self.rows[start..start + self.bound as usize];
        for (slot, row) in block.iter_mut().enumerate() {
            *row = rows.get(slot).copied().unwrap_or_default();
            row.padding = 0;
        }
        Ok(())
    }

    /// Folds the table into a state hash, in faction order.
    ///
    /// A table that holds no row writes nothing, so a world in which nobody
    /// advertised hashes as it did before the board existed.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        if self.rows.is_empty() {
            return hash;
        }
        hash.write(bytemuck::cast_slice(&self.rows))
            .write_u64(u64::from(self.factions))
            .write_u64(u64::from(self.bound))
    }

    /// Reports whether the table holds its invariants.
    ///
    /// The table is either empty or exactly one block for each faction, and
    /// every padding byte is zero.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.rows.is_empty() {
            return true;
        }
        self.rows.len() == self.factions as usize * self.bound as usize
            && self.rows.iter().all(|row| row.padding == 0)
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
    /// The tiles of each land side, two lists for each row.
    ///
    /// The list of the give side sits at twice the row index, and the list of
    /// the take side directly after it. A side that is not land holds an empty
    /// list. The list is the content of the row's consideration and it sits
    /// here because a row is plain data and a list has no fixed size.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/draft/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    land: Vec<Vec<TileIdx>>,
    /// How many factions the world holds.
    factions: u16,
}

impl TradeTable {
    /// Builds an empty plane for a world with this many factions.
    #[must_use]
    pub const fn new(factions: u16) -> Self {
        Self {
            rows: Vec::new(),
            land: Vec::new(),
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
            self.land = vec![Vec::new(); 2 * count];
        }
        self.rows.get_mut(index)
    }

    /// Returns the tiles of one side of one row. The give side is `false`
    /// and the take side is `true`.
    ///
    /// A side that is not land answers an empty slice.
    #[must_use]
    pub fn land_of(&self, index: usize, take_side: bool) -> &[TileIdx] {
        self.land
            .get(2 * index + usize::from(take_side))
            .map_or(&[], Vec::as_slice)
    }

    /// Replaces the tiles of one side of one row.
    ///
    /// The tiles are stored in ascending order, so a reader and the transfer
    /// both see one order.[^1] The plane must hold rows already.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn set_land(&mut self, index: usize, take_side: bool, mut tiles: Vec<TileIdx>) {
        tiles.sort_unstable();
        tiles.dedup();
        if let Some(slot) = self.land.get_mut(2 * index + usize::from(take_side)) {
            *slot = tiles;
        }
    }

    /// Empties both tile lists of one row.
    pub fn clear_land(&mut self, index: usize) {
        for side in [false, true] {
            if let Some(slot) = self.land.get_mut(2 * index + usize::from(side)) {
                slot.clear();
            }
        }
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
        let hash = hash
            .write(bytemuck::cast_slice(&self.rows))
            .write_u64(u64::from(self.factions));
        // The tiles of every land side follow, in row order and then in
        // ascending tile order. The count precedes each list, so two lists
        // that share a prefix cannot collide.
        self.land.iter().fold(hash, |hash, tiles| {
            hash.write_u64(tiles.len() as u64)
                .write(bytemuck::cast_slice(tiles))
        })
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
        if self.land.len() != 2 * self.rows.len() {
            return false;
        }
        self.rows.iter().enumerate().all(|(index, row)| {
            let give_land = self.land_of(index, false);
            let take_land = self.land_of(index, true);
            let give_fits = row.give_tag != KIND_LAND
                || !row.is_live()
                || give_land.len() == row.give_amount as usize;
            let take_fits = row.take_tag != KIND_LAND
                || !row.is_live()
                || take_land.len() == row.take_amount as usize;
            row.status < TRADE_STATUS_COUNT
                && row.given <= row.give_amount
                && row.taken <= row.take_amount
                && row.give_tag < CONSIDERATION_KIND_COUNT
                && row.take_tag < CONSIDERATION_KIND_COUNT
                && row.padding == [0; 6]
                && (row.give_tag == KIND_LAND || give_land.is_empty())
                && (row.take_tag == KIND_LAND || take_land.is_empty())
                && give_fits
                && take_fits
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
