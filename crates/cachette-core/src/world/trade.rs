//! A trade between two factions, from the offer to the settled contract.
//!
//! A trade is spoken in turns. One faction offers, the other counters,
//! accepts or refuses, and a settled trade becomes a contract that carriers
//! deliver. The whole exchange sits in one module, because each act is legal
//! only against the state the previous act left.

use super::errors::StepError;
use super::gather::gather_order_of;
use super::World;
use crate::hex::Axial;
use crate::holding::Holder;
use crate::position::WORK_COMMODITY;
use crate::resource::{Amount, CarryLoad, ResourceKind};
use crate::sim_math;
use crate::sort::BoundedKey;
use crate::trade::{
    self, Advert, Consideration, TradeError, TradeRow, TradeSpoken, ACT_ACCEPT, ACT_CLOSE,
    ACT_COUNTER, ACT_DEFAULT, ACT_OFFER, ACT_REFUSE, ACT_REOPEN, ACT_SETTLE, ACT_STEP_RELATION,
    ACT_TRANSFER_LAND, KIND_LAND, KIND_RELATION, KIND_RESOURCE, TRADE_BOUND, TRADE_COUNTERED,
    TRADE_DEFAULTED, TRADE_IDLE, TRADE_OFFERED, TRADE_SETTLED,
};
use crate::types::{Entity, FactionId, Fix32, Tick, TileIdx};

/// Returns the tile one step away, or `None` when nothing may stand there.
///
/// The call answers two refusals with one value, because a caller that wanted
/// to tell them apart would have to ask the grid and the terrain separately
/// and would then hold the rule twice.[^1]
///
/// A neighbour outside the world is no tile. The world is a rhombus and it
/// does not wrap.[^2]
///
/// Ground that admits no unit is no target. The capacity table is the one
/// statement of which ground admits a unit.[^3]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
/// [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// One delivery that a bound contract admits.
///
/// The record is built in unit slot order and then sorted on a total key
/// before anything moves, so nothing downstream reads the order it was
/// collected in.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Copy, Debug)]
pub(super) struct ContractDelivery {
    /// The unit that carries the load.
    pub(super) unit: Entity,
    /// The slot of the settlement that receives it.
    pub(super) site: u32,
    /// The kind of resource that the contract names for this party.
    pub(super) kind: ResourceKind,
    /// The index of the row in the negotiation plane.
    pub(super) row: usize,
    /// Whether this party is the one that opened the pair.
    pub(super) owes_as_proposer: bool,
}

impl World {
    /// Returns the negotiation and the contract between one ordered pair.
    ///
    /// The pair is ordered. The row for the proposer and the responder, in
    /// that order, holds the negotiation that the proposer opened toward the
    /// responder. A pair that nobody ever spoke about answers an idle row.
    ///
    /// Returns `None` when either identifier is at or above the faction count
    /// of this world.
    #[must_use]
    pub fn trade_row(&self, proposer: FactionId, responder: FactionId) -> Option<TradeRow> {
        self.trade.row(proposer, responder)
    }

    /// Returns every row of the negotiation plane, in pair order.
    ///
    /// The slice is empty until somebody speaks. The index of a pair is the
    /// proposer times the faction count plus the responder.
    #[must_use]
    pub fn trade_book(&self) -> &[TradeRow] {
        self.trade.rows()
    }

    /// Returns what the last step said about trade.
    ///
    /// The log holds one entry for each speech act and for each settlement or
    /// default that the step resolved. A control plane reads it at the frame
    /// barrier and never inside a step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn trade_log(&self) -> &[TradeSpoken] {
        &self.trade_log
    }

    /// Returns the trade log as raw bytes.
    ///
    /// The event type is plain data with declared padding, so the bytes are
    /// the log and nothing else.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn trade_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.trade_log)
    }

    /// Refuses a faction identifier that this world does not hold.
    pub(super) fn check_faction(&self, faction: FactionId) -> Result<(), TradeError> {
        if faction.0 >= self.config.faction_count {
            return Err(TradeError::NoSuchFaction(faction));
        }
        Ok(())
    }

    /// Refuses a pair that this world cannot hold a negotiation for.
    fn check_pair(&self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_faction(speaker)?;
        self.check_faction(other)?;
        if speaker == other {
            return Err(TradeError::SameFaction(speaker));
        }
        Ok(())
    }

    /// Returns the orientation of the live row of an unordered pair.
    ///
    /// **One unordered pair holds at most one live negotiation.** Two players
    /// discuss one thing at a time, so the answer names the row rather than
    /// leaving the caller to guess which of the two orientations is live.
    pub(super) fn live_orientation(
        &self,
        speaker: FactionId,
        other: FactionId,
    ) -> Result<(FactionId, FactionId), TradeError> {
        if self
            .trade
            .row(speaker, other)
            .is_some_and(|row| row.is_live())
        {
            return Ok((speaker, other));
        }
        if self
            .trade
            .row(other, speaker)
            .is_some_and(|row| row.is_live())
        {
            return Ok((other, speaker));
        }
        Err(TradeError::NothingOpen)
    }

    /// Returns the faction whose turn it is to answer a live row.
    pub(super) fn turn_of(
        row: TradeRow,
        proposer: FactionId,
        responder: FactionId,
    ) -> Option<FactionId> {
        match row.status {
            TRADE_OFFERED => Some(responder),
            TRADE_COUNTERED => Some(proposer),
            _ => None,
        }
    }

    /// Writes one entry into the trade log.
    fn say(&mut self, proposer: FactionId, responder: FactionId, act: u8, status: u8) {
        self.trade_log.push(TradeSpoken::new(
            self.tick, proposer, responder, act, status,
        ));
    }

    /// Opens a negotiation from one faction toward another.
    ///
    /// The terms bind both parties. The give side is what the proposer owes
    /// and the take side is what the responder owes. Each is a whole quantity
    /// of one resource kind, so no term of a contract is a floating point
    /// number.[^1]
    ///
    /// The term is how many ticks the contract runs for once it binds. The
    /// acceptance turns it into a deadline. A contract that cannot fail is not
    /// a contract, so a term of zero is refused.
    ///
    /// **The offer passes the presence gate.** A unit of the proposer must
    /// stand on ground that the responder holds.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when a party names no faction of this world, when the
    /// two parties are one faction, when either kind names no resource, when
    /// either quantity is zero, when the term is zero, when the unordered pair
    /// already holds a live negotiation, when a terminal refusal closed this
    /// direction, or when the proposer has no presence.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    // The terms of a contract are six values and the parties are two more.
    // A structure that held them would be a second name for the row this
    // function writes, and the control plane would then state the terms twice:
    // once to build it and once to read the row back.
    #[allow(clippy::too_many_arguments)]
    pub fn offer_trade(
        &mut self,
        proposer: FactionId,
        responder: FactionId,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        term: u32,
    ) -> Result<(), TradeError> {
        self.offer_consideration(
            proposer,
            responder,
            Consideration::resource(give_kind, give_amount),
            Consideration::resource(take_kind, take_amount),
            term,
        )
    }

    /// Opens a negotiation from one faction toward another, with a tagged
    /// consideration on each side.
    ///
    /// **Each side is one tag and the content the tag names.**[^1] A resource
    /// side is a kind and a quantity, and a unit carries it. A land side is a
    /// list of tiles that the debtor holds, and the holder changes when the
    /// other side is delivered in full. A relation side is stored and applies
    /// as a logged no-op until the relation matrix exists.
    ///
    /// The give side is what the proposer owes and the take side is what the
    /// responder owes. A land side is checked against its debtor: every tile
    /// of the give side must be held by the proposer, and every tile of the
    /// take side by the responder.[^2] A land side whose tiles carry an
    /// upgrade is refused while the question of what happens to the upgrade
    /// is open.[^3]
    ///
    /// # Errors
    ///
    /// Returns every error the resource verb returns, and also an error when
    /// a tag names no kind, when a land side is empty or names more tiles than
    /// the bound, when a tile lies outside the world, when the debtor does not
    /// hold a tile, or when a tile carries an upgrade.
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^2]: ADR-0147, a contract consideration is a tagged kind, decision D4. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^3]: Blockers register, BLK-036. `docs/BLOCKERS.md`
    pub fn offer_consideration(
        &mut self,
        proposer: FactionId,
        responder: FactionId,
        give: Consideration,
        take: Consideration,
        term: u32,
    ) -> Result<(), TradeError> {
        self.check_pair(proposer, responder)?;
        // An offer across a pair at war is refused before anything else is
        // read. The predicate is the relation module's, so the trade verbs
        // and the contest read one statement of the war band.[^war]
        //
        // [^war]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        if !self.relations.permits_offer(proposer, responder) {
            return Err(TradeError::AtWar);
        }
        let give = self.check_consideration(proposer, give)?;
        let take = self.check_consideration(responder, take)?;
        if term == 0 {
            return Err(TradeError::NoDeadline);
        }
        if self.live_orientation(proposer, responder).is_ok() {
            return Err(TradeError::AlreadyOpen);
        }
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        if row.closed_until.0 > self.tick.0 {
            return Err(TradeError::Closed(row.closed_until));
        }
        if !self.stands_in_territory_of(proposer, responder) {
            return Err(TradeError::NoPresence);
        }
        let tick = self.tick;
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NoSuchFaction(proposer))?;
        entry.clear();
        entry.opened = tick;
        entry.give_tag = give.tag;
        entry.give_kind = give.kind;
        entry.give_amount = give.amount;
        entry.take_tag = take.tag;
        entry.take_kind = take.kind;
        entry.take_amount = take.amount;
        entry.term = term;
        entry.status = TRADE_OFFERED;
        entry.rounds = 1;
        self.trade.set_land(index, false, give.tiles);
        self.trade.set_land(index, true, take.tiles);
        self.say(proposer, responder, ACT_OFFER, TRADE_OFFERED);
        Ok(())
    }

    /// Checks one side of a contract against its debtor and returns it in
    /// the form the plane stores.
    ///
    /// A land side comes back sorted and without a repeated tile, and its
    /// amount is the tile count. The other kinds come back as given.
    fn check_consideration(
        &self,
        debtor: FactionId,
        mut side: Consideration,
    ) -> Result<Consideration, TradeError> {
        match side.tag {
            KIND_RESOURCE => {
                trade::kind_of(side.kind)?;
                if side.amount == 0 {
                    return Err(TradeError::EmptyTerms);
                }
                side.tiles.clear();
            }
            KIND_LAND => {
                side.tiles.sort_unstable();
                side.tiles.dedup();
                if side.tiles.is_empty() {
                    return Err(TradeError::EmptyTerms);
                }
                let count = u32::try_from(side.tiles.len()).unwrap_or(u32::MAX);
                if count > self.land_list_bound {
                    return Err(TradeError::TooMuchLand(count, self.land_list_bound));
                }
                let holders = self.holding.holders();
                let wanted = Holder::of(debtor);
                for tile in &side.tiles {
                    let Some(holder) = holders.get(tile.0 as usize) else {
                        return Err(TradeError::NoSuchTile);
                    };
                    if *holder != wanted {
                        return Err(TradeError::LandNotHeld(*tile));
                    }
                    // **An upgrade changes hands with the ground**, so a
                    // land side that carries one needs no refusal and no
                    // arithmetic. The upgrade is stored against the tile and
                    // no owner stands beside it, so the ground carries it.[^32]
                    //
                    // [^32]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
                }
                side.kind = 0;
                side.amount = count;
            }
            KIND_RELATION => {
                if side.amount == 0 {
                    return Err(TradeError::EmptyTerms);
                }
                side.tiles.clear();
            }
            other => return Err(TradeError::NoSuchTag(other)),
        }
        Ok(side)
    }

    /// Returns every tile of the level 1 cell that covers one address, in
    /// ascending tile index.
    ///
    /// A cell on the world edge is partial, and the call returns the tiles
    /// that exist. This is the set a land side names when it names a cell.
    ///
    /// # Errors
    ///
    /// Returns an error when the address lies outside the world.
    pub fn cell_tiles(&self, address: Axial) -> Result<Vec<TileIdx>, TradeError> {
        let layout = self.pyramid.layout();
        let tile = self.grid.index_of(address).ok_or(TradeError::NoSuchCell)?;
        let key = layout.key_of(tile).ok_or(TradeError::NoSuchCell)?;
        let block = layout.block_of_key(key);
        let edge = layout.block_edge();
        let across = block % layout.blocks_wide();
        let down = block / layout.blocks_wide();
        let first_q = across * edge;
        let last_q = (first_q + edge).min(self.grid.width());
        let first_r = down * edge;
        let last_r = (first_r + edge).min(self.grid.height());
        let mut tiles = Vec::with_capacity((edge * edge) as usize);
        for r in first_r..last_r {
            for q in first_q..last_q {
                let here = Axial::new(
                    i32::try_from(q).map_err(|_| TradeError::NoSuchCell)?,
                    i32::try_from(r).map_err(|_| TradeError::NoSuchCell)?,
                );
                tiles.push(self.grid.index_of(here).ok_or(TradeError::NoSuchCell)?);
            }
        }
        Ok(tiles)
    }

    /// Returns the tiles of one side of one ordered pair. The give side is
    /// `false` and the take side is `true`.
    ///
    /// A side that is not land answers an empty slice, and so does a pair
    /// that names no faction.
    #[must_use]
    pub fn trade_land(
        &self,
        proposer: FactionId,
        responder: FactionId,
        take_side: bool,
    ) -> &[TileIdx] {
        match self.trade.index_of(proposer, responder) {
            Some(index) => self.trade.land_of(index, take_side),
            None => &[],
        }
    }

    /// Returns the most tiles one land consideration may name.
    #[must_use]
    pub const fn land_list_bound(&self) -> u32 {
        self.land_list_bound
    }

    /// Sets the most tiles one land consideration may name.
    ///
    /// The bound is a balance value and not a budget.[^1] A bound of zero
    /// refuses every land side.
    ///
    /// # References
    ///
    /// [^1]: Balance register, land list bound. `docs/reference/balance.md`
    pub fn set_land_list_bound(&mut self, bound: u32) {
        self.land_list_bound = bound;
    }

    /// Returns how many advertisement rows one faction's board holds.
    #[must_use]
    pub const fn board_rows(&self) -> u16 {
        self.market.bound()
    }

    /// Sets how many advertisement rows one faction's board holds.
    ///
    /// The row count is a balance value and not a budget.[^1] A change
    /// empties every board, because the table is laid out by the bound.
    ///
    /// # References
    ///
    /// [^1]: Balance register, board size. `docs/reference/balance.md`
    pub fn set_board_rows(&mut self, rows: u16) {
        self.market.set_bound(rows);
    }

    /// Replaces the whole board of one faction.
    ///
    /// A board says what a faction offers and wants. It is a statement and
    /// not a speech act, so it passes no presence gate and costs no standing.
    /// A reader of another faction's board learns what that faction posted
    /// and nothing else.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction names no faction of this world, when
    /// the rows outnumber the bound, when a good names no resource kind, or
    /// when a row names neither offers nor wants. No row changes on an error.
    pub fn advertise(&mut self, faction: FactionId, rows: &[Advert]) -> Result<(), TradeError> {
        self.check_faction(faction)?;
        self.market.advertise(faction, rows)
    }

    /// Returns the board of one faction, empty rows included.
    ///
    /// The slice is empty for a faction that never advertised and for a
    /// number that names no faction.
    #[must_use]
    pub fn market(&self, faction: FactionId) -> &[Advert] {
        self.market.board(faction)
    }

    /// Restates the terms of a live negotiation.
    ///
    /// The speaker is the party that did not speak last. The terms are always
    /// stated in the orientation of the row, so the give side is what the
    /// party that opened the pair owes, whoever is speaking now.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, when either kind names no resource, when either quantity is zero,
    /// or when the speaker has no presence.
    pub fn counter_trade(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
    ) -> Result<(), TradeError> {
        self.counter_consideration(
            speaker,
            other,
            Consideration::resource(give_kind, give_amount),
            Consideration::resource(take_kind, take_amount),
        )
    }

    /// Restates the terms of a live negotiation, with a tagged consideration
    /// on each side.
    ///
    /// The terms are stated in the orientation of the row, whoever speaks.
    /// The give side is what the party that opened the pair owes, and a land
    /// side is checked against that party. The take side is what the other
    /// party owes, and a land side is checked against it.
    ///
    /// # Errors
    ///
    /// Returns every error the resource verb returns, and every error the
    /// tagged offer returns for a side.
    pub fn counter_consideration(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        give: Consideration,
        take: Consideration,
    ) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        // A counter across a pair at war is refused, as an offer is. A war
        // declared during a negotiation therefore ends the talking.
        if !self.relations.permits_offer(speaker, other) {
            return Err(TradeError::AtWar);
        }
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let give = self.check_consideration(proposer, give)?;
        let take = self.check_consideration(responder, take)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let status = if row.status == TRADE_OFFERED {
            TRADE_COUNTERED
        } else {
            TRADE_OFFERED
        };
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.give_tag = give.tag;
        entry.give_kind = give.kind;
        entry.give_amount = give.amount;
        entry.take_tag = take.tag;
        entry.take_kind = take.kind;
        entry.take_amount = take.amount;
        entry.status = status;
        entry.rounds = entry.rounds.saturating_add(1);
        self.trade.set_land(index, false, give.tiles);
        self.trade.set_land(index, true, take.tiles);
        self.say(proposer, responder, ACT_COUNTER, status);
        Ok(())
    }

    /// Agrees to the terms of a live negotiation, so a contract binds both.
    ///
    /// The speaker is the party that did not speak last. The deadline is this
    /// tick plus the term the offer named.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, or when the speaker has no presence.
    pub fn accept_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let deadline = Tick(self.tick.0.saturating_add(u64::from(row.term)));
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.status = TRADE_BOUND;
        entry.deadline = deadline;
        entry.rounds = entry.rounds.saturating_add(1);
        self.say(proposer, responder, ACT_ACCEPT, TRADE_BOUND);
        Ok(())
    }

    /// Declines the terms of a live negotiation.
    ///
    /// **This is a refusal and not a closed door.** The pair is idle after it,
    /// and either party may open a new negotiation on the next call. A player
    /// that wants the other to stop asking calls the closing verb instead.
    ///
    /// # Errors
    ///
    /// Returns an error when the pair holds no live negotiation, when the
    /// terms already bind both parties, when the other party has not answered
    /// yet, or when the speaker has no presence.
    pub fn refuse_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.clear();
        self.trade.clear_land(index);
        self.say(proposer, responder, ACT_REFUSE, TRADE_IDLE);
        Ok(())
    }

    /// Declines the terms and closes the direction for a stated number of
    /// ticks.
    ///
    /// **This is the terminal refusal.** It ends the negotiation, and it also
    /// stops the other party from opening a new one toward the speaker until
    /// the tick it names. The closure is directional: the speaker may still
    /// open a negotiation toward the other party, because the speaker closed
    /// its own door and promised no silence of its own.
    ///
    /// The tick that opens the direction again is readable. A caller reads the
    /// closure from the row for the other party and the speaker, in that
    /// order, which is the row the other party would open. A player that
    /// cannot tell a refusal from a closed door asks for ever.
    ///
    /// Only the speaker opens the direction early, through the opening verb.
    /// Nothing the other party does shortens the closure. That is what makes
    /// it terminal.
    ///
    /// # Errors
    ///
    /// Returns an error when the duration is zero, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no presence.
    pub fn close_trade(
        &mut self,
        speaker: FactionId,
        other: FactionId,
        ticks: u32,
    ) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        if ticks == 0 {
            return Err(TradeError::NoDuration);
        }
        let (proposer, responder) = self.live_orientation(speaker, other)?;
        let row = self
            .trade
            .row(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        if row.is_bound() {
            return Err(TradeError::AlreadyBound);
        }
        if Self::turn_of(row, proposer, responder) != Some(speaker) {
            return Err(TradeError::NotYourTurn);
        }
        if !self.stands_in_territory_of(speaker, other) {
            return Err(TradeError::NoPresence);
        }
        let until = Tick(self.tick.0.saturating_add(u64::from(ticks)));
        let index = self
            .trade
            .index_of(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        let entry = self
            .trade
            .row_mut(proposer, responder)
            .ok_or(TradeError::NothingOpen)?;
        entry.clear();
        self.trade.clear_land(index);
        // The closure sits on the row the other party would open, which is
        // the pair with the other party first. Writing it on the live row
        // would close the speaker's own door and leave the other party free
        // to ask again, which is the opposite of what the verb promises.
        let door = self
            .trade
            .row_mut(other, speaker)
            .ok_or(TradeError::NothingOpen)?;
        door.closed_until = until;
        self.say(proposer, responder, ACT_CLOSE, TRADE_IDLE);
        Ok(())
    }

    /// Opens a direction that this faction closed, before the closure ends.
    ///
    /// Only the faction that closed the direction opens it again.
    ///
    /// # Errors
    ///
    /// Returns an error when the speaker closed nothing toward this party.
    pub fn reopen_trade(&mut self, speaker: FactionId, other: FactionId) -> Result<(), TradeError> {
        self.check_pair(speaker, other)?;
        let row = self
            .trade
            .row(other, speaker)
            .ok_or(TradeError::NoSuchFaction(other))?;
        if row.closed_until.0 <= self.tick.0 {
            return Err(TradeError::NothingClosed);
        }
        let door = self
            .trade
            .row_mut(other, speaker)
            .ok_or(TradeError::NoSuchFaction(other))?;
        door.closed_until = Tick(0);
        self.say(other, speaker, ACT_REOPEN, row.status);
        Ok(())
    }

    /// Moves the load of every unit that stands on the site of a faction that
    /// a contract obliges it to deliver to, and then fails every contract that
    /// reached its deadline with a debt.
    ///
    /// **A contract moves nothing on its own.** A quantity reaches the other
    /// party because a unit carried it onto the tile of a settlement that
    /// party holds.[^1] The engine already moves a load this way when a unit
    /// stands on its own site, and this pass is the same transfer against
    /// another faction's site.
    ///
    /// **A delivery is admitted by sort, then by transfer.** Two units of one
    /// faction may deliver into one store, and a store saturates at its
    /// ceiling, so a saturating add is not order-free.[^2] The pass orders the
    /// deliveries by the site and then by the identity of the unit, which is
    /// the order the ordinary delivery already uses.[^3] [^4]
    ///
    /// **A load the store cannot hold stays in the carry.** A quantity that
    /// vanished without a record would break the conservation equality.[^2]
    ///
    /// **A delivery never passes the debt.** The transfer takes the smallest
    /// of what the unit carries, what the party owes, and what the store can
    /// hold. A contract therefore moves the quantity it named and no more.
    ///
    /// **The deadline is checked after the delivery of this tick.** A contract
    /// whose deadline is this tick gets this tick's delivery, and it fails
    /// only when a debt survives it.
    ///
    /// **A default costs the defaulting party the direction it would ask on
    /// again, for as long as the contract ran.** The duration is the term of
    /// the contract itself, so no balance figure decides it. The quantities
    /// that already moved stay where they arrived, because taking them back
    /// would need a transfer that no unit carried.[^1]
    ///
    /// The pass runs on the calling thread. It writes one store at a time in a
    /// stated order, so it names no thread and depends on no thread
    /// count.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the ordering refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decisions D1 and D4. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0004, iteration order is explicit, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn settle_trades(&mut self, threads: usize) -> Result<(), StepError> {
        if self.trade.is_empty() {
            return Ok(());
        }
        self.carry_contract_loads(threads)?;
        self.apply_priced_sides(threads);
        self.fail_overdue_contracts();
        Ok(())
    }

    /// Applies every land set and every relation step whose price has
    /// arrived.
    ///
    /// **A side that no unit carries applies when the other side is delivered
    /// in full.**[^1] A land set applies by giving every tile in it to the
    /// creditor, in ascending tile index. A relation step applies as a logged
    /// no-op, because the relation matrix arrives with a later pass; the log
    /// entry is the record that the side was delivered, and nothing else
    /// moves. When neither side is carried, both apply on the first pass
    /// after the contract binds.
    ///
    /// The walk is over the plane in pair order, after the carriers of this
    /// tick have delivered and before the deadline check, so a contract whose
    /// resource side settles this tick delivers its land this tick. The order
    /// between two land sets that name one tile is the pair order, and no
    /// thread and no hash order enters it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn apply_priced_sides(&mut self, threads: usize) {
        let count = self.trade.rows().len();
        for index in 0..count {
            let Some(row) = self.trade.row_at(index) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let (proposer, responder) = self.pair_of(index);
            let responder_done = row.owed_by_responder() == 0 || !row.responder_side_is_carried();
            let proposer_done = row.owed_by_proposer() == 0 || !row.proposer_side_is_carried();
            let mut applied = false;
            if !row.proposer_side_is_carried() && row.owed_by_proposer() > 0 && responder_done {
                self.apply_side(index, false, responder, row.give_tag, threads);
                applied = true;
            }
            if !row.responder_side_is_carried() && row.owed_by_responder() > 0 && proposer_done {
                self.apply_side(index, true, proposer, row.take_tag, threads);
                applied = true;
            }
            if !applied {
                continue;
            }
            let paid = self.trade.row_at_mut(index).is_some_and(|entry| {
                if entry.is_paid() {
                    entry.status = TRADE_SETTLED;
                    true
                } else {
                    false
                }
            });
            if paid {
                self.say(proposer, responder, ACT_SETTLE, TRADE_SETTLED);
                // A contract delivered in full warms both directions, from
                // this settle site as from the carried one.
                self.relations
                    .on_contract_delivered(self.tick, proposer, responder);
            }
        }
    }

    /// Applies one side that no unit carries, and marks it delivered.
    fn apply_side(
        &mut self,
        index: usize,
        take_side: bool,
        creditor: FactionId,
        tag: u8,
        threads: usize,
    ) {
        let act = match tag {
            KIND_LAND => {
                let tiles = self.trade.land_of(index, take_side).to_vec();
                self.holding.transfer(&tiles, Holder::of(creditor), threads);
                ACT_TRANSFER_LAND
            }
            KIND_RELATION => {
                // The relation matrix does not exist yet. The step is stored
                // and logged, and it moves nothing. A later pass replaces this
                // arm with the move, and until then this side is inert on
                // purpose.
                ACT_STEP_RELATION
            }
            _ => return,
        };
        if let Some(entry) = self.trade.row_at_mut(index) {
            if take_side {
                entry.taken = entry.take_amount;
            } else {
                entry.given = entry.give_amount;
            }
        }
        let (proposer, responder) = self.pair_of(index);
        self.say(proposer, responder, act, TRADE_BOUND);
    }

    /// Returns every delivery a bound contract admits this tick, in slot
    /// order.
    ///
    /// The walk is over the unit slots and it reads no derived structure. The
    /// order is the slot order, which does not depend on the thread count. The
    /// caller sorts it on a total key before it transfers anything.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn contract_carriers(&self, threads: usize) -> Vec<ContractDelivery> {
        let _ = threads;
        let mut found = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            if load == CarryLoad::EMPTY {
                continue;
            }
            let Some(faction) = self.soldiers.faction(unit) else {
                continue;
            };
            let Some(address) = self.soldiers.address(unit) else {
                continue;
            };
            let Some(site) = self.settlements.on_tile(address) else {
                continue;
            };
            let Some(host) = self.settlements.faction(site) else {
                continue;
            };
            if host == faction {
                continue;
            }
            let Some(slot) = self.settlements.slot_of(site) else {
                continue;
            };
            let Ok((proposer, responder)) = self.live_orientation(faction, host) else {
                continue;
            };
            let (Some(index), Some(row)) = (
                self.trade.index_of(proposer, responder),
                self.trade.row(proposer, responder),
            ) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let owes_as_proposer = faction == proposer;
            // Only a resource is carried. A land set and a relation step
            // apply in the pass that follows the carriers, and a load
            // delivered against either would pay a debt that no unit can
            // pay.
            let carried = if owes_as_proposer {
                row.proposer_side_is_carried()
            } else {
                row.responder_side_is_carried()
            };
            if !carried {
                continue;
            }
            let (kind, owed) = if owes_as_proposer {
                (row.give_kind, row.owed_by_proposer())
            } else {
                (row.take_kind, row.owed_by_responder())
            };
            if owed == 0 {
                continue;
            }
            let Some(kind) = ResourceKind::from_u8(kind) else {
                continue;
            };
            if load.of(kind).0 == 0 {
                continue;
            }
            found.push(ContractDelivery {
                unit,
                site: slot,
                kind,
                row: index,
                owes_as_proposer,
            });
        }
        found
    }

    /// Transfers what every contract carrier may deliver this tick.
    fn carry_contract_loads(&mut self, threads: usize) -> Result<(), StepError> {
        let carriers = self.contract_carriers(threads);
        if carriers.is_empty() {
            return Ok(());
        }
        let keys: Vec<BoundedKey> = carriers
            .iter()
            .map(|delivery| BoundedKey::new(u64::from(delivery.site), delivery.unit.to_bits()))
            .collect();
        let ceiling = u64::from(self.settlements.slot_count().saturating_sub(1));
        let order = gather_order_of(&keys, ceiling)?;

        for position in order {
            let delivery = carriers[position as usize];
            let Some(row) = self.trade.row_at(delivery.row) else {
                continue;
            };
            if !row.is_bound() {
                continue;
            }
            let owed = if delivery.owes_as_proposer {
                row.owed_by_proposer()
            } else {
                row.owed_by_responder()
            };
            if owed == 0 {
                continue;
            }
            let Some(load) = self.soldiers.carry(delivery.unit) else {
                continue;
            };
            let carried = load.of(delivery.kind).0;
            if carried == 0 {
                continue;
            }
            let commodity = WORK_COMMODITY[delivery.kind.index()];
            let Some(held) = self
                .settlements
                .store_column()
                .get(delivery.site as usize)
                .and_then(|store| store.quantity(commodity))
            else {
                continue;
            };
            // The room of the store, in whole units. The subtract cannot go
            // below zero because the ceiling is the largest value the scale
            // holds.
            let room = sim_math::sub(Fix32::MAX, held).to_int_floor();
            let moved = carried.min(owed).min(u32::try_from(room).unwrap_or(0));
            if moved == 0 {
                continue;
            }
            // The conversion is exact in both directions: a whole number that
            // the room admits fits the scale, and the scale holds it with no
            // fractional part.
            let quantity = Fix32::from_int(i16::try_from(moved).unwrap_or(i16::MAX));
            let moved = u32::try_from(quantity.to_int_floor()).unwrap_or(0);
            if moved == 0 {
                continue;
            }
            let after = sim_math::add(held, quantity);
            if !self.set_store_quantity(delivery.site, commodity, after) {
                continue;
            }
            self.soldiers
                .take_carry(delivery.unit, delivery.kind, Amount(moved));
            // The delivered account links the carry account to the store
            // account. A transfer that forgot it would break the conservation
            // check on the frame of the first contract delivery.
            self.delivered[delivery.kind.index()] += u64::from(moved);
            let paid = match self.trade.row_at_mut(delivery.row) {
                Some(entry) => {
                    if delivery.owes_as_proposer {
                        entry.given = entry.given.saturating_add(moved);
                    } else {
                        entry.taken = entry.taken.saturating_add(moved);
                    }
                    if entry.is_paid() {
                        entry.status = TRADE_SETTLED;
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
            if paid {
                let (proposer, responder) = self.pair_of(delivery.row);
                self.say(proposer, responder, ACT_SETTLE, TRADE_SETTLED);
                // A contract delivered in full warms both directions.
                self.relations
                    .on_contract_delivered(self.tick, proposer, responder);
            }
        }
        Ok(())
    }

    /// Returns the ordered pair that one row index names.
    pub(super) fn pair_of(&self, index: usize) -> (FactionId, FactionId) {
        let width = (self.trade.factions() as usize).max(1);
        let proposer = (index / width) as u16;
        let responder = (index % width) as u16;
        (FactionId(proposer), FactionId(responder))
    }

    /// Fails every contract that reached its deadline with a debt.
    ///
    /// The walk is over the plane in pair order, so it names no thread and it
    /// reads no hash order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn fail_overdue_contracts(&mut self) {
        let tick = self.tick;
        let mut failed: Vec<(usize, bool, bool, u32)> = Vec::new();
        for (index, row) in self.trade.rows().iter().enumerate() {
            if !row.is_bound() || row.deadline.0 > tick.0 {
                continue;
            }
            // A side that no unit carries cannot be short on its own. It
            // waits on the other side, so only a carried side owes at the
            // deadline.
            failed.push((
                index,
                row.owed_by_proposer() > 0 && row.proposer_side_is_carried(),
                row.owed_by_responder() > 0 && row.responder_side_is_carried(),
                row.term,
            ));
        }
        for (index, proposer_owes, responder_owes, term) in failed {
            let (proposer, responder) = self.pair_of(index);
            if let Some(entry) = self.trade.row_at_mut(index) {
                entry.status = TRADE_DEFAULTED;
            }
            // The party that was owed cools toward the party that defaulted.
            if proposer_owes {
                self.relations.on_contract_failed(tick, responder, proposer);
            }
            if responder_owes {
                self.relations.on_contract_failed(tick, proposer, responder);
            }
            let until = Tick(tick.0.saturating_add(u64::from(term)));
            // The defaulting party loses the direction it would ask on again,
            // for as long as the contract ran. The duration comes from the
            // contract, so no balance figure decides it.
            if proposer_owes {
                if let Some(door) = self.trade.row_mut(proposer, responder) {
                    door.closed_until = until;
                }
            }
            if responder_owes {
                if let Some(door) = self.trade.row_mut(responder, proposer) {
                    door.closed_until = until;
                }
            }
            self.say(proposer, responder, ACT_DEFAULT, TRADE_DEFAULTED);
        }
    }

    /// Returns every live unit that stands on the tile of its home site, with
    /// the slot of that site.
    ///
    /// The walk is over the unit slots and it reads no derived structure. A
    /// unit with no home, a unit whose home is gone, and a unit that stands
    /// somewhere else all give nothing.
    ///
    /// The order is the slot order, which does not depend on the thread
    /// count. The caller sorts it on a total key before it transfers
    /// anything, so nothing downstream reads this order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn carriers_at_home(&self, threads: usize) -> Vec<(Entity, u32)> {
        let _ = threads;
        let mut found = Vec::new();
        for unit in self.soldiers.iter() {
            let Some(Some(home)) = self.soldiers.home(unit) else {
                continue;
            };
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            if load == CarryLoad::EMPTY {
                continue;
            }
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            let stands_at_home = self
                .settlements
                .tile_column()
                .get(home as usize)
                .is_some_and(|site_tile| *site_tile == tile);
            if stands_at_home {
                found.push((unit, home));
            }
        }
        found
    }
}
