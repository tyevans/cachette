#!/usr/bin/env python3
"""Check the backlog numbering and the front matter of each item.

One item is one file, and the three directories are the index. There is no
separate registry, so nothing but this script can tell that two items took
one number.

The rule that allocates a number reads the highest number and adds one. That
rule is correct when one person works at a time. It has no defence against
two people who read the same highest number, and it gives no signal when they
both act on it: both files exist, both look right, and the collision is
visible only to somebody who lists the directory.[^1]

The front matter states the number and the status a second time. The directory
holds the status and the file name holds the number, so each of those two facts
has two declaration sites. A move by `git mv` changes one site and leaves the
other, and nothing fails.[^2] This script compares the copies.

It cannot tell a done item from an open one. Only a reader of the code can do
that, and the register records what the absence of such a check has cost.[^3]

# References

[^1]: Backlog guide. `docs/backlog/README.md`
[^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^3]: Findings register, FND-526 and FND-534. `docs/FINDINGS.md`
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BACKLOG = ROOT / "docs" / "backlog"
DIRECTORIES = ("proposed", "refined", "complete")
NAME = re.compile(r"^(\d{4})-[a-z0-9-]+\.md$")
FIELD = re.compile(r"^(id|status):[ \t]*(\S*)[ \t]*$", re.M)


def relative(path: Path) -> str:
    """Return the path as the repository sees it, or whole if it sits outside."""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def front_matter(item: Path) -> dict[str, str]:
    """Return the id and the status the front matter of one item declares.

    The front matter ends at the second line that holds three dashes. A field
    the file does not declare is absent from the result, so the caller can tell
    a missing field from an empty one.
    """
    text = item.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---", 4)
    block = text[4:end] if end != -1 else text
    return {match.group(1): match.group(2) for match in FIELD.finditer(block)}


def declared(item: Path, number: str, directory: str) -> list[str]:
    """Return one failure for each front matter field the tree contradicts.

    The file name holds the number and the directory holds the status. The
    front matter holds both a second time. Nothing else compares the copies,
    so a `git mv` that changes the directory and leaves the status behind is
    silent, and the item then states a status that is not its own.
    """
    fields = front_matter(item)
    where = relative(item)
    failures = []
    for name, expected in (("id", number), ("status", directory)):
        actual = fields.get(name)
        if actual is None:
            failures.append(f"{where}: the front matter declares no {name}")
        elif actual != expected:
            failures.append(
                f"{where}: the front matter says {name} {actual!r},"
                f" and the tree says {expected!r}"
            )
    return failures


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
            failures.extend(declared(item, match.group(1), directory))
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
