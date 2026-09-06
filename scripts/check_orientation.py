#!/usr/bin/env python3
"""Fail when two copies of the project orientation disagree.

A harness reads the orientation file it is named for. One reads `AGENTS.md`,
one reads `CLAUDE.md`, one reads `GEMINI.md`. The project needs all three names
and holds one document, so the shape is one fact with more than one declaration
site, and the failure is silent: an agent reads the stale copy and reads
something false.

The project has met this once. A sweep repaired the orientation, and the
finding that recorded the sweep called the orientation one document. It was two
tracked files and a symlink. The repaired copy said what the blockers register
says. The other still said that no measurement exists on the target platform,
and every check in this repository stayed green.[^1]

The rule the project already states is the defence: declare a value once, and
when a second site must exist, add a check that fails when the copies
disagree.[^2]

## The rule this check applies

**One orientation file is canonical, and every other name is the same
document.** The canonical file is `AGENTS.md`. Each other name is a symlink to
it, which is how this repository already mirrors its rules directory and its
skills directory. A symlink cannot go stale, so the strongest form of the check
is that the copies are not copies at all.

The check does not require the symlink, because a checkout on a filesystem that
carries none must still work. It compares the content each name reads, after a
normalisation, and it fails on the first line that differs. A symlink passes
that comparison for free. A regular file passes it only while somebody keeps it
identical.

**The normalisation is two rewrites and no more.** It rewrites the `.agents`
directory to the `.claude` directory in a path, and it rewrites the name of an
orientation file. Both are the same thing named differently for a different
reader. A wider rewrite would make two files compare equal while they say
different things, so do not add a rewrite in order to make a failure go away.

The check reads the rules directory and the skills directory the same way. Each
file under the canonical directory must have a mirror that reads the same after
the rewrite.

## What this check cannot catch

- **A disagreement the normalisation erases.** The two rewrites are the design.
  A file that differs only in a path this check rewrites is treated as one file,
  by intent.
- **A copy that no name in the list reaches.** The list of orientation names is
  written down here. A fourth harness that reads a fourth name is a new line in
  this file, and nothing discovers it.
- **A document that is wrong in both copies.** The check compares two copies. It
  says nothing about whether either is true.

Give a directory as the first argument to check that tree instead of the
repository, which is how the probe recipe proves the check can fail.

Exit 0 when every pair agrees, 1 otherwise. No dependencies beyond the standard
library. Run it with `scripts/check-orientation.sh`.

# References

[^1]: Findings register, FND-259. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# The canonical orientation, and the other names a harness reads.
CANONICAL = "AGENTS.md"
MIRROR_NAMES = ("CLAUDE.md", "GEMINI.md")

# The canonical directory, and the directory that mirrors it.
CANONICAL_DIR = ".agents"
MIRROR_DIR = ".claude"
MIRRORED_TREES = ("rules", "skills")


def normalise(text: str) -> list[str]:
    """Rewrite the directory name and the orientation file name.

    Two harnesses read the same document under two names, and each names its
    own directory inside it. Nothing else is rewritten.
    """
    text = text.replace(f"{CANONICAL_DIR}/", f"{MIRROR_DIR}/")
    for name in MIRROR_NAMES:
        text = text.replace(name, CANONICAL)
    return text.splitlines()


def compare(name: str, canonical: Path, mirror: Path, failures: list[str]) -> None:
    if not mirror.exists():
        failures.append(f"{name}: no such file, and {canonical.name} expects a mirror")
        return
    left = normalise(canonical.read_text(encoding="utf-8"))
    right = normalise(mirror.read_text(encoding="utf-8"))
    for number, (a, b) in enumerate(zip(left, right), start=1):
        if a != b:
            failures.append(
                f"{name}: line {number} differs from the canonical copy\n"
                f"        canonical: {a.strip()}\n"
                f"        this copy: {b.strip()}"
            )
            return
    if len(left) != len(right):
        longer, shorter = ("the canonical copy", name)
        if len(right) > len(left):
            longer, shorter = (name, "the canonical copy")
        failures.append(
            f"{name}: {longer} holds {abs(len(left) - len(right))} lines that "
            f"{shorter} does not, after line {min(len(left), len(right))}"
        )


def main() -> int:
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT
    if not root.is_dir():
        print(f"no such directory: {root}", file=sys.stderr)
        return 2

    failures: list[str] = []
    pairs = 0

    canonical = root / CANONICAL
    if not canonical.is_file():
        print(f"FAIL: {CANONICAL}: the canonical orientation is missing", file=sys.stderr)
        return 1

    for name in MIRROR_NAMES:
        mirror = root / name
        if not mirror.exists() and root != ROOT:
            # A fixture holds the pair it is testing and nothing else.
            continue
        pairs += 1
        compare(name, canonical, mirror, failures)

    for tree in MIRRORED_TREES:
        source = root / CANONICAL_DIR / tree
        if not source.is_dir():
            continue
        for path in sorted(source.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(source)
            pairs += 1
            compare(
                f"{MIRROR_DIR}/{tree}/{relative}",
                path,
                root / MIRROR_DIR / tree / relative,
                failures,
            )

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    print(f"\ncompared {pairs} orientation copies: {len(failures)} disagree")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
