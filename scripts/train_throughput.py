#!/usr/bin/env python3
"""Measure how many simulated ticks a second the learner batch runs.

Every throughput figure this project holds comes from a development machine
on x86-64. The engine targets `aarch64-unknown-linux-gnu`, and the register
of costs says to measure on the target rather than at home, because the
development machines have a different cache line size and mislead on false
sharing.[^1] This script takes the measurement that a training run on the
target can carry back.

# What a tick is here, and why the world size belongs beside the figure

One tick is one step of one world. The batch steps every world that is still
running, so one crossing of the boundary advances `decision_interval` ticks
in each live world. This script counts them exactly: it multiplies the live
world count by the decision interval at every crossing and adds the result.
Nothing here is derived from a mean or from a reported duration.

**Ticks a second does not compare across world sizes.** A tick of a 48 by 48
world with three factions is not a tick of a 256 by 256 world, and the table
below names the extent on every row for that reason. The default here is the
world the learner trains on, because the question this answers is what a
training run costs.

# Ticks a second for each core says whether the parallelism is real

The batch takes a worker count. If sixty-four workers give four times what
sixteen give, the figure for each core is flat and the work divides. If it
does not, the batch step has a bottleneck, and that is worth knowing before
anybody rents a larger machine. The script therefore sweeps the worker count
rather than measuring one.

# Cost for each million ticks

Dollars an hour compares two prices. It does not compare two machines. Once
the throughput is known, the cost of a million ticks compares them directly,
and that is the figure that says whether a larger instance is worth its
price.

# References

[^1]: Target platform costs. `docs/reference/graviton-costs.md`
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
import time
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from cachette.learn.env import EnvConfig, VectorEnv, viable_seeds
from cachette.learn.reward import Weighting

# The world the learner trains on. A figure measured here answers what a
# training run costs, which is the question this script exists for.
DEFAULT_WIDTH = 48
DEFAULT_HEIGHT = 48
DEFAULT_FACTIONS = 3
DEFAULT_DECISION_INTERVAL = 10

# The scoring the probe runs under. The reward never changes the tick count,
# and it never changes what the engine does, so any weighting measures the
# same throughput. This one is the cheapest to read.
PROBE_WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=0.0, lost=0.0, drawn=0.0)


@dataclass(frozen=True)
class Measurement:
    """What one worker count reached."""

    workers: int
    worlds: int
    decisions: int
    ticks: int
    seconds: float

    @property
    def ticks_per_second(self) -> float:
        """Return the simulated ticks a second the whole batch reached."""
        return self.ticks / self.seconds if self.seconds else 0.0

    @property
    def ticks_per_second_per_worker(self) -> float:
        """Return the ticks a second that one worker contributed."""
        return self.ticks_per_second / self.workers if self.workers else 0.0

    def dollars_per_million_ticks(self, price_per_hour: float) -> float:
        """Return what a million ticks cost on a machine at this price."""
        rate = self.ticks_per_second
        if rate <= 0.0:
            return 0.0
        return price_per_hour / 3600.0 * (1_000_000.0 / rate)


def measure(
    config: EnvConfig,
    worlds: int,
    workers: int,
    decisions: int,
) -> Measurement:
    """Step a batch of worlds, and count the ticks and the seconds exactly.

    The batch drops a world whose episode has ended, so the live count falls
    as the run goes on. The tick count multiplies the live count by the
    decision interval at every crossing, so it stays exact.
    """
    vector = VectorEnv(config, PROBE_WEIGHTING, count=worlds, workers=workers)
    # **Not every seed builds a world a faction can start in.** The engine
    # refuses a seed where no faction found a place, so the probe asks the
    # trainer's own reader for seeds that work rather than counting from a
    # number. A world that refuses to build measures nothing.
    #
    # The reader returns the same seeds for the same configuration, so two
    # runs of the probe on one machine compare.
    vector.reset(viable_seeds(config, worlds, 1000))

    # The no-op is action zero, and it is legal in every world. A probe that
    # chose real verbs would measure the verbs of a policy that does not
    # exist yet.
    actions = [0] * worlds
    ticks = 0
    taken = 0
    started = time.perf_counter()
    for _ in range(decisions):
        if vector.done:
            break
        live = sum(1 for env in vector.envs if not env.done)
        vector.step(actions)
        ticks += live * config.decision_interval
        taken += 1
    seconds = time.perf_counter() - started
    return Measurement(
        workers=workers,
        worlds=worlds,
        decisions=taken,
        ticks=ticks,
        seconds=seconds,
    )


def machine_facts() -> dict[str, object]:
    """Return what the machine is, so a row says where it was taken."""
    facts: dict[str, object] = {
        "uname": " ".join(platform.uname()[:1] + platform.uname()[2:3]),
        "machine": platform.machine(),
        "cpu_count": os.cpu_count() or 0,
        "numpy": np.__version__,
        "python": platform.python_version(),
    }
    line = Path("/sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size")
    try:
        facts["cache_line_bytes"] = int(line.read_text().strip())
    except (OSError, ValueError):
        facts["cache_line_bytes"] = 0
    return facts


def table(
    measurements: list[Measurement],
    config: EnvConfig,
    price_per_hour: float,
    facts: dict[str, object],
) -> str:
    """Return the measurements as a tab separated table with its facts."""
    lines = [f"# {name}\t{value}" for name, value in facts.items()]
    lines.append(f"# extent\t{config.width}x{config.height}")
    lines.append(f"# faction_count\t{config.faction_count}")
    lines.append(f"# decision_interval\t{config.decision_interval}")
    lines.append(f"# price_per_hour\t{price_per_hour}")
    lines.append(
        "workers\tworlds\tdecisions\tticks\tseconds"
        "\tticks_per_second\tticks_per_second_per_worker\tdollars_per_million_ticks"
    )
    for row in measurements:
        lines.append(
            f"{row.workers}\t{row.worlds}\t{row.decisions}\t{row.ticks}"
            f"\t{row.seconds:.3f}\t{row.ticks_per_second:.1f}"
            f"\t{row.ticks_per_second_per_worker:.1f}"
            f"\t{row.dollars_per_million_ticks(price_per_hour):.6f}"
        )
    return "\n".join(lines)


def scaling(measurements: list[Measurement]) -> str:
    """Say whether the throughput divided over the workers.

    A perfect division holds the figure for each worker flat. The line below
    states the fall from the smallest worker count to the largest, which is
    the number that says whether a larger machine buys what it costs.
    """
    if len(measurements) < 2:
        return "  One worker count. Nothing here says whether the work divides."
    first, last = measurements[0], measurements[-1]
    if first.ticks_per_second <= 0 or first.workers <= 0:
        return "  The smallest worker count measured nothing."
    speedup = last.ticks_per_second / first.ticks_per_second
    ideal = last.workers / first.workers
    efficiency = speedup / ideal if ideal else 0.0
    verdict = "the work divides" if efficiency >= 0.8 else "the batch step is a bottleneck"
    return (
        f"  {first.workers} workers to {last.workers}: "
        f"{speedup:.2f} times the throughput against {ideal:.2f} times the workers, "
        f"so {efficiency * 100:.0f} percent of the ideal. {verdict}."
    )


def main() -> int:
    """Sweep the worker count, and print what each one reached."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--width", type=int, default=DEFAULT_WIDTH)
    parser.add_argument("--height", type=int, default=DEFAULT_HEIGHT)
    parser.add_argument("--factions", type=int, default=DEFAULT_FACTIONS)
    parser.add_argument(
        "--decision-interval", type=int, default=DEFAULT_DECISION_INTERVAL
    )
    parser.add_argument(
        "--worlds",
        type=int,
        default=0,
        help="how many worlds the batch holds. The default is the worker count",
    )
    parser.add_argument(
        "--decisions",
        type=int,
        default=20,
        help="how many crossings of the boundary to time",
    )
    parser.add_argument(
        "--workers",
        type=str,
        default="",
        help="the worker counts to sweep, separated by commas. "
        "The default halves the core count down to one",
    )
    parser.add_argument(
        "--price",
        type=float,
        default=0.0,
        help="the dollars an hour the machine costs",
    )
    parser.add_argument("--out", type=Path, help="write the table here as well")
    parser.add_argument(
        "--json", action="store_true", help="write the rows as JSON and no table"
    )
    arguments = parser.parse_args()

    cores = os.cpu_count() or 1
    if arguments.workers:
        counts = [int(part) for part in arguments.workers.split(",") if part.strip()]
    else:
        counts = []
        count = cores
        while count >= 1:
            counts.append(count)
            count //= 2
        counts.reverse()

    config = EnvConfig(
        width=arguments.width,
        height=arguments.height,
        faction_count=arguments.factions,
        seat=0,
        tick_limit=2500,
        horizon=2500 // arguments.decision_interval,
        decision_interval=arguments.decision_interval,
    )

    measurements: list[Measurement] = []
    for workers in counts:
        # The batch holds one world for each worker by default, so a worker
        # that has no world cannot make the figure look flat.
        worlds = arguments.worlds or workers
        print(f"  measuring {workers} workers over {worlds} worlds", file=sys.stderr)
        measurements.append(measure(config, worlds, workers, arguments.decisions))

    facts = machine_facts()
    if arguments.json:
        print(
            json.dumps(
                {
                    "facts": facts,
                    "extent": f"{config.width}x{config.height}",
                    "faction_count": config.faction_count,
                    "decision_interval": config.decision_interval,
                    "price_per_hour": arguments.price,
                    "rows": [
                        {
                            "workers": row.workers,
                            "worlds": row.worlds,
                            "decisions": row.decisions,
                            "ticks": row.ticks,
                            "seconds": row.seconds,
                            "ticks_per_second": row.ticks_per_second,
                            "ticks_per_second_per_worker": (
                                row.ticks_per_second_per_worker
                            ),
                            "dollars_per_million_ticks": (
                                row.dollars_per_million_ticks(arguments.price)
                            ),
                        }
                        for row in measurements
                    ],
                },
                indent=2,
            )
        )
        return 0

    rendered = table(measurements, config, arguments.price, facts)
    print(rendered)
    print(f"# world {config.width}x{config.height}, {config.faction_count} factions")
    print(scaling(measurements))
    if arguments.out:
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(rendered + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
