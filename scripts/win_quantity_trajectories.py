#!/usr/bin/env python3
"""Record what every faction of a game reaches on every win quantity, over time.

A win condition needs a threshold, and a threshold needs a distribution. This
tool plays whole games and samples the published quantity of **every** faction
at a fixed tick interval, so a reader sees the trajectory and not only the
final value. A quantity that saturates early makes a poor threshold, and an
end-of-game reading alone cannot tell a saturating quantity from a contested
one.

# Why this is a new tool and not an extension of the two that exist

Two tools already report on endings. The endings module joins each win path to
its signal and summarises a set of finished episodes, and it reads the signal
values a record stored **at episode end**.[^1] The controller version tool
seats whole controller configurations, rotates the seats and reports win paths
and end ticks, and it reads the end record and nothing else.[^2]

Neither reads a faction mid-game, and neither reads a faction that is not the
training seat. This tool adds exactly those two readings. It takes the seat
schedule, the version family, the world builder and the spread reader from the
controller version tool rather than restating any of them, so the players here
are the players that tool ranks.[^2]

# Every faction, and not the seat

A training run reads one seat. A win condition binds every faction, so a
distribution taken from one seat states what one seat reaches and not what the
game affords. Each sample of this tool holds one row for each faction of the
world.

# The engine states the scale, and this tool states none

A share crosses as a fixed-point integer and the engine publishes the value
that stands for one. A count crosses as a compressed magnitude and the engine
publishes the parameters that invert it. This tool divides by the published
unit and calls the published inversion, so no compression rule is written
here. A rule written here would be a second copy of an engine rule, with
nothing that fails when the copies disagree.[^3]

# What it costs

The cost is the cost of the games. One sample reads one observation for each
faction, and an observation is one array the engine already builds for the
policy. A sample interval of a hundred ticks adds about one read for each
hundred steps.

References
----------
[^1]: The endings module. ``python/cachette/learn/endings.py``

[^2]: The controller version tool. ``scripts/controller_versions.py``

[^3]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
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

import numpy as np

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterator, Sequence

    from cachette._core import World
    from cachette.learn.signals import SignalCatalogue

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "python"))

from cachette._core import Batch
from cachette.learn.env import viable_seeds
from cachette.learn.record import end_tick_of
from cachette.learn.signals import SignalCatalogue as Catalogue

# How many ticks pass between two samples of a game by default. A game of the
# standard world runs a few thousand ticks, so this gives a few dozen samples
# of each faction.
DEFAULT_SAMPLE = 100

# How many worlds each field of versions plays by default, and how the tool
# spends the machine. The defaults stay small, because a training run usually
# holds the machine.
DEFAULT_WORLDS = 8
DEFAULT_CHUNK = 8
DEFAULT_WORKERS = 2
DEFAULT_THREADS = 1

# The name the report gives a game that no win reader ended.
NO_WINNER = "none"

# The published shares this tool follows. The four win path shares come first,
# and the three beside them carry the raw progress a path is built on.
SHARES = (
    "domination_progress",
    "ground_progress",
    "wonder_track_progress",
    "renown_progress",
    "domination_leader",
    "ground_leader",
    "wonder_track_leader",
    "renown_leader",
    "wonder_progress",
    "best_renown_share",
    "held_share_world",
    "tick_share",
)

# The published counts this tool follows, each recovered through the inversion
# the engine publishes for it. The store total is the quantity the shaped
# training reward calls wealth.
COUNTS = (
    "store_total",
    "held_tiles",
    "population",
    "live_units",
    "settlements",
    "military_strength",
    "best_renown",
    "finished_upgrades",
    "rival_seats_held",
)


def _sibling(name: str) -> ModuleType:
    """Import a script that sits beside this one, because there is no package.

    The controller version tool holds the version family, the seat schedule
    and the world builder. A second copy of any of them would be one rule
    stored twice.
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


_versions = _sibling("controller_versions")
FAMILY = _versions.FAMILY
build_world = _versions.build_world
family_versions = _versions.family_versions
schedule = _versions.schedule
HELDOUT_START = _versions.HELDOUT_START
WORLD = _versions.WORLD


# The two fields of players this tool plays by default. One holds the calm end
# of the family and the other holds the aggressive end, so a distribution can
# be read against the kind of player that produced it.
class Seating(Protocol):
    """What this module needs of one game of the schedule.

    The controller version tool builds the schedule and owns the shape. This
    states the part of it that this tool reads, so a reader of this file needs
    no other file to follow the code.
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


FIELDS: dict[str, tuple[str, ...]] = {
    "calm": ("quietist", "settler", "mason"),
    "fierce": ("default", "warlord", "hunter"),
}


@dataclass(frozen=True)
class Sample:
    """What every faction of one game held at one tick.

    The shares entry and the counts entry each hold one array for each
    quantity followed, and each array holds one value for each faction in
    seat order. A share runs from zero to one. A count is the quantity the
    engine's own inversion recovers from the published magnitude.
    """

    tick: int
    shares: dict[str, list[float]]
    counts: dict[str, list[float]]

    def as_dict(self) -> dict[str, Any]:
        """Return this sample as plain values, for a report file."""
        return {
            "tick": self.tick,
            "shares": {
                name: [round(value, 5) for value in row]
                for name, row in self.shares.items()
            },
            "counts": {
                name: [round(value, 3) for value in row]
                for name, row in self.counts.items()
            },
        }


@dataclass(frozen=True)
class GameTrace:
    """One whole game: who sat where, how it ended, and what it reached."""

    index: int
    field: str
    world: int
    seed: int
    rotation: int
    seats: tuple[str, ...]
    winner_seat: int | None
    winner: str | None
    path: str
    end_tick: int
    reached_limit: bool
    samples: tuple[Sample, ...]

    def as_dict(self) -> dict[str, Any]:
        """Return this trace as plain values, for a report file."""
        return {
            "index": self.index,
            "field": self.field,
            "world": self.world,
            "seed": self.seed,
            "rotation": self.rotation,
            "seats": list(self.seats),
            "winner_seat": self.winner_seat,
            "winner": self.winner,
            "path": self.path,
            "end_tick": self.end_tick,
            "reached_limit": self.reached_limit,
            "samples": [sample.as_dict() for sample in self.samples],
        }


class Reader:
    """Reads the followed quantities of every faction of one world.

    The catalogue comes from the schema of a world, so the positions and the
    value forms are the engine's own. A quantity the schema does not publish
    is refused when the reader is built, and never read as a zero.
    """

    def __init__(self, catalogue: SignalCatalogue, factions: int) -> None:
        """Hold the signals to follow, and refuse one the schema lacks."""
        self._factions = factions
        self._shares = {name: catalogue.signal(name) for name in SHARES}
        self._counts = {name: catalogue.signal(name) for name in COUNTS}
        for name, signal in self._shares.items():
            if signal.form is None or not signal.form.unit:
                message = (
                    f"{name!r} publishes no value form, so nothing states the "
                    "value that stands for one"
                )
                raise ValueError(message)
        for name, signal in self._counts.items():
            if not signal.invertible:
                message = (
                    f"{name!r} does not invert, so no count can be recovered "
                    "from what the engine publishes for it"
                )
                raise ValueError(message)

    @property
    def precision(self) -> dict[str, float]:
        """The relative error the compression puts on each recovered count."""
        return {
            name: signal.form.relative_precision
            for name, signal in self._counts.items()
            if signal.form is not None
        }

    def sample(self, world: World) -> Sample:
        """Read every followed quantity of every faction of one world."""
        seen = [
            np.asarray(world.faction_observation(seat))
            for seat in range(self._factions)
        ]
        shares = {
            name: [float(signal.read(row)) / float(signal.form.unit) for row in seen]
            for name, signal in self._shares.items()
            if signal.form is not None
        }
        counts = {
            name: [float(signal.quantities(row)[0]) for row in seen]
            for name, signal in self._counts.items()
        }
        return Sample(tick=int(world.tick), shares=shares, counts=counts)


def play_chunk(
    reader: Reader,
    field: str,
    names: Sequence[str],
    seatings: Sequence[Seating],
    first: int,
    interval: int,
    workers: int,
    threads: int,
) -> list[GameTrace]:
    """Play a set of games together, sampling each at the tick interval.

    The engine drives every seat, so this sends no action. The batch reports
    its rows in world index order and the results are read in index order
    after the last step, so nothing here reads which world finished first.
    """
    versions = family_versions(names)
    worlds = [
        build_world(seating.seed, [versions[player] for player in seating.players])
        for seating in seatings
    ]
    traces: dict[int, list[Sample]] = {index: [] for index in range(len(worlds))}
    for index, world in enumerate(worlds):
        traces[index].append(reader.sample(world))
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
        for index in live:
            world = worlds[index]
            if int(world.tick) % interval == 0:
                traces[index].append(reader.sample(world))
        now = [index for index in live if running(index)]
        if now != live:
            for index in live:
                if index not in now:
                    traces[index].append(reader.sample(worlds[index]))
            live = now
            if live:
                batch = Batch([worlds[index] for index in live], workers)
    return [
        _read_game(reader, field, names, first + offset, seating, world, held)
        for offset, (seating, world, held) in enumerate(
            zip(seatings, worlds, [traces[i] for i in range(len(worlds))], strict=True)
        )
    ]


def _read_game(
    reader: Reader,
    field: str,
    names: Sequence[str],
    index: int,
    seating: Seating,
    world: World,
    samples: Sequence[Sample],
) -> GameTrace:
    """Read the outcome of one finished game and pair it with its trajectory."""
    end = world.game_end()
    end_tick = end_tick_of(world)
    limit = int(world.tick_limit)
    winner_seat = None if end is None else int(end["winner"])
    seats = tuple(names[player] for player in seating.players)
    return GameTrace(
        index=index,
        field=field,
        world=seating.world,
        seed=seating.seed,
        rotation=seating.rotation,
        seats=seats,
        winner_seat=winner_seat,
        winner=None if winner_seat is None else seats[winner_seat],
        path=NO_WINNER if end is None else str(end["path"]),
        end_tick=end_tick,
        reached_limit=limit > 0 and end_tick >= limit,
        samples=tuple(samples),
    )


def play_field(
    reader: Reader,
    field: str,
    names: Sequence[str],
    seeds: Sequence[int],
    worlds: int,
    interval: int,
    chunk: int,
    workers: int,
    threads: int,
) -> list[GameTrace]:
    """Play one field of players over the seeds, a chunk of worlds at a time."""
    seatings = schedule(len(names), WORLD.faction_count, seeds, worlds)
    traced: list[GameTrace] = []
    for start in range(0, len(seatings), chunk):
        held = seatings[start : start + chunk]
        traced.extend(
            play_chunk(reader, field, names, held, start, interval, workers, threads)
        )
    return traced


def build_reader() -> Reader:
    """Build the quantity reader from the schema of the standard world."""
    world = build_world(0, family_versions(["default"] * WORLD.faction_count))
    return Reader(Catalogue.of_world(world), WORLD.faction_count)


def named_fields(named: Sequence[str]) -> Iterator[tuple[str, tuple[str, ...]]]:
    """Give back each named field of players, and refuse a name it lacks."""
    for name in named:
        held = FIELDS.get(name)
        if held is None:
            known = ", ".join(FIELDS)
            message = f"{name!r} names no field. Take one of: {known}"
            raise ValueError(message)
        yield name, held


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Read the arguments of one run of this tool."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fields", nargs="+", default=sorted(FIELDS))
    parser.add_argument("--worlds", type=int, default=DEFAULT_WORLDS)
    parser.add_argument("--sample", type=int, default=DEFAULT_SAMPLE)
    parser.add_argument("--chunk", type=int, default=DEFAULT_CHUNK)
    parser.add_argument("--workers", type=int, default=DEFAULT_WORKERS)
    parser.add_argument("--threads", type=int, default=DEFAULT_THREADS)
    parser.add_argument("--seed-start", type=int, default=HELDOUT_START)
    parser.add_argument("--out", type=Path, required=True)
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Play every named field, and write the trajectories to one file."""
    args = parse_args(argv)
    reader = build_reader()
    seeds = viable_seeds(WORLD, args.worlds, args.seed_start)
    traced: list[GameTrace] = []
    for name, names in named_fields(args.fields):
        found = play_field(
            reader,
            name,
            names,
            seeds,
            args.worlds,
            args.sample,
            args.chunk,
            args.workers,
            args.threads,
        )
        traced.extend(found)
        print(f"{name}: {len(found)} games", flush=True)
    report = {
        "world": {
            "width": WORLD.width,
            "height": WORLD.height,
            "faction_count": WORLD.faction_count,
            "tick_limit": WORLD.tick_limit,
        },
        "fields": {name: list(names) for name, names in named_fields(args.fields)},
        "seeds": list(seeds),
        "sample_interval": args.sample,
        "count_precision": reader.precision,
        "games": [trace.as_dict() for trace in traced],
    }
    args.out.write_text(json.dumps(report), encoding="utf-8")
    print(f"wrote {len(traced)} games to {args.out}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
