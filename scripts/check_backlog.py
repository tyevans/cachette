#!/usr/bin/env python3
"""Check the backlog numbering.

One item is one file, and the three directories are the index. There is no
separate registry, so nothing but this script can tell that two items took
one number.

The rule that allocates a number reads the highest number and adds one. That
rule is correct when one person works at a time. It has no defence against
two people who read the same highest number, and it gives no signal when they
both act on it: both files exist, both look right, and the collision is
visible only to somebody who lists the directory.[^1]

# References

[^1]: Backlog guide. `docs/backlog/README.md`
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BACKLOG = ROOT / "docs" / "backlog"
DIRECTORIES = ("proposed", "refined", "complete")
NAME = re.compile(r"^(\d{4})-[a-z0-9-]+\.md$")


def relative(path: Path) -> str:
    """Return the path as the repository sees it, or whole if it sits outside."""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def main() -> int:
    backlog = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else BACKLOG
    if not backlog.is_dir():
        print(f"no such directory: {backlog}", file=sys.stderr)
        return 2

    by_number: dict[str, list[Path]] = defaultdict(list)
    failures: list[str] = []
    counted = 0

    for directory in DIRECTORIES:
        path = backlog / directory
        if not path.is_dir():
            continue
        for item in sorted(path.glob("*.md")):
            match = NAME.match(item.name)
            if not match:
                failures.append(
                    f"{relative(item)}: the name is not NNNN-short-slug.md"
                )
                continue
            by_number[match.group(1)].append(item)
            counted += 1

    for number, items in sorted(by_number.items()):
        if len(items) > 1:
            where = ", ".join(relative(i) for i in items)
            failures.append(f"{number} names more than one item: {where}")

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    print(f"\nchecked {counted} backlog items: {len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
