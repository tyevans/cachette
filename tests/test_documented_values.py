"""Pin every number that the published reference states about a value.

The reference takes its prose from the Rust doc comments of the bindings
crate.[^1] A doc comment that states a default, a bound or a ceiling is a
second place that declares the value. The first place is the code. Nothing
fails when two such places disagree, and that is the defect shape this
project names first in its own rule.[^2]

Each test below reads one number that a doc comment states, through the
public interface of the package. A change to the code that moves the number
fails a test here, and the failure names the doc comment to repair.

References
----------
[^1]: ADR-0107, the Python reference is generated from the compiled module,
decision D2.
``docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md``

[^2]: Recurring Defect Shapes, shape 1, redundant declaration sites.
``.claude/rules/recurring-defects.md``

[^3]: ADR-0152, a faction plans its roads and zones with one solver,
decision D3.
``docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md``
"""

from __future__ import annotations

import pytest

import cachette

# The doc comment of the `World` class states these four defaults.
DOCUMENTED_WIDTH = 64
DOCUMENTED_HEIGHT = 64
DOCUMENTED_SEED = 81985529216486895
DOCUMENTED_FACTION_COUNT = 4

# The doc comment of the `World` class states this ceiling.
DOCUMENTED_FACTION_CEILING = 63

# The doc comment of the `Camera` class states this default and this range.
DOCUMENTED_TILE_SIZE = 12.0
DOCUMENTED_SMALLEST_TILE = 2.0
DOCUMENTED_LARGEST_TILE = 64.0

# The doc comment of `World.window_census` states this default and this
# ceiling, and the doc comment of `VerbError` states the ceiling as well.
DOCUMENTED_RADIUS = 8
DOCUMENTED_RADIUS_CEILING = 64

# The doc comment of `World.site_economy` states that the world holds one
# commodity, numbered zero.
DOCUMENTED_COMMODITY = 0

# The doc comment of `World.order_gather` states that the resource kinds are
# zero, one and two, and that three and above name none.
DOCUMENTED_RESOURCE_KINDS = (0, 1, 2)
DOCUMENTED_FIRST_REFUSED_KIND = 3

# The doc comment of `World.order_build` states that a road is zero, a
# terrace is one, a wonder is two and a store is three, and that four and
# above name no upgrade kind. The doc comments of `World.build_order` and
# `World.tile_report` state the same numbering.
DOCUMENTED_ROAD = 0
DOCUMENTED_TERRACE = 1
DOCUMENTED_WONDER = 2
DOCUMENTED_STORE = 3
DOCUMENTED_WALL = 4
DOCUMENTED_UPGRADE_KINDS = (
    DOCUMENTED_ROAD,
    DOCUMENTED_TERRACE,
    DOCUMENTED_WONDER,
    DOCUMENTED_STORE,
    DOCUMENTED_WALL,
)
# The table holds one open category above the five the doc comment names, so
# the first number that names no category is one above that.
DOCUMENTED_FIRST_REFUSED_UPGRADE_KIND = 6

# The doc comment of `World.direction_offsets` states that a tile of this
# world has this many neighbours.
DOCUMENTED_NEIGHBOUR_COUNT = 6
# The doc comments of `World.define_unit_type` and `World.set_unit_types`
# state that the unit type table holds eight rows, and that eight and above
# name none.
DOCUMENTED_UNIT_TYPES = (0, 1, 2, 3, 4, 5, 6, 7)
DOCUMENTED_FIRST_REFUSED_UNIT_TYPE = 8

# The doc comment of `World.define_unit_type` states the fixed-point scale of
# an attack and of an armour: one whole casualty is this value.
DOCUMENTED_FIXED_POINT_ONE = 65536


def _worker_columns(world: cachette.World) -> dict[str, int]:
    """Return the six columns beyond attack and armour, as the worker holds them.

    The worker is row zero of the table a new world is built with. The values
    are read from the engine rather than written here, so the test holds no
    second copy of them.
    """
    table = world.unit_type_table()
    return {
        name: int(table[name][0])  # type: ignore[literal-required]
        for name in table
        if name not in ("attack", "armour")
    }


def test_the_constructor_defaults_are_the_documented_ones() -> None:
    default = cachette.World()
    named = cachette.World(
        width=DOCUMENTED_WIDTH,
        height=DOCUMENTED_HEIGHT,
        seed=DOCUMENTED_SEED,
        faction_count=DOCUMENTED_FACTION_COUNT,
    )
    assert default.width == DOCUMENTED_WIDTH
    assert default.height == DOCUMENTED_HEIGHT
    # The seed and the faction count reach no property, so the state hash
    # stands for them. Two worlds that hold one state give one hash.
    assert default.state_hash() == named.state_hash()


def test_a_side_of_zero_is_refused() -> None:
    with pytest.raises(cachette.ConfigError):
        cachette.World(width=0, height=8)
    with pytest.raises(cachette.ConfigError):
        cachette.World(width=8, height=0)


def test_the_faction_count_holds_the_documented_bounds() -> None:
    assert cachette.World(width=8, height=8, faction_count=0).check_invariants()
    ceiling = DOCUMENTED_FACTION_CEILING
    assert cachette.World(width=8, height=8, faction_count=ceiling).check_invariants()
    with pytest.raises(cachette.ConfigError):
        cachette.World(width=8, height=8, faction_count=ceiling + 1)


def test_the_camera_tile_size_holds_the_documented_default_and_range() -> None:
    assert cachette.Camera().tile_width == DOCUMENTED_TILE_SIZE
    assert cachette.Camera().tile_height == DOCUMENTED_TILE_SIZE
    small = cachette.Camera(tile_size=DOCUMENTED_SMALLEST_TILE / 2)
    assert small.tile_width == DOCUMENTED_SMALLEST_TILE
    large = cachette.Camera(tile_size=DOCUMENTED_LARGEST_TILE * 2)
    assert large.tile_width == DOCUMENTED_LARGEST_TILE


def test_the_window_census_radius_holds_the_documented_default_and_ceiling(
    seed: int,
) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    assert world.window_census(4, 4)["radius"] == DOCUMENTED_RADIUS
    ceiling = DOCUMENTED_RADIUS_CEILING
    assert world.window_census(4, 4, radius=ceiling)["radius"] == ceiling
    with pytest.raises(cachette.VerbError):
        world.window_census(4, 4, radius=ceiling + 1)


def test_the_world_holds_one_commodity_and_its_number_is_zero(seed: int) -> None:
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    site = world.found_group(8, 0)["site"]
    assert world.site_economy(site)["commodity"] == DOCUMENTED_COMMODITY
    assert world.site_economy(site, DOCUMENTED_COMMODITY)["commodity"] == 0
    with pytest.raises(cachette.ViewError):
        world.site_economy(site, DOCUMENTED_COMMODITY + 1)


def test_the_gather_order_takes_three_resource_kinds_and_refuses_the_fourth(
    seed: int,
) -> None:
    # The verb checks the kind before it resolves the set, so an empty set
    # reads the check alone.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=1)
    for kind in DOCUMENTED_RESOURCE_KINDS:
        world.order_gather([], kind)
    with pytest.raises(cachette.VerbError):
        world.order_gather([], DOCUMENTED_FIRST_REFUSED_KIND)


def test_a_ground_kind_that_overlaps_a_resource_kind_is_not_refused(
    seed: int,
) -> None:
    # The reference says this plainly, and the reader must be able to trust
    # it: the ground kinds water, plain and forest are 0, 1 and 2, and each
    # of those numbers also names a resource kind. The verb accepts them.
    # Nothing here asserts that the behaviour is right. It asserts that the
    # sentence in the doc comment describes what the engine does.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=1)
    ground_kinds_that_overlap = (0, 1, 2)
    for ground_kind in ground_kinds_that_overlap:
        world.order_gather([], ground_kind)
    ground_kinds_that_do_not_overlap = (3, 4)
    for ground_kind in ground_kinds_that_do_not_overlap:
        with pytest.raises(cachette.VerbError):
            world.order_gather([], ground_kind)


def test_the_build_order_takes_every_upgrade_kind_and_refuses_the_next(
    seed: int,
) -> None:
    # The verb checks the kind before it resolves the set, so an empty set
    # reads the check alone.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=1)
    for kind in DOCUMENTED_UPGRADE_KINDS:
        world.order_build([], kind)
    with pytest.raises(cachette.VerbError):
        world.order_build([], DOCUMENTED_FIRST_REFUSED_UPGRADE_KIND)


def test_every_upgrade_kind_carries_the_documented_number(
    seed: int,
) -> None:
    # The doc comment of `World.order_build` names each number. A read of the
    # order and a read of the tile must both report the number that went in.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=1)
    for kind in DOCUMENTED_UPGRADE_KINDS:
        world = cachette.World(width=16, height=16, seed=seed, faction_count=1)
        address = _first_open_address(world)
        # A unit builds anything but a road only on ground its own faction
        # holds, and a faction holds the ground its cities reach.
        world.found_settlements([address], faction=0)
        world.step(threads=1)
        units = world.spawn_soldiers([address], faction=0)
        # A category whose row asks for no held ground is laid only inside a
        # project, so the plan zones the tile before the order.[^3]
        try:
            world.zone_projects(0, [address], kind)
        except cachette.VerbError:
            pass
        # A category whose row does not fit the ground under this tile is
        # refused, and the number it carries is still the documented one.
        table = world.upgrade_table()
        ground = world.tile_report(*address)["kind"]
        levels = len(table["ground_fit"]) // (DOCUMENTED_FIRST_REFUSED_UPGRADE_KIND)
        if not int(table["ground_fit"][kind * levels]) & (1 << ground):
            with pytest.raises(cachette.VerbError):
                world.order_build(units, kind)
            continue
        world.order_build(units, kind)
        assert world.build_order(int(units[0])) == kind
        world.step(threads=1)
        assert world.tile_report(*address)["upgrade"] == kind


def test_a_resource_kind_that_overlaps_an_upgrade_kind_is_not_refused(
    seed: int,
) -> None:
    # The doc comment of `World.order_build` says the engine accepts every
    # resource kind, because each of those numbers also names an upgrade
    # kind. Nothing here asserts that the behaviour is right. It asserts that
    # the sentence describes what the engine does.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=1)
    for resource_kind in DOCUMENTED_RESOURCE_KINDS:
        world.order_build([], resource_kind)
    with pytest.raises(cachette.VerbError):
        world.order_build([], DOCUMENTED_FIRST_REFUSED_UPGRADE_KIND)


def test_a_tile_has_the_documented_number_of_neighbours() -> None:
    assert len(cachette.World.direction_offsets()) == DOCUMENTED_NEIGHBOUR_COUNT


def _first_open_address(world: cachette.World) -> tuple[int, int]:
    """Return an address of ground that admits a unit."""
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def test_the_unit_type_table_holds_the_documented_rows() -> None:
    """The table takes every documented row and refuses the first one above."""
    world = cachette.World()
    for unit_type in DOCUMENTED_UNIT_TYPES:
        world.define_unit_type(
            unit_type, DOCUMENTED_FIXED_POINT_ONE, 0, **_worker_columns(world)
        )
    with pytest.raises(cachette.VerbError):
        world.define_unit_type(
            DOCUMENTED_FIRST_REFUSED_UNIT_TYPE, 0, 0, **_worker_columns(world)
        )


def test_one_whole_casualty_is_the_documented_fixed_point_value() -> None:
    """An attack of one whole casualty ends one defender for each attacker.

    The doc comment states the value of one in the fixed-point scale. This
    reads it back through the engine: two attackers of an attack of that
    value end exactly two defenders in one frame.
    """
    world = cachette.World(width=1, height=1, seed=1, faction_count=2)
    world.define_unit_type(0, DOCUMENTED_FIXED_POINT_ONE, 0, **_worker_columns(world))
    attackers = world.spawn_soldiers([(0, 0), (0, 0)], 0)
    defenders = world.spawn_soldiers([(0, 0)] * 5, 1)
    world.set_unit_types(attackers, 0)
    world.set_unit_types(defenders, 0)
    # The contest fires only when the pair is in the war band, so the fixture
    # declares war first. The value sits below every edge the register could
    # set, and it is not a copy of an edge.
    world.set_relation(0, 1, -(1 << 20))
    assert world.relation_band(0, 1) == 0
    world.step(2)
    # Each side reaches the other, so each side loses what the other paid for.
    assert world.faction_population()[1] == 3
    assert world.faction_population()[0] == 0
