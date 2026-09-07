"""Several learner seats in one world, so one game teaches more than one thing.

A world holds three factions. The single-seat loop puts the learner in one of
them and gives the other two to the built-in controller, so one whole game
yields one number. **Two thirds of every simulated game is thrown away.**

This module puts a candidate in more than one seat of the same world. One game
then scores as many candidates as it has learner seats, and the sample cost of
a generation falls by that factor.

# The seats are not equivalent, so a naive version measures the seat

With the same built-in controller in all three seats of twenty-four held-out
worlds, the seats do not win equally often.[^1] The spread is start-position
luck with opponent quality held constant. A run that scored one candidate in
one seat and another candidate in another seat would rank the seats and call
it learning.

**A perturbation and its mirror therefore play the same seat on the same
seed.** The antithetic difference already cancels seed luck, and putting both
halves of a pair in one seat makes it cancel seat luck in the same way. Two
worlds of one seed host both halves of two pairs:

    world A, seed s:  seat 0 = pair 1 plus,   seat 1 = pair 2 plus
    world B, seed s:  seat 0 = pair 1 minus,  seat 1 = pair 2 minus

Pair 1 is scored twice in seat 0 and pair 2 twice in seat 1, both on seed s.

**The pairing cancels the seat and the seed. It does not cancel the
opponent.** The plus half of pair 1 meets the plus half of pair 2, and the
minus half meets the minus half, so the difference between the two scores of
a pair holds the other pair's perturbation as well as its own. That is a real
confound and it is not removed here. It is noise rather than bias, because
the other perturbation is drawn independently and its sign is as likely to
help as to hurt, so it averages out over pairs and over generations. It does
widen the variance of one generation.

A run that cannot accept that should use one learner seat, where the opponent
is the built-in controller in every world and does not move at all.

# One seat stays with the built-in controller

**The controller keeps a seat.** Filling every seat with candidates removes
the opponent the run is measured against, and in a three-player game with one
winner it also pins the win rate at one third whatever any policy does. A run
would then report a number that no policy could move.

# The fog rule is unchanged

Every seat reads its own observation and its own legality answer, through the
readers that answer for one faction.[^2] A seat learns nothing about another
seat that a player of it could not see.

# References

[^1]: Reinforcement learning parameters, the seat measurement.
``docs/reference/rl-costs.md``
[^2]: PRD-0001, a faction sees only what it observes.
``docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md``
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import TYPE_CHECKING

import numpy as np

from cachette._core import Batch, World

from .env import EnvConfig
from .reward import Reward, Weighting

if TYPE_CHECKING:
    from collections.abc import Sequence

    from .policy import Policy


@dataclass(frozen=True)
class SeatAssignment:
    """One candidate, the seat it plays, and the world it plays in.

    The candidate entry indexes the candidate list of a generation. The seat
    entry names the faction. The world entry indexes the world list, so two
    assignments that share a world index sit in one game.
    """

    candidate: int
    seat: int
    world: int


def seat_matched_plan(
    pairs: int, seeds: int, learner_seats: Sequence[int]
) -> tuple[list[SeatAssignment], list[int]]:
    """Lay out the worlds of one generation, and say which seed each takes.

    A candidate index is ``2 * pair`` for the plus half of a pair and
    ``2 * pair + 1`` for the minus half, which is the order the trainer
    builds them in.

    **Both halves of a pair get the same seat and the same seed.** The pairs
    are grouped so that as many as there are learner seats share a world, and
    the plus halves and the minus halves go to two worlds of the same seed.

    Returns the assignments and the seed index of each world.
    """
    if not learner_seats:
        message = "a seated plan needs at least one learner seat"
        raise ValueError(message)
    width = len(learner_seats)
    assignments: list[SeatAssignment] = []
    world_seed: list[int] = []
    # Pairs are taken in groups the width of the learner seat list. A group
    # that is short of a full width simply leaves the spare seats to the
    # built-in controller, so a population that does not divide evenly still
    # runs rather than refusing.
    for first in range(0, pairs, width):
        group = list(range(first, min(first + width, pairs)))
        for seed in range(seeds):
            for half in (0, 1):
                world = len(world_seed)
                world_seed.append(seed)
                for offset, pair in enumerate(group):
                    assignments.append(
                        SeatAssignment(
                            candidate=2 * pair + half,
                            seat=learner_seats[offset],
                            world=world,
                        )
                    )
    return assignments, world_seed


class SeatedGame:
    """One world, several learner seats, and one reward for each of them.

    The environment of a single seat holds one reward. This holds one for
    each learner seat, and it drives them together through one world.

    **The whole game ends at once.** A win is a public fact of the world, so
    every seat of one game finishes on the same decision, and each seat reads
    its own outcome from its own reward.
    """

    def __init__(
        self, config: EnvConfig, weighting: Weighting, seats: Sequence[int]
    ) -> None:
        """Build the game over one configuration and a list of learner seats."""
        self._config = config
        self._weighting = weighting
        self._seats = list(seats)
        self._world: World | None = None
        self._rewards: dict[int, Reward] = {}
        self._decisions = 0
        self._done = False

    @property
    def seats(self) -> list[int]:
        """The seats a candidate plays in this game."""
        return list(self._seats)

    @property
    def done(self) -> bool:
        """Whether the game has ended."""
        return self._done

    def reset(self, seed: int) -> World:
        """Build a world, take every learner seat, and return the world."""
        config = self._config
        world = World(
            width=config.width,
            height=config.height,
            seed=seed,
            faction_count=config.faction_count,
        )
        world.seed_world()
        world.set_win_readers_enabled(True)
        world.set_tick_limit(config.tick_limit)
        # Only the learner seats leave the built-in controller. Every other
        # seat keeps it, which is what holds the yardstick in the game.
        for seat in self._seats:
            world.set_externally_controlled(seat, True)
        self._world = world
        self._rewards = {
            seat: Reward(world, seat, self._weighting) for seat in self._seats
        }
        self._decisions = 0
        self._done = False
        return world

    @property
    def world(self) -> World:
        """The world of the game.

        **A caller that reads this holds the whole truth of the world.** The
        game itself calls only the readers that answer for one faction. This
        exists so a batch can hold the world, and for a test.

        A batch steps a world and a reward reads one, and neither takes the
        narrower door that one faction sees through. The game builds its own
        world and never adopts one, so it holds the world itself.
        """
        return self._require()

    def _require(self) -> World:
        """Return the world of the game, or refuse."""
        if self._world is None:
            message = "the game has no world. Call reset first."
            raise RuntimeError(message)
        return self._world

    def observations(self) -> np.ndarray:
        """Return one observation row for each learner seat, in seat order."""
        world = self._require()
        return np.stack([world.faction_observation(seat) for seat in self._seats])

    def masks(self) -> np.ndarray:
        """Return one legality row for each learner seat, in seat order."""
        world = self._require()
        return np.stack([world.legal_actions(seat) for seat in self._seats])

    def apply(self, actions: Sequence[int]) -> None:
        """Send one action for each learner seat, in seat order."""
        world = self._require()
        if self._done:
            return
        for seat, action in zip(self._seats, actions, strict=True):
            world.act(seat, int(action))

    def settle(self) -> dict[int, float]:
        """Read the reward of every seat, and return it by seat.

        The caller steps the world between ``apply`` and this. The game ends
        when any seat reads a terminal outcome, because the end of a game is
        one fact about the world rather than one fact about a seat.
        """
        world = self._require()
        self._decisions += 1
        earned: dict[int, float] = {}
        for seat, reward in self._rewards.items():
            reading = reward.read(world)
            earned[seat] = reading.value
            if reading.outcome in ("won", "lost", "drawn"):
                self._done = True
        if self._decisions >= self._config.horizon:
            self._done = True
        return earned

    def outcome(self, seat: int) -> str:
        """Name the state of the run for one seat."""
        reward = self._rewards.get(seat)
        return "running" if reward is None else reward.outcome


class SeatedVector:
    """Many seated games, stepped in one crossing of the boundary.

    The batch keeps the index order of the games, and a game whose episode
    has ended leaves the batch. **No result reaches the caller in the order a
    worker finished.**
    """

    def __init__(
        self,
        config: EnvConfig,
        weighting: Weighting,
        seats: Sequence[int],
        count: int,
        workers: int = 1,
    ) -> None:
        """Build a vector of seated games over one configuration."""
        if count < 1:
            message = "a vector holds at least one game"
            raise ValueError(message)
        self._config = config
        self._workers = max(1, workers)
        self._games = [SeatedGame(config, weighting, seats) for _ in range(count)]
        self._batch: Batch | None = None
        self._live: list[int] = []
        self.world_ticks = 0

    def __len__(self) -> int:
        """Return how many games the vector holds."""
        return len(self._games)

    @property
    def games(self) -> Sequence[SeatedGame]:
        """The games, in index order."""
        return self._games

    @property
    def done(self) -> bool:
        """Whether every game of the vector has ended."""
        return all(game.done for game in self._games)

    def reset(self, seeds: Sequence[int]) -> None:
        """Start one game in each slot, on the seed named for that slot."""
        if len(seeds) != len(self._games):
            message = (
                f"the vector holds {len(self._games)} games and {len(seeds)} seeds"
            )
            raise ValueError(message)
        worlds = [
            game.reset(int(seed)) for game, seed in zip(self._games, seeds, strict=True)
        ]
        self._live = list(range(len(self._games)))
        self._batch = Batch(worlds)
        self.world_ticks = 0

    def step(self, actions: Sequence[Sequence[int]]) -> list[dict[int, float]]:
        """Apply one action for every seat of every live game, and step them.

        The actions entry holds one row for each game, and each row holds one
        action for each learner seat, in seat order.
        """
        if self._batch is None:
            message = "the vector has no episode. Call reset first."
            raise RuntimeError(message)
        live = [index for index, game in enumerate(self._games) if not game.done]
        for index in live:
            self._games[index].apply(actions[index])

        if live != self._live:
            self._batch = Batch([self._games[index].world for index in live])
            self._live = live
        self.world_ticks += len(live) * self._config.decision_interval
        for _ in range(self._config.decision_interval):
            rows = self._batch.step(self._workers, self._config.threads)
            for row in rows:
                if row.error is not None:
                    failed = live[row.index]
                    message = f"the world at index {failed} refused: {row.error}"
                    raise RuntimeError(message)

        earned: list[dict[int, float]] = []
        for index, game in enumerate(self._games):
            earned.append(game.settle() if index in set(live) else {})
        return earned


def run_seated_population(
    config: EnvConfig,
    weighting: Weighting,
    candidates: Sequence[Policy],
    seeds: Sequence[int],
    learner_seats: Sequence[int],
    workers: int,
) -> tuple[np.ndarray, int]:
    """Score every candidate by playing them together, and return the returns.

    The result holds one row for each candidate and one column for each seed,
    in the shape the single-seat runner returns, so a trainer swaps one for
    the other without changing how it ranks.
    """
    pairs = len(candidates) // 2
    assignments, world_seed = seat_matched_plan(pairs, len(seeds), learner_seats)
    vector = SeatedVector(
        config, weighting, learner_seats, count=len(world_seed), workers=workers
    )
    vector.reset([seeds[index] for index in world_seed])

    # Which candidate sits in which seat of which world. The seat order of a
    # game is the order the game reports, so the lookup is by seat.
    by_world: dict[int, dict[int, int]] = {}
    for row in assignments:
        by_world.setdefault(row.world, {})[row.seat] = row.candidate

    returns = np.zeros((len(candidates), len(seeds)))
    while not vector.done:
        rows: list[list[int]] = []
        for index, game in enumerate(vector.games):
            if game.done:
                rows.append([0] * len(learner_seats))
                continue
            observations = game.observations()
            masks = game.masks()
            chosen: list[int] = []
            for offset, seat in enumerate(game.seats):
                candidate = by_world[index][seat]
                chosen.extend(
                    candidates[candidate].choose_many(
                        observations[offset : offset + 1], masks[offset : offset + 1]
                    )
                )
            rows.append(chosen)
        for index, earned in enumerate(vector.step(rows)):
            for seat, value in earned.items():
                returns[by_world[index][seat], world_seed[index]] += value
    return returns, vector.world_ticks


def controller_seats(config: EnvConfig, learner_seats: Sequence[int]) -> list[int]:
    """Return the seats the built-in controller keeps.

    A caller checks this before a run. **An empty answer means the run
    removed its own yardstick**, and in a game with one winner it also pins
    the win rate at one over the faction count.
    """
    return [
        seat for seat in range(config.faction_count) if seat not in set(learner_seats)
    ]


__all__ = [
    "SeatAssignment",
    "SeatedGame",
    "SeatedVector",
    "controller_seats",
    "replace",
    "run_seated_population",
    "seat_matched_plan",
]
