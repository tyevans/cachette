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

# The world count belongs to the trainer, not to the worker count

By default this script gives each worker one world. **The trainer never runs
that shape.** A generation holds one world for every pair of a candidate and
a seed, so the world count is the population times the seeds whatever the
worker count is. Pass `--worlds` with that number to measure what a training
run does. A sweep that leaves the default measures the script instead.

The difference is not small. The batch crosses into the engine once for each
tick, and it waits for the slowest world at every crossing. A world count
that divides evenly by the worker count wastes little at that wait. A world
count that does not divides into a ragged tail, and the tail sets the cost.

# One process of many workers against many processes of few workers

The same cores hold either shape. One process of sixty-four workers keeps
one batch, one barrier, and one Python interpreter, and the interpreter picks
the actions for every world between the decisions while the workers wait.
Four processes of sixteen workers keep four of each, and the four run at
once.

Pass `--shape` to measure both, such as `1x64,2x32,4x16`. Every process of a
shape builds its own worlds and then waits, so they all measure one window.
The row reports what the whole machine reached.

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
import subprocess
import sys
import tempfile
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
    """What one shape reached.

    A shape is a process count and a worker count. One process with sixty
    four workers and four processes with sixteen workers each hold the same
    cores, and they do not reach the same throughput.
    """

    workers: int
    worlds: int
    decisions: int
    ticks: int
    seconds: float
    processes: int = 1

    @property
    def total_workers(self) -> int:
        """Return how many workers the whole shape holds."""
        return self.processes * self.workers

    @property
    def ticks_per_second(self) -> float:
        """Return the simulated ticks a second the whole shape reached."""
        return self.ticks / self.seconds if self.seconds else 0.0

    @property
    def ticks_per_second_per_worker(self) -> float:
        """Return the ticks a second that one worker contributed."""
        total = self.total_workers
        return self.ticks_per_second / total if total else 0.0

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
    barrier: Path | None = None,
) -> Measurement:
    """Step a batch of worlds, and count the ticks and the seconds exactly.

    The batch drops a world whose episode has ended, so the live count falls
    as the run goes on. The tick count multiplies the live count by the
    decision interval at every crossing, so it stays exact.

    **The barrier makes concurrent processes measure one window.** Building
    the worlds takes longer than the measurement, so processes that start
    together do not measure together. Each process builds its worlds, reports
    that it is ready, and waits. The parent starts them all at once. Without
    this, an early process measures a machine the other processes have not
    loaded yet, and the figure is too high.
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
    if barrier is not None:
        wait_at_barrier(barrier)
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


# How long a process waits for the others to build their worlds. A build of a
# few hundred worlds takes about a minute, and a process that waits longer
# than this has lost a sibling rather than found a slow one.
BARRIER_TIMEOUT_SECONDS = 600.0


def wait_at_barrier(barrier: Path) -> None:
    """Report that this process is ready, then wait for the start signal.

    The parent counts the ready files and writes the start file. This process
    polls for the start file. A poll of ten milliseconds costs nothing against
    a measurement of several seconds.
    """
    ready = barrier / f"ready.{os.getpid()}"
    ready.write_text("ready\n")
    start = barrier / "start"
    deadline = time.monotonic() + BARRIER_TIMEOUT_SECONDS
    while not start.exists():
        if time.monotonic() >= deadline:
            message = (
                f"the start signal never came in {BARRIER_TIMEOUT_SECONDS} seconds"
            )
            raise RuntimeError(message)
        time.sleep(0.01)


def measure_shape(
    config: EnvConfig,
    worlds: int,
    workers: int,
    decisions: int,
    processes: int,
) -> Measurement:
    """Run one shape, and return what the whole machine reached.

    One process measures itself. Two or more processes each measure
    themselves, and this function adds their ticks and divides by the longest
    of them. The longest is the honest denominator, because the machine held
    every process until the last one finished.
    """
    if processes <= 1:
        return measure(config, worlds, workers, decisions)

    with tempfile.TemporaryDirectory(prefix="cachette-probe-") as name:
        barrier = Path(name)
        children = []
        for _ in range(processes):
            command = [
                sys.executable,
                str(Path(__file__).resolve()),
                "--child-of",
                str(barrier),
                "--width",
                str(config.width),
                "--height",
                str(config.height),
                "--factions",
                str(config.faction_count),
                "--decision-interval",
                str(config.decision_interval),
                "--worlds",
                str(worlds),
                "--workers",
                str(workers),
                "--decisions",
                str(decisions),
            ]
            children.append(
                subprocess.Popen(
                    command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
                )
            )

        # Every child builds its worlds before it reports ready. The start
        # file goes out only when they all have, so they measure one window.
        deadline = time.monotonic() + BARRIER_TIMEOUT_SECONDS
        while len(list(barrier.glob("ready.*"))) < processes:
            if any(child.poll() not in (None, 0) for child in children):
                break
            if time.monotonic() >= deadline:
                ready = len(list(barrier.glob("ready.*")))
                message = f"only {ready} of {processes} children were ready"
                raise RuntimeError(message)
            time.sleep(0.05)
        (barrier / "start").write_text("go\n")

        ticks = 0
        seconds = 0.0
        taken = 0
        for child in children:
            out, err = child.communicate()
            if child.returncode != 0:
                message = f"a probe process failed: {err.strip()[-400:]}"
                raise RuntimeError(message)
            row = json.loads(out)
            ticks += int(row["ticks"])
            seconds = max(seconds, float(row["seconds"]))
            taken = max(taken, int(row["decisions"]))

    return Measurement(
        workers=workers,
        worlds=worlds,
        decisions=taken,
        ticks=ticks,
        seconds=seconds,
        processes=processes,
    )


def parse_shapes(text: str, cores: int) -> list[tuple[int, int]]:
    """Read a shape list such as ``1x64,4x16`` as process and worker pairs."""
    shapes: list[tuple[int, int]] = []
    for part in text.split(","):
        part = part.strip()
        if not part:
            continue
        processes, _, workers = part.partition("x")
        if not workers:
            message = f"the shape {part} is not a process count times a worker count"
            raise ValueError(message)
        shapes.append((int(processes), int(workers)))
    for processes, workers in shapes:
        if processes * workers > cores:
            print(
                f"  warning: {processes}x{workers} asks for "
                f"{processes * workers} workers on {cores} cores",
                file=sys.stderr,
            )
    return shapes


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
        "processes\tworkers\tworlds\tdecisions\tticks\tseconds"
        "\tticks_per_second\tticks_per_second_per_worker\tdollars_per_million_ticks"
    )
    for row in measurements:
        lines.append(
            f"{row.processes}\t{row.workers}\t{row.worlds}\t{row.decisions}"
            f"\t{row.ticks}"
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
    ideal = last.total_workers / first.total_workers
    # **Two shapes of the same core count do not compare as a speedup.** The
    # ideal is then one, and an efficiency against it says nothing about
    # whether the work divides. The question there is which shape holds the
    # cores better, so the line names the best shape instead.
    if abs(ideal - 1.0) < 1e-9:
        best = max(measurements, key=lambda row: row.ticks_per_second)
        worst = min(measurements, key=lambda row: row.ticks_per_second)
        if worst.ticks_per_second <= 0:
            return "  A shape measured nothing."
        gain = best.ticks_per_second / worst.ticks_per_second
        return (
            f"  Every shape holds {first.total_workers} workers. "
            f"{best.processes}x{best.workers} reached the most at "
            f"{best.ticks_per_second:.1f} ticks a second, which is {gain:.2f} "
            f"times what {worst.processes}x{worst.workers} reached."
        )
    efficiency = speedup / ideal if ideal else 0.0
    verdict = (
        "the work divides" if efficiency >= 0.8 else "the batch step is a bottleneck"
    )
    return (
        f"  {first.total_workers} workers to {last.total_workers}: "
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
    parser.add_argument(
        "--shape",
        type=str,
        default="",
        help="the shapes to sweep, as a process count times a worker count, "
        "separated by commas, such as 1x64,2x32,4x16. It replaces --workers",
    )
    parser.add_argument(
        "--child-of",
        type=Path,
        default=None,
        help="run one measurement for a parent probe, wait at the barrier in "
        "this directory, and print the row as JSON. A person does not use this",
    )
    parser.add_argument("--out", type=Path, help="write the table here as well")
    parser.add_argument(
        "--json", action="store_true", help="write the rows as JSON and no table"
    )
    arguments = parser.parse_args()

    cores = os.cpu_count() or 1
    if arguments.workers and "," not in arguments.workers:
        single_workers = int(arguments.workers)
    else:
        single_workers = 0
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

    # A child runs one measurement, waits for its siblings, and reports the
    # row to the parent. It prints nothing else, because the parent reads its
    # whole output as JSON.
    if arguments.child_of is not None:
        worlds = arguments.worlds or single_workers or 1
        row = measure(
            config,
            worlds,
            single_workers or 1,
            arguments.decisions,
            barrier=arguments.child_of,
        )
        print(
            json.dumps(
                {
                    "workers": row.workers,
                    "worlds": row.worlds,
                    "decisions": row.decisions,
                    "ticks": row.ticks,
                    "seconds": row.seconds,
                }
            )
        )
        return 0

    if arguments.shape:
        shapes = parse_shapes(arguments.shape, cores)
    else:
        shapes = [(1, workers) for workers in counts]

    measurements: list[Measurement] = []
    for processes, workers in shapes:
        # The batch holds one world for each worker by default, so a worker
        # that has no world cannot make the figure look flat. A run that
        # measures the trainer names the world count the trainer uses, which
        # is the population times the seeds.
        worlds = arguments.worlds or workers
        print(
            f"  measuring {processes} process(es) of {workers} workers "
            f"over {worlds} worlds each",
            file=sys.stderr,
        )
        measurements.append(
            measure_shape(config, worlds, workers, arguments.decisions, processes)
        )

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
                            "processes": row.processes,
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
