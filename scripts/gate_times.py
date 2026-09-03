#!/usr/bin/env python3
"""Times each gate recipe separately and reports each against the whole.

The gate suite reports one figure: the wall clock of the whole run against a
budget.[1] That figure says a gate grew. It does not say which one. This
script splits the run into its recipes and its command lines, so that a
reader can name the gate that holds the cost.

The recipes come from the justfile, read through `just --dump`.[2] Nothing
here restates a command. A second copy of a gate command would decay, because
nothing fails when two copies disagree.[3]

`just` runs each line of a recipe body in its own shell. This script does the
same, with the shell the justfile sets. A run here is therefore the same work
in the same order as the suite, and the sum of the rows is the cost of the
suite plus the small cost of starting `just` once for each dependency.

**A figure this script prints describes one machine at one moment.** Wall
clock on a loaded machine is not evidence. The report prints the load average
before and after the run so that a reader can throw a contended figure away.
The script never fails on a figure, for the reason the testing rule gives: a
timing assertion teaches everyone to ignore a red pipeline.[4]

References

1. The cost report. scripts/gate-budget.sh
2. The gate recipes. justfile
3. Recurring defect shapes, section 1. .claude/rules/recurring-defects.md
4. Testing rules, section 3. .claude/rules/testing.md
5. Development budgets, the local register. docs/reference/development-budgets.md
"""

from __future__ import annotations

import argparse
import json
import platform
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Every figure a run reports about itself, in the output of the tools the
# gates call. The wall clock this script measures includes the process start,
# the fingerprint check and the link; the inner figure does not. The gap
# between them is the overhead of the gate rather than the work in it.
INNER_TIME = re.compile(
    r"(?:finished in|passed in|failed in|target\(s\) in) ([0-9]+(?:\.[0-9]+)?)s"
)
COMPILING = re.compile(r"^\s*(?:Compiling|Building) ")


@dataclass
class Row:
    """One command line of one recipe, and what it cost."""

    recipe: str
    command: str
    seconds: float = 0.0
    inner_seconds: float = 0.0
    compiled: int = 0
    status: int = 0
    share: float = 0.0

    def as_dict(self) -> dict[str, object]:
        return {
            "recipe": self.recipe,
            "command": self.command,
            "seconds": round(self.seconds, 2),
            "inner_seconds": round(self.inner_seconds, 2),
            "compiled": self.compiled,
            "status": self.status,
            "share": round(self.share, 4),
        }


@dataclass
class Plan:
    """The leaf recipes of one target, in the order the suite runs them."""

    target: str
    shell: list[str]
    rows: list[Row] = field(default_factory=list)


def read_justfile() -> dict:
    """Read the justfile as data. This is the one source for a gate command."""
    done = subprocess.run(
        ["just", "--dump", "--dump-format", "json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        sys.exit(f"just --dump failed: {done.stderr.strip()}")
    return json.loads(done.stdout)


def shell_words(dump: dict) -> list[str]:
    """Give the shell the justfile sets, or the default just uses."""
    setting = dump.get("settings", {}).get("shell")
    if isinstance(setting, dict) and setting.get("command"):
        return [setting["command"], *setting.get("arguments", [])]
    return ["sh", "-cu"]


def leaves(dump: dict, name: str, seen: set[str] | None = None) -> list[str]:
    """Give the recipes that hold commands, in the order just runs them.

    A recipe with dependencies contributes its dependencies, not itself. The
    gate target is such a recipe: it holds no body of its own.
    """
    seen = set() if seen is None else seen
    if name in seen:
        return []
    seen.add(name)
    recipes = dump["recipes"]
    if name not in recipes:
        sys.exit(f"the justfile holds no recipe named {name}")
    recipe = recipes[name]
    found: list[str] = []
    for dependency in recipe["dependencies"]:
        found.extend(leaves(dump, dependency["recipe"], seen))
    if recipe["body"]:
        found.append(name)
    return found


def body_lines(dump: dict, name: str) -> list[str]:
    """Give the command lines of one recipe body.

    A fragment that is not a plain string is an interpolation, and a recipe
    that holds one takes a parameter. This script refuses such a recipe rather
    than choosing a value for it, because the value would be a second
    declaration of something the justfile already states.
    """
    lines: list[str] = []
    for fragments in dump["recipes"][name]["body"]:
        if not all(isinstance(fragment, str) for fragment in fragments):
            sys.exit(
                f"recipe {name} interpolates a parameter, so this script "
                "cannot time it without inventing a value"
            )
        line = "".join(fragments).strip()
        if not line or line.startswith("#"):
            continue
        while line[:1] in {"@", "-"}:
            line = line[1:].lstrip()
        if line:
            lines.append(line)
    return lines


def build_plan(target: str, only: str | None) -> Plan:
    dump = read_justfile()
    names = [only] if only else leaves(dump, target)
    plan = Plan(target=only or target, shell=shell_words(dump))
    for name in names:
        for line in body_lines(dump, name):
            plan.rows.append(Row(recipe=name, command=line))
    return plan


def run_row(row: Row, shell: list[str]) -> None:
    """Run one line, pass its output through, and time it."""
    start = time.monotonic()
    process = subprocess.Popen(
        [*shell, row.command],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert process.stdout is not None
    for line in process.stdout:
        sys.stdout.write(line)
        sys.stdout.flush()
        for figure in INNER_TIME.findall(line):
            row.inner_seconds += float(figure)
        if COMPILING.match(line):
            row.compiled += 1
    row.status = process.wait()
    row.seconds = time.monotonic() - start


def load_average() -> str:
    try:
        one, five, fifteen = __import__("os").getloadavg()
    except (OSError, AttributeError):
        return "unknown"
    return f"{one:.2f} {five:.2f} {fifteen:.2f}"


def budget_seconds() -> int | None:
    done = subprocess.run(
        [str(ROOT / "scripts" / "gate-budget-figure.sh"), platform.machine()],
        capture_output=True,
        text=True,
    )
    if done.returncode != 0 or not done.stdout.strip():
        return None
    return int(done.stdout.strip())


def report(plan: Plan, before: str, after: str, started: str) -> None:
    total = sum(row.seconds for row in plan.rows)
    for row in plan.rows:
        row.share = row.seconds / total if total else 0.0

    by_recipe: dict[str, list[Row]] = {}
    for row in plan.rows:
        by_recipe.setdefault(row.recipe, []).append(row)

    print()
    print(f"Per-recipe cost of `just {plan.target}`.")
    print(f"Machine: {platform.machine()}, {platform.node()}. Started {started}.")
    print(f"Load average before: {before}. After: {after}.")
    compiled = sum(row.compiled for row in plan.rows)
    if compiled:
        print(
            f"{compiled} crate builds ran. This is a cold run, and the register "
            "holds a separate row for one. Do not read it as a gate that grew."
        )
    else:
        print("Nothing was rebuilt. This is the run the budget describes.")
    print()

    width = max(len(name) for name in by_recipe) if by_recipe else 10
    print(f"{'recipe'.ljust(width)}  {'wall':>8}  {'share':>6}  {'inner':>8}  built")
    print(f"{'-' * width}  {'-' * 8}  {'-' * 6}  {'-' * 8}  -----")
    for name, rows in by_recipe.items():
        seconds = sum(row.seconds for row in rows)
        inner = sum(row.inner_seconds for row in rows)
        built = sum(row.compiled for row in rows)
        share = seconds / total if total else 0.0
        print(
            f"{name.ljust(width)}  {seconds:8.1f}  {share * 100:5.1f}%  "
            f"{inner:8.1f}  {built:5d}"
        )
    print(f"{'-' * width}  {'-' * 8}  {'-' * 6}  {'-' * 8}  -----")
    inner_total = sum(row.inner_seconds for row in plan.rows)
    print(
        f"{'total'.ljust(width)}  {total:8.1f}  {100.0:5.1f}%  "
        f"{inner_total:8.1f}  {compiled:5d}"
    )

    print()
    print("The slowest command lines.")
    for row in sorted(plan.rows, key=lambda item: item.seconds, reverse=True)[:12]:
        print(f"{row.seconds:8.1f}  {row.share * 100:5.1f}%  {row.recipe}: {row.command}")

    budget = budget_seconds()
    print()
    if budget is None:
        print(f"No budget row for {platform.machine()} in the development budget")
        print("register. Measure this machine and add a row before you read this.")
    elif total > budget:
        print(f"Budget for {platform.machine()}: {budget} s.")
        print(f"The suite is over its budget by {total - budget:.0f} s.")
        print("Find the gate that grew. Do not raise the budget to cover one.")
    else:
        print(f"Budget for {platform.machine()}: {budget} s.")
        print(f"The suite is inside its budget by {budget - total:.0f} s.")

    print()
    print("The `inner` column is what each tool reported about itself. The gap")
    print("between `wall` and `inner` is what the gate spends outside its own")
    print("work: the process start, the fingerprint check and the link.")
    print("This figure describes a development machine. It is not evidence")
    print("about the target platform.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--target",
        default="gates",
        help="the recipe whose leaves are timed. It defaults to the gate suite",
    )
    parser.add_argument(
        "--recipe",
        default=None,
        help="time one recipe rather than every leaf of the target",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="print what would run, in order, and run nothing",
    )
    parser.add_argument(
        "--json",
        default=None,
        help="write the rows to this path, so two runs can be compared",
    )
    arguments = parser.parse_args()

    plan = build_plan(arguments.target, arguments.recipe)

    if arguments.list:
        print(f"`just {plan.target}` runs these lines, in this order.")
        print(f"The shell is: {' '.join(plan.shell)}")
        for row in plan.rows:
            print(f"  {row.recipe}: {row.command}")
        return 0

    started = time.strftime("%Y-%m-%d %H:%M:%S")
    before = load_average()
    failed = 0
    for row in plan.rows:
        print(f"\n=== {row.recipe}: {row.command}", flush=True)
        run_row(row, plan.shell)
        if row.status != 0:
            failed = row.status
    after = load_average()

    report(plan, before, after, started)

    if arguments.json:
        Path(arguments.json).write_text(
            json.dumps(
                {
                    "target": plan.target,
                    "machine": platform.machine(),
                    "node": platform.node(),
                    "started": started,
                    "load_before": before,
                    "load_after": after,
                    "rows": [row.as_dict() for row in plan.rows],
                },
                indent=2,
            )
            + "\n"
        )
        print(f"\nWrote the rows to {arguments.json}.")

    return failed


if __name__ == "__main__":
    raise SystemExit(main())
