#!/usr/bin/env python3
"""Read a training log, and say whether the run is worth its remaining cost.

A training run on a rented machine spends money for every minute it lives.
The owner of the run needs four answers while it runs, and a log of raw
lines answers none of them:

1. Is the policy better than the opponent it plays?
2. Is the search still searching, or has it stopped?
3. How much has this cost so far, and how much per generation?
4. How long is left?

This module reads what the trainer prints and answers those four. It opens
no socket and starts no process, so a person runs it against a finished log
on a laptop the same way a launcher runs it against a live one.

# The mean of a generation is not progress

The seed set moves at every generation by design, so a rising mean may only
mean that the new worlds are easier. This module reports the mean as one
column of the table and never as the headline, and the summary says so.

**The held-out figure is the one that decides the spend.** The trainer plays
the built-in controller on the held-out seeds before it trains anything, and
writes that row to the report. That row is the bar. The validation figure,
which the trainer takes every few generations on a third seed set, is the
only in-flight measure of the centre, so this module carries it beside the
bar.

# A spread of zero means the search stopped

Each generation ranks its candidates by score. When the highest and the
lowest score the same, the ranking carries no information and the update
that follows it is noise. A run in that state keeps billing and learns
nothing, so this module names it and the launcher can end the run on it.

# What the parser reads

The trainer prints one line for each generation, one heading for each
strategy, and one row for each baseline at the end of a strategy. Those
lines are the interface this module reads. A line it cannot parse is
skipped rather than fatal, so a trainer that gains a column keeps working
here.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

# One generation line. Every field after the best score is optional, because
# the trainer prints the spread, the win share and the validation figure only
# when it has them.
GENERATION = re.compile(
    r"^\s+(?P<name>\S+) generation\s+(?P<generation>\d+)\s+"
    r"mean\s+(?P<mean>-?[\d.]+)\s+"
    r"best\s+(?P<best>-?[\d.]+)"
    r"(?:\s+spread\s+(?P<spread>-?[\d.]+))?"
    r"(?:\s+won\s+(?P<won>-?[\d.]+))?"
    r"(?:\s+ticks\s+(?P<ticks>\d+))?"
    r"(?:\s+valid\s+(?P<validation>-|-?[\d.]+))?"
    r"\s+\[(?P<seconds>[\d.]+)s\]"
)

# The heading that starts a strategy, for example `=== conquer (linear) ===`.
HEADING = re.compile(r"^=== (?P<name>[\w-]+)(?: \((?P<kind>\w+)\))? ===\s*$")

# A baseline row at the end of a strategy.
BASELINE = re.compile(
    r"^\s+(?P<label>trained|untrained|random|controller)\s+"
    r"return\s+(?P<return>-?[\d.]+)\s+"
    r"tiles\s+(?P<tiles>-?[\d.]+)\s+"
    r"won\s+(?P<won>-?[\d.]+)\s+"
    r"lost\s+(?P<lost>-?[\d.]+)"
)

# The controller baseline, printed once before any training, as a dictionary.
CONTROLLER = re.compile(r"^\s+controller (?P<body>\{.*\})\s*$")

# The number of generations behind the collapse test, and the spread under
# which a generation counts as collapsed. A spread is a difference of two
# returns, so the threshold is in the units of the reward and a caller that
# changes the reward scale must change it too.
COLLAPSE_WINDOW = 3
COLLAPSE_SPREAD = 1.0


@dataclass
class Generation:
    """What one generation scored, and when it ended."""

    name: str
    generation: int
    mean: float
    best: float
    seconds: float
    spread: float | None = None
    # How many world-ticks the generation ran. One world stepped one tick is
    # one. A run started before the trainer counted them leaves this unset,
    # and a reader must not read that as zero work.
    ticks: int | None = None
    won: float | None = None
    validation: float | None = None


@dataclass
class Strategy:
    """One policy under training, and what it has scored so far."""

    name: str
    kind: str = ""
    generations: list[Generation] = field(default_factory=list)
    baselines: dict[str, dict[str, float]] = field(default_factory=dict)

    @property
    def last_validation(self) -> float | None:
        """Return the newest validation figure, or nothing if it has none."""
        for row in reversed(self.generations):
            if row.validation is not None:
                return row.validation
        return None

    @property
    def collapsed(self) -> bool:
        """Say whether the search has stopped.

        The last few generations decide it. A run collapses when every
        candidate of each of them scored the same, because the ranking that
        drives the update then ranks nothing.
        """
        recent = [row.spread for row in self.generations[-COLLAPSE_WINDOW:]]
        if len(recent) < COLLAPSE_WINDOW or any(value is None for value in recent):
            return False
        return all(value < COLLAPSE_SPREAD for value in recent if value is not None)


@dataclass
class Progress:
    """Everything one log says about one run."""

    strategies: list[Strategy] = field(default_factory=list)
    controller: dict[str, float] = field(default_factory=dict)

    @property
    def generations_done(self) -> int:
        """Return how many generations the whole run has finished."""
        return sum(len(strategy.generations) for strategy in self.strategies)

    @property
    def elapsed_seconds(self) -> float:
        """Return the wall clock the run has used.

        The trainer counts from zero at each strategy, so the total is the
        sum of the last figure of each one.
        """
        return sum(
            strategy.generations[-1].seconds
            for strategy in self.strategies
            if strategy.generations
        )

    @property
    def collapsed(self) -> bool:
        """Say whether the strategy under training has stopped searching."""
        return bool(self.strategies) and self.strategies[-1].collapsed


def parse(text: str) -> Progress:
    """Read a training log, and return what it says."""
    progress = Progress()
    current: Strategy | None = None

    for line in text.splitlines():
        heading = HEADING.match(line)
        if heading:
            name = heading.group("name")
            if name == "controller baseline":  # pragma: no cover - unreachable
                continue
            current = Strategy(name=name, kind=heading.group("kind") or "")
            progress.strategies.append(current)
            continue

        # The controller baseline sits under its own heading, which the
        # pattern above reads as a strategy named `controller`. The row that
        # follows it is a dictionary rather than a table, so it is read here
        # and the empty strategy is dropped below.
        controller = CONTROLLER.match(line)
        if controller and not progress.controller:
            try:
                progress.controller = json.loads(
                    controller.group("body").replace("'", '"')
                )
            except json.JSONDecodeError:
                pass
            continue

        row = GENERATION.match(line)
        if row and current is not None:
            validation = row.group("validation")
            current.generations.append(
                Generation(
                    name=row.group("name"),
                    generation=int(row.group("generation")),
                    mean=float(row.group("mean")),
                    best=float(row.group("best")),
                    seconds=float(row.group("seconds")),
                    spread=(
                        float(row.group("spread"))
                        if row.group("spread") is not None
                        else None
                    ),
                    won=(
                        float(row.group("won"))
                        if row.group("won") is not None
                        else None
                    ),
                    ticks=(
                        int(row.group("ticks"))
                        if row.group("ticks") is not None
                        else None
                    ),
                    validation=(
                        None if validation in (None, "-") else float(str(validation))
                    ),
                )
            )
            continue

        baseline = BASELINE.match(line)
        if baseline and current is not None:
            current.baselines[baseline.group("label")] = {
                "return": float(baseline.group("return")),
                "held_tiles": float(baseline.group("tiles")),
                "won": float(baseline.group("won")),
                "lost": float(baseline.group("lost")),
            }

    # The heading of the controller baseline holds a space, so the pattern
    # above never matched it and no empty strategy was made for it. A
    # strategy that holds nothing at all is dropped, because a heading with
    # no row under it says only that a strategy started.
    progress.strategies = [
        strategy
        for strategy in progress.strategies
        if strategy.generations or strategy.baselines
    ]
    return progress


def read_report(path: Path) -> dict[str, object]:
    """Read the report the trainer writes, or return nothing if it has none."""
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return loaded if isinstance(loaded, dict) else {}


def spend(elapsed_seconds: float, price_per_hour: float) -> float:
    """Return the dollars an instance at this price has cost by now.

    A spot instance bills by the second after the first minute, so the
    elapsed time multiplied by the hourly rate is the charge and not an
    approximation of it. The figure excludes the root volume, which costs
    about one cent for a run of a few hours.
    """
    return elapsed_seconds / 3600.0 * price_per_hour


def projection(
    progress: Progress,
    price_per_hour: float,
    total_generations: int,
) -> dict[str, float]:
    """Return the cost so far, the cost of one generation, and what is left.

    The rate comes from the generations this run has already finished, so a
    machine that runs faster or slower than the estimate corrects itself
    after the first few of them.
    """
    done = progress.generations_done
    elapsed = progress.elapsed_seconds
    dollars = spend(elapsed, price_per_hour)
    per_generation = elapsed / done if done else 0.0
    remaining = max(0, total_generations - done)
    return {
        "generations_done": float(done),
        "generations_remaining": float(remaining),
        "elapsed_seconds": elapsed,
        "seconds_per_generation": per_generation,
        "dollars_spent": dollars,
        "dollars_per_generation": dollars / done if done else 0.0,
        "seconds_remaining": per_generation * remaining,
        "dollars_remaining": spend(per_generation * remaining, price_per_hour),
        "dollars_total": dollars + spend(per_generation * remaining, price_per_hour),
    }


def clock(seconds: float) -> str:
    """Return a duration as hours and minutes."""
    minutes = int(seconds // 60)
    return f"{minutes // 60}h{minutes % 60:02d}m"


def render(
    progress: Progress,
    price_per_hour: float,
    total_generations: int,
    instance_type: str = "",
    zone: str = "",
) -> str:
    """Return the dashboard a person reads to decide whether to keep paying."""
    money = projection(progress, price_per_hour, total_generations)
    lines: list[str] = []
    machine = " ".join(part for part in (instance_type, zone) if part)
    lines.append(f"=== training run {machine} at ${price_per_hour:.4f}/hr ===")

    # 1. The bar. The controller is the opponent, and beating it is the only
    # result that says the spend bought something.
    bar = progress.controller
    if bar:
        lines.append(
            f"  bar to beat   controller wins {bar.get('won', 0.0):.3f} "
            f"return {bar.get('return', 0.0):9.1f} "
            f"on {int(bar.get('episodes', 0))} held-out seeds"
        )
    else:
        lines.append("  bar to beat   not measured yet")

    # 2. What each strategy has reached. A finished strategy has a held-out
    # row, which is the figure that decides. A running one has only the
    # validation figure, which is weaker and is labelled as such.
    for strategy in progress.strategies:
        rows = strategy.generations
        label = f"{strategy.name} ({strategy.kind})" if strategy.kind else strategy.name
        lines.append(f"  --- {label}: {len(rows)} generations")
        trained = strategy.baselines.get("trained")
        controller = strategy.baselines.get("controller") or bar
        if trained:
            verdict = "BEATS the controller"
            if controller and trained["won"] <= controller.get("won", 0.0):
                verdict = "loses to the controller"
            lines.append(
                f"      held out   wins {trained['won']:.3f} "
                f"return {trained['return']:9.1f}   {verdict}"
            )
        else:
            latest = strategy.last_validation
            lines.append(
                "      held out   not measured until this strategy ends"
                + (
                    f"; validation {latest:.1f}"
                    if latest is not None
                    else "; no validation yet"
                )
            )
        if rows:
            recent = rows[-5:]
            lines.append("      generation  mean       best       spread   won")
            for row in recent:
                spread = "     -" if row.spread is None else f"{row.spread:9.1f}"
                won = "    -" if row.won is None else f"{row.won:5.2f}"
                lines.append(
                    f"      {row.generation:10d} {row.mean:10.1f} "
                    f"{row.best:10.1f} {spread} {won}"
                )
        if strategy.collapsed:
            lines.append(
                f"      SEARCH STOPPED: the spread was under {COLLAPSE_SPREAD} "
                f"for {COLLAPSE_WINDOW} generations. Every candidate scored the"
            )
            lines.append(
                "      same, so the update carries no information. Stop the run."
            )

    # 3. The money, and 4. the time. The two questions he asked to be able
    # to answer stand together at the bottom, where a reader ends.
    lines.append("  --- cost and time")
    lines.append(
        f"      spent      ${money['dollars_spent']:.2f} over "
        f"{clock(money['elapsed_seconds'])}, "
        f"${money['dollars_per_generation']:.3f} for each generation"
    )
    lines.append(
        f"      remaining  {int(money['generations_remaining'])} generations, "
        f"about {clock(money['seconds_remaining'])} and "
        f"${money['dollars_remaining']:.2f}"
    )
    lines.append(f"      full run   ${money['dollars_total']:.2f} at this rate")
    lines.append("  The mean above is not a learning curve. The seed set moves every")
    lines.append("  generation, so only the held-out row and the bar compare.")
    return "\n".join(lines)


def rows(progress: Progress, run_id: str) -> list[dict[str, object]]:
    """Return one record for each generation, for a store or a dashboard.

    The launcher writes these beside the log, so a finished run leaves a
    table and not only prose.
    """
    out: list[dict[str, object]] = []
    for strategy in progress.strategies:
        for row in strategy.generations:
            out.append(
                {
                    "run_id": run_id,
                    "strategy": strategy.name,
                    "kind": strategy.kind,
                    "generation": row.generation,
                    "mean": row.mean,
                    "best": row.best,
                    "spread": row.spread,
                    "won": row.won,
                    "validation": row.validation,
                    "seconds": row.seconds,
                }
            )
    return out


def main() -> int:
    """Read a log, and print the dashboard or the rows."""
    parser = argparse.ArgumentParser(description="Report the progress of a run.")
    parser.add_argument("log", type=Path, help="the log the trainer wrote")
    parser.add_argument(
        "--price",
        type=float,
        default=0.0,
        help="the dollars an hour the machine costs",
    )
    parser.add_argument(
        "--generations",
        type=int,
        default=0,
        help="how many generations the whole run will take",
    )
    parser.add_argument("--instance-type", type=str, default="")
    parser.add_argument("--zone", type=str, default="")
    parser.add_argument("--run-id", type=str, default="local")
    parser.add_argument(
        "--report",
        type=Path,
        help="the report file, which holds the controller bar before the log does",
    )
    parser.add_argument(
        "--rows",
        action="store_true",
        help="write one JSON record for each generation, and no dashboard",
    )
    parser.add_argument(
        "--collapsed",
        action="store_true",
        help="exit 3 when the search has stopped, and print nothing",
    )
    arguments = parser.parse_args()

    try:
        text = arguments.log.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        print(f"Could not read {arguments.log}: {error}", file=sys.stderr)
        return 1

    progress = parse(text)
    if arguments.report:
        report = read_report(arguments.report)
        controller = report.get("controller")
        if isinstance(controller, dict) and not progress.controller:
            progress.controller = controller

    if arguments.collapsed:
        return 3 if progress.collapsed else 0

    if arguments.rows:
        for row in rows(progress, arguments.run_id):
            print(json.dumps(row))
        return 0

    print(
        render(
            progress,
            arguments.price,
            arguments.generations,
            arguments.instance_type,
            arguments.zone,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
