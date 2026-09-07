"""A road draws as a way, and not as a filled tile.

A road is not a property of one tile. It runs from somewhere to somewhere, it
joins another road at a junction, it bends, and it ends. A coloured cell says
that a tile carries the road property. It does not draw a road.

Every test here starts at the Python boundary. The fixture builds its roads
through the engine's own verbs, so a join the engine would not make cannot
reach the drawing.

**The fixture holds every shape a way takes.** A world whose roads all stand
alone, or all stand in one straight line, measures the fixture and not the
code.[^1] The network below holds a road that stands alone, a road that ends,
a road that runs straight through, a road that bends, and a junction of three.

The device renderer draws the same ways, and a test in the module that holds
the two renderers together asserts that the two frames agree.[^2]

References
----------
[^1]: Testing Rules, section 2a. ``.agents/rules/testing.md``

[^2]: The two renderers, and the bound they agree within.
``tests/test_demo_sketch_gl.py``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import Camera, World
from cachette.demo import sketch as ink
from cachette.demo.sketch import Sketch
from cachette.demo.surface import Surface
from cachette.demo.view import View

# The upgrade category of a road, and the last category the engine holds, as
# the engine numbers them. The second one gives the width of the table, so the
# fixture finds a row without stating how many levels a category has.
ROAD = 0
OPEN = 6

# The side of the fixture world, the seed it grows from, and the size of the
# frame the drawing tests fill.
SIDE = 24
SEED = 7
WIDTH = 320
HEIGHT = 240

# The work the fixture gives one level of a road.
#
# A builder that works for hundreds of steps starves before the level stands.
# This is short enough that it finishes first. It measures nothing.
QUICK_WORK = 8

# How many steps a build is given before the fixture calls it stuck. A build
# takes several steps and no test here states how many.
STEP_CEILING = 400

# The tiles the shaped network runs over.
#
# The spine runs along one row. A branch leaves it, which makes a junction of
# three, and the branch turns, which makes a bend. One tile stands away from
# every other, so nothing reaches it.
SPINE = ((4, 5), (5, 5), (6, 5), (7, 5))
BRANCH = ((6, 6), (6, 7), (7, 7))
ALONE = ((3, 8),)
NETWORK = SPINE + BRANCH + ALONE

# The joins that each tile of the network must carry, as the bit for each of
# the six directions the engine publishes.
#
# **The expected value is written as the neighbours and not as a number.** A
# number would be a second statement of the direction order, and the engine
# already publishes that order.
EXPECTED_JOINS = {
    (4, 5): {(5, 5)},
    (5, 5): {(4, 5), (6, 5)},
    (6, 5): {(5, 5), (7, 5), (6, 6)},
    (7, 5): {(6, 5), (6, 6)},
    (6, 6): {(6, 5), (7, 5), (6, 7)},
    (6, 7): {(6, 6), (7, 7)},
    (7, 7): {(6, 7)},
    (3, 8): set(),
}


def _world() -> World:
    """Give back a fixture world in which a road is quick to build.

    **The fixture makes every road level cheap.** A builder that works for
    hundreds of steps starves, and the engine takes it away before the level
    stands, so a fixture that waited would never reach the second level. The
    upgrade table is data, so the fixture rewrites the work of each row rather
    than waiting.[^1]

    Nothing else about a row changes. Each row is read from the table and put
    back with one field moved, so the fixture states no column of its own.

    The work here is not a measurement of anything. It is short enough that a
    builder finishes before it goes hungry.

    [^1]: ADR-0151, an upgrade is a category with a ground fit and a level,
    decision D1.
    ``docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md``
    """
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=2)
    table = world.upgrade_table()
    levels = len(table["ground_fit"]) // (OPEN + 1)
    for level in range(1, levels + 1):
        at = ROAD * levels + level - 1
        row = {name: int(np.asarray(column)[at]) for name, column in table.items()}
        row["work"] = QUICK_WORK
        world.define_upgrade_row(ROAD, level, **row)
    return world


def _build_roads(
    world: World, places: tuple[tuple[int, int], ...], levels: int = 1
) -> None:
    """Build a road at every place, up to a level, through the engine's verbs.

    The call is set-valued at every step. Nothing here loops over a tile to
    send a command, because the control plane sends one command for a set.
    """
    for place in places:
        assert world.tile_report(*place)["passable"], (
            f"the fixture needs open ground at {place}, and the world has "
            f"none there, so this test would measure the refusal"
        )
    units = world.spawn_soldiers(list(places), faction=0)
    for level in range(1, levels + 1):
        world.zone_projects(0, list(places), ROAD)
        world.order_build(units, ROAD)
        # **The test is the level and not the flag.** A tile whose first level
        # stands already reports a finished upgrade, so a fixture that read
        # the flag would leave the second build unstarted and would then say
        # that the two levels draw alike.
        for _ in range(STEP_CEILING):
            world.step(threads=2)
            if all(
                world.tile_report(*place)["upgrade_level"] >= level for place in places
            ):
                break
        else:  # pragma: no cover - the fixture asserts its own outcome
            message = f"the roads at {places} did not reach level {level}"
            raise AssertionError(message)


def _joins_by_place(world: World) -> dict[tuple[int, int], set[tuple[int, int]]]:
    """Read the roads back as the set of neighbours each one runs to."""
    ways = world.road_ways()
    steps = World.direction_offsets()
    answer: dict[tuple[int, int], set[tuple[int, int]]] = {}
    for column, row, joins in zip(ways["q"], ways["r"], ways["joins"], strict=True):
        reached = {
            (int(column) + step_q, int(row) + step_r)
            for direction, (step_q, step_r) in enumerate(steps)
            if (int(joins) >> direction) & 1
        }
        answer[(int(column), int(row))] = reached
    return answer


@pytest.fixture(name="shaped")
def _shaped() -> World:
    """Give back a world whose roads hold every shape a way takes."""
    world = _world()
    _build_roads(world, NETWORK)
    return world


# ----------------------------------------------------------------------
# The reader.


def test_the_engine_reports_which_neighbours_each_road_runs_to(
    shaped: World,
) -> None:
    """The joins are the joins of the network the fixture built.

    A reader that answered no join, or that answered every neighbour, fails
    here. So does one that answers a join for a tile that carries no road.
    """
    assert _joins_by_place(shaped) == EXPECTED_JOINS


def test_the_shapes_a_way_takes_are_all_in_the_answer(shaped: World) -> None:
    """The fixture supplies an end, a through-way, a junction and a mark.

    A fixture whose roads all stand alone, or all stand in one line, measures
    itself. This asserts that the network under test does not.
    """
    counts = {len(reached) for reached in _joins_by_place(shaped).values()}
    assert counts == {0, 1, 2, 3}


def test_a_road_that_nothing_reaches_runs_nowhere() -> None:
    """One road on its own joins nothing.

    A reader that looked at the whole upgrade set rather than at the
    neighbours of a tile would answer a join here.
    """
    world = _world()
    _build_roads(world, SPINE + ALONE)
    assert _joins_by_place(world)[ALONE[0]] == set()


def test_the_reader_answers_nothing_for_a_world_with_no_road() -> None:
    ways = _world().road_ways()
    assert ways["q"].size == 0
    assert ways["r"].size == 0
    assert ways["level"].size == 0
    assert ways["joins"].size == 0


def test_the_reader_carries_the_level_of_each_road() -> None:
    """A road built twice stands one level above a road built once."""
    once = _world()
    _build_roads(once, SPINE)
    twice = _world()
    _build_roads(twice, SPINE, levels=2)
    assert set(once.road_ways()["level"].tolist()) == {1}
    assert set(twice.road_ways()["level"].tolist()) == {2}


def test_a_road_that_is_destroyed_leaves_the_network(shaped: World) -> None:
    """The answer follows the world. It is not read once and held."""
    assert (6, 5) in _joins_by_place(shaped)
    assert shaped.destroy_upgrades([(6, 5)]) == 1
    after = _joins_by_place(shaped)
    assert (6, 5) not in after
    # The neighbours no longer run to it, so the removal reaches them too.
    assert after[(5, 5)] == {(4, 5)}
    assert after[(7, 5)] == {(6, 6)}


# ----------------------------------------------------------------------
# The drawing.


def _frame(world: World, camera: Camera, relief: float = ink.RELIEF) -> np.ndarray:
    """Draw one frame with the sketch renderer and give back the pixels."""
    surface = Surface(WIDTH, HEIGHT)
    Sketch(world, view=View(), relief=relief)(camera, WIDTH, HEIGHT, surface.pixels)
    return surface.pixels.copy()


def _middle_of(fit: np.ndarray, want: float) -> int:
    """Give back the middle pixel of the frame that shows one point of a page.

    A zoom puts several pixels of the frame on one point of the page. Taking
    the first of them puts the reading up to one whole point away from the
    place the caller named, and a test then reads a mark from beside the
    place rather than at it.
    """
    gap = np.abs(fit - want)
    same = np.flatnonzero(gap == gap.min())
    return int(same[len(same) // 2])


def _at(world: World, camera: Camera, place: tuple[float, float]) -> tuple[int, int]:
    """Say which pixel of the frame shows one place in the grid.

    The place is given in tiles, and it may lie between two tiles. The answer
    comes from the renderer's own projection and its own fit, which are the
    two rules that both renderers share, so this test states no geometry of
    its own.

    **The ground is drawn flat here.** The lift moves a point up the page by
    the height of the ground under it, and a test that inverted the lift would
    hold a second copy of it. A flat page needs no inverse, and what the tests
    below read is where a mark falls across a tile.
    """
    sketch = Sketch(world, view=View(), relief=0.0)
    window = (0, 0, world.width, world.height)
    stood = sketch.projection(
        window, WIDTH, HEIGHT, sketch.detail_of(camera, WIDTH, HEIGHT)
    )
    column, flat_row = stood.to_page(
        np.array([place[0]], dtype=np.float64), np.array([place[1]], dtype=np.float64)
    )
    row = flat_row + stood.rise + ink.LIFT_MARGIN
    take_x, take_y = sketch.fit_lists(
        stood, sketch.box_of(stood), camera, WIDTH, HEIGHT
    )
    across = _middle_of(take_x, column[0])
    down = _middle_of(take_y, row[0])
    # **A place the frame does not show is a fault in the test.** The nearest
    # column is the nearest whether or not the place is on the frame, so a
    # place off the edge would silently come back as an edge pixel and the
    # test would read a mark that belongs to somewhere else.
    assert abs(int(take_x[across]) - column[0]) <= 1.0, (
        f"the frame does not show the place {place}: it lies off the side"
    )
    assert abs(int(take_y[down]) - row[0]) <= 1.0, (
        f"the frame does not show the place {place}: it lies off the top or the foot"
    )
    return across, down


def _marked(before: np.ndarray, after: np.ndarray, at: tuple[int, int]) -> bool:
    """Report whether the roads drew anything within one pixel of a place.

    The fit puts a whole page point under each pixel of the frame, so a place
    named in tiles lands within a pixel of where the drawing put it. The test
    therefore reads a three by three block and not a single pixel.
    """
    across, down = at
    first = before[max(down - 1, 0) : down + 2, max(across - 1, 0) : across + 2]
    second = after[max(down - 1, 0) : down + 2, max(across - 1, 0) : across + 2]
    return bool((first != second).any())


def _road_ink(world: World, camera: Camera) -> int:
    """Count the pixels that the roads of a world put on the page.

    The world is drawn as it stands, then every road is destroyed, then it is
    drawn again, at one camera. The pixels that differ are the pixels the
    roads drew, and nothing else about the two frames differs.

    **The call takes the roads out of the world it is given.** A caller that
    needs a second reading builds a second world.
    """
    ways = world.road_ways()
    places = [(int(q), int(r)) for q, r in zip(ways["q"], ways["r"], strict=True)]
    before = _frame(world, camera)
    assert world.destroy_upgrades(places) == len(places)
    after = _frame(world, camera)
    return int((before != after).sum())


def test_a_road_marks_its_ground_on_the_sketch(shaped: World) -> None:
    """The sketch draws something where a road runs.

    A renderer that read no road at all draws nothing here, and every other
    drawing test below would then compare nothing against nothing.
    """
    camera = Camera.fitting(shaped, WIDTH, HEIGHT)
    assert _road_ink(shaped, camera) > 0


def _road_frames(world: World, camera: Camera) -> tuple[np.ndarray, np.ndarray]:
    """Give back the flat frame with the roads and the flat frame without them.

    The two frames differ where the roads drew and nowhere else.
    """
    ways = world.road_ways()
    places = [(int(q), int(r)) for q, r in zip(ways["q"], ways["r"], strict=True)]
    before = _frame(world, camera, relief=0.0).reshape(HEIGHT, WIDTH)
    assert world.destroy_upgrades(places) == len(places)
    after = _frame(world, camera, relief=0.0).reshape(HEIGHT, WIDTH)
    return before, after


def test_the_way_reaches_a_neighbour_it_joins_and_stops_short_of_one_it_does_not() -> (
    None
):
    """The ribbon runs out through the edge it shares with a road, and no other.

    Two roads stand side by side. The middle of the edge between them carries
    the way, because each side draws out to that point and the two halves
    meet. The middle of every other edge of the same tile carries nothing,
    because no road stands beyond it.

    **This is the connection logic, read off the picture.** A renderer that
    drew an arm in every direction marks the second place. A renderer that
    drew no arm at all marks neither.
    """
    world = _world()
    joined = (6, 5)
    beside = (7, 5)
    _build_roads(world, (joined, beside))
    camera = Camera(tile_size=Camera.fitting(world, WIDTH, HEIGHT).tile_width * 4.0)
    before, after = _road_frames(world, camera)
    steps = World.direction_offsets()
    towards = (beside[0] - joined[0], beside[1] - joined[1])
    for step in steps:
        edge = (joined[0] + step[0] * 0.5, joined[1] + step[1] * 0.5)
        marked = _marked(before, after, _at(world, camera, edge))
        if step == towards:
            assert marked, (
                f"the way did not reach the edge towards {beside}, so the "
                f"two roads do not join"
            )
        else:
            assert not marked, (
                f"the way reached the edge in the direction {step}, and no "
                f"road stands beyond it"
            )


# Where a test reads for bare ground around a road, in tiles from the middle
# of the tile.
#
# **The place is not a corner of the rhombus that a tile covers.** The lattice
# is hexagonal, and the point half a step along both axes is the middle of the
# edge between the two neighbours that lie there. A way runs through it when
# those two carry roads, so a test that read it would fail on a network and
# pass on a pair.
#
# A way that stands and joins nothing marks a disc of about 0.28 of a tile
# across the middle of its cell, so this reach clears it. A tint of the cell
# reaches half a tile on this axis and further at a corner, so a renderer that
# tinted the cell marks every place below.
BARE_REACH = 0.45


def test_a_road_that_nothing_reaches_leaves_its_whole_tile_bare() -> None:
    """One road on its own marks the middle and nothing else.

    A renderer that tinted the cell of a road covers every place below. The
    mark of a way that goes nowhere is a mark, and the ground around it still
    shows.
    """
    world = _world()
    _build_roads(world, ALONE)
    camera = Camera(tile_size=Camera.fitting(world, WIDTH, HEIGHT).tile_width * 4.0)
    before, after = _road_frames(world, camera)
    middle = ALONE[0]
    assert _marked(before, after, _at(world, camera, middle)), (
        "the road drew nothing at all, so this test compares nothing"
    )
    for step_q, step_r in World.direction_offsets():
        out = (
            middle[0] + step_q * BARE_REACH,
            middle[1] + step_r * BARE_REACH,
        )
        assert not _marked(before, after, _at(world, camera, out)), (
            f"the road at {middle} marked the ground {BARE_REACH} of a tile "
            f"away in the direction ({step_q}, {step_r}), so it is drawing "
            f"the cell rather than the way"
        )
    # The two long corners of the rhombus that a tile covers. They stand
    # further from the middle than any edge does, so a cell tint reaches them
    # first and a ribbon reaches them last.
    for corner in ((0.45, 0.45), (-0.45, -0.45)):
        out = (middle[0] + corner[0], middle[1] + corner[1])
        assert not _marked(before, after, _at(world, camera, out)), (
            f"the road at {middle} marked its own corner at the offset "
            f"{corner}, so it is drawing the cell rather than the way"
        )


def test_a_way_that_runs_one_way_leaves_the_rest_of_its_tile_bare() -> None:
    """A road that joins one neighbour marks nothing across the tile.

    Two roads stand side by side, so the ribbon runs along one axis alone.
    The ground on either side of it still shows.
    """
    world = _world()
    joined = (6, 5)
    _build_roads(world, (joined, (7, 5)))
    camera = Camera(tile_size=Camera.fitting(world, WIDTH, HEIGHT).tile_width * 4.0)
    before, after = _road_frames(world, camera)
    for across in ((0.45, 0.45), (-0.45, -0.45), (0.0, 0.45), (0.0, -0.45)):
        out = (joined[0] + across[0], joined[1] + across[1])
        assert not _marked(before, after, _at(world, camera, out)), (
            f"the road at {joined} marked the ground at the offset {across}, "
            f"and no way runs there"
        )


def test_roads_that_join_draw_a_longer_way_than_roads_that_do_not() -> None:
    """The joins reach the drawing, and they decide what is drawn.

    Two worlds carry the same number of roads. In one the roads touch, so the
    ribbon runs the whole length of the run. In the other no road touches
    another, so each draws a mark and nothing runs between them.

    A renderer that ignored the joins draws the same thing in both worlds. A
    renderer that drew an arm in every direction draws *more* in the world
    where nothing joins, so the test fails from either side.
    """
    joined = _world()
    _build_roads(joined, ((4, 5), (5, 5), (6, 5), (7, 5), (8, 5)))
    apart = _world()
    _build_roads(apart, ((4, 5), (6, 5), (8, 5), (4, 9), (6, 9)))
    camera = Camera.fitting(joined, WIDTH, HEIGHT)
    run = _road_ink(joined, camera)
    marks = _road_ink(apart, camera)
    assert run > marks * 1.5, (
        f"a run of five joined roads marked {run} pixels and five roads that "
        f"join nothing marked {marks}, so the joins do not reach the drawing"
    )


def test_the_better_road_reads_as_the_better_road() -> None:
    """The two levels of a road do not draw alike.

    The width carries the level and the best road adds a line down its
    middle, so the better road puts more ink on the page over the same
    ground. A renderer that ignored the level draws the two alike.
    """
    first = _world()
    _build_roads(first, SPINE)
    second = _world()
    _build_roads(second, SPINE, levels=2)
    camera = Camera.fitting(first, WIDTH, HEIGHT)
    poorer = _road_ink(first, camera)
    better = _road_ink(second, camera)
    assert better > poorer, (
        f"a level 2 road marked {better} pixels and a level 1 road marked "
        f"{poorer}, so the level does not reach the drawing"
    )


def test_a_road_under_work_does_not_draw_as_a_road_that_stands() -> None:
    """A way that is being made reads as marked-out ground.

    A watcher must be able to tell a road that carries traffic from one that
    is still being dug.
    """
    standing = _world()
    _build_roads(standing, SPINE)
    begun = _world()
    for place in SPINE:
        assert begun.tile_report(*place)["passable"]
    begun.zone_projects(0, list(SPINE), ROAD)
    units = begun.spawn_soldiers(list(SPINE), faction=0)
    begun.order_build(units, ROAD)
    begun.step(threads=2)
    assert set(begun.road_ways()["level"].tolist()) == {0}
    camera = Camera.fitting(standing, WIDTH, HEIGHT)
    assert _road_ink(begun, camera) < _road_ink(standing, camera)
