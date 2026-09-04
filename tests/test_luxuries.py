"""The control plane seeds luxuries and reads the variety score.

Every test here goes through the front door. It uses only names that the
package exports.

A luxury is a presence and not a quantity. A tile carries a luxury or it does
not, and no unit gathers one. The three gatherable kinds are a separate and
fixed catalogue, and nothing here touches them.

The caller names every placement in one call. A control plane that placed one
luxury for each call would be looping over tiles, which this project forbids.

Nothing in the engine reads the variety. It is a score for the control plane,
and no simulation pass consumes it.

References
----------
ADR-0040, Python is a control plane, not a data plane, decision D1.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

ADR-0001, one binary gives one answer at any thread count, decision D4.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``

Decisions register, DEC-200.
``docs/DECISIONS.md``

Testing rules, section 2a.
``.claude/rules/testing.md``
"""

from __future__ import annotations

import pytest

import cachette


def a_world(seed: int) -> cachette.World:
    """Build the world that every test reads."""
    return cachette.World(width=16, height=16, seed=seed, faction_count=2)


def test_a_bare_world_carries_no_luxury(seed: int) -> None:
    world = a_world(seed)
    assert world.world_variety == 0
    assert world.luxury_deposits == 0
    assert world.luxury_tile_count == 0
    assert world.luxuries_at(0) == 0
    assert world.variety_at(0) == 0


def test_the_caller_names_every_placement_in_one_call(seed: int) -> None:
    world = a_world(seed)
    world.seed_luxuries([(3, 0), (9, 1), (9, 2)])
    assert world.world_variety == 3
    assert world.luxury_deposits == 3
    assert world.luxury_tile_count == 2
    assert world.variety_at(9) == 2
    assert world.luxuries_at(9) == 0b110
    assert world.luxuries_at(3) == 0b1


def test_one_luxury_on_two_tiles_is_one_variety_and_two_deposits(seed: int) -> None:
    world = a_world(seed)
    world.seed_luxuries([(1, 5), (2, 5)])
    assert world.world_variety == 1
    assert world.luxury_deposits == 2


def test_two_worlds_of_different_luxuries_differ(seed: int) -> None:
    rich = a_world(seed)
    rich.seed_luxuries([(1, 0), (2, 1), (3, 2)])
    poor = a_world(seed)
    poor.seed_luxuries([(1, 0)])
    assert rich.world_variety != poor.world_variety
    assert rich.state_hash() != poor.state_hash()


def test_two_worlds_of_one_set_of_luxuries_agree(seed: int) -> None:
    placements = [(1, 0), (2, 1), (3, 2)]
    one = a_world(seed)
    one.seed_luxuries(placements)
    other = a_world(seed)
    other.seed_luxuries(list(reversed(placements)))
    assert one.world_variety == other.world_variety
    assert one.state_hash() == other.state_hash()


def test_the_world_takes_one_seed_only(seed: int) -> None:
    world = a_world(seed)
    world.seed_luxuries([(1, 0)])
    with pytest.raises(cachette.ConfigError):
        world.seed_luxuries([(2, 1)])
    assert world.world_variety == 1


def test_the_catalogue_refuses_a_luxury_above_its_ceiling(seed: int) -> None:
    ceiling = cachette.World.luxury_ceiling()
    assert ceiling == 64
    world = a_world(seed)
    before = world.state_hash()
    with pytest.raises(cachette.ConfigError):
        world.seed_luxuries([(1, ceiling)])
    assert world.state_hash() == before
    assert world.world_variety == 0
    # The luxury one below the ceiling is accepted, so the refusal is the
    # ceiling and not the whole range.
    world.seed_luxuries([(1, ceiling - 1)])
    assert world.world_variety == 1


def test_a_tile_outside_the_world_is_refused(seed: int) -> None:
    world = a_world(seed)
    with pytest.raises(cachette.ConfigError):
        world.seed_luxuries([(world.tile_count, 0)])
    assert world.world_variety == 0


def test_a_world_that_carries_the_whole_catalogue_reports_the_ceiling(
    seed: int,
) -> None:
    ceiling = cachette.World.luxury_ceiling()
    world = a_world(seed)
    world.seed_luxuries([(4, luxury) for luxury in range(ceiling)])
    assert world.world_variety == ceiling
    assert world.luxury_deposits == ceiling
    assert world.luxury_tile_count == 1
    assert world.luxuries_at(4) == (1 << ceiling) - 1


def test_a_cell_holds_the_luxuries_of_its_tiles(seed: int) -> None:
    world = a_world(seed)
    world.seed_luxuries([(0, 0), (1, 1), (2, 1)])
    # The world is smaller than one block, so one cell covers every tile.
    assert world.cell_variety(0) == 2
    with pytest.raises(cachette.ViewError):
        world.cell_variety(1 << 20)


def test_a_faction_that_holds_nothing_has_no_variety(seed: int) -> None:
    world = a_world(seed)
    world.seed_luxuries([(1, 0), (2, 1)])
    assert world.faction_variety(0) == 0
    assert world.world_variety == 2


def test_a_stepped_world_gives_one_answer_at_every_thread_count(seed: int) -> None:
    answers = []
    for threads in (1, 2, 12):
        world = a_world(seed)
        world.seed_luxuries([(1, 0), (1, 4), (7, 63)])
        for _ in range(4):
            world.step(threads=threads)
        assert world.check_invariants()
        answers.append((world.state_hash(), world.world_variety))
    assert answers[0] == answers[1]
    assert answers[0] == answers[2]
    assert answers[0][1] == 3
