"""A faction at war raises a campaign, and a caller can raise one too.

The controller inside the step raises a campaign against a faction in the war
band, through the same core function this side calls. Python sets the
relation, steps, and reads the register, the log and the census. It drives no
unit.[^1]

References
----------
[^1]: ADR-0144, a faction controller runs inside the step and acts only through
the caller's verbs.
``docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md``
[^2]: ADR-0163, an event declares its layout once and the binding derives
every column.
``docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md``
"""

from __future__ import annotations

import pytest

from cachette import VerbError, World

EXTENT = 48
SEED = 0x0CAC_4E77_0478
A = 0
B = 1

# Deep in the war band, so the drift toward peace cannot end the war inside
# the wait.
FAR_BELOW_EVERY_EDGE = -(1 << 20)

# The most ticks a test waits for the war weight to roll a raise.
PATIENCE = 400

# The rows of the default table.
WORKER = 0
SOLDIER = 1

# The event kinds and the states, as the engine numbers them.
RAISED = 0
LIVE = 1


def _seeded() -> World:
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    reports = world.seed_world()
    assert all(report["seated"] for report in reports)
    return world


def _declare_war(world: World) -> None:
    world.set_relation(A, B, FAR_BELOW_EVERY_EDGE)
    world.set_relation(B, A, FAR_BELOW_EVERY_EDGE)


def _step_until_raised(world: World, faction: int) -> int:
    for _ in range(PATIENCE):
        world.step(2)
        log = world.campaign_log_columns()
        for row in range(len(log["tick"])):
            if int(log["kind"][row]) == RAISED and int(log["faction"][row]) == faction:
                return int(log["tick"][row])
    message = f"no campaign raised in {PATIENCE} ticks"
    raise AssertionError(message)


def test_a_faction_at_peace_raises_no_campaign() -> None:
    world = _seeded()
    for _ in range(64):
        world.step(2)
        assert len(world.campaign_log_columns()["tick"]) == 0
    for faction in (A, B):
        assert all(int(state) == 0 for state in world.campaigns(faction)["state"])
    assert world.subsystem_census()["campaigns_raised"] == 0


def test_the_controller_raises_a_campaign_at_war_and_the_register_shows_it() -> None:
    world = _seeded()
    _declare_war(world)
    tick = _step_until_raised(world, A)
    rows = world.campaigns(A)
    live = [row for row in range(len(rows["state"])) if int(rows["state"][row]) == LIVE]
    assert len(live) == 1, "one live campaign for one faction"
    row = live[0]
    assert int(rows["raised_at_tick"][row]) == tick
    assert 1 <= int(rows["cohort_size"][row]) <= world.campaign_cohort_size
    assert int(rows["objective_kind"][row]) in (0, 1)
    # The census row is a total for the run and for every faction. Both
    # factions are at war, so it counts at least the raise of A.
    assert world.subsystem_census()["campaigns_raised"] >= 1
    # The soldiers of the cohort are the units sent on the plane of the faction.
    log = world.campaign_log_columns()
    # The engine declares the fields of the event in one place, and the method
    # gives one column for each of them. The two address columns are the
    # exception: the event holds an index, and the grid turns it into an
    # address.[^2]
    from cachette import _core

    declared = {column for column, _ in _core.event_schema()["campaign_event"]}
    assert set(log) == declared | {"objective_q", "objective_r"}


def test_a_caller_raises_a_campaign_through_the_same_path() -> None:
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    open_tile = None
    for q in range(world.width):
        for r in range(world.height):
            if bool(world.tile_report(q, r)["passable"]):
                open_tile = (q, r)
                break
        if open_tile is not None:
            break
    assert open_tile is not None
    units = world.spawn_soldiers([open_tile] * 3, A)
    world.raise_campaign(A, open_tile[0], open_tile[1], 2)
    rows = world.campaigns(A)
    assert int(rows["state"][0]) == LIVE
    assert int(rows["cohort_size"][0]) == 2
    assert (int(rows["objective_q"][0]), int(rows["objective_r"][0])) == open_tile
    log = world.campaign_log_columns()
    assert len(log["tick"]) == 1
    assert int(log["kind"][0]) == RAISED
    # The two lowest identities became soldiers, and the third kept its type.
    types = [world.unit_type(int(unit)) for unit in sorted(int(unit) for unit in units)]
    assert types == [SOLDIER, SOLDIER, WORKER]
    # A second raise is refused while the first is live, and so is a raise for
    # a faction the world does not hold.
    with pytest.raises(VerbError):
        world.raise_campaign(A, open_tile[0], open_tile[1], 1)
    with pytest.raises(VerbError):
        world.raise_campaign(7, open_tile[0], open_tile[1], 1)
    with pytest.raises(VerbError):
        world.campaigns(7)


def test_the_cohort_size_is_a_parameter_the_caller_sets() -> None:
    world = _seeded()
    world.set_campaign_cohort_size(1)
    assert world.campaign_cohort_size == 1
    _declare_war(world)
    _step_until_raised(world, A)
    rows = world.campaigns(A)
    live = [row for row in range(len(rows["state"])) if int(rows["state"][row]) == LIVE]
    assert int(rows["cohort_size"][live[0]]) == 1
