#!/usr/bin/env python3
"""Separate two causes of the throughput decay inside one pass.

A heartbeat line states the live world count and the ticks a second over the
window since the last line. The rate falls through a pass. Two things can
make it fall. The pool can run out of live worlds to give its workers, and
the tick itself can get dearer as a game develops.

The first is a schedule loss and a change can win it back. The second is the
cost of the simulation and no schedule touches it.

This script separates them. It models the batch as it is: worker ``w`` takes
the worlds ``w``, ``w + workers`` and so on, and one tick costs the time of
the worker that holds the most worlds. Under that model the time of one tick
is ``ceil(live / workers)`` times the cost of one world-tick. The implied
cost of one world-tick therefore follows from the rate and the live count.

A rate that falls while the implied cost stays flat is a schedule loss. A
rate that falls while the implied cost rises is the simulation getting
dearer.

# How to run it

Give the worker count of a generation, the worker count of a baseline pass,
and one or more training logs. The trainer writes the heartbeat lines that
this reads.[^1]

# References

[^1]: The population batch, the heartbeat line. `python/cachette/learn/rollout.py`
"""

from __future__ import annotations

import collections
import math
import re
import sys

PAT = re.compile(
    r"^\s+(?P<label>.+?) working\s+decisions\s+(?P<dec>\d+)\s+live\s+(?P<live>\d+)"
    r"/(?P<total>\d+)\s+ticks\s+(?P<ticks>\d+)\s+rate\s+(?P<rate>[\d.]+) t/s"
    r"\s+\[(?P<el>\d+)s\]$"
)


def workers_of(label: str, generation: int, baseline: int) -> int:
    """Return how many engine workers the pass behind one label held."""
    if "generation" in label:
        return generation
    return baseline


def cost_of(live: int, rate: float, workers: int) -> float:
    """Return the implied cost of one world-tick, in milliseconds.

    One tick of the batch costs the time of the worker that holds the most
    worlds, which is ``ceil(live / workers)`` world-ticks. The rate divided
    by that count therefore gives the cost of one world-tick.
    """
    return 1000.0 * live / (rate * math.ceil(live / workers))


def load_of(live: int, workers: int) -> float:
    """Return the share of the workers that hold a world through one tick.

    A worker that holds fewer worlds than the busiest worker finishes early
    and waits. This is one minus that waste.
    """
    return live / (workers * math.ceil(live / workers))


def main() -> None:
    """Read the logs named on the command line and print one table for each pass.

    The first argument is the worker count of a generation. The second is the
    worker count of a baseline pass. The rest are the logs to read.
    """
    generation = int(sys.argv[1])
    baseline = int(sys.argv[2])
    rows = collections.defaultdict(list)
    for path in sys.argv[3:]:
        with open(path, errors="replace") as handle:
            for line in handle:
                match = PAT.match(line.rstrip("\n"))
                if match:
                    rows[match["label"]].append(
                        (
                            int(match["dec"]),
                            int(match["live"]),
                            int(match["total"]),
                            float(match["rate"]),
                            int(match["el"]),
                        )
                    )
    for label, seq in sorted(rows.items()):
        # A window that ran no tick states no rate, and it divides nothing.
        seq = sorted(row for row in seq if row[1] > 0 and row[3] > 0.0)
        if len(seq) < 3:
            continue
        workers = workers_of(label, generation, baseline)
        print(f"\n{label}  worlds {seq[0][2]}  workers {workers}")
        print(
            f"  {'elapsed':>8s} {'live':>5s} {'rate':>8s} {'rounds':>6s} "
            f"{'ms/world-tick':>13s} {'util':>6s}"
        )
        for _, live, _, rate, elapsed in seq:
            print(
                f"  {elapsed:8d} {live:5d} {rate:8.1f} "
                f"{math.ceil(live / workers):6d} "
                f"{cost_of(live, rate, workers):13.2f} "
                f"{load_of(live, workers):6.2f}"
            )
        first = seq[0]
        last = seq[-1]
        fell = last[3] / first[3]
        schedule = load_of(last[1], workers) / load_of(first[1], workers)
        dearer = cost_of(last[1], last[3], workers) / cost_of(
            first[1], first[3], workers
        )
        print(
            f"  rate fell to {fell:.2f}. "
            f"The schedule accounts for {schedule:.2f}. "
            f"The tick got dearer by {dearer:.2f}."
        )


if __name__ == "__main__":
    main()
