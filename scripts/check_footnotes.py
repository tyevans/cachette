#!/usr/bin/env python3
"""Check the footnotes of every Markdown document in the tree.

The documentation rule states that a document numbers its footnotes in the
order they occur in the body, and that it never repeats one: reuse the marker
when one source supports more than one claim.[^1] Nothing checked either. A
review of four drafts found that three broke one rule or both, and the gate
passed on all three.[^2]

A parallel run makes the shape common rather than rare. Two branches that each
append an entry to one register take the same next label, and the merge puts
both definitions in one reference section.

The check reads Markdown only. A footnote inside a Python or Rust docstring is
not read, because a comment marker in front of a definition is a second syntax
and the citation check already reads those files for a different rule.[^3]

It fails on:

  undefined   the body cites a marker that the document does not define. A
              reader sees the raw label
  duplicate   the document defines one label twice. This is the collision
              shape after a merge keeps both sides
  repeated    two labels of one document hold the same definition text. This
              is the rule against repeating a footnote
  uncited     the document defines a label that no marker cites. This is the
              collision shape after a merge renumbers one side and misses its
              marker

It reports, without failing:

  order       the labels of a document, taken in the order of first occurrence
              in the body, do not ascend

**The ordering test reports and does not fail, on purpose.** Documents across
every directory break it, three of them belong to the project owner, and the
repair is a renumbering sweep across a whole document, which is the operation
this project gets wrong most often.[^4] A gate nobody can turn green is a gate
everybody learns to skip. Run with `--notes` to list the documents.

The ordering test reads a document only when every label in it is a number. A
register that labels a footnote by the entry that owns it carries no order, so
it cannot be out of order. The test reads the order of first occurrence and
not the completeness of the sequence, so a gap does not report.

The failing tests carry a baseline of the labels that already break them, in
`scripts/footnote-baseline.txt`. The baseline is falsifiable: an entry that
matches nothing fails, so the list can only shrink and can never go stale. Do
not add to it. Repair the document instead.

Give a directory as the first argument to scan that tree instead of the
repository. The default baseline is not applied to an explicit scan, which is
how the probe recipe proves the four failing tests can fail. Set
`CACHETTE_FOOTNOTE_BASELINE` to read another baseline, which is applied to any
scan, and which is how the probe recipe proves a stale baseline entry fails.

Exit 0 when every test passes, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-footnotes.sh`.

# References

[^1]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^2]: Findings register, FND-130. `docs/FINDINGS.md`
[^3]: The citation check. `scripts/check_citations.py`
[^4]: Recurring Defect Shapes, shape 2. `.claude/rules/recurring-defects.md`
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BASELINE = Path(__file__).resolve().parent / "footnote-baseline.txt"
# The probe recipe points the check at a baseline that names a failure the tree
# does not hold, and requires the check to reject it. That is the proof that
# the baseline cannot go stale.
OVERRIDE = os.environ.get("CACHETTE_FOOTNOTE_BASELINE")
BASELINE = Path(OVERRIDE).resolve() if OVERRIDE else DEFAULT_BASELINE

SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".ruff_cache"}
SKIP_PATHS = {
    # A worktree holds another checkout of this repository. Reading it reports
    # one failure against two paths and blames the wrong tree.
    ROOT / ".claude" / "worktrees",
    # Deliberately broken. The probe recipe scans these on purpose and fails
    # when the check passes over them.
    ROOT / "tests" / "fixtures",
    # A tool's session scratch, not project prose. Nobody maintains it, and
    # the citation check passes over it for the same reason.[^3]
    ROOT / "docs" / "superpowers",
    ROOT / ".claude" / "tackline",
}

DEFINITION = re.compile(r"^\s{0,3}\[\^([^\]\s]+)\]:")
MARKER = re.compile(r"\[\^([^\]\s]+)\]")
FENCE = re.compile(r"^\s*(?:```|~~~)")
CODE_SPAN = re.compile(r"`+[^`\n]*`+")


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def is_skipped(path: Path, scan: Path) -> bool:
    if any(part in SKIP_DIRS for part in path.parts):
        return True
    if scan != ROOT:
        return False
    return any(path == p or p in path.parents for p in SKIP_PATHS)


def readable(text: str) -> list[str]:
    """Return the lines with the code removed and the offsets kept.

    A fenced block becomes empty lines, and the contents of a code span become
    spaces. A line number taken from the result is the line number in the file.
    The documentation rule exempts both, and a document that discusses a
    footnote must be able to quote a marker.
    """
    out: list[str] = []
    fenced = False
    for line in text.splitlines():
        if FENCE.match(line):
            fenced = not fenced
            out.append("")
            continue
        if fenced:
            out.append("")
            continue
        out.append(CODE_SPAN.sub(lambda m: " " * len(m.group(0)), line))
    return out


class Document:
    """One Markdown document, read for its footnotes."""

    def __init__(self, path: Path, text: str) -> None:
        self.path = path
        self.lines = readable(text)
        # The same lines with the code left in. A footnote names its source in
        # a code span, so two footnotes that read alike in prose and name two
        # different files are two sources, not one.
        self.raw = text.splitlines()
        # Label to the line that first defines it, in the order defined.
        self.defined: dict[str, int] = {}
        # Label to the source text it names, for the first definition.
        self.source: dict[str, str] = {}
        # A label defined more than once, and the line of the repeat.
        self.repeats: list[tuple[str, int]] = []
        # Label to the line that first cites it, in the order first cited.
        self.cited: dict[str, int] = {}

        for number, line in enumerate(self.lines, start=1):
            match = DEFINITION.match(line)
            if match is None:
                continue
            label = match.group(1)
            if label in self.defined:
                self.repeats.append((label, number))
                continue
            self.defined[label] = number
            self.source[label] = self.raw[number - 1].split(":", 1)[1].strip()

        for number, line in enumerate(self.lines, start=1):
            if DEFINITION.match(line):
                continue
            for match in MARKER.finditer(line):
                self.cited.setdefault(match.group(1), number)

    def empty(self) -> bool:
        return not self.defined and not self.cited

    def undefined(self) -> list[tuple[str, str, str]]:
        return [
            (relative(self.path), label, f"line {line} cites it")
            for label, line in self.cited.items()
            if label not in self.defined
        ]

    def duplicate(self) -> list[tuple[str, str, str]]:
        return [
            (relative(self.path), label, f"line {line} defines it again")
            for label, line in self.repeats
        ]

    def repeated(self) -> list[tuple[str, str, str]]:
        """Two labels that hold the same definition text.

        The first label to appear owns the source. Every later label that
        names the same source repeats it, and the rule asks the body to reuse
        the first marker instead.
        """
        owner: dict[str, str] = {}
        out: list[tuple[str, str, str]] = []
        for label, source in self.source.items():
            first = owner.setdefault(source, label)
            if first != label:
                out.append(
                    (relative(self.path), label, f"repeats the source of [^{first}]")
                )
        return out

    def uncited(self) -> list[tuple[str, str, str]]:
        return [
            (relative(self.path), label, f"line {line} defines it")
            for label, line in self.defined.items()
            if label not in self.cited
        ]

    def out_of_order(self) -> str | None:
        order = list(self.cited)
        if not order or not all(label.isdigit() for label in order):
            return None
        numbers = [int(label) for label in order]
        if numbers == sorted(numbers):
            return None
        return ", ".join(str(n) for n in numbers)


TESTS = {
    "undefined": "cites a marker the document does not define",
    "duplicate": "defines one label twice",
    "repeated": "gives one source two labels",
    "uncited": "defines a label that nothing cites",
}


def load_baseline() -> tuple[set[tuple[str, str, str]], list[str]]:
    """Read the baseline. Return its keys and the lines that are malformed."""
    keys: set[tuple[str, str, str]] = set()
    bad: list[str] = []
    if not BASELINE.is_file():
        return keys, bad
    for number, line in enumerate(BASELINE.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 3 or parts[1] not in TESTS:
            bad.append(f"{BASELINE.name}: line {number} is not a path, a test and a label")
            continue
        keys.add((parts[0], parts[1], parts[2]))
    return keys, bad


def main() -> int:
    argv = [a for a in sys.argv[1:] if a != "--notes"]
    notes_wanted = "--notes" in sys.argv[1:]
    scan = Path(argv[0]).resolve() if argv else ROOT
    if not scan.is_dir():
        print(f"no such directory: {scan}", file=sys.stderr)
        return 2

    if scan == ROOT or OVERRIDE:
        baseline, failures = load_baseline()
    else:
        baseline, failures = set(), []
    used: set[tuple[str, str, str]] = set()
    notes: list[str] = []
    read = 0

    for path in sorted(scan.rglob("*.md")):
        if not path.is_file() or is_skipped(path, scan):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        document = Document(path, text)
        if document.empty():
            continue
        read += 1

        for test, description in TESTS.items():
            for name, label, where in getattr(document, test)():
                key = (name, test, label)
                if key in baseline:
                    used.add(key)
                    continue
                failures.append(f"{name}: [^{label}] {description} ({where})")

        disorder = document.out_of_order()
        if disorder is not None:
            notes.append(
                f"{relative(path)} numbers its footnotes out of body order: {disorder}"
            )

    for key in sorted(baseline - used):
        failures.append(
            f"{key[0]}: the baseline names [^{key[2]}] under {key[1]}, "
            f"which the check no longer finds. Take the line out of the baseline"
        )

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    if notes_wanted:
        for note in notes:
            print(f"note: {note}")

    tail = "" if notes_wanted else " (run with --notes to list them)"
    print(
        f"\nchecked the footnotes of {read} documents: {len(failures)} failures, "
        f"{len(used)} baselined, {len(notes)} out of body order{tail}"
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
