"""Turn the sampled curves of a sweep into one saturation table.

The sweep records the whole curve of every quantity over a run.[^1] This script
reduces each curve to two numbers, and reports the spread of each number over
the seed set rather than its mean. **A mean hides the seed that matters.**

The two numbers are these.

**The ninety percent tick.** The first sampled tick at which a quantity has
made ninety percent of the whole move it makes in the run. A quantity that
reaches its ninety percent tick at 400 of 20000 ticks did all its work in the
first two percent of the run.

**The flat share.** The share of the sampled ticks at which the quantity sits
within five percent of its final value. A quantity with a flat share of 0.95
is at rest for almost the whole run.

A third number sits beside them. **The last move** is the last sampled tick at
which the quantity changed at all. Read it against the game end tick in the
header: a quantity whose last move is the game end stopped because the game
ended and the controllers stopped, and not because it ran out of room.

A quantity that never moves reports a ninety percent tick of 0 and a flat share
of 1, and the table marks it as one that never moved.

References
----------
The sweep. ``scripts/saturation_sweep.py``
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

TOLERANCE_NUMERATOR = 5
TOLERANCE_DENOMINATOR = 100
REACH_NUMERATOR = 9
REACH_DENOMINATOR = 10


def quantile(values: list[int], numerator: int, denominator: int) -> int:
    """Return the value at a rank of the sorted list, without a float."""
    if not values:
        return 0
    order = sorted(values)
    index = (len(order) - 1) * numerator // denominator
    return order[index]


def reach_tick(ticks: list[int], values: list[int]) -> int:
    """Return the first tick at which the curve made nine tenths of its move."""
    first = values[0]
    final = values[-1]
    move = final - first
    if move == 0:
        return 0
    for tick, value in zip(ticks, values, strict=True):
        if move > 0 and (value - first) * REACH_DENOMINATOR >= move * REACH_NUMERATOR:
            return tick
        if move < 0 and (value - first) * REACH_DENOMINATOR <= move * REACH_NUMERATOR:
            return tick
    return ticks[-1]


def last_move(ticks: list[int], values: list[int]) -> int:
    """Return the last tick at which the value changed from the sample before."""
    last = ticks[0]
    for index in range(1, len(values)):
        if values[index] != values[index - 1]:
            last = ticks[index]
    return last


def flat_share(values: list[int]) -> int:
    """Return the share in hundredths of samples within a twentieth of the end."""
    final = values[-1]
    width = max(abs(final), 1) * TOLERANCE_NUMERATOR
    inside = sum(
        1 for value in values if abs(value - final) * TOLERANCE_DENOMINATOR <= width
    )
    return inside * 100 // len(values)


def summarise(report: dict) -> list[dict]:
    """Reduce every quantity of every run to the two numbers."""
    runs = report["runs"]
    names = sorted(set(runs[0]["samples"][0]) - {"tick"})
    rows = []
    for name in names:
        reaches = []
        shares = []
        finals = []
        lasts = []
        moved = 0
        for run in runs:
            ticks = [s["tick"] for s in run["samples"]]
            values = [s[name] for s in run["samples"]]
            reaches.append(reach_tick(ticks, values))
            lasts.append(last_move(ticks, values))
            shares.append(flat_share(values))
            finals.append(values[-1])
            if values[-1] != values[0] or min(values) != max(values):
                moved += 1
        rows.append(
            {
                "name": name,
                "moved_in_seeds": moved,
                "seeds": len(runs),
                "reach_min": min(reaches),
                "reach_median": quantile(reaches, 1, 2),
                "reach_max": max(reaches),
                "last_median": quantile(lasts, 1, 2),
                "share_min": min(shares),
                "share_median": quantile(shares, 1, 2),
                "share_max": max(shares),
                "final_min": min(finals),
                "final_median": quantile(finals, 1, 2),
                "final_max": max(finals),
            }
        )
    rows.sort(key=lambda row: (row["reach_median"], row["name"]))
    return rows


def render(report: dict, rows: list[dict]) -> str:
    """Write the table, ordered by how early each quantity saturates."""
    ends = [run["end_tick"] for run in report["runs"]]
    named = [end for end in ends if end is not None]
    header = (
        f"{'quantity':<38} {'moved':>5} "
        f"{'t90 min':>8} {'t90 med':>8} {'t90 max':>8} {'last med':>8} "
        f"{'flat%min':>8} {'flat%med':>8} {'flat%max':>8} "
        f"{'end min':>12} {'end med':>12} {'end max':>12}"
    )
    lines = [
        f"extent {report['extent']}, factions {report['factions']}, "
        f"tick limit {report['tick_limit']}, sample every {report['sample']}, "
        f"seeds {len(report['runs'])}",
        f"game end fired in {len(named)} of {len(ends)} seeds, "
        f"ticks {min(named) if named else '-'} to {max(named) if named else '-'}, "
        f"median {quantile(named, 1, 2) if named else '-'}",
        "",
        "MOVED, ordered by the tick the median seed makes nine tenths of its move",
        header,
    ]

    def row_line(row: dict) -> str:
        return (
            f"{row['name']:<38} {row['moved_in_seeds']:>2}/{row['seeds']:<2} "
            f"{row['reach_min']:>8} {row['reach_median']:>8} {row['reach_max']:>8} "
            f"{row['last_median']:>8} "
            f"{row['share_min']:>8} {row['share_median']:>8} {row['share_max']:>8} "
            f"{row['final_min']:>12} {row['final_median']:>12} {row['final_max']:>12}"
        )

    lines.extend(row_line(row) for row in rows if row["moved_in_seeds"])
    lines.append("")
    lines.append("NEVER MOVED IN ANY SEED, so the subsystem behind it did nothing")
    lines.append(header)
    lines.extend(row_line(row) for row in rows if not row["moved_in_seeds"])
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    """Read the sweep report and write the table."""
    parser = argparse.ArgumentParser(prog="saturation_summary")
    parser.add_argument(
        "--json", type=pathlib.Path, default=pathlib.Path("target/saturation.json")
    )
    args = parser.parse_args(argv)
    report = json.loads(args.json.read_text(encoding="utf-8"))
    sys.stdout.write(render(report, summarise(report)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
