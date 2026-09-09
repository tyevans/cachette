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

[^6]: Findings register, FND-692. ``docs/FINDINGS.md``

[^7]: Findings register, FND-679. ``docs/FINDINGS.md``

[^8]: Findings register, FND-700. ``docs/FINDINGS.md``
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
    TIMING_ROWS,
    UNSET_WEIGHTING,
    Reward,
    RewardStep,
    TermError,
    UnsetWeightError,
    Weighting,
)
from cachette.learn.signals import SignalCatalogue

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


# The tick limit of a whole run below. The engine compares held ground at the
# limit and records a winner, so a run that reaches it still ends won or
# lost.
TIMED_LIMIT = 600

# Seeds of whole games under the tick limit above, chosen by a probe. The
# commit body holds the probe and every figure it read.
#
# Faction zero wins each of the first three, by domination at tick 90, by
# domination at tick 241, and by held ground at the tick limit. It loses the
# last two, by domination at tick 529 and by held ground at the limit.
EARLY_WIN_SEED = 4
LATER_WIN_SEED = 7
LIMIT_WIN_SEED = 3
EARLY_LOSS_SEED = 10
LIMIT_LOSS_SEED = 2

# What a win, a loss and the time left on the clock pay in the tests below.
# Each is a fixture value and never a proposal.[^3]
WIN_WEIGHT = 100.0
LOSS_WEIGHT = -10.0
EARLY_WEIGHT = 50.0


def a_timed_weighting(early: float) -> Weighting:
    """Return a weighting that pays the outcome and the time left, and nothing else."""
    return Weighting(
        terms={},
        won=WIN_WEIGHT,
        lost=LOSS_WEIGHT,
        drawn=0.0,
        won_early=early,
    )


def a_whole_run(seed: int, weighting: Weighting) -> RewardStep:
    """Run one whole game under the tick limit, and give back its last reading.

    The reading that ends the run is the one that pays, so this returns it
    rather than a total. Nothing after it pays anything.

    One step of the world runs one tick, and its argument is a thread count.
    This reads after every tick, so the reading of the end stands at the tick
    the game ended at.
    """
    world = a_seeded_world(seed)
    world.set_tick_limit(TIMED_LIMIT)
    reward = Reward(world, 0, weighting)
    step = reward.read(world)
    for _ in range(TIMED_LIMIT + 4):
        if step.done:
            break
        world.step(1)
        step = reward.read(world)
    assert step.done, "the fixture must reach the end of a game"
    return step


def test_two_wins_at_different_end_ticks_do_not_score_the_same() -> None:
    """A win with time left on the clock pays more than a win at the limit.

    A game that runs to the tick limit is a signal of indecisive play, and
    the three runs here end at three different ticks.[^6]
    """
    weighting = a_timed_weighting(EARLY_WEIGHT)
    early = a_whole_run(EARLY_WIN_SEED, weighting)
    later = a_whole_run(LATER_WIN_SEED, weighting)
    limit = a_whole_run(LIMIT_WIN_SEED, weighting)
    assert early.outcome == later.outcome == limit.outcome == "won"
    assert early.remaining_share > later.remaining_share > limit.remaining_share
    assert limit.remaining_share == 0.0, "a win at the limit leaves no clock"
    assert early.value > later.value > limit.value
    for step in (early, later, limit):
        assert step.early == pytest.approx(EARLY_WEIGHT * step.remaining_share)
        assert step.value == pytest.approx(WIN_WEIGHT + step.early)


def test_an_early_weight_of_zero_scores_a_win_as_the_win_weight_alone() -> None:
    """The test above with the weight put back to zero.

    This proves that the test above measures the weight and not the fixture.
    The three runs still end at three different ticks, and the three scores
    are equal. A stored score measured before this term existed therefore
    stays comparable with a score measured after it.[^6]
    """
    weighting = a_timed_weighting(0.0)
    early = a_whole_run(EARLY_WIN_SEED, weighting)
    later = a_whole_run(LATER_WIN_SEED, weighting)
    limit = a_whole_run(LIMIT_WIN_SEED, weighting)
    assert early.remaining_share > later.remaining_share > limit.remaining_share
    for step in (early, later, limit):
        assert step.early == 0.0
        assert step.value == WIN_WEIGHT


def test_a_loss_pays_nothing_for_the_time_it_left_on_the_clock() -> None:
    """A loss pays the loss weight alone, early or at the limit.

    A term that paid the time left on any outcome would pay a faction for
    losing quickly. A faction that gave up early would then outscore a
    faction that held on and lost at the limit.[^6]
    """
    weighting = a_timed_weighting(EARLY_WEIGHT)
    early = a_whole_run(EARLY_LOSS_SEED, weighting)
    limit = a_whole_run(LIMIT_LOSS_SEED, weighting)
    assert early.outcome == limit.outcome == "lost"
    assert early.remaining_share > limit.remaining_share
    assert early.early == limit.early == 0.0
    assert early.value == limit.value == LOSS_WEIGHT


def test_a_world_with_no_tick_limit_leaves_no_time_on_a_clock() -> None:
    """A world with no limit holds no clock, so a win in it pays no early term.

    The field the share comes from reads zero in such a world, and a zero
    there means the first tick rather than the last. A new world carries a
    tick limit of its own, so this test clears it.
    """
    world = a_seeded_world(ENDING_SEED)
    world.set_tick_limit(0)
    assert world.tick_limit == 0
    reward = Reward(world, 0, a_timed_weighting(EARLY_WEIGHT))
    step = reward.read(world)
    for _ in range(ENDING_BOUND):
        if step.done:
            break
        world.step(1)
        step = reward.read(world)
    assert step.done, "the fixture must reach a game end"
    assert step.outcome in ("won", "lost")
    assert step.remaining_share == 0.0
    assert step.early == 0.0


def test_the_register_holds_one_row_for_each_timing_term() -> None:
    assert _register_rows("## The terminal timing terms") == list(TIMING_ROWS)


def one_level(name: str, weight: float | None) -> Weighting:
    """Return a weighting that weighs the level of one field and no outcome."""
    return Weighting(levels={name: weight}, won=0.0, lost=0.0, drawn=0.0)


def a_published_unit(world: World, name: str) -> float:
    """Return the unit the engine published for one field of the observation.

    A test that wrote the unit here would hold a second copy of an engine
    rule, and nothing would fail when the engine moved and the test did
    not.[^5]
    """
    form = SignalCatalogue.of_world(world).signal(name).form
    assert form is not None, f"the schema must publish a form for {name}"
    return float(form.unit)


def a_trajectory(seed: int, weighting: Weighting, ticks: int) -> tuple[float, ...]:
    """Play one world and return what the reward paid on each decision.

    The caller sums the answer to get the undiscounted episode return, which
    is what an evolution strategy ranks a candidate by.
    """
    world = a_seeded_world(seed)
    reward = Reward(world, 0, weighting)
    paid: list[float] = []
    for _ in range(ticks):
        world.step(1)
        paid.append(reward.read(world).value)
    return tuple(paid)


def test_a_level_weight_reads_the_level_and_a_change_weight_reads_the_change(
    seed: int,
) -> None:
    """The two shaped forms read two quantities of one field.

    A level weight pays the bounded value of the field on every decision. A
    change weight pays the movement of the published value, and the two
    therefore pay different numbers on the same decision.
    """
    world = a_seeded_world(seed)
    level = Reward(world, 0, one_level("held_tiles", 1.0))
    change = Reward(world, 0, one_term("held_tiles", 1.0))
    for _ in range(3):
        world.step(1)
    from_level = level.read(world)
    from_change = change.read(world)

    assert from_level.shares["held_tiles"] > 0.0, "the fixture must hold ground"
    assert from_level.shares["held_tiles"] <= 1.0, "a level is bounded by its unit"
    assert from_level.levels["held_tiles"] == from_level.shares["held_tiles"]
    assert from_level.shaped == from_level.levels["held_tiles"]
    assert from_change.changes["held_tiles"] > 0
    assert from_level.value != from_change.value


def test_a_level_weight_scales_the_contribution(seed: int) -> None:
    world = a_seeded_world(seed)
    single = Reward(world, 0, one_level("held_tiles", 1.0))
    doubled = Reward(world, 0, one_level("held_tiles", 2.0))
    for _ in range(3):
        world.step(1)
    one = single.read(world)
    two = doubled.read(world)
    assert one.value > 0.0
    assert two.value == pytest.approx(2.0 * one.value)


def test_a_level_weight_of_zero_contributes_nothing_while_the_level_holds(
    seed: int,
) -> None:
    """The defect put back. The level is read, and the weight holds it at zero."""
    world = a_seeded_world(seed)
    reward = Reward(world, 0, one_level("held_tiles", 0.0))
    for _ in range(3):
        world.step(1)
    step = reward.read(world)
    assert step.shares["held_tiles"] > 0.0, "the fixture must still hold ground"
    assert step.levels["held_tiles"] == 0.0
    assert step.value == 0.0


def test_two_runs_of_one_endpoint_score_alike_under_a_change_weight() -> None:
    """The paired run: one seed, two run lengths, one pair of endpoints.

    An evolution strategy sums the reward of every decision with no discount.
    A sum of changes therefore collapses to the last reading less the first,
    so **a change weight scores a whole episode by its endpoints alone.**

    This plays one seed for a short run and for a run nearly four times as
    long. The field reaches the same value in both, so the two runs share
    their endpoints and differ only in how long they held that value. The
    change weight scores them identically, which is the defect. The level
    weight scores the longer run higher, because its sum is the area under
    the curve of the field.[^7]

    References
    ----------
    [^7]: Findings register, FND-679. ``docs/FINDINGS.md``
    """
    short = 40
    long_run = 150
    change_short = a_trajectory(RUNNING_SEED, one_term("held_tiles", 1.0), short)
    change_long = a_trajectory(RUNNING_SEED, one_term("held_tiles", 1.0), long_run)
    level_short = a_trajectory(RUNNING_SEED, one_level("held_tiles", 1.0), short)
    level_long = a_trajectory(RUNNING_SEED, one_level("held_tiles", 1.0), long_run)

    world = a_seeded_world(RUNNING_SEED)
    unit = a_published_unit(world, "held_tiles")
    reward = Reward(world, 0, one_level("held_tiles", 1.0))
    first = reward.read(world).shares["held_tiles"]
    for _ in range(short):
        world.step(1)
    ended_short = reward.read(world).shares["held_tiles"]
    for _ in range(long_run - short):
        world.step(1)
    ended_long = reward.read(world).shares["held_tiles"]

    assert ended_short > first, "the fixture must raise the field"
    assert ended_short == ended_long, (
        "the fixture must reach the same endpoint on both runs, or the two "
        "runs are not a pair"
    )

    assert sum(change_short) == pytest.approx((ended_short - first) * unit, abs=1.0)
    assert sum(change_long) == pytest.approx(sum(change_short)), (
        "a change weight scores two runs of one endpoint alike"
    )
    assert sum(level_long) > 2.0 * sum(level_short), (
        "a level weight must pay for the time the field held its level"
    )


def test_a_change_weight_pays_on_almost_no_decision_of_an_episode() -> None:
    """The density of the two shaped forms, counted decision by decision.

    A change weight pays only where the field moved. The field of this
    fixture moves once, when the seeding seats the faction, and holds after
    that. A change weight therefore pays one number for a whole episode, and
    an evolution strategy has nothing to climb between the first decision and
    the last.

    A level weight pays on every decision. This counts both, so the claim is
    a measurement of the fixture and not a reading of the arithmetic.[^8]

    References
    ----------
    [^8]: Findings register, FND-700. ``docs/FINDINGS.md``
    """
    ticks = 40
    changes = a_trajectory(RUNNING_SEED, one_term("held_tiles", 1.0), ticks)
    levels = a_trajectory(RUNNING_SEED, one_level("held_tiles", 1.0), ticks)

    silent = [index for index, paid in enumerate(changes) if paid == 0.0]
    assert len(silent) > ticks // 2, (
        "the fixture must hold the field still on most of its decisions"
    )
    assert all(levels[index] > 0.0 for index in silent), (
        "a level weight must pay while a change weight is silent"
    )
    assert all(paid > 0.0 for paid in levels)


def test_a_field_cannot_be_read_as_a_level_and_as_a_change_at_once() -> None:
    with pytest.raises(TermError, match="belongs in one entry"):
        Weighting(
            terms={"held_tiles": 1.0},
            levels={"held_tiles": 1.0},
            won=0.0,
            lost=0.0,
            drawn=0.0,
        )


def test_a_weighting_of_change_weights_alone_says_that_it_telescopes() -> None:
    """A weighting reports the defect, so a check can refuse it.

    Nothing failed while the whole strategy table held change weights alone,
    because a change weight is a legal weight and reads as dense. This
    property is what a check reads.[^8]

    References
    ----------
    [^8]: Findings register, FND-700. ``docs/FINDINGS.md``
    """
    assert one_term("held_tiles", 1.0).telescopes
    assert not one_level("held_tiles", 1.0).telescopes
    assert not Weighting(
        terms={"held_tiles": 1.0},
        levels={"settlements": 1.0},
        won=0.0,
        lost=0.0,
        drawn=0.0,
    ).telescopes
    assert not Weighting(terms={}, won=1.0, lost=-1.0, drawn=0.0).telescopes


def test_a_level_weight_that_names_no_field_of_the_schema_is_refused(
    seed: int,
) -> None:
    world = a_seeded_world(seed)
    with pytest.raises(TermError):
        Reward(world, 0, one_level("no_such_field", 1.0))


def test_a_level_weight_over_many_positions_is_refused(seed: int) -> None:
    world = a_seeded_world(seed)
    with pytest.raises(TermError):
        Reward(world, 0, one_level("relation", 1.0))


def test_a_finished_episode_pays_no_level(seed: int) -> None:
    """A run that has ended pays nothing, and the level rows still report.

    A level term is paid on every decision, so a reader of a finished episode
    must see zero rather than the last level again. The row set does not
    change, because a caller that reads a different set of names on the last
    row cannot combine the rows.
    """
    world = a_seeded_world(ENDING_SEED)
    weighting = Weighting(levels={"held_tiles": 1.0}, won=1.0, lost=-1.0, drawn=0.0)
    reward = Reward(world, 0, weighting)
    step = reward.read(world)
    for _ in range(ENDING_BOUND):
        if step.done:
            break
        world.step(1)
        step = reward.read(world)
    assert step.done, "the fixture must reach a game end"
    after = reward.read(world)
    assert after.value == 0.0
    assert after.levels == {"held_tiles": 0.0}
    assert after.shares == {"held_tiles": 0.0}
    del seed
