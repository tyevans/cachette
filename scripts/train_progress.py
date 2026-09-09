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
import time
from dataclasses import dataclass, field
from pathlib import Path

# One generation line, for example:
#   conquer generation  1 mean 884.5 best 1586.5 spread 1298.8 abs-spread
#   1298.8 won 0.06 ticks 852690 refused 0.00 valid 870.4 [1231s]
#
# **The fields are read by name, not by their order.** A pattern that fixed
# the order stopped matching every generation line the moment `abs-spread`
# appeared between the spread and the win share. The feed then reported a run
# with no generation, no elapsed time and no spend, rather than an error, and
# it reported that for every real log.
GENERATION = re.compile(
    r"^\s+(?P<name>\S+) generation\s+(?P<generation>\d+)\s+"
    r"(?P<body>.*?)\s*\[(?P<seconds>[\d.]+)s\]\s*$"
)

# One `name value` pair of the body. A value is a number or a bare `-`.
FIELD = re.compile(r"(?P<key>[a-z][a-z-]*)\s+(?P<value>-?[\d.]+|-)")

# The marker that separates a pass in flight from a pass that ended, for
# example:
#   wonder_rush generation  1 shard 1/2 working  decisions 202 live 214/512
#   ticks 804310 rate 1754.8 t/s [373s]
#
# **Read the marker, never the fields.** The pattern above matches this line
# as well, because it reads the body by name. A reader that told the two
# kinds apart by which fields it found took a heartbeat for a generation
# that scored nothing. The fields of this trainer move: `abs-spread` arrived
# between the spread and the win share, and `shard 1/2` arrived between the
# generation and the marker. The marker itself has not moved.
IN_FLIGHT = re.compile(r"(?:^|\s)working(?:\s|$)")

# A heartbeat, which one process prints about every thirty seconds while it
# scores. The strategy name and the shard are both optional. One process
# measures the shared controller baseline for every strategy and names none
# of them, and a pass that one process scores whole reports no shard.
WORKING = re.compile(
    r"^\s+(?:(?P<name>\S+) )?(?P<what>generation\s+\d+|yardstick|baseline"
    r"|validation\s+\d+)"
    r"(?: shard (?P<shard>\d+)/(?P<shards>\d+))?"
    r" working\s+decisions\s+(?P<decisions>\d+)\s+"
    r"live\s+(?P<live>\d+)/(?P<worlds>\d+)\s+"
    r"ticks\s+(?P<ticks>\d+)\s+rate\s+(?P<rate>[\d.]+) t/s\s+"
    r"\[(?P<seconds>[\d.]+)s\]"
)


def in_flight(body: str) -> bool:
    """Say whether this line reports work in flight rather than a result."""
    return IN_FLIGHT.search(body) is not None


def fields(body: str) -> dict[str, float | None]:
    """Return the named fields of a generation line, a dash meaning none."""
    found: dict[str, float | None] = {}
    for match in FIELD.finditer(body):
        text = match.group("value")
        found[match.group("key")] = None if text == "-" else float(text)
    return found


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
class Beat:
    """The newest heartbeat of one shard, as numbers."""

    decisions: int
    live: int
    worlds: int
    rate: float
    seconds: float


@dataclass
class Flight:
    """The pass one strategy is running now, with every shard combined.

    A strategy splits a generation over several processes. Each of them
    scores one shard of the population and prints its own heartbeat, so one
    pass reports several lines a round. The parts combine this way.

    - The live worlds and the worlds add. The shards hold disjoint parts of
      the population, so a sum counts each world once.
    - The decisions add. Each shard counts only the decisions it took.
    - The rates add. The shards run at the same time on different cores.
    - The elapsed time is the longest. The pass ends with its slowest shard.

    **A pass in flight contributes nothing to a derived figure.** The spend
    for each generation, the generations remaining and the estimate of the
    time left all divide by the generations that finished. A pass that is
    six minutes into a seven minute generation has produced no result, so
    counting it would report a generation that cost six minutes and scored
    nothing, and it would shorten every estimate that follows.
    """

    what: str
    live: int = 0
    worlds: int = 0
    decisions: int = 0
    rate: float = 0.0
    seconds: float = 0.0
    shards: int = 0

    @property
    def share(self) -> float | None:
        """Return the part of the pass that has finished, from 0 to 1."""
        return None if self.worlds <= 0 else (self.worlds - self.live) / self.worlds


@dataclass
class Strategy:
    """One policy under training, and what it has scored so far."""

    name: str
    kind: str = ""
    generations: list[Generation] = field(default_factory=list)
    baselines: dict[str, dict[str, float]] = field(default_factory=dict)
    # The newest heartbeat of each shard of the pass this strategy runs now,
    # keyed on the shard number. A terminal line empties it.
    working: dict[int, Beat] = field(default_factory=dict)
    pass_name: str = ""

    @property
    def flight(self) -> Flight | None:
        """Return the pass in flight, or nothing when none is running."""
        beats = self.working
        if not beats:
            return None
        return Flight(
            what=self.pass_name,
            live=sum(beat.live for beat in beats.values()),
            worlds=sum(beat.worlds for beat in beats.values()),
            decisions=sum(beat.decisions for beat in beats.values()),
            rate=sum(beat.rate for beat in beats.values()),
            seconds=max(beat.seconds for beat in beats.values()),
            shards=len(beats),
        )

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
    # The wall clock the run has used, which the caller measures from the
    # start of the run. The log cannot supply it, so a caller that knows the
    # start must set it.
    wall_clock_seconds: float | None = None

    @property
    def generations_done(self) -> int:
        """Return how many generations the whole run has finished."""
        return sum(len(strategy.generations) for strategy in self.strategies)

    @property
    def elapsed_seconds(self) -> float:
        """Return the wall clock the run has used.

        **The wall clock the caller measured wins.** A run spends money from
        the first minute, through the engine build and through the controller
        baseline, and the log names no generation in that time.

        The log is the fallback, and it gives the cumulative seconds of the
        newest generation. The strategies run at the same time, so the
        longest one is the wall clock and the sum of them is not. A reader
        that summed them reported six times the truth on a run of six
        strategies, and reported zero before the first generation finished.
        """
        if self.wall_clock_seconds is not None:
            return self.wall_clock_seconds
        return max(
            (
                strategy.generations[-1].seconds
                for strategy in self.strategies
                if strategy.generations
            ),
            default=0.0,
        )

    @property
    def collapsed(self) -> bool:
        """Say whether the strategy under training has stopped searching."""
        return bool(self.strategies) and self.strategies[-1].collapsed


def parse(text: str) -> Progress:
    """Read a training log, and return what it says."""
    progress = Progress()
    current: Strategy | None = None
    # **A log can hold several strategies at once.** A run starts one trainer
    # process for each strategy, and they all append to one file, so the
    # heading of one strategy is not the owner of the next line. A generation
    # row names its own strategy, so the row goes to the strategy it names.
    # This index is how a row finds it.
    by_name: dict[str, Strategy] = {}

    def strategy_named(name: str, kind: str = "") -> Strategy:
        """Return the strategy of this name, and make it if it is new."""
        found = by_name.get(name)
        if found is None:
            found = Strategy(name=name, kind=kind)
            by_name[name] = found
            progress.strategies.append(found)
        elif kind and not found.kind:
            found.kind = kind
        return found

    for line in text.splitlines():
        heading = HEADING.match(line)
        if heading:
            name = heading.group("name")
            if name == "controller baseline":  # pragma: no cover - unreachable
                continue
            current = strategy_named(name, heading.group("kind") or "")
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

        # **The marker decides the kind of the line, and it is read first.**
        # The pattern for a finished generation matches a heartbeat as well,
        # because both open with the strategy and the generation number. A
        # heartbeat that reached that pattern was read as a generation with
        # no mean and no best.
        if in_flight(line):
            beat = WORKING.match(line)
            if beat is None or beat.group("name") is None:
                continue
            owner = strategy_named(beat.group("name"))
            what = " ".join(beat.group("what").split())
            if owner.pass_name != what:
                owner.working = {}
                owner.pass_name = what
            owner.working[int(beat.group("shard") or 0)] = Beat(
                decisions=int(beat.group("decisions")),
                live=int(beat.group("live")),
                worlds=int(beat.group("worlds")),
                rate=float(beat.group("rate")),
                seconds=float(beat.group("seconds")),
            )
            continue

        row = GENERATION.match(line)
        if row:
            values = fields(row.group("body"))
            mean = values.get("mean")
            best = values.get("best")
            if mean is None or best is None:
                continue
            # The row names its strategy, so it does not go to whichever
            # heading came last. With several trainers writing one file, the
            # last heading is usually another strategy entirely.
            owner = strategy_named(row.group("name"))
            ticks = values.get("ticks")
            owner.generations.append(
                Generation(
                    name=row.group("name"),
                    generation=int(row.group("generation")),
                    mean=mean,
                    best=best,
                    seconds=float(row.group("seconds")),
                    spread=values.get("spread"),
                    won=values.get("won"),
                    ticks=None if ticks is None else int(ticks),
                    validation=values.get("valid"),
                )
            )
            # A finished generation ends every shard of the pass. A frozen
            # heartbeat left behind would report a pass that runs.
            owner.working = {}
            owner.pass_name = ""
            continue

        # **A baseline row does not name its strategy.** It goes to the
        # heading that came last, which is right when one trainer writes the
        # file. When several write it, a baseline can land on the wrong
        # strategy. The trainer must print the name for this to be safe.
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
        if strategy.generations or strategy.baselines or strategy.working
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

    **Only a finished generation counts here.** A generation in flight has
    produced no result, so it cannot say what a generation costs. A run that
    counted the pass it is running would divide the elapsed time by one more
    generation than it has, report a generation cheaper than any it
    finished, and shorten the estimate of the time left.
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


# How many generations of each strategy the dashboard prints. A strategy runs
# to twenty or more, and the last few say only what just happened. Fifteen
# shows the shape of the search without pushing the bar off the screen.
RECENT_GENERATIONS = 15


def render(
    progress: Progress,
    price_per_hour: float,
    total_generations: int,
    instance_type: str = "",
    zone: str = "",
    recent_generations: int = RECENT_GENERATIONS,
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
        lines.append(f"  --- {label}: {len(rows)} generations finished")
        trained = strategy.baselines.get("trained")
        controller = strategy.baselines.get("controller") or bar
        flight = strategy.flight
        if flight is not None:
            reached = "-" if flight.share is None else f"{flight.share:.0%}"
            shards = f", {flight.shards} shards" if flight.shards > 1 else ""
            lines.append(
                f"      in flight  {flight.what}, {reached} of "
                f"{flight.worlds} worlds{shards}, "
                f"{flight.rate:.0f} ticks/s, {flight.decisions} decisions "
                f"[{flight.seconds:.0f}s]"
            )
        elif trained:
            lines.append("      in flight  nothing, this strategy ended")
        else:
            lines.append("      in flight  nothing, between passes")
        if trained:
            verdict = "BEATS the controller"
            if controller and trained["won"] <= controller["won"]:
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
            recent = rows[-recent_generations:] if recent_generations > 0 else rows
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
    parser.add_argument(
        "--recent",
        type=int,
        default=RECENT_GENERATIONS,
        help="how many generations of each strategy to print. Zero prints "
        f"every one. Default {RECENT_GENERATIONS}",
    )
    parser.add_argument(
        "--started",
        type=float,
        default=0.0,
        help="the Unix time the run started. It gives the wall clock, and "
        "without it the newest generation of the log is the fallback",
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
    if arguments.started > 0:
        progress.wall_clock_seconds = max(0.0, time.time() - arguments.started)
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
            arguments.recent,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
