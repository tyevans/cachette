"""The chain from a queued settler to a founded city, driven end to end.

A faction founds a city through four links, and each one gates the next. The
queue verb takes a unit type, so one row of the action table names the
settler. The push reaches the queue of a site the engine picks. The production
pass finishes the entry. The settle verb then founds from the settler that
stands on ground a city may take.

No trained policy of this project has founded a city, and the settle row was
never offered as legal in a recorded sample of decisions. **A measurement
showed that no link of the chain refuses: the settler row is legal and no
policy takes it.** These tests hold that finding, so a change that breaks a
link fails here rather than in a training run three days later.

# Every row is resolved by identity

The action table has been renumbered once. No test here holds a row number.
Each test reads the schema the engine publishes, takes the block of the verb
by name, and takes the row that names the settler by the stride of the unit
type position.[^1] It finds the settler by the settle column of the unit type
table, and never by a type index.[^2]

# The tests drive the engine and not the mechanism

The action verb is what a learner reaches, so every test below sends an
action integer. A test that pushed a queue entry directly would prove that
the queue works and nothing about whether an action reaches it.[^3]

# References

[^1]: ADR-0176, an action integer is a mixed radix over the argument positions
each verb declares, decision D1.
``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
[^2]: ADR-0145, a unit type is a row of capability columns, and zero means
cannot, decision D2.
``docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md``
[^3]: Testing rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import ViewError, World
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.policy import ActionTable
from cachette.learn.reward import Weighting
from cachette.learn.signals import SignalCatalogue

# The world the tests play. It is the world the training runs play, because
# the finding this file holds was measured on that world.
EXTENT = 48
FACTIONS = 3
SEAT = 0

# The ticks one test drives. The queue advance acts on a schedule and the
# entry needs several advances, so a short run reaches no settler at all.
TICKS = 600

# Where the seed search starts. The probe script reads from the same start,
# so a failure here repeats under that script.
SEED_START = 50_000

# A weighting with every terminal weight set, so the reward runs. The values
# are the test's own and they state no rule of the downstream game.
WEIGHTING = Weighting(terms={}, won=1.0, lost=-1.0, drawn=0.0)

CONFIG = EnvConfig(
    width=EXTENT,
    height=EXTENT,
    faction_count=FACTIONS,
    seat=SEAT,
    tick_limit=TICKS,
    horizon=120,
    decision_interval=5,
)


def a_world(seed: int) -> World:
    """Build a seeded world that gives the seat to the caller."""
    world = World(width=EXTENT, height=EXTENT, seed=seed, faction_count=FACTIONS)
    world.seed_world()
    world.set_win_readers_enabled(True)
    world.set_tick_limit(TICKS)
    world.set_externally_controlled(SEAT, True)
    return world


def settler_type_of(world: World) -> int:
    """Return the unit type row whose settle column is above zero."""
    column = np.asarray(world.unit_type_table()["settle_group"])
    rows = np.flatnonzero(column > 0)
    assert rows.size > 0, "no row of the unit type table founds a city"
    return int(rows[0])


def rows_of(world: World) -> tuple[int, int]:
    """Return the row that queues a settler and the row that founds a city."""
    table = ActionTable.of_schema(world.action_schema())
    queue = table.named("queue")
    settle = table.named("settle")
    assert queue is not None, "the table holds no queue verb"
    assert settle is not None, "the table holds no settle verb"
    candidates = queue.candidates()
    assert "unit_type" in candidates, (
        f"the queue verb declares no unit type position. It declares {candidates}."
    )
    coordinates = [0] * len(candidates)
    coordinates[candidates.index("unit_type")] = settler_type_of(world)
    return int(queue.row_of(coordinates)), int(settle.first)


def test_the_settler_row_of_the_action_table_is_offered_as_legal() -> None:
    """The first link of the chain does not refuse.

    **This is the measurement that redirected the work.** The row is legal
    while the seat owns a site whose queue has room, which is the state a
    seeded world starts in. A policy that never founds a city is therefore
    not blocked here.
    """
    world = a_world(viable_seeds(CONFIG, 1, SEED_START)[0])
    queue_row, settle_row = rows_of(world)
    mask = world.legal_actions(SEAT)

    assert mask[queue_row] == 1, "the seat owns a site with room, so the row is legal"
    assert mask[settle_row] == 0, "the seat holds no settler, so it founds nothing"


def test_the_settler_row_reaches_the_site_queue() -> None:
    """The second link of the chain does not refuse.

    A refused push answers one byte and changes nothing, so the test reads the
    queue of the site as well as the answer. **The answer alone would pass
    against a verb that reported success and pushed nothing.**
    """
    world = a_world(viable_seeds(CONFIG, 1, SEED_START)[0])
    queue_row, _ = rows_of(world)
    settler = settler_type_of(world)
    site = int(world.log("settlement_founded")["settlement"][0])
    before = len(world.site_queue(site)["unit_type"])

    applied = world.act(SEAT, queue_row)

    after = np.asarray(world.site_queue(site)["unit_type"])
    assert applied, "the verb took the action"
    assert len(after) == before + 1, "the push reached the queue of the site"
    assert int(after[-1]) == settler, "the entry names the settler"


def test_a_queued_settler_finishes_and_founds_a_city() -> None:
    """The third and the fourth links of the chain do not refuse.

    The test queues a settler whenever the row is legal and founds whenever
    the settle row is legal. It drives the action verb, so it proves that an
    action integer reaches a founded city.

    **The founding log is read after each action and after each tick.** An
    action founds a city between two ticks and the step empties the log, so a
    reader that only looked after a tick would miss every founding.
    """
    world = a_world(viable_seeds(CONFIG, 1, SEED_START)[0])
    queue_row, settle_row = rows_of(world)
    seeded = {int(site) for site in world.log("settlement_founded")["settlement"]}
    founded: set[int] = set()
    settlers_seen = 0

    def collect() -> None:
        log = world.log("settlement_founded")
        for site, faction in zip(log["settlement"], log["faction"], strict=True):
            if int(faction) == SEAT and int(site) not in seeded:
                founded.add(int(site))

    for _ in range(TICKS):
        mask = world.legal_actions(SEAT)
        if mask[settle_row]:
            world.act(SEAT, settle_row)
        elif mask[queue_row]:
            world.act(SEAT, queue_row)
        collect()
        world.step(1)
        collect()
        settlers_seen = max(settlers_seen, world.settler_count(SEAT))
        if founded:
            break

    assert settlers_seen > 0, (
        "the production pass finished no settler, so the third link refuses"
    )
    assert founded, "the settle verb founded no city, so the fourth link refuses"


def test_the_observation_publishes_the_settlers_of_the_seat() -> None:
    """The catalogue names the settler count, and the value inverts to it.

    A reward term that pays for a settler reads this signal. **A tuple of
    names written by hand would be a second declaration of what the engine
    publishes**, so the test asks the catalogue.
    """
    world = a_world(viable_seeds(CONFIG, 1, SEED_START)[0])
    catalogue = SignalCatalogue.of_world(world)

    assert "settlers" in catalogue, (
        f"the catalogue publishes no settler count. It publishes "
        f"{sorted(signal.name for signal in catalogue)}."
    )
    signal = catalogue.signal("settlers")
    assert signal.scalar, "the settler count holds one position"
    assert signal.invertible, "the settler count is a magnitude, so it inverts"

    empty = world.faction_observation(SEAT)
    assert signal.read(empty) == 0.0, "the seat holds no settler at the start"

    queue_row, _ = rows_of(world)
    for _ in range(TICKS):
        if world.legal_actions(SEAT)[queue_row]:
            world.act(SEAT, queue_row)
        world.step(1)
        if world.settler_count(SEAT) > 0:
            break

    count = world.settler_count(SEAT)
    assert count > 0, "the fixture reached no settler, so it measured nothing"
    held = world.faction_observation(SEAT)
    assert signal.read(held) > 0.0, "the array publishes the settler the seat holds"
    assert round(float(signal.quantities(held)[0])) == count, (
        "the published value inverts to the count the engine reader gives"
    )


def test_the_decision_carries_the_reason_the_settle_verb_refused() -> None:
    """The environment reports why each settler founded nothing.

    **A refused verb answers one byte and names nothing.** A caller that read
    that byte could not tell a seat with no settler from a seat whose settler
    stands on held ground, and those two states ask for different actions.
    """
    env = Env(CONFIG, WEIGHTING)
    env.reset(viable_seeds(CONFIG, 1, SEED_START)[0])
    queue_row, _ = rows_of(env.world)

    result = env.step(0)
    assert result.info["settle_refusals"] == (), (
        "a seat with no settler names no refusal, because it holds no settler"
    )

    reasons: tuple[str, ...] = ()
    while not env.done:
        mask = env.action_mask()
        action = queue_row if mask[queue_row] else 0
        result = env.step(action)
        reasons = result.info["settle_refusals"]
        if reasons:
            break

    assert reasons, "the seat reached no settler, so the fixture measured nothing"
    known = {
        "accepted",
        "ground_is_held",
        "settlement_stands",
        "ground_admits_nobody",
        "too_close_to_a_city",
        "no_such_unit",
        "not_a_settler",
        "outside_world",
        "faction_may_not_found",
        "founding_refused_the_place",
    }
    assert set(reasons) <= known, (
        f"the engine named a reason nobody declared: {reasons}"
    )
    assert len(reasons) == env.world.settler_count(SEAT), (
        "the reader names one reason for each settler and not one for each refusal"
    )


def test_the_readers_refuse_a_faction_the_world_does_not_hold() -> None:
    """Both readers answer for one faction, and they refuse any other."""
    world = a_world(viable_seeds(CONFIG, 1, SEED_START)[0])
    with pytest.raises(ViewError, match="names no faction"):
        world.settle_refusals(FACTIONS + 1)
    with pytest.raises(ViewError, match="names no faction"):
        world.settler_count(FACTIONS + 1)
