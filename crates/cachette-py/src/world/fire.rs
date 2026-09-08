//! The fire on the ground, and the verbs that raise and put it out.
//!
//! This module holds the verb that ignites a set of tiles, the readers of the
//! burning and the spent tiles, the verbs that douse and stop dousing, and the
//! chance that lightning starts a fire.
//!
//! The grouping is by the subject. Fire is one state of the ground with its
//! own verbs, so it reads as one file.

use super::PyWorld;
use crate::errors::VerbError;
use crate::world::identity::resolve;
use cachette_core::Axial;
use pyo3::prelude::*;

#[pymethods]
impl PyWorld {
    /// Sets a set of tiles alight, and returns how many caught.
    ///
    /// The tiles are a sequence of `(q, r)` addresses. The call is one command
    /// over the whole set, so the control plane names the set and loops over
    /// nothing.
    ///
    /// A tile does not catch when its ground carries no fuel, when the weather
    /// has wetted it, when it already burns, and when it has burned already.
    /// Open water and bare rock therefore never catch.
    ///
    /// The count is how many of the named tiles caught. A caller that named
    /// one tile and got zero named ground that will not burn.
    fn ignite(&self, tiles: Vec<(i32, i32)>) -> PyResult<usize> {
        let mut world = self.lock();
        let grid = world.grid();
        let mut indices = Vec::with_capacity(tiles.len());
        for (q, r) in &tiles {
            let address = Axial::new(*q, *r);
            let index = grid.index_of(address).ok_or_else(|| {
                VerbError::new_err(format!("the address ({q}, {r}) lies outside the world"))
            })?;
            indices.push(index);
        }
        Ok(world.ignite_set(&indices))
    }

    /// The tiles that burn now, as a list of `(q, r)` addresses.
    ///
    /// The list is in ascending tile order. A caller hands it straight to
    /// `send_units_to` as the seed set of a destination plane, so no caller
    /// walks a tile of its own.
    fn burning_tiles(&self) -> Vec<(i32, i32)> {
        let world = self.lock();
        let grid = world.grid();
        world
            .burning_tiles()
            .into_iter()
            .filter_map(|tile| grid.address_of(tile))
            .map(|address| (address.q, address.r))
            .collect()
    }

    /// The number of tiles that burn now, as an integer.
    #[getter]
    fn burning_tile_count(&self) -> usize {
        self.lock().fire().burning_count()
    }

    /// The number of tiles that have burned over the life of the world.
    ///
    /// Ground that has burned never catches again, and that is what makes a
    /// fire end rather than run for ever.
    #[getter]
    fn spent_tile_count(&self) -> usize {
        self.lock().fire().spent().len()
    }

    /// Orders every soldier the identities name to fight the fire, and sends
    /// the whole set at it.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. The destination is the
    /// number of the plane the engine seeds with every burning tile.
    ///
    /// **This is one command over a set, and it starts one field.** Every unit
    /// then climbs that field, so no unit searches for a fire and the caller
    /// walks nothing. Call it again on a later tick to re-aim the crew at the
    /// fire as it moves.
    ///
    /// A unit under this order that stands on a burning tile stays there, it
    /// takes intensity off the fire, and it risks its life. Returns `None`.
    #[pyo3(signature = (units, destination = 0))]
    fn order_douse(&self, units: Vec<u64>, destination: u16) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        world
            .order_douse_set(&resolved, destination)
            .map(|_| ())
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Tells every soldier the identities name to stop fighting the fire.
    ///
    /// The units are a sequence of identities. The order stops, and the unit
    /// keeps whatever send it holds. Returns `None`.
    fn stop_dousing(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for unit in resolved {
            world.order_douse(unit, false);
        }
        Ok(())
    }

    /// The chance that lightning starts a fire on one tick, as an integer.
    ///
    /// The chance is stated out of one million. A world answers zero until a
    /// caller says otherwise, so a world never catches by itself unless
    /// somebody asks for it.
    #[getter]
    fn lightning_chance(&self) -> u64 {
        self.lock().lightning_chance()
    }

    /// Sets the chance that lightning starts a fire on one tick.
    ///
    /// The chance is stated out of one million. A value of one thousand gives
    /// about one strike in a thousand ticks. The engine clamps a value above
    /// one million. Returns `None`.
    fn set_lightning_chance(&self, chance: u64) {
        self.lock().set_lightning_chance(chance);
    }
}
