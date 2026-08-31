#!/usr/bin/env python3
"""Check the numbering of the three registers.

The findings, blockers and decisions registers each hold numbered rows, and
each states the next free number in its own text. That pointer is an
allocator, and it is a second declaration site for a value the rows already
carry.

Nothing failed when the two disagreed. A finding was written, the register
was later restored from an older copy, and the entry went out of the file
without any check noticing. The pointer stayed behind and named a number that
an entry already used.[^1]

The rule this enforces is the one the backlog check enforces for backlog
items: a number names one thing, and a value derived from the tree needs
something that fails when the copies disagree.[^2]

# References

[^1]: Findings register, FND-052. `docs/FINDINGS.md`
[^2]: The backlog check. `scripts/check_backlog.py`
"""

import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REGISTERS = (
    ("FND", ROOT / "docs" / "FINDINGS.md"),
    ("BLK", ROOT / "docs" / "BLOCKERS.md"),
    ("DEC", ROOT / "docs" / "DECISIONS.md"),
)


def check(prefix: str, path: Path) -> list[str]:
    if not path.is_file():
        return [f"{path.name}: the register is missing"]

    text = path.read_text(encoding="utf-8")
    entries = re.findall(rf"^#{{2,4}}\s*{prefix}-(\d+)\b", text, re.M)
    failures: list[str] = []

    if not entries:
        return [f"{path.name}: the register holds no {prefix} entry"]

    for number, count in sorted(Counter(entries).items()):
        if count > 1:
            failures.append(
                f"{path.name}: {prefix}-{number} names {count} entries"
            )

    pointer = re.search(rf"Next number:\s*{prefix}-(\d+)", text)
    if pointer is None:
        failures.append(f"{path.name}: the register states no next number")
        return failures

    width = len(pointer.group(1))
    expected = max(int(n) for n in entries) + 1
    stated = int(pointer.group(1))
    if stated != expected:
        failures.append(
            f"{path.name}: the next number is stated as {prefix}-{stated:0{width}d}, "
            f"but the highest entry is {prefix}-{max(int(n) for n in entries):0{width}d}, "
            f"so the next number is {prefix}-{expected:0{width}d}"
        )
    return failures


def main() -> int:
    failures: list[str] = []
    counted = 0
    for prefix, path in REGISTERS:
        found = check(prefix, path)
        failures += found
        if path.is_file():
            counted += len(re.findall(
                rf"^#{{2,4}}\s*{prefix}-\d+\b",
                path.read_text(encoding="utf-8"),
                re.M,
            ))

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    print(f"\nchecked {counted} register entries: {len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
