//! The contracts two factions exchange, and the market they read.
//!
//! This module holds the verbs that offer, counter, accept, refuse, close and
//! reopen a contract, the readers of a contract and of the book, the
//! advertisement a faction posts, the market it reads, and the bounds and
//! schedules the trade pass runs on.
//!
//! The grouping is by the subject. Every method here addresses a contract, an
//! advertisement or the board that carries them.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use cachette_core::TileIdx;
use cachette_core::{Advert, Consideration, KIND_LAND, KIND_RELATION, KIND_RESOURCE};
use cachette_core::{Axial, FactionId};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Opens a trade negotiation from one faction toward another.
    ///
    /// A trade has two halves. This half is the conversation. The other half
    /// is the contract that an acceptance makes, and the engine enforces that
    /// one.
    ///
    /// The proposer and the responder are faction numbers. The pair is
    /// ordered, and the row the engine writes belongs to the proposer and the
    /// responder in that order.
    ///
    /// The give side is what the proposer owes. The take side is what the
    /// responder owes. Each is a whole quantity of one resource kind: food is
    /// zero, wood is one and stone is two. No term of a contract is a
    /// fractional number.
    ///
    /// The term is how many steps the contract runs for once it binds. The
    /// acceptance turns it into a deadline. A contract that cannot fail is not
    /// a contract, so a term of zero is refused.
    ///
    /// **One unit of the proposer must stand on ground that the responder
    /// holds.** This is the same rule that governs a message between two
    /// players. A trade is a thing two players say to each other.[^1]
    ///
    /// The call gives the offer. It moves nothing. Read `trade_status` for
    /// what the pair now holds.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when a kind names no resource, when a
    /// quantity is zero, when the term is zero, when the pair already holds a
    /// live negotiation or a live contract, when a terminal refusal closed
    /// this direction, or when no unit of the proposer stands on the
    /// responder's ground. The message says which, and a closure message
    /// states the step that opens the direction again.
    ///
    /// # References
    ///
    /// **Each side is a tagged consideration.** The tag is zero for a
    /// resource, one for land and two for a relation step. Both tags are zero
    /// when the call names none, so every call that states two resources
    /// keeps working.
    ///
    /// A land side names `give_cell` or `take_cell`, an address `(q, r)` whose
    /// level 1 cell is the set, or `give_tiles` or `take_tiles`, a list of
    /// addresses, or both. The kind and the amount of a land side are ignored,
    /// because the amount is the tile count. Every tile must be held by the
    /// party that owes it, and no tile may carry an upgrade while the question
    /// of what happens to the upgrade is open.[^2]
    ///
    /// A relation side keeps its kind and its amount. It is stored, and it
    /// delivers as a logged no-op until the relation matrix exists.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    /// [^2]: Blockers register, BLK-036. `docs/BLOCKERS.md`
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        proposer,
        responder,
        give_kind,
        give_amount,
        take_kind,
        take_amount,
        term,
        *,
        give_tag = 0,
        take_tag = 0,
        give_tiles = None,
        take_tiles = None,
        give_cell = None,
        take_cell = None,
    ))]
    fn offer_trade(
        &self,
        proposer: u16,
        responder: u16,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        term: u32,
        give_tag: u8,
        take_tag: u8,
        give_tiles: Option<Vec<(i32, i32)>>,
        take_tiles: Option<Vec<(i32, i32)>>,
        give_cell: Option<(i32, i32)>,
        take_cell: Option<(i32, i32)>,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let give = consideration_of(
            &world,
            give_tag,
            give_kind,
            give_amount,
            give_tiles,
            give_cell,
        )?;
        let take = consideration_of(
            &world,
            take_tag,
            take_kind,
            take_amount,
            take_tiles,
            take_cell,
        )?;
        world
            .offer_consideration(FactionId(proposer), FactionId(responder), give, take, term)
            .map_err(trade_refusal)
    }

    /// Restates the terms of a live negotiation.
    ///
    /// The speaker is the party that did not speak last. The `turn` entry of
    /// `trade_status` names the faction that may speak now.
    ///
    /// The terms are always stated in the orientation of the row. The give
    /// side is what the party that opened the pair owes, whoever speaks now.
    ///
    /// **One unit of the speaker must stand on ground that the other party
    /// holds.**
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, when a kind names no resource, when a
    /// quantity is zero, or when the speaker has no unit on the other party's
    /// ground.
    ///
    /// The tag and the target keyword arguments are those of `offer_trade`.
    /// A land side on the give side is checked against the party that opened
    /// the pair, and one on the take side against the other party.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        speaker,
        other,
        give_kind,
        give_amount,
        take_kind,
        take_amount,
        *,
        give_tag = 0,
        take_tag = 0,
        give_tiles = None,
        take_tiles = None,
        give_cell = None,
        take_cell = None,
    ))]
    fn counter_trade(
        &self,
        speaker: u16,
        other: u16,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        give_tag: u8,
        take_tag: u8,
        give_tiles: Option<Vec<(i32, i32)>>,
        take_tiles: Option<Vec<(i32, i32)>>,
        give_cell: Option<(i32, i32)>,
        take_cell: Option<(i32, i32)>,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let give = consideration_of(
            &world,
            give_tag,
            give_kind,
            give_amount,
            give_tiles,
            give_cell,
        )?;
        let take = consideration_of(
            &world,
            take_tag,
            take_kind,
            take_amount,
            take_tiles,
            take_cell,
        )?;
        world
            .counter_consideration(FactionId(speaker), FactionId(other), give, take)
            .map_err(trade_refusal)
    }

    /// Agrees to the terms of a live negotiation.
    ///
    /// The terms then bind both parties, and the engine enforces them. The
    /// deadline is the current step plus the term the offer named. A counter
    /// restates the quantities, and it never restates the term.
    ///
    /// The speaker is the party that did not speak last.
    ///
    /// **One unit of the speaker must stand on ground that the other party
    /// holds.**
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no unit on the
    /// other party's ground.
    fn accept_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .accept_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Declines the terms of a live negotiation.
    ///
    /// **This is a refusal and not a closed door.** The pair is idle after
    /// it, and either party may open a new negotiation at once. Call
    /// `close_trade` for the terminal refusal that stops the other party from
    /// asking again.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no unit on the
    /// other party's ground.
    fn refuse_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .refuse_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Declines the terms and stops the other party from asking again.
    ///
    /// **This is the terminal refusal, and it differs from `refuse_trade`.**
    /// It ends the negotiation, and it also closes the direction the other
    /// party would open, for the number of steps named here.
    ///
    /// The closure is directional. The other party cannot open a negotiation
    /// toward the speaker until the closure ends. The speaker may still open
    /// one toward the other party. The speaker closed its own door, and it
    /// promised no silence of its own.
    ///
    /// **The step that opens the direction again is readable.** Read
    /// `trade_status` with the other party first and the speaker second, and
    /// take the `closed_until` entry. An offer made before that step raises
    /// an error whose message states the step. A player that cannot tell a
    /// refusal from a closed door asks for ever.
    ///
    /// Only the speaker opens the direction early, through `reopen_trade`.
    /// Nothing the other party does shortens the closure.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the number of steps is zero,
    /// when the pair holds no live negotiation, when the terms already bind
    /// both parties, when the other party has not answered yet, or when the
    /// speaker has no unit on the other party's ground.
    fn close_trade(&self, speaker: u16, other: u16, steps: u32) -> PyResult<()> {
        let mut world = self.lock();
        world
            .close_trade(FactionId(speaker), FactionId(other), steps)
            .map_err(trade_refusal)
    }

    /// Opens a direction that this faction closed, before the closure ends.
    ///
    /// Only the faction that closed the direction opens it again.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, or when this faction closed nothing
    /// toward the other party.
    fn reopen_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .reopen_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Returns what stands between one ordered pair of factions.
    ///
    /// The pair is ordered. The row belongs to the proposer and the responder
    /// in that order, and it holds the negotiation that the proposer opened
    /// toward the responder. A pair that nobody ever spoke about answers an
    /// idle row. Every entry of it is zero. The one exception is
    /// `closed_until`, and it holds the step that opens a direction a
    /// terminal refusal closed.
    ///
    /// The dictionary holds these entries.
    ///
    /// - `status`, an integer. Zero is idle, one means the proposer spoke
    ///   last, two means the responder spoke last, three means a contract
    ///   binds both, four means both delivered in full, and five means the
    ///   deadline passed with a debt.
    /// - `turn`, an integer or `None`. Which faction answers next. It is
    ///   `None` when nobody is waiting on an answer.
    /// - `give_tag` and `take_tag`. The kind of each side: zero a resource,
    ///   one land, two a relation step.
    /// - `give_kind` and `give_amount`. What the proposer owes. For land the
    ///   amount is the tile count.
    /// - `take_kind` and `take_amount`. What the responder owes.
    /// - `give_tiles` and `take_tiles`, `numpy.uint32`. The tile indices of a
    ///   land side, ascending. Empty for the other kinds.
    /// - `given` and `taken`. What each party has already delivered.
    /// - `opened`, the step the negotiation opened at.
    /// - `deadline`, the step a bound contract fails at. Zero until it binds.
    /// - `term`, how many steps a bound contract runs for.
    /// - `closed_until`, the step at which this direction opens again. Zero
    ///   when nothing closed it. **This is how a caller tells a refusal from a
    ///   closed door.**
    /// - `rounds`, how many times somebody spoke.
    ///
    /// **The engine answers for any pair, and it holds no notion of who is
    /// asking.** A game that keeps a negotiation private between its two
    /// parties enforces that in the control plane.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when a number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D5. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    fn trade_status<'py>(
        &self,
        python: Python<'py>,
        proposer: u16,
        responder: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let row = world
            .trade_row(FactionId(proposer), FactionId(responder))
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "the pair {proposer} and {responder} names no faction of this world"
                ))
            })?;
        let turn: Option<u16> = match row.status {
            cachette_core::TRADE_OFFERED => Some(responder),
            cachette_core::TRADE_COUNTERED => Some(proposer),
            _ => None,
        };
        let entries = PyDict::new(python);
        entries.set_item("status", row.status)?;
        entries.set_item("turn", turn)?;
        entries.set_item("give_tag", row.give_tag)?;
        entries.set_item("take_tag", row.take_tag)?;
        let give_tiles: Vec<u32> = world
            .trade_land(FactionId(proposer), FactionId(responder), false)
            .iter()
            .map(|tile| tile.0)
            .collect();
        let take_tiles: Vec<u32> = world
            .trade_land(FactionId(proposer), FactionId(responder), true)
            .iter()
            .map(|tile| tile.0)
            .collect();
        entries.set_item("give_tiles", give_tiles.to_pyarray(python))?;
        entries.set_item("take_tiles", take_tiles.to_pyarray(python))?;
        entries.set_item("give_kind", row.give_kind)?;
        entries.set_item("give_amount", row.give_amount)?;
        entries.set_item("take_kind", row.take_kind)?;
        entries.set_item("take_amount", row.take_amount)?;
        entries.set_item("given", row.given)?;
        entries.set_item("taken", row.taken)?;
        entries.set_item("opened", row.opened.0)?;
        entries.set_item("deadline", row.deadline.0)?;
        entries.set_item("term", row.term)?;
        entries.set_item("closed_until", row.closed_until.0)?;
        entries.set_item("rounds", row.rounds)?;
        Ok(entries)
    }

    /// Returns every pair the faction forms with another faction, as columns.
    ///
    /// This is the read a player uses to decide. It crosses once and it holds
    /// no loop over pairs in Python. Every array has one entry for each pair
    /// the faction forms with another faction, and every array is the same
    /// length.
    ///
    /// - `proposer` and `responder`, `numpy.uint16`. The ordered pair.
    /// - `status`, `numpy.uint8`. The same numbering `trade_status` states.
    /// - `give_tag`, `take_tag`, `numpy.uint8`. The kind of each side.
    /// - `give_kind`, `take_kind`, `numpy.uint8`.
    /// - `give_amount`, `take_amount`, `given`, `taken`, `term`,
    ///   `numpy.uint32`.
    /// - `opened`, `deadline`, `closed_until`, `numpy.uint64`.
    /// - `rounds`, `numpy.uint8`.
    ///
    /// The book is empty until somebody speaks. A world in which nobody
    /// traded holds no row at all, so every array is empty.
    ///
    /// This method copies each column.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn trade_book<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if u32::from(faction) >= u32::from(world.faction_count()) {
            return Err(ViewError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let width = usize::from(world.faction_count());
        let rows = world.trade_book();
        let mut chosen: Vec<(u16, u16, cachette_core::TradeRow)> = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let proposer = (index / width.max(1)) as u16;
            let responder = (index % width.max(1)) as u16;
            if proposer != faction && responder != faction {
                continue;
            }
            if proposer == responder {
                continue;
            }
            chosen.push((proposer, responder, *row));
        }
        let columns = PyDict::new(python);
        let proposer: Vec<u16> = chosen.iter().map(|entry| entry.0).collect();
        let responder: Vec<u16> = chosen.iter().map(|entry| entry.1).collect();
        let status: Vec<u8> = chosen.iter().map(|entry| entry.2.status).collect();
        let give_tag: Vec<u8> = chosen.iter().map(|entry| entry.2.give_tag).collect();
        let take_tag: Vec<u8> = chosen.iter().map(|entry| entry.2.take_tag).collect();
        let give_kind: Vec<u8> = chosen.iter().map(|entry| entry.2.give_kind).collect();
        let take_kind: Vec<u8> = chosen.iter().map(|entry| entry.2.take_kind).collect();
        let give_amount: Vec<u32> = chosen.iter().map(|entry| entry.2.give_amount).collect();
        let take_amount: Vec<u32> = chosen.iter().map(|entry| entry.2.take_amount).collect();
        let given: Vec<u32> = chosen.iter().map(|entry| entry.2.given).collect();
        let taken: Vec<u32> = chosen.iter().map(|entry| entry.2.taken).collect();
        let term: Vec<u32> = chosen.iter().map(|entry| entry.2.term).collect();
        let opened: Vec<u64> = chosen.iter().map(|entry| entry.2.opened.0).collect();
        let deadline: Vec<u64> = chosen.iter().map(|entry| entry.2.deadline.0).collect();
        let closed_until: Vec<u64> = chosen.iter().map(|entry| entry.2.closed_until.0).collect();
        let rounds: Vec<u8> = chosen.iter().map(|entry| entry.2.rounds).collect();
        columns.set_item("proposer", proposer.to_pyarray(python))?;
        columns.set_item("responder", responder.to_pyarray(python))?;
        columns.set_item("status", status.to_pyarray(python))?;
        columns.set_item("give_tag", give_tag.to_pyarray(python))?;
        columns.set_item("take_tag", take_tag.to_pyarray(python))?;
        columns.set_item("give_kind", give_kind.to_pyarray(python))?;
        columns.set_item("take_kind", take_kind.to_pyarray(python))?;
        columns.set_item("give_amount", give_amount.to_pyarray(python))?;
        columns.set_item("take_amount", take_amount.to_pyarray(python))?;
        columns.set_item("given", given.to_pyarray(python))?;
        columns.set_item("taken", taken.to_pyarray(python))?;
        columns.set_item("term", term.to_pyarray(python))?;
        columns.set_item("opened", opened.to_pyarray(python))?;
        columns.set_item("deadline", deadline.to_pyarray(python))?;
        columns.set_item("closed_until", closed_until.to_pyarray(python))?;
        columns.set_item("rounds", rounds.to_pyarray(python))?;
        Ok(columns)
    }

    /// Replaces the whole board of one faction.
    ///
    /// A board says what a faction offers and what it wants. Each row is a
    /// tuple `(good, quantity, wants, asking_good, asking_quantity)`. The good
    /// and the asking good are resource kinds. `wants` is zero when the
    /// faction offers the good and one when it wants the good. The quantities
    /// are whole numbers.
    ///
    /// The call replaces every row the faction had. An empty list clears the
    /// board. Posting is a statement and not a speech act, so it passes no
    /// presence gate and costs no standing.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world, when
    /// the list holds more rows than `board_rows()`, when a good names no
    /// resource kind, or when `wants` is neither zero nor one. No row changes
    /// on an error.
    fn advertise(&self, faction: u16, rows: Vec<(u8, u32, u8, u8, u32)>) -> PyResult<()> {
        let mut world = self.lock();
        let rows: Vec<Advert> = rows
            .into_iter()
            .map(|(good, quantity, wants, asking_good, asking_quantity)| {
                Advert::new(good, quantity, wants, asking_good, asking_quantity)
            })
            .collect();
        world
            .advertise(FactionId(faction), &rows)
            .map_err(trade_refusal)
    }

    /// Returns the board of any faction, as columns.
    ///
    /// Every array has one entry for each row that says something, and every
    /// array is the same length. A faction that never advertised has an empty
    /// board, and so does a faction that posted an empty list.
    ///
    /// - `good`, `numpy.uint8`. The resource kind the row is about.
    /// - `quantity`, `numpy.uint32`. How much of it.
    /// - `wants`, `numpy.uint8`. Zero when the faction offers the good, one
    ///   when it wants the good.
    /// - `asking_good`, `numpy.uint8`. The resource kind asked in return.
    /// - `asking_quantity`, `numpy.uint32`. How much of that.
    ///
    /// Reading a board costs nothing and moves no relation. This method
    /// copies each column.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn market<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(ViewError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let rows: Vec<Advert> = world
            .market(FactionId(faction))
            .iter()
            .copied()
            .filter(|row| !row.is_empty())
            .collect();
        let columns = PyDict::new(python);
        let good: Vec<u8> = rows.iter().map(|row| row.good).collect();
        let quantity: Vec<u32> = rows.iter().map(|row| row.quantity).collect();
        let wants: Vec<u8> = rows.iter().map(|row| row.wants).collect();
        let asking_good: Vec<u8> = rows.iter().map(|row| row.asking_good).collect();
        let asking_quantity: Vec<u32> = rows.iter().map(|row| row.asking_quantity).collect();
        columns.set_item("good", good.to_pyarray(python))?;
        columns.set_item("quantity", quantity.to_pyarray(python))?;
        columns.set_item("wants", wants.to_pyarray(python))?;
        columns.set_item("asking_good", asking_good.to_pyarray(python))?;
        columns.set_item("asking_quantity", asking_quantity.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns how many rows one faction's board holds.
    ///
    /// The value is a balance parameter and the register calls it unset. The
    /// engine holds a stand-in until a caller sets one.
    fn board_rows(&self) -> u16 {
        self.lock().board_rows()
    }

    /// Sets how many rows one faction's board holds.
    ///
    /// The change empties every board, because the table is laid out by the
    /// bound.
    fn set_board_rows(&self, rows: u16) {
        self.lock().set_board_rows(rows);
    }

    /// Returns the most tiles one land side of a contract may name.
    ///
    /// The value is a balance parameter and the register calls it unset. The
    /// engine holds a stand-in until a caller sets one.
    fn land_list_bound(&self) -> u32 {
        self.lock().land_list_bound()
    }

    /// Sets the most tiles one land side of a contract may name.
    fn set_land_list_bound(&self, bound: u32) {
        self.lock().set_land_list_bound(bound);
    }

    /// How many ticks lie between two board writes, and the offset inside
    /// that period, as a tuple. The two values are a row of the balance
    /// register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the advertisement schedule. `docs/reference/balance.md`
    #[getter]
    fn advertisement_schedule(&self) -> (u32, u32) {
        self.lock().advertisement_schedule()
    }

    /// Sets how often the controller rewrites the board of a faction, and the
    /// offset inside the period. Returns `None`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that
    /// the scaling multiply takes.
    fn set_advertisement_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_advertisement_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// The store above which a faction offers a good, and below which it
    /// wants one, as an integer. The value is a row of the balance
    /// register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the surplus mark. `docs/reference/balance.md`
    #[getter]
    fn surplus_mark(&self) -> u32 {
        self.lock().surplus_mark()
    }

    /// Sets the surplus mark. Returns `None`.
    fn set_surplus_mark(&self, mark: u32) {
        self.lock().set_surplus_mark(mark);
    }

    /// How many carriers one faction assigns to one contract, as an integer.
    /// The value is a row of the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
    #[getter]
    fn carriers_per_contract(&self) -> u32 {
        self.lock().carriers_per_contract()
    }

    /// Sets how many carriers one faction assigns to one contract. A count of
    /// zero assigns none. Returns `None`.
    fn set_carriers_per_contract(&self, carriers: u32) {
        self.lock().set_carriers_per_contract(carriers);
    }

    /// How many ticks a contract that the controller opens runs for, as an
    /// integer. The value is a row of the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the contract term. `docs/reference/balance.md`
    #[getter]
    fn contract_term(&self) -> u32 {
        self.lock().contract_term()
    }

    /// Sets how many ticks a contract that the controller opens runs for.
    /// Returns `None`.
    fn set_contract_term(&self, term: u32) {
        self.lock().set_contract_term(term);
    }
}

/// Turns a refusal of a trade verb into the exception a caller catches.
///
/// The message states which rule refused. A closure states the step that
/// opens the direction again, because a player that cannot tell a refusal
/// from a closed door asks for ever.[^1]
///
/// # References
///
/// [^1]: ADR-0127, a terminal refusal closes an ordered pair until a named tick, decision D3. `docs/adrs/draft/adr-0127-a-terminal-refusal-closes-a-pair-until-a-named-tick.md`
/// Builds one side of a contract from the keyword arguments of a trade verb.
///
/// A land side takes its tiles from the cell, from the list, or from both.
/// The other kinds keep the kind and the amount and hold no tile.
fn consideration_of(
    world: &cachette_core::World,
    tag: u8,
    kind: u8,
    amount: u32,
    tiles: Option<Vec<(i32, i32)>>,
    cell: Option<(i32, i32)>,
) -> PyResult<Consideration> {
    match tag {
        KIND_RESOURCE => Ok(Consideration::resource(kind, amount)),
        KIND_RELATION => Ok(Consideration::relation(kind, amount)),
        KIND_LAND => {
            let mut set: Vec<TileIdx> = Vec::new();
            if let Some((q, r)) = cell {
                let found = world.cell_tiles(Axial::new(q, r)).map_err(|_| {
                    VerbError::new_err(format!(
                        "the address ({q}, {r}) lies outside the world, so it names no cell"
                    ))
                })?;
                set.extend(found);
            }
            for (q, r) in tiles.unwrap_or_default() {
                let tile = world.grid().index_of(Axial::new(q, r)).ok_or_else(|| {
                    VerbError::new_err(format!("the address ({q}, {r}) lies outside the world"))
                })?;
                set.push(tile);
            }
            if set.is_empty() {
                return Err(VerbError::new_err(
                    "a land side names a cell, a list of tiles, or both",
                ));
            }
            Ok(Consideration::land(set))
        }
        other => Err(VerbError::new_err(format!(
            "{other} names no consideration kind: zero is a resource, one is land and two is a relation step"
        ))),
    }
}

fn trade_refusal(error: cachette_core::TradeError) -> PyErr {
    use cachette_core::TradeError as Refusal;
    let said = match error {
        Refusal::SameFaction(faction) => {
            format!("the faction {} does not trade with itself", faction.0)
        }
        Refusal::NoSuchFaction(faction) => {
            format!("{} names no faction of this world", faction.0)
        }
        Refusal::NoSuchKind(kind) => format!("{kind} names no resource kind"),
        Refusal::EmptyTerms => "each side of a contract binds a quantity above zero".to_string(),
        Refusal::NoDeadline => "a contract runs for a term above zero, because a contract that cannot fail is not a contract".to_string(),
        Refusal::AlreadyOpen => {
            "the two parties already hold a live negotiation or a live contract".to_string()
        }
        Refusal::NothingOpen => "the two parties hold no live negotiation".to_string(),
        Refusal::NotYourTurn => {
            "the other party has not answered yet, so this party may not speak again".to_string()
        }
        Refusal::AlreadyBound => {
            "the terms bind both parties, so nobody restates them and nobody refuses them"
                .to_string()
        }
        Refusal::Closed(until) => format!(
            "a terminal refusal closed this direction, and it opens again at step {}",
            until.0
        ),
        Refusal::NothingClosed => "this party closed nothing toward the other party".to_string(),
        Refusal::NoPresence => {
            "no unit of the speaker stands on ground that the other party holds".to_string()
        }
        Refusal::AtWar => {
            "one of the pair is in the war band toward the other, so no offer opens".to_string()
        }
        Refusal::NoDuration => {
            "a terminal refusal closes the direction for a number of steps above zero".to_string()
        }
        Refusal::NoSuchTag(tag) => format!(
            "{tag} names no consideration kind: zero is a resource, one is land and two is a relation step"
        ),
        Refusal::NoSuchTile => "a land side names a tile that lies outside the world".to_string(),
        Refusal::NoSuchCell => "a land side names a cell that lies outside the world".to_string(),
        Refusal::LandNotHeld(tile) => format!(
            "the party that owes the land does not hold tile {}",
            tile.0
        ),
        Refusal::TooMuchLand(count, bound) => format!(
            "a land side names {count} tiles, and the bound is {bound}"
        ),
        Refusal::NoSuchSide(wants) => format!(
            "{wants} names neither offers, which is zero, nor wants, which is one"
        ),
        Refusal::BoardOverfull(count, bound) => format!(
            "the board holds {bound} rows, and the write names {count}"
        ),
    };
    VerbError::new_err(said)
}
