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
writes that row to the report. That row is the bar. The trainer also plays
the best centre on the held-out seeds every few generations, so a run that a
wall clock cap ends leaves an honest figure behind.

The validation figure comes from the seeds that choose the centre, so it
selects rather than measures. This module carries it, and it says which
seeds every figure it prints came from.

# The verdict reads a win share and never a return

A win share measures play. The mean shaped return is what the search
maximises, and a run can raise it while its policy takes one unit and
wanders. The verdict of each strategy therefore compares win shares, and it
names the measurement it used.[^1]

# A spread of zero means the search stopped

Each generation ranks its candidates by score. When the highest and the
lowest score the same, the ranking carries no information and the update
that follows it is noise. A run in that state keeps billing and learns
nothing, so this module names it and the launcher can end the run on it.

**Every strategy answers that question, and the threshold is a share of the
reward.** The strategies of a run write their lines into one file, so a test
that read the last strategy parsed tested an arbitrary quarter of a run of
four. A spread is a difference of two returns, so an absolute threshold
holds for one reward scale only, and the styles of this project score two
orders of magnitude apart.

# The run-level controller bar answers for one weighting

One process measures the controller bar once for the whole run, under the
weighting of the first strategy the run names. A return under one weighting
does not compare with a return under another, so this module names the
weighting beside the return. The win share of the same bar compares, because
no weighting changes who won.

# What the parser reads

The trainer prints one line for each generation, one heading for each
strategy, and one row for each baseline at the end of a strategy. Those
lines are the interface this module reads. A line it cannot parse is
skipped rather than fatal, so a trainer that gains a column keeps working
here.

# References

[^1]: What is wrong with training and evaluation, items 1 and 2.
`docs/research/what-is-wrong-with-training-and-evaluation.md`
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
#
# **The first count is the decisions of a batch or the episodes of a queue.**
# A pass this process plays steps every world of a batch together, so it
# counts decisions. A generation of a training run holds one episode in each
# task of a queue, so it counts the episodes that finished. Both are a count
# of work done since the pass started, and the reader treats them alike.
WORKING = re.compile(
    r"^\s+(?:(?P<name>\S+) )?(?P<what>generation\s+\d+|yardstick|baseline"
    r"|validation\s+\d+|holdout\s+\d+)"
    r"(?: shard (?P<shard>\d+)/(?P<shards>\d+))?"
    r" working\s+(?:decisions|episodes)\s+(?P<decisions>\d+)\s+"
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

# The line that names the weighting the run-level controller bar was measured
# under, for example:
#   the controller bar is measured under the conquer weighting
#
# **A return under one weighting does not compare with a return under
# another.** One process measures the bar under the first strategy of the run
# and every strategy later measures its own, so the run-level figure answers
# for one strategy and the dashboard must say which.
CONTROLLER_WEIGHTING = re.compile(
    r"^\s+the controller bar is measured under the (?P<name>\S+) weighting"
)

# The number of generations behind the collapse test, and the share of the
# reward scale under which a generation counts as collapsed.
#
# **The threshold is a share and never a reward.** A spread is a difference of
# two returns, so an absolute threshold holds for one reward scale only. This
# threshold was an absolute 1.0, and a comment beside it told a caller who
# changed the reward scale to change the number too. That is one rule stored
# in two places with nothing that fails when the copies disagree. The retired
# policies scored in the thousands, where a spread under 1.0 could never fire,
# and the play styles score in the tens, where the same 1.0 is a tenth of the
# whole quantity.
#
# The share below is that absolute 1.0 read against a generation whose scores
# were of order one thousand, which is the scale it was written at.
COLLAPSE_WINDOW = 3
COLLAPSE_SHARE = 0.001


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
    # The mean shaped return of the validation pass, and the win share of the
    # same pass. **The validation seeds choose the centre**, so both figures
    # select and neither is a measurement.
    validation: float | None = None
    validation_won: float | None = None
    # The mean shaped return and the win share of the held-out pass. **The
    # held-out seeds choose nothing**, so these measure. A generation that
    # took no held-out pass holds neither.
    holdout: float | None = None
    holdout_won: float | None = None
    # The share of decisions of the validation pass on which the policy
    # emitted its most common action, and the share of its episodes whose
    # unmasked argmax changed. Both are instruments and neither gates a run.
    most_common_share: float | None = None
    preference_varies: float | None = None

    @property
    def reward_scale(self) -> float:
        """Return how large the reward this generation scored is.

        The mean and the best of one generation are two readings of the same
        reward, so the larger magnitude of the two states the scale that a
        spread of this generation must be read against. Both are means over
        the seeds of a candidate, so neither carries the terminal quantum of
        a single seed on its own.
        """
        return max(abs(self.mean), abs(self.best))

    @property
    def carries_no_information(self) -> bool:
        """Say whether the candidates of this generation scored the same.

        **The test is a share of the reward scale and never an absolute
        return.** A run whose style scores in the tens and a run whose style
        scores in the thousands must answer this question alike, and an
        absolute threshold answers it for one of the two.

        A generation that scored zero on both readings has no scale, so only
        a spread of exactly zero counts as carrying nothing.
        """
        if self.spread is None:
            return False
        return self.spread <= COLLAPSE_SHARE * self.reward_scale


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


@dataclass(frozen=True)
class Measurement:
    """One win share of a policy, and which seeds gave it.

    **A win share is the quantity that measures play**, and the mean shaped
    return is the training signal. A dashboard that printed the return said
    nothing about winning, and the paid run it watched published four
    policies that take one unit and wander.[^1]

    The source entry names the seeds and the generation, so a reader knows
    what the figure is. The selected entry is true when those seeds chose the
    centre, which makes the figure a maximum over the passes of the run
    rather than a measurement.

    References
    ----------
    [^1]: What is wrong with training and evaluation, item 1.
    `docs/research/what-is-wrong-with-training-and-evaluation.md`
    """

    won: float
    source: str
    selected: bool


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
    def last_holdout(self) -> Generation | None:
        """Return the newest generation that took a held-out pass, or nothing.

        **The newest generation is not the newest held-out pass.** The trainer
        takes that pass at an interval, so a reader that looked only at the
        last row would report no held-out figure for every generation between
        two intervals.
        """
        for row in reversed(self.generations):
            if row.holdout is not None or row.holdout_won is not None:
                return row
        return None

    @property
    def latest_measurement(self) -> Measurement | None:
        """Return the newest win share of this policy, and where it came from.

        **The verdict renders from whatever the newest honest figure is.** It
        used to render only from the row a finished strategy leaves, and a
        wall clock cap ended a paid run before any strategy finished. The
        dashboard then showed a shaped return for the whole run and never
        once said anything about winning.[^1]

        The three sources rank by how much they measure. The row of a
        finished strategy plays the held-out seeds after the run. A periodic
        held-out pass plays the same seeds during the run. The validation
        pass plays the seeds that chose the centre, so it selects rather than
        measures, and the source says so.

        References
        ----------
        [^1]: What is wrong with training and evaluation, item 1.
        `docs/research/what-is-wrong-with-training-and-evaluation.md`
        """
        trained = self.baselines.get("trained")
        if trained is not None and "won" in trained:
            return Measurement(
                won=trained["won"],
                source="the held-out seeds, after this strategy ended",
                selected=False,
            )
        for row in reversed(self.generations):
            if row.holdout_won is not None:
                return Measurement(
                    won=row.holdout_won,
                    source=(
                        f"the held-out seeds at generation {row.generation}, "
                        "which chose nothing"
                    ),
                    selected=False,
                )
        for row in reversed(self.generations):
            if row.validation_won is not None:
                return Measurement(
                    won=row.validation_won,
                    source=(
                        f"the validation seeds at generation {row.generation}, "
                        "which chose the centre"
                    ),
                    selected=True,
                )
        return None

    @property
    def collapsed(self) -> bool:
        """Say whether the search of this strategy has stopped.

        The last few generations decide it. A strategy collapses when every
        candidate of each of them scored the same, because the ranking that
        drives the update then ranks nothing.

        Each generation answers against its own reward scale, so a strategy
        that scores in the tens and a strategy that scores in the thousands
        are read alike.
        """
        recent = self.generations[-COLLAPSE_WINDOW:]
        if len(recent) < COLLAPSE_WINDOW or any(row.spread is None for row in recent):
            return False
        return all(row.carries_no_information for row in recent)


@dataclass
class Progress:
    """Everything one log says about one run."""

    strategies: list[Strategy] = field(default_factory=list)
    controller: dict[str, float] = field(default_factory=dict)
    # The strategy whose weighting the run-level controller bar was measured
    # under. One process measures that bar once for the whole run, and it
    # reads under the first strategy the run names. A return under one
    # weighting does not compare with a return under another, so a reader that
    # met the figure without this name took it for the whole run.
    controller_weighting: str = ""
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
    def collapsed_strategies(self) -> list[str]:
        """Return the name of every strategy whose search has stopped."""
        return [strategy.name for strategy in self.strategies if strategy.collapsed]

    @property
    def collapsed(self) -> bool:
        """Say whether the whole run has stopped searching.

        **Every strategy answers, and not the last one parsed.** The
        strategies of a run write their lines into one file, so the last one
        in this list is whichever process wrote last. A guard that read that
        one tested a quarter of a run of four strategies and never tested the
        other three.

        The run ends when every strategy has stopped, because the caller of
        this property ends the run on it. A strategy that is still searching
        is worth the machine, whatever its neighbours do, and the dashboard
        names each stopped strategy so that a person can end one early.
        """
        return bool(self.strategies) and all(
            strategy.collapsed for strategy in self.strategies
        )


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

        # The line that qualifies the run-level bar sits beside that row, and
        # it names the weighting the return reads under.
        weighting = CONTROLLER_WEIGHTING.match(line)
        if weighting and not progress.controller_weighting:
            progress.controller_weighting = weighting.group("name")
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
                    validation_won=values.get("valid-won"),
                    holdout=values.get("holdout"),
                    holdout_won=values.get("holdout-won"),
                    most_common_share=values.get("top-share"),
                    preference_varies=values.get("varies"),
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


def verdict_lines(strategy: Strategy, controller: dict[str, float] | None) -> list[str]:
    """Return the verdict of one strategy, and which measurement gave it.

    **The verdict compares win shares and never returns.** A win share
    measures play. The mean shaped return is the training signal, and a run
    can raise it while its policy takes one unit and wanders.[^1]

    The verdict names the seeds it read, because the three sources answer
    different questions. A figure from the seeds that chose the centre is a
    maximum over the passes of the run, so the line marks it as a selection
    figure rather than a measurement.

    References
    ----------
    [^1]: Findings register, FND-707. `docs/FINDINGS.md`
    """
    measured = strategy.latest_measurement
    if measured is None:
        return ["      verdict    nothing measures winning yet"]
    if not controller or "won" not in controller:
        return [
            f"      verdict    wins {measured.won:.3f}, and no controller bar "
            "to compare",
            f"                 from {measured.source}",
        ]
    beats = measured.won > controller["won"]
    word = "BEATS the controller" if beats else "loses to the controller"
    caution = ", which selects" if measured.selected else ""
    return [
        f"      verdict    wins {measured.won:.3f} against the controller's "
        f"{controller['won']:.3f}: {word}",
        f"                 from {measured.source}{caution}",
    ]


def weighting_lines(weighting: str) -> list[str]:
    """Return the qualification the run-level controller bar needs.

    **The return of that bar answers for one weighting.** One process
    measures the bar once for the whole run, under the weighting of the first
    strategy the run names, and every strategy later measures its own bar
    under its own weighting. A reader who met the run-level return without
    this line took a figure for the run that answered for a quarter of it.

    The win share of the same bar does compare, because a win is a fact of
    the game and no weighting changes it. The verdict of each strategy reads
    that win share, so the qualification bounds the return alone.
    """
    if not weighting:
        return [
            "                the weighting behind this return is not named in the "
            "log, so the",
            "                return compares with nothing. The win share compares.",
        ]
    return [
        f"                the return reads under the {weighting} weighting, and a "
        "return under",
        "                one weighting does not compare with a return under "
        "another. The win",
        "                share compares, because no weighting changes who won.",
    ]


def collapse_lines(strategy: Strategy) -> list[str]:
    """Return what the dashboard says about a strategy that stopped searching.

    The message names the share of the reward scale that the test used and
    the scale it read, so a person can see why a run of one reward size and a
    run of another answered the same question alike.
    """
    recent = strategy.generations[-COLLAPSE_WINDOW:]
    scale = max((row.reward_scale for row in recent), default=0.0)
    return [
        f"      SEARCH STOPPED: the spread stayed under {COLLAPSE_SHARE:.3%} of a "
        f"reward scale of {scale:.1f}",
        f"      for {COLLAPSE_WINDOW} generations. Every candidate scored the same, "
        "so the update carries",
        "      no information. Stop this strategy.",
    ]


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
        lines.extend(weighting_lines(progress.controller_weighting))
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
            lines.append(
                f"      held out   wins {trained['won']:.3f} "
                f"return {trained['return']:9.1f}"
            )
        else:
            held = strategy.last_holdout
            latest = strategy.last_validation
            if held is not None:
                wins = "-" if held.holdout_won is None else f"{held.holdout_won:.3f}"
                gave = "-" if held.holdout is None else f"{held.holdout:9.1f}"
                lines.append(
                    f"      held out   wins {wins} return {gave} "
                    f"at generation {held.generation}, during the run"
                )
            elif latest is not None:
                lines.append(
                    f"      held out   not measured yet; validation {latest:.1f}, "
                    "which chose the centre"
                )
            else:
                lines.append("      held out   not measured yet; no validation yet")
        lines.extend(verdict_lines(strategy, controller))
        if rows:
            recent = rows[-recent_generations:] if recent_generations > 0 else rows
            lines.append(
                "      generation  mean       best       spread   won    top   vary"
            )
            for row in recent:
                spread = "     -" if row.spread is None else f"{row.spread:9.1f}"
                won = "    -" if row.won is None else f"{row.won:5.2f}"
                top = (
                    "    -"
                    if row.most_common_share is None
                    else f"{row.most_common_share:5.2f}"
                )
                vary = (
                    "    -"
                    if row.preference_varies is None
                    else f"{row.preference_varies:5.2f}"
                )
                lines.append(
                    f"      {row.generation:10d} {row.mean:10.1f} "
                    f"{row.best:10.1f} {spread} {won} {top} {vary}"
                )
            lines.append(
                "      top is the share of decisions on the most common action, "
                "and vary is"
            )
            lines.append(
                "      the share of episodes whose unmasked preference moved. "
                "Both are instruments."
            )
        if strategy.collapsed:
            lines.extend(collapse_lines(strategy))

    # Every strategy answers the collapse test, and the run ends only when
    # all of them have stopped. A reader who met one stopped strategy needs
    # to know whether the machine is still buying anything.
    stopped = progress.collapsed_strategies
    if stopped and not progress.collapsed:
        lines.append(
            f"  --- {len(stopped)} of {len(progress.strategies)} strategies stopped "
            f"searching: {', '.join(stopped)}"
        )
        lines.append("      The run keeps paying, because the rest still search.")
    elif progress.collapsed:
        lines.append(
            "  --- EVERY STRATEGY STOPPED SEARCHING. The run buys nothing more."
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
                    "validation_won": row.validation_won,
                    "holdout": row.holdout,
                    "holdout_won": row.holdout_won,
                    "most_common_share": row.most_common_share,
                    "preference_varies": row.preference_varies,
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
        weighting = report.get("controller_weighting")
        if isinstance(weighting, str) and not progress.controller_weighting:
            progress.controller_weighting = weighting

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
