"""The controller plays inside the step, and the demonstration reads the end.

A world seeds itself from its seed. A controller inside the step orders each
faction to gather and to build, the game ends on territory at a tick limit, and
a census says what every subsystem produced. Python drives no verb here: it
seeds once, steps, and reads.[^1]

The census names come from one Rust table. The test that lists them pins the
public interface, so a name that leaves the table fails here rather than in a
watcher's terminal.[^2]

References
----------
[^1]: ADR-0144, a faction controller runs inside the step and acts only through
the caller's verbs.
``docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md``

[^2]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import pytest

from cachette import VerbError, World
from cachette.demo.app import main

# A world small enough to seed and step many times in a test, and large enough
# for the founding survey to seat a faction.
EXTENT = 48
SEED = 0x0CAC_4E77_0472
FACTIONS = 2

# The names the engine table holds, in table order.
CENSUS_NAMES = [
    "units",
    "settlements",
    "seats_filled",
    "characters",
    "upgrades_complete",
    "luxury_tiles",
    "storms_raised",
    "contracts",
    "controller_commands",
    "controller_refused",
    "game_ended",
    "relation_moves",
    "wars_declared",
]


def seeded_world() -> World:
    """Build and seed a small world."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    reports = world.seed_world()
    assert any(report["seated"] for report in reports)
    return world


def test_a_world_seeds_itself_once_and_the_seeding_verbs_still_serve() -> None:
    """One call founds every faction and places the luxuries; a second refuses."""
    world = seeded_world()
    assert world.soldier_count > 0
    assert world.settlement_count > 0
    assert world.luxury_tile_count > 0
    with pytest.raises(VerbError):
        world.seed_world()
    # A caller that wants its own founding still has the verb.
    other = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    reports = other.found_run_for_every_faction(64)
    assert len(reports) == FACTIONS


def test_the_faction_weights_are_whole_numbers_from_the_seed() -> None:
    """The vector has four whole weights, and a wrong faction number raises."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    same = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    weights = world.faction_weights(0)
    assert set(weights) == {"war", "trade", "build", "renown"}
    assert all(isinstance(value, int) and value > 0 for value in weights.values())
    assert weights == same.faction_weights(0)
    with pytest.raises(VerbError):
        world.faction_weights(FACTIONS)


def test_the_externally_controlled_flag_is_off_and_settable() -> None:
    """The flag starts off, a caller sets it, and the census shows the silence."""
    world = seeded_world()
    assert world.is_externally_controlled(0) is False
    world.step(1)
    active = world.subsystem_census()["controller_commands"]
    assert active == FACTIONS * world.controller_evaluations
    world.set_externally_controlled(0, True)
    world.set_externally_controlled(1, True)
    assert world.is_externally_controlled(1) is True
    world.step(1)
    assert world.subsystem_census()["controller_commands"] == 0
    with pytest.raises(VerbError):
        world.set_externally_controlled(FACTIONS, True)


def test_the_game_ends_once_on_territory_and_the_world_keeps_stepping() -> None:
    """The record appears at the limit, names a winner, and does not move."""
    world = seeded_world()
    assert world.game_end() is None
    world.set_tick_limit(3)
    assert world.tick_limit == 3
    for _ in range(3):
        world.step(1)
    end = world.game_end()
    assert end is not None
    assert end["path"] == "territory"
    assert end["tick"] == 3
    assert end["winner"] in range(FACTIONS)
    assert isinstance(world.score(end["winner"]), int)
    before = world.state_hash()
    for _ in range(5):
        world.step(1)
    assert world.game_end() == end
    assert world.tick == 8
    assert world.state_hash() != before
    assert world.subsystem_census()["controller_commands"] == 0
    assert world.subsystem_census()["game_ended"] == 1
    with pytest.raises(VerbError):
        world.score(FACTIONS)


def test_the_census_keys_are_the_names_of_the_one_rust_table() -> None:
    """The dictionary keys equal the table names, in table order."""
    world = seeded_world()
    world.step(1)
    census = world.subsystem_census()
    assert list(census) == CENSUS_NAMES
    assert all(isinstance(value, int) for value in census.values())
    assert census["units"] > 0
    assert census["luxury_tiles"] > 0
    assert census["controller_commands"] > 0


def test_the_demonstration_runs_to_the_end_and_prints_the_census(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The headless run names the winner once and prints every census row."""
    status = main(
        [
            "--run-to-end",
            "--extent",
            str(EXTENT),
            "--factions",
            str(FACTIONS),
            "--tick-limit",
            "4",
            "--threads",
            "1",
        ]
    )
    assert status == 0
    out = capsys.readouterr().out
    lines = out.splitlines()
    wins = [line for line in lines if "wins by territory" in line]
    assert wins == ["tick 4: faction " + wins[0].split("faction ")[1]]
    assert any(line.startswith("the game ended at tick 4:") for line in lines)
    assert any(line.startswith("census at tick 4") for line in lines)
    for name in CENSUS_NAMES:
        assert any(line.strip().startswith(f"{name}:") for line in lines), name
