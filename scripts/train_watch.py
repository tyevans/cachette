#!/usr/bin/env python3
"""Render one screen that says what a training run is doing and what it did.

A training run starts one trainer process for each strategy, and they all
append to one log. This script reads that log and prints a screen small
enough for a one second refresh.

# What the screen answers

A reader of a run asks six questions, and the screen answers them in the
order that they matter.

1. **Can I see the run?** The caller says whether it reached the instance,
   how old its copy of the log is, and whether the instance still exists.
   The screen never reports a silent instance as a dead one.
2. **What is it doing?** A run builds the engine, measures the ticks a
   second, measures the controller baseline, and then trains. The screen
   names the phase.
3. **Is it working?** Every strategy prints a heartbeat while a pass runs,
   and a terminal line when the pass ends. A strategy whose last word was a
   terminal line has finished that pass and started nothing. A strategy that
   has fallen behind the others is quiet.
4. **How fast, and how busy?** Each heartbeat carries the ticks a second
   that the strategy reached. The screen adds the heartbeats of the working
   strategies only. The load average against the core count says whether
   the machine is full.
5. **Is it winning?** The yardstick is what the built-in controller scored
   on the validation seeds. The screen prints the newest validation figure
   of each strategy beside the difference from its own yardstick.
6. **What does it cost, and how long is left?** The wall clock and the price
   an hour give the money. The generations finished give the rest.

# The rule that keeps the screen honest

**A frozen line is not a working process.** A pass that ends leaves its last
heartbeat in the log for as long as the log lives. A reader that took the
last heartbeat of each strategy for the state of that strategy counted four
finished baseline passes as four live processes, and added their frozen
rates into one machine rate of about four times the truth.[^1]

The order of the lines settles it. A heartbeat says a pass runs. A terminal
line for the same pass says it ended. Whichever came last is the truth.

**A training-seed column does not compare across generations.** The trainer
draws a new training seed set for each generation, so the mean, the best,
the spread and the win share answer a different question every generation.
The screen marks each of those columns with a star. Only the validation
figure holds still, because the validation seeds never move.

# What it does not do

It takes no measurement of its own and it reaches no machine. It reads a log
file. The caller brings the log and every fact about the machine.

# References

[^1]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

# A finished generation, for example:
#   conquer generation  3 mean -1594.6 best -638.9 spread 1353.5 won 0.10 ...
# **The fields are read by name, not by their order.** The trainer prints
# what it has, and it has gained a field before: a reader that fixed the order
# stopped matching every line the moment `abs-spread` appeared between the
# spread and the win share, and it reported an empty run rather than an error.
DONE = re.compile(
    r"^\s+(?P<name>\S+) generation\s+(?P<generation>\d+)\s+"
    r"(?P<body>.*?)\s*\[(?P<seconds>[\d.]+)s\]\s*$"
)

# One `name value` pair of the body. A value is a number or a bare `-`.
FIELD = re.compile(r"(?P<key>[a-z][a-z-]*)\s+(?P<value>-?[\d.]+|-)")

# A heartbeat from inside a long pass, for example:
#   conquer generation  3 working  decisions 120 live 87/144 ticks 174000
#   rate 1893.0 t/s [92s]
#
# **The pass names itself, and the screen holds no list of the passes.** The
# controller baseline was added to this alternation after a run spent over
# nine minutes on it in silence. Every strategy of that run looked stopped,
# and the load average over a remote connection was the only evidence that
# the machine was alive.
WORKING = re.compile(
    r"^\s+(?P<name>\S+) (?P<what>generation\s+\d+|yardstick|baseline"
    r"|validation\s+\d+)"
    r" working\s+decisions\s+(?P<decisions>\d+)\s+"
    r"live\s+(?P<live>\d+)/(?P<worlds>\d+)\s+"
    r"ticks\s+(?P<ticks>\d+)\s+rate\s+(?P<rate>[\d.]+) t/s\s+"
    r"\[(?P<seconds>[\d.]+)s\]"
)

# A process that waits for another process to measure the same baseline, for
# example:
#   land-net baseline waiting 60s for another process to measure the same
#   number
#
# **A waiting process takes no ticks, so it cannot report a rate.** A line in
# the heartbeat shape with a rate of zero would read as a process that
# stopped, and this shape says what it is instead.
WAITING = re.compile(
    r"^\s+(?P<name>\S+) (?P<what>baseline) waiting (?P<seconds>[\d.]+)s"
)

# The controller yardstick, printed once for each strategy. It is the bar to
# beat on the validation seeds, and it is the terminal line of the yardstick
# pass.
YARDSTICK = re.compile(r"^\s+(?P<name>\S+) controller yardstick\s+(?P<value>-?[\d.]+)")

# The end of the baseline pass of one strategy. A process that measured the
# number says `measured`, and a process that took it from another process
# says `cached`. Either way the pass has ended, for example:
#   conquer controller measured return     1480.2 won  0.34
BASELINE_END = re.compile(
    r"^\s+(?P<name>\S+) controller (?:measured|cached) return\s+"
    r"(?P<value>-?[\d.]+)\s+won\s+(?P<won>-?[\d.]+)"
)

# The held-out controller row, printed once before any training.
CONTROLLER = re.compile(r"^\s+controller (?P<body>\{.*\})\s*$")

# The heading the trainer prints for one strategy, for example:
#   === land-structured (structured) ===
HEADING = re.compile(r"^===\s+(?P<name>\S+)(?:\s+\((?P<kind>[^)]*)\))?\s+===\s*$")

# The throughput probe the run takes before it trains anything.
PROBE = re.compile(r"^\s+measuring\s+(?P<processes>\d+) process")

# How many rounds of heartbeats from the other strategies may pass before a
# strategy counts as quiet. Every strategy heartbeats on the same cadence,
# so three whole rounds of silence from one of them is not scheduling noise.
QUIET_ROUNDS = 3

# How often a strategy heartbeats. The trainer prints one line for each
# strategy about this often, so three of these is the silence that means a
# worker went rather than a pass being long.
HEARTBEAT_SECONDS = 30.0

# The colour of each meaning the screen carries. A name states the meaning,
# never the colour, so a reader of the code cannot use green for a warning.
CODES = {
    "good": "32",
    "bad": "31",
    "warn": "33",
    "note": "36",
    "dim": "2",
    "head": "1",
}


class Paint:
    """Wrap text in an ANSI colour, or hand it back unchanged.

    Colour carries meaning on this screen and never decoration. A reader
    with colour turned off must lose nothing, so every coloured field also
    says its meaning in words.
    """

    def __init__(self, enabled: bool) -> None:
        """Hold whether this painter writes escapes at all."""
        self.enabled = enabled

    def __call__(self, meaning: str, text: str) -> str:
        """Return the text in the colour of this meaning."""
        code = CODES.get(meaning)
        if not self.enabled or code is None:
            return text
        return f"\x1b[{code}m{text}\x1b[0m"


def wants_colour(mode: str, stream: object) -> bool:
    """Say whether to write ANSI escapes.

    The `NO_COLOR` convention wins over every other setting, because a
    reader who asked for no colour asked last. `watch` hands its child a
    pipe rather than a terminal, so a caller that wants colour under `watch`
    must say so with `CLICOLOR_FORCE` or with the mode.

    # References

    [^1]: The NO_COLOR convention. https://no-color.org/
    """
    if os.environ.get("NO_COLOR"):
        return False
    if mode == "never":
        return False
    if mode == "always" or os.environ.get("CLICOLOR_FORCE"):
        return True
    return bool(getattr(stream, "isatty", lambda: False)())


def fields(body: str) -> dict[str, float | None]:
    """Return the named fields of a row, with a bare dash as no value."""
    found: dict[str, float | None] = {}
    for match in FIELD.finditer(body):
        text = match.group("value")
        found[match.group("key")] = None if text == "-" else float(text)
    return found


@dataclass
class Row:
    """One finished generation of one strategy."""

    generation: int
    mean: float
    best: float
    spread: float | None
    won: float | None
    validation: float | None


@dataclass
class Strategy:
    """What one strategy has done, and what its last word says it is doing.

    The three `last_` fields hold the newest line of each shape, and
    `heartbeat_mark` holds how many heartbeats the whole log had seen when
    this strategy last spoke. The difference between that mark and the total
    is how far behind the other strategies this one has fallen.
    """

    name: str
    kind: str = ""
    done: list[Row] = field(default_factory=list)
    last_working: dict[str, str] | None = None
    last_waiting: dict[str, str] | None = None
    last_ended: str | None = None
    yardstick: float | None = None
    baseline_return: float | None = None
    heartbeat_mark: int = 0

    @property
    def last_validation(self) -> float | None:
        """Return the newest validation figure, or nothing if it has none."""
        for row in reversed(self.done):
            if row.validation is not None:
                return row.validation
        return None

    @property
    def validations(self) -> list[tuple[int, float]]:
        """Return every validation figure against its generation number."""
        return [
            (row.generation, row.validation)
            for row in self.done
            if row.validation is not None
        ]

    @property
    def margin(self) -> float | None:
        """Return how far the newest validation figure sits above the bar."""
        latest = self.last_validation
        if latest is None or self.yardstick is None:
            return None
        return latest - self.yardstick


@dataclass
class Reading:
    """Everything one log says about one run."""

    strategies: dict[str, Strategy] = field(default_factory=dict)
    controller: dict[str, float] | None = None
    heartbeats: int = 0
    probed: bool = False


def read(text: str) -> Reading:
    """Return what each strategy has reached, and the controller row.

    **The order of the lines decides what a strategy is doing.** A heartbeat
    says a pass runs, and a terminal line for that pass says it ended. The
    later line wins, so a finished pass never counts as a working one.
    """
    reading = Reading()

    def named(name: str, kind: str = "") -> Strategy:
        found = reading.strategies.get(name)
        if found is None:
            found = Strategy(name=name, kind=kind)
            reading.strategies[name] = found
        elif kind and not found.kind:
            found.kind = kind
        return found

    def ended(strategy: Strategy, what: str) -> None:
        """Record that a pass of this strategy reached its end."""
        strategy.last_ended = what
        strategy.last_working = None
        strategy.last_waiting = None
        strategy.heartbeat_mark = reading.heartbeats

    for line in text.splitlines():
        heading = HEADING.match(line)
        if heading:
            named(heading.group("name"), heading.group("kind") or "")
            continue
        if PROBE.match(line):
            reading.probed = True
            continue
        row = DONE.match(line)
        if row and "working" not in row.group("body"):
            values = fields(row.group("body"))
            mean = values.get("mean")
            best = values.get("best")
            if mean is None or best is None:
                continue
            strategy = named(row.group("name"))
            strategy.done.append(
                Row(
                    generation=int(row.group("generation")),
                    mean=mean,
                    best=best,
                    spread=values.get("spread"),
                    won=values.get("won"),
                    validation=values.get("valid"),
                )
            )
            ended(strategy, f"generation {row.group('generation')}")
            continue
        work = WORKING.match(line)
        if work:
            reading.heartbeats += 1
            strategy = named(work.group("name"))
            strategy.last_working = work.groupdict()
            strategy.last_waiting = None
            strategy.last_ended = None
            strategy.heartbeat_mark = reading.heartbeats
            continue
        held = WAITING.match(line)
        if held:
            reading.heartbeats += 1
            strategy = named(held.group("name"))
            strategy.last_waiting = held.groupdict()
            strategy.last_working = None
            strategy.last_ended = None
            strategy.heartbeat_mark = reading.heartbeats
            continue
        yard = YARDSTICK.match(line)
        if yard:
            strategy = named(yard.group("name"))
            strategy.yardstick = float(yard.group("value"))
            ended(strategy, "yardstick")
            continue
        base = BASELINE_END.match(line)
        if base:
            strategy = named(base.group("name"))
            strategy.baseline_return = float(base.group("value"))
            ended(strategy, "baseline")
            continue
        if reading.controller is None:
            found = CONTROLLER.match(line)
            if found:
                try:
                    reading.controller = json.loads(
                        found.group("body").replace("'", '"')
                    )
                except ValueError:
                    reading.controller = None
    return reading


def rounds_behind(reading: Reading, strategy: Strategy) -> int:
    """Return how many whole rounds of heartbeats this strategy has missed.

    Every strategy heartbeats on the same cadence, so the heartbeats of the
    others are a clock that needs no wall clock. This keeps the reading of a
    stored log fixed, whatever the hour it is read at.
    """
    others = max(1, len(reading.strategies) - 1)
    return (reading.heartbeats - strategy.heartbeat_mark) // others


def state_of(reading: Reading, strategy: Strategy, log_quiet: float) -> str:
    """Return one word for what this strategy is doing.

    The words are `working`, `waiting`, `quiet`, `ended` and `silent`.
    `ended` says the last pass finished and nothing has started since.
    `quiet` says a pass claims to run while the strategy has stopped
    speaking, which is the shape a crashed worker leaves.
    """
    if strategy.last_working is None and strategy.last_waiting is None:
        return "ended" if strategy.last_ended else "silent"
    if log_quiet >= QUIET_ROUNDS * HEARTBEAT_SECONDS:
        return "quiet"
    if rounds_behind(reading, strategy) >= QUIET_ROUNDS:
        return "quiet"
    return "waiting" if strategy.last_waiting else "working"


def phase(reading: Reading, states: dict[str, str]) -> str:
    """Return what the run is doing right now, in the reader's words.

    A run builds the engine, measures the ticks a second, measures the
    controller baseline and the yardstick, and then trains. The phase comes
    from the passes that are running, never from a guess about the order.
    """
    if not reading.strategies:
        return (
            "measuring the ticks a second"
            if reading.probed
            else "building the engine, the trainer has said nothing"
        )
    passes = [
        " ".join(strategy.last_working["what"].split())
        for name, strategy in reading.strategies.items()
        if strategy.last_working and states[name] in {"working", "waiting"}
    ]
    for pass_name in ("baseline", "yardstick"):
        if any(what.startswith(pass_name) for what in passes):
            return f"measuring the controller {pass_name}"
    training = [what for what in passes if what.startswith("generation")]
    if training:
        numbers = sorted(int(what.split()[-1]) for what in training)
        return f"training generation {numbers[0]}"
    if any(word == "quiet" for word in states.values()):
        return "nothing has spoken lately"
    return "between passes"


def clock(seconds: float) -> str:
    """Return a duration as hours and minutes."""
    return f"{int(seconds) // 3600}h{(int(seconds) % 3600) // 60:02d}m"


def link_words(link: str, asked: float) -> tuple[str, str]:
    """Return the sentence about the connection, and its meaning for colour.

    Three states look the same to a careless reader and are not the same
    thing. `cached` says this watcher did not ask. `silent` says it asked
    and got nothing. `gone` says the instance no longer exists, which is
    what a reclaimed spot instance leaves behind.
    """
    age = f"{asked:.0f}s old" if asked >= 0 else "of an unknown age"
    if link == "live":
        return "live, the instance answered just now", "good"
    if link == "cached":
        return f"live, showing a copy {age}, no read due yet", "good"
    if link == "silent":
        return f"the instance did not answer, showing data {age}", "warn"
    if link == "gone":
        return f"the instance is gone, showing the last data, {age}", "bad"
    if link == "local":
        return "the run ended, reading the log it brought back", "note"
    return f"unknown, showing data {age}", "warn"


@dataclass
class Facts:
    """What the caller measured about the machine and the run."""

    heading: str = "training run"
    link: str = "unknown"
    asked: float = -1.0
    log_quiet: float = -1.0
    elapsed: float = 0.0
    price: float = 0.0
    generations: int = 0
    load: float = -1.0
    cores: int = 0


def header(
    reading: Reading, states: dict[str, str], facts: Facts, paint: Paint
) -> list[str]:
    """Return the lines above the table.

    **A price of zero is an unknown price, not a free run.** The teardown of
    a run removes the state file that holds the price. A screen of an ended
    run must not report that the run cost nothing.
    """
    lines = [paint("head", f"=== {facts.heading}")]

    sentence, meaning = link_words(facts.link, facts.asked)
    quiet = ""
    if facts.log_quiet >= 0:
        quiet_meaning = (
            "warn" if facts.log_quiet >= QUIET_ROUNDS * HEARTBEAT_SECONDS else "dim"
        )
        quiet = "   " + paint(
            quiet_meaning, f"the run last wrote {facts.log_quiet:.0f}s ago"
        )
    lines.append(f"  link      {paint(meaning, sentence)}{quiet}")

    working = [name for name, word in states.items() if word == "working"]
    busy = ""
    if facts.load >= 0 and facts.cores:
        share = facts.load / facts.cores
        load_meaning = "good" if 0.5 <= share <= 1.15 else "warn"
        busy = "   " + paint(
            load_meaning, f"load {facts.load:.1f} of {facts.cores} cores ({share:.0%})"
        )
    lines.append(
        f"  phase     {paint('note', phase(reading, states))}"
        f"   {len(working)} of {len(reading.strategies)} strategies working{busy}"
    )

    total_done = sum(len(s.done) for s in reading.strategies.values())
    spent = f"spent ${facts.price * facts.elapsed / 3600.0:.2f}"
    if facts.price <= 0:
        spent = "spend unknown, no price for this run"
    target = f"/{facts.generations}" if facts.generations > 0 else ""
    left = ""
    projected = ""
    if total_done and facts.elapsed and facts.generations > 0:
        each = facts.elapsed / total_done
        remaining = max(facts.generations - total_done, 0) * each
        left = f"   left ~{clock(remaining)}"
        if facts.price > 0:
            whole = facts.price * (facts.elapsed + remaining) / 3600.0
            projected = f"   whole run ~${whole:.2f}"
    lines.append(
        f"  clock     elapsed {clock(facts.elapsed)}   {spent}"
        f"   generations {total_done}{target}{left}{projected}"
    )

    beats = [
        reading.strategies[name].last_working
        for name in working
        if reading.strategies[name].last_working
    ]
    rate = sum(float(beat["rate"]) for beat in beats if beat)
    lines.append(
        f"  machine   {rate:.0f} ticks/s across {len(working)} working "
        f"{'process' if len(working) == 1 else 'processes'}"
    )
    if reading.controller:
        lines.append(
            paint(
                "dim",
                f"  baseline  the controller took return "
                f"{reading.controller.get('return', 0.0):.1f} "
                f"and won {reading.controller.get('won', 0.0):.3f} of its games",
            )
        )
    return lines


def now_words(strategy: Strategy, word: str) -> str:
    """Return the short sentence for the state column of one strategy."""
    if word == "silent":
        return "no word yet"
    if word == "ended":
        ended = " ".join((strategy.last_ended or "").split())
        return f"ended {ended}, nothing started since"
    if strategy.last_waiting:
        held = strategy.last_waiting
        return (
            f"{held['what']} waiting on another process [{float(held['seconds']):.0f}s]"
        )
    work = strategy.last_working
    if work is None:
        return "no word yet"
    what = " ".join(work["what"].split())
    body = (
        f"{what} d{work['decisions']} live {work['live']}/{work['worlds']} "
        f"{float(work['rate']):.0f}t/s [{float(work['seconds']):.0f}s]"
    )
    return f"QUIET, last said {body}" if word == "quiet" else body


STATE_MEANING = {
    "working": "good",
    "waiting": "note",
    "quiet": "bad",
    "ended": "dim",
    "silent": "warn",
}


def table(reading: Reading, states: dict[str, str], paint: Paint) -> list[str]:
    """Return the one row for each strategy, newest figures first.

    The starred columns come from the training seeds of their own
    generation, and that seed set moves every generation. They do not
    compare with the same column of another generation.
    """
    lines = [
        paint(
            "head",
            f"  {'strategy':<19}{'kind':<11}{'gen':>4} {'mean*':>10} {'best*':>10} "
            f"{'spread*':>9} {'won*':>5} {'valid':>10} {'vs bar':>8}  state",
        )
    ]
    for name in sorted(reading.strategies):
        strategy = reading.strategies[name]
        if strategy.done:
            newest = strategy.done[-1]
            spread = "        -" if newest.spread is None else f"{newest.spread:9.1f}"
            won = "    -" if newest.won is None else f"{newest.won:5.2f}"
            body = paint(
                "dim",
                f"{newest.generation:>4} {newest.mean:>10.1f} {newest.best:>10.1f} "
                f"{spread} {won}",
            )
        else:
            body = paint("dim", f"{'-':>4} {'-':>10} {'-':>10} {'-':>9} {'-':>5}")
        latest = strategy.last_validation
        valid = f"{'-':>10}" if latest is None else f"{latest:>10.1f}"
        margin = strategy.margin
        if margin is None:
            versus = f"{'-':>8}"
        else:
            versus = paint("good" if margin > 0 else "bad", f"{margin:>+8.0f}")
        word = states[name]
        lines.append(
            f"  {name:<19}{strategy.kind or '-':<11}{body} {valid} {versus}  "
            + paint(STATE_MEANING.get(word, "dim"), now_words(strategy, word))
        )
    lines.append(
        paint(
            "dim",
            "  * taken on the training seeds of that generation. The seed set moves "
            "every generation, so a starred column does not compare across them.",
        )
    )
    lines.append(
        paint(
            "dim",
            "    `valid` and `vs bar` play the validation seeds, which never move. "
            "`vs bar` is the newest validation figure less the controller yardstick.",
        )
    )
    return lines


def curves(reading: Reading, paint: Paint) -> list[str]:
    """Return one line for each strategy that has a validation series.

    Every other column moves with the seed set of its generation. These play
    the same seeds every time, so this is the row that says whether the run
    is learning.
    """
    lines: list[str] = []
    for name in sorted(reading.strategies):
        strategy = reading.strategies[name]
        series = strategy.validations
        if not series:
            continue
        if not lines:
            lines.append("")
            lines.append(paint("head", "  --- the validation seeds, which never move"))
        body = " ".join(f"g{number}:{value:.0f}".ljust(13) for number, value in series)
        margin = strategy.margin
        verdict = "   bar not measured yet"
        if margin is not None:
            verdict = paint(
                "good" if margin > 0 else "bad",
                f"   {'above' if margin > 0 else 'below'} the bar "
                f"{strategy.yardstick:.0f} by {abs(margin):.0f}",
            )
        lines.append(f"  {name:<19}{body}{verdict}")
    if lines:
        lines.append(
            paint(
                "dim",
                "  The bar plays the built-in controller in every seat, so it is the "
                "chance line and not a standard of play.",
            )
        )
    return lines


def recent(reading: Reading, keep: int, paint: Paint) -> list[str]:
    """Return the newest finished generations across every strategy."""
    lines = [
        "",
        paint("head", "  --- recent generations"),
        paint(
            "head",
            f"  {'strategy':<19}{'gen':>4} {'mean*':>10} {'best*':>10} "
            f"{'spread*':>9} {'won*':>5} {'valid':>9}",
        ),
    ]
    rows: list[tuple[int, str, str]] = []
    for name in sorted(reading.strategies):
        for row in reading.strategies[name].done:
            spread = "        -" if row.spread is None else f"{row.spread:9.1f}"
            won = "    -" if row.won is None else f"{row.won:5.2f}"
            valid = "        -" if row.validation is None else f"{row.validation:9.1f}"
            rows.append(
                (
                    row.generation,
                    name,
                    f"  {name:<19}{row.generation:>4} {row.mean:>10.1f} "
                    f"{row.best:>10.1f} {spread} {won} {valid}",
                )
            )
    rows.sort(key=lambda row: (row[0], row[1]))
    if not rows:
        lines.append("  none finished yet. The rows above say what is running.")
        return lines
    for _, _, text in rows[-keep:]:
        lines.append(text)
    return lines


def render(
    reading: Reading, facts: Facts, keep: int = 15, paint: Paint | None = None
) -> str:
    """Return the whole screen.

    The painter is a parameter so that a test reads plain text and a
    terminal reads colour from the same code.
    """
    paint = paint or Paint(False)
    states = {
        name: state_of(reading, strategy, facts.log_quiet)
        for name, strategy in reading.strategies.items()
    }
    lines = header(reading, states, facts, paint)
    lines.append("")
    lines.extend(table(reading, states, paint))
    lines.extend(recent(reading, keep, paint))
    lines.extend(curves(reading, paint))
    return "\n".join(lines)


def main() -> int:
    """Read the log the caller names, and print the screen."""
    parser = argparse.ArgumentParser(description="Render one training watch screen.")
    parser.add_argument("log", type=Path)
    parser.add_argument("--price", type=float, default=0.0)
    parser.add_argument(
        "--generations",
        type=int,
        default=100,
        help="how many generations the whole run holds, across every strategy",
    )
    parser.add_argument(
        "--elapsed",
        type=float,
        default=0.0,
        help="the wall clock seconds since the run started",
    )
    parser.add_argument("--facts", type=str, default="training run")
    parser.add_argument(
        "--link",
        choices=["live", "cached", "silent", "gone", "local", "unknown"],
        default="unknown",
        help="what the caller knows about reaching the instance",
    )
    parser.add_argument(
        "--asked",
        type=float,
        default=-1.0,
        help="seconds since the caller last got an answer from the instance",
    )
    parser.add_argument(
        "--log-quiet",
        type=float,
        default=-1.0,
        help="seconds since the run last wrote a line, measured on the instance",
    )
    parser.add_argument("--load", type=float, default=-1.0)
    parser.add_argument("--cores", type=int, default=0)
    parser.add_argument("--recent", type=int, default=15)
    parser.add_argument(
        "--colour",
        choices=["auto", "always", "never"],
        default="auto",
        help="auto writes colour to a terminal only. NO_COLOR wins over all three.",
    )
    arguments = parser.parse_args()

    paint = Paint(wants_colour(arguments.colour, sys.stdout))

    text = ""
    if arguments.log.exists():
        text = arguments.log.read_text(errors="replace")
    facts = Facts(
        heading=arguments.facts,
        link=arguments.link,
        asked=arguments.asked,
        log_quiet=arguments.log_quiet,
        elapsed=arguments.elapsed,
        price=arguments.price,
        generations=arguments.generations,
        load=arguments.load,
        cores=arguments.cores,
    )
    print(render(read(text), facts, arguments.recent, paint))
    return 0


if __name__ == "__main__":
    sys.exit(main())
