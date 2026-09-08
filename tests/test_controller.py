"""The controller plays inside the step, and the demonstration reads the end.

A world seeds itself from its seed. A controller inside the step orders each
faction to gather and to build, the game ends on territory at a tick limit, and
a census says what every subsystem produced. Python drives no verb here: it
seeds once, steps, and reads.[^1]

The census names come from one Rust table, and that table is the only
declaration of the list.[^2] A list written here as well would be a second
declaration site, and nothing would fail when the two disagreed. The tests
below therefore derive the names from the census and pin only the few names
this file names itself.

References
----------
[^1]: ADR-0144, a faction controller runs inside the step and acts only through
the caller's verbs.
``docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md``

[^2]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``

[^3]: ADR-0152, a faction plans its roads and zones with one solver,
decision D5.
``docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md``
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
    """The vector has five whole weights, and a wrong faction number raises."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    same = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    weights = world.faction_weights(0)
    assert set(weights) == {"war", "trade", "build", "renown", "settle"}
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
    # The stage emits the evaluation commands and, for a faction whose plan
    # the solver filled, one project order beside them.[^3]
    assert active >= FACTIONS * world.controller_evaluations
    world.set_externally_controlled(0, True)
    world.set_externally_controlled(1, True)
    assert world.is_externally_controlled(1) is True
    world.step(1)
    # The row is a total for the run, so the silence shows as a count that
    # stops rising. It does not show as a zero.
    assert world.subsystem_census()["controller_commands"] == active
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
    busy = world.subsystem_census()["controller_commands"]
    assert busy > 0, "the controller acted before the game ended"
    for _ in range(5):
        world.step(1)
    assert world.game_end() == end
    assert world.tick == 8
    assert world.state_hash() != before
    # The controller emits nothing after the end. The census still says what
    # it did, because the row is a total for the run and not a reading of the
    # last tick.
    assert world.subsystem_census()["controller_commands"] == busy
    assert world.subsystem_census()["game_ended"] == 1
    with pytest.raises(VerbError):
        world.score(FACTIONS)


def test_the_census_answers_one_integer_for_each_name_of_the_rust_table() -> None:
    """Every key is a distinct name and every value is an integer.

    **This test does not list the names.** The engine table is the only
    declaration of the list, and a copy of it here would be a second
    declaration site with nothing to fail when the two disagreed.[^2] It once
    held such a copy, the table gained nine rows, and this test failed for a
    reason that was not a defect.

    What is left is what the boundary can get wrong on its own: a duplicated
    key, a value that is not an integer, and an order that moves between two
    readings of one world. The four names below are the ones this file reads,
    so they are the interface it depends on.
    """
    world = seeded_world()
    world.step(1)
    census = world.subsystem_census()
    assert census, "the census answered no row at all"
    assert len(set(census)) == len(census), "the census repeated a name"
    assert all(isinstance(name, str) and name for name in census)
    assert all(isinstance(value, int) for value in census.values())
    assert list(census) == list(world.subsystem_census()), "the order moved"
    assert census["units"] > 0
    assert census["luxury_tiles"] > 0
    assert census["controller_commands"] > 0
    assert "game_ended" in census


def test_the_demonstration_runs_to_the_end_and_prints_the_census(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The headless run names the winner once and prints every census row.

    The names to look for come from the census of a world of the same shape,
    so the check is derived from the engine table rather than from a list kept
    in this file.[^2]
    """
    # The seed is named. The demonstration draws its own seed when none is
    # given, so a test that leaves it out builds a different world on every
    # run, and some of those worlds seat no faction at all.
    status = main(
        [
            "--run-to-end",
            "--seed",
            hex(SEED),
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
    # The demonstration names a faction, so the line carries a name and not
    # the word "faction". The test asserts the shape of the line and that
    # exactly one appears, because the run must name the winner once.
    wins = [line for line in lines if "wins by territory" in line]
    assert len(wins) == 1, f"the run must name the winner once, and it gave {wins}"
    assert wins[0].startswith("tick 4: ")
    assert wins[0].endswith(" wins by territory")
    assert any(line.startswith("the game ended at tick 4:") for line in lines)
    assert any(line.startswith("census of the run at tick 4") for line in lines)
    printed = World(
        width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS
    ).subsystem_census()
    assert printed, "the census answered no row to look for"
    for name in printed:
        assert any(line.strip().startswith(f"{name}:") for line in lines), name


def test_the_standing_of_a_faction_names_every_path_and_the_paths_are_four() -> None:
    """One running value per path, and a stranger is refused."""
    world = seeded_world()
    world.step(1)
    standing = world.standing(0)
    assert list(standing) == [
        "held_tiles",
        "seats_held",
        "live_units",
        "store_total",
        "best_renown",
        "wonder_progress",
    ]
    assert all(isinstance(value, int) for value in standing.values())
    assert standing["held_tiles"] == world.score(0)
    with pytest.raises(VerbError):
        world.standing(FACTIONS)
    census = world.subsystem_census()
    assert census["wonders_complete"] == 0
    assert census["stores_built"] == 0


def test_a_character_at_the_renown_target_ends_the_game_by_renown() -> None:
    """The renown path fires only when a caller writes the column."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    world.set_tick_limit(1000)
    address = next(
        (q, r)
        for r in range(EXTENT)
        for q in range(EXTENT)
        if world.tile_report(q, r)["passable"]
    )
    world.spawn_soldiers([address], faction=0)
    world.spawn_soldiers([address], faction=1)
    person = world.create_characters(faction=1, count=1)[0]
    # The engine states the target, and this test reads it. A copy here went
    # stale the day the project owner raised the bar.
    target = world.renown_target
    world.set_character_renown([int(person)], target - 1)
    world.step(1)
    assert world.game_end() is None
    assert world.standing(1)["best_renown"] == target - 1
    world.set_character_renown([int(person)], target)
    world.step(1)
    end = world.game_end()
    assert end is not None
    assert end["path"] == "renown"
    assert end["winner"] == 1
