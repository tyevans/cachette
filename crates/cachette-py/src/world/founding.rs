//! The verbs and readers that seat a faction on the ground.
//!
//! This module holds the survey that scores a place, the verbs that found a
//! group and a run, the settlement positions, the preference a caller writes
//! on a site, and the schedule the position pass runs on.
//!
//! The grouping follows one sequence. A caller surveys, founds, then reads
//! where the settlements stand.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::resolve_site;
use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, FactionId, Fix32, ResourceKind};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// The number of settlements standing in the world, as an integer.
    ///
    /// The count covers the settlements that stand now. A settlement the
    /// world removes no longer counts.
    #[getter]
    fn settlement_count(&self) -> u32 {
        self.lock().settlements().len()
    }

    /// Founds a settlement at each address and returns their identities.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. The
    /// faction is the number of the faction that owns the new settlements.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each address, in the order of the addresses. Pass an entry as a
    /// Python integer to `site_economy`, `site_positions`, `site_preference`
    /// or `prefer_at_sites`.
    ///
    /// **A settlement founded here earns nothing.** The production rate comes
    /// from the ground that a survey read. This call runs no survey. Call
    /// `found_group` for a settlement that produces.[^1]
    ///
    /// **The set is all or nothing.** When the world refuses an address, the
    /// call destroys every settlement it made and raises.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full, when an address is outside
    /// the world, when the ground admits nobody, when the world has no such
    /// faction, or when a settlement already stands on the tile. The error
    /// names the address that refused.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    fn found_settlements<'py>(
        &self,
        python: Python<'py>,
        addresses: Vec<(i32, i32)>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut made: Vec<u64> = Vec::with_capacity(addresses.len());
        for (q, r) in addresses {
            match world.found_settlement(Axial::new(q, r), FactionId(faction)) {
                Ok(site) => made.push(site.to_bits()),
                Err(error) => {
                    // Leave nothing half-made, in the same way the soldier
                    // spawn does.
                    for site in &made {
                        let entity = world
                            .resolve_settlement(*site)
                            .expect("this call made the identity a moment ago");
                        world.destroy_settlement(entity);
                    }
                    return Err(VerbError::new_err(format!(
                        "the address ({q}, {r}) refused a settlement: {error}"
                    )));
                }
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Changes what a set of sites wants of one kind of work.
    ///
    /// **The command names no unit.** It says what a place wants. The engine
    /// turns that into a number of positions of each kind at the next
    /// rebalance. A caller that named the workers would be looping over
    /// entities, and the control plane never does that.[^1]
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned. Returns `None`.
    ///
    /// The kind is the kind of work: food is zero, wood is one and stone is
    /// two. It is the same numbering the gather log uses in its `kind`
    /// column.
    ///
    /// **The target is a Q16.16 value as its raw integer.** Multiply the
    /// share you want by 65536. A new site wants 65536, which is one, of
    /// every kind. The engine holds no floating point number in simulated
    /// state, because float addition is not associative.[^2]
    ///
    /// The engine acts on the new target at the next rebalance.
    /// `set_position_schedule` says how often that runs.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the target
    /// is checked, before anything is written.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the number names no kind, or when the target is
    /// below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    fn prefer_at_sites(&self, sites: Vec<u64>, kind: u8, target: i32) -> PyResult<()> {
        let mut world = self.lock();
        let kind = ResourceKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no kind of work")))?;
        let mut resolved = Vec::with_capacity(sites.len());
        for site in &sites {
            resolved.push(resolve_site(&world, *site)?);
        }
        world
            .prefer_at_sites(&resolved, kind, Fix32(target))
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the positions that one site holds, as a `dict` of NumPy
    /// arrays.
    ///
    /// A position is one seat of work at a settlement. The site is one
    /// settlement identity, as a Python integer.
    ///
    /// Every array has one entry for each position. All three arrays are the
    /// same length.
    ///
    /// - `kind`, `numpy.uint8`. The kind of work: food is zero, wood is one
    ///   and stone is two.
    /// - `rank`, `numpy.uint8`. The rank of the position inside its kind,
    ///   counting from zero.
    /// - `holder`, `numpy.uint64`. The identity of the unit that holds the
    ///   position, and zero where a position holds nobody.
    ///
    /// The columns hold the positions of the site and nothing else. An entry
    /// that is no position does not appear.
    ///
    /// **A site holds no position until a rebalance runs.** A site founded in
    /// this step reports three empty arrays. Step the world. Read
    /// `set_position_schedule` for how often the rebalance runs.
    ///
    /// The holder column carries the whole identity of the unit that holds
    /// each position. It is zero where a position holds nobody. It is not a
    /// slot index.[^1]
    ///
    /// **This read stays singular while the write verb takes a set.** A set
    /// form would have to answer for a dead identity with a value that stands
    /// for nothing. The unit read follows the same rule.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn site_positions<'py>(&self, python: Python<'py>, site: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let row = world
            .site_positions(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let held: Vec<_> = row.iter().filter(|entry| entry.exists()).collect();
        let columns = PyDict::new(python);
        let kind: Vec<u8> = held.iter().map(|entry| entry.kind_number()).collect();
        let rank: Vec<u8> = held.iter().map(|entry| entry.rank()).collect();
        let holder: Vec<u64> = held.iter().map(|entry| entry.holder_bits()).collect();
        columns.set_item("kind", kind.to_pyarray(python))?;
        columns.set_item("rank", rank.to_pyarray(python))?;
        columns.set_item("holder", holder.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns what one site wants of each kind of work, as a NumPy array.
    ///
    /// The site is one settlement identity, as a Python integer.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// resource kind, in the order food, wood, stone.
    ///
    /// **Each entry is a Q16.16 value as its raw integer.** Divide by 65536.
    /// A new site wants 65536, which is one, of every kind. Write a target
    /// with `prefer_at_sites`.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    fn site_preference<'py>(
        &self,
        python: Python<'py>,
        site: u64,
    ) -> PyResult<Bound<'py, PyArray1<i32>>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let preference = world
            .site_preference(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let raw: Vec<i32> = ResourceKind::ALL
            .iter()
            .map(|kind| preference.target(*kind).0)
            .collect();
        Ok(raw.to_pyarray(python))
    }

    /// Sets how often the engine rebalances the positions of every site.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one rebalances on every step. The phase is the offset inside the
    /// period. With a period of four and a phase of one, the engine
    /// rebalances on ticks one, five and nine. A phase at or above the
    /// period wraps into it.
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// A rebalance turns the targets that `prefer_at_sites` wrote into the
    /// positions that `site_positions` reports.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that
    /// the scaling multiply takes. The message names the limit.
    fn set_position_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_position_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the survey the engine would run for this group and faction,
    /// as a `dict`.
    ///
    /// The group is the number of people that would settle. The faction is
    /// the number of the faction that would settle them. The faction chooses
    /// the sample, so two factions read two samples.[^1]
    ///
    /// Eleven entries are NumPy arrays with one entry for each candidate
    /// place, and three are plain integers.
    ///
    /// - `q` and `r`, `numpy.int32`. The address of the candidate place.
    /// - `score`, `numpy.int64`. The weighted sum that ranks the place.
    ///   **This is a Q16.16 value as its raw integer. Divide by 65536.**
    /// - `food`, `wood` and `stone`, `numpy.uint32`. How much of each
    ///   resource the disc around the place holds. These are whole units of
    ///   stock, and they are not fixed point.
    /// - `open_ground`, `numpy.uint32`. How many tiles of the disc admit a
    ///   unit.
    /// - `room`, `numpy.uint32`. How many units the open tiles of the disc
    ///   hold together.
    /// - `water_edge`, `numpy.uint32`. How many of the six neighbours of the
    ///   centre hold open water.
    /// - `eligible`, `numpy.uint8`. One when the founding would accept the
    ///   place, and zero when it refuses it.
    /// - `separated`, `numpy.uint8`. One when the place keeps its distance
    ///   from every place a founding before it took.
    /// - `drawn`, a plain integer. How many candidate places the survey drew.
    /// - `considered`, a plain integer. How many distinct places it read. Two
    ///   draws may name one tile, and the survey reads such a tile once.
    /// - `tiles_read`, a plain integer. How many tiles it read in all.
    ///
    /// The survey draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each one. Neither number grows with the size
    /// of the world, so this call costs the same in a large world as in a
    /// small one.[^2]
    ///
    /// The call writes nothing, and it founds nothing. It shows how the
    /// engine makes the score: the columns hold the counts the survey read,
    /// and the score column holds the engine's weighted sum of them.[^3]
    ///
    /// The rows are the candidates, in the order the founding ranks them,
    /// best first. Row zero is the place a founding would take. A row whose
    /// `eligible` entry is zero is a place the founding refuses.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, or when the ordering
    /// of the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^3]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    fn founding_survey<'py>(
        &self,
        python: Python<'py>,
        group: u32,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let survey = world
            .survey_founding(group, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        let ranked = survey.candidates();
        let columns = PyDict::new(python);
        let q: Vec<i32> = ranked.iter().map(|place| place.address().q).collect();
        let r: Vec<i32> = ranked.iter().map(|place| place.address().r).collect();
        let score: Vec<i64> = ranked.iter().map(|place| place.score().0).collect();
        let food: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().food.0)
            .collect();
        let wood: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().wood.0)
            .collect();
        let stone: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().stone.0)
            .collect();
        let open_ground: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().open_ground)
            .collect();
        let room: Vec<u32> = ranked.iter().map(|place| place.provision().room).collect();
        let water_edge: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().water_edge)
            .collect();
        let eligible: Vec<u8> = ranked
            .iter()
            .map(|place| u8::from(place.is_eligible()))
            .collect();
        let separated: Vec<u8> = ranked
            .iter()
            .map(|place| u8::from(place.is_separated()))
            .collect();
        columns.set_item("q", q.to_pyarray(python))?;
        columns.set_item("r", r.to_pyarray(python))?;
        columns.set_item("score", score.to_pyarray(python))?;
        columns.set_item("food", food.to_pyarray(python))?;
        columns.set_item("wood", wood.to_pyarray(python))?;
        columns.set_item("stone", stone.to_pyarray(python))?;
        columns.set_item("open_ground", open_ground.to_pyarray(python))?;
        columns.set_item("room", room.to_pyarray(python))?;
        columns.set_item("water_edge", water_edge.to_pyarray(python))?;
        columns.set_item("eligible", eligible.to_pyarray(python))?;
        columns.set_item("separated", separated.to_pyarray(python))?;
        // The survey counts these as it reads. They are measurements of the
        // run and not a second copy of the sample size.
        columns.set_item("drawn", survey.drawn())?;
        columns.set_item("considered", survey.considered())?;
        columns.set_item("tiles_read", survey.tiles_read())?;
        Ok(columns)
    }

    /// Founds a group the way the engine founds one, and reports what it
    /// chose, as a `dict`.
    ///
    /// The group is the number of people to settle. The faction is the number
    /// of the faction that settles them.
    ///
    /// Every entry is a plain integer.
    ///
    /// - `site`. The identity of the settlement the founding made. Pass it to
    ///   `site_economy`, `site_positions` or `prefer_at_sites`.
    /// - `q` and `r`. The address the founding took.
    /// - `faction`. The faction the call took.
    /// - `seated`. **How many people the founding seated**, and not whether
    ///   it seated any. The report of `found_run_for_every_faction` uses the
    ///   same key for a `bool`.
    /// - `score`. The weighted sum of the chosen place. **This is a Q16.16
    ///   value as its raw integer. Divide by 65536.**
    /// - `food`, `wood`, `stone`, `open_ground`, `room` and `water_edge`.
    ///   What the survey read at the chosen place. Each is a whole count, and
    ///   `founding_survey` describes each one.
    /// - `drawn`, `considered` and `tiles_read`. What the survey did.
    ///
    /// This is the whole loop in one call. The survey reads the ground. The
    /// founding takes the best place the sample offered. It seats the group
    /// over the disc around that place. It sets the production rate of the
    /// site from the food the survey read.[^1] [^2] A caller that founds at
    /// an address of its own gets a site that earns nothing. The rate comes
    /// from the survey.
    ///
    /// Every number in the report is the engine's own. This binding
    /// recomputes no score.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, when the world has no
    /// such faction, when the sample offered no place that admits the group,
    /// or when the seating refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    fn found_group<'py>(
        &self,
        python: Python<'py>,
        group: u32,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut world = self.lock();
        let founding = world
            .found_run(group, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        let survey = founding.survey();
        let chosen = survey.chosen().ok_or_else(|| {
            VerbError::new_err("the founding reports no chosen place".to_string())
        })?;
        let provision = chosen.provision();
        let report = PyDict::new(python);
        report.set_item("site", founding.settlement().to_bits())?;
        report.set_item("q", founding.place().q)?;
        report.set_item("r", founding.place().r)?;
        report.set_item("faction", faction)?;
        report.set_item("seated", founding.people().len())?;
        report.set_item("score", chosen.score().0)?;
        report.set_item("food", provision.food.0)?;
        report.set_item("wood", provision.wood.0)?;
        report.set_item("stone", provision.stone.0)?;
        report.set_item("open_ground", provision.open_ground)?;
        report.set_item("room", provision.room)?;
        report.set_item("water_edge", provision.water_edge)?;
        report.set_item("drawn", survey.drawn())?;
        report.set_item("considered", survey.considered())?;
        report.set_item("tiles_read", survey.tiles_read())?;
        Ok(report)
    }

    /// Founds one run for every faction and keeps the report.
    ///
    /// **This is a set-valued command, not a loop.** One call seats every
    /// faction the world has. A founding must keep its distance from the
    /// foundings before it. A caller that founded one faction at a time would
    /// carry that state across the boundary itself.
    ///
    /// The binding keeps the report. The frame marks each founded place, and
    /// the panel names each refusal. A founded place is history, and the
    /// engine holds no copy of it.
    ///
    /// The group is how many people to seat for each faction. It defaults to
    /// 64.
    ///
    /// Returns a `list` with one `dict` for each faction of the world, in
    /// faction order. Every entry is a plain integer, a `bool` or a `str`.
    ///
    /// Every report holds these two.
    ///
    /// - `faction`, an integer. The faction the report is about.
    /// - `seated`, a `bool`. **Whether the faction got a place**, and not how
    ///   many people it seated. The report of `found_group` uses the same key
    ///   for a count.
    ///
    /// A report whose `seated` entry is `True` holds these as well.
    ///
    /// - `q` and `r`, integers. The address the founding took.
    /// - `people`, an integer. How many people it seated.
    /// - `considered`, an integer. How many distinct places the survey read.
    /// - `food`, `wood`, `stone`, `open_ground` and `water_edge`, integers.
    ///   What the survey read at the chosen place. Each is a whole count.
    /// - `carries_its_group`, a `bool`. Whether the food the survey read
    ///   holds the whole group.
    ///
    /// A report whose `seated` entry is `False` holds `refusal` instead, a
    /// `str` that says why the faction got no place.
    ///
    /// **A report holds no site identity.** Call `found_group` for a founding
    /// that hands one back.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when no faction was seated. A run with no group is
    /// not a run.
    #[pyo3(signature = (group = cachette_core::FOUNDING_GROUP_DEFAULT))]
    fn found_run_for_every_faction<'py>(
        &self,
        python: Python<'py>,
        group: u32,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let outcomes = {
            let mut world = self.lock();
            world.found_run_for_every_faction(group)
        };
        founding_reports(self, python, outcomes, group)
    }

    /// Seeds the world from its seed: founds one run for every faction and
    /// places the luxuries. Takes nothing.
    ///
    /// **A caller that builds a world from a seed calls this once and names
    /// no group and no place.** The founding group and the deposit count are
    /// values in the balance register, and the engine holds them.[^1] The
    /// founding is the one `found_run_for_every_faction` makes with the
    /// default group, and the luxuries are placed by a keyed draw on the
    /// deposit index, so two worlds with one seed seed alike.[^2]
    ///
    /// Returns the same `list` of founding reports that
    /// `found_run_for_every_faction` returns, and keeps the report for the
    /// panel in the same way.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the world was seeded before, or when no
    /// faction was seated.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the founding group and the luxury deposits. `docs/reference/balance.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    fn seed_world<'py>(&self, python: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let outcomes = {
            let mut world = self.lock();
            world
                .seed_world()
                .map_err(|error| VerbError::new_err(error.to_string()))?
        };
        founding_reports(
            self,
            python,
            outcomes,
            cachette_core::FOUNDING_GROUP_DEFAULT,
        )
    }
}

/// Turns founding outcomes into the reports a caller prints, and keeps the
/// outcomes for the panel.
fn founding_reports<'py>(
    world: &PyWorld,
    python: Python<'py>,
    outcomes: Vec<FoundingOutcome>,
    group: u32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    if !outcomes.iter().any(FoundingOutcome::is_seated) {
        return Err(VerbError::new_err(
            "no faction found a place, so the run has nothing in it".to_string(),
        ));
    }

    let mut reports = Vec::with_capacity(outcomes.len());
    for outcome in &outcomes {
        let report = PyDict::new(python);
        report.set_item("faction", outcome.faction().0)?;
        match outcome.result() {
            Ok(founding) => {
                let place = founding.place();
                report.set_item("seated", true)?;
                report.set_item("q", place.q)?;
                report.set_item("r", place.r)?;
                report.set_item("people", founding.people().len())?;
                report.set_item("considered", founding.survey().considered())?;
                if let Some(chosen) = founding.survey().chosen() {
                    let reached = chosen.provision();
                    report.set_item("food", reached.food.0)?;
                    report.set_item("wood", reached.wood.0)?;
                    report.set_item("stone", reached.stone.0)?;
                    report.set_item("open_ground", reached.open_ground)?;
                    report.set_item("water_edge", reached.water_edge)?;
                    // The food the survey reached is the number of people
                    // the site can carry, because the production rate and
                    // the ration are both a sixteenth and the two cancel.
                    report.set_item("carries_its_group", reached.food.0 >= group)?;
                }
            }
            Err(error) => {
                report.set_item("seated", false)?;
                report.set_item("refusal", error.to_string())?;
            }
        }
        reports.push(report);
    }
    world.presenter().outcomes = outcomes;
    Ok(reports)
}
