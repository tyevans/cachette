//! The relation between two factions, and the campaigns it raises.
//!
//! This module holds the relation reader and its band, the verbs that set and
//! move a relation, and the campaign a faction raises against another.
//!
//! A relation is a plane over ordered pairs of factions.[^1] A campaign reads
//! that plane, so the two sit together.
//!
//! # References
//!
//! [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`

use super::PyWorld;
use crate::errors::VerbError;
use crate::world::identity::resolve;
use cachette_core::campaign::CampaignRow;
use cachette_core::TileIdx;
use cachette_core::{Axial, FactionId};
use numpy::ToPyArray;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Returns what one faction feels toward another, as an integer.
    ///
    /// The relation is one signed whole number for each ordered pair. The
    /// entry for `(a, b)` is what `a` feels toward `b`, and `(b, a)` is a
    /// separate entry. A new world holds every pair at the peace edge. The
    /// edges that cut the range into bands are rows of the balance
    /// register.[^1] [^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decisions D1 and D2. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^2]: Balance register, the relation. `docs/reference/balance.md`
    fn relation(&self, from_faction: u16, to_faction: u16) -> PyResult<i32> {
        if from_faction == to_faction {
            return Err(VerbError::new_err(
                "a faction holds no relation toward itself",
            ));
        }
        self.lock()
            .relation(FactionId(from_faction), FactionId(to_faction))
            .ok_or_else(|| VerbError::new_err("a number names no faction of this world"))
    }

    /// Returns the band of what one faction feels toward another, as an
    /// integer.
    ///
    /// The number counts the edges at or below the value. Zero is below the
    /// war edge, one is at or above it and below the peace edge, two is at or
    /// above the peace edge and below the alliance edge, and three is at or
    /// above the alliance edge. The engine holds no band name.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn relation_band(&self, from_faction: u16, to_faction: u16) -> PyResult<u8> {
        if from_faction == to_faction {
            return Err(VerbError::new_err(
                "a faction holds no relation toward itself",
            ));
        }
        self.lock()
            .relation_band(FactionId(from_faction), FactionId(to_faction))
            .ok_or_else(|| VerbError::new_err("a number names no faction of this world"))
    }

    /// Writes what one faction feels toward another, outright.
    ///
    /// **This is the caller's own path and it holds no gate.** A god sets the
    /// relation a scenario starts from. A crossing of the war edge is logged
    /// as any other cause logs it. Returns `None`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    fn set_relation(&self, from_faction: u16, to_faction: u16, value: i32) -> PyResult<()> {
        if !self
            .lock()
            .set_relation(FactionId(from_faction), FactionId(to_faction), value)
        {
            return Err(VerbError::new_err(
                "a number names no faction of this world, or the two numbers name one faction",
            ));
        }
        Ok(())
    }

    /// Moves what the faction of a speaker unit feels toward another faction
    /// by a bounded step, and returns the value after the move.
    ///
    /// **The verb refuses a speaker whose type has a command reach of
    /// zero.** The gate reads the type column of the unit and no per-faction
    /// flag.[^1] The step is bounded in either direction, and the bound is a
    /// row of the balance register.[^2] A leader may always declare, so the
    /// verb reads no band before it moves.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier. Raises
    /// `VerbError` when the faction number names no faction, when it names
    /// the speaker's own faction, when the speaker's type has no command
    /// reach, and when the step is above the bound.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn move_relation(&self, speaker: u64, faction: u16, step: i32) -> PyResult<i32> {
        let mut world = self.lock();
        let entity = resolve(&world, speaker)?;
        world
            .move_relation(entity, FactionId(faction), step)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Raises a campaign: takes the idle units of a faction, makes them
    /// soldiers and sends them at an objective tile. Returns `None`.
    ///
    /// **This is the one path the controller and a caller share.** The raise
    /// acts through the set form of the type verb and through the send verb,
    /// on the destination plane whose number is the faction number, and it
    /// writes one row of the campaign register.[^1]
    ///
    /// An idle unit is a live unit of the faction that nobody has sent
    /// anywhere. The cohort is the lowest identities among them, up to
    /// `cohort`. A settlement of the faction itself makes a relief, and any
    /// other tile makes a take. The campaign closes when the holder of the
    /// tile changes or when every unit of the cohort has fallen.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction, when the address
    /// is outside the world, when `cohort` is zero, when the faction holds a
    /// live campaign, when it has no idle unit, or when the world holds no
    /// destination plane for it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn raise_campaign(
        &self,
        faction: u16,
        objective_q: i32,
        objective_r: i32,
        cohort: u32,
    ) -> PyResult<()> {
        self.lock()
            .raise_campaign(
                FactionId(faction),
                Axial::new(objective_q, objective_r),
                cohort,
            )
            .map(|_| ())
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the campaign register of one faction, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays, one entry for
    /// each row of the register of the faction, in slot order. A row whose
    /// `state` is zero holds no campaign. The keys are:
    ///
    /// - `raised_at_tick`, `numpy.uint64`. The tick the campaign was raised on.
    /// - `objective_q` and `objective_r`, `numpy.int32`. The objective tile.
    /// - `cohort_size`, `numpy.uint32`. How many units the raise took.
    /// - `holder_at_raise`, `numpy.uint16`. The faction that held the
    ///   objective at the raise, or the largest value when nobody did.
    /// - `objective_kind`, `numpy.uint8`. Zero takes a site, one relieves an
    ///   own site, two wears an upgrade.
    /// - `state`, `numpy.uint8`. Zero is empty, one is live, two is won,
    ///   three is lost, four is ended by a holder change to a third party.
    ///
    /// This method copies each column.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn campaigns<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let rows: Vec<CampaignRow> = world.campaigns_of(FactionId(faction)).to_vec();
        if rows.is_empty() {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let grid = world.grid();
        let address = |tile: u32| grid.address_of(TileIdx(tile)).unwrap_or(Axial::new(0, 0));
        let columns = PyDict::new(python);
        let raised_at: Vec<u64> = rows.iter().map(|row| row.raised_at.0).collect();
        let q: Vec<i32> = rows
            .iter()
            .map(|row| address(row.objective_tile).q)
            .collect();
        let r: Vec<i32> = rows
            .iter()
            .map(|row| address(row.objective_tile).r)
            .collect();
        let cohort: Vec<u32> = rows.iter().map(|row| row.cohort_size).collect();
        let holder: Vec<u16> = rows.iter().map(|row| row.holder_at_raise).collect();
        let kind: Vec<u8> = rows.iter().map(|row| row.objective_kind).collect();
        let state: Vec<u8> = rows.iter().map(|row| row.state).collect();
        columns.set_item("raised_at_tick", raised_at.to_pyarray(python))?;
        columns.set_item("objective_q", q.to_pyarray(python))?;
        columns.set_item("objective_r", r.to_pyarray(python))?;
        columns.set_item("cohort_size", cohort.to_pyarray(python))?;
        columns.set_item("holder_at_raise", holder.to_pyarray(python))?;
        columns.set_item("objective_kind", kind.to_pyarray(python))?;
        columns.set_item("state", state.to_pyarray(python))?;
        Ok(columns)
    }

    /// How many units the controller takes when it raises a campaign, as an
    /// integer. The value is a row of the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the campaign cohort size. `docs/reference/balance.md`
    #[getter]
    fn campaign_cohort_size(&self) -> u32 {
        self.lock().campaign_cohort_size()
    }

    /// Sets how many units the controller takes when it raises a campaign.
    /// Returns `None`.
    fn set_campaign_cohort_size(&self, cohort: u32) {
        self.lock().set_campaign_cohort_size(cohort);
    }
}
