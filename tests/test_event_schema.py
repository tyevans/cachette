"""Tests of the one declaration that every event column comes from.

The engine declares the fields of an event in one place. The compiled module
reports that declaration, the column methods build their arrays from it, and
a script writes the type stub from it.[^1]

These tests hold no field list of their own. A test that wrote one would be a
second declaration site of the thing under test.[^2]

References
----------
[^1]: ADR-0163, an event declares its layout once and the binding derives
    every column.
    ``docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md``
[^2]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import numpy as np

from cachette import World, _core

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts" / "generate_event_stubs.py"

# The schema names an event. Some events also carry a named method that
# returns their columns, beside the general reader every event has. This
# pairing is the only thing the test states, because a method name is not part
# of the declaration, and it covers the events that have one rather than the
# whole schema.
METHODS = {
    "tile_changed": "event_log_columns",
    "resource_taken": "gather_log_columns",
    "unit_fell": "fell_log_columns",
    "unit_starved": "starved_log_columns",
    "site_shortfall": "shortfall_log_columns",
    "site_rationed": "rationed_log_columns",
    "unit_promoted": "promoted_log_columns",
    "relation_crossed": "relation_log_columns",
    "unit_converted": "converted_log_columns",
    "trade_spoken": "trade_log_columns",
    "campaign_event": "campaign_log_columns",
}


def test_the_module_reports_a_schema_for_every_event() -> None:
    # Compare the schema against the register of logs the engine offers, and
    # not against a list this file holds. A list here would be a second
    # declaration of the thing under test.
    world = World(width=8, height=8, seed=7)
    schema = _core.event_schema()
    assert set(schema) == set(world.log_names()), (
        "the schema and the register of logs disagree"
    )
    for event, fields in schema.items():
        assert fields, f"{event} declares no column"
        names = [column for column, _ in fields]
        assert len(names) == len(set(names)), f"{event} repeats a column name"
        for _, dtype in fields:
            assert not dtype.startswith("float"), f"{event} crosses a float"


def _check_columns(
    columns: dict[str, object], fields: list[tuple[str, str]], reader: str
) -> None:
    """Check one reading against the fields the schema declares for it."""
    for column, dtype in fields:
        assert column in columns, f"{reader} gives no {column} column"
        assert columns[column].dtype == np.dtype(dtype)
    lengths = {len(array) for array in columns.values()}
    assert len(lengths) == 1, f"{reader} gives columns of unequal length"


def test_every_log_gives_the_columns_the_schema_declares() -> None:
    # Drive the engine, not the declaration. A method that dropped a column
    # would pass a test of the declaration alone. Every event is reached
    # through the general reader, so no event escapes this by having no named
    # method.
    world = World(width=8, height=8, seed=7)
    world.step(threads=1)
    schema = _core.event_schema()
    for event in world.log_names():
        _check_columns(world.log(event), schema[event], f"log({event!r})")


def test_a_named_method_gives_what_the_general_reader_gives() -> None:
    # A named method is a second way to reach one log. The two must agree,
    # or a caller reads a different thing depending on which it picked.
    world = World(width=8, height=8, seed=7)
    world.step(threads=1)
    schema = _core.event_schema()
    for event, method in METHODS.items():
        columns = getattr(world, method)()
        _check_columns(columns, schema[event], method)
        assert set(columns) == set(world.log(event)), (
            f"{method} and log({event!r}) give different columns"
        )


def test_the_type_stub_follows_the_engine() -> None:
    # The stub blocks are written from the schema. This runs the generator in
    # check mode, so a field added to an event fails here until the stub
    # follows.
    done = subprocess.run(
        [sys.executable, str(GENERATOR), "--check"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert done.returncode == 0, done.stdout + done.stderr
