"""Black-box tests of the unit type reads and of the five step logs.

Every test here starts at the Python boundary. The core held the type table,
the five logs and the upkeep rate before any binding read one, and their own
Rust tests passed the whole time. A test that built the mechanism again would
prove the same thing again. Each test below drives the installed package.

References
----------
[^1]: Findings register, FND-460 and FND-461. ``docs/FINDINGS.md``

Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette

# The raw integer of one whole unit in the project fixed-point scale. Every
# attack, every armour and every rate below is a multiple of it.
ONE = 65536

# The row of the type table that a bowman takes, and the row a tank takes.
# Neither number means anything to the engine. The rows the test writes are
# what give them meaning.
BOWMAN = 0
TANK = 1


def _worker_columns(world: cachette.World) -> dict[str, int]:
    """Return the six columns beyond attack and armour, as the worker holds them.

    The worker is row zero of the table a new world is built with. A fighter
    row that keeps these columns differs from the default in the two columns
    the contest reads and in nothing else. The values are read from the engine
    rather than written here, so the test holds no second copy of them.
    """
    table = world.unit_type_table()
    return {
        name: int(table[name][0])  # type: ignore[literal-required]
        for name in table
        if name not in ("attack", "armour")
    }


# How many bowmen stand against the one tank.
#
# **The number is the point of the test.** The threshold refuses each bowman
# before anything is added, so a sum of zeroes stays zero however many stand
# there. A fixture with two bowmen would pass against an engine that added
# them up and still fell short.
BOWMEN = 32

# The steps the tank needs to end that many bowmen, with room to spare.
CONTEST_CEILING = 40

# The world that the log tests run in.
#
# **The logs need a world that goes short, and a small world does not.** A
# world of two factions on 32 tiles a side feeds itself: every unit keeps a
# full need for hundreds of steps, and no site ever rations. The extent and
# the faction count below are what make the sites compete for ground, and a
# probe that stepped the smaller world 400 times recorded no entry in any of
# the three logs.
SHORT_EXTENT = 64
SHORT_FACTIONS = 4

# How many steps each log test may take before it gives up.
#
# A rationed entry lands on the first scheduled step, a promotion inside the
# first fifty, and a starvation inside the first two hundred. The ceiling
# leaves room and fails rather than looping.
LOG_CEILING = 220


def _passable(world: cachette.World, q: int, r: int) -> bool:
    """Report whether the ground at an address admits a unit."""
    return bool(world.tile_report(q, r)["passable"])


def _open_address(world: cachette.World) -> tuple[int, int]:
    """Return an address of ground that admits a unit."""
    for q in range(world.width):
        for r in range(world.height):
            if _passable(world, q, r):
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def _armed_world(seed: int) -> cachette.World:
    """Return a world whose table holds a bowman row and a tank row.

    The bowman delivers one whole casualty and carries no armour. The tank
    delivers four and carries two, so the attack of a bowman does not exceed
    the armour of a tank.
    """
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    world.define_unit_type(BOWMAN, 1 * ONE, 0, **_worker_columns(world))
    world.define_unit_type(TANK, 4 * ONE, 2 * ONE, **_worker_columns(world))
    return world


def _short_world(seed: int) -> cachette.World:
    """Return a founded world whose sites cannot feed everybody."""
    world = cachette.World(
        width=SHORT_EXTENT,
        height=SHORT_EXTENT,
        seed=seed,
        faction_count=SHORT_FACTIONS,
    )
    world.found_run_for_every_faction()
    return world


def _step_until(
    world: cachette.World, read: str, threads: int = 2
) -> dict[str, np.ndarray]:
    """Step until one named log holds an entry, and return that log."""
    for _ in range(LOG_CEILING):
        world.step(threads=threads)
        columns: dict[str, np.ndarray] = getattr(world, read)()
        if len(columns["tick"]):
            return columns
    message = f"{read} held no entry in {LOG_CEILING} steps"
    raise AssertionError(message)


def test_one_tank_beats_any_number_of_bowmen(seed: int) -> None:
    # The whole path, from the boundary: write two rows, give a set of units
    # each row, step, and read what the meeting cost each faction.
    world = _armed_world(seed)
    place = _open_address(world)
    bowmen = world.spawn_soldiers([place] * BOWMEN, faction=0)
    tank = world.spawn_soldiers([place], faction=1)
    world.set_unit_types(bowmen, BOWMAN)
    world.set_unit_types(tank, TANK)
    assert world.faction_population() == [BOWMEN, 1]

    for _ in range(CONTEST_CEILING):
        world.step(threads=2)
        # The tank never falls. The bowmen do not reach it at any count, so
        # this holds on every step and not only at the end.
        assert world.faction_population()[1] == 1
        if world.faction_population()[0] == 0:
            break
    assert world.faction_population() == [0, 1]


def test_a_table_nobody_wrote_leaves_every_unit_harmless(seed: int) -> None:
    # The control for the test above. The same units meet, and no row of the
    # table is written, so every unit holds zero attack and nobody falls.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    place = _open_address(world)
    world.spawn_soldiers([place] * BOWMEN, faction=0)
    world.spawn_soldiers([place], faction=1)
    for _ in range(8):
        world.step(threads=2)
    assert world.faction_population() == [BOWMEN, 1]


def test_the_type_table_reads_back_what_the_caller_wrote(seed: int) -> None:
    world = _armed_world(seed)
    table = world.unit_type_table()
    assert len(table["attack"]) == len(table["armour"])
    assert table["attack"][BOWMAN] == 1 * ONE
    assert table["armour"][BOWMAN] == 0
    assert table["attack"][TANK] == 4 * ONE
    assert table["armour"][TANK] == 2 * ONE
    # A row that nobody wrote holds zero, and a unit of that row reaches
    # nothing.
    unwritten = [
        row for row in range(len(table["attack"])) if row not in (BOWMAN, TANK)
    ]
    assert unwritten, "the table must hold a row that this test did not write"
    for row in unwritten:
        assert table["attack"][row] == 0
        assert table["armour"][row] == 0


def test_a_soldier_reads_back_the_type_the_set_gave_it(seed: int) -> None:
    world = _armed_world(seed)
    place = _open_address(world)
    units = world.spawn_soldiers([place] * 3, faction=0)
    # A soldier that nothing gave a type carries row zero.
    assert [world.unit_type(int(unit)) for unit in units] == [0, 0, 0]
    world.set_unit_types(units, TANK)
    assert [world.unit_type(int(unit)) for unit in units] == [TANK] * 3


def test_a_type_on_an_identity_the_world_lost_changes_nothing(seed: int) -> None:
    world = _armed_world(seed)
    place = _open_address(world)
    units = world.spawn_soldiers([place] * 2, faction=0)
    world.set_unit_types(units, TANK)
    dead = int(units[0])
    world.despawn_soldiers([dead])
    live = int(units[1])

    with pytest.raises(cachette.ViewError):
        world.set_unit_types([live, dead], BOWMAN)

    # The set is all or nothing. The live soldier keeps the type it held.
    assert world.unit_type(live) == TANK


def test_a_type_the_table_does_not_hold_changes_nothing(seed: int) -> None:
    world = _armed_world(seed)
    place = _open_address(world)
    units = world.spawn_soldiers([place], faction=0)
    world.set_unit_types(units, TANK)
    width = len(world.unit_type_table()["attack"])

    with pytest.raises(cachette.VerbError):
        world.set_unit_types(units, width)

    assert world.unit_type(int(units[0])) == TANK


def test_the_type_read_refuses_an_identity_the_world_lost(seed: int) -> None:
    world = _armed_world(seed)
    place = _open_address(world)
    units = world.spawn_soldiers([place], faction=0)
    dead = int(units[0])
    world.despawn_soldiers([dead])
    with pytest.raises(cachette.ViewError):
        world.unit_type(dead)


def test_a_step_that_starved_nobody_gives_an_empty_starved_log(seed: int) -> None:
    # A world that nothing has starved yet must answer with an empty log, and
    # not with a stale one.
    world = _armed_world(seed)
    place = _open_address(world)
    world.spawn_soldiers([place] * 4, faction=0)
    world.step(threads=2)
    columns = world.starved_log_columns()
    assert len(columns["tick"]) == 0
    assert len(columns["unit"]) == 0
    assert len(columns["deficit"]) == 0


def test_the_starved_log_names_the_units_a_shortage_ended(seed: int) -> None:
    world = _short_world(seed)
    before = world.soldier_count
    columns = _step_until(world, "starved_log_columns")
    assert len(columns["unit"]) == len(columns["tick"]) == len(columns["deficit"])
    assert world.soldier_count == before - len(columns["unit"])
    for tick in columns["tick"]:
        assert tick == world.tick
    for deficit in columns["deficit"]:
        # The deficit carries the fixed-point scale, and a unit ends above
        # zero deficit.
        assert deficit > 0
    for unit in columns["unit"]:
        # The unit is dead, so its identity never resolves again.
        with pytest.raises(cachette.ViewError):
            world.soldier_tile(int(unit))


def test_the_starved_log_holds_the_last_step_alone(seed: int) -> None:
    world = _short_world(seed)
    columns = _step_until(world, "starved_log_columns")
    assert len(columns["tick"])
    # The next step clears the log before it does anything. The scan runs on
    # a period, so the step directly after a scan never ends anybody.
    world.step(threads=2)
    assert len(world.starved_log_columns()["tick"]) == 0


def test_the_rationed_log_names_a_site_that_could_not_serve(seed: int) -> None:
    world = _short_world(seed)
    columns = _step_until(world, "rationed_log_columns")
    for index in range(len(columns["tick"])):
        assert columns["tick"][index] == world.tick
        # The store stopped at zero, so it gave less than the cohorts asked.
        assert columns["granted"][index] < columns["demanded"][index]
        assert columns["demanded"][index] > 0
        assert columns["commodity"][index] == 0
        # The site is alive, and the identity resolves against the engine.
        report = world.site_economy(int(columns["site"][index]))
        assert report["rationed"] is True
        assert report["demanded"] == columns["demanded"][index]
        assert report["granted"] == columns["granted"][index]


def test_the_promoted_log_names_the_soldier_and_the_character(seed: int) -> None:
    world = _short_world(seed)
    columns = _step_until(world, "promoted_log_columns")
    for index in range(len(columns["tick"])):
        assert columns["tick"][index] == world.tick
        # The soldier stays alive, so the engine still answers for it.
        world.soldier_tile(int(columns["unit"][index]))
        assert columns["character"][index] != 0
        # The deeds are a whole count of stock, and a promotion needs some.
        assert columns["deeds"][index] > 0
        assert columns["faction"][index] < SHORT_FACTIONS


def test_a_site_that_spends_nothing_falls_short_of_nothing(seed: int) -> None:
    # The shortfall log answers empty while no site holds an upkeep rate. A
    # new world writes none.
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    world.found_group(64, 0)
    for _ in range(20):
        world.step(threads=2)
        assert len(world.shortfall_log_columns()["tick"]) == 0


def test_the_shortfall_log_names_a_site_that_could_not_pay(seed: int) -> None:
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    site = int(world.found_group(64, 0)["site"])
    # The site earns what the survey read and now spends far more than that,
    # so the store stops at zero and the upkeep cannot take the rest.
    world.spend_at_sites([site], rate=64 * ONE)
    columns = _step_until(world, "shortfall_log_columns")
    assert len(columns["tick"]) == 1
    assert columns["tick"][0] == world.tick
    assert columns["site"][0] == site
    assert columns["amount"][0] > 0
    assert columns["commodity"][0] == 0
    # The next step clears the log, and the rate applies on a period.
    world.step(threads=2)
    assert len(world.shortfall_log_columns()["tick"]) == 0


def test_the_upkeep_verb_refuses_a_site_the_world_does_not_hold(seed: int) -> None:
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    site = int(world.found_group(64, 0)["site"])
    before = world.site_economy(site)["upkeep"]
    with pytest.raises(cachette.ViewError):
        world.spend_at_sites([site, site + 1], rate=8 * ONE)
    assert world.site_economy(site)["upkeep"] == before


def test_the_upkeep_verb_refuses_a_rate_below_zero(seed: int) -> None:
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    site = int(world.found_group(64, 0)["site"])
    before = world.site_economy(site)["upkeep"]
    # **The hash is the assertion, and the rate read is not enough.** The
    # engine opens its rate table before it checks the rate, and an opened
    # table is a change that the whole-world hash covers while every rate in
    # it still reads back as zero. A refusal must change nothing at all.
    hash_before = world.state_hash()
    with pytest.raises(cachette.VerbError):
        world.spend_at_sites([site], rate=-1)
    assert world.site_economy(site)["upkeep"] == before
    assert world.state_hash() == hash_before


def test_the_meeting_gives_one_answer_at_every_thread_count(seed: int) -> None:
    # The binding adds no parallel section, and the pass it reads must not
    # depend on the thread count. One run for each count, compared.
    answers = []
    for threads in (1, 2, 12):
        world = _armed_world(seed)
        place = _open_address(world)
        bowmen = world.spawn_soldiers([place] * BOWMEN, faction=0)
        tank = world.spawn_soldiers([place], faction=1)
        world.set_unit_types(bowmen, BOWMAN)
        world.set_unit_types(tank, TANK)
        for _ in range(4):
            world.step(threads=threads)
        answers.append((world.faction_population(), world.state_hash()))
    assert answers[0] == answers[1] == answers[2]
