"""An upgrade is a category with a ground fit and a level.

The engine declares the columns of a row once, and the type stub names them
by hand. A test here reads the stub and asserts that it names the columns the
engine returns, in the same order, so the table is declared once and a second
copy fails when it drifts.[^1]

The build verb names a category and no level. The engine resolves the row
from the ground under the tile and the level that stands there, and it
refuses when no row fits.[^2]

References
----------
[^1]: Recurring Defect Shapes, shape 1.
``.agents/rules/recurring-defects.md``

[^2]: ADR-0151, an upgrade is a category with a ground fit and a level,
decisions D1 to D4.
``docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md``
"""

from __future__ import annotations

import ast
import pathlib

import numpy as np
import pytest

import cachette

# The categories the doc comment of ``World.order_build`` names.
ROAD = 0
TERRACE = 1
WONDER = 2
STORE = 3
WALL = 4
OPEN = 5

# The ground numbers that ``World.tile_report`` states.
WATER = 0
PLAIN = 1

# The stub is hand-written and nothing regenerates it, so a test reads it
# from the source tree beside the package rather than from the install.
STUB = (
    pathlib.Path(__file__).resolve().parent.parent / "python" / "cachette" / "_core.pyi"
)


def _stub_column_names() -> list[str]:
    """Return the field names of ``UpgradeColumns`` in the stub, in order."""
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == "UpgradeColumns":
            return [
                statement.target.id
                for statement in node.body
                if isinstance(statement, ast.AnnAssign)
                and isinstance(statement.target, ast.Name)
            ]
    message = "the stub declares no UpgradeColumns class"
    raise AssertionError(message)


def _stub_define_keywords() -> list[str]:
    """Return the keyword-only parameter names of ``define_upgrade_row``."""
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == "define_upgrade_row":
            return [argument.arg for argument in node.args.kwonlyargs]
    message = "the stub declares no define_upgrade_row method"
    raise AssertionError(message)


def _row(**changes: int) -> dict[str, int]:
    """Return a full row at zero, with the named columns changed."""
    row = dict.fromkeys(_stub_column_names(), 0)
    row.update(changes)
    return row


def _address_of_ground(world: cachette.World, ground: int) -> tuple[int, int]:
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["kind"] == ground:
                return (q, r)
    message = "the world holds no such ground"
    raise AssertionError(message)


def _levels(world: cachette.World) -> int:
    """Return how many levels the table holds for each category."""
    return len(world.upgrade_table()["ground_fit"]) // (OPEN + 1)


def _row_at(world: cachette.World, category: int, level: int) -> int:
    """Return the index of one category at one level in the column arrays."""
    return category * _levels(world) + level - 1


def test_the_stub_names_the_columns_the_engine_returns_in_the_same_order() -> None:
    """The typed dictionary in the stub and the engine table agree.

    The engine derives its keys from the row declaration, so this compares
    the hand-written stub against the one declaration.
    """
    table = cachette.World().upgrade_table()
    assert list(table) == _stub_column_names()


def test_the_stub_keyword_arguments_are_the_columns_of_a_row() -> None:
    table = cachette.World().upgrade_table()
    assert _stub_define_keywords() == list(table)


def test_every_column_is_a_wide_integer_array_of_the_table_width() -> None:
    table = cachette.World().upgrade_table()
    widths = {len(table[name]) for name in table}  # type: ignore[literal-required]
    assert len(widths) == 1
    for name in table:
        assert table[name].dtype == np.int64  # type: ignore[literal-required]


def test_a_new_world_holds_the_default_rows() -> None:
    world = cachette.World()
    table = world.upgrade_table()
    # The road raises what a tile holds and takes work.
    assert table["capacity_change"][_row_at(world, ROAD, 1)] > 0
    assert table["work"][_row_at(world, ROAD, 1)] > 0
    # The road and the terrace hold a second level, and it asks for more work
    # than the first.
    for category in (ROAD, TERRACE):
        first = _row_at(world, category, 1)
        second = _row_at(world, category, 2)
        assert table["ground_fit"][second] != 0
        assert table["work"][second] > table["work"][first]
    # The terrace raises what a unit takes, and the second level raises it
    # further.
    assert (
        table["yield_change"][_row_at(world, TERRACE, 2)]
        > table["yield_change"][_row_at(world, TERRACE, 1)]
    )
    # The open category holds no row at all.
    for level in range(1, _levels(world) + 1):
        assert table["ground_fit"][_row_at(world, OPEN, level)] == 0


def test_no_row_fits_the_ground_that_holds_nobody() -> None:
    world = cachette.World()
    table = world.upgrade_table()
    for fit in table["ground_fit"]:
        assert not int(fit) & (1 << WATER)


def test_define_upgrade_row_takes_the_whole_row_and_reads_it_back() -> None:
    world = cachette.World()
    values = {name: index + 1 for index, name in enumerate(_stub_column_names())}
    world.define_upgrade_row(OPEN, 1, **values)
    table = world.upgrade_table()
    at = _row_at(world, OPEN, 1)
    for name, value in values.items():
        assert table[name][at] == value  # type: ignore[literal-required]


def test_define_upgrade_row_refuses_a_partial_row() -> None:
    """There is no two-column form."""
    world = cachette.World()
    with pytest.raises(TypeError):
        world.define_upgrade_row(OPEN, 1, work=4)  # type: ignore[call-arg]


def test_define_upgrade_row_refuses_a_category_and_a_level_it_does_not_hold() -> None:
    world = cachette.World()
    with pytest.raises(cachette.VerbError):
        world.define_upgrade_row(OPEN + 1, 1, **_row())
    with pytest.raises(cachette.VerbError):
        world.define_upgrade_row(OPEN, 0, **_row())
    with pytest.raises(cachette.VerbError):
        world.define_upgrade_row(OPEN, _levels(world) + 1, **_row())


def test_a_build_order_the_ground_refuses_states_the_refusal(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=1)
    address = _address_of_ground(world, PLAIN)
    world.found_settlements([address], faction=0)
    world.step(threads=1)
    units = world.spawn_soldiers([address], faction=0)
    # The open category holds no row, so the engine refuses and says so.
    with pytest.raises(cachette.VerbError, match="refused"):
        world.order_build(units, OPEN)
    assert world.build_order(int(units[0])) is None


def test_a_row_written_at_run_time_takes_a_build_order(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=1)
    address = _address_of_ground(world, PLAIN)
    world.found_settlements([address], faction=0)
    world.step(threads=1)
    units = world.spawn_soldiers([address], faction=0)
    world.define_upgrade_row(OPEN, 1, **_row(ground_fit=1 << PLAIN, work=2))
    # The row asks for no held ground, so a project must zone the tile.
    world.zone_projects(0, [address], OPEN)
    world.order_build(units, OPEN)
    assert world.build_order(int(units[0])) == OPEN
    world.step(threads=1)
    world.step(threads=1)
    report = world.tile_report(*address)
    assert report["upgrade"] == OPEN
    assert report["upgrade_level"] == 1
    assert report["upgrade_complete"]


def test_the_tile_report_states_the_level_and_the_work_toward_the_next(
    seed: int,
) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=1)
    address = _address_of_ground(world, PLAIN)
    world.found_settlements([address], faction=0)
    world.step(threads=1)
    units = world.spawn_soldiers([address], faction=0)
    # A road asks for no held ground, so a project must zone the tile first.
    world.zone_projects(0, [address], ROAD)
    world.order_build(units, ROAD)
    world.step(threads=1)
    report = world.tile_report(*address)
    assert report["upgrade"] == ROAD
    assert report["upgrade_level"] == 0
    assert report["upgrade_progress"] > 0
    assert not report["upgrade_complete"]
