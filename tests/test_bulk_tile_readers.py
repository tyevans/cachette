"""Four bulk tile readers, and the reader each one must agree with.

The engine holds the height, the ground kind, the cloud and the wind of every
tile. A caller that reads one tile at a time crosses the boundary once for each
tile, and the control plane rule forbids that. These four readers answer the
whole world in one crossing.

Each test drives the reader that already answers the same fact for one tile or
for one cell, and compares the two. A bulk reader that agrees with nothing can
drift away from the engine, and nothing fails when it does.

Every test goes through the published interface.

References
----------
Testing rules, sections 2 and 6. ``.agents/rules/testing.md``
Findings register, FND-569. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import numpy as np

import cachette

EXTENT = 64
"""The side of the world that most tests here build."""

SEED = 2
"""A seed that gives a world with no open water. Only a god wets it."""

COASTAL_SEED = 0x0123456789ABCDEF
"""A seed that gives a world with open water. The sea lifts water on its own."""

PLANET_CENTRE = 0
"""The centre latitude of a whole planet, in hundredths of a degree."""

PLANET_SPAN = 18000
"""The latitude span of a whole planet, in hundredths of a degree.

**The span decides how much of the sky a world clouds.** A world is one region
of a planet unless the caller states otherwise. The resting sky of a region
clouds every tile of this world, and it blows no wind at the middle of it. The
tests that need a dry sky or a moving wind state the span of the planet, as the
constructor documents it.
"""


def _index_of(q: int, r: int, width: int = EXTENT) -> int:
    """Return the row-major index of one address."""
    return r * width + q


def test_every_column_holds_one_integer_entry_for_each_tile() -> None:
    world = cachette.World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    holders = world.tile_holders()
    heights = world.tile_heights()
    kinds = world.tile_kinds()
    clouds = world.cloud_shares()
    winds = world.tile_winds()
    for column in (heights, kinds, clouds, winds["q"], winds["r"]):
        assert column.shape == holders.shape, "every column covers the world once"
        assert column.dtype.kind in "iu", "no column is a floating point array"
    assert heights.dtype.name == "int32"
    assert kinds.dtype.name == "uint8"
    assert clouds.dtype.name == "int32"
    assert winds["q"].dtype.name == "int32"
    assert winds["r"].dtype.name == "int32"


def test_the_height_column_sums_to_the_cell_the_summary_reports() -> None:
    """Each level 1 cell holds the exact sum of the heights of its tiles."""
    world = cachette.World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    across = world.cells_wide
    assert EXTENT % across == 0, "the fixture must take a whole number of cells"
    edge = EXTENT // across
    heights = world.tile_heights().reshape(EXTENT, EXTENT)
    for row in range(across):
        for column in range(across):
            summary = world.region_summary(column * edge, row * edge)
            # The derivation of the block is checked here. A wrong edge gives
            # a wrong tile count, and the sum below would then measure the
            # test rather than the reader.
            assert summary["tiles"] == edge * edge
            block = heights[
                row * edge : (row + 1) * edge, column * edge : (column + 1) * edge
            ]
            assert int(block.sum()) == summary["height_total"], (
                f"the cell at ({column}, {row}) must hold the sum of its tiles"
            )


def test_the_height_column_holds_the_raw_fixed_point_value() -> None:
    """The boundary carries Q16.16 and never a quantity the engine rounded."""
    world = cachette.World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    heights = world.tile_heights()
    # A column that carried whole units would sum far below the summary, and
    # the height of a tile runs between zero and one unit.
    assert int(heights.sum()) == sum(
        world.region_summary(
            column * (EXTENT // world.cells_wide), row * (EXTENT // world.cells_wide)
        )["height_total"]
        for row in range(world.cells_wide)
        for column in range(world.cells_wide)
    )
    assert heights.max() > 1, "the entries are raw Q16.16 integers"


def test_the_kind_column_agrees_with_the_single_tile_report() -> None:
    """The bulk kind of a tile is the kind that ``tile_report`` names."""
    world = cachette.World(width=32, height=32, seed=SEED, faction_count=2)
    kinds = world.tile_kinds()
    for r in range(32):
        for q in range(32):
            report = world.tile_report(q, r)
            assert kinds[_index_of(q, r, 32)] == report["kind"], (
                f"the tile ({q}, {r}) must report one kind"
            )


def test_the_kind_column_holds_more_than_one_kind() -> None:
    """A fixture of one kind everywhere would hide a wrong order."""
    world = cachette.World(width=32, height=32, seed=SEED, faction_count=2)
    assert len(set(world.tile_kinds().tolist())) > 1


def test_the_cloud_column_agrees_with_the_air_reader() -> None:
    """A tile carries cloud exactly when the air over it holds water.

    The world stands under the span of the planet. The sky of a region clouds
    every tile of it, and the test then meets no dry tile to compare.
    """
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=2,
        latitude_centre=PLANET_CENTRE,
        latitude_span=PLANET_SPAN,
    )
    for _ in range(40):
        world.step(2)
    clouds = world.cloud_shares()
    air = np.array(
        [
            world.air_at(index % EXTENT, index // EXTENT)
            for index in range(EXTENT * EXTENT)
        ]
    )
    assert (air > 0).any(), "the fixture must put water in the air"
    assert (air == 0).any(), "the fixture must leave some sky dry"
    assert ((clouds > 0) == (air > 0)).all(), "the two readers must name one sky"
    assert clouds.min() >= 0
    assert clouds.max() <= world.cloud_share_whole


def test_the_cloud_column_is_flat_over_one_weather_cell() -> None:
    """Weather stands on the cell, so every tile of one cell reads alike."""
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=2,
    )
    for _ in range(40):
        world.step(2)
    edge = world.weather_cell_tiles
    clouds = world.cloud_shares().reshape(EXTENT, EXTENT)
    blocks = clouds.reshape(EXTENT // edge, edge, EXTENT // edge, edge)
    assert (blocks.min(axis=(1, 3)) == blocks.max(axis=(1, 3))).all()
    assert len(set(clouds.ravel().tolist())) > 1, "the fixture must vary between cells"


def _a_stormed_world() -> cachette.World:
    """Build a world that carries one raised storm over several cells.

    **The pitch puts one weather cell on each tile.** The engine default puts
    a few cells across a world this size, and a storm then reaches every one
    of them at one depth. A fixture like that measures nothing about the
    cone.[^1]

    The verb puts a storm in the list and the solve writes the deficit plane,
    so the world steps once after the storm is raised.

    [^1]: Testing rules, section 2a. ``.agents/rules/testing.md``
    """
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=1,
    )
    for _ in range(20):
        world.step(2)
    world.raise_cyclone((EXTENT // 2, EXTENT // 2), kind="tropical")
    world.step(2)
    return world


def test_the_storm_column_agrees_with_the_storms_the_world_carries() -> None:
    """The bulk deficit answers where a storm stands, and how deep it is.

    The column has no single-tile reader at the boundary to agree with, so it
    agrees with the storms the world reports. A world that carries a storm
    must read a deficit somewhere, and a world that carries none must read
    zero everywhere.

    The deficit is a cone, so it must grade. A column that answered one value
    over the whole footprint would draw a flat disc.
    """
    quiet = cachette.World(
        width=EXTENT, height=EXTENT, seed=SEED, faction_count=2, weather_cell_tiles=2
    )
    assert not quiet.cyclones(), "the fixture must start with no storm"
    assert (quiet.storm_depths() == 0).all(), (
        "a world with no storm reads a deficit somewhere"
    )

    world = _a_stormed_world()
    assert world.cyclones(), "the fixture must carry a storm"
    depths = world.storm_depths()
    assert depths.shape == (EXTENT * EXTENT,)
    assert depths.min() >= 0
    assert depths.max() <= world.storm_depth_whole
    under = depths > 0
    assert under.any(), "the storm reaches no tile of the world"
    assert (~under).any(), "the storm covers the whole world, so nothing is beside it"
    assert len(set(depths[under].tolist())) > 1, (
        "the cone answers one value over its whole footprint, so nothing grades it"
    )


def test_the_storm_column_is_not_the_cloud_column() -> None:
    """A storm rains its own sky out, so the cover cannot stand in for it.

    A renderer once read the top of the cover range as a storm. This states
    why that reading is wrong: the tiles under the storm do not carry the
    highest cover of the world, so the guess picks the wrong tiles.[^2]

    [^2]: Findings register, FND-723. ``docs/FINDINGS.md``
    """
    world = _a_stormed_world()
    depths = world.storm_depths()
    cover = world.cloud_shares()
    under = depths > 0
    assert under.any() and (~under).any(), "the fixture must reach both cases"
    highest = cover >= np.quantile(cover, 0.9)
    caught = int((highest & under).sum())
    assert caught < int(under.sum()) // 2, (
        f"the top of the cover range names {caught} of {int(under.sum())} "
        f"stormed tiles, so it could stand in for the deficit"
    )


def test_the_storm_column_is_flat_over_one_weather_cell() -> None:
    """Weather stands on the cell, so every tile of one cell reads alike."""
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=2,
    )
    for _ in range(20):
        world.step(2)
    world.raise_cyclone((EXTENT // 2, EXTENT // 2), kind="tropical")
    world.step(2)
    edge = world.weather_cell_tiles
    depths = world.storm_depths().reshape(EXTENT, EXTENT)
    assert (depths > 0).any(), "the fixture must reach a stormed tile"
    blocks = depths.reshape(EXTENT // edge, edge, EXTENT // edge, edge)
    assert (blocks.min(axis=(1, 3)) == blocks.max(axis=(1, 3))).all()


def _a_struck_world() -> tuple[cachette.World, tuple[int, int]]:
    """Build a dry world in which one faction holds ground, and strike it.

    The world stands under the span of the planet. The sky of a region blows
    no wind at the middle of this world, and a plume on still air goes nowhere.
    """
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=SEED,
        faction_count=2,
        weather_cell_tiles=2,
        latitude_centre=PLANET_CENTRE,
        latitude_span=PLANET_SPAN,
    )
    place = (EXTENT // 2, EXTENT // 2)
    world.spawn_soldiers([place], 0)
    world.found_settlements([place], faction=0)
    for _ in range(16):
        world.step(1)
    assert world.weather_totals()["raised"] == 0, "the fixture must start dry"
    world.inflict_weather(0, [place], world.weather_strength_ceiling)
    return world, place


def _air_centre(world: cachette.World) -> tuple[float, float]:
    """Return the column and the row of the centre of the water in the air.

    The reading covers the world alone. The caller must therefore prove that
    the whole plume is still inside the world before it trusts the answer.
    """
    across = world.weather_cells_wide
    down = world.weather_cell_count // across
    plane = world.weather_air().astype(float).reshape(down, across)
    total = plane.sum()
    assert total > 0
    assert total == world.weather_totals()["air"], (
        "water has reached the margin, and the centre no longer reads the plume"
    )
    rows, columns = np.mgrid[0:down, 0:across]
    return float((plane * columns).sum() / total), float((plane * rows).sum() / total)


def test_the_wind_column_points_the_way_the_air_travels() -> None:
    """The air moves along the wind, so the two axes cannot be swapped.

    The wind reader has no single-tile reader to agree with, because it is the
    first wind the boundary publishes. It agrees instead with what the wind
    does: the engine carries the water in the air along it. A reader that swaps
    the two axes, or that negates one, fails here.

    **The span is one step, and it cannot be longer.** The engine steps a
    margin of cells outside the world, a weather array crops the margin away,
    and the plume of one strike reaches both borders of this world on the
    second step. From there the centre of the cropped array measures what left
    the world and not what the wind carried. The helper asserts that the whole
    plume is still inside the world, so a longer span fails loudly rather than
    reading a number that means nothing.

    One step is also the whole of what this test can claim. It reads the wind
    of one cell, and the plume moves on the wind of every cell it reaches, so
    a span of several steps integrates a field that this test never read.
    """
    world, place = _a_struck_world()
    winds = world.tile_winds()
    index = _index_of(*place)
    along_q = int(winds["q"][index])
    along_r = int(winds["r"][index])
    assert (along_q, along_r) != (0, 0), "the fixture must hold a moving wind"
    first = _air_centre(world)
    world.step(1)
    last = _air_centre(world)
    moved_q = last[0] - first[0]
    moved_r = last[1] - first[1]
    assert np.sign(moved_q) == np.sign(along_q), (
        "the water travels along the first axis"
    )
    assert np.sign(moved_r) == np.sign(along_r), (
        "the water travels along the second axis"
    )
    assert (abs(moved_q) > abs(moved_r)) == (abs(along_q) > abs(along_r)), (
        "the faster axis carries the water further"
    )


def test_the_wind_column_is_flat_over_one_weather_cell() -> None:
    """The wind stands on the cell, so every tile of one cell reads alike."""
    world, _ = _a_struck_world()
    edge = world.weather_cell_tiles
    winds = world.tile_winds()
    for axis in ("q", "r"):
        plane = winds[axis].reshape(EXTENT, EXTENT)
        blocks = plane.reshape(EXTENT // edge, edge, EXTENT // edge, edge)
        assert (blocks.min(axis=(1, 3)) == blocks.max(axis=(1, 3))).all()
        assert len(set(plane.ravel().tolist())) > 1, (
            "the fixture must vary between cells"
        )


def test_a_bulk_read_leaves_the_state_hash_where_it_was() -> None:
    """The four readers read. None of them changes the world.

    A reader that changed one value would change the hash, and a changed hash
    is a changed simulation.
    """
    world = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=2,
    )
    for _ in range(8):
        world.step(2)
    before = world.state_hash()
    for _ in range(3):
        world.tile_heights()
        world.tile_kinds()
        world.cloud_shares()
        world.tile_winds()
    assert world.state_hash() == before
    # The step after a read must land where a step with no read lands.
    world.step(2)
    stepped = world.state_hash()
    plain = cachette.World(
        width=EXTENT,
        height=EXTENT,
        seed=COASTAL_SEED,
        faction_count=2,
        weather_cell_tiles=2,
    )
    for _ in range(9):
        plain.step(2)
    assert stepped == plain.state_hash()
