#!/usr/bin/env python3
"""Fail when a document states in prose what a register holds.

A register is the current statement of a fact. The blockers register holds
what is unanswered. The findings register holds what the project corrected.
The measurement registers hold what a run measured. A document that repeats
one of those in its own words becomes false the moment the register moves,
and nothing fails, because a document is prose.

The project has met this shape three times. The third time one sentence about
the state of measurement had reached about ninety documents.[^1] The defence
is already written down: a document names a register by citation and never
states its content in its own words.[^2] Nothing enforced it. That absence was
tested rather than assumed: after the sweep, one repaired sentence went back
into a record in its stale form, and all eight document checks passed.[^3]

## The rule this check applies

**A paragraph that states a register's content must carry a route back to the
register. A paragraph that states it and cites nothing is a restatement.**

The check reads every Markdown document in the tree. It joins each paragraph
into one line before it matches, because prose here wraps at about 78 columns
and a phrase of five words is usually split across the break. A line-based
sweep already reported clean against a tree that still held the sites.[^4]

It then matches each paragraph against a named family of phrases. A match is
a failure unless that paragraph holds a footnote marker, or names a register
row: a blocker, a finding, a decision or a decision record. Either one gives
the reader one hop to the register, so a reader who suspects the sentence can
reach the thing that decides it. A bare paragraph gives the reader nothing,
and the search that starts from a register number can never find it.[^4]

The paragraph is the unit, and a row of a table is a paragraph of its own. A
sentence is too small, because a writer states a claim in one sentence and
cites it in the next. A whole table is too large, because one cited row would
excuse every other row.

## What this check cannot catch

State this plainly rather than trust the check further than it reaches.

- **A restatement in a paragraph that cites the register.** The check passes
  it. The sentence
  is still a second declaration site, and it still goes false when the register
  moves. The citation only makes the staleness recoverable by a reader who
  doubts it. Failing every cited site would fail on about forty records that
  nobody may edit, and a gate nobody can turn green is a gate everybody skips.
- **A family nobody has written down.** One family is implemented: the state
  of measurement, which is the family with three recorded instances. A register
  that has never gone stale gets no family, because a wide family produces
  noise and noise is how a check dies.
- **A paraphrase outside the family.** The patterns are phrases, not meaning.
  A writer who states the same fact in new words passes.
- **Prose outside Markdown.** A Rust doc comment and a Python docstring hold
  the same family and this check does not read them. The citation check reads
  those files for a different rule, and extending this one to them is separate
  work.

## The exemptions and the baseline

A register owns its own statement, so a match inside one is the current
statement and not a copy of it. The check exempts the blockers register, the
findings register, the decisions register and the two measurement registers,
by path. The decisions register is there because an open row states what
nobody has measured about its own options, which is the choice itself.

Every other site is either repaired or listed in the baseline. The baseline
carries the sites a sweep may not repair: an accepted decision record is
frozen, and a review and a completed backlog item are each a record of a
moment. Each line holds the path, the family and the whole normalised
sentence. The check fails when a baseline line matches nothing, so the list
cannot go stale and can only shrink. Do not add to it. Repair the document.

Give a directory as the first argument to scan that tree instead of the
repository. The default baseline is not applied to an explicit scan, which is
how the probe recipe proves the check can fail. Set
`CACHETTE_REGISTER_PROSE_BASELINE` to read another baseline, which is applied
to any scan.

Run with `--notes` to list the cited sites, which the check permits and which
a reader may still want to see.

Exit 0 when every test passes, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-register-prose.sh`.

# References

[^1]: Findings register, FND-223. `docs/FINDINGS.md`
[^2]: Documentation Rules, section 3. `.agents/rules/documentation.md`
[^3]: Findings register, FND-258. `docs/FINDINGS.md`
[^4]: Findings register, FND-260. `docs/FINDINGS.md`
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BASELINE = Path(__file__).resolve().parent / "register-prose-baseline.txt"
OVERRIDE = os.environ.get("CACHETTE_REGISTER_PROSE_BASELINE")
BASELINE = Path(OVERRIDE).resolve() if OVERRIDE else DEFAULT_BASELINE

SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".ruff_cache"}
SKIP_PATHS = {
    # A worktree holds another checkout of this repository.
    ROOT / ".claude" / "worktrees",
    # Deliberately broken. The probe recipe scans these on purpose.
    ROOT / "tests" / "fixtures",
    # A tool's session scratch, not project prose.
    ROOT / "docs" / "superpowers",
    ROOT / ".claude" / "tackline",
}

# The registers that own the statement. A sentence here is the current
# statement of the fact, not a copy of it.
OWNERS = {
    ROOT / "docs" / "BLOCKERS.md",
    ROOT / "docs" / "FINDINGS.md",
    ROOT / "docs" / "reference" / "budgets.md",
    ROOT / "docs" / "reference" / "graviton-costs.md",
    ROOT / "docs" / "DECISIONS.md",
}

# The families. One name, one description, one pattern.
#
# Add a family only for a register that has gone stale in this tree. The item
# that added this check says so, and a wide family produces noise.
FAMILIES = {
    "measurement": (
        "states the state of measurement, which the blockers register owns",
        re.compile(
            r"no (?:measurement|benchmark) (?:of [^.]{0,80} )?(?:exists|has run|ran"
            r"|has been (?:made|taken|run))"
            r"|nobody has measured"
            r"|(?:has|have) not been measured"
            r"|never been measured"
            r"|derived (?:rather than|and not) measured"
            r"|derived, not measured",
            re.IGNORECASE,
        ),
    ),
}

# A route back to the register: a footnote marker, or a register row by number.
CITATION = re.compile(r"\[\^[^\]\s]+\]|\b(?:BLK|FND|DEC|PRD)-\d+|\bADR-\d{4}")

FENCE = re.compile(r"^\s*(?:```|~~~)")
SENTENCE = re.compile(r"(?<=[.!?])\s+")


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


def paragraphs(text: str) -> list[tuple[int, str]]:
    """Return each paragraph joined into one line, with its first line number.

    A fenced block is dropped. The documentation rule exempts a code block, and
    a script that quotes its own search pattern must not fail its own check.
    """
    out: list[tuple[int, str]] = []
    start = 0
    buffer: list[str] = []
    fenced = False

    def flush() -> None:
        if buffer:
            joined = re.sub(r"\s+", " ", " ".join(buffer)).strip()
            if joined:
                out.append((start, joined))
        buffer.clear()

    for number, line in enumerate(text.splitlines(), start=1):
        if FENCE.match(line):
            flush()
            fenced = not fenced
            continue
        if fenced:
            continue
        if not line.strip():
            flush()
            continue
        if line.lstrip().startswith("|"):
            # A row of a table is a unit of its own. A whole table read as one
            # paragraph would let one cited row excuse every other row.
            flush()
            out.append((number, re.sub(r"\s+", " ", line.strip())))
            continue
        if not buffer:
            start = number
        buffer.append(line.strip())
    flush()
    return out


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
        if len(parts) != 3 or parts[1] not in FAMILIES:
            bad.append(
                f"{BASELINE.name}: line {number} is not a path, a family and a sentence"
            )
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
    cited = 0

    for path in sorted(scan.rglob("*.md")):
        if not path.is_file() or is_skipped(path, scan):
            continue
        if scan == ROOT and path in OWNERS:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        read += 1
        name = relative(path)

        for line_number, paragraph in paragraphs(text):
            # The route back to the register may sit in a neighbouring
            # sentence of the same paragraph, so the citation is read over the
            # paragraph and the failure is reported against the sentence.
            routed = CITATION.search(paragraph) is not None
            for sentence in SENTENCE.split(paragraph):
                sentence = sentence.strip()
                for family, (description, pattern) in FAMILIES.items():
                    if not pattern.search(sentence):
                        continue
                    if routed:
                        cited += 1
                        notes.append(f"{name}: near line {line_number}: {sentence}")
                        continue
                    key = (name, family, sentence)
                    if key in baseline:
                        used.add(key)
                        continue
                    failures.append(
                        f"{name}: near line {line_number}, a sentence {description} "
                        f"and cites nothing: {sentence}"
                    )

    for key in sorted(baseline - used):
        failures.append(
            f"{key[0]}: the baseline names a {key[1]} sentence that the check no "
            f"longer finds. Take the line out of the baseline: {key[2]}"
        )

    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)

    if notes_wanted:
        for note in notes:
            print(f"note: {note}")

    tail = "" if notes_wanted else " (run with --notes to list them)"
    print(
        f"\nread {read} documents for register prose: {len(failures)} failures, "
        f"{len(used)} baselined, {cited} cited{tail}"
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
