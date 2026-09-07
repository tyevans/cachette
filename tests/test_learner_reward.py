"""Tests of the reward one faction earns, computed in the control plane.

The engine holds no reward. What a faction should be rewarded for is a rule
of the downstream game, so the reward lives in Python.[^1]

Every test drives a real world. It seeds the world, steps it, and reads the
reward through the public interface. A test that fed the reward a table of
numbers would prove that the arithmetic works and not that anything reaches
the engine.[^2]

Every test states its own weighting. The module states none, because one
blocker holds the rules of the downstream game.[^3] A weight here is a
fixture value and never a proposal.

References
----------
[^1]: PRD-0056, a learner plays one faction against the controllers.
``docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md``

[^2]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``

[^3]: Blockers register, BLK-050. ``docs/BLOCKERS.md``

[^4]: Findings register, FND-580. ``docs/FINDINGS.md``

[^5]: Recurring Defect Shapes, shape 1, redundant declaration sites.
``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import pathlib
import re

import pytest

from cachette import World
from cachette.learn import (
    OUTCOMES,
    SHAPED_ROWS,
    TERMINAL_ROWS,
    UNSET_WEIGHTING,
    Reward,
    TermError,
    UnsetWeightError,
    Weighting,
)

# A world small enough to run to a game end inside a test, and wide enough
# that the seeding seats a faction in it.
EXTENT = 24

# A seed whose run reaches a game end quickly. The probe that chose it is in
# the commit body.
ENDING_SEED = 19

# The tick by which the run of that seed has ended. The probe measured the
# end at tick 161, and this bound leaves room.
ENDING_BOUND = 400

# A seed whose run does not end inside the bound above.
RUNNING_SEED = 0x00C0FFEE

FACTIONS = 2


def a_seeded_world(seed: int, factions: int = FACTIONS) -> World:
    """Build a seeded world, so a faction holds ground and units."""
    world = World(width=EXTENT, height=EXTENT, seed=seed, faction_count=factions)
    reports = world.seed_world()
    assert any(report["seated"] for report in reports), (
        "the fixture must seat at least one faction"
    )
    return world


def one_term(name: str, weight: float | None) -> Weighting:
    """Return a weighting that weighs one shaped term and no outcome."""
    return Weighting(terms={name: weight}, won=0.0, lost=0.0, drawn=0.0)


def test_a_weighting_that_leaves_a_weight_unset_refuses_to_run(seed: int) -> None:
    world = a_seeded_world(seed)
    with pytest.raises(UnsetWeightError) as caught:
        Reward(world, 0, UNSET_WEIGHTING)
    message = str(caught.value)
    for name in (*SHAPED_ROWS, *TERMINAL_ROWS):
        assert name in message, f"the refusal must name {name}"
    assert "BLK-050" in message


def test_the_unset_weighting_holds_one_row_for_each_reserved_term() -> None:
    assert set(UNSET_WEIGHTING.terms) == set(SHAPED_ROWS)
    assert UNSET_WEIGHTING.unset_names() == (*SHAPED_ROWS, *TERMINAL_ROWS)


def test_a_term_that_names_no_field_of_the_schema_is_refused(seed: int) -> None:
    world = a_seeded_world(seed)
    with pytest.raises(TermError):
        Reward(world, 0, one_term("no_such_field", 1.0))


def test_a_term_over_many_positions_is_refused(seed: int) -> None:
    # The relation field holds one position for each faction, so a weight on
    # it states no position.
    world = a_seeded_world(seed)
    with pytest.raises(TermError):
        Reward(world, 0, one_term("relation", 1.0))


def test_the_first_reading_pays_nothing(seed: int) -> None:
    world = a_seeded_world(seed)
    reward = Reward(world, 0, one_term("held_tiles", 1.0))
    step = reward.read(world)
    assert step.value == 0.0, "no step ran, so no term moved"
    assert step.outcome == "running"


def test_the_reward_moves_when_the_thing_it_rewards_moves(seed: int) -> None:
    # The faction takes ground as it settles, so the held tile term rises.
    world = a_seeded_world(seed)
    reward = Reward(world, 0, one_term("held_tiles", 1.0))
    for _ in range(3):
        world.step(1)
    step = reward.read(world)
    assert step.changes["held_tiles"] > 0, "the fixture must move the term"
    assert step.terms["held_tiles"] == float(step.changes["held_tiles"])
    assert step.shaped == step.terms["held_tiles"]
    assert step.value > 0.0


def test_a_term_with_a_zero_weight_contributes_nothing_while_it_still_moves(
    seed: int,
) -> None:
    # This is the test above with the weight put back to zero. It proves
    # that the assertion above measures the weight and not the fixture: the
    # term moves by the same amount, and the reward is zero.
    world = a_seeded_world(seed)
    reward = Reward(world, 0, one_term("held_tiles", 0.0))
    for _ in range(3):
        world.step(1)
    step = reward.read(world)
    assert step.changes["held_tiles"] > 0, "the fixture must still move the term"
    assert step.terms["held_tiles"] == 0.0
    assert step.value == 0.0


def test_the_weight_scales_the_contribution(seed: int) -> None:
    world = a_seeded_world(seed)
    single = Reward(world, 0, one_term("held_tiles", 1.0))
    doubled = Reward(world, 0, one_term("held_tiles", 2.0))
    for _ in range(3):
        world.step(1)
    one = single.read(world)
    two = doubled.read(world)
    assert one.value > 0.0
    assert two.value == pytest.approx(2.0 * one.value)


def test_a_shaped_reward_sums_the_terms_it_weighs(seed: int) -> None:
    world = a_seeded_world(seed)
    weighting = Weighting(
        terms={"held_tiles": 1.0, "live_units": 10.0},
        won=0.0,
        lost=0.0,
        drawn=0.0,
    )
    reward = Reward(world, 0, weighting)
    for _ in range(4):
        world.step(1)
    step = reward.read(world)
    assert step.shaped == step.terms["held_tiles"] + step.terms["live_units"]
    assert step.terms["live_units"] == 10.0 * float(step.changes["live_units"])


def test_a_weighting_with_no_shaped_weight_gives_the_terminal_reward() -> None:
    # A terminal reward needs no mode. It is a weighting whose shaped
    # weights are all zero.
    world = a_seeded_world(ENDING_SEED)
    weighting = Weighting(
        terms=dict.fromkeys(SHAPED_ROWS, 0.0),
        won=1.0,
        lost=-1.0,
        drawn=0.0,
    )
    winner = Reward(world, 0, weighting)
    loser = Reward(world, 1, weighting)
    values = {0: 0.0, 1: 0.0}
    for _ in range(ENDING_BOUND):
        world.step(1)
        values[0] += winner.read(world).value
        values[1] += loser.read(world).value
        if winner.done:
            break
    end = world.game_end()
    assert end is not None, "the fixture must reach a game end"
    assert winner.done and loser.done
    assert {winner.outcome, loser.outcome} == {"won", "lost"}
    assert values[end["winner"]] == 1.0
    assert values[1 - end["winner"]] == -1.0


def test_a_run_that_reaches_no_end_pays_no_terminal(seed: int) -> None:
    del seed
    world = a_seeded_world(RUNNING_SEED)
    weighting = Weighting(
        terms=dict.fromkeys(SHAPED_ROWS, 0.0),
        won=1.0,
        lost=-1.0,
        drawn=-0.5,
    )
    reward = Reward(world, 0, weighting)
    for _ in range(50):
        world.step(1)
        step = reward.read(world)
        assert step.outcome == "running"
        assert step.terminal == 0.0
    assert not reward.done


def test_the_tick_limit_draws_a_run_that_no_reader_ended() -> None:
    # A caller may turn the win readers off. The run then reaches the tick
    # limit with no winner, and the reward reports a draw.
    world = a_seeded_world(RUNNING_SEED)
    world.set_win_readers_enabled(False)
    world.set_tick_limit(5)
    weighting = Weighting(
        terms=dict.fromkeys(SHAPED_ROWS, 0.0),
        won=1.0,
        lost=-1.0,
        drawn=-0.5,
    )
    reward = Reward(world, 0, weighting)
    total = 0.0
    for _ in range(8):
        world.step(1)
        total += reward.read(world).value
    assert reward.outcome == "drawn"
    assert reward.done
    assert total == -0.5, "the terminal is paid once"


def test_the_terminal_is_paid_once() -> None:
    world = a_seeded_world(ENDING_SEED)
    weighting = Weighting(
        terms=dict.fromkeys(SHAPED_ROWS, 0.0),
        won=1.0,
        lost=-1.0,
        drawn=0.0,
    )
    reward = Reward(world, 0, weighting)
    for _ in range(ENDING_BOUND):
        world.step(1)
        if reward.read(world).done:
            break
    assert reward.done
    for _ in range(3):
        world.step(1)
        after = reward.read(world)
        assert after.value == 0.0
        assert after.done
        assert after.outcome == reward.outcome


def test_a_reset_starts_the_run_again(seed: int) -> None:
    world = a_seeded_world(seed)
    reward = Reward(world, 0, one_term("held_tiles", 1.0))
    for _ in range(3):
        world.step(1)
    first = reward.read(world)
    assert first.value > 0.0
    reward.reset(world)
    assert reward.read(world).value == 0.0, "a reset takes a new baseline"


def test_a_faction_that_holds_no_unit_and_no_person_is_not_alive() -> None:
    # The seeding of this world seats two of three factions. The third
    # holds nothing, so it cannot act.
    world = a_seeded_world(RUNNING_SEED, factions=3)
    weighting = Weighting(
        terms=dict.fromkeys(SHAPED_ROWS, 0.0),
        won=1.0,
        lost=-1.0,
        drawn=0.0,
    )
    seated = Reward(world, 0, weighting)
    unseated = Reward(world, 2, weighting)
    for _ in range(5):
        world.step(1)
    assert seated.read(world).alive
    step = unseated.read(world)
    assert not step.alive
    # A faction that cannot act has still not lost. Nothing terminal fires
    # until a reader ends the game.[^4]
    assert step.outcome == "running"
    assert step.terminal == 0.0


def test_every_outcome_the_module_names_has_a_row(seed: int) -> None:
    del seed
    assert OUTCOMES == ("running", *TERMINAL_ROWS)


def _register_rows(heading: str) -> list[str]:
    """Return the value column of one section of the reward register."""
    root = pathlib.Path(__file__).resolve().parent.parent
    text = (root / "docs" / "reference" / "rl-costs.md").read_text(encoding="utf-8")
    section = text.split(heading, 1)[1].split("\n## ", 1)[0]
    rows = []
    for line in section.splitlines():
        if not line.startswith("| "):
            continue
        first = line.split("|")[1].strip()
        match = re.fullmatch(r"`([a-z_]+)`.*", first)
        if match:
            rows.append(match.group(1))
    return rows


def test_the_register_holds_one_row_for_each_shaped_term() -> None:
    # The register and the module both name the terms. Nothing fails when
    # two declaration sites disagree, so this test is what fails.[^5]
    assert _register_rows("## The shaped terms") == list(SHAPED_ROWS)


def test_the_register_holds_one_row_for_each_terminal_outcome() -> None:
    assert _register_rows("## The terminal outcomes") == list(TERMINAL_ROWS)
