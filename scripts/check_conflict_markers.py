#!/usr/bin/env python3
"""Fail when a merge conflict marker reaches the tree.

The decisions register carried an unresolved marker in its reference section,
on the main branch, across several commits. A reader found it while rebasing.
Every gate ran green over it each time, because the register checks read the
entries and the record checks read the records, and neither reads the shape of
a line.[^1]

A gate that passes is evidence that the rules the gate encodes hold, and
nothing more. This check encodes one more rule: a merge is finished when the
markers are gone.

It reads every file in the tree, not a list of directories somebody thought
of. A marker in a source file, a test fixture or a register is one defect.

It matches the four markers git writes at the start of a line. Three come from
the default conflict style, and the fourth from the `diff3` and `zdiff3`
styles, which a contributor may set in a local configuration:

  seven `<`   the start of the ours side
  seven `|`   the base, in the diff3 styles
  seven `=`   the divider
  seven `>`   the end of the theirs side

A run of eight or more does not match, so a rule of `=` characters under a
heading is safe.

Give a directory as the first argument to scan that tree instead of the
repository. The repository scan passes over the broken fixture by name, and an
explicit scan of it reads it, which is how the probe recipe proves the check
can fail.

Exit 0 when the tree holds no marker, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-conflict-markers.sh`.

# References

[^1]: Findings register, FND-136. `docs/FINDINGS.md`
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Directories that hold no project text. A build directory and a virtual
# environment hold third-party files, and reading them says nothing about this
# tree.
SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".ruff_cache"}

SKIP_PATHS = {
    # A worktree holds another checkout of this repository. A worker mid-rebase
    # legitimately holds a marker there, and reading it blames the wrong tree.
    ROOT / ".claude" / "worktrees",
    # Deliberately broken. The probe recipe scans this directory on purpose and
    # fails when the check passes over it.
    ROOT / "tests" / "fixtures" / "conflict-broken",
    # This file states the markers as a pattern. The pattern is not a marker,
    # but the name is skipped so that no future edit of the docstring can turn
    # the check against itself.
    Path(__file__).resolve(),
}

# Exactly seven characters at the start of a line, followed by a space or by
# the end of the line. A longer run is a rule or a fence, not a marker.
MARKER = re.compile(r"^(<{7}|\|{7}|={7}|>{7})(?![<|=>])(?: |$)")

# A file larger than this is data, not text somebody merged by hand.
LIMIT = 4 * 1024 * 1024


def is_skipped(path: Path, scan: Path) -> bool:
    """Say whether the scan passes over this path.

    A path in SKIP_PATHS is skipped during a scan of the repository. An
    explicit scan of one of those directories reads it, because that is the
    only way a fixture can prove the check can fail.
    """
    if any(part in SKIP_DIRS for part in path.parts):
        return True
    if scan != ROOT:
        return False
    return any(path == p or p in path.parents for p in SKIP_PATHS)


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def main() -> int:
    scan = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT
    if not scan.is_dir():
        print(f"no such directory: {scan}", file=sys.stderr)
        return 2

    failures: list[str] = []
    read = 0

    for path in sorted(scan.rglob("*")):
        if not path.is_file() or path.is_symlink():
            continue
        if is_skipped(path, scan):
            continue
        try:
            if path.stat().st_size > LIMIT:
                continue
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        read += 1
        for number, line in enumerate(text.splitlines(), start=1):
            if MARKER.match(line):
                failures.append(
                    f"{relative(path)}:{number}: a merge conflict marker "
                    f"({line.strip()[:40]!r})"
                )

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    # A clean report is not a clean tree unless the reader knows what the scan
    # passed over. A scan from the repository root skips every worktree, so a
    # worker whose files live in one gets a clean answer about files this run
    # never opened. Naming the skipped roots makes that visible in the output
    # rather than leaving it to be rediscovered.
    skipped = sorted(
        relative(p) for p in SKIP_PATHS if scan == ROOT and p.is_dir() and p.exists()
    )
    for name in skipped:
        print(f"note: {name} was not read. Run the check there to cover it")

    print(f"\nchecked {read} files for a conflict marker: {len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
