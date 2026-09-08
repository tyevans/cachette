//! The readers that answer about a tile, a cell or a window of tiles.
//!
//! This module holds the whole-world tile columns, the report for one tile,
//! the census of a window, the tiles of one level 1 cell and the summary of a
//! region.
//!
//! The grouping is by what the answer addresses. A method here takes a tile
//! address, a cell address or a window, and answers for the ground rather than
//! for a faction.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::resolve;
use cachette_core::census::{census, CensusError};
use cachette_core::TileIdx;
use cachette_core::{Axial, Holder, ResourceKind, TileKind};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Copies the tile value column into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`.
    ///
    /// **Each entry is a Q16.16 fixed-point value as its raw integer.**
    /// Divide by 65536 to read it as a quantity.[^1]
    ///
    /// This method copies, and it also generates. The world holds no array
    /// of tile values, so the call visits every tile. A caller that wants
    /// one tile calls `tile_report` instead.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn tile_values<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let world = self.lock();
        let raw: Vec<i32> = world
            .copy_tile_values()
            .iter()
            .map(|value| value.0)
            .collect();
        raw.to_pyarray(python)
    }

    /// Copies the tile holder column into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.uint16`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`.
    ///
    /// **Each entry is a faction number, or 65535 for a tile that nobody
    /// holds.** A faction number counts from zero. A world holds at most 63
    /// factions, so 65535 can never name one.[^1]
    ///
    /// **This is one call and it reads no tile from Python.** The engine holds
    /// the holders as one dense column, so the call copies that column. A
    /// caller that read one address at a time with `tile_report` would cross
    /// the boundary once for each tile. The control plane rule forbids that.[^2]
    ///
    /// The array covers the whole world and never a window. It has the same
    /// shape as the array that `tile_values` returns, so the two index alike.
    ///
    /// A holder changes only inside a step, so this call needs no freshness
    /// check and raises nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    fn tile_holders<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<u16>> {
        let world = self.lock();
        let raw: Vec<u16> = world
            .holding()
            .holders()
            .iter()
            .map(|holder| holder.to_bits())
            .collect();
        raw.to_pyarray(python)
    }

    /// Returns where every road runs, as a `dict` of NumPy arrays.
    ///
    /// **A road is a way and not a tile.** It runs from somewhere to
    /// somewhere, it joins another road at a junction, it bends, and it ends.
    /// A renderer that only knows which tiles carry a road can draw a
    /// coloured cell. It cannot draw a road. This answer carries the joins,
    /// so a renderer draws the ribbon that runs through the ground.
    ///
    /// The keys are:
    ///
    /// - `q` and `r`, arrays of `numpy.int32`. The address of each road tile.
    /// - `level`, an array of `numpy.uint8`. The level that stands there.
    ///   Zero means that the first level is still under construction.
    /// - `joins`, an array of `numpy.uint8`. Which of the six neighbours
    ///   carry a road, as one bit for each. Bit `i` is the neighbour in
    ///   direction `i`, in the engine's own direction order.
    ///
    /// Every array is the same length, and entry `n` of each one describes
    /// the same road tile. The order is ascending tile order, so two calls on
    /// one world answer in one order.[^1]
    ///
    /// **The cost follows the roads and not the world.** The engine stores an
    /// upgrade sparsely, so the whole road set is one slice however large the
    /// world is.[^2] A caller crosses the boundary once for the whole
    /// network, rather than asking about a tile at a time.[^3]
    ///
    /// **The derivation is the engine's own, and no caller repeats it.** The
    /// join of one road to the next is worked out in one place, so the
    /// renderers cannot draw two road networks from one world.[^4]
    ///
    /// A world in which nobody built a road answers four empty arrays.
    ///
    /// This reads the world and writes nothing to it.[^5]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^3]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^5]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    fn road_ways<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let ways = python.detach(|| {
            let world = self.lock();
            cachette_view::ways::road_ways(&world)
        });
        let mut columns = Vec::with_capacity(ways.len());
        let mut rows = Vec::with_capacity(ways.len());
        let mut levels = Vec::with_capacity(ways.len());
        let mut joins = Vec::with_capacity(ways.len());
        for way in ways {
            columns.push(way.address.q);
            rows.push(way.address.r);
            levels.push(way.level);
            joins.push(way.joins);
        }
        let answer = PyDict::new(python);
        answer.set_item("q", columns.to_pyarray(python))?;
        answer.set_item("r", rows.to_pyarray(python))?;
        answer.set_item("level", levels.to_pyarray(python))?;
        answer.set_item("joins", joins.to_pyarray(python))?;
        Ok(answer)
    }

    /// Copies the tile height column into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`. The order is the order that `tile_holders` uses, so
    /// the two arrays index alike.
    ///
    /// **Each entry is a Q16.16 fixed-point height as its raw integer.**
    /// Divide by 65536 to read it as a quantity. The boundary carries the raw
    /// integer, because a float crossing would give a caller a value the
    /// engine never held.[^1]
    ///
    /// **The height of a tile is a pure function of the seed and the
    /// address**, so this answer never changes over the life of a world.[^2]
    /// The climate leaves the height alone, so this column is the column that
    /// the level 1 summary sums into `height_total`.
    ///
    /// This method copies, and it also generates. The world stores no array
    /// of heights, so the call visits every tile.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    fn tile_heights<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let raw: Vec<i32> = python.detach(|| {
            let world = self.lock();
            let grid = world.grid();
            (0..grid.tile_count())
                .map(|index| {
                    grid.address_of(TileIdx(index))
                        .and_then(|address| world.tile_terrain(address))
                        .map_or(0, |tile| tile.height.0)
                })
                .collect()
        });
        raw.to_pyarray(python)
    }

    /// Copies the terrain kind of every tile into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.uint8`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`. The order is the order that `tile_holders` uses.
    ///
    /// **Each entry is the same number that the `kind` key of `tile_report`
    /// carries**, so a caller reads one tile and the whole world through one
    /// set of numbers. The kinds are water, plain, forest, hill and mountain,
    /// and the engine numbers them.
    ///
    /// The climate over a tile reaches the answer, so a cold cell reports the
    /// kind that the classifier gives under that climate.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    fn tile_kinds<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<u8>> {
        let raw: Vec<u8> = python.detach(|| {
            let world = self.lock();
            let grid = world.grid();
            (0..grid.tile_count())
                .map(|index| {
                    grid.address_of(TileIdx(index))
                        .and_then(|address| world.tile_kind(address))
                        .map_or(0, TileKind::to_u8)
                })
                .collect()
        });
        raw.to_pyarray(python)
    }

    /// Returns every tile index of the level 1 cell that covers one address,
    /// ascending, as `numpy.uint32`.
    ///
    /// A cell on the world edge is partial, and the array holds the tiles
    /// that exist. This is the set a land side names when it names a cell.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    fn cell_tiles<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyArray1<u32>>> {
        let world = self.lock();
        let tiles: Vec<u32> = world
            .cell_tiles(Axial::new(q, r))
            .map_err(|_| {
                ViewError::new_err(format!("the address ({q}, {r}) lies outside the world"))
            })?
            .iter()
            .map(|tile| tile.0)
            .collect();
        Ok(tiles.to_pyarray(python))
    }

    /// Returns what one tile holds, as a `dict`.
    ///
    /// The address is the column `q` and the row `r`, both counting from
    /// zero.
    ///
    /// - `q` and `r`, integers. The address the call took.
    /// - `kind`, an integer. The ground: water is zero, plain is one, forest
    ///   is two, hill is three and mountain is four.
    /// - `passable`, a `bool`. Whether the ground admits a unit.
    /// - `capacity`, an integer. How many units the tile holds.
    /// - `stock`, `generated` and `taken`, lists of integers. One entry for
    ///   each resource kind, in the order food, wood, stone. Each is a whole
    ///   count of units of stock, and none of them is fixed point.
    /// - `value`, an integer. The tile value. **A Q16.16 value as its raw
    ///   integer. Divide by 65536.**
    /// - `holder`, an integer or `None`. The faction that holds the ground,
    ///   and `None` for ground that nobody holds.[^1]
    /// - `upgrade`, an integer or `None`. The category the tile carries,
    ///   standing or under construction, and `None` for a tile that carries
    ///   none. A road is zero, a terrace is one, a wonder is two, a store is
    ///   three, a wall is four and a lodging is five.
    /// - `upgrade_level`, an integer. The level that stands on the tile, and
    ///   zero when nothing stands there yet.
    /// - `upgrade_progress`, an integer. The work that has gone into the next
    ///   level, and zero for a tile that carries none. The number never rises
    ///   above the work that the next row asks for, and it returns to zero
    ///   when the level rises.[^2]
    /// - `upgrade_complete`, a `bool`. Whether a level stands on the tile.
    ///   `False` for a tile that carries none.
    ///
    /// The four upgrade entries are what a watcher of a build reads. An
    /// unfinished upgrade changes nothing else in this report, so a caller
    /// that watches only the capacity sees nothing until the build ends.[^3]
    ///
    /// For each kind, the stock entry is the generated entry less the taken
    /// entry. The engine computes that difference.[^4]
    ///
    /// The capacity composes the ground with the finished upgrade, which is
    /// what admission reads. The binding holds neither table.[^5]
    ///
    /// **This call reports no unit.** A count of the units on a tile comes
    /// from the derived bridge. The bridge answers only after a step. A
    /// reader of the ground should not be refused because the population
    /// moved. Ask `window_census` for the units.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: Findings register, FND-011. `docs/FINDINGS.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D2 and D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    fn tile_report<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let address = Axial::new(q, r);
        let outside = || ViewError::new_err(format!("({q}, {r}) lies outside this world"));
        let kind = world.tile_kind(address).ok_or_else(outside)?;
        let capacity = world.tile_capacity(address).ok_or_else(outside)?;
        let mut stock: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        let mut generated: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        let mut taken: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        for resource in ResourceKind::ALL {
            stock.push(world.tile_stock(address, resource).ok_or_else(outside)?.0);
            generated.push(
                world
                    .original_stock(address, resource)
                    .ok_or_else(outside)?
                    .0,
            );
            taken.push(world.taken_from(address, resource).ok_or_else(outside)?.0);
        }
        let report = PyDict::new(python);
        report.set_item("q", q)?;
        report.set_item("r", r)?;
        report.set_item("kind", kind.to_u8())?;
        report.set_item("passable", kind.is_passable())?;
        report.set_item("capacity", capacity)?;
        report.set_item("stock", stock)?;
        report.set_item("generated", generated)?;
        report.set_item("taken", taken)?;
        report.set_item("value", world.tile_value(address).ok_or_else(outside)?.0)?;
        match world.tile_holder(address).and_then(Holder::faction) {
            Some(faction) => report.set_item("holder", faction.0)?,
            None => report.set_item("holder", python.None())?,
        }
        let site = world.upgrade_at(address);
        match site {
            Some(site) => report.set_item("upgrade", site.category.to_u8())?,
            None => report.set_item("upgrade", python.None())?,
        }
        report.set_item("upgrade_level", site.map_or(0, |site| site.level))?;
        report.set_item("upgrade_progress", site.map_or(0, |site| site.progress.0))?;
        report.set_item(
            "upgrade_complete",
            site.is_some_and(cachette_core::upgrade::UpgradeSite::is_complete),
        )?;
        Ok(report)
    }

    /// Returns what one window of the world holds, as a `dict`.
    ///
    /// The window is the square of the given radius around the address,
    /// clipped to the world. **The radius counts tiles.** A radius of zero
    /// reads one tile. The default radius is 8, and the ceiling is 64.
    ///
    /// - `q`, `r` and `radius`, integers. The arguments the call took.
    /// - `first_q`, `first_r`, `last_q` and `last_r`, integers. The corners
    ///   of the window after the engine clipped it to the world.
    /// - `tiles`, an integer. How many tiles the window covers.
    /// - `by_kind`, a list of integers. One count for each ground kind, in
    ///   the order water, plain, forest, hill, mountain.
    /// - `open_tiles`, an integer. How many tiles admit a unit.
    /// - `units`, an integer. How many units stand in the window.
    /// - `crowd_worst`, an integer. The largest number of units on any one
    ///   tile of the window.
    /// - `tiles_at_capacity`, an integer. How many tiles hold as many units
    ///   as their capacity.
    /// - `crowded_q` and `crowded_r`, integers or `None`. The address that
    ///   holds the most units. Both are `None` when the window holds none.
    ///
    /// The engine walks the window and answers once. A caller that walked
    /// the addresses itself would loop over the world from the control
    /// plane. This boundary does not permit that.[^1]
    ///
    /// **The cost follows the radius and never the world.** The engine
    /// refuses a radius above 64.
    ///
    /// The unit counts come from the derived unit-to-tile bridge, which
    /// rebuilds at the barrier. The engine refuses a caller that changed the
    /// population and did not step. It does not answer from a stale bridge.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the radius is above 64. The message names the
    /// ceiling. Raises `ViewError` when the window covers no
    /// address of the world, or when the bridge holds no answer.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[pyo3(signature = (q, r, radius = 8))]
    fn window_census<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
        radius: u32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let counted = census(&world, Axial::new(q, r), radius).map_err(|error| match error {
            CensusError::RadiusAboveCeiling { .. } => VerbError::new_err(error.to_string()),
            _ => ViewError::new_err(error.to_string()),
        })?;
        let report = PyDict::new(python);
        report.set_item("q", q)?;
        report.set_item("r", r)?;
        report.set_item("radius", radius)?;
        report.set_item("first_q", counted.first().q)?;
        report.set_item("first_r", counted.first().r)?;
        report.set_item("last_q", counted.last().q)?;
        report.set_item("last_r", counted.last().r)?;
        report.set_item("tiles", counted.tiles())?;
        report.set_item("by_kind", counted.by_kind().to_vec())?;
        report.set_item("open_tiles", counted.open_tiles())?;
        report.set_item("units", counted.units())?;
        report.set_item("crowd_worst", counted.crowd_worst())?;
        report.set_item("tiles_at_capacity", counted.tiles_at_capacity())?;
        match counted.crowded_most() {
            Some(address) => {
                report.set_item("crowded_q", address.q)?;
                report.set_item("crowded_r", address.r)?;
            }
            None => {
                report.set_item("crowded_q", python.None())?;
                report.set_item("crowded_r", python.None())?;
            }
        }
        Ok(report)
    }

    /// Returns the tile that one soldier stands on, as an integer.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned, or of the `unit` column of the
    /// gather log.
    ///
    /// The result is a row-major tile index. Take `index % world.width` for
    /// the column and `index // world.width` for the row.
    ///
    /// The engine resolves the identity against the arena. A soldier that
    /// died leaves its slot to another soldier. This method refuses the dead
    /// identity rather than report on the new occupant.[^1]
    ///
    /// **This read stays singular while the write verbs take a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. That value is the false
    /// answer the record forbids. The read therefore answers for one
    /// identity and says which one failed.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn soldier_tile(&self, unit: u64) -> PyResult<u32> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        world
            .soldiers()
            .tile(entity)
            .map(|tile| tile.0)
            .ok_or_else(|| ViewError::new_err(format!("the identity {unit} names no live soldier")))
    }

    /// Returns the summary of the cell that covers one tile, as a `dict`.
    ///
    /// A cell is a square block of tiles. The world summarises each block, so
    /// a reader asks about a cell without reading its tiles. Give the
    /// address of any tile, and the call answers about the cell that covers
    /// it.
    ///
    /// Every entry is a plain integer.
    ///
    /// - `tiles`. How many tiles the cell covers.
    /// - `open_tiles`. How many of them admit a unit.
    /// - `units`. How many units stand on them.
    /// - `held_tiles`. How many of them a faction holds. The entry does not
    ///   say which faction.
    /// - `value_total`. The sum of the tile values. **This is a Q16.16 value
    ///   as its raw integer. Divide by 65536.**
    /// - `height_total`. The sum of the tile heights. **This is also a Q16.16
    ///   value as its raw integer. Divide by 65536.**
    /// - `food_total`. The food the tiles still hold. **This one is a whole
    ///   count of units of stock. Do not divide it.**
    ///
    /// **Two of the three totals carry the fixed-point scale and the third
    /// does not.** A reader that divides all three reports a food total
    /// 65536 times too small.
    ///
    /// Level 0 is the only truth, and this level is derived from it. Every
    /// entry is an exact integer total over the tiles of the cell. A reader
    /// can add the tiles of the cell and get the same number back.[^1] [^2]
    ///
    /// The call reads one cell. It starts no pass over the world.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, or when
    /// the pyramid holds no cell for it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    fn region_summary<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let summary = world
            .summary_covering(Axial::new(q, r))
            .ok_or_else(|| ViewError::new_err(format!("({q}, {r}) names no cell of this world")))?;
        let fields = PyDict::new(python);
        fields.set_item("tiles", summary.tiles())?;
        fields.set_item("open_tiles", summary.open_tiles())?;
        fields.set_item("units", summary.units())?;
        fields.set_item("held_tiles", summary.held_tiles())?;
        fields.set_item("value_total", summary.value_total().0)?;
        fields.set_item("height_total", summary.height_total().0)?;
        fields.set_item("food_total", summary.food_total().0)?;
        Ok(fields)
    }
}
