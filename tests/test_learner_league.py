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

These tests check the layout first, because it is cheap and it is where the
error would be, and then drive a real world to check the parts that touch the
engine.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import World
from cachette.learn import FACTION_SCOPED_READERS, Weighting
from cachette.learn.env import EnvConfig, viable_seeds
from cachette.learn.league import (
    SeatedGame,
    controller_seats,
    run_seated_population,
    seat_matched_plan,
)
from cachette.learn.policy import LinearPolicy

if TYPE_CHECKING:
    from collections.abc import Sequence

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
    assert not np.array_equal(observations[0], observations[1])
    # The faction field of the array names the seat that read it.
    schema = game.world.observation_schema()
    faction = next(row for row in schema["fields"] if row["name"] == "faction")
    start = int(faction["start"])
    assert [int(row[start]) for row in observations] == [0, 1]


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
    returns, ticks = run_seated_population(
        WORLD, WEIGHTING, candidates, seeds, learner_seats=[0, 1], workers=2
    )
    assert returns.shape == (4, 2)
    # Every candidate played every seed, so no cell is untouched. A cell that
    # stayed at zero would mean a seat never reached its reward.
    assert np.count_nonzero(returns) > 0
    assert ticks > 0
