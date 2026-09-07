"""One verb writes the weight vector of one faction, and the engine reads it.

The weight a faction gives each option is that faction's policy. One verb
writes it, and a Python caller, the built-in controller and a learner all use
that one verb.[^1] The vector is simulated state, so it enters the state hash
and two worlds that differ in a weight part on the next tick.[^2]

Every test here goes through the public interface of the package. The test
that shows the engine reads the vector drives the step and reads the census
afterwards, rather than driving the mechanism.[^3]

**The write-back test is the one that pins the boundary.** A world whose
weights a caller wrote back unchanged must give one event log and one state
hash with an untouched world. A verb that wrote anything else, or that touched
a faction the caller did not name, fails there. The test beside it proves that
the write-back test can fail.[^4]

References
----------
[^1]: ADR-0156, a faction's option weights are policy, set through one verb,
decision D3.
``docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md``

[^2]: ADR-0156, a faction's option weights are policy, set through one verb,
decision D1.
``docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md``

[^3]: Testing Rules, section 5. ``.agents/rules/testing.md``

[^4]: Testing Rules, section 1. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import re

import pytest

from cachette import VerbError, World

# A world small enough to step many times in a test, and large enough for the
# founding survey to seat a faction.
EXTENT = 48
SEED = 0x0CAC_4E77_0472
FACTIONS = 2

# A weight no bound of this project holds. The verb refuses it, and the
# refusal names the bound, so no test here declares the bound a second
# time.[^1]
#
# [^1]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
WEIGHT_OUTSIDE_EVERY_BOUND = 255

# How many steps the write-back test runs. Long enough for the controller, the
# plans and the upgrades to act many times.
EQUIVALENCE_STEPS = 40

# How many steps the behaviour test runs. The tile event log of a world whose
# factions want everything parts from the log of a world whose factions want
# little at about one hundred steps, so this leaves headroom above that.
BEHAVIOUR_STEPS = 120


def seeded_world() -> World:
    """Build and seed a small world."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    reports = world.seed_world()
    assert any(report["seated"] for report in reports)
    return world


def run(world: World, steps: int, threads: int) -> tuple[int, bytes]:
    """Step a world and return its state hash and its whole event log."""
    logs = []
    for _ in range(steps):
        world.step(threads)
        logs.append(world.event_log_bytes())
    return world.state_hash(), b"".join(logs)


def write_back(world: World) -> None:
    """Write every faction the weights it already holds."""
    for faction in range(FACTIONS):
        world.set_faction_weights(faction, **world.faction_weights(faction))


def set_every_weight(world: World, value: int) -> None:
    """Give every faction the same value for every weight a pass reads."""
    for faction in range(FACTIONS):
        weights = world.faction_weights(faction)
        weights["war"] = value
        weights["trade"] = value
        weights["build"] = value
        weights["settle"] = value
        world.set_faction_weights(faction, **weights)


def weight_bound(world: World) -> tuple[int, int]:
    """Read the lowest and the highest weight the verb accepts.

    The verb refuses a weight past its bound, and the refusal names the bound.
    A test therefore reads the two numbers rather than writing them, so no test
    here holds a second copy of a value the balance register governs.[^1]

    References
    ----------
    [^1]: Recurring Defect Shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    outside = WEIGHT_OUTSIDE_EVERY_BOUND
    with pytest.raises(VerbError) as refusal:
        world.set_faction_weights(
            0, war=outside, trade=outside, build=outside, renown=outside, settle=outside
        )
    named = re.findall(r"\d+", str(refusal.value))
    assert len(named) == 2, "the refusal must name the two ends of the bound"
    low, high = (int(value) for value in named)
    assert low < high
    return low, high


def test_the_read_and_the_write_hold_one_shape() -> None:
    """The vector the reader returns is the vector the verb accepts."""
    # A weight the reader reports and the verb refuses, or the other way
    # about, is one value with two declaration sites. This call fails when
    # they part.
    world = seeded_world()
    world.set_faction_weights(0, **world.faction_weights(0))


def test_the_verb_writes_one_faction_and_leaves_the_other() -> None:
    """A write names one faction, and no other faction moves."""
    world = seeded_world()
    before = world.faction_weights(0)
    untouched = world.faction_weights(1)
    written = world.faction_weights(0)
    written["build"] = before["build"] % 2 + 1
    world.set_faction_weights(0, **written)
    assert world.faction_weights(0) == written
    assert world.faction_weights(1) == untouched


def test_the_verb_refuses_a_weight_outside_the_bound_and_changes_nothing() -> None:
    """A weight past the bound raises, and the verb writes nothing."""
    world = seeded_world()
    before = world.faction_weights(0)
    written = world.faction_weights(0)
    written["war"] = WEIGHT_OUTSIDE_EVERY_BOUND
    with pytest.raises(VerbError):
        world.set_faction_weights(0, **written)
    assert world.faction_weights(0) == before

    # The refusal names the bound, so the bound is read here rather than
    # written here.
    low, high = weight_bound(world)
    for value in before.values():
        # The seeding must draw every weight inside the bound the verb states.
        assert isinstance(value, int)
        assert low <= value <= high

    # Every value of the bound is accepted, and no value outside it is.
    for value in (low, high):
        world.set_faction_weights(
            0, war=value, trade=value, build=value, renown=value, settle=value
        )
    for value in (low - 1, high + 1):
        with pytest.raises(VerbError):
            world.set_faction_weights(
                0, war=value, trade=value, build=value, renown=value, settle=value
            )


def test_the_verb_refuses_a_faction_that_is_not_there() -> None:
    """A number that names no faction raises."""
    world = seeded_world()
    weights = world.faction_weights(0)
    with pytest.raises(VerbError):
        world.set_faction_weights(FACTIONS, **weights)


def test_a_weight_reaches_the_state_hash() -> None:
    """A written weight moves the hash at once, with no step between."""
    world = seeded_world()
    before = world.state_hash()
    weights = world.faction_weights(0)
    weights["build"] = weights["build"] % 2 + 1
    world.set_faction_weights(0, **weights)
    assert world.state_hash() != before


@pytest.mark.parametrize("threads", [1, 12])
def test_writing_back_the_weights_of_every_faction_changes_nothing(
    threads: int,
) -> None:
    """A world every caller wrote back matches a world nobody touched."""
    untouched = run(seeded_world(), EQUIVALENCE_STEPS, threads)
    written = seeded_world()
    write_back(written)
    assert run(written, EQUIVALENCE_STEPS, threads) == untouched


def test_the_write_back_test_can_fail() -> None:
    """One weight moved by one step parts the two runs."""
    # Without this, the test above would pass for a verb that wrote nothing at
    # all.
    untouched = run(seeded_world(), EQUIVALENCE_STEPS, 1)
    perturbed = seeded_world()
    weights = perturbed.faction_weights(0)
    weights["build"] = weights["build"] % 2 + 1
    perturbed.set_faction_weights(0, **weights)
    assert run(perturbed, EQUIVALENCE_STEPS, 1) != untouched


def test_the_thread_count_does_not_change_a_written_world() -> None:
    """A world whose weights a caller wrote runs alike at any thread count."""
    high = weight_bound(seeded_world())[1]
    results = []
    for threads in (1, 2, 12):
        world = seeded_world()
        set_every_weight(world, high)
        results.append(run(world, EQUIVALENCE_STEPS, threads))
    assert results[0] == results[1] == results[2]


def test_the_engine_reads_the_weights_a_caller_writes() -> None:
    """The step behaves differently for two worlds that differ only here."""
    # This drives the engine and reads what the engine did afterwards. A verb
    # that wrote a vector nothing reads would pass every test above and fail
    # this one.
    low, high = weight_bound(seeded_world())
    modest = seeded_world()
    set_every_weight(modest, low)
    eager = seeded_world()
    set_every_weight(eager, high)

    modest_log = run(modest, BEHAVIOUR_STEPS, 2)[1]
    eager_log = run(eager, BEHAVIOUR_STEPS, 2)[1]

    modest_census = modest.subsystem_census()
    eager_census = eager.subsystem_census()
    assert modest_census["controller_commands"] != eager_census["controller_commands"]
    assert modest_log != eager_log
