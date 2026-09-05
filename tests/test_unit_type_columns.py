"""A unit type is a row of capability columns, and zero means cannot.

The engine declares the columns of a row once, and the type stub names them
by hand. A test here reads the stub and asserts that it names the columns
the engine returns, in the same order, so the table is declared once and a
second copy fails when it drifts.[^1]

The verbs that give an order refuse a unit whose type cannot take it, and
the verb that defines a type takes the whole row.[^2]

References
----------
[^1]: Recurring Defect Shapes, shape 1.
``.agents/rules/recurring-defects.md``

[^2]: ADR-0145, a unit type is a row of capability columns, and zero means
cannot, decisions D2 and D5.
``docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md``
"""

from __future__ import annotations

import ast
import pathlib

import numpy as np
import pytest

import cachette

ONE = 65536

# The row numbers the tests write. They sit above the five default rows, so
# the worker every new soldier carries is untouched.
CANNOT = 5
CAN = 6

# The stub is hand-written and nothing regenerates it, so a test reads it
# from the source tree beside the package rather than from the install.
STUB = (
    pathlib.Path(__file__).resolve().parent.parent / "python" / "cachette" / "_core.pyi"
)


def _stub_column_names() -> list[str]:
    """Return the field names of ``UnitTypeColumns`` in the stub, in order."""
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == "UnitTypeColumns":
            return [
                statement.target.id
                for statement in node.body
                if isinstance(statement, ast.AnnAssign)
                and isinstance(statement.target, ast.Name)
            ]
    message = "the stub declares no UnitTypeColumns class"
    raise AssertionError(message)


def _stub_define_keywords() -> list[str]:
    """Return the keyword-only parameter names of ``define_unit_type``."""
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == "define_unit_type":
            return [argument.arg for argument in node.args.kwonlyargs]
    message = "the stub declares no define_unit_type method"
    raise AssertionError(message)


def _open_address(world: cachette.World) -> tuple[int, int]:
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def _row(**changes: int) -> dict[str, int]:
    """Return a full row at zero, with the named columns changed."""
    row = dict.fromkeys(_stub_column_names(), 0)
    row.update(changes)
    return row


def _define(world: cachette.World, number: int, **changes: int) -> None:
    row = _row(**changes)
    attack = row.pop("attack")
    armour = row.pop("armour")
    world.define_unit_type(number, attack, armour, **row)


def test_the_stub_names_the_columns_the_engine_returns_in_the_same_order() -> None:
    """The typed dictionary in the stub and the engine table agree.

    The engine derives its keys from the row declaration, so this compares
    the hand-written stub against the one declaration.
    """
    table = cachette.World().unit_type_table()
    assert list(table) == _stub_column_names()


def test_the_stub_keyword_arguments_are_the_columns_beyond_attack_and_armour() -> None:
    table = cachette.World().unit_type_table()
    assert ["attack", "armour", *_stub_define_keywords()] == list(table)


def test_every_column_is_a_wide_integer_array_of_the_table_width() -> None:
    table = cachette.World().unit_type_table()
    widths = {len(table[name]) for name in table}  # type: ignore[literal-required]
    assert len(widths) == 1
    for name in table:
        assert table[name].dtype == np.int64  # type: ignore[literal-required]


def test_a_new_world_holds_the_default_rows() -> None:
    """Row zero is a worker: it gathers, builds and carries, and it does not fight."""
    table = cachette.World().unit_type_table()
    assert table["attack"][0] == 0
    assert table["gather_rate"][0] > 0
    assert table["build_rate"][0] > 0
    assert table["carry_capacity"][0] > 0
    # The soldier row fights and does nothing else.
    assert table["attack"][1] > 0
    assert table["gather_rate"][1] == 0
    assert table["carry_capacity"][1] == 0


def test_define_unit_type_takes_the_whole_row_and_reads_it_back() -> None:
    world = cachette.World()
    values = {name: index + 1 for index, name in enumerate(_stub_column_names())}
    _define(world, CAN, **values)
    table = world.unit_type_table()
    for name, value in values.items():
        assert table[name][CAN] == value  # type: ignore[literal-required]


def test_define_unit_type_refuses_a_partial_row() -> None:
    """There is no two-column form."""
    world = cachette.World()
    with pytest.raises(TypeError):
        world.define_unit_type(CAN, ONE, 0)  # type: ignore[call-arg]


def test_define_unit_type_refuses_a_negative_fixed_point_column() -> None:
    world = cachette.World()
    with pytest.raises(cachette.VerbError, match="gather_rate"):
        _define(world, CAN, gather_rate=-1)


def test_order_gather_refuses_a_unit_whose_type_cannot_gather() -> None:
    world = cachette.World(width=16, height=16, seed=1, faction_count=2)
    _define(world, CANNOT, gather_rate=0, carry_capacity=1 << 20)
    _define(world, CAN, gather_rate=ONE, carry_capacity=1 << 20)
    place = _open_address(world)
    units = world.spawn_soldiers([place, place], faction=0)
    world.set_unit_types(units[:1], CANNOT)
    world.set_unit_types(units[1:], CAN)

    with pytest.raises(cachette.VerbError, match="cannot gather"):
        world.order_gather(units, 0)

    # The set is all or nothing. The unit that can gather took no order.
    world.step(1)
    assert len(world.gather_log_columns()["unit"]) == 0

    # The unit that can gather takes the order on its own.
    world.order_gather(units[1:], 0)


def test_order_build_refuses_a_unit_whose_type_cannot_build() -> None:
    world = cachette.World(width=16, height=16, seed=1, faction_count=2)
    _define(world, CANNOT, build_rate=0)
    _define(world, CAN, build_rate=ONE)
    place = _open_address(world)
    units = world.spawn_soldiers([place, place], faction=0)
    world.set_unit_types(units[:1], CANNOT)
    world.set_unit_types(units[1:], CAN)

    with pytest.raises(cachette.VerbError, match="cannot build"):
        world.order_build(units, 0)

    world.order_build(units[1:], 0)
