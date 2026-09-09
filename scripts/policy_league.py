"""Rate the stored policies by the games they play against each other.

Every policy this project holds was trained against the built-in controller
and scored under its own reward weighting. **No two of those scores
compare.** A policy that rewards people is scored in people, and a policy
that rewards ground is scored in ground, so a table of them ranks each
policy against its own bar and never against another policy. The project
owner played two of them by hand. He found that the ground policy beat the
people policy every time. The people policy stood further above its own bar.

This tool answers the other question. It seats the policies against each
other, it counts who won, and it fits one strength number for each of them
from the wins alone. The built-in controller takes a seat as one of the
players, so the scale is anchored to the opponent the project already
understands.

# The seats do not win equally often, so a naive round robin measures the seat

The balance register holds the seat share of the built-in controller over a
fixed seed set, and the seats do not win equally often in it.[^1] That spread
is start-position luck with opponent quality held constant. A player left in
one seat would carry that seat's luck into its rating, and nothing in the
rating would say so.

**The register measures a wider world than this league plays**, so the tool
also reports the seat share of its own games. Every player holds every seat
the same number of times, so that share is the seat luck of the league world
with the quality of the players averaged out. Read it as the size of the
effect the rotation cancels.

**Every player therefore plays every seat the same number of times.** A world
holds three factions, so a game seats three players. The schedule takes each
unordered triple of the players and plays it three times on one seed, turning
the triple by one seat each time. Player ``j`` of the triple takes seat
``(j + r) % 3`` on rotation ``r``, so over the three rotations each player of
the triple holds each seat once. The three rotations share one world, so the
turn is inside one map and one weather sequence.

The tool counts the seats of the finished schedule and refuses a schedule
that is not balanced. It counts them through the reader the league module
already holds for that job.[^2]

# The rating is a Luce fit, which is Bradley-Terry for more than two players

Bradley-Terry states the odds that one of two players wins. A game here has
three players and one winner, so the model is the Luce choice model, which
Bradley-Terry is the two-player case of. Each player holds a strength
``theta``, and the chance that player ``i`` wins a game over the set ``S``
is:

    exp(theta_i) / sum over j in S of exp(theta_j)

The fit maximises the log likelihood of the recorded winners, under a wide
Gaussian prior on ``theta``. **The prior does two jobs.** The likelihood is
unchanged when one constant is added to every strength, so without a prior
the fit has no unique answer. A player that wins no game at all also drives
its own strength to minus infinity, and the prior holds it finite. The prior
is wide, so it barely moves a player that won games.

The solver runs a fixed number of Newton steps and tests nothing for
convergence, so two runs of the tool give the same numbers.

The report states the strength on the Elo scale, which is the strength in
natural units times four hundred over the natural logarithm of ten. The
controller sits at zero.

# The error bars come from a bootstrap over worlds, not over games

The three rotations of one triple share a world. Their results are therefore
correlated, and an error bar that treated each game as independent would be
too narrow. **The bootstrap resamples worlds and not games.** One world
carries all three of its rotations into a resample, or none of them.

The report gives the error bar of each rating and the error bar of each
pairwise difference. The difference is the quantity that matters, and it is
free of the anchor. This measurement cannot order a pair whose difference does
not clear two of its own standard errors. The report names such a pair.

# The second measurement is where the games end

The engine holds four win paths: domination, a comparison of held ground at
the tick limit, a finished wonder, and a renown target. A measurement against
the built-in controller found that weaker play pushes games toward the limit.
The share of games a player takes to the limit is therefore a signal about
the quality of its play.

The tick of the end comes from the game end record through the one reader the
project holds for it.[^3] A report row once spelled it from a signal named
``tick``, no world publishes a signal of that name, and every episode the
project recorded read zero there.[^4]

**The three players of one game share its end tick and its win path**, because
the end of a game is one fact about the world. The share reported for a
player is therefore the share of the games that player sat in. It moves
because a player changes how its games end.

# What the tool reads, and what it never reads

The strategy of a stored policy comes from the checkpoint file beside its
weights, and the world it plays comes from the strategy table of the training
package. **No name of a policy and no name of a strategy is written here.**
The tool reads the checkpoint directory, and a checkpoint that names a
strategy the table does not hold is refused by name.

The win, the path and the end tick are public facts of the world. Nothing
here reads the reward of a seat, so the reward weighting a policy was trained
under decides nothing about its rating.

# References

[^1]: Budgets and costs, the seat share row of the balance table.
``docs/reference/balance.md``
[^2]: The seat counter of the league module.
``python/cachette/learn/league.py``
[^3]: ADR-0148, a game end is recorded once and stops the controllers,
decision D1.
``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``
[^4]: Findings register, FND-689. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import argparse
import importlib.util
import itertools
import json
import math
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from types import ModuleType
from typing import TYPE_CHECKING, Any, Protocol

import numpy as np

from cachette._core import Batch
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.league import SeatAssignment, SeatedGame, seat_counts
from cachette.learn.policy import PolicyFit, load_policy
from cachette.learn.record import end_tick_of
from cachette.learn.reward import Weighting

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Callable, Mapping, Sequence

    from cachette._core import GameEnd
    from cachette.learn.policy import Policy
    from cachette.learn.reward import Scoring

# Where the held-out seed search starts. Every other measurement of this
# project reads from the same start, so the tools share worlds.
HELDOUT_START = 50_000

# The name the report gives the built-in controller as a player. It is the
# anchor of the rating scale.
CONTROLLER = "controller"

# How the strength of a player in natural units reads on the Elo scale. A
# difference of this many Elo points is a difference of one natural unit,
# which is odds of e to one in a two-player game.
ELO_PER_UNIT = 400.0 / math.log(10.0)

# The width of the Gaussian prior on the strength, in natural units. It fixes
# the free constant of the likelihood and holds a player that won nothing at
# a finite strength. Four units is about seven hundred Elo, which is wider
# than any spread this measurement could find over a few hundred games.
PRIOR_WIDTH = 4.0

# How many Newton steps the fit takes. The count is fixed and nothing tests
# for convergence, so two runs of the tool give the same numbers. The problem
# is nine strengths over a few hundred games, and the step is quadratically
# convergent, so this is many more steps than the fit needs.
NEWTON_STEPS = 40

# How many resamples the error bars come from, and the seed the resampler
# draws on. Both are stated so that two runs give the same error bars.
BOOTSTRAP_DRAWS = 500
BOOTSTRAP_SEED = 20260908

# The seed the schedule shuffles the triple order with. The shuffle spreads
# the world seeds over the triples, so no player draws a systematically
# easier world set than another.
SCHEDULE_SEED = 11

# How many standard errors a pairwise difference must clear before the report
# calls the pair ordered. Two is the usual bar and it is about a 95 percent
# interval under a normal approximation.
CLEARANCE = 2.0

# The smallest error bar, in Elo, that the report reads as a measurement. A
# resample that never varied gives a standard deviation of float noise rather
# than of zero, and a difference divided by that noise clears any bar. **A
# smoke run of three games over one world reported two ordered pairs on an
# error bar of two parts in ten to the thirteenth.** Nothing failed, and the
# report read like a firm ordering.
ERROR_FLOOR = 1e-6

# How many worlds a bootstrap needs before it states an error at all. A
# resample of one world always draws that world, so every fit is the same fit
# and the spread is not an error bar.
LEAST_WORLDS = 2

# How many fits a standard deviation needs. The spread is taken with one
# degree of freedom removed, so a single fit gives no spread at all.
LEAST_FITS = 2


def _sibling(name: str) -> ModuleType:
    """Import a script that sits beside this one, because there is no package.

    The instrumental population script already holds the spread of a set of
    numbers and the count of the win paths of a set of games. **A second copy
    of either would be one rule stored twice**, with nothing that fails when
    the copies disagree, so this tool reads them from that script rather than
    restating them.
    """
    path = Path(__file__).resolve().parent / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        message = f"cannot import the script at {path}"
        raise ImportError(message)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


_instrumental = _sibling("instrumental_population")
Spread = _instrumental.Spread
path_counts = _instrumental.path_counts
NO_PATH = _instrumental.NO_PATH


class EndedWorld(Protocol):
    """What this module needs of a world in order to place its end.

    The tick limit and the clock are parameters the caller supplied. The end
    record is the public fact of how the game finished.
    """

    @property
    def tick(self) -> int:
        """Give back the tick the world stands at."""

    @property
    def tick_limit(self) -> int:
        """Give back the tick the limit fires at, or zero for no limit."""

    def game_end(self) -> GameEnd | None:
        """Give back how the game ended, or nothing while it runs."""


def reached_the_limit(world: EndedWorld, end_tick: int) -> bool:
    """Say whether a finished game ran as far as the tick limit.

    A world with no limit reaches none, so a limit of zero is never reached.
    A game the limit ended stands exactly at the limit, and a game a win
    reader ended before it stands short of it.

    **The instrumental population script holds this same rule inline.** A test
    asserts that the two sites agree over the cases that separate them. One of
    those cases is a game that ended exactly at the limit.
    """
    limit = int(world.tick_limit)
    return limit > 0 and end_tick >= limit


@dataclass(frozen=True)
class Player:
    """One thing that takes a seat, and how the engine drives that seat.

    A player with a policy takes the seat away from the built-in controller
    and chooses one action for each decision. **A player with no policy is
    the built-in controller itself.** Its seat is never taken, so the engine
    drives it. The tool sends no action for it at all.

    The strategy entry names the row of the strategy table the policy was
    trained under. It is empty for the controller, which no objective shaped.
    """

    name: str
    policy: Policy | None
    strategy: str = ""

    @property
    def seated(self) -> bool:
        """Whether this player takes its seat away from the controller."""
        return self.policy is not None


@dataclass(frozen=True)
class Seating:
    """One game: the world it plays, and which player holds each seat.

    The players entry holds one player index for each seat, in seat order.
    The world entry groups the games that share a map, so an error bar can
    resample worlds rather than games.
    """

    world: int
    seed: int
    rotation: int
    players: tuple[int, ...]

    def seat_of(self, player: int) -> int:
        """Return the seat one player holds in this game."""
        return self.players.index(player)


def schedule(
    players: int, seats: int, seeds: Sequence[int], per_triple: int
) -> list[Seating]:
    """Lay out every game the league plays, and fix the seat of each player.

    The schedule walks every unordered group of ``seats`` players. It takes
    them in the order the combinations reader gives, and then it shuffles that
    order on a stated seed. **The shuffle spreads the world seeds over the groups.**
    Without the shuffle a group of low-numbered players would always draw the
    first seeds of the list. A rating would then carry the difficulty of
    those worlds.

    Each group plays ``per_triple`` worlds. Each world hosts one rotation for
    each seat, so player ``j`` of the group takes seat ``(j + r) % seats`` on
    rotation ``r``. Over the rotations of one world each player of the group
    holds each seat once, and the whole group shares the map and the weather.

    The seed of a world is taken from the seed list in turn, so a run with
    fewer seeds than worlds reuses them evenly.

    Raises ``ValueError`` when the player count is short of the seat count,
    when the seat count is under two, or when no seed was given.
    """
    if seats < 2:
        message = f"a game seats at least two players, and this names {seats}"
        raise ValueError(message)
    if players < seats:
        message = (
            f"a league of {players} players cannot fill {seats} seats with "
            "distinct players"
        )
        raise ValueError(message)
    if not seeds:
        message = "a schedule needs at least one seed"
        raise ValueError(message)
    if per_triple < 1:
        message = "a group plays at least one world"
        raise ValueError(message)
    groups = [tuple(group) for group in itertools.combinations(range(players), seats)]
    order = np.random.default_rng(SCHEDULE_SEED).permutation(len(groups))
    laid: list[Seating] = []
    worlds = 0
    for position in order:
        group = groups[int(position)]
        for _ in range(per_triple):
            seed = seeds[worlds % len(seeds)]
            world = worlds
            worlds += 1
            for rotation in range(seats):
                seated = tuple(
                    group[(seat - rotation) % seats] for seat in range(seats)
                )
                laid.append(
                    Seating(
                        world=world,
                        seed=int(seed),
                        rotation=rotation,
                        players=seated,
                    )
                )
    return laid


def balance_of(seatings: Sequence[Seating]) -> dict[int, dict[int, int]]:
    """Count how many games each player played in each seat.

    The count comes from the seat counter of the league module, which already
    answers this question for a training generation. **A second counter here
    would be one rule stored twice.**
    """
    return seat_counts(
        [
            SeatAssignment(candidate=player, seat=seat, world=index)
            for index, seating in enumerate(seatings)
            for seat, player in enumerate(seating.players)
        ]
    )


def require_balance(seatings: Sequence[Seating], seats: int) -> None:
    """Refuse a schedule in which one player played one seat more than another.

    **This is the check the whole design turns on.** The seats of this world
    do not win equally often. A player that held one seat more often than
    another therefore carries that seat's luck into its rating. Nothing in the
    rating would say so, and no assertion on the ratings alone would find
    it.
    """
    counted = balance_of(seatings)
    for player, seats_of in sorted(counted.items()):
        held = [seats_of.get(seat, 0) for seat in range(seats)]
        if len(set(held)) != 1:
            message = (
                f"player {player} played the seats {held} times, and a rating "
                "over an unbalanced schedule measures the seat"
            )
            raise ValueError(message)


@dataclass(frozen=True)
class GameResult:
    """What one game of the schedule ended at.

    The winner entry names the player that won, and it is absent for a game
    that no win reader ended. The path entry names the reader that ended it,
    and it is the absent-path name for the same case.

    The end tick and the path are one fact about the world, so all three
    players of the game share them.
    """

    index: int
    world: int
    seed: int
    rotation: int
    players: tuple[int, ...]
    winner: int | None
    path: str
    end_tick: int
    reached_limit: bool
    decisions: int

    def as_dict(self) -> dict[str, Any]:
        """Return this result as plain values, for a report file."""
        return {
            "index": self.index,
            "world": self.world,
            "seed": self.seed,
            "rotation": self.rotation,
            "players": list(self.players),
            "winner": self.winner,
            "path": self.path,
            "end_tick": self.end_tick,
            "reached_limit": self.reached_limit,
            "decisions": self.decisions,
        }


def read_game(
    index: int, seating: Seating, game: SeatedGame, decisions: int
) -> GameResult:
    """Read the outcome of one finished game from the world it played.

    **The winner comes from the game end record and never from a reward.** A
    seat the built-in controller holds has no scorer. A rule that read the
    outcome of a seat would therefore answer for some seats of a game and not
    for others. The end record answers for every seat, and the win is a
    public fact of the world.
    """
    world = game.world
    end = world.game_end()
    end_tick = end_tick_of(world)
    winner_seat = None if end is None else int(end["winner"])
    return GameResult(
        index=index,
        world=seating.world,
        seed=seating.seed,
        rotation=seating.rotation,
        players=seating.players,
        winner=None if winner_seat is None else seating.players[winner_seat],
        path=NO_PATH if end is None else str(end["path"]),
        end_tick=end_tick,
        reached_limit=reached_the_limit(world, end_tick),
        decisions=decisions,
    )


def play_chunk(
    config: EnvConfig,
    scoring: Scoring,
    players: Sequence[Player],
    seatings: Sequence[Seating],
    first: int,
    workers: int,
) -> list[GameResult]:
    """Play a set of games together, and return the results in schedule order.

    Each game is a seated game of the league module, so the seats a policy
    takes leave the built-in controller and every other seat keeps it.[^1]

    **Nothing here reads which world finished first.** The batch reports its
    rows in world index order, the live list is rebuilt in index order, and
    the results are read in index order after the last decision.

    A game the batch reports an error for stops the whole chunk. A league
    with a missing game is unbalanced, and a rating over it measures the
    seat.

    References
    ----------
    [^1]: The seated game of the league module.
    ``python/cachette/learn/league.py``
    """
    games = [
        SeatedGame(
            config,
            scoring,
            [
                seat
                for seat, player in enumerate(seating.players)
                if players[player].seated
            ],
        )
        for seating in seatings
    ]
    worlds = [
        game.reset(seating.seed) for game, seating in zip(games, seatings, strict=True)
    ]
    batch = Batch(worlds, workers)
    live = list(range(len(games)))
    decisions = [0] * len(games)

    def running(index: int) -> bool:
        """Say whether one game still takes decisions."""
        return not games[index].done and games[index].world.game_end() is None

    while live:
        for index in live:
            game = games[index]
            observations = game.observations()
            masks = game.masks()
            chosen: list[int] = []
            for offset, seat in enumerate(game.seats):
                policy = players[seatings[index].players[seat]].policy
                if policy is None:
                    message = "a seated seat holds no policy"
                    raise RuntimeError(message)
                chosen.extend(
                    policy.choose_many(
                        observations[offset : offset + 1], masks[offset : offset + 1]
                    )
                )
            game.apply(chosen)
            decisions[index] += 1
        for _ in range(config.decision_interval):
            for row in batch.step(config.threads):
                if row.error is not None:
                    failed = live[row.index]
                    message = f"the world of game {first + failed} refused: {row.error}"
                    raise RuntimeError(message)
        for index in live:
            games[index].settle()
        now = [index for index in live if running(index)]
        if now != live:
            live = now
            if live:
                batch = Batch([games[index].world for index in live], workers)
    return [
        read_game(first + offset, seating, game, decisions[offset])
        for offset, (seating, game) in enumerate(zip(seatings, games, strict=True))
    ]


def play_league(
    config: EnvConfig,
    scoring: Scoring,
    players: Sequence[Player],
    seatings: Sequence[Seating],
    workers: int,
    say: Callable[[int, int], None] | None = None,
) -> list[GameResult]:
    """Play every game of the schedule, in chunks of the worker width.

    The chunks are cut in schedule order, and the results are joined in the
    same order. The chunk width therefore changes the wall clock and nothing
    else.
    """
    results: list[GameResult] = []
    for first in range(0, len(seatings), workers):
        chunk = seatings[first : first + workers]
        results.extend(play_chunk(config, scoring, players, chunk, first, workers))
        if say is not None:
            say(len(results), len(seatings))
    return results


def win_table(
    results: Sequence[GameResult],
) -> tuple[np.ndarray, np.ndarray]:
    """Return the members of each rated game and the player that won it.

    A game that no win reader ended is left out, because it states nothing
    about which player is stronger. The caller reports how many were left
    out.

    The members array holds one row for each rated game, and each row holds
    the player index of each seat in seat order.
    """
    rated = [row for row in results if row.winner is not None]
    members = np.array([list(row.players) for row in rated], dtype=np.intp).reshape(
        len(rated), -1
    )
    winners = np.array([row.winner for row in rated], dtype=np.intp)
    return members, winners


def choice_matrix(members: np.ndarray, strengths: np.ndarray) -> np.ndarray:
    """Return the win chance of each player in each game, as a wide array.

    The array holds one row for each game and one column for each player. A
    player that did not play a game holds zero in that row. A sum down a
    column is therefore the expected win count of that player.

    The strengths are shifted by their own highest value before the
    exponential. The shift changes no chance, and it keeps the exponential
    inside the range of a float.
    """
    games = members.shape[0]
    weights = np.exp(strengths - strengths.max())
    seated = weights[members]
    chances = seated / seated.sum(axis=1, keepdims=True)
    wide = np.zeros((games, len(strengths)))
    np.put_along_axis(wide, members, chances, axis=1)
    return wide


def fit_strengths(
    members: np.ndarray, winners: np.ndarray, players: int
) -> tuple[np.ndarray, np.ndarray]:
    """Fit one strength for each player, and return it with its covariance.

    The model is the Luce choice model, which Bradley-Terry is the
    two-player case of. The fit maximises the log likelihood of the recorded
    winners plus a wide Gaussian prior on the strengths.

    **The prior fixes the free constant of the likelihood.** Adding one
    constant to every strength leaves every win chance where it was, so the
    likelihood alone has no unique answer. The prior also holds a player that
    won no game at a finite strength.

    The solver takes a fixed number of Newton steps and tests nothing for
    convergence. The information matrix of this model is the difference
    between two terms. The first is the diagonal of the expected win counts,
    and the second is the outer product of the per-game chances. The prior
    adds a constant to the diagonal of it.

    Returns the strengths in natural units, and the covariance of them.
    """
    wins = np.bincount(winners, minlength=players).astype(float)
    strengths = np.zeros(players)
    ridge = np.eye(players) / (PRIOR_WIDTH * PRIOR_WIDTH)
    information = ridge
    for _ in range(NEWTON_STEPS):
        wide = choice_matrix(members, strengths)
        expected = wide.sum(axis=0)
        gradient = wins - expected - strengths / (PRIOR_WIDTH * PRIOR_WIDTH)
        information = np.diag(expected) - wide.T @ wide + ridge
        strengths = strengths + np.linalg.solve(information, gradient)
    return strengths, np.linalg.inv(information)


def bootstrap_strengths(
    results: Sequence[GameResult], players: int, draws: int
) -> np.ndarray:
    """Refit the strengths over resampled worlds, and return every fit.

    **The resample draws worlds and never games.** The rotations of one world
    share the map and the weather, so their results are correlated. An error
    bar that treated each game as independent would be too narrow.

    The returned array holds one row for each draw and one column for each
    player, in natural units. A draw that leaves a player with no game still
    fits, because the prior holds that player at a finite strength.

    The resampler draws on a stated seed, so two runs give the same error
    bars.

    Raises ``ValueError`` for a league of fewer than two worlds. Every
    resample of one world draws that world, so every fit is the same fit and
    the spread of them is not an error bar.

    **A resample in which no game recorded a winner is dropped.** Such a
    resample states nothing. A row of zeros left in its place would read as a
    fit in which every player is equal, and it would narrow every error bar
    without anything failing.
    """
    by_world: dict[int, list[GameResult]] = {}
    for row in results:
        by_world.setdefault(row.world, []).append(row)
    worlds = sorted(by_world)
    if len(worlds) < LEAST_WORLDS:
        message = (
            f"a bootstrap over {len(worlds)} world states no error, because "
            "every resample draws the same world. Play at least "
            f"{LEAST_WORLDS} worlds."
        )
        raise ValueError(message)
    generator = np.random.default_rng(BOOTSTRAP_SEED)
    fitted: list[np.ndarray] = []
    for _draw in range(draws):
        picked = generator.integers(0, len(worlds), size=len(worlds))
        taken = [row for index in picked for row in by_world[worlds[int(index)]]]
        members, winners = win_table(taken)
        if len(winners) == 0:
            continue
        strengths, _covariance = fit_strengths(members, winners, players)
        fitted.append(strengths)
    if len(fitted) < LEAST_FITS:
        message = (
            f"only {len(fitted)} of {draws} resamples recorded a winner, and "
            f"a spread needs {LEAST_FITS} fits"
        )
        raise ValueError(message)
    return np.stack(fitted)


@dataclass(frozen=True)
class Rating:
    """The rating of one player, and how firm it is.

    The elo entry is the strength on the Elo scale, with the anchor player at
    zero. The error entry is the standard deviation of the same quantity over
    the world bootstrap, so it already carries the anchor.

    The games entry counts the rated games the player sat in, and the wins
    entry counts the ones it won. A game that no win reader ended is in
    neither.
    """

    player: str
    elo: float
    error: float
    games: int
    wins: int

    @property
    def win_share(self) -> float:
        """The share of its rated games the player won."""
        if self.games == 0:
            return 0.0
        return self.wins / self.games

    def as_dict(self) -> dict[str, Any]:
        """Return this rating as plain values, for a report file."""
        return {
            "player": self.player,
            "elo": self.elo,
            "error": self.error,
            "games": self.games,
            "wins": self.wins,
            "win_share": self.win_share,
        }


def anchored(strengths: np.ndarray, anchor: int) -> np.ndarray:
    """Return the strengths on the Elo scale, with the anchor player at zero.

    The likelihood is unchanged when one constant is added to every strength,
    so the level of the scale is a choice and not a measurement. The anchor
    is the built-in controller, which is the opponent every policy of this
    project was trained against.
    """
    return (strengths - strengths[..., anchor, None]) * ELO_PER_UNIT


def rate(
    results: Sequence[GameResult],
    players: Sequence[Player],
    anchor: int,
    draws: int = BOOTSTRAP_DRAWS,
) -> tuple[list[Rating], np.ndarray, np.ndarray]:
    """Rate every player, and return the ratings with the bootstrap fits.

    The second return is the bootstrap fits on the Elo scale after the
    anchor, and the third is the raw bootstrap fits in natural units. A
    caller reads the second for the error of one rating and the third for the
    error of a difference, which is free of the anchor.
    """
    count = len(players)
    members, winners = win_table(results)
    if len(winners) == 0:
        message = "no game of the league recorded a winner, so nothing rates"
        raise ValueError(message)
    strengths, _ = fit_strengths(members, winners, count)
    raw = bootstrap_strengths(results, count, draws)
    scale = anchored(strengths, anchor)
    spread = anchored(raw, anchor)
    played = np.bincount(members.ravel(), minlength=count)
    won = np.bincount(winners, minlength=count)
    ratings = [
        Rating(
            player=players[index].name,
            elo=float(scale[index]),
            error=float(spread[:, index].std(ddof=1)),
            games=int(played[index]),
            wins=int(won[index]),
        )
        for index in range(count)
    ]
    return ratings, spread, raw


def difference_errors(raw: np.ndarray) -> np.ndarray:
    """Return the standard error of every pairwise Elo difference.

    **A difference is free of the anchor**, so it is the quantity to test. The
    error comes from the same world bootstrap. It is taken on the difference
    itself rather than built from two separate errors, so it carries the
    correlation between the two fits.
    """
    players = raw.shape[1]
    errors = np.zeros((players, players))
    for one in range(players):
        for two in range(players):
            if one == two:
                continue
            gap = (raw[:, one] - raw[:, two]) * ELO_PER_UNIT
            errors[one, two] = gap.std(ddof=1)
    return errors


def ordered_pairs(
    ratings: Sequence[Rating], errors: np.ndarray, clearance: float = CLEARANCE
) -> list[tuple[str, str, float, float]]:
    """Return the pairs whose Elo difference clears its own error bar.

    Each entry names the stronger player, the weaker player, the difference
    and the error of it. **An empty answer means the measurement ordered no
    pair**, and a report of it must say so rather than print a ranking.

    A pair whose error bar is under the error floor is left out. A resample
    that never varied gives a standard deviation of float noise, and a
    difference divided by that noise clears any bar. That is not a
    measurement, and it reads like one.
    """
    found: list[tuple[str, str, float, float]] = []
    for one in range(len(ratings)):
        for two in range(one + 1, len(ratings)):
            gap = ratings[one].elo - ratings[two].elo
            error = float(errors[one, two])
            if error < ERROR_FLOOR or abs(gap) < clearance * error:
                continue
            high, low = (one, two) if gap > 0 else (two, one)
            found.append((ratings[high].player, ratings[low].player, abs(gap), error))
    return sorted(found, key=lambda row: -row[2])


def seat_wins(results: Sequence[GameResult]) -> dict[int, int]:
    """Count how many rated games each seat won, in seat order.

    **This is the direct measurement of the seat luck of the league world.**
    Every player holds every seat the same number of times, so the quality of
    the players averages out of this count and the seat alone is left.

    A count that is far from even says the rotation earned its cost. A count
    that is even says the seats of this world are fair, and the rotation cost
    nothing.
    """
    counted: dict[int, int] = {}
    for row in results:
        if row.winner is None:
            continue
        seat = row.players.index(row.winner)
        counted[seat] = counted.get(seat, 0) + 1
    return {seat: counted.get(seat, 0) for seat in sorted(counted)}


def head_to_head(
    results: Sequence[GameResult], players: int
) -> tuple[np.ndarray, np.ndarray]:
    """Return the wins of each player against each other, and the game counts.

    A game seats three players and one of them wins, so a pairwise reading of
    it needs a stated rule. **The rule counts only the games one of the two
    players won.** The share is the games the row player won, out of that
    count. A game the third player won states nothing about which of the two
    is stronger. It is therefore out of the denominator and in the count of
    shared games.

    Returns the win count of the row player against the column player, and
    the number of games the pair both sat in.
    """
    wins = np.zeros((players, players))
    together = np.zeros((players, players))
    for row in results:
        if row.winner is None:
            continue
        for one in row.players:
            for two in row.players:
                if one == two:
                    continue
                together[one, two] += 1
                if row.winner == one:
                    wins[one, two] += 1
    return wins, together


@dataclass(frozen=True)
class Ending:
    """Where the games of one player ended.

    The limit entry is the share of the player's games that ran as far as the
    tick limit. The paths entry counts the win path of each of those games,
    and the won paths entry counts only the games the player itself won.

    **The three players of one game share its end tick and its win path.** The
    share reported here is therefore over the games the player sat in, and it
    moves because a player changes how its games end.
    """

    player: str
    games: int
    reached_limit: int
    end_tick: Any
    paths: Mapping[str, int]
    won_paths: Mapping[str, int]

    @property
    def limit_share(self) -> float:
        """The share of the player's games that reached the tick limit."""
        if self.games == 0:
            return 0.0
        return self.reached_limit / self.games

    def as_dict(self) -> dict[str, Any]:
        """Return this ending as plain values, for a report file."""
        return {
            "player": self.player,
            "games": self.games,
            "reached_limit": self.reached_limit,
            "limit_share": self.limit_share,
            "end_tick": self.end_tick.as_dict(),
            "paths": dict(self.paths),
            "won_paths": dict(self.won_paths),
        }


def endings(results: Sequence[GameResult], players: Sequence[Player]) -> list[Ending]:
    """Return where the games of each player ended, in player order.

    The spread of the end ticks and the count of the win paths come from the
    instrumental population script, which already holds both.
    """
    found: list[Ending] = []
    for index, player in enumerate(players):
        played = [row for row in results if index in row.players]
        if not played:
            message = f"player {player.name} played no game"
            raise ValueError(message)
        found.append(
            Ending(
                player=player.name,
                games=len(played),
                reached_limit=sum(1 for row in played if row.reached_limit),
                end_tick=Spread.of([float(row.end_tick) for row in played]),
                paths=path_counts(played),
                won_paths=path_counts([row for row in played if row.winner == index]),
            )
        )
    return found


def stored_players(
    directory: Path,
    strategies: Mapping[str, tuple[EnvConfig, Any, str]],
    fit: PolicyFit,
) -> list[Player]:
    """Read every checkpoint of a directory, and return one player for each.

    A checkpoint states the strategy it was trained under, so **no name of a
    policy and no name of a strategy is written in this tool.** A checkpoint
    that names a strategy the table does not hold is refused with the list of
    the rows the table does hold.

    The players come back in the order of the strategy name, so two runs over
    one directory hold the players in one order.

    **The whole directory is read before the first weight file is opened.** A
    refusal therefore costs nothing, and it names the checkpoint rather than
    the archive inside it.

    Raises ``ValueError`` when a checkpoint holds no strategy, when it names
    an unknown one, or when two checkpoints name the same one.
    """
    weights: dict[str, Path] = {}
    for path in sorted(directory.glob("*.json")):
        meta = json.loads(path.read_text(encoding="utf-8"))
        strategy = meta.get("strategy")
        if not strategy:
            message = f"{path} names no strategy"
            raise ValueError(message)
        if strategy not in strategies:
            message = (
                f"{strategy!r} of {path} names no strategy. The table holds "
                f"{sorted(strategies)}."
            )
            raise ValueError(message)
        if strategy in weights:
            message = f"two checkpoints name the strategy {strategy!r}"
            raise ValueError(message)
        weights[strategy] = path.with_name(str(meta.get("file") or f"{path.stem}.npz"))
    return [
        Player(
            name=name,
            policy=load_policy(weights[name], fit)[0],
            strategy=name,
        )
        for name in sorted(weights)
    ]


def probe_scoring() -> Weighting:
    """Return a scoring that pays for nothing, for a run that reads no reward.

    The rating reads the game end record and never a reward, so the scoring
    only has to be one the seated game accepts. A weighting with no term and
    no outcome weight scores every reading at zero. The outcome of a seat
    still reads correctly, because the outcome comes from a reader of the
    world and not from the weights.
    """
    return Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0)


def one_world(strategies: Mapping[str, tuple[EnvConfig, Any, str]]) -> EnvConfig:
    """Return the one world every strategy plays, and refuse a table of two.

    Every stored policy of this measurement was trained on one world, and a
    rating over two different worlds would compare games nobody can compare.
    **The world comes from the strategy table** and never from the command
    line.
    """
    held = {
        (
            config.width,
            config.height,
            config.faction_count,
            config.tick_limit,
            config.horizon,
            config.decision_interval,
        ): config
        for config, _scoring, _kind in strategies.values()
    }
    if len(held) != 1:
        message = (
            "the strategies play more than one world, and a rating over two "
            f"worlds compares nothing: {sorted(held)}"
        )
        raise ValueError(message)
    return next(iter(held.values()))


def _say_ratings(ratings: Sequence[Rating], clearance: float) -> None:
    """Print the rating of each player, strongest first."""
    print(
        f"  {'player':>22s} {'games':>6s} {'wins':>6s} {'share':>7s} "
        f"{'elo':>9s} {'error':>7s} {'interval':>18s}"
    )
    for row in sorted(ratings, key=lambda item: -item.elo):
        low = row.elo - clearance * row.error
        high = row.elo + clearance * row.error
        print(
            f"  {row.player:>22s} {row.games:6d} {row.wins:6d} "
            f"{row.win_share:7.3f} {row.elo:9.1f} {row.error:7.1f} "
            f"{low:8.1f} to {high:7.1f}"
        )


def _say_endings(found: Sequence[Ending]) -> None:
    """Print the tick limit share and the end tick spread of each player."""
    print(
        f"  {'player':>22s} {'games':>6s} {'at limit':>9s} {'share':>7s} "
        f"{'lowest':>8s} {'quarter':>8s} {'middle':>8s} {'quarter':>8s} "
        f"{'highest':>8s}"
    )
    for row in sorted(found, key=lambda item: -item.limit_share):
        spread = row.end_tick
        print(
            f"  {row.player:>22s} {row.games:6d} {row.reached_limit:9d} "
            f"{row.limit_share:7.3f} {spread.lowest:8.0f} "
            f"{spread.lower_quarter:8.0f} {spread.middle:8.0f} "
            f"{spread.upper_quarter:8.0f} {spread.highest:8.0f}"
        )


def _say_paths(found: Sequence[Ending]) -> None:
    """Print the win path of the games each player sat in, and of its wins."""
    names = sorted({name for row in found for name in row.paths})
    won = sorted({name for row in found for name in row.won_paths})
    print("  the path that ended the games each player sat in")
    print(f"  {'player':>22s} " + " ".join(f"{name:>12s}" for name in names))
    for row in found:
        cells = " ".join(f"{row.paths.get(name, 0):12d}" for name in names)
        print(f"  {row.player:>22s} " + cells)
    print("\n  the path each player won by")
    print(f"  {'player':>22s} " + " ".join(f"{name:>12s}" for name in won))
    for row in found:
        cells = " ".join(f"{row.won_paths.get(name, 0):12d}" for name in won)
        print(f"  {row.player:>22s} " + cells)


def _say_head_to_head(
    players: Sequence[Player], wins: np.ndarray, together: np.ndarray
) -> None:
    """Print the share of the decided games of each pair that the row won.

    A cell holds the share of the decided games of the pair that the row
    player won. A game is decided when one of the two won it. A game the
    third player won is in neither part of the share.
    """
    names = [player.name for player in players]
    print(f"  {'':>22s} " + " ".join(f"{name[:11]:>11s}" for name in names))
    for one, name in enumerate(names):
        cells: list[str] = []
        for two in range(len(names)):
            if one == two:
                cells.append(f"{'-':>11s}")
                continue
            decided = wins[one, two] + wins[two, one]
            if decided == 0:
                cells.append(f"{'none':>11s}")
                continue
            cells.append(f"{wins[one, two] / decided:11.3f}")
        print(f"  {name:>22s} " + " ".join(cells))
    shared = together[together > 0]
    if shared.size:
        print(
            f"\n  every pair sat in {int(shared.max())} games together at "
            f"most, and {int(shared.min())} at least"
        )


def _say_clearance(
    ratings: Sequence[Rating], errors: np.ndarray, clearance: float
) -> None:
    """Print which pairs the measurement ordered, and say plainly when none.

    **A rating whose error bars overlap for every pair says nothing.** A
    report of it must state that rather than print a ranking, because a
    ranking of noise reads exactly like a ranking of play.
    """
    found = ordered_pairs(ratings, errors, clearance)
    pairs = len(ratings) * (len(ratings) - 1) // 2
    if not found:
        print(
            f"  none of the {pairs} pairs clears {clearance:.0f} standard "
            "errors. **These players are indistinguishable on this "
            "measurement.**"
        )
        return
    print(
        f"  {len(found)} of the {pairs} pairs clear {clearance:.0f} standard errors\n"
    )
    for high, low, gap, error in found:
        print(f"  {high:>22s} over {low:<22s} {gap:8.1f} elo, error {error:6.1f}")


def report(
    config: EnvConfig,
    seeds: Sequence[int],
    seatings: Sequence[Seating],
    results: Sequence[GameResult],
    ratings: Sequence[Rating],
    errors: np.ndarray,
    found: Sequence[Ending],
    wins: np.ndarray,
    together: np.ndarray,
) -> dict[str, Any]:
    """Return every game and every figure, so a later reader asks a new question.

    A run that kept only the printed table would have to be played again for
    the next question, and the league is the expensive part.
    """
    return {
        "world": {
            "width": config.width,
            "height": config.height,
            "faction_count": config.faction_count,
            "tick_limit": config.tick_limit,
            "horizon": config.horizon,
            "decision_interval": config.decision_interval,
        },
        "seeds": [int(seed) for seed in seeds],
        "games": len(seatings),
        "worlds": len({seating.world for seating in seatings}),
        "undecided": sum(1 for row in results if row.winner is None),
        "prior_width": PRIOR_WIDTH,
        "bootstrap_draws": BOOTSTRAP_DRAWS,
        "bootstrap_seed": BOOTSTRAP_SEED,
        "schedule_seed": SCHEDULE_SEED,
        "seat_wins": {str(seat): count for seat, count in seat_wins(results).items()},
        "ratings": [row.as_dict() for row in ratings],
        "difference_errors": errors.tolist(),
        "ordered_pairs": [
            {"stronger": high, "weaker": low, "elo": gap, "error": error}
            for high, low, gap, error in ordered_pairs(ratings, errors)
        ],
        "endings": [row.as_dict() for row in found],
        "head_to_head": {"wins": wins.tolist(), "together": together.tolist()},
        "results": [row.as_dict() for row in results],
    }


def main() -> None:
    """Play the league, fit the ratings, and print what the games said."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--checkpoints",
        type=Path,
        default=Path("checkpoints/place"),
        help="the directory of stored policies. Every checkpoint plays.",
    )
    parser.add_argument(
        "--seeds",
        type=int,
        default=12,
        help="how many held-out worlds the schedule draws from, in turn",
    )
    parser.add_argument("--seed-start", type=int, default=HELDOUT_START)
    parser.add_argument(
        "--worlds-per-triple",
        type=int,
        default=1,
        help=(
            "how many worlds each group of three players plays. Each world "
            "hosts one rotation for each seat, so this multiplies the game "
            "count by three times itself."
        ),
    )
    parser.add_argument("--workers", type=int, default=12)
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument(
        "--draws",
        type=int,
        default=BOOTSTRAP_DRAWS,
        help="how many resamples the error bars come from",
    )
    parser.add_argument("--out", type=Path, default=None)
    arguments = parser.parse_args()

    from cachette.learn.__main__ import STRATEGIES
    from cachette.learn.train import first_scoring

    strategies = {
        name: (config, first_scoring(scoring), kind)
        for name, (config, scoring, kind) in STRATEGIES.items()
    }
    config = replace(one_world(strategies), threads=arguments.threads)
    scoring = probe_scoring()
    probe = Env(config, scoring)
    players = [
        *stored_players(arguments.checkpoints, strategies, PolicyFit.of_env(probe)),
        Player(name=CONTROLLER, policy=None),
    ]
    anchor = len(players) - 1
    seats = config.faction_count
    seeds = viable_seeds(config, arguments.seeds, arguments.seed_start)
    seatings = schedule(len(players), seats, seeds, arguments.worlds_per_triple)
    require_balance(seatings, seats)

    print(
        f"world {config.width} by {config.height}, {seats} factions, tick "
        f"limit {config.tick_limit}, horizon {config.horizon} decisions of "
        f"{config.decision_interval} ticks"
    )
    print(
        f"observation version {probe.observation_version}, action version "
        f"{probe.action_version}"
    )
    print(f"{len(players)} players: {[player.name for player in players]}")
    print(f"{len(seeds)} held-out seeds from {arguments.seed_start}: {seeds}")
    held = balance_of(seatings)
    print(
        f"{len(seatings)} games over "
        f"{len({row.world for row in seatings})} worlds, and every player "
        f"holds each seat {held[0][0]} times\n"
    )

    def say(done: int, total: int) -> None:
        """Print how far the league has run."""
        print(f"  played {done}/{total} games", flush=True)

    results = play_league(config, scoring, players, seatings, arguments.workers, say)
    undecided = sum(1 for row in results if row.winner is None)
    print(f"\n{len(results)} games played, {undecided} with no recorded winner")

    ratings, _spread, raw = rate(results, players, anchor, arguments.draws)
    errors = difference_errors(raw)
    found = endings(results, players)
    wins, together = head_to_head(results, len(players))

    seats_won = seat_wins(results)
    rated = sum(seats_won.values())
    print("\nwhat each seat won, with every player holding every seat alike\n")
    for seat, count in seats_won.items():
        print(f"  seat {seat} won {count:4d}/{rated:<4d} {count / rated:7.3f}")

    print(f"\nthe rating, with the {CONTROLLER} anchored at zero")
    print(
        "the anchor holds an error of zero by construction. Read the pairwise "
        "errors below.\n"
    )
    _say_ratings(ratings, CLEARANCE)
    print("\nwhich pairs the measurement ordered\n")
    _say_clearance(ratings, errors, CLEARANCE)
    print("\nwhere the games of each player ended\n")
    _say_endings(found)
    print()
    _say_paths(found)
    print("\nthe head to head share of the decided games of each pair\n")
    _say_head_to_head(players, wins, together)

    if arguments.out is not None:
        payload = report(
            config, seeds, seatings, results, ratings, errors, found, wins, together
        )
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        print(f"\nwrote {arguments.out}")


if __name__ == "__main__":
    main()
