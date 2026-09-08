//! The tables that parameterise a unit, and the queues that build one.
//!
//! This module holds the unit type table, the upgrade table, the build cost
//! table, the production queue of a settlement, and the verbs that raise,
//! stop and read a build.
//!
//! A unit type and an upgrade set are data, not code. A type is an index into
//! a shared table, so a type parameterises a verb rather than multiplying the
//! verbs.[^1] The tables and the queues sit together because a queue entry
//! names a row of a table.
//!
//! # References
//!
//! [^1]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::{resolve, resolve_site};
use cachette_core::plan::PlanRules;
use cachette_core::production::{BuildCostRow, QueueOrder};
use cachette_core::rates::RateSchedule;
use cachette_core::site::COMMODITY_COUNT;
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow};
use cachette_core::upgrade::{UpgradeCategory, UpgradeRow};
use cachette_core::{Axial, CommodityId, FactionId, Fix32};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Writes one row of the shared unit type table.
    ///
    /// A unit type is an index into this table. The table is data that the
    /// world holds. It holds no code, and the engine reads it rather than
    /// branching on a type name.[^1]
    ///
    /// The `unit_type` is the row number, as a Python integer. The table
    /// holds eight rows, numbered zero to seven. A number of eight or above
    /// names no row. A new soldier carries row zero, which the world builds
    /// as the worker row.
    ///
    /// **The call takes the whole row.** A row is nine capability columns,
    /// and a zero in a column means that the type cannot do what the column
    /// names. There is no two-column form, because a caller that gave two
    /// columns would leave the rest at zero and would define a unit that
    /// fights and does nothing else without knowing it.[^3]
    ///
    /// The `attack` is the harm that one unit of this type delivers in one
    /// resolution. The value is a Python integer in the project fixed-point
    /// scale. The unit of that scale is a whole casualty. The scale holds 16
    /// fractional bits, so one whole casualty is the value 65536. An attack
    /// of 65536 therefore ends one unit for each attacker. An attack of
    /// 32768 ends one unit for every two.
    ///
    /// The `armour` is the attack that an attacker must exceed to reach a
    /// unit of this type. The value is in the same scale.
    ///
    /// The keyword arguments follow, and each is a Python integer.
    ///
    /// - `gather_rate`. The scale on what the unit takes from a tile in one
    ///   tick, in the same fixed-point scale. 65536 takes the tile rate.
    ///   Zero takes nothing, and `order_gather` refuses the unit.
    /// - `build_rate`. The scale on the work the unit adds to an upgrade in
    ///   one tick, in the fixed-point scale. Zero adds nothing, and
    ///   `order_build` refuses the unit.
    /// - `carry_capacity`. The most the unit carries, summed over every
    ///   kind, as a whole count. A gather never raises a load above it. Zero
    ///   means the unit never carries, and so never gathers.
    /// - `move_cost_scale`. The scale on the movement cost the unit pays, in
    ///   the fixed-point scale. **No pass reads this column yet.**
    /// - `command_reach`. A whole count. Nonzero means the unit may move a
    ///   relation. **No pass reads this column yet.**
    /// - `weather_reach`. A whole count. Nonzero means the faction may
    ///   inflict weather while it holds the unit. **No pass reads this
    ///   column yet.**
    /// - `water_crossing`. A whole count. Nonzero means the unit may stand on
    ///   a water tile, and the terrain table states how many such units one
    ///   water tile holds. Zero refuses the unit at the shoreline.
    /// - `settle_group`. A whole count. Zero means the type founds no city,
    ///   and `order_settle` refuses the unit. A count above zero is the
    ///   group the founding seats at the new city.
    ///
    /// **An attacker whose attack does not exceed the defender's armour
    /// contributes exactly zero, however many attackers stand there.** The
    /// engine applies that test for each attacker type before it adds
    /// anything. No number of weak attackers therefore reaches a strong
    /// defender.[^2]
    ///
    /// The call changes the table and moves nothing. Step the world to make
    /// two factions on one tile resolve their meeting. Then read
    /// `faction_population` for what it cost.
    ///
    /// Returns `None`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no row of the table. Raises
    /// `VerbError` when a fixed-point column is below zero. Raises
    /// `OverflowError` when a whole-count column is below zero or above the
    /// range of a 32-bit unsigned integer.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
    /// [^3]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D2 and D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    #[pyo3(signature = (
        unit_type,
        attack,
        armour,
        *,
        gather_rate,
        build_rate,
        carry_capacity,
        move_cost_scale,
        command_reach,
        weather_reach,
        water_crossing,
        settle_group,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn define_unit_type(
        &self,
        unit_type: u8,
        attack: i32,
        armour: i32,
        gather_rate: i32,
        build_rate: i32,
        carry_capacity: u32,
        move_cost_scale: i32,
        command_reach: u32,
        weather_reach: u32,
        water_crossing: u32,
        settle_group: u32,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let row = UnitTypeRow {
            attack: Fix32(attack),
            armour: Fix32(armour),
            gather_rate: Fix32(gather_rate),
            build_rate: Fix32(build_rate),
            carry_capacity,
            move_cost_scale: Fix32(move_cost_scale),
            command_reach,
            weather_reach,
            water_crossing,
            settle_group,
        };
        world
            .define_unit_type(unit_type, row)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the shared unit type table, as a `dict` of NumPy arrays.
    ///
    /// A unit type is a row of this table. A soldier carries the row number
    /// alone. The table is data that the world holds, and it is not code.[^1]
    ///
    /// Every array holds one entry for each row, and every array is the
    /// same length. **That length is the number of types the world holds.**
    /// Nothing else states the width. A caller reads the width from this
    /// return value, not from a second number that could disagree.[^3]
    ///
    /// The keys are the column names of the row, in the order the engine
    /// declares them. Each value is a `numpy.int64` array. The keys and the
    /// keyword arguments of `define_unit_type` are the same names, and each
    /// entry carries the value that call took: a fixed-point column keeps
    /// its raw Q16.16 value, so one whole casualty is 65536, and a whole
    /// count column keeps its count.
    ///
    /// **The width of the table is fixed, and the values are configurable.**
    /// The world builds the table with the default rows: a worker, a
    /// soldier, a merchant, a leader, one open row, and zero above them.
    /// `define_unit_type` writes one row. A row whose every column is zero
    /// is a unit that can do nothing: it reaches nothing, nothing reaches
    /// it, and it gathers, builds and carries nothing.[^1] [^5]
    ///
    /// The values are content. A record may not hold a number that a content
    /// choice can move, so no record holds one.[^4]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^4]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
    /// [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D2 and D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn unit_type_table<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let rows = world.unit_types().rows();
        let columns = PyDict::new(python);
        // The names and the values both come from the row declaration, so
        // the dictionary cannot name a column the row does not hold.
        for (index, name) in UnitTypeRow::COLUMN_NAMES.iter().enumerate() {
            let column: Vec<i64> = rows.iter().map(|row| row.columns()[index]).collect();
            columns.set_item(name, column.to_pyarray(python))?;
        }
        Ok(columns)
    }

    /// Returns the upgrade table, as a `dict` of NumPy arrays.
    ///
    /// The table holds one row for each pair of a category and a level. A row
    /// names the ground it fits, the work it takes and what it changes.[^1]
    ///
    /// Each key is a column name and each value is a `numpy.int64` array with
    /// one entry for each row. The rows run by category and then by level, so
    /// the entry of a category at a level sits at the category number times
    /// the level count, plus the level, minus one.
    ///
    /// The `ground_fit` column holds one bit for each ground kind, at the bit
    /// the ground number names. A `ground_fit` of zero says that the table
    /// holds no row there.
    ///
    /// **The names come from the Rust row declaration.** A test asserts that
    /// the type stub names the same columns, so no second list can rot.[^2]
    ///
    /// This method copies each column.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn upgrade_table<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let rows = world.upgrade_table().rows();
        let columns = PyDict::new(python);
        // The names and the values both come from the row declaration, so
        // the dictionary cannot name a column the row does not hold.
        for (index, name) in UpgradeRow::COLUMN_NAMES.iter().enumerate() {
            let column: Vec<i64> = rows.iter().map(|row| row.columns()[index]).collect();
            columns.set_item(name, column.to_pyarray(python))?;
        }
        Ok(columns)
    }

    /// Writes one row of the upgrade table.
    ///
    /// The category is a row group, as a Python integer. The level is one or
    /// two. The caller gives every column. There is no partial form, because
    /// a caller that gave two columns would leave the rest at zero and would
    /// define an upgrade that changes nothing else without knowing it.[^1]
    ///
    /// A `ground_fit` of zero removes the row. A build order that names a
    /// category with no row at the next level is refused.
    ///
    /// **The values are content and not a budget.** A game sets them, and no
    /// record holds one.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no category, and when the
    /// level is not one the table holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
    #[pyo3(signature = (
        category,
        level,
        *,
        ground_fit,
        work,
        yield_change,
        recovery_change,
        capacity_change,
        capacity_of_store_change,
        housing_change,
        victory_claim,
        own_ground_required,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn define_upgrade_row(
        &self,
        category: u8,
        level: u8,
        ground_fit: u32,
        work: u32,
        yield_change: u32,
        recovery_change: u32,
        capacity_change: u32,
        capacity_of_store_change: u32,
        housing_change: u32,
        victory_claim: u32,
        own_ground_required: u32,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let row = UpgradeRow {
            ground_fit,
            work,
            yield_change,
            recovery_change,
            capacity_change,
            capacity_of_store_change,
            housing_change,
            victory_claim,
            own_ground_required,
        };
        world
            .define_upgrade_row(category, level, row)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Puts one entry at the back of the build queue of one site.
    ///
    /// The faction is a faction number of this world. The site is a
    /// settlement identity, as `found_settlements` returns them. The unit
    /// type is a row of the shared unit type table, as an integer. Returns
    /// `None`.
    ///
    /// **A site builds a typed unit from its queue, and the store pays.** One
    /// stage advances the front entry of each site. A finished entry takes
    /// the residents its cost row names and the goods it costs, and one unit
    /// of that type then stands at the site.[^1]
    ///
    /// **The engine holds no rule about what to queue.** This verb, the
    /// built-in controller and a learner all reach the same path, and none of
    /// them is physics.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the identity names no site that stands, when
    /// the site belongs to another faction, when the number names no row of
    /// the unit type table, and when the queue already holds its bound.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    fn queue_unit(&self, faction: u16, site: u64, unit_type: u8) -> PyResult<()> {
        let mut world = self.lock();
        let entity = resolve_site(&world, site)?;
        let kind = UnitTypeId::from_u8(unit_type)
            .ok_or_else(|| VerbError::new_err(format!("{unit_type} names no unit type")))?;
        world
            .order_site_queue(FactionId(faction), entity, QueueOrder::Push(kind))
            .map_err(|refusal| VerbError::new_err(refusal.to_string()))
    }

    /// Takes one entry out of the build queue of one site.
    ///
    /// The faction is a faction number of this world. The site is a
    /// settlement identity. The position is the place of the entry in the
    /// queue, counting from zero. The entries behind it keep their order.
    /// Returns `None`.
    ///
    /// **The work the store already paid for is lost.** The store paid it as
    /// the entry advanced, and nothing returns a charge.[^1]
    ///
    /// This call and `queue_unit` reach one verb of the engine.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the identity names no site that stands, when
    /// the site belongs to another faction, and when the position holds no
    /// entry.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    fn clear_queue_entry(&self, faction: u16, site: u64, position: u8) -> PyResult<()> {
        let mut world = self.lock();
        let entity = resolve_site(&world, site)?;
        world
            .order_site_queue(FactionId(faction), entity, QueueOrder::Clear(position))
            .map_err(|refusal| VerbError::new_err(refusal.to_string()))
    }

    /// Returns the build queue of one site, as a `dict` of NumPy arrays.
    ///
    /// The site is a settlement identity. Every array holds one entry for
    /// each entry of the queue, in queue position order, and every array is
    /// the same length. **That length is the entries the site holds.**
    ///
    /// - `unit_type`, `numpy.uint8`. The row of the unit type table that the
    ///   entry names.
    /// - `work`, `numpy.uint32`. The work done toward that type.
    ///
    /// The order is the order the entries were queued, and nothing reorders
    /// them. The front entry is the one that advances.[^1]
    ///
    /// This method copies each column.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no site that stands.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn site_queue<'py>(&self, python: Python<'py>, site: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let entries = world
            .site_queue(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let columns = PyDict::new(python);
        let unit_type: Vec<u8> = entries.iter().map(|entry| entry.unit_type.0).collect();
        let work: Vec<u32> = entries.iter().map(|entry| entry.work).collect();
        columns.set_item("unit_type", unit_type.to_pyarray(python))?;
        columns.set_item("work", work.to_pyarray(python))?;
        Ok(columns)
    }

    /// Writes the build cost of one unit type.
    ///
    /// The unit type is a row of the shared unit type table, as an integer.
    /// The work is the advances the entry takes. The people are the residents
    /// a finished entry spends. The goods are one quantity for each
    /// commodity, in commodity order, as raw Q16.16 integers. Returns `None`.
    ///
    /// **The costs sit in their own table and not in the unit type row.** A
    /// unit type row is a set of capability columns, and a zero in one means
    /// that the type cannot do what the column names.[^1] A build work of
    /// zero means a type that finishes at once, which is a different meaning
    /// for one zero.
    ///
    /// Every value is a balance row, and one blocker governs all of them.[^2]
    /// [^3]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no row of the unit type
    /// table, and when the goods list is not one quantity for each commodity.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Balance register, the production queue. `docs/reference/balance.md`
    /// [^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    fn define_build_cost(
        &self,
        unit_type: u8,
        work: u32,
        people: u32,
        goods: Vec<i32>,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let mut quantities = [Fix32::ZERO; COMMODITY_COUNT];
        if goods.len() != COMMODITY_COUNT {
            return Err(VerbError::new_err(format!(
                "the goods list holds {} quantities and the world holds {COMMODITY_COUNT} commodities",
                goods.len()
            )));
        }
        for (slot, value) in quantities.iter_mut().zip(goods) {
            *slot = Fix32(value);
        }
        world
            .define_build_cost(
                unit_type,
                BuildCostRow {
                    work,
                    people,
                    goods: quantities,
                },
            )
            .map_err(|refusal| VerbError::new_err(refusal.to_string()))
    }

    /// Returns the entries one site may hold in its queue, as an integer.
    fn queue_bound(&self) -> usize {
        self.lock().queue_bound()
    }

    /// Sets the entries one site may hold in its queue.
    ///
    /// **A bound of zero turns the queue off.** No site then holds an entry,
    /// and no unit is built. A caller that wants the engine to leave its
    /// units alone sets it. Returns `None`.
    ///
    /// The bound is a balance row.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the bound is above the width of the stored
    /// block.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the production queue, the queue bound row. `docs/reference/balance.md`
    fn set_queue_bound(&self, bound: usize) -> PyResult<()> {
        if self.lock().set_queue_bound(bound) {
            return Ok(());
        }
        Err(VerbError::new_err(format!(
            "the bound {bound} is above the width of the stored block"
        )))
    }

    /// Sets how often the queue advance acts, and its offset in the period.
    ///
    /// The period is the ticks between two advances. The phase is the offset
    /// inside the period. Returns `None`.
    ///
    /// The schedule is a balance row.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero or above the range.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the production queue, the schedule row. `docs/reference/balance.md`
    fn set_queue_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        let schedule = RateSchedule::new(period, phase).ok_or_else(|| {
            VerbError::new_err(format!("the period {period} is outside the range"))
        })?;
        self.lock().set_queue_schedule(schedule);
        Ok(())
    }

    /// Sets the quantity of one good that one advance of a queue costs.
    ///
    /// The commodity is a commodity number. The quantity is a raw Q16.16
    /// integer. Returns `None`.
    ///
    /// **A queue is never free.** A site whose store cannot pay the charge
    /// makes no progress, and its entry stays where it is.[^1] The charge is
    /// a balance row.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the commodity is outside the set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: Balance register, the production queue, the charge row. `docs/reference/balance.md`
    fn set_queue_charge(&self, commodity: u16, quantity: i32) -> PyResult<()> {
        if self
            .lock()
            .set_queue_charge(CommodityId(commodity), Fix32(quantity))
        {
            return Ok(());
        }
        Err(VerbError::new_err(format!(
            "the commodity {commodity} is outside the set"
        )))
    }

    /// Tells every soldier the identities name to build one category of
    /// upgrade.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The category is a row group of the upgrade table, as an integer. A
    /// road is zero, a terrace is one, a wonder is two, a store is three, a
    /// wall is four and a lodging is five. The argument has no default. A road lets more units
    /// stand on the tile. A terrace lets a unit take more from the tile in
    /// one step. A wonder asks for a large amount of work, and its completion
    /// wins the game for the faction that holds the ground under it.[^6] A
    /// store raises the store capacity of a settlement on or beside its tile,
    /// and **nothing in the engine reads that raise today**. Read
    /// `site_economy` for the sum. A lodging raises the housing of a
    /// settlement on or beside its tile, so the site holds more people. Read
    /// `site_housing` for what stands.
    ///
    /// **The order names no level.** The engine reads the ground under each
    /// tile and the level that stands there, and it resolves the row of the
    /// table for the next level. It refuses a tile that no row fits.[^7] Read
    /// `upgrade_table` for the ground each row fits and the work it takes.
    ///
    /// Each soldier adds to the upgrade on the tile it stands on, at every
    /// step, until something stops it. A soldier does not have to stay. A
    /// soldier that walks away stops adding, and the work it did stays on the
    /// tile.[^1] Several soldiers on one tile add to one total. That total
    /// is the same at every thread count.[^2]
    ///
    /// **An unfinished build changes nothing about the tile.** The tile
    /// changes when the work reaches the amount its kind asks for.[^1] Read
    /// `tile_report` for the work done so far.
    ///
    /// The call gives the order and builds nothing. Step the world to make
    /// the soldiers build.
    ///
    /// **The category here is an upgrade category. It is not a resource kind
    /// and it is not a ground kind.** More than one scale in this module
    /// starts at zero. The call accepts every resource kind, and each of
    /// those numbers also names an upgrade category. It raises nothing, and
    /// the soldiers build the wrong thing. The engine sees a number and not
    /// the scale the caller meant.[^3]
    ///
    /// **The engine does not check who holds the ground.** A soldier builds
    /// on the tile it stands on, whatever faction holds that tile. A caller
    /// that wants the other rule holds it in Python. A finding records the
    /// gap.[^4]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind
    /// is checked, before the engine gives any order.
    ///
    /// **A unit whose type cannot build refuses the order.** The build rate
    /// of the row the unit indexes is zero, so the unit would keep the order
    /// and add nothing on every step. The verb refuses instead, so the
    /// caller learns at once. Read `unit_type_table` for the rate of each
    /// row.[^5]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no category of the table. The
    /// message names the number that refused. Raises `VerbError` when a
    /// unit's type has a build rate of zero. Raises `VerbError` when the
    /// engine refuses an order, and the message states the refusal.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D2 and D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: Findings register, FND-352. `docs/FINDINGS.md`
    /// [^4]: Findings register, FND-380. `docs/FINDINGS.md`
    /// [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^6]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^7]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    fn order_build(&self, units: Vec<u64>, category: u8) -> PyResult<()> {
        let mut world = self.lock();
        let category = UpgradeCategory::from_u8(category)
            .ok_or_else(|| VerbError::new_err(format!("{category} names no upgrade category")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            let entity = resolve(&world, *unit)?;
            let row = world
                .unit_type_row(entity)
                .expect("a resolved identity names a live soldier");
            if row.build_rate == Fix32::ZERO {
                return Err(VerbError::new_err(format!(
                    "the unit {unit} is of a type whose build rate is zero, and it cannot build"
                )));
            }
            resolved.push(entity);
        }
        // The set form is the one path the controller takes too, so one loop
        // serves both callers.
        let (refused, reason) = world.order_build_set_reporting(&resolved, category);
        // The engine refuses a ground the row does not fit, a category at its
        // top, a tile that carries another category, and a build outside the
        // builder's own ground.[^8] The verb answers with a count and the
        // first refusal, and the boundary turns those into an error rather
        // than a silent partial order.
        //
        // [^8]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        if refused > 0 {
            let reason = reason.map_or_else(String::new, |refusal| refusal.to_string());
            return Err(VerbError::new_err(format!(
                "the world refused {refused} of {} build orders: {reason}",
                resolved.len()
            )));
        }
        Ok(())
    }

    /// Tells every soldier the identities name to stop building.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The work each soldier already did stays on its tile. A soldier that
    /// takes the order again continues rather than restarts.[^1] Nothing here
    /// removes an upgrade: call `destroy_upgrades` for that.
    ///
    /// A soldier that builds nothing takes this order and stays as it was.
    ///
    /// **The set is all or nothing.** Every identity resolves before the
    /// engine stops any order.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    fn stop_build(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for entity in resolved {
            assert!(
                world.stop_build(entity),
                "a resolved identity must name a soldier the arena can order"
            );
        }
        Ok(())
    }

    /// Returns what one soldier builds, as an integer, or `None`.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned.
    ///
    /// The result is the upgrade category that `order_build` took: a road is
    /// zero, a terrace is one, a wonder is two, a store is three, a wall is
    /// four and a lodging is five. The result is `None` when the soldier
    /// builds nothing.
    ///
    /// **This read stays singular while the write verbs take a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. The second is a false
    /// answer the record forbids.[^1] The read therefore answers for one
    /// identity and says which one failed.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn build_order(&self, unit: u64) -> PyResult<Option<u8>> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let order = world.build_order(entity).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} names no live soldier"))
        })?;
        Ok(order.map(UpgradeCategory::to_u8))
    }

    /// Removes the upgrade at each address and returns how many it removed.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. Returns an
    /// integer, which counts the tiles that carried an upgrade.
    ///
    /// Each tile returns to the world that the generator made. Nothing else
    /// stores a property of an improved tile. Removing the entry is the whole
    /// of the return.[^1] The removal takes effect at once, and it needs no
    /// step.
    ///
    /// The call removes a finished upgrade and an unfinished one alike. An
    /// unfinished upgrade loses the work that went into it.
    ///
    /// **An address that carries no upgrade is not a refusal.** The engine
    /// removes nothing there and does not count it. Two calls for one address
    /// therefore count one removal and then none.
    ///
    /// **The call removes no build order.** A soldier that stands on the tile
    /// and holds an order starts the upgrade again at the next step. Call
    /// `stop_build` for that soldier first.
    ///
    /// **The engine does not check who holds the ground.** Any caller may
    /// remove any upgrade, and the removal is instant.
    ///
    /// **The set is all or nothing.** Every address is checked against the
    /// world before the engine removes anything.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world. The message
    /// names the address that refused.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    fn destroy_upgrades(&self, addresses: Vec<(i32, i32)>) -> PyResult<usize> {
        let mut world = self.lock();
        for (q, r) in &addresses {
            if world.tile_kind(Axial::new(*q, *r)).is_none() {
                return Err(ViewError::new_err(format!(
                    "({q}, {r}) lies outside this world"
                )));
            }
        }
        let mut removed = 0;
        for (q, r) in addresses {
            if world.destroy_upgrade(Axial::new(q, r)) {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Zones one project for one faction at each address.
    ///
    /// The faction is a faction number of this world. The addresses are a
    /// sequence of `(q, r)` pairs of integers. The category is an upgrade
    /// category, as an integer: a road is zero, a terrace is one, a wonder is
    /// two, a store is three, a wall is four and a lodging is five. Returns
    /// `None`.
    ///
    /// **A plan says where a unit may build.** A category whose row asks for
    /// no held ground, such as a road, is laid only inside a project. That is
    /// how a faction reaches ground it does not yet hold, and the plan is the
    /// bound that stops it spreading everywhere.[^1] [^2]
    ///
    /// The solver of the engine writes into the same list, and a unit cannot
    /// tell a project a caller wrote from one the solver wrote.[^3]
    ///
    /// **The set is all or nothing.** Every address resolves before the plan
    /// takes any project, and the first refusal fails the whole call.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world. Raises
    /// `VerbError` when the number names no category, when the plan is full,
    /// when no row of the table fits the ground, and when the row asks for
    /// held ground that the faction does not hold.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn zone_projects(
        &self,
        faction: u16,
        addresses: Vec<(i32, i32)>,
        category: u8,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let category = UpgradeCategory::from_u8(category)
            .ok_or_else(|| VerbError::new_err(format!("{category} names no upgrade category")))?;
        for (q, r) in &addresses {
            if world.tile_kind(Axial::new(*q, *r)).is_none() {
                return Err(ViewError::new_err(format!(
                    "({q}, {r}) lies outside this world"
                )));
            }
        }
        for (q, r) in &addresses {
            if let Err(refusal) =
                world.zone_project(FactionId(faction), Axial::new(*q, *r), category)
            {
                return Err(VerbError::new_err(format!(
                    "the world refused a project at ({q}, {r}): {refusal:?}"
                )));
            }
        }
        Ok(())
    }

    /// Sets the values the plan and its solver read.
    ///
    /// Every argument is a whole number. The bound is the most projects one
    /// faction may hold. The solver passes are the passes the solver makes
    /// over the reads of one faction. The projects per pass are the most
    /// projects one pass writes. The path passes are the passes the path
    /// search makes over its window. The radius is the hex steps a path may
    /// span. Returns `None`.
    ///
    /// **The call clears every plan.** The bound decides the size of the
    /// register, so a register built with another bound holds its rows
    /// elsewhere. A caller that changes the rules zones again.
    ///
    /// **A bound of zero turns the plan off.** The plan then takes no
    /// project, the solver writes nothing, and no unit is sent to one. A test
    /// that wants the engine to leave its units alone sets it.
    ///
    /// Every value is a balance row, and one blocker governs all of them.[^1]
    /// [^2]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the plan. `docs/reference/balance.md`
    /// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    fn set_plan_rules(
        &self,
        bound: u32,
        solver_passes: u32,
        projects_per_pass: u32,
        path_passes: u32,
        radius: u32,
    ) {
        let mut world = self.lock();
        world.set_plan_rules(PlanRules::new(
            bound,
            solver_passes,
            projects_per_pass,
            path_passes,
            radius,
        ));
    }

    /// Removes the project of one faction at each address.
    ///
    /// The faction is a faction number of this world. The addresses are a
    /// sequence of `(q, r)` pairs of integers. Returns an integer, which
    /// counts the addresses that carried a project.
    ///
    /// The solver and a caller both reach this verb, so a caller may clear a
    /// project the solver wrote.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    fn clear_projects(&self, faction: u16, addresses: Vec<(i32, i32)>) -> PyResult<usize> {
        let mut world = self.lock();
        for (q, r) in &addresses {
            if world.tile_kind(Axial::new(*q, *r)).is_none() {
                return Err(ViewError::new_err(format!(
                    "({q}, {r}) lies outside this world"
                )));
            }
        }
        let mut cleared = 0;
        for (q, r) in addresses {
            if world.clear_project(FactionId(faction), Axial::new(q, r)) {
                cleared += 1;
            }
        }
        Ok(cleared)
    }

    /// Returns the plan of one faction, in ascending tile order.
    ///
    /// The faction is a faction number of this world. The result is a list of
    /// `(q, r, category)` triples of integers. The list is empty when the
    /// faction has zoned nothing.
    ///
    /// The order is the tile order and never the order a caller wrote in, so
    /// two runs that zoned the same tiles read one list.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    fn plan(&self, faction: u16) -> Vec<(i32, i32, u8)> {
        let world = self.lock();
        world
            .plan_of(FactionId(faction))
            .iter()
            .filter_map(|project| {
                let address = world.grid().address_of(project.tile)?;
                Some((address.q, address.r, project.category.to_u8()))
            })
            .collect()
    }

    /// Returns the category one faction zoned at one address, or `None`.
    ///
    /// The faction is a faction number of this world. The address is the pair
    /// `q` and `r`, as integers. The result is an upgrade category, as an
    /// integer, or `None` when the faction zoned nothing there.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    fn project_at(&self, faction: u16, q: i32, r: i32) -> PyResult<Option<u8>> {
        let world = self.lock();
        if world.tile_kind(Axial::new(q, r)).is_none() {
            return Err(ViewError::new_err(format!(
                "({q}, {r}) lies outside this world"
            )));
        }
        Ok(world
            .project_at(FactionId(faction), Axial::new(q, r))
            .map(UpgradeCategory::to_u8))
    }
}
