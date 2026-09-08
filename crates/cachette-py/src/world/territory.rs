//! The readers that answer who holds ground and who may stand on it.
//!
//! This module holds the holder of one tile, the reach of one city, the rule
//! that admits a guest to a host's territory, and the mask of factions present
//! on each tile.
//!
//! The grouping is by the question. Each method here answers about a claim on
//! ground rather than about the ground itself.

use super::PyWorld;
use crate::errors::ViewError;
use crate::world::identity::resolve_site;
use cachette_core::{Axial, FactionId};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;

#[pymethods]
impl PyWorld {
    /// Reports whether a unit of one faction stands on ground another holds.
    ///
    /// `guest` is the faction whose units the question is about. `host` is the
    /// faction that holds the ground. Both are faction numbers, counted from
    /// zero. Returns a `bool`.
    ///
    /// The call answers one entry of `presence_masks`. It costs the same. Ask
    /// this one about one pair. Ask `presence_masks` about several.
    ///
    /// **The answer is `False` when `guest` and `host` name one faction.** A
    /// faction is never a guest on its own ground.
    ///
    /// The answer states the world as the last step left it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the world holds no such faction. The message
    /// names the number that refused. Raises `ViewError` when the population
    /// changed since the last step.
    fn stands_in_territory(&self, guest: u16, host: u16) -> PyResult<bool> {
        let world = self.lock();
        let factions = world.config().faction_count;
        for (name, number) in [("guest", guest), ("host", host)] {
            if number >= factions {
                return Err(ViewError::new_err(format!(
                    "the {name} faction {number} is outside a world of {factions} factions"
                )));
            }
        }
        world
            .stands_in_territory(FactionId(guest), FactionId(host))
            .map_err(|error| ViewError::new_err(error.to_string()))
    }

    /// Reports whether the speaker has a unit on the ground the listener
    /// holds, as a boolean.
    ///
    /// This is the gate that every trade verb passes. A player speaks to
    /// another player only while one of its own units stands in that
    /// player's territory.
    ///
    /// The read costs one column read for each unit alive. It answers one
    /// ordered pair. A caller makes this read before it offers a trade.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when a number names no faction of this world.
    fn stands_in_territory_of(&self, speaker: u16, listener: u16) -> PyResult<bool> {
        let world = self.lock();
        let count = world.faction_count();
        if speaker >= count || listener >= count {
            return Err(ViewError::new_err(format!(
                "the pair {speaker} and {listener} names no faction of this world"
            )));
        }
        Ok(world.stands_in_territory_of(FactionId(speaker), FactionId(listener)))
    }

    /// Reports whether one faction holds one tile.
    ///
    /// `faction` is a faction number, counted from zero. `q` and `r` are the
    /// address of the tile. Returns a `bool`.
    ///
    /// **A faction holds the ground its cities reach.** A tile that no city
    /// of the faction reaches is not held by it, whatever stands on the
    /// tile.[^1]
    ///
    /// The answer states the world as the last step left it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the world holds no such faction, and when the
    /// address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    fn holds(&self, faction: u16, q: i32, r: i32) -> PyResult<bool> {
        let world = self.lock();
        let factions = world.config().faction_count;
        if faction >= factions {
            return Err(ViewError::new_err(format!(
                "the faction {faction} is outside a world of {factions} factions"
            )));
        }
        world
            .holds(FactionId(faction), Axial::new(q, r))
            .ok_or_else(|| {
                ViewError::new_err(format!("the address ({q}, {r}) is outside the world"))
            })
    }

    /// Returns how far one city reaches, in hex steps.
    ///
    /// `site` is a settlement identity, as `found_settlements` returns them.
    /// Returns an `int`.
    ///
    /// **The reach is a base plus one step for each block of finished
    /// upgrades on the ground the city held at the end of the previous
    /// step, and it never passes a bound.**[^1] The three values are balance
    /// values, and the register holds the rows.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: Balance register, the holding. `docs/reference/balance.md`
    fn city_reach(&self, site: u64) -> PyResult<u32> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        world
            .city_reach(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live city")))
    }

    /// Returns which factions stand on the ground of which other factions.
    ///
    /// **This is one call and it names no unit.** It answers for every pair of
    /// factions at once. A caller that walked the population to reach the same
    /// answer would cross the boundary twice for each unit. The control plane
    /// rule forbids that.[^1]
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one entry for
    /// each faction the world was built with. Entry `host` is a set of
    /// factions, held as one bit for each faction. Bit `guest` is one when a
    /// live unit of faction `guest` stands on a tile that faction `host`
    /// holds.
    ///
    /// ```python
    /// presence = world.presence_masks()
    /// may_speak = bool(presence[other_god] & (1 << my_god))
    /// ```
    ///
    /// **The size of the answer does not change when the population changes.**
    /// A faction is one bit of a 64-bit word. A world holds at most 63
    /// factions, so the whole relation is one word for each faction.[^2]
    ///
    /// **A unit that stands on ground its own faction holds sets no bit.** The
    /// question is whether the people of one side stand on the ground of
    /// another side. Bit `host` of entry `host` is therefore always zero.
    ///
    /// **The answer is exact.** The engine reads the holder of the exact tile
    /// that each unit stands on. No summary reaches the answer. A bit that is
    /// zero means that no unit of that faction is there.
    ///
    /// The answer states the world as the last step left it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the population changed since the last step. A
    /// call to `spawn_soldiers` or to `despawn_soldiers` makes the answer
    /// stale. The engine refuses rather than answering. Call `step` and ask
    /// again.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn presence_masks<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let factions = world.config().faction_count as usize;
        let raw: Vec<u64> = world
            .presence_rows()
            .map_err(|error| ViewError::new_err(error.to_string()))?
            .iter()
            .take(factions)
            .map(|row| row.to_bits())
            .collect();
        Ok(raw.to_pyarray(python))
    }
}
