//! The upgrades that stand on a tile: the order, the build, and the wear.
//!
//! An upgrade is content on the ground that a unit builds and weather, fire
//! or an army wears down. The table, the build order, the build pass and the
//! wear pass sit together, because each stage reads the level the previous
//! one left.

use super::errors::StepError;
use super::World;
use crate::event::{
    SettlementFounded, UpgradeCollapsed, UpgradeFinished, WEAR_CAUSE_ARMY, WEAR_CAUSE_BOTH,
    WEAR_CAUSE_FIRE, WEAR_CAUSE_ORDERED, WEAR_CAUSE_WEATHER,
};
use crate::fire::FireField;
use crate::hex::Axial;
use crate::holding::{Holder, Holding};
use crate::plan::{self, PlanRegister};
use crate::sim_math;
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::sort::{BoundedKey, SortError};
use crate::types::{Entity, FactionId, TileIdx};
use crate::unit_type::{UnitTypeId, UnitTypeTable};
use crate::upgrade::{
    self, BuildRefusal, UpgradeCategory, UpgradeMap, UpgradeRow, UpgradeSite, UpgradeTable,
    UpgradeTableError,
};

/// One unit that is building, and what it is building.
#[derive(Clone, Copy, Debug)]
struct BuildIntent {
    /// The unit that builds.
    unit: Entity,
    /// The tile that the unit stands on.
    tile: TileIdx,
    /// The category that the unit builds.
    category: UpgradeCategory,
    /// The type of the unit. The advance reads the build rate of the row it
    /// indexes.
    unit_type: UnitTypeId,
}

/// Returns the order in which the advance reads the build intents.
///
/// The order is the key vector sort: by the tile and the kind together, then
/// by the identity of the unit.[^1] It depends on the key values alone, so it
/// is the same at any thread count, and it does not follow the slot order of
/// the arena.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// The sound sort is used in the perturbed build as well. The advance sums a
/// count of builders, and integer addition is order-free, so a perturbed order
/// would change nothing and a probe over it would assert nothing.[^3]
///
/// [^3]: ADR-0004, iteration order is explicit, decision D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn build_order_of(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    crate::sort::order_bounded(keys, ceiling)
}

/// Reports whether a builder may build one row on the ground it stands on.
///
/// **One function states the rule, and two paths call it.** The verb that
/// gives a build order calls it at the moment of the order. The pass that
/// collects the build intents calls it on every step. Two tests that drifted
/// apart would let a build the verb refused finish anyway.[^1] [^2]
///
/// The holder of the tile must be the builder's own faction when the row asks
/// for it. A row whose own ground column is zero crosses ground nobody holds,
/// because that is how a faction reaches ground it does not yet hold.[^1] That
/// row is permitted only where the builder's own faction zoned a project of
/// the same category, so the plan is the bound on the reach.[^4]
///
/// **The plan binds every category and not only the reaching one.** An order
/// that names one category on a tile the builder's own faction zones for
/// another is refused, whatever the row asks for. Without that clause a
/// faction plants an upgrade of one category on a tile its own plan zones for
/// another, and the tile then carries one category and no order can ever
/// raise the other.[^4] [^5]
///
/// The rule reads a column of the resolved row. It names no category, so a
/// row a caller wrote at run time obeys the same rule as a row of the default
/// table.[^3]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
/// [^4]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^5]: Findings register, FND-496. `docs/FINDINGS.md`
#[must_use]
const fn build_is_permitted(
    holder: Holder,
    faction: FactionId,
    row: UpgradeRow,
    category: UpgradeCategory,
    zoned: Option<UpgradeCategory>,
) -> bool {
    // The plan is read once, and both clauses below read that one answer. A
    // second lookup would be a second declaration of the same fact.[^6]
    //
    // [^6]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    if let Some(held) = zoned {
        if held.to_u8() != category.to_u8() {
            return false;
        }
    }
    if row.own_ground_required == 0 {
        // A row that asks for no held ground is how a faction reaches ground
        // it does not hold. The plan is the bound that stops it: a unit lays
        // one only inside a project of its own faction.[^4]
        //
        // [^4]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        return zoned.is_some();
    }
    match holder.faction() {
        Some(held) => held.0 == faction.0,
        None => false,
    }
}

/// Resolves the row that a build order names on one tile.
///
/// **This is the one statement of the resolution, and two paths call it.**
/// The verb that gives a build order calls it at the moment of the order. The
/// pass that collects the build intents calls it on every step. Two tests
/// that drifted apart would let a build the verb refused finish anyway.[^2]
///
/// The next level is one when the tile carries no upgrade of the category,
/// and one above the level that stands there otherwise.[^1]
///
/// **A level that owes a repair resolves to the row that stands there.** The
/// work of a worker on such a site buys condition first, so the row the order
/// reads is the row it mends. Without this arm a worker on a crumbling
/// top-level upgrade would be refused for the category being at its top, and
/// nothing could ever mend one.
///
/// A level whose gap in condition is worth less than one unit of work owes no
/// repair, and it resolves to the row above in the usual way. That is what
/// lets a level under light wear still rise.
///
/// # Errors
///
/// Returns a refusal when the tile carries another category, when the
/// category holds no row above the level that stands there, and when the row
/// does not fit the ground.
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub(super) fn resolve_build_row(
    table: &UpgradeTable,
    ground: crate::terrain::TileKind,
    standing: Option<UpgradeSite>,
    category: UpgradeCategory,
) -> Result<UpgradeRow, BuildRefusal> {
    let level = match standing {
        Some(site) if site.category != category => {
            return Err(BuildRefusal::TileHoldsAnother {
                standing: site.category,
                asked: category,
            })
        }
        Some(site) => site.level,
        None => upgrade::NO_LEVEL,
    };
    if let Some(site) = standing {
        // The price is the one statement of whether a repair is due, and the
        // build pass spends the same value. A gap worth less than one unit of
        // work is priced at zero, and the order then reads the row above in
        // the usual way.[^3]
        //
        // [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
        if site.repair_price(table) > 0 {
            if let Some(row) = table.row(category, site.level) {
                return Ok(row);
            }
        }
    }
    let row = table
        .row(category, level + 1)
        .ok_or(BuildRefusal::CategoryAtTop { category, level })?;
    if !row.fits(ground) {
        return Err(BuildRefusal::GroundDoesNotFit { category, ground });
    }
    Ok(row)
}

/// Returns the build intent of each live soldier that carries an order.
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the join reads the slots in slot order. The result never depends
/// on thread completion order.[^1]
///
/// A soldier with no order builds nothing and produces no intent, so a world
/// in which nobody was told to build costs one pass over the live set.
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// Returns the build intent of one unit, or `None` when it builds nothing now.
///
/// **One function states what a unit builds, and three paths call it.** The
/// build pass collects the intents. The movement pass asks whether the unit
/// stands on the work it was ordered to do. The verb that gives the order
/// calls the two rules inside it directly, at the moment of the order.[^1]
///
/// The read is gated on the build order, which is one byte of one column. A
/// unit that carries no order costs that byte and nothing else, so a
/// population that does not build pays one column and no lookup.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
fn build_intent_of(
    unit: Entity,
    soldiers: &SoldierArena,
    holding: &Holding,
    ground: plan::Ground<'_>,
    plan: &PlanRegister,
) -> Option<BuildIntent> {
    let plan::Ground {
        grid,
        terrain,
        upgrades,
        table,
    } = ground;
    let category = soldiers.build_order(unit)??;
    let tile = soldiers.tile(unit)?;
    let unit_type = soldiers.unit_type(unit)?;
    // One function resolves the row, and the verb that gives the order calls
    // the same one. A build the table no longer holds, and a build whose tile
    // gained another category since the order, stop here.[^3]
    //
    // [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    let kind = terrain.kind(grid.address_of(tile)?)?;
    let row = resolve_build_row(table, kind, upgrades.at(tile), category).ok()?;
    // One function states the ground rule, and the verb that gives the order
    // calls the same one. A build whose ground changed hands since the order
    // stops here.[^2]
    //
    // [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    let holder = holding
        .holders()
        .get(tile.0 as usize)
        .copied()
        .unwrap_or(Holder::NOBODY);
    let faction = soldiers.faction(unit)?;
    if !build_is_permitted(holder, faction, row, category, plan.zones(faction, tile)) {
        return None;
    }
    Some(BuildIntent {
        unit,
        tile,
        category,
        unit_type,
    })
}

/// Returns the work that one unit of this type adds to a site in one tick.
///
/// **This is the one statement of what a builder contributes.** The build
/// advance sums it, and the movement hold reads it to decide whether the unit
/// is doing anything by standing still. A second expression of the same
/// product would let a unit be held for work it never adds.[^1]
///
/// A row whose build rate is zero contributes zero, and so does a row whose
/// rate scales the base work below one. Zero means cannot.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
#[must_use]
const fn build_contribution(row: crate::unit_type::UnitTypeRow) -> i64 {
    sim_math::scale_work(upgrade::BUILD_RATE, row.build_rate)
}

/// Reports whether the tile a unit stands on is work it must stay for.
///
/// **A build order is per unit and per tile, and movement takes its direction
/// from a field over cells.** A hold cannot be written into that field
/// without pinning every unit of the cell, so the movement pass asks this
/// question of each unit instead. The record states the rule and the
/// reasoning.[^1] [^2]
///
/// **Nothing stores the answer.** The pass derives it from the order, the
/// ground, the plan and the type on the tick it reads it, so there is no hold
/// to clear when the work finishes, when the plan drops the project, or when
/// the unit dies. A stored hold that outlived one of those three would freeze
/// a unit for the rest of the run.[^1]
///
/// **A unit that adds no work is never held.** The soldier row, the merchant
/// row and the leader row all build at zero, and the controller orders every
/// unit of its faction that stands on a zoned tile, whatever its type.[^3] A
/// hold without this clause would freeze one of them for ever, because no
/// work would ever finish the site it stands on.
///
/// # References
///
/// [^1]: ADR-0168, a build order holds a unit on its tile, and the hold is derived, decisions D1, D2 and D3. `docs/adrs/draft/adr-0168-a-build-order-holds-a-unit-on-its-tile.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^3]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
pub(super) fn build_holds_unit(
    unit: Entity,
    soldiers: &SoldierArena,
    building: &Building<'_>,
    ground: plan::Ground<'_>,
) -> bool {
    let Building {
        holding,
        plan,
        unit_types,
        upgrades: _,
        table: _,
        fire: _,
    } = *building;
    build_intent_of(unit, soldiers, holding, ground, plan)
        .is_some_and(|intent| build_contribution(unit_types.row(intent.unit_type)) > 0)
}

fn build_intents(
    soldiers: &SoldierArena,
    holding: &Holding,
    ground: plan::Ground<'_>,
    plan: &PlanRegister,
    threads: usize,
) -> Result<Vec<BuildIntent>, StepError> {
    let plan::Ground {
        grid,
        terrain,
        upgrades,
        table,
    } = ground;
    let live: Vec<Entity> = soldiers.iter().collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<BuildIntent>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    std::thread::scope(|scope| {
        for (chunk, slot) in live.chunks(chunk_len).zip(slots.entries_mut()) {
            scope.spawn(move || {
                *slot = chunk
                    .iter()
                    .filter_map(|unit| {
                        build_intent_of(
                            *unit,
                            soldiers,
                            holding,
                            plan::Ground {
                                grid,
                                terrain,
                                upgrades,
                                table,
                            },
                            plan,
                        )
                    })
                    .collect();
            });
        }
    });

    Ok(slots.combine(Vec::new(), |mut joined, slot| {
        joined.extend_from_slice(slot);
        joined
    }))
}

/// What the movement pass reads to answer whether a unit is building here.
///
/// The three references are the three the build rule needs and that the
/// steering does not: who holds the ground, what the faction's plan zones,
/// and what the unit's type can do.
pub(super) struct Building<'a> {
    /// The held-ground column that the build rule reads.
    pub(super) holding: &'a Holding,
    /// The plan register that bounds where a unit may build.
    pub(super) plan: &'a PlanRegister,
    /// The shared type table that states what a unit contributes.
    pub(super) unit_types: &'a UnitTypeTable,
    /// The sparse upgrade map that holds what already stands on a tile.
    pub(super) upgrades: &'a UpgradeMap,
    /// The shared upgrade table that resolves the row a build order names.
    pub(super) table: &'a UpgradeTable,
    /// The tiles that burn, which the fire hold below reads.
    pub(super) fire: &'a FireField,
}

impl World {
    /// Returns every upgrade in the world, in ascending tile order.
    ///
    /// A world in which nobody built returns an empty slice. The map holds
    /// one entry for each improved tile and none for any other, so the length
    /// of this slice is the whole storage cost of the upgrades.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub fn upgrade_sites(&self) -> &[UpgradeSite] {
        self.upgrades.sites()
    }

    /// Returns the upgrade on one tile, finished or under construction.
    ///
    /// Returns `None` when the address lies outside the world, and when the
    /// tile carries no upgrade.
    #[must_use]
    pub fn upgrade_at(&self, address: Axial) -> Option<UpgradeSite> {
        self.upgrades.at(self.grid.index_of(address)?)
    }

    /// Returns how sound the upgrade on one tile is.
    ///
    /// The value runs from nothing to the full condition. A level that has
    /// just been finished stands at the full condition, the weather and a
    /// hostile army take from it, and a worker on the tile puts it back.
    ///
    /// Returns `None` when the tile carries no upgrade. A site under
    /// construction reports the full condition, because nothing stands on its
    /// tile for anything to wear.
    #[must_use]
    pub fn upgrade_condition(&self, address: Axial) -> Option<i64> {
        self.upgrade_at(address).map(|site| site.condition.0)
    }

    /// Returns the upgrades the wear pass removed since the last step began.
    ///
    /// **A removed upgrade leaves nothing behind.** The tile returns to the
    /// world the generator made, so this log is the only record that anything
    /// stood there. A reader that misses a step misses the events.
    ///
    /// The log is ordered by tile within the tick.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn collapsed_log(&self) -> &[UpgradeCollapsed] {
        &self.collapsed_log
    }

    /// Returns the upgrade levels that finished since the last step began.
    ///
    /// The log is ordered by tile within the tick. A reader that misses a
    /// step misses the events.
    #[must_use]
    pub fn finished_log(&self) -> &[UpgradeFinished] {
        &self.finished_log
    }

    /// Returns the settlements founded since the last step began.
    ///
    /// A founding by a caller between two steps lands here and stays until
    /// the next step clears it. The log is ordered by the order the foundings
    /// were made in, which no thread fixes: the settle path and every caller
    /// path are serial.
    #[must_use]
    pub fn founded_log(&self) -> &[SettlementFounded] {
        &self.founded_log
    }

    /// Returns how many upgrades the last wear collapsed.
    ///
    /// The count describes one tick, in the same way the visit count of the
    /// last build advance does. No pass reads it and it enters no state hash.
    #[must_use]
    pub fn upgrade_collapses(&self) -> u64 {
        self.upgrades.last_wear_collapses()
    }

    /// Returns the finished upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none, and when the upgrade there
    /// is still under construction. An unfinished build changes nothing about
    /// the tile.
    #[must_use]
    pub fn finished_upgrade(&self, address: Axial) -> Option<UpgradeCategory> {
        let site = self.upgrade_at(address)?;
        if site.is_complete() {
            Some(site.category)
        } else {
            None
        }
    }

    /// Returns the level that stands on one tile.
    ///
    /// Returns zero when the tile carries no upgrade, and when the upgrade
    /// there has not reached its first level.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub fn upgrade_level(&self, address: Axial) -> u8 {
        self.upgrade_at(address)
            .map_or(upgrade::NO_LEVEL, |site| site.level)
    }

    /// Returns the row that stands on one tile.
    ///
    /// Returns `None` when the tile carries no upgrade, and when the upgrade
    /// there has not reached its first level. This is how a pass asks what an
    /// upgrade does: it reads a column of the row, and it names no
    /// category.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub fn standing_upgrade_row(&self, address: Axial) -> Option<UpgradeRow> {
        let tile = self.grid.index_of(address)?;
        self.upgrades.standing(tile, &self.upgrade_table)
    }

    /// Returns every entry that stands at a level, with the row it indexes.
    ///
    /// An entry whose first level is still under construction is not here,
    /// because it changes nothing about its tile. The walk is over the sparse
    /// map in tile order, so it is not a walk over the world.[^1]
    ///
    /// A pass that asks what the upgrades of a world do reads this and then
    /// reads a column. It names no category.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub(super) fn standing_rows(&self) -> impl Iterator<Item = (UpgradeSite, UpgradeRow)> + '_ {
        self.upgrades.sites().iter().filter_map(|site| {
            self.upgrade_table
                .row(site.category, site.level)
                .map(|row| (*site, row))
        })
    }

    /// Returns the table that a category and a level index.
    ///
    /// The table holds one row for each pair of a category and a level. A row
    /// names the ground it fits, the work it takes and the columns a pass
    /// reads.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub const fn upgrade_table(&self) -> &UpgradeTable {
        &self.upgrade_table
    }

    /// Writes one row of the upgrade table.
    ///
    /// The caller gives the whole row. There is no partial form, because a
    /// caller that gave two columns would leave the rest at zero and would
    /// define an upgrade that changes nothing else without knowing it.[^1]
    ///
    /// **The values are content and not a budget.** No record holds one,
    /// because a record may hold no number that a content choice can
    /// move.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no category, and when the level
    /// is not one the table holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
    pub fn define_upgrade_row(
        &mut self,
        category: u8,
        level: u8,
        row: UpgradeRow,
    ) -> Result<(), UpgradeTableError> {
        self.upgrade_table.define(category, level, row)
    }

    /// Returns the number of units that may stand on one tile.
    ///
    /// This is the one reader of the ground table and the upgrade table
    /// together. Admission calls the same function, so no caller can read one
    /// table without the other.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub fn tile_capacity(&self, address: Axial) -> Option<u32> {
        self.tile_capacity_for(address, crate::terrain::NO_WATER_CROSSING)
    }

    /// Returns the number of units of a given water crossing that may stand
    /// on one tile.
    ///
    /// The argument is the water crossing column of a unit type row, and zero
    /// means cannot.[^2] The reader still reads the ground table and the
    /// upgrade table together, and it still states no rule of its own about
    /// which ground admits whom.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    #[must_use]
    pub fn tile_capacity_for(&self, address: Axial, water_crossing: u32) -> Option<u32> {
        let ground = self.terrain.kind(address)?.capacity_for(water_crossing);
        Some(upgrade::capacity_with(
            ground,
            self.standing_upgrade_row(address),
        ))
    }

    /// Returns the number of entries that the last build advance read.
    ///
    /// The advance reads the builders and the sites. It reads no tile, so
    /// this number does not grow with the size of the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub const fn last_build_visits(&self) -> u64 {
        self.upgrades.last_advance_visits()
    }

    /// Tells one soldier to build, or to stop building.
    ///
    /// The soldier adds to the upgrade on the tile it stands on, on every
    /// tick, until something stops it. It does not have to stay: a soldier
    /// that walks away stops adding, and the work it did stays on the
    /// tile.[^1]
    ///
    /// **The order names a category and never a level.** The engine resolves
    /// the row from the ground under the tile and the level that stands
    /// there, and it refuses when no row fits.[^3]
    ///
    /// # Errors
    ///
    /// Returns a refusal when the identity is dead, when the tile carries
    /// another category, when the category is at its top, when the row does
    /// not fit the ground, and when the row asks for ground that the
    /// builder's own faction does not hold.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    pub fn order_build(
        &mut self,
        entity: Entity,
        category: UpgradeCategory,
    ) -> Result<(), BuildRefusal> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check, so the answer and the verb
        // cannot state two different rules.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        if let Err(refusal) = self.build_refusal(entity, category) {
            // The plan counts the refusals the permission rule makes, and
            // not the ones the table or the identity make. The split is the
            // one the verb kept before the check moved out of it.
            if matches!(
                refusal,
                BuildRefusal::ProjectHoldsAnother { .. }
                    | BuildRefusal::NoProject { .. }
                    | BuildRefusal::GroundNotHeld { .. }
            ) {
                self.plan.count_refusal();
            }
            return Err(refusal);
        }
        if self.soldiers.set_build_order(entity, Some(category)) {
            Ok(())
        } else {
            Err(BuildRefusal::NoSuchBuilder)
        }
    }

    /// Reports whether the build verb would refuse one builder, without
    /// ordering anything.
    ///
    /// **This is the one statement of the rule.** The build verb calls it
    /// before it writes an order, and the legality answer calls it to fill
    /// one row of the action table.[^1] Nothing here mutates, so a caller
    /// may ask about every candidate and change nothing.
    ///
    /// # Errors
    ///
    /// Returns the refusal the build verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn build_refusal(
        &self,
        entity: Entity,
        category: UpgradeCategory,
    ) -> Result<(), BuildRefusal> {
        // The verb refuses at the moment of the order, so a caller learns at
        // once. The pass below applies the same test on every step, so a
        // build whose ground changed hands stops.[^2]
        //
        // [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        let (Some(tile), Some(faction)) =
            (self.soldiers.tile(entity), self.soldiers.faction(entity))
        else {
            return Err(BuildRefusal::NoSuchBuilder);
        };
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        let ground = self
            .grid
            .address_of(tile)
            .and_then(|address| self.terrain.kind(address))
            .ok_or(BuildRefusal::NoSuchBuilder)?;
        let row = resolve_build_row(
            &self.upgrade_table,
            ground,
            self.upgrades.at(tile),
            category,
        )?;
        let zoned = self.plan.zones(faction, tile);
        if !build_is_permitted(holder, faction, row, category, zoned) {
            // The three refusals answer three rules. A project of another
            // category met the plan first, because it refuses whatever the
            // row asks for. A row that asks for held ground then met the
            // ground rule. A row that asks for none met the plan.
            return Err(match zoned {
                Some(held) if held != category => BuildRefusal::ProjectHoldsAnother {
                    zoned: held,
                    asked: category,
                },
                _ if row.own_ground_required == 0 => BuildRefusal::NoProject { category },
                _ => BuildRefusal::GroundNotHeld { category },
            });
        }
        if self.soldiers.slot_of(entity).is_some() {
            Ok(())
        } else {
            Err(BuildRefusal::NoSuchBuilder)
        }
    }

    /// Tells one soldier to stop building.
    ///
    /// Returns `false` when the identity is dead.
    pub fn stop_build(&mut self, entity: Entity) -> bool {
        self.soldiers.set_build_order(entity, None)
    }

    /// Returns the build order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier builds.
    #[must_use]
    pub fn build_order(&self, entity: Entity) -> Option<Option<UpgradeCategory>> {
        self.soldiers.build_order(entity)
    }

    /// Removes the upgrade from one tile and reports whether it removed one.
    ///
    /// The tile returns to the world the generator made. Nothing stores a
    /// property of an improved tile except this map, so removing the entry is
    /// the whole of the return and no second copy can survive it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn destroy_upgrade(&mut self, address: Axial) -> bool {
        let Some(tile) = self.grid.index_of(address) else {
            return false;
        };
        let Some(site) = self.upgrades.remove(tile) else {
            return false;
        };
        // **An upgrade has two sinks, and both say so.** The wear pass writes
        // the same event with a cause of its own. A destruction that wrote
        // nothing would leave a watcher with a tile that changed and no
        // reason for it.
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        self.collapsed_log.push(UpgradeCollapsed::new(
            self.tick,
            tile,
            holder,
            site.category.0,
            site.level,
            WEAR_CAUSE_ORDERED,
        ));
        true
    }

    /// Advances every upgrade that a unit is building.
    ///
    /// The pass reads the builders and the upgrade map. It reads no tile
    /// column and it takes no tile count, so a world of any size in which one
    /// unit builds costs the same.[^1]
    ///
    /// The builders of one tile are gathered into one contribution and the
    /// map is merged once, in ascending tile order. The contribution is a
    /// sum over the builders of a whole-number rate scaled by the build rate
    /// of each builder's type, so it is the same whatever order the threads
    /// produced the intents in.[^2] [^3]
    ///
    /// **A tile carries one upgrade.** When builders on one tile name
    /// different kinds, the kind already standing there wins. A tile that
    /// holds no site takes the lowest kind number present, which is the first
    /// in the sorted order, so the answer does not depend on which unit
    /// arrived first.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, or when the
    /// sort refuses the keys.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn build(&mut self, threads: usize) -> Result<(), StepError> {
        let intents = build_intents(
            &self.soldiers,
            &self.holding,
            plan::Ground {
                grid: self.grid,
                terrain: self.terrain,
                upgrades: &self.upgrades,
                table: &self.upgrade_table,
            },
            &self.plan,
            threads,
        )?;
        if intents.is_empty() {
            // The merge is still called, so the visit count describes this
            // tick rather than the last one that built anything.
            self.upgrades.merge_ascending(&[], &self.upgrade_table);
            return Ok(());
        }

        let keys: Vec<BoundedKey> = intents
            .iter()
            .map(|intent| {
                BoundedKey::new(
                    upgrade::site_key(intent.tile, intent.category),
                    intent.unit.to_bits(),
                )
            })
            .collect();
        let ceiling = upgrade::key_ceiling(self.grid.tile_count());
        let order = build_order_of(&keys, ceiling)?;

        // The key packs the tile above the kind, so the sorted order is tile
        // major and every builder of one tile sits in one run.
        let mut run: Vec<(TileIdx, UpgradeCategory, i64)> = Vec::new();
        let mut at = 0usize;
        while at < order.len() {
            let tile = intents[order[at] as usize].tile;
            let mut end = at;
            while end < order.len() && intents[order[end] as usize].tile == tile {
                end += 1;
            }
            let held = self.upgrades.at(tile).map(|site| site.category);
            let winner = held.unwrap_or(intents[order[at] as usize].category);
            // Each builder adds the builder rate scaled by the build rate of
            // its type. A build rate of zero adds nothing, so a unit that
            // cannot build keeps its order and moves no site. The sum is
            // integer addition, so it is the same in any order.[^5]
            //
            // [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
            let work = order[at..end]
                .iter()
                .map(|position| intents[*position as usize])
                .filter(|intent| intent.category == winner)
                .fold(0i64, |total, intent| {
                    total.saturating_add(build_contribution(self.unit_types.row(intent.unit_type)))
                });
            if work > 0 {
                run.push((tile, winner, work));
            }
            at = end;
        }
        // The level that stands at each tile of the run, read before the
        // merge. The raise below compares it against the level the merge
        // produced, so nothing else states which build finished.
        let before: Vec<u8> = run
            .iter()
            .map(|(tile, _, _)| {
                self.upgrades
                    .at(*tile)
                    .map_or(upgrade::NO_LEVEL, |site| site.level)
            })
            .collect();
        self.upgrades.merge_ascending(&run, &self.upgrade_table);
        self.publish_the_finished_levels();
        self.lodge_the_finished_levels(&run, &before);
        Ok(())
    }

    /// Writes one event for each level that the merge just raised.
    ///
    /// **A level that finished is a moment, and this is the one place that
    /// says so.** The census counts how many upgrades stand and how many of
    /// them claim a wonder. A count says that a thing happened and it does
    /// not say where, whose it is, or which category rose.
    ///
    /// The merge gathers the raises in ascending tile order, so the log is
    /// ordered by tile within the tick. Nothing here reads a thread
    /// completion order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn publish_the_finished_levels(&mut self) {
        if self.upgrades.last_merge_raised().is_empty() {
            return;
        }
        let tick = self.tick;
        let holders = self.holding.holders();
        let written: Vec<UpgradeFinished> = self
            .upgrades
            .last_merge_raised()
            .iter()
            .map(|site| {
                let holder = holders
                    .get(site.tile.0 as usize)
                    .copied()
                    .unwrap_or(Holder::NOBODY);
                UpgradeFinished::new(tick, site.tile, holder, site.category.0, site.level)
            })
            .collect();
        self.finished_log.extend_from_slice(&written);
    }

    /// Takes condition from every upgrade that the weather or an army wears.
    ///
    /// **This is the only sink an upgrade has that a caller does not drive by
    /// hand.** Before it existed the engine removed no upgrade and damaged
    /// none, so every road, terrace, store and wall ever built stood for the
    /// rest of the run and the built world only accumulated.
    ///
    /// Two causes take condition, and the pass sums them.
    ///
    /// A storm over the tile takes a fixed amount for each tick. The pass
    /// reads the wetness of the level 1 cell that holds the tile, which is
    /// the reader the gather resolve already uses, so the weather is read in
    /// one way and not two.[^1] [^2] The weather solve runs later in the
    /// step, so this pass reads the field the previous step left.
    ///
    /// A hostile unit on the tile takes a fixed amount for each unit. A unit
    /// is hostile when the faction that holds the tile is at war with the
    /// faction of the unit. An upgrade on ground nobody holds therefore wears
    /// only from the weather, because no faction owns it to be at war with.
    ///
    /// The pass walks the sites and the units on their tiles. It takes no
    /// grid and no tile count, so a world in which nobody built does no work
    /// here, at any tile count.[^3]
    ///
    /// The walk is serial and it runs in ascending tile order, which the map
    /// holds its entries in. Nothing here reads a thread completion order,
    /// and the sum over the units of one tile is integer addition, so it is
    /// the same in any order.[^4] [^5] The pass makes no random draw: wear is
    /// a rate and not a chance.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub(super) fn wear_upgrades(&mut self) {
        if self.upgrades.is_empty() {
            return;
        }
        let holders = self.holding.holders();
        let mut cursor = self.bridge.tile_cursor();
        // The sites are held in ascending tile order, so the run this builds
        // is in ascending tile order and the tile reader walks forward.
        let worn: Vec<(TileIdx, i64, u8, Holder)> = self
            .upgrades
            .sites()
            .iter()
            .filter(|site| site.is_complete())
            .map(|site| {
                let tile = site.tile;
                let storm = match self.weather_cell_of(tile) {
                    Some(cell) if self.weather.cell_is_wet(cell) => {
                        upgrade::WEATHER_WEAR_FOR_EACH_TICK
                    }
                    _ => 0,
                };
                let holder = holders
                    .get(tile.0 as usize)
                    .copied()
                    .unwrap_or(Holder::NOBODY);
                let army = match holder.faction() {
                    Some(owner) => {
                        let hostile = self
                            .bridge
                            .units_on_tile(&mut cursor, tile)
                            .iter()
                            .filter(|unit| {
                                self.soldiers
                                    .faction(**unit)
                                    .is_some_and(|guest| self.relations.war_between(owner, guest))
                            })
                            .count() as i64;
                        hostile.saturating_mul(upgrade::ARMY_WEAR_FOR_EACH_UNIT)
                    }
                    None => 0,
                };
                // **What stands on burning ground burns with it.** The pass
                // that owns wear owns this too, so no second site removes an
                // upgrade and the collapse event stays the one record that
                // anything stood there.[^7]
                //
                // The fire stage runs later in this step, so this reads the
                // fire the previous step left. That is the order the storm
                // term already reads the weather in.
                //
                // [^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
                let burns = if self.fire.is_burning(tile) {
                    crate::fire::WEAR_FOR_EACH_TICK
                } else {
                    0
                };
                let cause = if burns > 0 {
                    WEAR_CAUSE_FIRE
                } else {
                    match (storm > 0, army > 0) {
                        (true, true) => WEAR_CAUSE_BOTH,
                        (true, false) => WEAR_CAUSE_WEATHER,
                        (false, true) => WEAR_CAUSE_ARMY,
                        (false, false) => 0,
                    }
                };
                (
                    tile,
                    storm.saturating_add(army).saturating_add(burns),
                    cause,
                    holder,
                )
            })
            .filter(|(_, taken, _, _)| *taken > 0)
            .collect();
        let run: Vec<(TileIdx, i64)> = worn
            .iter()
            .map(|(tile, taken, _, _)| (*tile, *taken))
            .collect();
        self.upgrades.wear_ascending(&run);
        // **An upgrade that collapsed leaves nothing behind, so the event is
        // the only record of it.** The removed sites come back in ascending
        // tile order and the run is in ascending tile order, so the two walk
        // together and the log is ordered by tile within the tick. Nothing
        // here reads a thread completion order.[^6]
        //
        // [^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        let removed = self.upgrades.last_wear_removed();
        if !removed.is_empty() {
            let tick = self.tick;
            let mut there = 0usize;
            let mut written = Vec::with_capacity(removed.len());
            for site in removed {
                while there < worn.len() && worn[there].0 .0 < site.tile.0 {
                    there += 1;
                }
                let (cause, holder) = worn
                    .get(there)
                    .map_or((0, Holder::NOBODY), |entry| (entry.2, entry.3));
                written.push(UpgradeCollapsed::new(
                    tick,
                    site.tile,
                    holder,
                    site.category.0,
                    site.level,
                    cause,
                ));
            }
            self.collapsed_log.extend_from_slice(&written);
        }
    }

    /// Raises the housing of a settlement for each level that the merge
    /// finished.
    ///
    /// **This is the one place that composes an upgrade row and the housing
    /// of a site.** The row states the housing that one level adds, the
    /// settlement column stores it, and no second declaration of the number
    /// exists.[^1] The record asks for a stored field rather than a
    /// composition derived on read, so the raise is written once when the
    /// level rises.[^2]
    ///
    /// A finished level raises the settlement on its own tile, or on one of
    /// the six tiles beside it. A level that stands beside no settlement
    /// houses nobody, which is the same rule the store raise applies.
    ///
    /// The pass names no category. It reads the housing column of the row
    /// the entry indexes.[^3]
    ///
    /// The run arrives in ascending tile order and this loop is serial, so
    /// two lodgings beside one settlement add in tile order and in no other.[^4]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn lodge_the_finished_levels(
        &mut self,
        run: &[(TileIdx, UpgradeCategory, i64)],
        before: &[u8],
    ) {
        for ((tile, _, _), stood) in run.iter().zip(before.iter()) {
            let Some(site) = self.upgrades.at(*tile) else {
                continue;
            };
            if site.level <= *stood {
                continue;
            }
            // Every level the merge crossed adds the housing of its own row.
            let mut housing = 0u32;
            let mut level = *stood + 1;
            while level <= site.level {
                if let Some(row) = self.upgrade_table.row(site.category, level) {
                    housing = housing.saturating_add(row.housing_change);
                }
                level += 1;
            }
            if housing == 0 {
                continue;
            }
            let Some(address) = self.grid.address_of(*tile) else {
                continue;
            };
            let Some(settlement) = core::iter::once(Some(address))
                .chain(self.grid.neighbours(address))
                .find_map(|place| place.and_then(|near| self.settlements.on_tile(near)))
            else {
                continue;
            };
            let held = self.settlements.housing(settlement).unwrap_or(0);
            self.settlements
                .set_housing(settlement, held.saturating_add(housing));
        }
    }

    /// Gives every soldier in the set the order to build one category.
    ///
    /// The set form, shared by the binding and the controller, as the gather
    /// order is.[^1] Returns how many entities the engine refused, and the
    /// first refusal it gave. A caller that reports the reason reads the
    /// second value, and a caller that counts reads the first.
    ///
    /// One loop serves both, so no second loop can apply another rule.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub fn order_build_set_reporting(
        &mut self,
        units: &[Entity],
        category: UpgradeCategory,
    ) -> (usize, Option<BuildRefusal>) {
        let mut refused = 0usize;
        let mut first = None;
        for entity in units {
            if let Err(refusal) = self.order_build(*entity, category) {
                refused += 1;
                first = first.or(Some(refusal));
            }
        }
        (refused, first)
    }

    /// Gives every soldier in the set the order to build one category, and
    /// returns how many the engine refused.
    pub fn order_build_set(&mut self, units: &[Entity], category: UpgradeCategory) -> usize {
        self.order_build_set_reporting(units, category).0
    }
}
