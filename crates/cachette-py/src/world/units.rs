//! The verbs that act on a set of units, and the readers of a population.
//!
//! This module holds the verbs that make units, end them, order them, retype
//! them, convert them and send them, together with the readers that count a
//! population and list the units a faction sees.
//!
//! Every verb here is set-valued. Python builds a set and sends one command,
//! and the verb resolves every identity before it writes.[^1]
//!
//! # References
//!
//! [^1]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::resolve;
use cachette_core::unit_type::UnitTypeId;
use cachette_core::{Axial, FactionId, Fix32, ResourceKind};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Adds a soldier at each address and returns their identities.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. The
    /// faction is the number of the faction that owns the new soldiers.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each address, in the order of the addresses. Keep the array and
    /// pass it to `order_gather` or `despawn_soldiers`. Take one entry as a
    /// Python integer for `soldier_tile` or `explain_choice`.
    ///
    /// The call takes a set and answers once. It is not a per-unit verb that
    /// a caller repeats. A soldier is the mass tier, and no caller walks that
    /// population.[^1] The identities come back as one column, in the order
    /// of the addresses.
    ///
    /// **The set is all or nothing.** An address the world refuses removes
    /// every soldier this call made and raises. A caller that got half a
    /// population and an error would have to work out which half. The engine
    /// already knows.
    ///
    /// The verb is set-valued at the boundary. It is still a loop inside, and
    /// spawning has no cheaper whole-set algorithm today.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full. It raises when an address
    /// is outside the world, or when the ground admits no unit. It raises
    /// when the world has no such faction. The error names the address that
    /// refused. Water is the ground that admits no unit.
    ///
    /// **A spawn reads no occupancy, so it may put a tile above its
    /// capacity.** An over-full tile is a state of the world and not a
    /// fault. Movement is what holds a tile to its capacity, and it only
    /// ever takes units off an over-full tile.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    /// [^2]: Decisions register, DEC-063. `docs/DECISIONS.md`
    /// [^3]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decisions D1 and D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
    fn spawn_soldiers<'py>(
        &self,
        python: Python<'py>,
        addresses: Vec<(i32, i32)>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut made: Vec<u64> = Vec::with_capacity(addresses.len());
        for (q, r) in addresses {
            match world.spawn_soldier(Axial::new(q, r), FactionId(faction)) {
                Ok(unit) => made.push(unit.to_bits()),
                Err(error) => {
                    // Leave nothing half-made. The founding takes the same
                    // path when a group will not fit the place it chose.
                    for unit in &made {
                        let entity = world
                            .resolve_soldier(*unit)
                            .expect("this call made the identity a moment ago");
                        world.despawn_soldier(entity);
                    }
                    // A refused set leaves the world as it found it, and
                    // that includes the derived unit structure.[^4]
                    world
                        .rebuild_bridge(1)
                        .map_err(|refusal| VerbError::new_err(refusal.to_string()))?;
                    return Err(VerbError::new_err(format!(
                        "the address ({q}, {r}) refused a soldier: {error}"
                    )));
                }
            }
        }
        // **The verb leaves the world readable.** A spawn moves the arena
        // past the derived unit structure, and a reader between two steps is
        // not obliged to step first.[^4]
        //
        // [^4]: Findings register, FND-647. `docs/FINDINGS.md`
        world
            .rebuild_bridge(1)
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        Ok(made.to_pyarray(python))
    }

    /// Removes every soldier the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// **The set is all or nothing.** Every identity resolves before the call
    /// removes any soldier. One dead identity removes nothing and raises.[^1]
    ///
    /// A removed soldier leaves its slot to the next soldier. Its identity is
    /// then stale, and every call that takes an identity refuses it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn despawn_soldiers(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for entity in resolved {
            assert!(
                world.despawn_soldier(entity),
                "a resolved identity must name a soldier the arena can remove"
            );
        }
        // **The verb leaves the world readable.** A removal moves the arena
        // past the derived unit structure, and a reader between two steps is
        // not obliged to step first.[^2]
        //
        // [^2]: Findings register, FND-647. `docs/FINDINGS.md`
        world
            .rebuild_bridge(1)
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        Ok(())
    }

    /// Tells every soldier the identities name to gather a kind of resource.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The kind is the resource kind: food is zero, wood is one and stone is
    /// two. It is the same number the gather log carries in its `kind`
    /// column.
    ///
    /// **The kind here is a resource kind and not a ground kind.** The two
    /// scales are separate, and both start at zero. Water, plain and forest
    /// are the ground kinds 0, 1 and 2. Each of those numbers also names a
    /// resource kind. The call therefore reads 0, 1 or 2 as a resource kind.
    /// It orders the resource of that number. It raises nothing, and the
    /// soldiers gather the wrong resource. The engine sees a number, and not
    /// the scale the caller meant, so no check reports this.[^1]
    ///
    /// The call gives the order. It takes nothing. Step the world to make the
    /// soldiers act, then read `gather_log_columns` for what they took.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind is
    /// checked, before any order is given.
    ///
    /// **A unit whose type cannot gather refuses the order.** The gather
    /// rate of the row the unit indexes is zero, so the unit would keep the
    /// order and take nothing on every step. The verb refuses instead, so
    /// the caller learns at once. Read `unit_type_table` for the rate of
    /// each row.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number is three or above, because that names no
    /// resource kind. Raises `VerbError` when a unit's type has a gather
    /// rate of zero.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-120. `docs/DECISIONS.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn order_gather(&self, units: Vec<u64>, kind: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = ResourceKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no resource kind")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            let entity = resolve(&world, *unit)?;
            let row = world
                .unit_type_row(entity)
                .expect("a resolved identity names a live soldier");
            if row.gather_rate == Fix32::ZERO {
                return Err(VerbError::new_err(format!(
                    "the unit {unit} is of a type whose gather rate is zero, and it cannot gather"
                )));
            }
            resolved.push(entity);
        }
        // The set form is the one path the controller takes too, so one loop
        // serves both callers.
        let refused = world.order_gather_set(&resolved, kind);
        assert_eq!(
            refused, 0,
            "a resolved identity must name a soldier the arena can order"
        );
        Ok(())
    }

    /// Founds a city from every settler the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The verb founds a settlement on the tile the unit stands on, for the
    /// faction of the unit, and it seats the group the column names. **The
    /// founding spends the settler**, and the group takes its place.[^1]
    ///
    /// The verb refuses a unit whose settle column is zero, a tile any
    /// faction holds, a tile that already carries a settlement, ground that
    /// admits no unit, and a place inside the founding distance of a city
    /// that stands.[^2] A refused unit changes nothing and keeps its life.
    ///
    /// **The set is not all or nothing.** Each unit is answered on its own,
    /// because a set of settlers stands in several places and one refusal
    /// says nothing about the rest. The call returns the number of cities it
    /// founded.
    ///
    /// The call founds. Step the world for the holding pass to give the new
    /// city its ground, then read `settlement_count` and `standing`.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    fn order_settle(&self, units: Vec<u64>) -> PyResult<usize> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        // The set form is the one path the controller takes too, so one loop
        // serves both callers.
        let outcomes = world.settle_set(&resolved);
        Ok(outcomes.iter().filter(|outcome| outcome.founded()).count())
    }

    /// Names why the settle verb refuses each settler of one faction.
    ///
    /// The faction is the number of the faction that holds the settlers.
    ///
    /// Returns one name for each settler the faction holds, in the order the
    /// engine walks the settlers. A settler the verb would accept carries the
    /// name `accepted`. A faction that holds no settler gets an empty list.
    ///
    /// **The verb answers one byte and names no reason.** A caller that took
    /// the settle action and read a refusal cannot say which rule refused, so
    /// it reads this instead. The names are the ones the engine states, and
    /// this call recomputes nothing.
    ///
    /// The names are `accepted`, `no_such_unit`, `not_a_settler`,
    /// `outside_world`, `faction_may_not_found`, `ground_is_held`,
    /// `settlement_stands`, `ground_admits_nobody`, `too_close_to_a_city` and
    /// `founding_refused_the_place`.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    fn settle_refusals(&self, faction: u16) -> PyResult<Vec<&'static str>> {
        self.lock()
            .settle_refusal_names(FactionId(faction))
            .ok_or_else(|| ViewError::new_err(format!("{faction} names no faction of this world")))
    }

    /// Returns how many settlers one faction holds, as an integer.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The observation of the faction publishes the same count under the
    /// name `settlers`, so a reader of either one reads the same quantity.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    fn settler_count(&self, faction: u16) -> PyResult<u32> {
        self.lock()
            .settler_count(FactionId(faction))
            .ok_or_else(|| ViewError::new_err(format!("{faction} names no faction of this world")))
    }

    /// Gives every soldier the identities name one unit type.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The `unit_type` is a row of the shared table, as a Python integer. The
    /// table holds eight rows, numbered zero to seven. A number of eight or
    /// above names no row. The world builds the table with the default rows,
    /// and `define_unit_type` writes one. A row whose every column is zero
    /// is a unit that can do nothing.[^1]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the type
    /// is checked, before any soldier is written. One refusal leaves the
    /// world unchanged and raises.[^2]
    ///
    /// The call gives the type. It takes nothing and it moves nothing. Step
    /// the world to make the soldiers act.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no row of the table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn set_unit_types(&self, units: Vec<u64>, unit_type: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = UnitTypeId::from_u8(unit_type)
            .ok_or_else(|| VerbError::new_err(format!("{unit_type} names no unit type")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let refused = world.set_unit_type_set(&resolved, kind);
        assert_eq!(
            refused, 0,
            "a resolved identity must name a soldier the arena can write"
        );
        Ok(())
    }

    /// Returns the unit type of one soldier, as an integer.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned, or of the `unit` column of the
    /// gather log.
    ///
    /// The result is a row of the shared table. Read the row itself with
    /// `unit_type_table`, and write it with `define_unit_type`. A new
    /// soldier carries row zero.[^1]
    ///
    /// **This read stays singular while the write verb takes a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. `soldier_tile` follows
    /// the same rule.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn unit_type(&self, unit: u64) -> PyResult<u8> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let found = world
            .unit_type(entity)
            .ok_or_else(|| ViewError::new_err(format!("{unit} names no live soldier")))?;
        Ok(found.0)
    }

    /// The number of soldiers alive in the world, as an integer.
    ///
    /// The engine counts them. A caller never counts a population by
    /// walking it, because a soldier is the mass tier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[getter]
    fn soldier_count(&self) -> u32 {
        self.lock().soldiers().len()
    }

    /// Returns the number of live units of each faction, by faction number.
    ///
    /// **This is one call and it names no unit.** The engine maintains the
    /// count where a unit is created and where a unit ends. This reads a
    /// small array and starts no pass over the population.[^1] A caller that
    /// counts the units of a faction in Python crosses the boundary once for
    /// each unit. The control plane rule forbids that.[^2]
    ///
    /// The list holds one entry for each faction the world was built with.
    /// The entry of a faction reads zero when its last unit ends. Nothing
    /// else the bindings expose says so.
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    fn faction_population(&self) -> Vec<u32> {
        let world = self.lock();
        let counts = world.population_by_faction();
        let factions = world.config().faction_count as usize;
        counts.iter().copied().take(factions).collect()
    }

    /// Returns the live soldiers of one faction, as columns.
    ///
    /// The argument is the number of a faction. The result is a `dict` of
    /// one-dimensional NumPy arrays. Every array holds one entry for each
    /// live soldier of that faction. The keys are:
    ///
    /// - `unit`, `numpy.uint64`. The identity of the soldier. Pass the whole
    ///   array to `send_units_to`, `order_gather` or `despawn_soldiers`.
    /// - `tile`, `numpy.uint32`. The tile it stands on, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    ///
    /// **This is one crossing, and it replaces a loop.** A caller that reads
    /// one unit through `soldier_tile` pays one crossing for that unit. The
    /// control plane never loops over the population.[^1] [^2]
    ///
    /// **Every entry names a live soldier, so no entry stands for nothing.**
    /// The engine builds the set at the moment of the call. It takes no
    /// identity from the caller, so nothing here can be stale. The result
    /// needs no validity mask. The singular read takes an identity, and it
    /// refuses a dead one.[^3]
    ///
    /// The order is the slot order of the arena. It is the same on every run
    /// and at every thread count. It is never a thread completion
    /// order.[^4] It is not the spawn order. A slot returns to the arena when
    /// a soldier dies, and the next soldier takes it.
    ///
    /// A faction with nobody in it gives two empty arrays. That is an answer
    /// and not an error. A number that names no faction of this world gives
    /// the same answer, because no soldier holds it.
    ///
    /// # References
    ///
    /// [^1]: Project orientation, the design principles. `CLAUDE.md`
    /// [^2]: Research report 20, what the Python interface should be, section 2.3. `docs/research/reports/20-the-python-interface.md`
    /// [^3]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn faction_units<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let soldiers = world.soldiers();
        let mut unit: Vec<u64> = Vec::new();
        let mut tile: Vec<u32> = Vec::new();
        for entity in soldiers.iter_faction(FactionId(faction)) {
            unit.push(entity.to_bits());
            tile.push(
                soldiers
                    .tile(entity)
                    .expect("a live identity from the walk names a tile")
                    .0,
            );
        }
        let columns = PyDict::new(python);
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns every unit one faction sees now, as a `dict` of arrays.
    ///
    /// A faction sees a unit when it sees the tile that unit stands on. It
    /// therefore reads its own units on the ground its own units watch, and
    /// it reads a rival that walks into that ground.[^1]
    ///
    /// **No argument asks for the truth.** A caller that wants every unit of
    /// one faction, seen or not, calls `faction_units`, which reports the
    /// units of the faction it names and nothing else.
    ///
    /// The three arrays hold one entry for each unit, at one index.
    ///
    /// - `unit`, `numpy.uint64`. The identity of the unit.
    /// - `tile`, `numpy.uint32`. The tile the unit stands on.
    /// - `faction`, `numpy.uint16`. The faction the unit belongs to.
    ///
    /// **The order is fixed.** The walk runs over the factions in faction
    /// order, and over the units of each faction in slot order, so two runs
    /// return one answer.[^2]
    ///
    /// **This answers the present frame and never a memory.** A remembered
    /// place reports no unit, so a unit that walked out of sight leaves this
    /// array.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn faction_visible_units<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let seen = world.units_seen_by(FactionId(faction));
        let mut unit: Vec<u64> = Vec::with_capacity(seen.len());
        let mut tile: Vec<u32> = Vec::with_capacity(seen.len());
        let mut owner: Vec<u16> = Vec::with_capacity(seen.len());
        for row in seen {
            unit.push(row.unit.to_bits());
            tile.push(row.tile.0);
            owner.push(row.faction.0);
        }
        let columns = PyDict::new(python);
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        columns.set_item("faction", owner.to_pyarray(python))?;
        Ok(columns)
    }

    /// Changes the faction of every soldier the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` or `faction_units` returned. The
    /// faction is the number of the faction that the units join. The number
    /// runs from zero to one below the faction count of the world. Returns
    /// `None`.
    ///
    /// A unit that changes faction keeps its identity. Every identity the
    /// caller holds still names the same unit. It keeps its type, the load it
    /// carries, the tile it stands on and the site it lives in. It loses its
    /// gather order, its build order and its destination. An order is an
    /// instruction from the faction that no longer holds it. A unit that
    /// carries a character takes that character with it.[^1]
    ///
    /// A unit that already belongs to the faction is left alone. Calling this
    /// twice with one set therefore has the same result as calling it once.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the faction
    /// is checked, before anything changes.
    ///
    /// **This is the deliberate route.** The engine also converts a unit on
    /// its own. That happens where another faction reaches the unit's place
    /// more strongly than its own faction does. Set that reach with
    /// `set_influence_source`, and read the result with
    /// `converted_log_columns`.[^2]
    ///
    /// ```python
    /// mine = world.faction_units(0)
    /// world.convert_units(mine["unit"], 1)
    /// ```
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decisions D2, D3 and D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    /// [^2]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decisions D1 and D4. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
    fn convert_units(&self, units: Vec<u64>, faction: u16) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        world
            .convert_units(&resolved, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Sends every soldier the identities name to a set of tiles.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. The seeds are a
    /// sequence of `(q, r)` pairs of integers. They are the places the caller
    /// wants the units at. The destination is the number of the destination
    /// plane that carries the order. The default is 0. Returns `None`.
    ///
    /// **One call names a whole set and the engine builds one field.** The
    /// engine takes the level 1 cell of each seed. It seeds the plane at all
    /// of them at once. It spreads a reach outward. Two seeds in one cell act
    /// as one, because the engine removes duplicate cells. Every unit the
    /// call names then reads one entry of the plane on each step. It takes
    /// one step along that direction. The cost of the field follows the cell
    /// count, and not the number of units. Sending a million units costs what
    /// sending one costs.[^1]
    ///
    /// **No unit searches for a route.** A unit reads the entry of its own
    /// cell. It reads no neighbouring cell. It computes nothing from its own
    /// address toward a seed. That is the rule the engine is built on. This
    /// call does not bend it.[^2]
    ///
    /// **A cell steers a whole block, so two units in one cell take one
    /// direction.** A caller cannot send half a cell one way and half the
    /// other.[^2]
    ///
    /// **A unit that cannot reach the seeds does not freeze.** A unit whose
    /// cell holds no direction takes a keyed draw instead. The draw is keyed
    /// on the frame, so the unit takes a different direction on the next
    /// frame. The same holds for a unit that arrived. It also holds for a
    /// unit whose ground refuses the direction the field gave it.[^3]
    ///
    /// **The call sends a set toward a place. It does not promise that the set
    /// arrives.** A cell steers a block of tiles. The water in front of one
    /// unit of that block is not a fact the block carries. A unit behind such
    /// a barrier walks to it, and then wanders beside it. It is not frozen. It
    /// does not get past.[^4]
    ///
    /// The order holds until the caller stops it with `stop_sending`. A unit
    /// that arrives keeps the order. It walks about inside the block it
    /// arrived in. Read `faction_units` for where the set is now.
    ///
    /// A caller that names a destination again replaces the seed set of that
    /// destination. Every unit already sent to it walks to the new one. Read
    /// `destination_count` for how many the world holds. Set it with
    /// `set_destination_count`.
    ///
    /// **The set is all or nothing.** Every identity resolves, every address
    /// is checked, and the destination is checked, before anything changes.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no destination plane of this
    /// world, and when a seed address is outside the world. Raises `ViewError`
    /// when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^4]: Findings register, FND-411. `docs/FINDINGS.md`
    #[pyo3(signature = (units, seeds, destination = 0))]
    fn send_units_to(
        &self,
        units: Vec<u64>,
        seeds: Vec<(i32, i32)>,
        destination: u16,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let addresses: Vec<Axial> = seeds.iter().map(|(q, r)| Axial::new(*q, *r)).collect();
        world
            .send_units_to(&resolved, &addresses, destination)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Stops sending every soldier the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// Each unit goes back to the option that it chose for itself.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// changes.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    fn stop_sending(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        world
            .stop_sending(&resolved)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// The number of destination planes the world holds, as an integer.
    ///
    /// A destination plane carries one order. The caller names the plane when
    /// it sends a set of units somewhere. The numbers run from zero to one
    /// below this.
    #[getter]
    fn destination_count(&self) -> u16 {
        self.lock().destination_count()
    }

    /// Sets the number of destination planes the world holds.
    ///
    /// The count says how many places the control plane may send units to at
    /// one time. The next place re-aims a plane the control plane already
    /// used. **The caller names the plane, and the engine allocates none.**[^1]
    ///
    /// The call clears the seed set of every plane. No order steers anything
    /// until the caller sends a set again. A unit that was sent to a plane the
    /// world no longer holds reads no direction. It takes a keyed draw rather
    /// than standing still.[^2]
    ///
    /// Set this before the run, in the way the other world parameters are set.
    /// Returns `None`.
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    fn set_destination_count(&self, count: u16) {
        self.lock().set_destination_count(count);
    }
}
