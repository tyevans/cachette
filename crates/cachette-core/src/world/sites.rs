//! A settlement, its store, and the siege that takes or razes it.
//!
//! A site changes hands to work and never to a moment. The siege pass, the
//! capture, the raze and the elimination of a faction that lost its last site
//! sit together, because each one is a stage of the same story. The store
//! readers sit here too, because a store belongs to a site.

use super::errors::{IdentityError, RazeError};
use super::World;
use crate::controller::FactionRow;
use crate::event::{FactionEliminated, SiteTaken, TAKE_KIND_CAPTURED, TAKE_KIND_RAZED};
use crate::hex::Axial;
use crate::sim_math;
use crate::site::{CommodityId, SettlementArena, SettlementError, COMMODITY_COUNT};
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx, FACTION_CEILING};

/// What the siege pass does to one site on one tick.
///
/// A site falls to work and never to a moment. The pass reads the trigger
/// for every site first, and it writes afterwards, so the answer for one
/// site is fixed before any write moves the world.[^1]
///
/// # References
///
/// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SiegeStep {
    /// A unit of the owning faction stands on the site tile, so any siege
    /// the site carried ends and its work is gone.
    Ends,
    /// Nobody presses the site this tick, and no unit of the owning faction
    /// stands on its tile. A siege the site carries waits, unchanged.
    Waits,
    /// A siege stands and has not reached the work a capture costs.
    Presses {
        /// The faction that stands on the site tile.
        besieger: FactionId,
        /// The work the siege has done, including this tick.
        work: i64,
    },
    /// A siege stands and has reached the work a capture costs.
    Falls {
        /// The faction that stands on the site tile.
        besieger: FactionId,
        /// The work the siege has done, including this tick.
        work: i64,
        /// The work a raze of this site costs.
        raze_work: i64,
        /// One when the besieging faction ordered a raze, zero otherwise.
        ordered: u8,
    },
}

/// The stock one settlement can hold, as a raw Q16.16 quantity summed over
/// every commodity.
///
/// A store holds one `Fix32` for each commodity, and a `Fix32` saturates at
/// `i32::MAX`. The product of the two is therefore the most stock one
/// settlement can ever report, whatever it produces and however long it
/// runs.
///
/// **This ceiling is the reason the wealth bar is not a free value.** A bar
/// below it is crossed by one settlement that only waits, because the store
/// rises and does not fall. A bar above it asks the faction for a second
/// settlement.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-543. `docs/FINDINGS.md`
pub const STOCK_CEILING_OF_ONE_SETTLEMENT: i64 = (i32::MAX as i64) * (COMMODITY_COUNT as i64);

impl World {
    /// Returns the settlements of the world.
    ///
    /// The settlement is one of the four fixed entity shapes, and it has
    /// its own column set. It is fixed to a tile and it holds pooled
    /// stores.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    #[must_use]
    pub const fn settlements(&self) -> &SettlementArena {
        &self.settlements
    }

    /// Destroys a settlement and reports whether it destroyed one.
    ///
    /// A stale identity destroys nothing and returns `false`. The identity
    /// of a destroyed settlement never resolves again, so the settlement
    /// founded next in that slot does not answer to it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn destroy_settlement(&mut self, entity: Entity) -> bool {
        // What the settlement held leaves the account here. A dead slot keeps
        // its bytes until a founding clears them, and those bytes are not a
        // holding of anybody. The account must fall by the same amount, or
        // the conservation check finds a difference that no rate made.
        //
        // The slot is read before the loss, because the identity stops
        // resolving the moment the arena frees the slot.[^1]
        //
        // [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
        let Some(slot) = self.settlements.slot_of(entity) else {
            return false;
        };
        let store = self
            .settlements
            .store(entity)
            .expect("the identity resolved to a live slot");
        if !self.settlements.destroy(entity) {
            return false;
        }
        // A rate belongs to the site that earned it. The site is gone, so the
        // rate goes with it and the slot does not pay its successor.
        self.rates.clear_slot(slot);
        // A position belongs to the site that opened it. The site is gone,
        // so its positions go with it and the settlement founded next in
        // that slot does not inherit a staff it never hired.
        self.positions.clear_slot(slot);
        // A lost settlement takes its queue with it. A block left as it was
        // would give the settlement founded next in that slot the orders of
        // the one before it.
        self.queues.clear_slot(slot);
        // A unit that drew from the lost site now belongs to no site. A home
        // left behind would name the slot, and the settlement founded next
        // in that slot would feed a population it never took.
        for unit in self.soldiers.iter().collect::<Vec<_>>() {
            if self.soldiers.home(unit) == Some(Some(slot)) {
                self.soldiers.set_home(unit, None);
            }
        }
        for (index, account) in self.store_account.iter_mut().enumerate() {
            let held = store
                .quantity(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            *account = sim_math::combine(*account, Accum(-i64::from(held.0)));
        }
        true
    }

    /// Presses the siege against every site that a rival occupies
    /// undefended, and takes or burns the ones that fall.
    ///
    /// **A site falls to work and never to a moment.** A faction that stands
    /// on a site tile with no unit of the owning faction on it besieges the
    /// site. The siege does one work for each besieging unit on the tile, on
    /// each tick it stands. The site changes hands when the work reaches what
    /// the site resists, and the resistance is the residents the site
    /// holds.[^6]
    ///
    /// **A garrison of one refuses the siege, and a relief force ends one.**
    /// The trigger is read again on every tick. The tick the owner puts a
    /// unit back on the tile, or the tick the besieger leaves, the siege ends
    /// and its work is gone. A besieger that returns starts at nothing. A
    /// besieged city therefore has two answers: keep a unit at home, or send
    /// one back before the work is done.[^7]
    ///
    /// **The occupier is the faction the occupancy list names, and that list
    /// is built once for the tick.** The lease pass reads it to move a lease
    /// and this pass reads it to decide a siege, so who stands on a tile is
    /// stated once. The list names the faction with the most units on the
    /// tile, and a tie goes to the lowest faction identifier. Two factions
    /// that could besiege one site therefore resolve by a rule and never by
    /// an iteration order, and the work of the faction that loses the tile is
    /// gone.[^2] [^3]
    ///
    /// **What stands at a kept site passes to the taker whole.** The store,
    /// the housing, the rates, the staff and the upgrades on the ground are
    /// untouched, and every unit that draws from the site changes faction
    /// with it. Only the queue is cleared, because a queue holds orders that
    /// the taker never gave.[^4]
    ///
    /// **The taker keeps a city it can supply and burns one it cannot, and
    /// burning costs more.** A city of the taker supplies the captured site
    /// when the site stands inside the reach of that city. The reach is the
    /// quantity the ground rule already computes for every city on every
    /// tick, and the upgrades a faction finishes inside its own ground extend
    /// it to a bound.[^5] A supplied site changes hands at the capture work.
    /// A site no city of the taker reaches does not fall there: the siege
    /// presses on to the raze work, which is a multiple of the capture work,
    /// and the site burns when the siege reaches that.[^6]
    ///
    /// The pass walks the settlements in slot order, and the occupancy list
    /// is in ascending tile order. Both are stable keys.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D3. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0180, a site changes hands or the taker destroys it, decisions D1 and D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^5]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^6]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^7]: ADR-0180, a site changes hands or the taker destroys it, decision D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    pub(super) fn capture_sites(&mut self, occupancy: &[(TileIdx, FactionId)]) {
        // **The pass reads every site before it writes one.** A write moves
        // the faction of a site and the faction of its residents, and a later
        // read would then see a world that the earlier sites of this same
        // tick had changed. The plan for each site is fixed first.
        let mut plans: Vec<(Entity, SiegeStep)> = Vec::new();
        let mut any_fell = false;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            let step = self.siege_step(site, occupancy);
            if matches!(step, SiegeStep::Falls { .. }) {
                any_fell = true;
            }
            plans.push((site, step));
        }
        // **The taker keeps a city it can supply and burns one it cannot.**
        // The reach of a city is the quantity the ground rule already
        // computes for every city on every tick, and the upgrades a faction
        // finishes inside its own ground are what extend it.[^8] A road
        // between two cities therefore decides which conquests a faction can
        // keep, and nothing here states a distance of its own.
        //
        // The list is built once for the tick, and only on a tick that has a
        // site at its capture work. A tick with none pays nothing for it.
        //
        // [^8]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        let cities = if any_fell {
            self.holding.cities(&self.settlements, &self.upgrades)
        } else {
            Vec::new()
        };
        let mut moved = false;
        for (site, step) in plans {
            match step {
                SiegeStep::Ends => {
                    if self.settlements.siege(site).is_some() {
                        self.sieges_relieved =
                            sim_math::combine(Accum(self.sieges_relieved), Accum(1)).0;
                    }
                    let _ = self.settlements.clear_siege(site);
                }
                SiegeStep::Waits => {}
                SiegeStep::Presses { besieger, work } => {
                    self.sieges_pressed = sim_math::combine(Accum(self.sieges_pressed), Accum(1)).0;
                    let _ = self.settlements.set_siege(site, besieger, work);
                }
                SiegeStep::Falls {
                    besieger,
                    work,
                    raze_work,
                    ordered,
                } => {
                    self.sieges_pressed = sim_math::combine(Accum(self.sieges_pressed), Accum(1)).0;
                    let Some(address) = self.settlements.address(site) else {
                        continue;
                    };
                    // **A faction that ordered a raze burns the site,
                    // whatever the reach says.** The order is the one way a
                    // caller states an intent the engine would not have
                    // chosen, and it costs the raze work like every other
                    // raze.
                    if ordered == 1 {
                        if work >= raze_work {
                            moved |= self.burn_site(site, besieger);
                        } else {
                            let _ = self.settlements.set_siege(site, besieger, work);
                        }
                        continue;
                    }
                    // The site being taken still belongs to the faction that
                    // is losing it, so the taker's own cities are the only
                    // ones this walk sees.
                    let mut holds_one = false;
                    let mut supplied = false;
                    for city in &cities {
                        if city.faction != besieger {
                            continue;
                        }
                        holds_one = true;
                        if address.distance(city.address) <= city.reach {
                            supplied = true;
                            break;
                        }
                    }
                    // **A taker that holds no city keeps what it takes.** The
                    // rule asks which of the taker's cities supplies this
                    // one, and a faction with none is not a faction that
                    // failed to reach it. A captured city is then the only
                    // city that faction has, and it supplies itself. Without
                    // this the last army of a beaten faction could never take
                    // a capital, and a faction that lost every city could
                    // never return.
                    if supplied || !holds_one {
                        moved |= self.take_site(site, besieger);
                    } else if work >= raze_work {
                        moved |= self.burn_site(site, besieger);
                    } else {
                        // The taker cannot supply the site, so the siege
                        // presses on to the raze work. The city stands, and
                        // its owner has every tick until then to relieve it.
                        let _ = self.settlements.set_siege(site, besieger, work);
                    }
                }
            }
        }
        let took = moved;
        // **A capture writes the faction of a unit, and that moves the arena
        // past the derived structure.** The unit stands where it stood, so
        // the rebuild gives the same structure back and only restamps the
        // revision. A pass after this one reads the structure and refuses a
        // stale one, so the restamp cannot wait for the next barrier.
        if took {
            debug_assert!(
                self.refresh_bridge().is_ok(),
                "the arena and the structure describe one world"
            );
            let _ = self.refresh_bridge();
        }
    }

    /// Returns what the siege pass does to one site this tick.
    ///
    /// The call writes nothing. It reads the trigger, adds this tick's work
    /// to the work the siege carried, and compares the total against what the
    /// site resists.[^1]
    ///
    /// **The work the siege carried counts only when the same faction did
    /// it.** The occupancy names one faction for a tile, so a site whose
    /// besieger loses the tile starts again at nothing under whoever takes
    /// it. Two factions that could besiege one site therefore resolve by the
    /// rule that decides the occupier, and never by an iteration order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D8 and D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    fn siege_step(&self, site: Entity, occupancy: &[(TileIdx, FactionId)]) -> SiegeStep {
        let Some(tile) = self.settlements.tile(site) else {
            return SiegeStep::Waits;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return SiegeStep::Waits;
        };
        // **A garrison of one refuses the siege, and it takes the work with
        // it.** The defence is read from the units that stand on the tile and
        // never from the occupancy, because the occupancy names only the
        // largest faction and a garrison of one is rarely that.
        let factions = self.soldiers.faction_column();
        let standing = self.bridge.on_tile_unguarded(tile);
        if standing
            .iter()
            .any(|unit| factions[unit.index() as usize] == owner)
        {
            return SiegeStep::Ends;
        }
        let Ok(entry) = occupancy.binary_search_by_key(&tile.0, |(tile, _)| tile.0) else {
            return SiegeStep::Waits;
        };
        let besieger = occupancy[entry].1;
        if besieger == owner {
            return SiegeStep::Waits;
        }
        // **The force is the units of the besieging faction on the tile.** A
        // besieger does one work a tick, which is what a builder does, so a
        // larger army takes a city sooner and one unit takes a long time.
        let force = standing
            .iter()
            .filter(|unit| factions[unit.index() as usize] == besieger)
            .count();
        if force == 0 {
            return SiegeStep::Waits;
        }
        let carried = match self.settlements.siege(site) {
            Some((who, work)) if who == besieger => work,
            _ => 0,
        };
        let work = sim_math::combine(Accum(carried), Accum(force as i64)).0;
        let residents = self.site_residents(site).unwrap_or(0);
        if work < self.siege_rules.capture_work(residents) {
            return SiegeStep::Presses { besieger, work };
        }
        SiegeStep::Falls {
            besieger,
            work,
            raze_work: self.siege_rules.raze_work(residents),
            ordered: u8::from(
                self.settlements.siege_intent(site) == Some(crate::site::SIEGE_INTENT_RAZE),
            ),
        }
    }

    /// Moves one standing site to a new faction, with everything it holds.
    ///
    /// **This is the one place a capture happens.** The site keeps its
    /// identity, so a stored handle to a captured site still resolves.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    fn take_site(&mut self, site: Entity, taker: FactionId) -> bool {
        let Some(slot) = self.settlements.slot_of(site) else {
            return false;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return false;
        };
        let Some(tile) = self.settlements.tile(site) else {
            return false;
        };
        if owner == taker || !self.settlements.set_faction(site, taker) {
            return false;
        }
        // The siege ends with the thing it stood against. A site that changed
        // hands is no longer besieged, and the work that took it is spent.
        let _ = self.settlements.clear_siege(site);
        // **The residents change hands with the site.** A resident is a live
        // unit whose home names this slot, and the household is the home
        // column read backwards. Conquest makes the taker larger, and a
        // resident left with its old faction would be a person inside a city
        // that is no longer its own.
        //
        // The walk is over the live units in slot order, which is a stable
        // key.
        for unit in self.soldiers.iter().collect::<Vec<_>>() {
            if self.soldiers.home(unit) != Some(Some(slot)) {
                continue;
            }
            self.soldiers.set_faction(unit, taker);
            // **A character changes hands with the unit that carries it.** A
            // unit names a character, and the character carries a faction of
            // its own. A resident that changed faction while its character
            // did not would leave the two disagreeing, and no pass reads both
            // to notice.[^1]
            //
            // [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
            let bits = self.soldiers.character_column()[unit.index() as usize];
            if let Some(character) = Entity::from_bits(bits) {
                self.characters.set_faction(character, taker);
            }
        }
        // A queue holds orders the previous faction gave. The taker gave none
        // of them, so the block is cleared by the same call a loss uses. Work
        // that stands on the ground is an upgrade on a tile and it is
        // untouched, so a part-built upgrade changes hands with the ground
        // and a part-built order does not.
        self.queues.clear_slot(slot);
        // The cohort table indexes the residents by site and by faction, and
        // the faction of every resident has just changed.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        self.sites_captured = sim_math::combine(Accum(self.sites_captured), Accum(1)).0;
        self.taken_log.push(SiteTaken::new(
            self.tick,
            site.to_bits(),
            tile,
            owner,
            taker,
            TAKE_KIND_CAPTURED,
        ));
        true
    }

    /// Orders the siege of a faction to destroy the site rather than take
    /// it.
    ///
    /// **A raze is an order and never a moment.** The order writes the
    /// intent of a siege that already stands. The siege then presses to the
    /// work a raze costs, and the site burns when the work reaches it. The
    /// order therefore costs the razing faction the same time and the same
    /// force that the engine charges every other raze.[^1]
    ///
    /// **This is the one thing a caller states that the engine would not.**
    /// The engine keeps a site its own reach supplies and burns one it does
    /// not.[^2] A caller that wants a supplied city burned orders this, and
    /// the reach then decides nothing.
    ///
    /// The order lasts as long as the siege. A relief force that ends the
    /// siege ends the order with it, and a besieger that comes back must
    /// order again.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no live site, when the razer
    /// owns the site, and when no siege of the razer stands against it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decisions D9 and D11. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^2]: ADR-0180, a site changes hands or the taker destroys it, decision D7. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    /// [^3]: ADR-0180, a site changes hands or the taker destroys it, decision D10. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    pub fn order_raze(&mut self, site: Entity, razer: FactionId) -> Result<(), RazeError> {
        let owner = self
            .settlements
            .faction(site)
            .ok_or(RazeError::NoSuchSite)?;
        if owner == razer {
            return Err(RazeError::OwnSite);
        }
        match self.settlements.siege(site) {
            Some((besieger, _)) if besieger == razer => {}
            _ => return Err(RazeError::NotBesieging),
        }
        if !self
            .settlements
            .set_siege_intent(site, crate::site::SIEGE_INTENT_RAZE)
        {
            return Err(RazeError::NotBesieging);
        }
        Ok(())
    }

    /// Destroys a site and pays its store to the faction that took it.
    ///
    /// **This is the one place a raze happens.** The caller's verb and the
    /// capture pass both go through it, and neither repeats the work. The
    /// caller checks the trigger and refreshes the derived unit structure
    /// afterwards, because the two callers reach this from different points
    /// of a step.
    ///
    /// Returns `false` when the identity names no live site.
    fn burn_site(&mut self, site: Entity, razer: FactionId) -> bool {
        let Some(slot) = self.settlements.slot_of(site) else {
            return false;
        };
        let Some(owner) = self.settlements.faction(site) else {
            return false;
        };
        let Some(tile) = self.settlements.tile(site) else {
            return false;
        };
        let store = self
            .settlements
            .store(site)
            .expect("the identity resolved to a live slot");
        // The plunder moves before the loss, because the loss subtracts what
        // the site still holds from the account. Moving it first and clearing
        // it leaves the account exactly as it was.
        if let Some(receiver) = self.nearest_site_of(razer, tile) {
            for index in 0..COMMODITY_COUNT {
                let commodity = CommodityId(index as u16);
                let carried = store
                    .quantity(commodity)
                    .expect("the index came from the commodity count");
                let held = self
                    .settlements
                    .store(receiver)
                    .expect("the nearest site is live")
                    .quantity(commodity)
                    .expect("the index came from the commodity count");
                let _ =
                    self.settlements
                        .set_store(receiver, commodity, sim_math::add(held, carried));
                let _ = self.settlements.set_store(site, commodity, Fix32::ZERO);
            }
        }
        // The upgrades on the tile go with the site. An upgrade changes hands
        // with the ground, so a razer that wanted the roads and the walls
        // should have captured instead.[^3]
        //
        // [^3]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
        self.upgrades.remove(tile);
        // The residents die with the site. The despawn is the one path a unit
        // leaves the world by, so a razed resident leaves the arena, the
        // structure and the cohorts by the route a killed one takes.
        let residents: Vec<Entity> = self
            .soldiers
            .iter()
            .filter(|unit| self.soldiers.home(*unit) == Some(Some(slot)))
            .collect();
        for unit in residents {
            self.despawn_soldier(unit);
        }
        if !self.destroy_settlement(site) {
            return false;
        }
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        self.sites_razed = sim_math::combine(Accum(self.sites_razed), Accum(1)).0;
        self.taken_log.push(SiteTaken::new(
            self.tick,
            site.to_bits(),
            tile,
            owner,
            razer,
            TAKE_KIND_RAZED,
        ));
        true
    }

    /// Returns the live site of a faction that stands nearest a tile.
    ///
    /// A tie between two sites at one distance goes to the lower settlement
    /// slot, because the walk is in ascending slot order and the comparison
    /// is strict.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn nearest_site_of(&self, faction: FactionId, tile: TileIdx) -> Option<Entity> {
        let address = self.grid.address_of(tile)?;
        let mut best: Option<(u32, Entity)> = None;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            let Some(seat) = self.settlements.address(site) else {
                continue;
            };
            let distance = address.distance(seat);
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, site));
            }
        }
        best.map(|(_, site)| site)
    }

    /// Reports whether a faction has left the game.
    ///
    /// A faction outside the ceiling has never been in the game, and this
    /// answers `false` for it.
    #[must_use]
    pub fn is_eliminated(&self, faction: FactionId) -> bool {
        self.eliminated
            .get(faction.0 as usize)
            .is_some_and(|state| *state == 1)
    }

    /// Removes from the game every faction that holds no site and no unit.
    ///
    /// **A faction leaves the game when nothing of it can act and nothing of
    /// it can grow.** A site grows people and a unit acts, so a faction with
    /// neither has no way back into the world. It cannot found, because a
    /// founding needs a unit. It cannot build, gather, fight or take ground.
    /// A rule that waited for more would wait for ever.[^1]
    ///
    /// **A character alone does not keep a faction in play.** A character
    /// carries no tile, so it stands nowhere. It holds no ground, it takes no
    /// step and it fights nothing.[^2] The pass removes the characters of a
    /// faction that leaves, so nothing outlives the faction and no stored row
    /// names a faction that is out.
    ///
    /// **The ground it held is released to nobody.** The spread of the next
    /// tick then gives each released tile to the nearest city that reaches
    /// it, by the rule that already decides every holder, and a tile that no
    /// city reaches stays with nobody.[^3] Nothing here states a second rule
    /// for who inherits.
    ///
    /// **The pass runs once for each tick and it does not cascade.** Ground
    /// released by one elimination reaches another faction on the next tick,
    /// through the spread. A pass that settled until quiet would run a
    /// variable number of times, and no pass in this engine does that.[^4]
    ///
    /// The walk is over the factions in ascending identifier order, which is
    /// a stable key. Two factions that leave on one tick are recorded in that
    /// order.[^5]
    ///
    /// # References
    ///
    /// [^1]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D1. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^2]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    /// [^3]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D3. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^4]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn eliminate_factions(&mut self, threads: usize) {
        let population = *self.soldiers.population_by_faction();
        let mut sites = [0u32; FACTION_CEILING as usize];
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            if let Some(faction) = self.settlements.faction(site) {
                sites[faction.0 as usize] += 1;
            }
        }
        let mut leaving: Vec<FactionId> = Vec::new();
        for number in 0..self.config.faction_count.max(1) {
            let faction = FactionId(number);
            let index = usize::from(number);
            if self.eliminated.get(index).copied() != Some(0) {
                continue;
            }
            // A faction that never founded and never spawned has not left the
            // game. It has not entered it. The seat is what says it entered,
            // and the controller keeps one for the first founding.
            if self
                .controller
                .row(faction)
                .and_then(FactionRow::seat)
                .is_none()
            {
                continue;
            }
            if sites[index] > 0 || population[index] > 0 {
                continue;
            }
            leaving.push(faction);
        }
        for faction in leaving {
            self.eliminated[faction.0 as usize] = 1;
            // The release ends the holder and the lease together. A lease
            // left behind would take the tile back on the next spread, from a
            // faction that can no longer raise it.
            let released = self.holding.release(faction, threads);
            let leftover: Vec<Entity> = self
                .characters
                .iter()
                .filter(|character| {
                    self.characters.faction_column()[character.index() as usize] == faction
                })
                .collect();
            for character in leftover {
                self.characters.remove(character);
            }
            self.eliminated_log
                .push(FactionEliminated::new(self.tick, released, faction));
        }
    }

    /// Returns the siege that stands against a site.
    ///
    /// The answer is the besieging faction and the work that faction has
    /// done. It is `None` when the identity names no live site, and `None`
    /// when no siege stands.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn siege_of(&self, site: Entity) -> Option<(FactionId, i64)> {
        self.settlements.siege(site)
    }

    /// Returns the work a siege must do before this site changes hands.
    ///
    /// The resistance is the residents the site holds, so the answer moves
    /// as the city grows and as a war empties it.[^1] Returns `None` when
    /// the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D8. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn capture_work_of(&self, site: Entity) -> Option<i64> {
        let residents = self.site_residents(site)?;
        Some(self.siege_rules.capture_work(residents))
    }

    /// Returns the work a siege must do before this site is destroyed.
    ///
    /// Returns `None` when the identity names no live site.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D9. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn raze_work_of(&self, site: Entity) -> Option<i64> {
        let residents = self.site_residents(site)?;
        Some(self.siege_rules.raze_work(residents))
    }

    /// Returns the sites that changed hands since the last step began.
    #[must_use]
    pub fn taken_log(&self) -> &[SiteTaken] {
        &self.taken_log
    }

    /// Returns the raw bytes of the taken log.
    #[must_use]
    pub fn taken_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.taken_log)
    }

    /// Returns the factions that left the game since the last step began.
    #[must_use]
    pub fn eliminated_log(&self) -> &[FactionEliminated] {
        &self.eliminated_log
    }

    /// Returns the raw bytes of the eliminated log.
    #[must_use]
    pub fn eliminated_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.eliminated_log)
    }

    /// Resolves the value of an identity back to the settlement it names.
    ///
    /// A caller outside this crate holds an identity as the value the engine
    /// gave it. It cannot build one, and this is the only way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds. It refuses a mismatch, and it never
    /// returns the settlement that now stands in the slot.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an identity, when the arena
    /// holds no such slot, or when the slot holds a later generation.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D2 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn resolve_settlement(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.settlements.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.settlements.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.settlements.generation_of(slot),
        })
    }

    /// Returns the settlement that stands on an address.
    ///
    /// Returns `None` when the address is outside the world, and `None`
    /// when no settlement stands there.
    #[must_use]
    pub fn settlement_on(&self, address: Axial) -> Option<Entity> {
        self.settlements.on_tile(address)
    }

    /// Returns the faction that holds a settlement.
    ///
    /// Returns `None` when the identity names no live settlement. A site
    /// changes hands, so this answer is not fixed at the founding and a
    /// caller must read it again.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0180, a site changes hands or the taker destroys it, decision D1. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
    #[must_use]
    pub fn settlement_faction(&self, entity: Entity) -> Option<FactionId> {
        self.settlements.faction(entity)
    }

    /// Returns the quantity of one commodity in the store of a settlement.
    ///
    /// Returns `None` when the identity is dead, and `None` when the
    /// commodity is outside the set.
    #[must_use]
    pub fn settlement_store(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        self.settlements
            .store(entity)
            .and_then(|store| store.quantity(commodity))
    }

    /// Writes the quantity of one commodity into the store of a settlement.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the commodity is outside the commodity set.
    pub fn set_settlement_store(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        quantity: Fix32,
    ) -> Result<bool, SettlementError> {
        // A write from outside the rate pass changes what the stores hold, so
        // it must change the account by the same amount. The old quantity is
        // read before the write, because after the write it is gone.
        let before = self
            .settlements
            .store(entity)
            .and_then(|store| store.quantity(commodity));
        let wrote = self.settlements.set_store(entity, commodity, quantity)?;
        if wrote {
            let before = before.expect("the write resolved the commodity");
            let index = commodity.0 as usize;
            let change = i64::from(quantity.0) - i64::from(before.0);
            self.store_account[index] = sim_math::combine(self.store_account[index], Accum(change));
        }
        Ok(wrote)
    }

    /// Writes a quantity into the store of one site slot and keeps the
    /// account.
    ///
    /// **The account and the store are written in one call.** They are two
    /// copies of one total, and the conservation check is what fails when they
    /// disagree, so a path that wrote one without the other would break that
    /// check on every later frame.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub(super) fn set_store_quantity(
        &mut self,
        slot: u32,
        commodity: CommodityId,
        after: Fix32,
    ) -> bool {
        let index = commodity.0 as usize;
        let Some(store) = self
            .settlements
            .store_update()
            .stores
            .get_mut(slot as usize)
        else {
            return false;
        };
        let Some(before) = store.quantity(commodity) else {
            return false;
        };
        if !store.set_quantity(commodity, after) {
            return false;
        }
        let change = sim_math::sub(after, before);
        self.store_account[index] = sim_math::accumulate(self.store_account[index], change);
        true
    }

    /// Returns how much the finished stores on or beside the tile of a
    /// settlement raise its store capacity, as a raw Q16.16 quantity.
    ///
    /// **This is the one place that states the "on or beside" rule.** A
    /// finished store on the tile of the settlement, or on one of its six
    /// neighbours, adds its raise. The raise of one kind is a catalogue
    /// row.[^1]
    ///
    /// **Nothing in the engine reads this.** The engine holds no store
    /// capacity, so the sum is a reading for the control plane and it changes
    /// no pass. Returns `None` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the store capacity raise. `docs/reference/balance.md`
    #[must_use]
    pub fn store_capacity_raise(&self, settlement: Entity) -> Option<i64> {
        let address = self.settlements.address(settlement)?;
        let mut raise = Accum(0);
        for place in core::iter::once(Some(address)).chain(self.grid.neighbours(address)) {
            let Some(row) = place.and_then(|near| self.standing_upgrade_row(near)) else {
                continue;
            };
            raise = sim_math::combine(raise, Accum(i64::from(row.capacity_of_store_change)));
        }
        Some(raise.0)
    }
}
