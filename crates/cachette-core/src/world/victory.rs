//! How a game ends: the paths to a win, and the standing of each faction.
//!
//! A game ends on a tick limit or on a win path. The readers of each path,
//! the standing they combine into and the settings that bound them sit
//! together, because a caller compares one path against another.

use super::World;
use crate::balance::Balance;
use crate::controller::{self, FactionRow, GameEnd, WinPath};
use crate::holding::Holder;
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Accum, FactionId};
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
    /// [^1]: ADR-0173, the wealth or wonder path has no reader, decision D1. `docs/adrs/draft/adr-0173-the-wealth-or-wonder-path-has-no-reader.md`
    pub wonder_progress: i64,
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

    /// Returns the largest victory claim, and the work toward it, on the
    /// ground of every faction, by faction number.
    ///
    /// The reader walks the sparse upgrade map and reads the victory claim
    /// column of two rows for each entry: the row that stands there, and the
    /// row above it. An entry that stands at a row with a claim reports the
    /// work of that row, and an entry that builds toward one reports the work
    /// done. It names no category.[^2]
    ///
    /// The walk is over one entry for each improved tile, so it is not a walk
    /// over the tiles.[^1] A claim on ground nobody holds counts for nobody.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub(crate) fn victory_claims(&self) -> Vec<(i64, i64)> {
        let mut best = vec![(0i64, 0i64); usize::from(self.config.faction_count.max(1))];
        for site in self.upgrades.sites() {
            let standing = self.upgrade_table.row(site.category, site.level);
            let next = self.upgrade_table.row(site.category, site.level + 1);
            let claimed = standing.map_or(0, |row| i64::from(row.victory_claim));
            let (claim, work) = if claimed > 0 {
                (claimed, standing.map_or(0, |row| i64::from(row.work)))
            } else if next.is_some_and(|row| row.victory_claim > 0) {
                (0, site.progress.0)
            } else {
                continue;
            };
            let holder = self
                .grid
                .address_of(site.tile)
                .and_then(|address| self.holding.holder(address))
                .and_then(Holder::faction);
            if let Some(slot) = holder.and_then(|faction| best.get_mut(usize::from(faction.0))) {
                slot.0 = slot.0.max(claim);
                slot.1 = slot.1.max(work);
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
