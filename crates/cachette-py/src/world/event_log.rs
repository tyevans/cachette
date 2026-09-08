//! The readers that hand a step's events to Python.
//!
//! This module holds the raw event log, the whole-log column report, the named
//! log readers and the count of each log.
//!
//! The grouping is by shape rather than by subject. Every method here answers
//! with the columns of a log, so a caller that reads one reads them all the
//! same way. A reader that answers about the world rather than about the last
//! step lives elsewhere.

use super::PyWorld;
use crate::columns::columns_of;
use crate::logs::{log_names, log_of, unknown_log_message};
use cachette_core::campaign::CampaignEvent;
use cachette_core::Axial;
use cachette_core::TileIdx;
use numpy::ToPyArray;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Returns the tile change log of the last step as a `bytes` object.
    ///
    /// The bytes are the event records of the last step, one after another,
    /// in ascending tile order. Each record holds the tick, the tile, the
    /// value, the holder, the change kind and its declared padding.
    ///
    /// **A caller that reads a field out of these bytes holds a copy of the
    /// record layout. Nothing fails when the layout changes.** Call
    /// `event_log_columns` instead. It gives the same events as arrays, by
    /// field name.
    ///
    /// This method exists for a caller that stores or ships the log without
    /// reading it.
    fn event_log_bytes<'py>(&self, python: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(python, self.lock().event_log_bytes())
    }

    /// Returns the tile change log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// Every array has one entry for each event, and all five arrays are the
    /// same length. That length is `event_count`. A new world gives five
    /// empty arrays.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the change happened.
    /// - `tile`, `numpy.uint32`. The tile that changed, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    /// - `value`, `numpy.int32`. The tile value after the change. **This is a
    ///   Q16.16 fixed-point value as its raw integer. Divide by 65536.**[^2]
    /// - `holder`, `numpy.uint16`. The faction that holds the tile, as the
    ///   step left it. The value 65535 means that nobody holds it. It sits
    ///   above the faction ceiling, so no faction collides with it.[^4]
    /// - `kind`, `numpy.uint8`. The kind of change. One means that the value
    ///   rose, and two means that it fell.
    ///
    /// The keys are the field names of the event. The caller reads a field
    /// by its name. No caller holds a byte offset, a field width, or a field
    /// order. Those live in the Rust source and nowhere else.[^1]
    ///
    /// The engine declares the fields of the event in one place, and this
    /// method builds the columns from that declaration. It names no field of
    /// its own. Read the declaration with `event_schema`.[^5]
    ///
    /// This method copies each column. The log of one step is small next to
    /// the world.[^3]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-060. `docs/DECISIONS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^5]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
    fn event_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.event_log())
    }

    /// Returns the name of every event log, as a `list` of `str`.
    ///
    /// **This is how a caller finds out what the engine publishes.** A log
    /// added to the engine appears here with no new method on this class, no
    /// new entry in the type stub and no new name for a caller to learn. Hand
    /// any name from this list to `log` and to `log_count`.
    ///
    /// The name of a log is the name its event declares, and `event_schema`
    /// gives the columns of every one of them under the same names.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
    fn log_names(&self) -> Vec<&'static str> {
        log_names()
    }

    /// Returns one event log, as a `dict` of NumPy arrays.
    ///
    /// The name is one of the names `log_names` gives. The keys are the
    /// column names the event declares, and `event_schema` states them and
    /// their element types. Every array has one entry for each record, and
    /// all of them are the same length. That length is `log_count` of the
    /// same name.
    ///
    /// **A log holds what happened since the last step began.** The step
    /// clears it before any system runs, so a caller reads it after each step
    /// and a caller that misses a step misses the events. This is the rule for
    /// every log, and it is not the rule for `subsystem_census`, whose rows
    /// are marked as a running total or as a count of what stands.
    ///
    /// No element type is a floating point type. A fixed-point column crosses
    /// as its raw integer.[^2]
    ///
    /// This method copies each column. The log of one step is small next to
    /// the world.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ValueError` when no log has the given name. The message lists
    /// every name.
    ///
    /// # References
    ///
    /// [^1]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn log<'py>(&self, python: Python<'py>, name: &str) -> PyResult<Bound<'py, PyDict>> {
        let entry = log_of(name).ok_or_else(|| PyValueError::new_err(unknown_log_message(name)))?;
        let world = self.lock();
        (entry.read)(python, &world)
    }

    /// Returns how many records one event log holds, as an integer.
    ///
    /// The name is one of the names `log_names` gives. The count covers the
    /// events since the last step began, in the same way `log` does.
    ///
    /// # Errors
    ///
    /// Raises `ValueError` when no log has the given name.
    fn log_count(&self, name: &str) -> PyResult<usize> {
        let entry = log_of(name).ok_or_else(|| PyValueError::new_err(unknown_log_message(name)))?;
        let world = self.lock();
        Ok((entry.count)(&world))
    }

    /// Returns the gather log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A gather event says that one unit took an amount of one resource from
    /// one tile. Every array has one entry for each event, and all five
    /// arrays are the same length. That length is `gather_count`.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the unit took the amount.
    /// - `unit`, `numpy.uint64`. The identity of the unit that took it.
    /// - `tile`, `numpy.uint32`. The tile it took from, as a row-major index.
    /// - `amount`, `numpy.uint32`. How much it took. This is a whole number
    ///   of units of stock, and it is not fixed point.
    /// - `kind`, `numpy.uint8`. The resource kind. Food is zero, wood is one
    ///   and stone is two.
    ///
    /// The unit column holds the whole identity of the unit that took the
    /// amount. It is not a slot index. A slot index survives the death of
    /// what it named. A reader that held one would report on the next
    /// occupant of the slot, with nothing failing.[^1]
    ///
    /// Hand a value from this column back to `soldier_tile` to read the
    /// unit. The engine resolves it, and it refuses a dead one.[^1]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn gather_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.gather_log())
    }

    /// The number of gather events the last step emitted, as an integer.
    ///
    /// The count covers the last step alone. A new world reports zero. Read
    /// the events themselves with `gather_log_columns`.
    #[getter]
    fn gather_count(&self) -> usize {
        self.lock().gather_log().len()
    }

    /// Returns the fallen log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A fallen event says that one unit fell in a meeting between two
    /// factions. Every array has one entry for each event, and all five
    /// arrays are the same length. That length is `fell_count`. A new world
    /// gives five empty arrays.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the unit fell.
    /// - `unit`, `numpy.uint64`. The identity of the unit that fell.
    /// - `tile`, `numpy.uint32`. The tile it stood on, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    /// - `faction`, `numpy.uint16`. The faction the unit belonged to. Every
    ///   entry names a faction of this world, because a unit always holds
    ///   one.
    /// - `unit_type`, `numpy.uint8`. The row of the shared type table that
    ///   the unit carried. It is the number `define_unit_type` writes.
    ///
    /// The keys are the field names of the event. The caller reads a field by
    /// its name, so no caller holds a byte offset, a field width or a field
    /// order.[^3]
    ///
    /// The entries come in ascending slot order, which is the order the step
    /// ended the units in. That order does not depend on the thread
    /// count.[^4]
    ///
    /// **The log names no killer.** The engine resolves a meeting for a whole
    /// group of units at one tile. No single attacker owns one death.[^5]
    /// The log says who fell, at which step, where it stood, and which
    /// faction and type it carried. It does not name the enemy. The caller
    /// reads the enemy from the tile and the step.
    ///
    /// **The log covers the last step alone, and the next step destroys it.**
    /// The step empties the log before it resolves a meeting. A step with no
    /// fight gives five empty arrays, and never the entries of an earlier
    /// step.[^6] Read the log after each `step` call whose deaths the caller
    /// wants, and keep what it needs. Every other log here holds the same
    /// rule.
    ///
    /// The unit column holds the whole identity of the unit that fell. It is
    /// not a slot index. A slot index survives the death of what it named.
    /// A reader that held one would report on the next occupant of the slot,
    /// with nothing failing.[^1]
    ///
    /// **Every identity in this column is dead**, because the step ended the
    /// unit that it names. `soldier_tile` refuses a dead identity. The
    /// `tile` column carries the ground the unit stood on. The caller needs
    /// no second read to place the death.[^1]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^3]: Decisions register, DEC-060. `docs/DECISIONS.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0123, casualties are whole units served to a keyed subset, decision D1. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
    /// [^6]: ADR-0121, a meeting between two factions resolves at the tile, decision D4. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
    fn fell_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.fell_log())
    }

    /// The number of units that fell in the last step, as an integer.
    ///
    /// The count covers the last step alone. A new world reports zero. Read
    /// the events themselves with `fell_log_columns`.
    #[getter]
    fn fell_count(&self) -> usize {
        self.lock().fell_log().len()
    }

    /// Returns the starved log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A starved event says that a shortage ended one unit. The engine writes
    /// one entry for each unit that the scan of this step removed, in
    /// ascending slot order.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The consumption pass runs on a schedule. The scan runs with it. On a
    /// step the schedule does not name, and on a step that ended nobody, the
    /// log is empty. A reader cannot tell the two cases apart, and nothing
    /// needs to.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the scan ended the unit.
    /// - `unit`, `numpy.uint64`. The identity of the unit that ended. It is
    ///   not a slot index. It never resolves again, because the unit is
    ///   dead.[^1]
    /// - `deficit`, `numpy.int32`. What the unit went short by. The value
    ///   carries the Q16.16 fixed-point scale, so 65536 is one whole unit of
    ///   need. It is at or above the bound that ends a unit.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn starved_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.starved_log())
    }

    /// Returns the shortfall log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A shortfall event says that one site could not pay its upkeep. The
    /// store stopped at zero rather than going below it. The amount is what
    /// the world must supply to make the site solvent.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The rate pass runs on a schedule. On a step the schedule does not
    /// name, and on a step in which every site paid, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the upkeep applied.
    /// - `site`, `numpy.uint64`. The identity of the settlement that could
    ///   not pay. It is not a slot index. Hand it back to
    ///   `site_economy`.[^1]
    /// - `amount`, `numpy.int32`. What the upkeep could not take. The value
    ///   carries the Q16.16 fixed-point scale, so 65536 is one whole unit of
    ///   the commodity. It is never zero.
    /// - `commodity`, `numpy.uint16`. The commodity that the site owed. A
    ///   commodity is not a resource kind. The world holds one commodity
    ///   today, and its number is zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn shortfall_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.shortfall_log())
    }

    /// Returns the rationed log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A rationed event says that one site could not serve every cohort that
    /// drew on it. A cohort is the group of units of one faction that draw
    /// from one site. The store stopped at zero rather than going below it.
    /// The granted amount is always below the demanded amount.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The consumption pass runs on a schedule. On a step the schedule does
    /// not name, the log is empty. On a step in which every site served every
    /// cohort, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the draw ran.
    /// - `site`, `numpy.uint64`. The identity of the settlement that could
    ///   not serve. It is not a slot index. Hand it back to
    ///   `site_economy`.[^1]
    /// - `demanded`, `numpy.int64`. What the cohorts of the site asked for.
    ///   The value carries the Q16.16 fixed-point scale, so 65536 is one
    ///   whole unit of the commodity.
    /// - `granted`, `numpy.int64`. What the store gave, in the same Q16.16
    ///   scale. It is always below the demanded amount.
    /// - `commodity`, `numpy.uint16`. The commodity that the cohorts drew. A
    ///   commodity is not a resource kind. The world holds one commodity
    ///   today, and its number is zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn rationed_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.rationed_log())
    }

    /// Returns the promotion log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A promotion event says that one soldier became a character. The engine
    /// writes one entry for each soldier the pass promoted, in rank order,
    /// with the highest deeds first.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The promotion pass runs on a schedule. On a step the schedule does not
    /// name, and on a step that promoted nobody, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the pass promoted the
    ///   soldier.
    /// - `unit`, `numpy.uint64`. The identity of the soldier. It is not a
    ///   slot index. The soldier stays alive, so `soldier_tile` answers for
    ///   it.[^1]
    /// - `character`, `numpy.uint64`. The identity of the character that the
    ///   promotion created. It is not a slot index.[^1]
    /// - `deeds`, `numpy.uint64`. What the soldier gathered, as a running
    ///   total. It is a whole number of units of stock. It carries no
    ///   fixed-point scale.
    /// - `faction`, `numpy.uint16`. The faction of the soldier and of the
    ///   character.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn promoted_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.promoted_log())
    }

    /// Returns the units that changed faction in the last step, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays. Every array
    /// holds one entry for each unit that changed faction. The keys are:
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `unit`, `numpy.uint64`. The identity of the unit. It is the identity
    ///   the unit had before, because a unit that changes faction keeps its
    ///   identity. Hand it back to `soldier_tile` or to `convert_units`.
    /// - `tile`, `numpy.uint32`. The tile the unit stood on, as a row-major
    ///   index. Take `index % world.width` for the column and
    ///   `index // world.width` for the row.
    /// - `from_faction`, `numpy.uint16`. The faction that lost the unit.
    /// - `to_faction`, `numpy.uint16`. The faction that gained it.
    ///
    /// **The log covers the last step alone.** The engine delivers it at the
    /// frame barrier.[^1] Read it after each `step`. The next step clears it.
    ///
    /// The log holds the units the engine converted and the units that
    /// `convert_units` converted. Both are the same change, so one log
    /// reports both.
    ///
    /// A step in which nobody changed faction gives arrays of length zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// ```python
    /// world.step(4)
    /// changed = world.converted_log_columns()
    /// gained = int((changed["to_faction"] == 0).sum())
    /// ```
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn converted_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.converted_log())
    }

    /// The number of units that changed faction in the last step, as an
    /// integer.
    ///
    /// The count covers the last step alone. Read the changes themselves with
    /// `converted_log_columns`.
    #[getter]
    fn converted_count(&self) -> usize {
        self.lock().converted_log().len()
    }

    /// Returns the crossings of the war edge on the last step, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays. Every array
    /// holds one entry for each ordered pair whose relation crossed the war
    /// edge on the last step, in the order the crossings happened. The keys
    /// are:
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `from_faction`, `numpy.uint16`. The faction whose feeling moved.
    /// - `to_faction`, `numpy.uint16`. The faction it feels toward.
    /// - `band_before`, `numpy.uint8`. The band number before the move.
    /// - `band_after`, `numpy.uint8`. The band number after the move. A
    ///   value below `band_before` is a declaration, and a value above it is
    ///   a peace.
    ///
    /// The log covers the last step alone. This method copies each
    /// column.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D6. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn relation_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.relation_log())
    }

    /// The number of relations that crossed the war edge in the last step,
    /// as an integer. Read the events themselves with
    /// `relation_log_columns`.
    #[getter]
    fn relation_crossed_count(&self) -> usize {
        self.lock().relation_log().len()
    }

    /// Returns what happened to the campaigns on the last step, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays, one entry for
    /// each event, in the order the events happened. The keys are:
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `faction`, `numpy.uint16`. The faction of the campaign.
    /// - `kind`, `numpy.uint8`. Zero is a raise, one is a win, two is a loss,
    ///   three is an end by a holder change to a third party.
    /// - `objective_kind`, `numpy.uint8`. As `campaigns` numbers it.
    /// - `objective_tile`, `numpy.uint32`. The objective, as a row-major
    ///   index.
    /// - `objective_q` and `objective_r`, `numpy.int32`. The same tile, as an
    ///   axial address. The engine reads it from the grid, because the event
    ///   holds no address of its own.
    /// - `cohort_size`, `numpy.uint32`. How many units the raise took.
    ///
    /// The engine declares the fields of the event in one place, and this
    /// method builds every column but the two addresses from that
    /// declaration.[^2]
    ///
    /// The log covers the last step alone, and a raise from this side lands in
    /// it until the next step. This method copies each column.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^2]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
    fn campaign_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log: Vec<CampaignEvent> = world.campaign_log().to_vec();
        let grid = world.grid();
        let address = |tile: u32| grid.address_of(TileIdx(tile)).unwrap_or(Axial::new(0, 0));
        let columns = columns_of(python, &log)?;
        let q: Vec<i32> = log
            .iter()
            .map(|event| address(event.objective_tile).q)
            .collect();
        let r: Vec<i32> = log
            .iter()
            .map(|event| address(event.objective_tile).r)
            .collect();
        columns.set_item("objective_q", q.to_pyarray(python))?;
        columns.set_item("objective_r", r.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns what the last step said about trade, as columns.
    ///
    /// The log holds one entry for each thing a party said. It holds one
    /// entry for each settlement or default the step resolved. It covers the
    /// last step alone, and the engine delivers it at the frame barrier.[^1]
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `proposer` and `responder`, `numpy.uint16`. The ordered pair.
    /// - `act`, `numpy.uint8`. Zero is an offer, one a counteroffer, two an
    ///   acceptance, three a refusal, four a terminal refusal, five an
    ///   opening, six a settlement, seven a default, eight a land transfer
    ///   and nine a relation step.
    /// - `status`, `numpy.uint8`. What the pair held after the act.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn trade_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        columns_of(python, world.trade_log())
    }

    /// Returns every carrier the controller has assigned, as columns.
    ///
    /// A carrier is a unit that the controller sent to the site of the other
    /// party of a contract. The list is simulated state and not a log of one
    /// step: a carrier stays in it until the contract settles or fails.
    ///
    /// - `unit`, `numpy.uint64`. The identity of the unit, opaque to a
    ///   caller.[^1]
    /// - `row`, `numpy.uint32`. The index of the contract in the negotiation
    ///   plane, which is the proposer times the faction count plus the
    ///   responder.
    /// - `faction`, `numpy.uint16`. The faction that assigned the unit.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn carrier_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let carriers = world.carrier_assignments();
        let columns = PyDict::new(python);
        let unit: Vec<u64> = carriers.iter().map(|entry| entry.unit).collect();
        let row: Vec<u32> = carriers.iter().map(|entry| entry.row).collect();
        let faction: Vec<u16> = carriers.iter().map(|entry| entry.faction.0).collect();
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("row", row.to_pyarray(python))?;
        columns.set_item("faction", faction.to_pyarray(python))?;
        Ok(columns)
    }
}
