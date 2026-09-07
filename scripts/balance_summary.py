"""Fold a sweep report into distributions, and never into a mean.

The sweep writes one record for each seed.[^1] This reads that file and gives,
for each figure, the smallest value, the three quartiles and the largest value
over the seed set. A mean hides the seed that matters, so no mean appears here.

References
----------
The sweep. ``scripts/balance_sweep.py``
"""

from __future__ import annotations

import argparse
import collections
import json
import pathlib
import sys

from cachette import stock_ceiling_of_one_settlement

WONDER_WORK = 2400

# A stock total wins no game, so there is no wealth bar to read a share
# against. The engine states the stock one settlement can hold, and this
# script reads that as the scale of the column. A copy here reported a share
# against a bar the engine no longer held.[^1] [^2]
#
# [^1]: Findings register, FND-551. ``docs/FINDINGS.md``
# [^2]: ADR-0174, a wonder is a win path and a stock total is not, decision D2.
#       ``docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md``
STOCK_SCALE_RAW = stock_ceiling_of_one_settlement()


def quantiles(values: list[int]) -> tuple[int, int, int, int, int]:
    """Give the smallest value, the three quartiles and the largest value."""
    ordered = sorted(values)
    last = len(ordered) - 1
    return tuple(ordered[round(last * share / 4)] for share in range(5))


def line(name: str, values: list[int], unit: str = "") -> str:
    """Write one row of the distribution table."""
    low, first, middle, third, high = quantiles(values)
    return f"{name:<28} {low:>8} {first:>8} {middle:>8} {third:>8} {high:>8} {unit}"


def main(argv: list[str] | None = None) -> int:
    """Read the sweep and write the distribution table."""
    parser = argparse.ArgumentParser(prog="balance_summary")
    parser.add_argument("json", type=pathlib.Path)
    args = parser.parse_args(argv)
    games = json.loads(args.json.read_text(encoding="utf-8"))["games"]

    paths = collections.Counter(game["path"] or "none" for game in games)
    seats = collections.Counter(str(game["winner"]) for game in games)
    lines = [
        f"seeds {len(games)}",
        "path shares: "
        + ", ".join(f"{k} {v}/{len(games)}" for k, v in sorted(paths.items())),
        "seat shares: "
        + ", ".join(f"{k} {v}/{len(games)}" for k, v in sorted(seats.items())),
        "",
        f"{'figure':<28} {'min':>8} {'q1':>8} {'median':>8} {'q3':>8} {'max':>8}",
        line("end tick", [game["tick"] for game in games]),
        line("units that fell", [game["fell"] for game in games]),
    ]
    last = [game["samples"][-1] for game in games]
    lines += [
        line("settlements at the end", [s["settlements"] for s in last]),
        line("largest population", [max(s["population"]) for s in last]),
        line("smallest population", [min(s["population"]) for s in last]),
        line("largest held tiles", [max(s["held"]) for s in last]),
        line("smallest held tiles", [min(s["held"]) for s in last]),
        line(
            "best store, percent",
            [max(s["store"]) * 100 // STOCK_SCALE_RAW for s in last],
        ),
        line(
            "best wonder, percent",
            [max(s["wonder"]) * 100 // WONDER_WORK for s in last],
        ),
    ]
    rows = (
        "campaigns_raised",
        "campaigns_won",
        "wars_declared",
        "projects_finished",
        "queue_produced",
        "upgrades_complete",
        "wonders_complete",
        "seats_filled",
    )
    for row in rows:
        lines.append(line(row, [s["census"].get(row, 0) for s in last]))

    # The tick at which the largest faction first reached each population, and
    # the tick at which the last sample still moved.
    def settled(game: dict) -> int:
        """Give the first tick at which the run reached its final shape."""
        samples = game["samples"]
        final = (samples[-1]["population"], samples[-1]["held"])
        for sample in samples:
            if (sample["population"], sample["held"]) == final:
                return sample["tick"]
        return samples[-1]["tick"]

    lines.append(line("tick the world settled", [settled(game) for game in games]))
    lines.append(
        line(
            "settled share of the run",
            [settled(game) * 100 // max(game["tick"], 1) for game in games],
        )
    )
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
