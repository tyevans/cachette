"""Several learner seats in one world, and the seat never decides the score.

A world holds three factions and the single-seat loop scores one of them, so
two thirds of every simulated game is discarded. Putting a candidate in more
than one seat recovers that, but only if the layout stops the seat itself from
deciding who ranks highest.

**The seats are not equivalent.** With the same built-in controller in all
three seats of twenty-four held-out worlds, seat 0 won 9, seat 1 won 4 and
seat 2 won 11. That is start-position luck with opponent quality held
constant, so a layout that put one candidate in seat 1 and another in seat 2
would rank the seats.

**The seat also decides the score across a whole generation, and the pairing
does not reach that.** The trainer ranks every candidate of a generation
together, so a pair that held the good seat for every seed of a generation
would rank above a pair that held the bad one. The seat therefore turns by one
position at each seed index, and a test below checks that each candidate
played every seat.

**A candidate is scored by its margin against the other seats of its own
world.** Two candidates in one game share the map, the weather and the
opponents, so the difference between their returns holds almost none of the
variance that either return holds on its own. The tests of that formula run
on made-up returns, because the claim is arithmetic and a world would only
hide it.

These tests check the layout first, because it is cheap and it is where the
error would be, and then drive a real world to check the parts that touch the
engine.
"""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import World
from cachette.learn import FACTION_SCOPED_READERS, Weighting
from cachette.learn.env import EnvConfig, viable_seeds
from cachette.learn.league import (
    SeatedGame,
    controller_seats,
    margins_of_world,
    run_seated_population,
    seat_counts,
    seat_matched_plan,
)
from cachette.learn.policy import LinearPolicy
from cachette.learn.train import TrainConfig, score_generation, train

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


def test_both_halves_of_a_pair_play_one_seat_on_one_seed() -> None:
    """The antithetic difference must cancel the seat, not carry it.

    This is the property the whole layout exists for. If the plus half of a
    pair played seat 2 and the minus half played seat 1, the difference
    between their scores would hold the seat advantage, and the update would
    follow it.
    """
    assignments, world_seed = seat_matched_plan(pairs=4, seeds=3, learner_seats=[0, 1])
    placed: dict[int, list[tuple[int, int]]] = {}
    for row in assignments:
        placed.setdefault(row.candidate, []).append((row.seat, world_seed[row.world]))

    for pair in range(4):
        plus = sorted(placed[2 * pair])
        minus = sorted(placed[2 * pair + 1])
        assert plus == minus, f"pair {pair} played different seats or seeds"


def test_every_candidate_is_scored_on_every_seed() -> None:
    """No candidate gets fewer worlds than another, which would bias the rank."""
    assignments, _ = seat_matched_plan(pairs=4, seeds=3, learner_seats=[0, 1])
    counts: dict[int, int] = {}
    for row in assignments:
        counts[row.candidate] = counts.get(row.candidate, 0) + 1
    assert set(counts) == set(range(8))
    assert len(set(counts.values())) == 1, counts


def test_one_world_never_seats_one_candidate_twice() -> None:
    """A candidate in two seats of one world would play against itself."""
    assignments, _ = seat_matched_plan(pairs=4, seeds=2, learner_seats=[0, 1])
    seen: dict[int, set[int]] = {}
    for row in assignments:
        occupants = seen.setdefault(row.world, set())
        assert row.candidate not in occupants
        occupants.add(row.candidate)


def test_two_learner_seats_halve_the_worlds_a_generation_needs() -> None:
    """The point of the layout is the sample cost, so measure it.

    One learner seat needs one world for each candidate and seed. Two learner
    seats put two candidates in each world, so the same generation needs half
    as many.
    """
    _, one = seat_matched_plan(pairs=4, seeds=3, learner_seats=[0])
    _, two = seat_matched_plan(pairs=4, seeds=3, learner_seats=[0, 1])
    assert len(one) == 8 * 3
    assert len(two) == len(one) // 2


def test_the_controller_keeps_a_seat() -> None:
    """A run that seats a candidate everywhere removes its own yardstick.

    In a game with one winner, every seat being a candidate also pins the win
    rate at one over the faction count, whatever any policy does.
    """
    assert controller_seats(WORLD, [0, 1]) == [2]
    assert controller_seats(WORLD, [0]) == [1, 2]
    assert controller_seats(WORLD, [0, 1, 2]) == []


def test_each_seat_reads_its_own_observation() -> None:
    """Two seats of one world do not see the same thing.

    A game that handed both seats one observation would train two candidates
    on one faction's view, and the fog rule would be broken without any reader
    outside the allowed set being called.
    """
    game = SeatedGame(WORLD, WEIGHTING, seats=[0, 1])
    game.reset(viable_seeds(WORLD, 1, 900)[0])
    observations = game.observations()
    assert observations.shape[0] == 2
    # The two seats read different arrays because they observe different
    # ground. **No position of the array names a seat**, so the test cannot
    # ask the array which seat read it, and a policy cannot learn a seat
    # number from it either.
    assert not np.array_equal(observations[0], observations[1])
    schema = game.world.observation_schema()
    names = {str(row["name"]) for row in schema["fields"]}
    assert "faction" not in names


class WatchedWorld:
    """A world that refuses any reader outside the faction-scoped set."""

    # The class declares both attributes, so a reader of ``seen`` gets the
    # set rather than whatever the fallback reader would give back. Only a
    # name that normal lookup misses reaches that reader.
    _world: World
    seen: set[str]

    def __init__(self, world: World) -> None:
        """Wrap one world."""
        self._world = world
        self.seen = set()

    def __getattr__(self, name: str) -> object:
        """Return the attribute, or refuse when it is not allowed."""
        if name not in FACTION_SCOPED_READERS:
            message = (
                f"the seated game called {name!r}, which answers from the "
                "truth of the whole world"
            )
            raise AssertionError(message)
        self.seen.add(name)
        return getattr(self._world, name)


def test_a_seated_game_reads_no_unfogged_reader() -> None:
    """Drive a whole seated episode through a world that watches the calls.

    The single-seat loop has this test already. A second seat path is a
    second place the rule can be broken, so it needs its own.
    """
    seed = viable_seeds(WORLD, 1, 900)[0]
    game = SeatedGame(WORLD, WEIGHTING, seats=[0, 1])
    real = game.reset(seed)
    watched = WatchedWorld(real)
    object.__setattr__(game, "_world", watched)

    for _ in range(4):
        if game.done:
            break
        game.observations()
        game.masks()
        game.apply([0, 0])
        for _ in range(WORLD.decision_interval):
            real.step(1)
        game.settle()

    assert watched.seen <= FACTION_SCOPED_READERS
    assert "faction_observation" in watched.seen


def test_the_watch_refuses_a_whole_world_reader() -> None:
    """The watch must be able to fail, or it proves nothing."""
    world = World(width=24, height=24, seed=900, faction_count=3)
    watched = WatchedWorld(world)
    with pytest.raises(AssertionError, match="truth of the whole world"):
        _ = watched.faction_units


def test_a_seated_population_scores_every_candidate() -> None:
    """Drive the real runner and check the shape and the ticks it reports.

    A layout test alone would not catch a runner that dropped a seat, so this
    goes through the engine.
    """
    seeds = viable_seeds(WORLD, 2, 900)
    # The schema states the lengths. **This test states none of its own**,
    # because a second declaration of a layout the engine already declares is
    # the defect shape this project names first, and the extent here is not
    # the extent the run uses.
    probe = SeatedGame(WORLD, WEIGHTING, seats=[0])
    world = probe.reset(seeds[0])
    actions = int(world.action_schema()["length"])
    features = int(world.observation_schema()["length"]) + 1
    rng = np.random.default_rng(0)
    candidates: Sequence[LinearPolicy] = [
        LinearPolicy(rng.standard_normal((actions, features)) * 0.5) for _ in range(4)
    ]
    result = run_seated_population(
        WORLD, WEIGHTING, candidates, seeds, learner_seats=[0, 1], workers=2
    )
    assert result.absolute.shape == (4, 2)
    assert result.relative.shape == (4, 2)
    # Every candidate played every seed, so no cell is untouched. A cell that
    # stayed at zero would mean a seat never reached its reward.
    assert np.count_nonzero(result.absolute) > 0
    assert result.ticks > 0
    # Two learner seats give two equal and opposite margins in each world, so
    # every column of the relative array sums to zero. A runner that scored a
    # candidate against a world it did not play would break this.
    assert result.relative.sum(axis=0) == pytest.approx(np.zeros(2), abs=1e-9)
    # The margin is not the return. A run where the two agreed would mean the
    # subtraction never happened.
    assert not np.allclose(result.relative, result.absolute)


def test_a_relative_run_refuses_one_learner_seat() -> None:
    """A margin with no opponent is an absolute return under another name."""
    with pytest.raises(ValueError, match="at least two learner seats"):
        run_seated_population(
            WORLD, WEIGHTING, [], seeds=[900], learner_seats=[0], workers=1
        )


def test_a_relative_run_refuses_a_population_that_leaves_a_world_half_empty() -> None:
    """A short group leaves one occupant in a world, and a margin needs two.

    The plan groups the pairs the width of the seat list, and a group short of
    a full width simply leaves the spare seats to the built-in controller. The
    absolute runner can live with that. A margin cannot, so the runner refuses
    the population rather than scoring one world against nothing.
    """
    rng = np.random.default_rng(0)
    candidates = [LinearPolicy(rng.standard_normal((2, 2))) for _ in range(6)]
    with pytest.raises(ValueError, match="multiple of 4"):
        run_seated_population(
            WORLD, WEIGHTING, candidates, seeds=[900], learner_seats=[0, 1], workers=1
        )


def test_a_relative_run_refuses_a_table_with_no_controller() -> None:
    """Seating a candidate everywhere removes the yardstick of the run."""
    with pytest.raises(ValueError, match=r"no\s+yardstick"):
        run_seated_population(
            WORLD, WEIGHTING, [], seeds=[900], learner_seats=[0, 1, 2], workers=1
        )


def test_each_candidate_plays_every_seat_across_a_generation() -> None:
    """The pairing cancels the seat inside a pair, and not between two pairs.

    The trainer ranks every candidate of a generation together. A pair that
    held one seat for the whole generation would carry that seat's advantage
    into that ranking, and no property of one pair would show it.
    """
    seats = [0, 1]
    assignments, _ = seat_matched_plan(pairs=4, seeds=4, learner_seats=seats)
    counted = seat_counts(assignments)
    assert set(counted) == set(range(8))
    for candidate, played in counted.items():
        assert set(played) == set(seats), f"candidate {candidate} played {played}"


def test_the_seat_rotation_is_balanced_when_the_seeds_divide_by_the_seats() -> None:
    """A candidate that played one seat more often still carries that seat."""
    assignments, _ = seat_matched_plan(pairs=6, seeds=4, learner_seats=[0, 1])
    for candidate, played in seat_counts(assignments).items():
        assert len(set(played.values())) == 1, f"candidate {candidate} played {played}"


def test_a_margin_is_a_return_minus_the_mean_of_the_other_seats() -> None:
    """State the formula on numbers, where nothing can hide it."""
    assert margins_of_world({0: 12.0, 1: 0.0, 2: 6.0}) == pytest.approx(
        {0: 9.0, 1: -9.0, 2: 0.0}
    )


def test_two_seats_give_equal_and_opposite_margins() -> None:
    """With two learner seats the population mean of the score is zero."""
    margins = margins_of_world({3: 10.0, 7: 4.0})
    assert margins == pytest.approx({3: 6.0, 7: -6.0})


def test_a_margin_removes_a_level_the_whole_world_shares() -> None:
    """This is the whole point of the change, so assert it directly.

    A hard map, a bad weather draw or a strong controller moves every return
    of one world by about the same amount. That level is the variance the
    absolute score carries and the margin does not.
    """
    plain = margins_of_world({0: 10.0, 1: 4.0, 2: -2.0})
    shifted = margins_of_world({0: 1010.0, 1: 1004.0, 2: 998.0})
    assert plain == pytest.approx(shifted)


def test_a_margin_needs_an_opponent() -> None:
    """One occupant has nothing to be relative to."""
    with pytest.raises(ValueError, match="at least two learner seats"):
        margins_of_world({0: 1.0})


def test_a_seated_generation_gives_one_answer_at_two_worker_counts() -> None:
    """A league is a new place for a completion order to reach the score.

    The plan fixes which candidate sits in which seat of which world, the
    batch reports its rows in world index order, and the accumulation runs
    over the world index. None of that depends on which worker finished
    first, so two worker counts must give the same array bit for bit.

    **The comparison must be able to fail.** A run compared against itself
    always passes, so the test also plays a different candidate set and
    requires a different answer.
    """
    seeds = viable_seeds(WORLD, 2, 900)
    probe = SeatedGame(WORLD, WEIGHTING, seats=[0])
    world = probe.reset(seeds[0])
    actions = int(world.action_schema()["length"])
    features = int(world.observation_schema()["length"]) + 1

    def population(draw: int) -> list[LinearPolicy]:
        # **A different scale is not a different population.** A linear policy
        # chooses by the highest score, and multiplying every weight by one
        # positive number scales every score by the same factor. The control
        # therefore draws different weights, not larger ones.
        #
        # Not every draw gives a different answer. Two random policies can
        # reach the same outcome on two seeds, so the control names a draw
        # that is known to differ rather than any draw.
        rng = np.random.default_rng(draw)
        return [
            LinearPolicy(rng.standard_normal((actions, features)) * 0.5)
            for _ in range(4)
        ]

    def play(candidates: Sequence[LinearPolicy], workers: int) -> np.ndarray:
        return run_seated_population(
            WORLD, WEIGHTING, candidates, seeds, learner_seats=[0, 1], workers=workers
        ).relative

    one = play(population(0), workers=1)
    four = play(population(0), workers=4)
    assert np.array_equal(one, four)
    assert not np.array_equal(one, play(population(1), workers=1))


def test_a_seated_generation_ranks_the_margin_and_reports_the_return() -> None:
    """Drive the trainer's own scoring pass, not the runner underneath it.

    The runner returns both instruments. Whether the update ranks the margin
    is a decision of the trainer, so the test starts at the trainer.
    """
    seeds = viable_seeds(WORLD, 2, 900)
    probe = SeatedGame(WORLD, WEIGHTING, seats=[0])
    world = probe.reset(seeds[0])
    actions = int(world.action_schema()["length"])
    features = int(world.observation_schema()["length"]) + 1
    rng = np.random.default_rng(0)
    candidates = [
        LinearPolicy(rng.standard_normal((actions, features)) * 0.5) for _ in range(4)
    ]
    config = TrainConfig(workers=2, learner_seats=(0, 1), relative=True)
    played = score_generation(WORLD, WEIGHTING, candidates, seeds, config)
    assert played.ranked.shape == (4,)
    assert float(played.ranked.sum()) == pytest.approx(0.0, abs=1e-9)
    assert not np.allclose(played.ranked, played.absolute)
    assert played.ticks > 0

    absolute = score_generation(
        WORLD, WEIGHTING, candidates, seeds, replace(config, relative=False)
    )
    assert np.allclose(absolute.ranked, absolute.absolute)


def test_a_league_run_reports_the_controller_yardstick_every_generation(
    tmp_path: Path,
) -> None:
    """A relative score cannot say whether the whole population improved.

    It is zero on average by construction, so a population that got worse
    together reads the same as one that got better together. The run must
    therefore carry an absolute measurement against the built-in controller,
    and it must carry it on every generation rather than on some of them.
    """
    pool = viable_seeds(WORLD, 2, 900)
    validation = viable_seeds(WORLD, 1, 4000)
    result = train(
        "league-probe",
        WORLD,
        WEIGHTING,
        TrainConfig(
            generations=2,
            population=4,
            seeds_per_generation=2,
            workers=2,
            learner_seats=(0, 1),
        ),
        tmp_path,
        pool,
        validation=validation,
        validate_every=1,
    )
    history = result["history"]
    assert len(history) == 2
    for row in history:
        assert row["yardstick"] is not None
        assert row["validation"] is not None
        assert row["above_controller"] is not None
        assert row["above_controller"] == pytest.approx(
            row["validation"] - row["yardstick"]
        )
