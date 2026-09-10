#!/usr/bin/env python3
"""Summarise the trajectories one probe recorded, as distributions.

A mean hides the case a threshold must be set against. A few games that come
close and many that do not read as the same number a uniform middle gives.
Every figure here is therefore a quantile, and every share carries the
standard error of the sample that produced it.

The companion tool plays the games and records every followed quantity of
every faction at a fixed tick interval.[^1] This tool reads that file and
states four things.

# The ending

The share of the games each win path ended, with the standard error of that
share, and the spread of the end tick of the games each path ended.

# The reach

For each quantity, the value each faction held when its game ended, and the
value the leading faction of that game held. The leader is the highest value
over the factions of the game, which the tool computes rather than reads, so
the leader of a quantity the engine publishes no leader for is still stated.

# The saturation

How far into a game a quantity arrives. For each game the tool takes the
leader value at each sample, and finds the first sample at which the leader
stood at nine tenths of the value it ended at. It reports that tick as a
fraction of the length of the game. **A quantity whose fraction is small
arrives early and stays**, and a threshold on it decides the game long before
the game is over.

# The trend

The leader value at fixed ticks, so a reader sees the shape rather than one
summary of it.

References
----------
[^1]: The trajectory probe. ``scripts/win_quantity_trajectories.py``
"""

from __future__ import annotations

import argparse
import json
import math
from itertools import pairwise
from pathlib import Path
from typing import TYPE_CHECKING, Any

import numpy as np

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

# The quantiles every distribution in this report states.
QUANTILES = (0.1, 0.5, 0.9)

# The share of its final value a quantity must reach before this tool calls it
# arrived. A quantity that reaches nine tenths of its end value early is a
# quantity that saturates.
ARRIVED = 0.9

# The ticks the trend table reads the leader at.
TREND_TICKS = (500, 1000, 2000, 3000, 4000, 5000, 6000)


def spread(values: Sequence[float]) -> dict[str, float]:
    """Return the quantiles of a set of numbers, and its highest value.

    A set of no number reads zero everywhere, and the count says so.
    """
    if not values:
        return {"count": 0, "p10": 0.0, "median": 0.0, "p90": 0.0, "max": 0.0}
    held = np.asarray(values, dtype=float)
    low, mid, high = np.quantile(held, QUANTILES)
    return {
        "count": int(held.size),
        "p10": float(low),
        "median": float(mid),
        "p90": float(high),
        "max": float(held.max()),
    }


def share_error(count: int, total: int) -> float:
    """Return the standard error of a share, from the share itself.

    A share of zero gives an error of zero, and that reading is too narrow.
    A reader that needs an interval at a share of zero reads the rule of
    three instead, and this tool states the count beside every share.
    """
    if total <= 0:
        return 0.0
    share = count / total
    return math.sqrt(share * (1.0 - share) / total)


def leader_series(game: dict[str, Any], group: str, name: str) -> list[float]:
    """Return the highest value over the factions, at each sample of one game."""
    return [max(sample[group][name]) for sample in game["samples"]]


def own_values(game: dict[str, Any], group: str, name: str) -> list[float]:
    """Return the last value of each faction of one game."""
    return list(game["samples"][-1][group][name])


def arrival_of(series: Sequence[float], ticks: Sequence[int]) -> float | None:
    """Return where in a game a series first stood at nine tenths of its end.

    The answer is a fraction of the length of the game. A series that ends at
    zero has no arrival, because every point of it is already at its end
    value and the fraction would state that the quantity arrived at the first
    tick.
    """
    if not series or series[-1] <= 0.0:
        return None
    bar = ARRIVED * series[-1]
    last = ticks[-1]
    if last <= 0:
        return None
    for tick, value in zip(ticks, series, strict=True):
        if value >= bar:
            return tick / last
    return 1.0


def rise_share(series: Sequence[float]) -> float | None:
    """Return the share of the steps of a series that went up.

    A quantity that accumulates goes up at nearly every step. A quantity that
    a contest moves goes up at some steps and down at others. **A step that
    holds level counts as neither**, because a quantity that stands still says
    nothing about the direction it would move in.

    A series of fewer than two samples has no step, and reads nothing.
    """
    steps = [after - before for before, after in pairwise(series)]
    moved = [step for step in steps if step != 0.0]
    if not moved:
        return None
    return sum(1 for step in moved if step > 0.0) / len(moved)


def trend_at(games: Sequence[dict[str, Any]], group: str, name: str) -> dict[str, Any]:
    """Return the median leader value at each fixed tick of the trend table.

    A game that ended before a tick contributes nothing at that tick, and the
    row states how many games it read.
    """
    rows: dict[str, Any] = {}
    for want in TREND_TICKS:
        held: list[float] = []
        for game in games:
            ticks = [sample["tick"] for sample in game["samples"]]
            if ticks[-1] < want:
                continue
            series = leader_series(game, group, name)
            index = max(i for i, tick in enumerate(ticks) if tick <= want)
            held.append(series[index])
        rows[str(want)] = {
            "games": len(held),
            "median": float(np.median(held)) if held else 0.0,
        }
    return rows


def quantity_rows(
    games: Sequence[dict[str, Any]], group: str, names: Sequence[str]
) -> dict[str, Any]:
    """Return the reach, the arrival and the trend of each quantity."""
    rows: dict[str, Any] = {}
    for name in names:
        own: list[float] = []
        leader: list[float] = []
        arrivals: list[float] = []
        rises: list[float] = []
        for game in games:
            own.extend(own_values(game, group, name))
            series = leader_series(game, group, name)
            leader.append(series[-1])
            ticks = [sample["tick"] for sample in game["samples"]]
            found = arrival_of(series, ticks)
            if found is not None:
                arrivals.append(found)
            rose = rise_share(series)
            if rose is not None:
                rises.append(rose)
        rows[name] = {
            "own": spread(own),
            "leader": spread(leader),
            "arrival": spread(arrivals),
            "rise": spread(rises),
            "trend": trend_at(games, group, name),
        }
    return rows


def ending_rows(games: Sequence[dict[str, Any]]) -> dict[str, Any]:
    """Return the share of the games each path ended, and the end tick spread."""
    paths = sorted({game["path"] for game in games})
    rows: dict[str, Any] = {}
    for path in paths:
        held = [game for game in games if game["path"] == path]
        rows[path] = {
            "games": len(held),
            "share": len(held) / len(games) if games else 0.0,
            "share_error": share_error(len(held), len(games)),
            "end_tick": spread([game["end_tick"] for game in held]),
        }
    return rows


def winner_rows(games: Sequence[dict[str, Any]]) -> dict[str, Any]:
    """Return the games each named player sat in, and the games it won."""
    rows: dict[str, dict[str, Any]] = {}
    for game in games:
        for name in game["seats"]:
            row = rows.setdefault(name, {"games": 0, "wins": 0})
            row["games"] += 1
        if game["winner"] is not None:
            rows[game["winner"]]["wins"] += 1
    for name, row in rows.items():
        row["share"] = row["wins"] / row["games"] if row["games"] else 0.0
        row["share_error"] = share_error(row["wins"], row["games"])
        rows[name] = row
    return rows


def summarise(
    report: dict[str, Any], subset: Sequence[dict[str, Any]]
) -> dict[str, Any]:
    """Return every table this tool states, over one subset of the games."""
    if not subset:
        return {"games": 0}
    share_names = sorted(subset[0]["samples"][0]["shares"])
    count_names = sorted(subset[0]["samples"][0]["counts"])
    return {
        "games": len(subset),
        "total_ticks": sum(int(game["end_tick"]) for game in subset),
        "end_tick": spread([game["end_tick"] for game in subset]),
        "reached_limit": sum(1 for game in subset if game["reached_limit"]),
        "endings": ending_rows(subset),
        "players": winner_rows(subset),
        "shares": quantity_rows(subset, "shares", share_names),
        "counts": quantity_rows(subset, "counts", count_names),
    }


def slices(report: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    """Return the subsets of the games this tool reports separately."""
    games = report["games"]
    found: dict[str, list[dict[str, Any]]] = {"all": list(games)}
    for field in sorted({game["field"] for game in games}):
        found[f"field:{field}"] = [game for game in games if game["field"] == field]
    for path in sorted({game["path"] for game in games}):
        found[f"path:{path}"] = [game for game in games if game["path"] == path]
    return found


def say(name: str, table: dict[str, Any]) -> None:
    """Print one subset of the report, as lines a reader can scan."""
    print(f"\n===== {name}  games={table['games']}")
    if not table["games"]:
        return
    tick = table["end_tick"]
    print(
        f"end tick  p10={tick['p10']:.0f} median={tick['median']:.0f} "
        f"p90={tick['p90']:.0f} max={tick['max']:.0f}  "
        f"at limit={table['reached_limit']}  total ticks={table['total_ticks']}"
    )
    for path, row in table["endings"].items():
        print(
            f"  ended {path:12s} {row['games']:4d}  share {row['share']:.3f} "
            f"+- {row['share_error']:.3f}  "
            f"end tick median {row['end_tick']['median']:.0f}"
        )
    for player, row in sorted(table["players"].items()):
        print(
            f"  player {player:10s} games {row['games']:4d} wins {row['wins']:4d} "
            f"share {row['share']:.3f} +- {row['share_error']:.3f}"
        )
    for group in ("shares", "counts"):
        print(f"  -- {group}")
        for quantity, row in table[group].items():
            own = row["own"]
            lead = row["leader"]
            arrive = row["arrival"]
            print(
                f"  {quantity:30s} own {own['p10']:9.3f}/{own['median']:9.3f}/"
                f"{own['p90']:9.3f}/{own['max']:9.3f}  lead "
                f"{lead['p10']:9.3f}/{lead['median']:9.3f}/{lead['p90']:9.3f}/"
                f"{lead['max']:9.3f}  arrive {arrive['median']:.2f}"
                f"  rise {row['rise']['median']:.2f}"
            )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Read the arguments of one run of this tool."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--trend", nargs="*", default=[])
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Read one trajectory file and print every distribution it holds."""
    args = parse_args(argv)
    report = json.loads(args.source.read_text(encoding="utf-8"))
    tables = {name: summarise(report, held) for name, held in slices(report).items()}
    for name, table in tables.items():
        say(name, table)
    for wanted in args.trend:
        group, _, quantity = wanted.partition(".")
        print(f"\n----- trend {wanted}")
        for name, table in tables.items():
            if not table["games"] or quantity not in table[group]:
                continue
            row = table[group][quantity]["trend"]
            line = " ".join(
                f"{tick}:{cell['median']:.3f}({cell['games']})"
                for tick, cell in row.items()
            )
            print(f"  {name:16s} {line}")
    if args.out is not None:
        args.out.write_text(json.dumps(tables, indent=1), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
