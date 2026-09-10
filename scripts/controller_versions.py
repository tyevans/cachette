#!/usr/bin/env python3
"""Play versions of the built-in controller against each other in one world.

The hunting rule of the built-in controller is governed by one number. A
faction that holds that multiple of the ground of another names it prey, moves
its relation toward war on every tick, and marches on its nearest settlement. A
ratio at or below zero takes the rule out of the game, and the faction then
moves only against the faction that holds the most ground.

**The ratio belongs to one faction and not to the world.** Each faction reads
its own value, so two versions of the controller take two seats of one world.
This tool seats them, plays the games and reports what each version did.

# One world, and not two

The older way to compare two settings was to run two whole worlds and to read
the difference. That difference holds the world as well as the setting, and
nothing in it says which of the two moved. Every game here holds every version
under test, so the map, the weather and the opponents are common to all of
them.

# The seats do not win equally often, so a version must play every seat

The balance register holds the seat share of the built-in controller over a
fixed seed set, and the seats do not win equally often in it.[^1] A version
left in one seat would carry that seat's luck into its win share.

Each version therefore plays each seat the same number of times. The schedule
takes each unordered group of the versions that fills the seats, and it plays
that group once for each seat on one world. Version ``j`` of the group takes
seat ``(j + r) % seats`` on rotation ``r``. The rotations of one world share
the map and the weather, so the turn is inside one world. This is the seat
rotation the training run already uses for a league of candidates.[^2]

The tool counts the seats of the finished schedule and refuses a schedule that
is not balanced. It reads the schedule, the balance check and the spread of a
set of numbers from the league tool, so no rule of this kind is stored
twice.[^3]

# What the report states, and what it cannot see

For each version the report gives the games it played, the games it won, its
win share, and the spread of the end tick of its games. The spread holds the
median and the mean, because the end tick piles up at the tick limit and a mean
alone hides that.

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
the value stays what that row states.[^1]

References
----------
[^1]: Budgets and costs, the overmatch ratio and the seat share rows.
``docs/reference/balance.md``

[^2]: The training entry point, the seat list of a league run.
``python/cachette/learn/__main__.py``

[^3]: The rating tool of the stored policies.
``scripts/policy_league.py``
"""

from __future__ import annotations

import argparse
import importlib.util
import json
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


def _sibling(name: str) -> ModuleType:
    """Import a script that sits beside this one, because there is no package.

    The rating tool of the stored policies already holds the seat schedule, the
    balance check and the spread of a set of numbers. **A second copy of any of
    them would be one rule stored twice**, with nothing that fails when the
    copies disagree.
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
class Version:
    """One version of the built-in controller, and the name the report gives it.

    The ratio is the raw Q16.16 factor the engine takes. The engine holds no
    other setting that parts one version of the controller from another today.
    """

    name: str
    ratio: int

    def as_dict(self) -> dict[str, Any]:
        """Return this version as plain values, for a report file."""
        return {"name": self.name, "ratio": self.ratio}


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
        Version(name="default", ratio=ratio),
        Version(name="off", ratio=0),
        Version(name="patient", ratio=ratio * 2),
    ]


def parse_versions(named: Sequence[str]) -> list[Version]:
    """Read a version list of the form ``name=ratio``, and refuse a bad entry.

    The ratio is the raw Q16.16 factor the engine takes, so a caller states the
    same number the engine stores.
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
    if len({version.name for version in versions}) != len(versions):
        message = "two versions carry one name, and a report cannot tell them apart"
        raise ValueError(message)
    return versions


def build_world(seed: int, ratios: Sequence[int]) -> World:
    """Build one game, and give each seat the ratio of the version that holds it.

    The world is the world every measurement of this project plays. The win
    readers are on, so the engine writes an end record when a reader fires and
    at the tick limit.

    **Each seat is written on its own.** Nothing here writes a ratio for every
    faction at once, so the seats hold the values the schedule gave them.
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
    for seat, ratio in enumerate(ratios):
        world.set_faction_overmatch_ratio(seat, int(ratio))
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

    Every seat of every world is driven by the built-in controller, so this
    sends no action at all. The batch reports its rows in world index order,
    the live list is rebuilt in index order, and the results are read in index
    order after the last step. **Nothing here reads which world finished
    first.**

    A game the batch reports an error for stops the whole chunk. A schedule
    with a missing game is unbalanced, and a win share over it measures the
    seat.
    """
    worlds = [
        build_world(
            seating.seed, [versions[player].ratio for player in seating.players]
        )
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


def version_rows(
    versions: Sequence[Version], results: Sequence[GameResult]
) -> list[dict[str, Any]]:
    """Return one report row for each version, in the order the caller named.

    The row holds the games the version played, the games it won, its win
    share, the games of it that no win reader ended, and the spread of the end
    tick of its games.
    """
    rows: list[dict[str, Any]] = []
    for index, version in enumerate(versions):
        played = [result for result in results if index in result.versions]
        won = [result for result in played if result.winner == index]
        no_winner = [result for result in played if result.winner is None]
        limit = [result for result in played if result.reached_limit]
        row: dict[str, Any] = {
            "name": version.name,
            "ratio": version.ratio,
            "games": len(played),
            "wins": len(won),
            "win_share": len(won) / len(played) if played else 0.0,
            "games_with_no_winner": len(no_winner),
            "games_at_the_tick_limit": len(limit),
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


def path_rows(results: Sequence[GameResult]) -> dict[str, int]:
    """Count how many games each win path ended, in path name order.

    **The paths come from the results and never from a list written here.** The
    engine owns the names, and a list here would read a name the engine stopped
    writing as a zero rather than fail.
    """
    counts: dict[str, int] = {}
    for result in results:
        counts[result.path] = counts.get(result.path, 0) + 1
    return {name: counts[name] for name in sorted(counts)}


def report(
    versions: Sequence[Version],
    worlds: int,
    chunk: int,
    workers: int,
    threads: int,
    games: bool,
) -> dict[str, Any]:
    """Play the schedule and return every figure the report states."""
    seats = int(WORLD.faction_count)
    seeds = viable_seeds(WORLD, worlds, HELDOUT_START)
    seatings = schedule(len(versions), seats, seeds, worlds)
    require_balance(seatings, seats)
    results = play(versions, seatings, chunk, workers, threads)
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
            "seed_start": HELDOUT_START,
            "seats_held": {
                str(player): dict(sorted(held.items()))
                for player, held in sorted(balance_of(seatings).items())
            },
        },
        "versions": version_rows(versions, results),
        "seats": seat_rows(seats, results),
        "paths": path_rows(results),
    }
    if games:
        answer["games"] = [result.as_dict() for result in results]
    return answer


def main() -> None:
    """Read the arguments, play the schedule and print the report."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--version",
        action="append",
        default=[],
        metavar="NAME=RATIO",
        help=(
            "a version of the built-in controller, named and given a raw "
            "Q16.16 overmatch ratio. Repeat the flag for each version. The "
            "tool plays three versions of its own when the flag is absent"
        ),
    )
    parser.add_argument(
        "--worlds",
        type=int,
        default=DEFAULT_WORLDS,
        help="how many worlds each group of versions plays",
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
    versions = (
        parse_versions(arguments.version) if arguments.version else default_versions()
    )
    print(
        json.dumps(
            report(
                versions,
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
