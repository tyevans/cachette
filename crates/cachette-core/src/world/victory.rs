//! How a game ends: the paths to a win, and the standing of each faction.
//!
//! A game ends on a tick limit or on a win path. The readers of each path,
//! the standing they combine into and the settings that bound them sit
//! together, because a caller compares one path against another.

use super::World;
use crate::balance::Balance;
use crate::controller::{self, FactionRow, GameEnd, WinPath};
use crate::holding::{nearest_settlement_of, Holder};
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx};
use crate::upgrade::{self, UpgradeCategory, UpgradeRow};

/// One game end reader: a pure function of the world that names the faction
/// that wins on its path, or nobody.
type GameEndReader = fn(&World) -> Option<FactionId>;

/// The running value of one faction on each win path.
///
/// A caller that reads this watches every path the readers watch: the held
/// tiles for territory, the seats and the live units for domination, the
/// best renown for renown, and the wonder work for the wonder. The stock
/// total feeds no reader, and it is reported because a caller may want to
/// see a faction grow rich.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
/// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decisions D2 and D4. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Standing {
    /// The tiles the faction holds. The territory reader compares it.
    pub held_tiles: i64,
    /// The seats the faction holds, its own and every rival's. The
    /// domination reader compares it against the seat count.
    pub seats_held: i64,
    /// The units of the faction that are alive. The domination reader
    /// compares it: the clause fires for a faction that still has a unit
    /// while every rival has none.
    pub live_units: i64,
    /// The sum of every store of every settlement of the faction, as a raw
    /// Q16.16 quantity. **No reader compares it**, because a stock total
    /// wins no game. It is reported and not read.[^2]
    ///
    /// # References
    ///
    /// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decision D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    pub store_total: i64,
    /// The highest renown of any live character of the faction, as a raw
    /// Q16.16 value. The renown reader compares it.
    pub best_renown: i64,
    /// The most work any wonder on ground the faction holds has reached.
    /// The wonder reader does not compare this. It compares whether a
    /// finished wonder stands, and this value is how far the furthest
    /// unfinished one has come.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    pub wonder_progress: i64,
}

/// One tile on which a wonder stands, or on which work builds toward one.
///
/// A wonder is an upgrade row that carries a victory claim above zero.[^1]
/// The wonder lookup returns one of these for each tile whose standing row
/// carries a claim, and for each tile whose next row carries one.
///
/// **The holder and the settlement say whose wonder it is.** The faction that
/// holds the tile gets the credit on the wonder path. The settlement is the
/// city of that faction that the ground belongs to, so a caller that wants to
/// stop a wonder knows which city to march on.
///
/// # References
///
/// [^1]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WonderSite {
    /// The tile that carries the work.
    pub tile: TileIdx,
    /// The victory claim of the row that stands on the tile.
    ///
    /// Zero means that the wonder is still under construction.
    pub claim: i64,
    /// The work done toward the wonder.
    ///
    /// A finished wonder reports the whole requirement, because the work of a
    /// site returns to zero when its level rises.
    pub work: i64,
    /// The work that the wonder row asks for.
    pub requirement: i64,
    /// The faction that holds the tile, or `None` when nobody holds it.
    pub holder: Option<FactionId>,
    /// The settlement of the holder that the tile belongs to, or `None` when
    /// nobody holds the tile or the holder has no live settlement.
    pub settlement: Option<Entity>,
}

impl WonderSite {
    /// Reports whether the wonder stands, and is not only under
    /// construction.
    #[must_use]
    pub const fn is_finished(self) -> bool {
        self.claim > 0
    }

    /// Returns the work done as a share of the requirement.
    ///
    /// A finished wonder reads one. A requirement of zero reads as one, so
    /// the share never divides by zero.
    #[must_use]
    pub const fn progress_share(self) -> Fix32 {
        sim_math::bounded_share(self.work, self.requirement)
    }
}

impl World {
    /// Returns the tick at which the territory reader fires.
    #[must_use]
    pub const fn tick_limit(&self) -> u64 {
        self.controller.tick_limit()
    }

    /// Sets the tick at which the territory reader fires.
    ///
    /// The limit is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the tick limit. `docs/reference/balance.md`
    pub const fn set_tick_limit(&mut self, tick_limit: u64) {
        self.controller.set_tick_limit(tick_limit);
    }

    /// Returns the game end record. It is empty until a reader fires.
    #[must_use]
    pub const fn game_end(&self) -> GameEnd {
        self.controller.game_end()
    }

    /// Returns the score of one faction on the territory path: the tiles it
    /// holds.
    ///
    /// The count is the running total the holding keeps, so this starts no
    /// pass.[^1] Returns `None` when the world has no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn score(&self, faction: FactionId) -> Option<i64> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        Some(self.holding.holding_of(faction))
    }

    /// Runs the game end readers, while the record is empty.
    ///
    /// The readers run in the fixed order domination, territory, wonder,
    /// renown. The first that fires writes the record, and the record is
    /// written once.[^1] Each reader is a pure function of the world, and
    /// each resolves a tie by the lowest faction identifier, because it
    /// visits the factions in ascending order and stops at the first that
    /// fires.[^2]
    ///
    /// **A caller may turn the readers off.** While they are off this
    /// function records nothing and the world runs to the tick limit. The
    /// readers decide when the step stops watching, and they change nothing
    /// else, so a run with the readers off holds the same event log as a run
    /// with the readers on that never fires.[^4]
    ///
    /// **A stock total wins no game.** The wealth path is gone, and the
    /// wonder is a path of its own with a reader in this table.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    /// [^4]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub(super) fn check_game_end(&mut self) {
        if !self.balance.win_readers_enabled() {
            return;
        }
        if self.controller.game_end().is_set() {
            return;
        }
        // The order of this table is a rule of the game and not a balance
        // value. A path that has no reader is absent from it.
        let readers: [(GameEndReader, WinPath); 4] = [
            (Self::domination_winner, WinPath::Domination),
            (Self::territory_winner, WinPath::Territory),
            (Self::wonder_winner, WinPath::Wonder),
            (Self::renown_winner, WinPath::Renown),
        ];
        for (reader, path) in readers {
            if let Some(winner) = reader(self) {
                self.controller.record_end(self.tick, winner, path);
                return;
            }
        }
    }

    /// The factions of the world, in ascending identifier order.
    pub(super) fn factions(&self) -> impl Iterator<Item = FactionId> {
        (0..self.config.faction_count.max(1)).map(FactionId)
    }

    /// The factions that may still win, in ascending identifier order.
    ///
    /// **A faction that has left the game wins nothing.** This is the one
    /// statement of that rule, and every game end reader walks this list
    /// rather than the faction list. A reader that walked the factions would
    /// let a faction with nothing alive take the world on held ground it can
    /// no longer defend.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D5. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^2]: Findings register, FND-579. `docs/FINDINGS.md`
    fn contenders(&self) -> impl Iterator<Item = FactionId> + '_ {
        self.factions()
            .filter(move |faction| !self.is_eliminated(*faction))
    }

    /// Returns the faction that holds the seat of a faction, or `None` when
    /// the faction has no seat or nobody holds it.
    fn seat_holder(&self, faction: FactionId) -> Option<FactionId> {
        let seat = self.controller.row(faction).and_then(FactionRow::seat)?;
        self.grid
            .address_of(seat)
            .and_then(|address| self.holding.holder(address))
            .and_then(Holder::faction)
    }

    /// The domination reader: one faction holds every seat, or every other
    /// faction has no units.
    ///
    /// A seat is the tile of the first founding of a faction, and the
    /// controller keeps it as one tile for each faction. The reader reads the
    /// holder of each seat tile, one lookup for each faction, and reads the
    /// live count of each faction, which the soldier arena keeps as a running
    /// total. It walks no unit and no tile.[^1]
    ///
    /// **A faction alone has dominated nothing.** The seat clause needs a
    /// rival seat to hold, so it fires only when the winner holds the seat of
    /// at least one other faction. The unit clause needs a rival to have
    /// lost, so it fires only in a world of two or more factions and only
    /// for a faction that still has a unit. Without both guards an empty
    /// world, or a world of one faction, would end on the first tick.
    ///
    /// A tie resolves by the lowest faction identifier: the walk is in
    /// ascending order and stops at the first faction that fires.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    fn domination_winner(&self) -> Option<FactionId> {
        if self.config.faction_count < 2 {
            return None;
        }
        let population = self.soldiers.population_by_faction();
        self.contenders().find(|candidate| {
            let mut rival_seats = 0u32;
            let mut holds_every_seat = true;
            let mut every_rival_is_empty = true;
            for other in self.factions() {
                // A faction that has left the game is no rival. Its seat is
                // not a seat to hold and its population is already zero.
                if other != *candidate && self.is_eliminated(other) {
                    continue;
                }
                if self
                    .controller
                    .row(other)
                    .and_then(FactionRow::seat)
                    .is_some()
                {
                    if other != *candidate {
                        rival_seats += 1;
                    }
                    holds_every_seat &= self.seat_holder(other) == Some(*candidate);
                }
                if other != *candidate && population[usize::from(other.0)] > 0 {
                    every_rival_is_empty = false;
                }
            }
            let by_seats = rival_seats > 0 && holds_every_seat;
            let by_units = every_rival_is_empty && population[usize::from(candidate.0)] > 0;
            by_seats || by_units
        })
    }

    /// The territory reader: at the tick limit, the faction with the most
    /// held tiles. The held count is a running total the holding keeps.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn territory_winner(&self) -> Option<FactionId> {
        if self.tick.0 < self.controller.tick_limit() {
            return None;
        }
        let held = self
            .contenders()
            .map(|faction| (faction, self.holding.holding_of(faction)))
            .collect::<Vec<_>>();
        controller::territory_winner(held.into_iter())
    }

    /// Returns the seats a faction holds: the seat tiles, its own and every
    /// rival's, whose holder is the faction.
    fn seats_held_by(&self, faction: FactionId) -> i64 {
        self.factions()
            .filter(|other| self.seat_holder(*other) == Some(faction))
            .count() as i64
    }

    /// Returns the stock total of every faction, by faction number.
    ///
    /// The total sums every commodity of every live settlement of the
    /// faction, as raw Q16.16 quantities, in a 64-bit accumulator. The
    /// walk is over the settlement arena in slot order, and it is not a walk
    /// over the population or the tiles.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    fn stock_totals(&self) -> Vec<Accum> {
        let mut totals = vec![Accum(0); usize::from(self.config.faction_count.max(1))];
        let live = self.settlements.live_column();
        let factions = self.settlements.faction_column();
        for (slot, store) in self.settlements.store_column().iter().enumerate() {
            if live[slot] == 0 {
                continue;
            }
            let Some(total) = totals.get_mut(usize::from(factions[slot].0)) else {
                continue;
            };
            for commodity in 0..COMMODITY_COUNT {
                let quantity = store
                    .quantity(CommodityId(commodity as u16))
                    .expect("the commodity index is below the count");
                *total = sim_math::combine(*total, Accum(i64::from(quantity.0)));
            }
        }
        totals
    }

    /// Returns the largest victory claim that stands on ground a faction
    /// holds, for every faction by faction number.
    ///
    /// The reader walks the entries that stand at a level and reads the
    /// victory claim column of each row. It names no category.[^2]
    ///
    /// The walk is over the sparse upgrade map, which holds one entry for
    /// each improved tile and nothing else, so it is not a walk over the
    /// tiles.[^1] A claim on ground nobody holds counts for nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn wonder_progress(&self) -> Vec<i64> {
        self.victory_claims()
            .into_iter()
            .map(|pair| pair.1)
            .collect()
    }

    /// Returns every wonder of the world: each tile on which a wonder stands,
    /// and each tile on which work builds toward one.
    ///
    /// **This is the one declaration of where a wonder stands, how far it has
    /// come, and whose it is.** The wonder reader, the standing, the
    /// observation and the controller read it, so none of them states the
    /// rule a second time.
    ///
    /// The contract of each entry:
    ///
    /// - A tile whose standing row carries a victory claim is a finished
    ///   wonder. Its claim is that claim, and its work and its requirement
    ///   are both the work of that row.
    /// - A tile whose next row carries a claim builds toward a wonder. Its
    ///   claim is zero, its work is the work done, and its requirement is the
    ///   work of the next row.
    /// - The holder is the faction that holds the tile now. A tile nobody
    ///   holds has no holder and no settlement.
    /// - The settlement is the live settlement of the holder nearest to the
    ///   tile. Two settlements at one distance resolve by the lower slot. The
    ///   rule reads no reach, so two settlements whose reaches both cover the
    ///   tile resolve by distance and then by slot. It is the rule that gives
    ///   a finished upgrade to one city of its faction, and it has one
    ///   statement.[^1]
    ///
    /// The entries are in ascending tile order, which is the order of the
    /// sparse upgrade map. No hash order and no thread order enters.[^2]
    ///
    /// The walk is over one entry for each improved tile, so it is not a walk
    /// over the tiles.[^3] Each held entry also walks the settlement slots
    /// once, and the entries are the wonder sites alone.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub fn wonder_sites(&self) -> Vec<WonderSite> {
        self.wonder_sites_unplaced()
            .map(|site| WonderSite {
                settlement: site
                    .holder
                    .and_then(|faction| {
                        let address = self.grid.address_of(site.tile)?;
                        nearest_settlement_of(self.grid, &self.settlements, faction, address)
                    })
                    .and_then(|slot| self.settlements.entity_at(slot)),
                ..site
            })
            .collect()
    }

    /// Returns every wonder of the world with its holder, and with no
    /// settlement named.
    ///
    /// The reader walks the sparse upgrade map and reads the victory claim
    /// column of the row that stands at each entry. It names no category.[^1]
    ///
    /// **Whether the work of an entry builds toward a claim has one
    /// statement, and this walk calls it.** The wonder work pass reads the
    /// same test to decide which work decays and which resets, so the work
    /// this walk counts is the work that pass touches.[^2]
    ///
    /// **The game end reader runs every step and needs no settlement**, so it
    /// reads this walk and pays nothing for the settlement rule. The public
    /// lookup adds the settlement to the same entries.
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn wonder_sites_unplaced(&self) -> impl Iterator<Item = WonderSite> + '_ {
        self.upgrades.sites().iter().filter_map(move |site| {
            let standing = self.upgrade_table.row(site.category, site.level);
            let claimed = standing.map_or(0, |row| i64::from(row.victory_claim));
            let (claim, work, requirement) = if claimed > 0 {
                let whole = standing.map_or(0, |row| i64::from(row.work));
                (claimed, whole, whole)
            } else if site.builds_toward_a_claim(&self.upgrade_table) {
                let whole = self.upgrade_table.work_above(site.category, site.level);
                (0, site.progress.0, whole)
            } else {
                return None;
            };
            let holder = self
                .grid
                .address_of(site.tile)
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction);
            Some(WonderSite {
                tile: site.tile,
                claim,
                work,
                requirement,
                holder,
                settlement: None,
            })
        })
    }

    /// Returns the largest victory claim, and the work toward it, on the
    /// ground of every faction, by faction number.
    ///
    /// This folds the wonder lookup by holder. An entry that stands at a row
    /// with a claim reports the work of that row, and an entry that builds
    /// toward one reports the work done. A claim on ground nobody holds
    /// counts for nobody.
    pub(crate) fn victory_claims(&self) -> Vec<(i64, i64)> {
        let mut best = vec![(0i64, 0i64); usize::from(self.config.faction_count.max(1))];
        for site in self.wonder_sites_unplaced() {
            if let Some(slot) = site
                .holder
                .and_then(|faction| best.get_mut(usize::from(faction.0)))
            {
                slot.0 = slot.0.max(site.claim);
                slot.1 = slot.1.max(site.work);
            }
        }
        best
    }

    /// The wonder reader: a finished wonder stands on ground the faction
    /// holds.
    ///
    /// A wonder is an upgrade row that carries a victory claim above zero.
    /// The reader walks the sparse upgrade map, reads the claim of the row
    /// that stands at each entry, and finds the first faction that holds a
    /// standing claim. A tie resolves by the lowest faction identifier.
    ///
    /// **A wonder costs work and stands on held ground, so it is an
    /// achievement.** A stock total is not, and the wealth clause that
    /// compared one is gone.[^1] The work a wonder costs and the claim its
    /// row carries are both columns of the upgrade table, and a caller sets
    /// them.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn wonder_winner(&self) -> Option<FactionId> {
        let claims = self.victory_claims();
        self.contenders()
            .find(|faction| claims[usize::from(faction.0)].0 > 0)
    }

    /// Returns the highest renown of any live character of every faction, by
    /// faction number, as raw Q16.16 values.
    ///
    /// The walk is over the character arena in slot order. It is not a walk
    /// over the units or the tiles.
    fn best_renown(&self) -> Vec<i64> {
        let mut best = vec![0i64; usize::from(self.config.faction_count.max(1))];
        for entity in self.characters.iter() {
            let (Some(faction), Some(renown)) = (
                self.characters.faction(entity),
                self.characters.renown(entity),
            ) else {
                continue;
            };
            if let Some(slot) = best.get_mut(usize::from(faction.0)) {
                *slot = (*slot).max(i64::from(renown.0));
            }
        }
        best
    }

    /// The renown reader: a character of the faction reaches the renown
    /// target.
    ///
    /// **The contest writes renown, and this reader fires in a seeded
    /// run.** The killer of each pair earns a share for each unit it felled,
    /// and the share goes to the champion of the faction. The control plane
    /// may also write the column. The blocker that governs the wider renown
    /// rule is open, and the target is a balance value under it.[^1] [^2] A
    /// tie resolves by the lowest faction identifier.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-150. `docs/BLOCKERS.md`
    /// [^2]: Balance register, the renown target. `docs/reference/balance.md`
    fn renown_winner(&self) -> Option<FactionId> {
        let best = self.best_renown();
        self.contenders()
            .find(|faction| best[usize::from(faction.0)] >= i64::from(self.balance.renown_target()))
    }

    /// Returns the running value of one faction on each win path.
    ///
    /// Returns `None` when the world has no such faction. The values are the
    /// ones the readers compare, so a caller can watch a path approach its
    /// end.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    #[must_use]
    pub fn standing(&self, faction: FactionId) -> Option<Standing> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        let at = usize::from(faction.0);
        Some(Standing {
            held_tiles: self.holding.holding_of(faction),
            seats_held: self.seats_held_by(faction),
            live_units: i64::from(self.soldiers.population_by_faction()[at]),
            store_total: self.stock_totals()[at].0,
            best_renown: self.best_renown()[at],
            wonder_progress: self.wonder_progress()[at],
        })
    }

    /// Returns the win-path balance values of the world.
    ///
    /// Each value holds a constant as its default, so a world that nobody
    /// configures behaves as it did before the table existed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D1. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    #[must_use]
    pub const fn balance(&self) -> Balance {
        self.balance
    }

    /// Sets the renown at which the renown reader fires, as a raw Q16.16
    /// value.
    ///
    /// This is a threshold. It decides when the renown reader fires and
    /// changes nothing else.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_renown_target(&mut self, raw: i32) {
        self.balance.set_renown_target(raw);
    }

    /// Sets the renown that one felled unit gives the champion of the faction
    /// that felled it, as a raw Q16.16 value.
    ///
    /// This is a rate. It changes what the simulation does, because the
    /// renown column is state that a later frame reads.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_renown_per_fell(&mut self, raw: i32) {
        self.balance.set_renown_per_fell(raw);
    }

    /// Sets whether the game end readers run.
    ///
    /// While they do not run, no reader records a game end and the world runs
    /// to the tick limit. A run with the readers off holds the same event log
    /// as a run with the readers on that never fires.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub const fn set_win_readers_enabled(&mut self, enabled: bool) {
        self.balance.set_win_readers_enabled(enabled);
    }

    /// Sets the work that finishes a wonder.
    ///
    /// The work is a column of the upgrade table row that holds the wonder,
    /// and this writes that column and leaves the other columns of the row
    /// where they are. The table is the one declaration site of the value.
    ///
    /// This is a rate. A wonder that costs more work takes longer to build,
    /// so the value changes what the simulation does.[^1]
    ///
    /// Returns `false` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub fn set_wonder_work(&mut self, work: u32) -> bool {
        let Some(row) = self
            .upgrade_table
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        else {
            return false;
        };
        self.upgrade_table
            .define(
                UpgradeCategory::WONDER.to_u8(),
                upgrade::WONDER_LEVEL,
                UpgradeRow { work, ..row },
            )
            .is_ok()
    }

    /// Sets the victory claim that the wonder row carries.
    ///
    /// The claim is a column of the upgrade table row that holds the wonder,
    /// and this writes that column and leaves the other columns of the row
    /// where they are. The table is the one declaration site of the value.
    ///
    /// This is a threshold. The wonder reader fires for the faction that
    /// holds a standing claim above zero, so a claim of zero takes the wonder
    /// path out of the game and changes nothing else.[^1]
    ///
    /// Returns `false` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    pub fn set_wonder_victory_claim(&mut self, claim: u32) -> bool {
        let Some(row) = self
            .upgrade_table
            .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        else {
            return false;
        };
        self.upgrade_table
            .define(
                UpgradeCategory::WONDER.to_u8(),
                upgrade::WONDER_LEVEL,
                UpgradeRow {
                    victory_claim: claim,
                    ..row
                },
            )
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::Axial;
    use crate::world::WorldConfig;

    /// Returns the claims by the walk that the wonder lookup replaced.
    ///
    /// **This is that walk, kept as the oracle.** The fold over the lookup
    /// must give the same pair for every faction, because the game end
    /// reader and the observation read the pair, and the state hash follows
    /// the game end.
    fn claims_by_the_replaced_walk(world: &World) -> Vec<(i64, i64)> {
        let mut best = vec![(0i64, 0i64); usize::from(world.config.faction_count.max(1))];
        for site in world.upgrades.sites() {
            let standing = world.upgrade_table.row(site.category, site.level);
            let next = world.upgrade_table.row(site.category, site.level + 1);
            let claimed = standing.map_or(0, |row| i64::from(row.victory_claim));
            let (claim, work) = if claimed > 0 {
                (claimed, standing.map_or(0, |row| i64::from(row.work)))
            } else if next.is_some_and(|row| row.victory_claim > 0) {
                (0, site.progress.0)
            } else {
                continue;
            };
            let holder = world
                .grid
                .address_of(site.tile)
                .and_then(|address| world.holding.holder(address))
                .and_then(Holder::faction);
            if let Some(slot) = holder.and_then(|faction| best.get_mut(usize::from(faction.0))) {
                slot.0 = slot.0.max(claim);
                slot.1 = slot.1.max(work);
            }
        }
        best
    }

    /// Returns an address that admits a unit, near the one asked for.
    fn ground_near(world: &World, wanted: Axial) -> Axial {
        (0..=14)
            .flat_map(|ring: i32| {
                (-ring..=ring).flat_map(move |column| {
                    (-ring..=ring)
                        .filter(move |row| column.abs().max(row.abs()) == ring)
                        .map(move |row| Axial::new(wanted.q + column, wanted.r + row))
                })
            })
            .find(|candidate| world.admits_a_unit(*candidate))
            .expect("the fixture finds ground that admits a unit")
    }

    /// Founds a city of one faction near one address, puts builders on its
    /// tile, and orders each to build a wonder. Returns the tile.
    fn a_wonder_underway(
        world: &mut World,
        faction: FactionId,
        wanted: Axial,
        builders: u32,
    ) -> Axial {
        let place = ground_near(world, wanted);
        world
            .spawn_soldier(place, faction)
            .expect("the ground admits a unit");
        world
            .found_settlement(place, faction)
            .expect("the ground admits a city");
        world.step(1).expect("the step runs");
        for _ in 0..builders {
            let unit = world
                .spawn_soldier(place, faction)
                .expect("the ground admits a unit");
            world
                .order_build(unit, UpgradeCategory::WONDER)
                .expect("the builder stands on ground its faction holds");
        }
        place
    }

    /// The fold over the wonder lookup gives the claims the replaced walk
    /// gave.
    ///
    /// **The fixture holds every case the fold reads.** One faction holds a
    /// finished wonder, whose work is the work of its row and not the work of
    /// the site. Another holds a wonder under construction. The third holds
    /// none. A fold that read the work of the site for a finished wonder, or
    /// that read the claim of the next row, would disagree with the oracle.
    #[test]
    fn the_fold_over_the_lookup_gives_the_claims_the_replaced_walk_gave() {
        let mut world = World::new(WorldConfig {
            width: 96,
            height: 96,
            seed: 0x0a1d_5eed_0003_0003,
            faction_count: 3,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
            ..WorldConfig::DEFAULT
        })
        .expect("the configuration describes a world");
        world
            .set_choice_schedule(12)
            .expect("the exponent is inside the range");
        world.set_win_readers_enabled(false);
        assert!(world.set_wonder_work(240));
        let finished_at = ground_near(&world, Axial::new(20, 20));
        let room = world
            .tile_capacity(finished_at)
            .expect("the tile lies inside the world")
            .saturating_sub(1)
            .max(1);
        a_wonder_underway(&mut world, FactionId(0), finished_at, room);
        a_wonder_underway(&mut world, FactionId(1), Axial::new(70, 70), 1);
        for _ in 0..400 {
            if world.finished_upgrade(finished_at) == Some(UpgradeCategory::WONDER) {
                break;
            }
            world.step(1).expect("the step runs");
        }
        world.step(1).expect("the step runs");

        let sites = world.wonder_sites();
        assert!(
            sites
                .iter()
                .any(|site| site.is_finished() && site.holder == Some(FactionId(0))),
            "the fixture must finish a wonder on the ground of faction 0"
        );
        assert!(
            sites.iter().any(|site| !site.is_finished()
                && site.work > 0
                && site.holder == Some(FactionId(1))),
            "the fixture must leave a wonder under construction on the ground of faction 1"
        );
        assert_eq!(
            world.victory_claims(),
            claims_by_the_replaced_walk(&world),
            "the fold over the lookup gives the claims of the replaced walk"
        );
    }
}
