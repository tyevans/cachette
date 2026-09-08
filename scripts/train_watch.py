#!/usr/bin/env python3
"""Render one screen that says what a training run is doing and what it did.

A training run starts one trainer process for each strategy, and they all
append to one log. This script reads that log and prints a screen small
enough for a one second refresh.

# What the screen answers

A reader of a run asks four questions, and the screen answers them in the
order that they matter.

1. **Is it working?** Every strategy prints a line while a generation runs,
   so a strategy that says nothing has stopped. The screen names the age of
   the last word from each one.
2. **How fast?** Each line carries the ticks a second that the strategy
   reached. The screen adds them, and that sum is what the machine reached.
3. **What has it learned?** The held-out bar is what the built-in controller
   scored. A strategy beats it or it does not.
4. **What does it cost?** The price an hour and the time so far give the
   money, and the generations left give the rest.

# What it does not do

It takes no measurement of its own and it reaches no machine. It reads a log
file. The caller brings the log.
"""

from __future__ import annotations

import argparse
import re
import sys
import time
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


def fields(body: str) -> dict[str, float | None]:
    """Return the named fields of a row, with a bare dash as no value."""
    found: dict[str, float | None] = {}
    for match in FIELD.finditer(body):
        text = match.group("value")
        found[match.group("key")] = None if text == "-" else float(text)
    return found


# A heartbeat from inside a generation, for example:
#   conquer generation  3 working  decisions 120 live 87/144 ticks 174000
#   rate 1893.0 t/s [92s]
WORKING = re.compile(
    r"^\s+(?P<name>\S+) (?P<what>generation\s+\d+|yardstick|validation\s+\d+)"
    r" working\s+decisions\s+(?P<decisions>\d+)\s+"
    r"live\s+(?P<live>\d+)/(?P<worlds>\d+)\s+"
    r"ticks\s+(?P<ticks>\d+)\s+rate\s+(?P<rate>[\d.]+) t/s\s+"
    r"\[(?P<seconds>[\d.]+)s\]"
)

# The controller yardstick, printed once for each strategy.
YARDSTICK = re.compile(r"^\s+(?P<name>\S+) controller yardstick\s+(?P<value>-?[\d.]+)")

# The held-out controller row, printed once before any training.
CONTROLLER = re.compile(r"^\s+controller (?P<body>\{.*\})\s*$")


@dataclass
class Strategy:
    """What one strategy has done and is doing."""

    name: str
    # One row for each finished generation: the number, the mean, the best,
    # the spread, the win share, and the validation score when the generation
    # validated. **The validation is the only figure that compares across
    # generations.** The seed set moves every generation, so the mean and the
    # win share answer a different question each time. The validation seeds
    # never move.
    done: list[tuple[int, float, float, float | None, float | None, float | None]] = (
        field(default_factory=list)
    )
    last_working: dict[str, str] | None = None
    yardstick: float | None = None
    validations: list[float] = field(default_factory=list)


def read(text: str) -> tuple[dict[str, Strategy], dict[str, float] | None]:
    """Return what each strategy has reached, and the controller row."""
    strategies: dict[str, Strategy] = {}
    controller: dict[str, float] | None = None

    def named(name: str) -> Strategy:
        found = strategies.get(name)
        if found is None:
            found = Strategy(name=name)
            strategies[name] = found
        return found

    for line in text.splitlines():
        row = DONE.match(line)
        if row and "working" not in row.group("body"):
            values = fields(row.group("body"))
            if "mean" not in values or "best" not in values:
                continue
            strategy = named(row.group("name"))
            mean = values["mean"]
            best = values["best"]
            if mean is None or best is None:
                continue
            valid = values.get("valid")
            strategy.done.append(
                (
                    int(row.group("generation")),
                    mean,
                    best,
                    values.get("spread"),
                    values.get("won"),
                    valid,
                )
            )
            if valid is not None:
                strategy.validations.append(valid)
            continue
        work = WORKING.match(line)
        if work:
            named(work.group("name")).last_working = work.groupdict()
            continue
        yard = YARDSTICK.match(line)
        if yard:
            named(yard.group("name")).yardstick = float(yard.group("value"))
            continue
        if controller is None:
            found = CONTROLLER.match(line)
            if found:
                try:
                    import json

                    controller = json.loads(found.group("body").replace("'", '"'))
                except ValueError:
                    controller = None
    return strategies, controller


def clock(seconds: float) -> str:
    """Return a duration as hours and minutes."""
    return f"{int(seconds) // 3600}h{(int(seconds) % 3600) // 60:02d}m"


def render(
    strategies: dict[str, Strategy],
    controller: dict[str, float] | None,
    price: float,
    generations: int,
    elapsed: float,
    log_age: float,
    facts: str,
    recent: int,
) -> str:
    """Return the screen."""
    lines: list[str] = []
    spent = price * elapsed / 3600.0
    lines.append(f"=== {facts}")

    total_done = sum(len(s.done) for s in strategies.values())
    # The caller gives the target for the whole run, not for one strategy, so
    # this script holds no count of the strategies. The log names them.
    target = generations
    rate_now = 0.0
    for strategy in strategies.values():
        if strategy.last_working:
            rate_now += float(strategy.last_working["rate"])

    left = ""
    if total_done and elapsed:
        # **This rate already holds the parallelism.** It is the wall clock
        # divided by every generation that finished anywhere, so the strategies
        # running at once are counted in it. Dividing again by the strategy
        # count was wrong, and it reported a quarter of the time that was left.
        each = elapsed / total_done
        remaining = max(target - total_done, 0) * each
        left = f"   left ~{clock(remaining)}"
    lines.append(
        f"  elapsed {clock(elapsed)}   spent ${spent:.2f}   "
        f"generations {total_done}/{target}{left}"
    )
    lines.append(
        f"  machine rate {rate_now:9.1f} ticks/s across "
        f"{sum(1 for s in strategies.values() if s.last_working)} live processes"
        + (f"   log age {log_age:.0f}s" if log_age >= 0 else "")
    )
    if controller:
        lines.append(
            f"  bar to beat   controller wins {controller.get('won', 0.0):.3f} "
            f"return {controller.get('return', 0.0):9.1f}"
        )
    lines.append("")

    header = (
        f"  {'strategy':<14}{'gen':>5} {'mean':>10} {'best':>10} "
        f"{'spread':>9} {'won':>5}  now"
    )
    recent_header = (
        f"  {'strategy':<14}{'gen':>5} {'mean':>10} {'best':>10} "
        f"{'spread':>9} {'won':>5} {'valid':>9}"
    )
    lines.append(header)
    for name in sorted(strategies):
        strategy = strategies[name]
        if strategy.done:
            generation, mean, best, spread, won, _ = strategy.done[-1]
            spread_text = "        -" if spread is None else f"{spread:9.1f}"
            won_text = "    -" if won is None else f"{won:5.2f}"
            body = (
                f"{generation:>5} {mean:>10.1f} {best:>10.1f} {spread_text} {won_text}"
            )
        else:
            body = f"{'-':>5} {'-':>10} {'-':>10} {'-':>9} {'-':>5}"
        now = "no word yet"
        if strategy.last_working:
            work = strategy.last_working
            now = (
                f"{work['what']} d{work['decisions']} "
                f"live {work['live']}/{work['worlds']} "
                f"{float(work['rate']):.0f}t/s [{float(work['seconds']):.0f}s]"
            )
        lines.append(f"  {name:<14}{body}  {now}")

    lines.append("")
    lines.append("  --- recent generations")
    lines.append(recent_header)
    rows: list[tuple[int, str, str]] = []
    for name in sorted(strategies):
        for generation, mean, best, spread, won, valid in strategies[name].done:
            spread_text = "        -" if spread is None else f"{spread:9.1f}"
            won_text = "    -" if won is None else f"{won:5.2f}"
            valid_text = "        -" if valid is None else f"{valid:9.1f}"
            rows.append(
                (
                    generation,
                    name,
                    f"  {name:<14}{generation:>5} {mean:>10.1f} {best:>10.1f} "
                    f"{spread_text} {won_text} {valid_text}",
                )
            )
    rows.sort(key=lambda row: (row[0], row[1]))
    for _, _, text in rows[-recent:]:
        lines.append(text)
    if not rows:
        lines.append("  none finished yet. The rows above say what is running.")

    # **The learning curve.** Every other column moves with the seed set of
    # its generation, so two of them do not compare. These play the same
    # seeds every time, so this row is the one that says whether the run is
    # learning.
    for name in sorted(strategies):
        strategy = strategies[name]
        series = [
            (generation, valid)
            for generation, _, _, _, _, valid in strategy.done
            if valid is not None
        ]
        if not series:
            continue
        lines.append("")
        yard = strategy.yardstick
        lines.append(
            f"  --- {name}: the validation seeds, which never move"
            + (f"   chance reaches {yard:.1f}" if yard is not None else "")
        )
        lines.append(
            "      "
            + "  ".join(f"g{generation}:{value:.0f}" for generation, value in series)
        )
        if len(series) >= 2:
            first, last = series[0][1], series[-1][1]
            moved = last - first
            way = "up" if moved > 0 else "down"
            gap = f", {yard - last:.0f} from chance" if yard is not None else ""
            lines.append(
                f"      moved {moved:+.0f} over {len(series)} validations, {way}{gap}"
            )

    for name in sorted(strategies):
        strategy = strategies[name]
        if strategy.validations and strategy.yardstick is not None:
            best = max(strategy.validations)
            # **The yardstick is the chance line, not a measure of skill.** The
            # yardstick world gives the learner seat back to the built-in
            # controller, so every faction of that game is the same controller.
            # One seat of a symmetric game takes one share of the wins for each
            # faction, whatever the controller does. A screen that reads
            # "beats the controller" invites a reader to take chance for
            # quality, and it did.
            verdict = "above" if best > strategy.yardstick else "below"
            lines.append(
                f"  {name:<14}validation best {best:9.1f} "
                f"{verdict} the yardstick {strategy.yardstick:9.1f}"
            )
            lines.append(
                f"  {'':<14}the yardstick plays the controller in every seat, "
                f"so it is the chance line and not a standard of play"
            )
    return "\n".join(lines)


def main() -> int:
    """Read the log the caller names, and print the screen."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--price", type=float, default=0.0)
    parser.add_argument(
        "--generations",
        type=int,
        default=100,
        help="how many generations the whole run holds, across every strategy",
    )
    parser.add_argument("--started", type=float, default=0.0)
    parser.add_argument("--facts", type=str, default="training run")
    parser.add_argument("--recent", type=int, default=15)
    arguments = parser.parse_args()

    if not arguments.log.exists():
        print(f"no log at {arguments.log} yet")
        return 0
    text = arguments.log.read_text(errors="replace")
    age = time.time() - arguments.log.stat().st_mtime
    elapsed = time.time() - arguments.started if arguments.started else 0.0
    strategies, controller = read(text)
    print(
        render(
            strategies,
            controller,
            arguments.price,
            arguments.generations,
            elapsed,
            age,
            arguments.facts,
            arguments.recent,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
