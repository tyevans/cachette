#!/usr/bin/env python3
"""Check every citation of a decision record, everywhere except the records.

The record check reads the records. This check reads everything else: source
files, tests, scripts, workflows, build manifests, rules and documents. A
citation in a comment is not compiled, so nothing else fails when it decays.

A record split renumbered the decisions and left 81 dangling citations in the
tree. Nothing caught it. The numbers had been reused, so a reader who followed
one reached a real record that stated a different claim. That is the failure
this check exists to prevent.[^1]

It fails when:

  record    a citation names an ADR-NNNN that no record file and no registry
            row has
  decision  a citation names ADR-NNNN Dn, the record for NNNN exists, and that
            record defines no decision Dn
  path      a footnote names a `docs/...` path that does not resolve on disk

A citation of a number whose registry row has no file passes. That is the
documented way to cite a reserved number, and the registry is where its status
lives.

A citation inside a code span is a mention, not a reference, and the check
ignores it. The documentation rule already exempts an identifier in code, and
a document that discusses a citation must be able to quote one. Write
`` `ADR-0002 D9` `` to name the token rather than follow it. A footnote path
is still read from inside its code span, because that is where the rule puts
it.

Give a directory as the first argument to scan that tree instead of the
repository. The records and the registry still come from the repository, so a
fixture tree proves the check can fail without a second copy of the records.

Exit 0 when every citation resolves, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-citations.sh`.

References

[^1]: Findings register, FND-040. `docs/FINDINGS.md`
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR_DIRS = [ROOT / "docs" / "adrs" / "draft", ROOT / "docs" / "adrs" / "accepted"]
REGISTRY = ROOT / "docs" / "adrs" / "REGISTRY.md"

# The records themselves. check_adrs.py already reads these, and reading them
# here would report every failure twice.
SKIP_DIRS = {".git", "target", ".venv", "node_modules", "__pycache__", ".ruff_cache"}
SKIP_PATHS = {
    ROOT / "docs" / "adrs",
    # The reports were written before the registry was derived from claims.
    # They cite the drafts that were deleted. Converting them is backlog item
    # 0004, and failing on them now would only train everyone to ignore this
    # check.
    ROOT / "docs" / "research" / "reports",
    ROOT / "docs" / "superpowers",
    # A tool's session scratch, not project prose. It quotes whatever the
    # session held at the moment it was written, including a retired number,
    # and nobody maintains it.
    ROOT / ".claude" / "tackline",
    # A worktree holds another checkout of this repository. Its files are
    # checked by the run that owns them, and reading them here reports one
    # failure against two paths and blames the wrong tree.
    ROOT / ".claude" / "worktrees",
    ROOT / "scripts" / "check_citations.py",
    # Deliberately broken. Continuous integration runs the check against this
    # directory on purpose and fails when the check passes.
    ROOT / "tests" / "fixtures",
}
SUFFIXES = {".md", ".rs", ".py", ".pyi", ".toml", ".yml", ".yaml", ".sh", ".txt"}
EXTRA_FILES = {ROOT / "justfile"}

CITE = re.compile(r"\bADR-(\d{4})(?:\s+D(\d+))?")
CODE_SPAN = re.compile(r"`+[^`\n]*`+")
FOOTNOTE_PATH = re.compile(r"`(docs/[^`]+\.md)`")
FILENAME = re.compile(r"^adr-(\d{4})-[a-z0-9-]+\.md$")
REGISTRY_ROW = re.compile(r"^\|\s*(\d{4})\s*\|\s*([^|]+?)\s*\|\s*(\w+)\s*\|", re.M)
DECISION = re.compile(r"^#{2,4}\s*(?:ADR-\d{4}\s+)?D(\d+)\b", re.M)


def is_skipped(path: Path, scan: Path) -> bool:
    """Say whether the scan passes over this path.

    A path named in SKIP_PATHS is skipped during a scan of the repository. An
    explicit scan of one of those directories reads it, because that is the
    only way a fixture can prove the check can fail.
    """
    if any(part in SKIP_DIRS for part in path.parts):
        return True
    if scan != ROOT:
        return False
    return any(path == p or p in path.parents for p in SKIP_PATHS)


def sources(scan: Path) -> list[Path]:
    found = [
        p
        for p in scan.rglob("*")
        if p.is_file() and p.suffix in SUFFIXES and not is_skipped(p, scan)
    ]
    if scan == ROOT:
        found += [p for p in EXTRA_FILES if p.is_file()]
    return sorted(found)


def line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def blank_code_spans(text: str) -> str:
    """Replace the contents of each code span with spaces.

    The offsets stay put, so a line number taken from the result is the line
    number in the file.
    """
    return CODE_SPAN.sub(lambda m: " " * len(m.group(0)), text)


def main() -> int:
    scan = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT
    if not scan.is_dir():
        print(f"no such directory: {scan}", file=sys.stderr)
        return 2

    decisions: dict[str, set[str]] = {}
    for directory in ADR_DIRS:
        if not directory.is_dir():
            continue
        for path in sorted(directory.glob("*.md")):
            m = FILENAME.match(path.name)
            if m:
                body = path.read_text(encoding="utf-8")
                decisions[m.group(1)] = set(DECISION.findall(body))

    registry = REGISTRY.read_text(encoding="utf-8") if REGISTRY.exists() else ""
    rows = {m.group(1) for m in REGISTRY_ROW.finditer(registry)}

    if not decisions:
        print("check_citations: no records found", file=sys.stderr)
        return 1

    failures: list[str] = []
    checked = 0
    files_with_citations = 0

    for path in sources(scan):
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        name = path.relative_to(ROOT) if ROOT in path.parents else path
        seen_here = 0
        prose = blank_code_spans(text)

        for m in CITE.finditer(prose):
            number, decision = m.group(1), m.group(2)
            checked += 1
            seen_here += 1
            if number not in decisions and number not in rows:
                failures.append(
                    f"{name}:{line_of(text, m.start())}: cites ADR-{number}, "
                    f"which no record and no registry row has"
                )
            elif (
                decision is not None
                and number in decisions
                and decision not in decisions[number]
            ):
                failures.append(
                    f"{name}:{line_of(text, m.start())}: cites ADR-{number} D{decision}, "
                    f"which ADR-{number} does not define"
                )

        for m in FOOTNOTE_PATH.finditer(text):
            if not (ROOT / m.group(1)).exists():
                failures.append(
                    f"{name}:{line_of(text, m.start())}: footnote path does not "
                    f"exist: {m.group(1)}"
                )

        if seen_here:
            files_with_citations += 1

    for f in failures:
        print(f"FAIL: {f}", file=sys.stderr)

    print(
        f"\nchecked {checked} citations in {files_with_citations} files "
        f"outside the records: {len(failures)} failures"
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
