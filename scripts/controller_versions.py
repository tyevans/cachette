#!/usr/bin/env python3
"""Play versions of the built-in controller against each other in one world.

A version is one seat configuration of the built-in controller. It names the
five option weights of a faction, the hunting ratio of that faction, and
whether an external caller holds the seat. **Every one of those values belongs
to one faction and not to the world**, so two versions take two seats of one
world. This tool seats them, plays the games and reports what each version did.

A design report proposes a family of such versions and predicts an order for
them.[^1] This tool holds that family by name, so a caller seats a member
rather than spelling its settings out.

# One weight steers nothing, and the tool refuses a pair that ties

The trade weight biases one draw, and that draw reaches a subsystem that
records no offer, no contract and no carrier over a whole run.[^1]

Two versions that differ only in the trade weight are therefore one player
twice. A ranking that ordered them would report noise as a result. The tool
reads the steering part of each version and refuses a schedule that holds one
of them twice.

**The renown weight now steers the campaign raise.** It reached no decision
until the engine split the raise away from the war weight, so a version that
names it is a version of its own.[^5]

# One world, and not two

The older way to compare two settings was to run two whole worlds and to read
the difference. That difference holds the world as well as the setting, and
nothing in it says which of the two moved. Every game here holds every version
under test, so the map, the weather and the opponents are common to all of
them.

# The seats do not win equally often, so a version must play every seat

The balance register holds the seat share of the built-in controller over a
fixed seed set, and the seats do not win equally often in it.[^2] A version
left in one seat would carry that seat's luck into its win share.

Each version therefore plays each seat the same number of times. The schedule
takes each unordered group of the versions that fills the seats, and it plays
that group once for each seat on one world. Version ``j`` of the group takes
seat ``(j + r) % seats`` on rotation ``r``. The rotations of one world share
the map and the weather, so the turn is inside one world. This is the seat
rotation the training run already uses for a league of candidates.[^3]

The tool counts the seats of the finished schedule and refuses a schedule that
is not balanced. It reads the schedule, the balance check, the clearance bar
and the spread of a set of numbers from the league tool, so no rule of this
kind is stored twice.[^4]

# Every win share carries an error bar, and a close pair stays unordered

A win share of a three-seat world has an even value of one third. A share near
one third carries a standard error near 0.059 over 64 games and near 0.029
over 256 games. **Two versions a few hundredths apart are therefore one
reading and not two.** The report of this tool gives the standard error and an
interval beside every share, and it names for each version the versions the
sample cannot part it from.

The tool parts a pair on the games the pair played together. A win for one
version of a game is a loss for the other, so the two shares of one game move
against each other. An error bar that took the two shares for independent
would state a smaller number than the sample holds. The paired figure takes
the games in common, counts the wins of each, and reads the difference against
the error of that difference.

**A handful of games parts nothing at all.** The error of a difference is a
normal approximation, and that approximation is loose over a few decided
games. The tool therefore states no separation for a pair that decided fewer
games between them than the least count it holds.

**The paired error still takes one game as independent of the next.** The
rotations of one world share a map, so the true error is a little wider than
the figure the tool states. Read a pair near the bar as unparted.

# What the report states, and what it cannot see

For each version the report gives the games it played, the games it won, its
win share with an error bar, the win paths of its games, and the spread of the
end tick of its games. The spread holds the median and the mean, because the
end tick piles up at the tick limit and a mean alone hides that.

**A game with no winner is a game the engine wrote no end record for.** The
tool reads the end record and nothing else, so it counts such a game and cannot
say why the game ended. The engine publishes no reason for an end that no win
reader fired. A separate item is giving the trainer that reporting, and this
tool will read it when it lands.

The versions of one game share its end tick, because the end of a game is one
fact about the world. The spread reported for a version is therefore the spread
of the games that version sat in. It moves because a version changes how its
games end.

**A run of exactly one group of versions gives every version one spread.** Such
a run seats every version in every game, so the games of one version are the
games of all of them. Name more versions than the world has seats, and the
groups then part.

# What this tool decides

Nothing. It measures. The default ratio is a row of the balance register, and
the value stays what that row states.[^2]

References
----------
[^1]: Report 44, a family of tunable controllers, and how to rank them.
``docs/research/reports/44-a-family-of-tunable-controllers.md``

[^2]: Budgets and costs, the overmatch ratio and the seat share rows.
``docs/reference/balance.md``

[^3]: The training entry point, the seat list of a league run.
``python/cachette/learn/__main__.py``

[^4]: The rating tool of the stored policies.
``scripts/policy_league.py``

[^5]: Findings register, FND-738. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import argparse
import importlib.util
import itertools
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType
from typing import TYPE_CHECKING, Any, Protocol

from cachette._core import Batch, World
from cachette.learn.__main__ import WORLD
from cachette.learn.env import viable_seeds
from cachette.learn.record import end_tick_of

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

# How many worlds each group of versions plays by default. The tool is meant
# to be cheap to run again, so the default is a few dozen games and not a few
# hundred.
DEFAULT_WORLDS = 8

# How many worlds the tool plays at once, and how many threads one step takes.
# Both stay small, because a training run usually holds the machine.
DEFAULT_CHUNK = 8
DEFAULT_WORKERS = 2
DEFAULT_THREADS = 2

# The name the report gives a game that no win reader ended.
NO_WINNER = "none"

# How many of the shared games a pair must decide between them before the tool
# states a separation. The error of a difference of two shares is a normal
# approximation, and that approximation is loose when the counts are small.
# **A handful of games separates nothing**, and a tool that said otherwise
# would report a run made to prove the tool as if it ranked the family.
LEAST_DECIDED = 10

# The raw value of one, at the fixed-point scale this project holds every
# simulated number at. The hunting ratio is a factor, so this raw value means
# parity: a faction hunts another that holds the same ground as it does.[^1]
#
# [^1]: Project orientation, the fixed-point scale. ``CLAUDE.md``
RATIO_ONE = 65536


def _sibling(name: str) -> ModuleType:
    """Import a script that sits beside this one, because there is no package.

    The rating tool of the stored policies already holds the seat schedule, the
    balance check, the clearance bar and the spread of a set of numbers. **A
    second copy of any of them would be one rule stored twice**, with nothing
    that fails when the copies disagree.
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


_league = _sibling("policy_league")
Spread = _league.Spread
schedule = _league.schedule
require_balance = _league.require_balance
balance_of = _league.balance_of
HELDOUT_START = _league.HELDOUT_START
CLEARANCE = _league.CLEARANCE


class Seating(Protocol):
    """What this module needs of one game of the schedule.

    The league tool builds the schedule and owns the shape. This states the
    part of it that the tool reads, so a reader of this file needs no other
    file to follow the code.
    """

    @property
    def world(self) -> int:
        """Give back the world the game plays, which groups the rotations."""

    @property
    def seed(self) -> int:
        """Give back the seed the world is built from."""

    @property
    def rotation(self) -> int:
        """Give back how far the seats are turned in this game."""

    @property
    def players(self) -> tuple[int, ...]:
        """Give back the version each seat holds, in seat order."""


@dataclass(frozen=True)
class Weights:
    """The five option weights of one faction.

    The engine draws every weight from the seed, and it holds each of them
    between one and eight. The verb that writes the vector refuses a weight
    outside that bound, so this holds no copy of the bound.[^1]

    The war weight biases the relation move. The renown weight biases the
    campaign raise. The build weight splits an evaluation between a gather
    order and a build order. The settle weight biases the founding draw.
    **The trade weight reaches no decision that changes the world today.**[^2]

    References
    ----------
    [^1]: The faction bindings, the weight verb.
    ``crates/cachette-py/src/world/faction_view.rs``

    [^2]: Report 44, the settings that steer nothing.
    ``docs/research/reports/44-a-family-of-tunable-controllers.md``
    """

    war: int
    trade: int
    build: int
    renown: int
    settle: int

    def as_dict(self) -> dict[str, int]:
        """Return this vector as plain values, for a report file."""
        return {
            "war": self.war,
            "trade": self.trade,
            "build": self.build,
            "renown": self.renown,
            "settle": self.settle,
        }


@dataclass(frozen=True)
class Version:
    """One version of the built-in controller, and the name the report gives it.

    A weight vector of ``None`` leaves the vector the seed drew. A ratio of
    ``None`` leaves the ratio the engine gives every new faction. **The
    external flag takes the seat away from the controller**, and this tool
    sends no action, so a version that raises the flag plays no move at all.

    The ratio is the raw fixed-point factor the engine takes.
    """

    name: str
    weights: Weights | None = None
    ratio: int | None = None
    external: bool = False

    def steering(self) -> tuple[object, ...]:
        """Return the part of this version that reaches a decision.

        Two versions with one steering value are one player twice. The trade
        weight is out of the answer, because no decision that changes the
        world reads it. The renown weight is in the answer, because it biases
        the campaign raise. A version under external control makes no
        decision at all, so its whole configuration is out.
        """
        drawn = self.weights
        if self.external:
            return ("external",)
        return (
            "played",
            None if drawn is None else drawn.war,
            None if drawn is None else drawn.build,
            None if drawn is None else drawn.renown,
            None if drawn is None else drawn.settle,
            self.ratio,
        )

    def as_dict(self) -> dict[str, Any]:
        """Return this version as plain values, for a report file."""
        return {
            "name": self.name,
            "weights": None if self.weights is None else self.weights.as_dict(),
            "overmatch_ratio": self.ratio,
            "external_control": self.external,
        }


# The family the design report proposes, in the order the report predicts.[^1]
# The weakest member comes first. Each entry names only the settings that
# member writes, so a member with no weight vector plays the vector the seed
# drew for its seat.
#
# **Every member carries its war weight in its renown weight as well.** The
# war weight decided the relation move and the campaign raise together when
# the report wrote this family. The engine now takes the campaign raise from
# the renown weight, so an equal pair keeps each member playing as the report
# describes it.[^2]
#
# [^1]: Report 44, the proposed variants.
# ``docs/research/reports/44-a-family-of-tunable-controllers.md``
#
# [^2]: Findings register, FND-738. ``docs/FINDINGS.md``
FAMILY: dict[str, Version] = {
    "mute": Version(name="mute", external=True),
    "quietist": Version(
        name="quietist",
        weights=Weights(war=1, trade=8, build=1, renown=1, settle=1),
    ),
    "settler": Version(
        name="settler",
        weights=Weights(war=1, trade=1, build=2, renown=1, settle=8),
    ),
    "mason": Version(
        name="mason",
        weights=Weights(war=1, trade=1, build=8, renown=1, settle=4),
    ),
    "default": Version(name="default"),
    "warlord": Version(
        name="warlord",
        weights=Weights(war=8, trade=1, build=3, renown=8, settle=4),
    ),
    "hunter": Version(
        name="hunter",
        weights=Weights(war=8, trade=1, build=1, renown=8, settle=1),
        ratio=RATIO_ONE,
    ),
}


@dataclass(frozen=True)
class GameResult:
    """What one game of the schedule ended at.

    The winner entry names the version that won, and it is absent for a game
    that no win reader ended. The end tick and the path are one fact about the
    world, so every version of the game shares them.
    """

    index: int
    world: int
    seed: int
    rotation: int
    versions: tuple[int, ...]
    winner: int | None
    path: str
    end_tick: int
    reached_limit: bool

    def as_dict(self) -> dict[str, Any]:
        """Return this result as plain values, for a report file."""
        return {
            "index": self.index,
            "world": self.world,
            "seed": self.seed,
            "rotation": self.rotation,
            "versions": list(self.versions),
            "winner": self.winner,
            "path": self.path,
            "end_tick": self.end_tick,
            "reached_limit": self.reached_limit,
        }


def default_ratio() -> int:
    """Return the ratio the engine gives every faction of a new world.

    **The default comes from the engine and never from a number written here.**
    The balance register holds the row, and a copy of the value in this script
    would state the row a second time.
    """
    world = World(
        width=WORLD.width,
        height=WORLD.height,
        faction_count=WORLD.faction_count,
        seed=0,
    )
    return int(world.faction_overmatch_ratio(0))


def default_versions() -> list[Version]:
    """Return the three versions the tool plays when the caller names none.

    The first keeps the engine default. The second takes the hunting rule out
    of the game, which is the behaviour that shipped before the rule. The third
    doubles the default, so a faction waits longer before it names a prey.
    """
    ratio = default_ratio()
    return [
        Version(name="default"),
        Version(name="off", ratio=0),
        Version(name="patient", ratio=ratio * 2),
    ]


def family_versions(named: Sequence[str]) -> list[Version]:
    """Return the named members of the family, and refuse a name it lacks."""
    versions: list[Version] = []
    for name in named:
        member = FAMILY.get(name)
        if member is None:
            known = ", ".join(FAMILY)
            message = f"{name!r} names no member of the family. Take one of: {known}"
            raise ValueError(message)
        versions.append(member)
    return versions


def parse_versions(named: Sequence[str]) -> list[Version]:
    """Read a version list of the form ``name=ratio``, and refuse a bad entry.

    The ratio is the raw fixed-point factor the engine takes, so a caller
    states the same number the engine stores. Such a version keeps the weight
    vector the seed drew.
    """
    versions: list[Version] = []
    for entry in named:
        name, _, raw = entry.partition("=")
        if not name or not raw:
            message = (
                f"{entry!r} is not a version. Write a name, an equals sign and a ratio."
            )
            raise ValueError(message)
        versions.append(Version(name=name, ratio=int(raw)))
    return versions


def require_named_apart(versions: Sequence[Version]) -> None:
    """Refuse a schedule that holds one name twice.

    The report names a row by the name of its version, so two rows under one
    name state nothing that a reader can tell apart.
    """
    if len({version.name for version in versions}) != len(versions):
        message = "two versions carry one name, and a report cannot tell them apart"
        raise ValueError(message)


def require_steering_apart(versions: Sequence[Version]) -> None:
    """Refuse a schedule that holds one player twice under two names.

    **A ranking that parts two identical players measures noise.** Two
    versions that differ only in the trade weight reach the world alike,
    because no decision that changes the world reads that weight.

    **One tie stays outside this check.** A version that leaves the hunting
    ratio alone and a version that spells the engine default out hold two
    values here, and the engine reads one. The check would have to build a
    world to see that, so it does not.
    """
    seen: dict[tuple[object, ...], str] = {}
    for version in versions:
        key = version.steering()
        held = seen.get(key)
        if held is not None:
            message = (
                f"{version.name!r} steers the world exactly as {held!r} does, "
                "so a ranking of the two would measure noise"
            )
            raise ValueError(message)
        seen[key] = version.name


def seat_a_version(world: World, seat: int, version: Version) -> None:
    """Write one version onto one seat of a world the engine has seeded.

    **The seeding draws the weight vector, so this writes after it.** Each verb
    names one faction, so nothing here touches a seat the caller did not name.
    """
    if version.weights is not None:
        world.set_faction_weights(
            seat,
            war=version.weights.war,
            trade=version.weights.trade,
            build=version.weights.build,
            renown=version.weights.renown,
            settle=version.weights.settle,
        )
    if version.ratio is not None:
        world.set_faction_overmatch_ratio(seat, int(version.ratio))
    if version.external:
        world.set_externally_controlled(seat, True)


def build_world(seed: int, seated: Sequence[Version]) -> World:
    """Build one game, and give each seat the version that holds it.

    The world is the world every measurement of this project plays. The win
    readers are on, so the engine writes an end record when a reader fires and
    at the tick limit.

    **Each seat is written on its own.** Nothing here writes a setting for
    every faction at once, so the seats hold the values the schedule gave them.
    """
    world = World(
        width=WORLD.width,
        height=WORLD.height,
        faction_count=WORLD.faction_count,
        seed=seed,
    )
    world.seed_world()
    world.set_win_readers_enabled(True)
    world.set_tick_limit(WORLD.tick_limit)
    for seat, version in enumerate(seated):
        seat_a_version(world, seat, version)
    return world


def read_game(index: int, seating: Seating, world: World) -> GameResult:
    """Read the outcome of one finished game from the world it played.

    **The winner comes from the end record and from nothing else.** No seat of
    this tool holds a policy, so no seat holds a score. The end record answers
    for every seat, and the win is a public fact of the world.

    The end tick comes from the one reader the record module holds for it.
    """
    end = world.game_end()
    end_tick = end_tick_of(world)
    limit = int(world.tick_limit)
    winner_seat = None if end is None else int(end["winner"])
    return GameResult(
        index=index,
        world=seating.world,
        seed=seating.seed,
        rotation=seating.rotation,
        versions=seating.players,
        winner=None if winner_seat is None else seating.players[winner_seat],
        path=NO_WINNER if end is None else str(end["path"]),
        end_tick=end_tick,
        reached_limit=limit > 0 and end_tick >= limit,
    )


def play_chunk(
    versions: Sequence[Version],
    seatings: Sequence[Seating],
    first: int,
    workers: int,
    threads: int,
) -> list[GameResult]:
    """Play a set of games together, and return the results in schedule order.

    The engine drives every seat that the built-in controller holds, so this
    sends no action at all. A seat under external control therefore plays
    nothing. The batch reports its rows in world index order, the live list is
    rebuilt in index order, and the results are read in index order after the
    last step. **Nothing here reads which world finished first.**

    A game the batch reports an error for stops the whole chunk. A schedule
    with a missing game is unbalanced, and a win share over it measures the
    seat.
    """
    worlds = [
        build_world(seating.seed, [versions[player] for player in seating.players])
        for seating in seatings
    ]
    live = list(range(len(worlds)))
    batch = Batch([worlds[index] for index in live], workers)

    def running(index: int) -> bool:
        """Say whether one world still has a game to play."""
        world = worlds[index]
        return world.game_end() is None and int(world.tick) <= int(world.tick_limit)

    while live:
        for row in batch.step(threads):
            if row.error is not None:
                failed = live[row.index]
                message = f"the world of game {first + failed} refused: {row.error}"
                raise RuntimeError(message)
        now = [index for index in live if running(index)]
        if now != live:
            live = now
            if live:
                batch = Batch([worlds[index] for index in live], workers)
    return [
        read_game(first + offset, seating, world)
        for offset, (seating, world) in enumerate(zip(seatings, worlds, strict=True))
    ]


def play(
    versions: Sequence[Version],
    seatings: Sequence[Seating],
    chunk: int,
    workers: int,
    threads: int,
) -> list[GameResult]:
    """Play the whole schedule, a chunk of worlds at a time."""
    results: list[GameResult] = []
    for start in range(0, len(seatings), chunk):
        held = seatings[start : start + chunk]
        results.extend(play_chunk(versions, held, start, workers, threads))
    return results


def share_error(wins: int, games: int) -> float:
    """Return the standard error of a win share, from the share itself.

    A share of zero gives an error of zero, and that reading is too narrow.
    The interval beside it is the figure to read for a version that won no
    game.
    """
    if games <= 0:
        return 0.0
    share = wins / games
    return math.sqrt(share * (1.0 - share) / games)


def share_interval(wins: int, games: int) -> dict[str, float]:
    """Return an interval for a win share that holds at a share of zero.

    The interval is the score interval of a binomial share, at the clearance
    the league tool holds. **A share of zero keeps a width here**, which the
    standard error alone does not give it.
    """
    if games <= 0:
        return {"low": 0.0, "high": 0.0}
    bar = float(CLEARANCE)
    share = wins / games
    weight = bar * bar / games
    middle = (share + weight / 2.0) / (1.0 + weight)
    half = (
        bar
        * math.sqrt(share * (1.0 - share) / games + weight / (4.0 * games))
        / (1.0 + weight)
    )
    return {"low": max(0.0, middle - half), "high": min(1.0, middle + half)}


def paired_gap(results: Sequence[GameResult], one: int, other: int) -> dict[str, Any]:
    """Return the gap between two versions over the games they played together.

    A win for one version of a game is a loss for the other, so the two shares
    of one game move against each other. **A gap read over the shared games
    holds that**, and a gap between two shares taken apart does not.

    The error is the error of the difference of two shares of one multinomial
    draw. The tool parts the pair when the gap exceeds the clearance times that
    error, and when the pair decided enough games between them for that error
    to mean anything.
    """
    shared = [
        result
        for result in results
        if one in result.versions and other in result.versions
    ]
    games = len(shared)
    if games == 0:
        return {"games": 0, "decided": 0, "gap": 0.0, "error": 0.0, "separated": False}
    won = sum(1 for result in shared if result.winner == one)
    lost = sum(1 for result in shared if result.winner == other)
    decided = won + lost
    gap = (won - lost) / games
    spread = (decided - (won - lost) ** 2 / games) / games**2
    error = math.sqrt(max(spread, 0.0))
    return {
        "games": games,
        "decided": decided,
        "gap": gap,
        "error": error,
        "separated": decided >= LEAST_DECIDED and abs(gap) > float(CLEARANCE) * error,
    }


def separation_rows(
    versions: Sequence[Version], results: Sequence[GameResult]
) -> list[dict[str, Any]]:
    """Return one row for each pair of versions, with the gap and the verdict."""
    rows: list[dict[str, Any]] = []
    for one, other in itertools.combinations(range(len(versions)), 2):
        found = paired_gap(results, one, other)
        rows.append(
            {
                "versions": [versions[one].name, versions[other].name],
                "shared_games": found["games"],
                "decided_games": found["decided"],
                "gap": found["gap"],
                "gap_error": found["error"],
                "separated": found["separated"],
            }
        )
    return rows


def order_rows(
    versions: Sequence[Version],
    results: Sequence[GameResult],
    rows: Sequence[dict[str, Any]],
) -> list[dict[str, Any]]:
    """Return the versions by win share, and name the ties the sample holds.

    **A rank alone implies an order that the sample may not hold.** Each row
    therefore names the versions this sample cannot part it from. A row that
    names another stands level with it, whichever of the two the rank puts
    first.
    """
    shares = {row["name"]: float(row["win_share"]) for row in rows}
    errors = {row["name"]: float(row["win_share_error"]) for row in rows}
    ties: dict[str, list[str]] = {version.name: [] for version in versions}
    for one, other in itertools.combinations(range(len(versions)), 2):
        if not paired_gap(results, one, other)["separated"]:
            ties[versions[one].name].append(versions[other].name)
            ties[versions[other].name].append(versions[one].name)
    order = sorted(versions, key=lambda version: -shares[version.name])
    return [
        {
            "rank": place + 1,
            "name": version.name,
            "win_share": shares[version.name],
            "win_share_error": errors[version.name],
            "not_separated_from": ties[version.name],
        }
        for place, version in enumerate(order)
    ]


def path_counts(results: Sequence[GameResult]) -> dict[str, int]:
    """Count how many games each win path ended, in path name order.

    **The paths come from the results and never from a list written here.** The
    engine owns the names, and a list here would read a name the engine stopped
    writing as a zero rather than fail.
    """
    counts: dict[str, int] = {}
    for result in results:
        counts[result.path] = counts.get(result.path, 0) + 1
    return {name: counts[name] for name in sorted(counts)}


def version_rows(
    versions: Sequence[Version], results: Sequence[GameResult]
) -> list[dict[str, Any]]:
    """Return one report row for each version, in the order the caller named.

    The row holds the settings the version writes, the games it played, the
    games it won, its win share with an error bar, the games of it that no win
    reader ended, the win paths of its games, and the spread of the end tick.

    **The paths it played and the paths it won answer two questions.** A
    version that builds fast raises the share of its games that end on the
    wonder path, whether or not it is the version that finishes the wonder.
    """
    rows: list[dict[str, Any]] = []
    for index, version in enumerate(versions):
        played = [result for result in results if index in result.versions]
        won = [result for result in played if result.winner == index]
        no_winner = [result for result in played if result.winner is None]
        limit = [result for result in played if result.reached_limit]
        row: dict[str, Any] = {
            "name": version.name,
            "settings": version.as_dict(),
            "games": len(played),
            "wins": len(won),
            "win_share": len(won) / len(played) if played else 0.0,
            "win_share_error": share_error(len(won), len(played)),
            "win_share_interval": share_interval(len(won), len(played)),
            "games_with_no_winner": len(no_winner),
            "games_at_the_tick_limit": len(limit),
            "paths_played": path_counts(played),
            "paths_won": path_counts(won),
            "end_tick": Spread.of(
                [float(result.end_tick) for result in played]
            ).as_dict()
            if played
            else None,
        }
        rows.append(row)
    return rows


def seat_rows(seats: int, results: Sequence[GameResult]) -> list[dict[str, Any]]:
    """Return the win share of each seat, with the versions averaged out.

    Every version holds every seat the same number of times, so this share is
    the seat luck of the world the tool plays. Read it as the size of the
    effect the rotation cancels.
    """
    rows: list[dict[str, Any]] = []
    for seat in range(seats):
        won = [
            result
            for result in results
            if result.winner is not None
            and result.versions.index(result.winner) == seat
        ]
        rows.append(
            {
                "seat": seat,
                "wins": len(won),
                "win_share": len(won) / len(results) if results else 0.0,
            }
        )
    return rows


def report(
    versions: Sequence[Version],
    worlds: int,
    chunk: int,
    workers: int,
    threads: int,
    games: bool,
) -> dict[str, Any]:
    """Play the schedule and return every figure the report states."""
    require_named_apart(versions)
    require_steering_apart(versions)
    seats = int(WORLD.faction_count)
    seeds = viable_seeds(WORLD, worlds, HELDOUT_START)
    seatings = schedule(len(versions), seats, seeds, worlds)
    require_balance(seatings, seats)
    results = play(versions, seatings, chunk, workers, threads)
    rows = version_rows(versions, results)
    answer: dict[str, Any] = {
        "world": {
            "extent": WORLD.width,
            "factions": seats,
            "tick_limit": WORLD.tick_limit,
        },
        "schedule": {
            "versions": [version.as_dict() for version in versions],
            "worlds_per_group": worlds,
            "games": len(seatings),
            "games_for_each_version": len(seatings) * seats // len(versions),
            "seeds": [int(seed) for seed in seeds],
            "seed_start": HELDOUT_START,
            "seats_held": {
                str(player): dict(sorted(held.items()))
                for player, held in sorted(balance_of(seatings).items())
            },
        },
        "versions": rows,
        "order": order_rows(versions, results, rows),
        "separation": separation_rows(versions, results),
        "seats": seat_rows(seats, results),
        "paths": path_counts(results),
    }
    if games:
        answer["games"] = [result.as_dict() for result in results]
    return answer


def chosen_versions(arguments: argparse.Namespace) -> list[Version]:
    """Return the versions the arguments name, in the order they name them.

    The family members come first, and the versions a caller spelled out come
    after them. The tool plays three versions of its own when the caller names
    none.
    """
    versions = family_versions(arguments.variant) + parse_versions(arguments.version)
    return versions if versions else default_versions()


def main() -> None:
    """Read the arguments, play the schedule and print the report."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--variant",
        action="append",
        default=[],
        choices=list(FAMILY),
        metavar="NAME",
        help=(
            "a named member of the family the design report proposes. Repeat "
            f"the flag for each member. The names are: {', '.join(FAMILY)}"
        ),
    )
    parser.add_argument(
        "--version",
        action="append",
        default=[],
        metavar="NAME=RATIO",
        help=(
            "a version of the built-in controller, named and given a raw "
            "fixed-point overmatch ratio. It keeps the weight vector the seed "
            "drew. Repeat the flag for each version"
        ),
    )
    parser.add_argument(
        "--worlds",
        type=int,
        default=DEFAULT_WORLDS,
        help=(
            "how many worlds each group of versions plays. The game count is "
            "that number times the group count times the seat count, and the "
            "report states the game count and the games each version played"
        ),
    )
    parser.add_argument(
        "--chunk",
        type=int,
        default=DEFAULT_CHUNK,
        help="how many games the tool plays at once",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=DEFAULT_WORKERS,
        help="how many worker threads the batch runs",
    )
    parser.add_argument(
        "--threads",
        type=int,
        default=DEFAULT_THREADS,
        help="how many threads one world step takes",
    )
    parser.add_argument(
        "--games",
        action="store_true",
        help="print one row for each game as well as the summary",
    )
    arguments = parser.parse_args()
    print(
        json.dumps(
            report(
                chosen_versions(arguments),
                arguments.worlds,
                arguments.chunk,
                arguments.workers,
                arguments.threads,
                arguments.games,
            ),
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
