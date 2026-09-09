//! What one faction sees, and the schemas a learner reads it through.
//!
//! This module holds the tile report and the region summary a faction sees,
//! the observation and the action a controller exchanges, the schemas that
//! describe both, the legal action set, the weights that score a choice, and
//! the explanation of a choice.
//!
//! The grouping is by the point of view. A method here answers for one
//! faction and hides what that faction cannot see, so a learner reads only
//! what a player reads.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::resolve;
use cachette_core::faction_observation::observation_schema;
use cachette_core::faction_view::{Admit, FactionTile};
use cachette_core::{Axial, FactionId, FactionWeights, WEIGHT_HIGH, WEIGHT_LOW};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[pymethods]
impl PyWorld {
    /// Returns what one faction may read about one tile, as a `dict`.
    ///
    /// **No argument asks for the truth.** The engine applies the sight rule
    /// inside the reader, so a caller cannot ask past the fog.[^1] A caller
    /// that wants the truth of the world calls `tile_report`, which names no
    /// faction and serves a developer who watches the engine.[^2]
    ///
    /// - `q` and `r`, integers. The address the call took.
    /// - `faction`, an integer. The faction the call took.
    /// - `sighting`, a string. One of `never`, `remembered` and `seen`.
    ///
    /// A `never` answer carries no other value. Every ground key is `None`,
    /// so a caller tells that answer from a place that holds nothing.
    ///
    /// A `remembered` answer carries the ground of the place and no more.
    /// The `kind`, `passable`, `height` and `generated` keys hold values, and
    /// every other key is `None` or zero. **The reader answers no unit, no
    /// holder and no upgrade**, because each of those is a fact of the
    /// present frame.[^3]
    ///
    /// A `seen` answer carries the present frame as well.
    ///
    /// - `value`, an integer. The value of the tile. **A raw Q16.16 value.**
    /// - `capacity`, an integer. How many units the tile admits.
    /// - `stock`, a list of integers. What each resource kind holds now.
    /// - `holder`, an integer or `None`. Who holds the tile now.
    /// - `upgrade`, an integer or `None`. The category that stands there.
    /// - `upgrade_level`, an integer. The level that stands there.
    /// - `units`, an integer. How many units stand on the tile.
    ///
    /// The `height` and `generated` keys hold the ground that the seed and
    /// the address fix, so a remembered answer and a seen answer agree on
    /// them.[^4]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    /// Raises `ViewError` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^3]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^4]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    fn faction_tile_report<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let read = world
            .faction_tile(FactionId(faction), Axial::new(q, r))
            .ok_or_else(|| ViewError::new_err(format!("({q}, {r}) lies outside this world")))?;

        let report = PyDict::new(python);
        report.set_item("q", q)?;
        report.set_item("r", r)?;
        report.set_item("faction", faction)?;
        report.set_item(
            "sighting",
            match read {
                FactionTile::Never => "never",
                FactionTile::Remembered(_) => "remembered",
                FactionTile::Seen(_) => "seen",
            },
        )?;
        match read.ground() {
            Some(ground) => {
                report.set_item("kind", ground.kind.to_u8())?;
                report.set_item("passable", ground.kind.is_passable())?;
                report.set_item("height", ground.height.0)?;
                report.set_item(
                    "generated",
                    ground
                        .generated
                        .iter()
                        .map(|amount| amount.0)
                        .collect::<Vec<u32>>(),
                )?;
            }
            None => {
                report.set_item("kind", python.None())?;
                report.set_item("passable", python.None())?;
                report.set_item("height", python.None())?;
                report.set_item("generated", python.None())?;
            }
        }
        match read {
            FactionTile::Seen(seen) => {
                report.set_item("value", seen.value.0)?;
                report.set_item("capacity", seen.capacity)?;
                report.set_item(
                    "stock",
                    seen.stock
                        .iter()
                        .map(|amount| amount.0)
                        .collect::<Vec<u32>>(),
                )?;
                match seen.holder {
                    Some(holder) => report.set_item("holder", holder.0)?,
                    None => report.set_item("holder", python.None())?,
                }
                match seen.upgrade {
                    Some(site) => {
                        report.set_item("upgrade", site.category.to_u8())?;
                        report.set_item("upgrade_level", site.level)?;
                    }
                    None => {
                        report.set_item("upgrade", python.None())?;
                        report.set_item("upgrade_level", 0u8)?;
                    }
                }
                report.set_item("units", seen.units)?;
            }
            FactionTile::Never | FactionTile::Remembered(_) => {
                report.set_item("value", python.None())?;
                report.set_item("capacity", python.None())?;
                report.set_item("stock", python.None())?;
                report.set_item("holder", python.None())?;
                report.set_item("upgrade", python.None())?;
                report.set_item("upgrade_level", 0u8)?;
                report.set_item("units", 0u32)?;
            }
        }
        Ok(report)
    }

    /// Returns the summary of one cell, over the tiles one faction may read.
    ///
    /// The cell is the cell that covers the address, and it is the cell that
    /// `region_summary` reads. **This reader combines only the tiles the
    /// sight rule admits**, so a cell cannot state what its tiles hide.[^1]
    ///
    /// The `admit` argument names which of the two rules the call took, and
    /// neither one widens the answer past the fog.[^2] It takes `now` for
    /// the tiles the faction sees this frame, and `ever` for the tiles it
    /// has ever seen. It defaults to `now`.
    ///
    /// - `q`, `r`, `faction` and `admit`. The arguments the call took.
    /// - `admitted`, an integer. How many tiles of the cell the rule
    ///   admitted.
    /// - `withheld`, an integer. How many tiles of the cell the rule
    ///   withheld. A zero here means that the faction reads the whole cell.
    /// - `tiles`, `open_tiles`, `units`, `held_tiles`, `value_total`,
    ///   `height_total` and `food_total`. The fields `region_summary`
    ///   reports, over the admitted tiles alone.
    ///
    /// **A tile the faction saw once and does not see now adds the ground
    /// alone.** It adds no unit, no held tile and no value, because each of
    /// those is a fact of the present frame.[^1]
    ///
    /// **The call walks the tiles of the cell.** The rebuilt cell counts
    /// tiles the faction has not seen, so the reader cannot use it. The cost
    /// follows the tiles of one cell and never the world.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world, or
    /// when the `admit` argument names neither rule. Raises `ViewError` when
    /// the address lies outside the world, and when the derived unit
    /// structure does not describe the units. The message names which of the
    /// two happened.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    /// [^2]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[pyo3(signature = (faction, q, r, admit = "now"))]
    fn faction_region_summary<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
        q: i32,
        r: i32,
        admit: &str,
    ) -> PyResult<Bound<'py, PyDict>> {
        let rule = match admit {
            "now" => Admit::SeenNow,
            "ever" => Admit::SeenEver,
            other => {
                return Err(VerbError::new_err(format!(
                    "{other} names no admission rule of this reader"
                )))
            }
        };
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let masked = world
            .faction_summary_covering(FactionId(faction), Axial::new(q, r), rule)
            .map_err(|error| ViewError::new_err(error.to_string()))?;
        let summary = masked.summary();
        let fields = PyDict::new(python);
        fields.set_item("q", q)?;
        fields.set_item("r", r)?;
        fields.set_item("faction", faction)?;
        fields.set_item("admit", admit)?;
        fields.set_item("admitted", masked.admitted())?;
        fields.set_item("withheld", masked.withheld())?;
        fields.set_item("tiles", summary.tiles())?;
        fields.set_item("open_tiles", summary.open_tiles())?;
        fields.set_item("units", summary.units())?;
        fields.set_item("held_tiles", summary.held_tiles())?;
        // The four counts below are relative to the faction that reads, and
        // none of them is indexed by a faction. A reader tells its own army
        // from an invading one, and it never asks after a named rival.
        fields.set_item("own_units", masked.own_units())?;
        fields.set_item("other_units", masked.other_units())?;
        fields.set_item("own_held_tiles", masked.own_held_tiles())?;
        fields.set_item("other_held_tiles", masked.other_held_tiles())?;
        fields.set_item("value_total", summary.value_total().0)?;
        fields.set_item("height_total", summary.height_total().0)?;
        fields.set_item("food_total", summary.food_total().0)?;
        Ok(fields)
    }

    /// Returns the observation of one faction, as one NumPy `int64` array.
    ///
    /// **The array holds what that faction observes, and nothing else.** No
    /// argument widens the answer. A caller that wants the truth of the
    /// world calls a reader that names no faction.[^1]
    ///
    /// `observation_schema` declares where every field of the array sits, and
    /// it is the only declaration of that layout. **Do not write an offset,
    /// a length or a bound into a file outside the engine.** Read the schema
    /// and decode by arithmetic over it.[^2]
    ///
    /// **The length is one number for every world.** No field of the layout
    /// follows the world shape, the faction count or the population.[^3] A
    /// faction that loses every unit reads an array of the same length as a
    /// faction that holds a million, and a policy trained on a small world
    /// reads a large one.
    ///
    /// No position holds a floating point number. Every position is a Q16.16
    /// value between minus one and one, so 65536 is one unit.[^4]
    ///
    /// **A field whose declared bounds are zero and zero is reserved.** It
    /// reads zero in every position until a layout revision claims it, and
    /// that zero does not state a real quantity of zero. The confidence
    /// statistic of each rival power quantity states how much of the estimate
    /// the faction observed, so a caller tells an unobserved estimate from a
    /// real zero.
    ///
    /// This method copies the array.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    /// Raises `ViewError` when the derived unit structure does not describe
    /// the units, and the message names which refusal the structure gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^4]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    fn faction_observation<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<i64>>> {
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        // **The message names the reason.** The reader returns a refusal
        // that names its cause, and a stale derived structure names both
        // revisions. A sentence that named none of it sent four readers of
        // one traceback looking for four different causes.[^5]
        //
        // [^5]: Findings register, FND-647. `docs/FINDINGS.md`
        let values = world
            .faction_observation(FactionId(faction))
            .map_err(|error| ViewError::new_err(error.to_string()))?;
        Ok(values.to_pyarray(python))
    }

    /// Returns the declared layout of the observation array, as a `dict`.
    ///
    /// **This is the only declaration of that layout.** The engine builds the
    /// schema and the array from one field list, so a caller that decodes by
    /// arithmetic over this schema cannot disagree with the array. No file
    /// outside the engine may state a position, a length or a bound.[^1]
    ///
    /// **The schema states the structure of the array and not only the
    /// position of each field.** A start and a width do not say how many
    /// cells a block holds, how many channels a cell holds, or which of the
    /// two axes runs first. A reader that guessed would read a plausible
    /// observation that does not exist, so the engine states every one of
    /// them.[^6]
    ///
    /// **The schema names the value form of each field, and it says what an
    /// inversion of that form needs.** A start, a width and a pair of bounds
    /// say where a value sits and how far it reaches. They do not say what
    /// the value means. A count crosses the boundary as a compressed
    /// magnitude, so a reader that wanted the count had to restate the
    /// compression, and that second copy of an engine rule fails silently
    /// when the engine moves.[^8]
    ///
    /// The dictionary holds seven keys.
    ///
    /// - `version`, an integer. The version of the layout. A field added,
    ///   removed, relengthened or rebounded changes the meaning of a stored
    ///   weight file, so a learner that loads a policy under another version
    ///   must stop.[^2]
    /// - `length`, an integer. How many positions the whole array holds. It
    ///   is the length `faction_observation` returns.
    /// - `ring_cells`, a list of integers. The cells of each ring of the
    ///   egocentric frame, in ring order. A cell index says nothing about its
    ///   ring and its sector on its own.
    /// - `channel_order`, a string. `cell_major` when every channel of one
    ///   place is adjacent, and `channel_major` when every place of one
    ///   channel is adjacent.
    /// - `spatial_gate`, a string. The channel that says whether a cell holds
    ///   a value at all. A cell of the frame outside the world reads zero in
    ///   every channel, and that zero is an absent value and not a quantity
    ///   of zero.[^6]
    /// - `value_forms`, a `dict` of `dict`. One entry for each value form the
    ///   layout uses, keyed by the name of the form. The table sits here
    ///   rather than inside each field row, because one form serves many
    ///   fields.
    /// - `fields`, a list of `dict`. One entry for each field, in the order
    ///   the array holds them.
    ///
    /// Each entry of `value_forms` holds ten keys.
    ///
    /// - `name`, a string. The name of the form, which is the key as well.
    /// - `low` and `high`, integers. The bounds every field of the form
    ///   declares.
    /// - `unit`, an integer. The integer that stands for one unit of the
    ///   fixed-point scale.
    /// - `uniform`, a bool. `True` when every position of a field of the form
    ///   holds that one form. A form that groups several forms over one
    ///   quantity holds `False`, and a reader of such a field must read the
    ///   channel and not the field.
    /// - `invertible`, a bool. `True` when a reader recovers the quantity
    ///   from the value and this entry alone.
    /// - `denominator`, a string or `None`. `per_field` when the field doc
    ///   names the whole the value divides by, and `sum_of_magnitudes` when
    ///   the form divides by the sum of the two magnitudes it compares.
    ///   Neither denominator travels with the value, so neither form is
    ///   invertible.
    /// - `log_base`, `log_offset` and `divisor_bits`, integers or `None`. A
    ///   compressed magnitude adds the offset to the quantity, takes the
    ///   logarithm in the base, and divides by the divisor. An inversion runs
    ///   that backwards. Every other form holds `None`.
    ///
    /// Each field entry holds nine keys.
    ///
    /// - `name`, a string. The name of the field.
    /// - `start`, an integer. The position the field starts at.
    /// - `positions`, an integer. How many positions the field holds. A field
    ///   is contiguous, so position `n` of it sits at `start + n`.
    /// - `dtype`, a string. The NumPy element type of every position.
    /// - `low` and `high`, integers. The lowest and the highest value any
    ///   position of the field may hold.
    /// - `form`, a string. The name of the value form of the field, which
    ///   keys the `value_forms` table. The form comes from the same
    ///   declaration that decides how the field is written, so the writer and
    ///   the published form cannot disagree.[^8]
    /// - `space`, a string or `None`. `ring` when the positions of the field
    ///   are cells of the egocentric frame, and `token` when they are tokens
    ///   of one set. A field of separate quantities holds `None`.
    /// - `channels`, a list of strings. The channels of one place of the
    ///   field, in the order the field stores them. A field that holds one
    ///   quantity for each position holds an empty list.
    ///
    /// **The engine publishes one token field for each token set.** The four
    /// sets hold different channel counts, so one field cannot state four
    /// shapes.[^7]
    ///
    /// A field whose bounds are the whole range of the element type has no
    /// tighter bound that the world parameters give. The engine states no
    /// measured figure, because a blocker governs every measured figure of
    /// this project.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the interpreter refuses to hold the dictionary.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    /// [^6]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    /// [^7]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D4. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    /// [^8]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D2. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
    fn observation_schema<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let schema = observation_schema();
        let fields = PyList::empty(python);
        for row in schema.rows() {
            let entry = PyDict::new(python);
            entry.set_item("name", row.name())?;
            entry.set_item("start", row.start)?;
            entry.set_item("positions", row.positions)?;
            entry.set_item("dtype", row.kind().numpy_name())?;
            entry.set_item("low", row.low)?;
            entry.set_item("high", row.high)?;
            entry.set_item("form", row.form_name())?;
            entry.set_item("space", row.space())?;
            entry.set_item("channels", PyList::new(python, row.channels())?)?;
            fields.append(entry)?;
        }
        let forms = PyDict::new(python);
        for form in schema.value_forms() {
            let entry = PyDict::new(python);
            entry.set_item("name", form.name)?;
            entry.set_item("low", form.low)?;
            entry.set_item("high", form.high)?;
            entry.set_item("unit", form.unit)?;
            entry.set_item("uniform", form.uniform)?;
            entry.set_item("invertible", form.invertible)?;
            entry.set_item("denominator", form.denominator)?;
            entry.set_item("log_base", form.log_base)?;
            entry.set_item("log_offset", form.log_offset)?;
            entry.set_item("divisor_bits", form.divisor_bits)?;
            forms.set_item(form.name, entry)?;
        }
        let out = PyDict::new(python);
        out.set_item("version", schema.version())?;
        out.set_item("length", schema.length())?;
        out.set_item("ring_cells", PyList::new(python, schema.ring_cells())?)?;
        out.set_item("channel_order", schema.channel_order())?;
        out.set_item("spatial_gate", schema.spatial_gate())?;
        out.set_item("value_forms", forms)?;
        out.set_item("fields", fields)?;
        Ok(out)
    }

    /// Returns the declared layout of the action table, as a `dict`.
    ///
    /// **This is the only declaration of that layout.** An action is one
    /// integer that indexes a bounded table the engine declares, and a
    /// caller decodes that integer by arithmetic over this schema and never
    /// by a table it holds.[^1] No file outside the engine may state a verb
    /// number, a position or a bound.
    ///
    /// The dictionary holds three keys.
    ///
    /// - `version`, an integer. The version of the layout. A verb added or a
    ///   bound changed moves every row above it, and that changes the
    ///   meaning of a stored weight file, so a learner that loads a policy
    ///   under another version must stop.
    /// - `length`, an integer. How many rows the whole table holds. It is
    ///   the length `legal_actions` returns.
    /// - `verbs`, a list of `dict`. One entry for each verb, in the order
    ///   the table holds them.
    ///
    /// Each verb entry holds four keys.
    ///
    /// - `name`, a string. The name of the verb.
    /// - `first`, an integer. The action integer of the first row of the
    ///   verb.
    /// - `rows`, an integer. How many rows of the table the verb holds. It
    ///   is the product of the bounds of its positions, and it is one for a
    ///   verb that declares no position.
    /// - `positions`, a list of `dict`. The argument positions the verb
    ///   declares, in order.
    ///
    /// Each position entry holds three keys.
    ///
    /// - `candidate`, a string. The kind of thing the candidate list of the
    ///   position holds.
    /// - `bound`, an integer. The ceiling on that candidate list. **No bound
    ///   follows the population, and no bound follows the world extent.**
    /// - `stride`, an integer. How far one step of the position moves the
    ///   action integer.
    ///
    /// A caller reads the action integer of one verb and its arguments as
    /// `first + sum(argument * stride)`, and it reverses that by division.
    ///
    /// **A verb whose content the engine resolves declares no position.** A
    /// crossing names a place, and the engine chooses that place at the tick
    /// the action applies, in the way it chooses it for the built-in
    /// controller.[^2]
    ///
    /// **A position whose candidate kind is `place` names a cell of the
    /// egocentric frame.** Its value zero names the whole frame, and the
    /// engine then resolves the place over every cell. A value above zero
    /// names cell `value - 1` of that frame, and the observation schema
    /// gives the cells of each ring of the same frame under `ring_cells`.
    /// **The bound is a constant of the frame**, so a policy trained on one
    /// world names the same place on a world of another size.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the interpreter refuses to hold the dictionary.
    ///
    /// # References
    ///
    /// [^1]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
    /// [^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
    /// [^3]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decisions D1, D2 and D4. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
    fn action_schema<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let schema = world.action_schema();
        let verbs = PyList::empty(python);
        for row in schema.rows() {
            let positions = PyList::empty(python);
            for position in &row.positions {
                let entry = PyDict::new(python);
                entry.set_item("candidate", position.candidate.name())?;
                entry.set_item("bound", position.bound)?;
                entry.set_item("stride", position.stride)?;
                positions.append(entry)?;
            }
            let entry = PyDict::new(python);
            entry.set_item("name", row.name())?;
            entry.set_item("first", row.first)?;
            entry.set_item("rows", row.rows)?;
            entry.set_item("positions", positions)?;
            verbs.append(entry)?;
        }
        let out = PyDict::new(python);
        out.set_item("version", schema.version())?;
        out.set_item("length", schema.length())?;
        out.set_item("verbs", verbs)?;
        Ok(out)
    }

    /// Returns one byte for each row of the action table, as a NumPy `uint8`
    /// array.
    ///
    /// The byte is one when the verb would take that row at this tick, and
    /// zero when it would refuse it.[^1] Row zero is the no-op, and it is
    /// always one, so the answer is never empty and a learner never learns
    /// legality by trial.
    ///
    /// **The answer holds only what the faction observes.** Two verbs act on
    /// a place, and the engine resolves each place through the readers that
    /// answer for one faction. A byte therefore states nothing about ground
    /// the faction has never seen.[^2]
    ///
    /// `action_schema` declares which row is which. This method copies the
    /// array.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn legal_actions<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<u8>>> {
        let world = self.lock();
        let answer = world.legal_actions(FactionId(faction)).ok_or_else(|| {
            VerbError::new_err(format!("{faction} names no faction of this world"))
        })?;
        Ok(answer.to_pyarray(python))
    }

    /// Returns the actions the built-in controller took for one faction on
    /// the last tick, as three NumPy arrays in one dictionary.
    ///
    /// The `action` column holds the action integer of each command, in the
    /// encoding the action table of this world states. The `applied` column
    /// holds one where the verb took the command. The `encoded` column holds
    /// one where the action column holds the encoding of the choice, and
    /// zero where the table could not express it. **Read the encoded column
    /// before the action column.** A row that states zero holds the no-op
    /// row in its action column, and the no-op is a real action, so the two
    /// are one value without this column.[^1]
    ///
    /// The engine empties the log at the start of each tick, so this answers
    /// for the last tick alone. A caller that wants a window of ticks reads
    /// it after each step.
    ///
    /// **This answers for one faction and reads no other.** The rows of the
    /// other factions stay in the engine, so a learner that records what the
    /// controller does in its own seat reads nothing the fog hides.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    fn controller_actions<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if faction >= world.faction_count().max(1) {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let rows: Vec<&cachette_core::ControllerCommand> = world
            .controller_log()
            .iter()
            .filter(|row| row.faction.0 == faction)
            .collect();
        let action: Vec<u32> = rows.iter().map(|row| row.action).collect();
        let applied: Vec<u8> = rows.iter().map(|row| row.applied).collect();
        let encoded: Vec<u8> = rows.iter().map(|row| row.encoded).collect();
        let out = PyDict::new(python);
        out.set_item("action", action.to_pyarray(python))?;
        out.set_item("applied", applied.to_pyarray(python))?;
        out.set_item("encoded", encoded.to_pyarray(python))?;
        Ok(out)
    }

    /// Applies one action of one faction, and returns whether the verb took
    /// it.
    ///
    /// The action is one integer that indexes the action table of this
    /// world. `action_schema` declares which row is which, and
    /// `legal_actions` says which rows the verbs would take now.
    ///
    /// **This runs the verb and reports what the verb did.** It does not
    /// read the legality answer first, so a row the answer allows and the
    /// verb then refuses is a defect rather than a silent disagreement.[^1]
    ///
    /// Every action goes through the same verbs a Python caller and the
    /// built-in controller go through, so a learner reaches no store the
    /// controller cannot reach.[^2] The engine writes one row of the
    /// controller log for the action, whether the verb took it or refused
    /// it, and that row carries the whole action integer.[^3]
    ///
    /// **Send one integer for one faction. Do not loop over entities.** A
    /// learner is a control plane.[^4]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world,
    /// and when the integer is at or above the length of the table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^4]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    fn act(&self, faction: u16, action: u32) -> PyResult<bool> {
        let mut world = self.lock();
        if faction >= world.faction_count().max(1) {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let length = world.action_schema().length();
        if action >= length {
            return Err(VerbError::new_err(format!(
                "the action {action} is at or above the length {length} of the action table"
            )));
        }
        Ok(world.act(FactionId(faction), action))
    }

    /// Returns why one unit chose the intent it carries, as a `dict`.
    ///
    /// The unit is one soldier identity, as a Python integer.
    ///
    /// - `tile`, an integer. The row-major index of the tile the unit stands
    ///   on.
    /// - `q` and `r`, integers. The address of that tile. Pass them to
    ///   `region_summary` to read the cell the unit scored.
    /// - `cell`, an integer. The index of that cell.
    /// - `need`, an integer. What the unit still needs. **A Q16.16 value as
    ///   its raw integer. Divide by 65536.**
    /// - `scores`, `fields` and `weights`, lists of integers. One entry for
    ///   each option, in option order. The field is what the option read, and
    ///   the weight is what the option carried. The score is the weighted sum
    ///   the engine made from the two. **All three hold Q16.16 values as raw
    ///   integers.**
    /// - `floor`, an integer. The score an option had to reach. **Also
    ///   Q16.16.**
    /// - `best`, an integer. The option the scores select, or the no-intent
    ///   value when every score is below the floor.
    /// - `best_name`, a `str` or `None`. The engine's own name for that
    ///   option. It is `None` for a hold.
    /// - `intent`, an integer. The intent the unit carries now.
    /// - `chooses_next_frame`, a `bool`. Whether the unit reads the world
    ///   again on the next step.
    ///
    /// The engine recomputes the answer from the world as it stands. It
    /// stores no score, so the explanation costs nothing when nobody
    /// asks.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live unit, or when the
    /// engine would say nothing about it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    fn explain_choice<'py>(&self, python: Python<'py>, unit: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let answer = world.explain_choice(entity).ok_or_else(|| {
            ViewError::new_err(format!(
                "the engine explains nothing about the identity {unit}"
            ))
        })?;
        let tile = world.soldiers().tile(entity).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} names no live soldier"))
        })?;
        let address = world.grid().address_of(tile).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} stands on no address"))
        })?;
        let report = PyDict::new(python);
        report.set_item("tile", tile.0)?;
        report.set_item("q", address.q)?;
        report.set_item("r", address.r)?;
        report.set_item("cell", answer.cell)?;
        report.set_item("need", answer.need.0)?;
        report.set_item(
            "scores",
            answer
                .scores
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item(
            "fields",
            answer
                .fields
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item(
            "weights",
            answer
                .weights
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item("floor", answer.floor.0)?;
        report.set_item("best", answer.best)?;
        report.set_item("best_name", answer.best_name())?;
        report.set_item("intent", answer.intent)?;
        report.set_item("chooses_next_frame", answer.chooses_next_frame)?;
        Ok(report)
    }

    /// Returns the weight vector of one faction, as a `dict`.
    ///
    /// The faction is a number. The keys are `war`, `trade`, `build`,
    /// `renown` and `settle`, and every value is a whole number inside the
    /// range the balance register holds.[^1] The seeding draws the vector
    /// when the world is built, so two worlds with one seed start on one
    /// vector. `set_faction_weights` writes it after that.
    ///
    /// The renown weight is the one weight that no pass reads today. Every
    /// other weight biases one controller evaluation.
    ///
    /// **The vector is the policy of the faction, and it is simulated
    /// state.**[^2] It enters the state hash, so two worlds that differ in a
    /// weight part on the next tick.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the weight vector range. `docs/reference/balance.md`
    /// [^2]: ADR-0156, a faction's option weights are policy, set through one verb, decision D1. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    fn faction_weights<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let weights = world.faction_weights(FactionId(faction)).ok_or_else(|| {
            VerbError::new_err(format!("{faction} names no faction of this world"))
        })?;
        let report = PyDict::new(python);
        report.set_item("war", weights.war)?;
        report.set_item("trade", weights.trade)?;
        report.set_item("build", weights.build)?;
        report.set_item("renown", weights.renown)?;
        report.set_item("settle", weights.settle)?;
        Ok(report)
    }

    /// Writes the whole weight vector of one faction.
    ///
    /// **The weight a faction gives each option is that faction's policy, and
    /// this is the one verb that writes it.**[^1] A Python caller calls it.
    /// The built-in controller calls it. A learner calls it. No second path
    /// exists, so a learner may write the weights the built-in controller
    /// would never write, and that is the game played correctly.
    ///
    /// The verb writes the whole vector and never a part of it. Read the
    /// vector with `faction_weights`, change what you want, and write it
    /// back. One write and one read then hold one shape, so no caller has to
    /// know which weights the vector holds.
    ///
    /// **A weight is simulated state and it enters the state hash.**[^2] The
    /// verb takes no floating point value: every weight is a whole
    /// number.[^3]
    ///
    /// **The verb states no preference.** It says where a weight lives and
    /// who writes it. It says nothing about which option a faction should
    /// favour.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world, or
    /// when a weight lies outside the range the balance register holds.[^4]
    /// A refused write changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0156, a faction's option weights are policy, set through one verb, decision D3. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^2]: ADR-0156, a faction's option weights are policy, set through one verb, decision D1. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^4]: Balance register, the weight vector range. `docs/reference/balance.md`
    #[pyo3(signature = (faction, *, war, trade, build, renown, settle))]
    fn set_faction_weights(
        &self,
        faction: u16,
        war: u8,
        trade: u8,
        build: u8,
        renown: u8,
        settle: u8,
    ) -> PyResult<()> {
        let weights = FactionWeights {
            war,
            trade,
            build,
            renown,
            settle,
        };
        if !weights.is_inside_bound() {
            return Err(VerbError::new_err(format!(
                "every weight must lie between {WEIGHT_LOW} and {WEIGHT_HIGH}"
            )));
        }
        let mut world = self.lock();
        if world.set_faction_weights(FactionId(faction), weights) {
            Ok(())
        } else {
            Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )))
        }
    }

    /// Says whether an external caller controls a faction.
    ///
    /// **A faction under external control receives no evaluation from the
    /// controller.** The flag is off for every faction of a new world, and
    /// nothing in the engine sets it. It exists so that a later player hook
    /// has a place to stand, and so that a test can prove the controller
    /// leaves such a faction alone.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn set_externally_controlled(&self, faction: u16, controlled: bool) -> PyResult<()> {
        let mut world = self.lock();
        if !world.set_externally_controlled(FactionId(faction), controlled) {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        Ok(())
    }

    /// Returns whether an external caller controls a faction, as a `bool`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    fn is_externally_controlled(&self, faction: u16) -> PyResult<bool> {
        self.lock()
            .is_externally_controlled(FactionId(faction))
            .ok_or_else(|| VerbError::new_err(format!("{faction} names no faction of this world")))
    }

    /// The number of evaluations the controller makes for one faction on one
    /// tick, as an integer.
    ///
    /// The count is a value in the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the controller evaluations per faction per tick. `docs/reference/balance.md`
    #[getter]
    fn controller_evaluations(&self) -> u32 {
        self.lock().controller_evaluations()
    }

    /// Sets how many evaluations the controller makes for one faction on one
    /// tick.
    ///
    /// Zero silences the controller. The count is state that every tick
    /// reads, so two worlds that differ in it hash differently.
    fn set_controller_evaluations(&self, evaluations: u32) {
        self.lock().set_controller_evaluations(evaluations);
    }
}
