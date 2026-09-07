"""The balance parameter manifest agrees with the engine.

The manifest is a file that a tuner reads so that it needs no knowledge of the
engine. It names one row for each win-path value a caller may set: the setter
to call, whether the value is a threshold or a rate, its scale, its default and
its search bounds.

**The manifest is a second declaration site for every default.** The engine is
the first. These tests derive the engine value from a world and compare, so a
default that drifts fails a check rather than misleading a tuner in silence.[^1]

References
----------
[^1]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import json
import pathlib

import pytest

from cachette import World

MANIFEST = (
    pathlib.Path(__file__).resolve().parents[1] / "scripts" / "balance_params.json"
)

KINDS = {"threshold", "rate"}
SCALES = {"fix32", "int"}
PATHS = {"domination", "territory", "renown", "wonder", None}


def manifest() -> list[dict]:
    """Read the rows of the manifest."""
    return json.loads(MANIFEST.read_text())["params"]


def a_world() -> World:
    """Build a small world that nobody configures."""
    return World(16, 16, seed=1, faction_count=2)


def test_the_manifest_holds_a_row_for_every_shape_the_tuner_expects() -> None:
    rows = manifest()
    assert rows, "the manifest holds at least one row"
    names = [row["name"] for row in rows]
    assert len(names) == len(set(names)), "each name occurs once"
    for row in rows:
        assert set(row) == {
            "name",
            "setter",
            "kind",
            "scale",
            "default_raw",
            "min_raw",
            "max_raw",
            "path",
            "doc",
        }, row["name"]
        assert row["kind"] in KINDS, row["name"]
        assert row["scale"] in SCALES, row["name"]
        assert row["path"] in PATHS, row["name"]
        assert row["doc"], row["name"]
        assert row["min_raw"] <= row["default_raw"] <= row["max_raw"], row["name"]


def test_every_setter_the_manifest_names_is_a_method_of_a_world() -> None:
    world = a_world()
    for row in manifest():
        setter = getattr(world, row["setter"], None)
        assert callable(setter), f"{row['setter']} is not a method of a world"


def test_every_default_the_manifest_holds_is_the_value_the_engine_holds() -> None:
    # The engine is the first declaration site. The manifest is the second.
    world = a_world()
    engine = {
        "renown_target": world.renown_target,
        "renown_per_fell": world.renown_per_fell,
        "wonder_work": world.wonder_work,
        "wonder_victory_claim": world.wonder_victory_claim,
        "tick_limit": world.tick_limit,
    }
    rows = {row["name"]: row for row in manifest()}
    assert set(rows) == set(engine), "the manifest and the engine name one set"
    for name, value in engine.items():
        assert rows[name]["default_raw"] == value, name


def test_every_setter_moves_the_value_the_manifest_names() -> None:
    # A setter that the manifest names and nothing reads back would leave a
    # tuner searching a value the engine never took.
    world = a_world()
    for row in manifest():
        setter = getattr(world, row["setter"])
        wanted = row["min_raw"]
        setter(wanted)
        assert getattr(world, row["name"]) == wanted, row["name"]


def test_the_readers_switch_is_off_and_on_and_the_default_is_on() -> None:
    world = a_world()
    assert world.win_readers_enabled is True
    world.set_win_readers_enabled(False)
    assert world.win_readers_enabled is False
    world.set_win_readers_enabled(True)
    assert world.win_readers_enabled is True


def test_a_wonder_setter_refuses_when_the_table_holds_no_wonder_row() -> None:
    from cachette import VerbError

    world = a_world()
    # An empty row takes the wonder out of the table, because a row that fits
    # no ground is the absence of a row.
    world.define_upgrade_row(
        2,
        1,
        ground_fit=0,
        work=0,
        yield_change=0,
        recovery_change=0,
        capacity_change=0,
        capacity_of_store_change=0,
        housing_change=0,
        victory_claim=0,
        own_ground_required=0,
    )
    with pytest.raises(VerbError):
        world.set_wonder_work(10)
    with pytest.raises(VerbError):
        world.set_wonder_victory_claim(1)


def test_the_standing_reports_a_running_value_for_every_win_path() -> None:
    # The tuner records trajectories through this reading, so it must hold the
    # quantity each of the four readers compares.
    world = a_world()
    world.seed_world()
    world.step(1)
    standing = world.standing(0)
    assert set(standing) == {
        "held_tiles",
        "seats_held",
        "live_units",
        "store_total",
        "best_renown",
        "wonder_progress",
    }
