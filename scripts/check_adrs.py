#!/usr/bin/env python3
"""Check the architecture decision records against the record scope rule.

Enforces the mechanical part of `.claude/rules/adr-scope.md`:

  structure  every record has the required sections, in order, with the
             `## References` section last, and a title that matches its number
  volatile   no version pin, latency figure, throughput figure or percentage in
             the body of a record (footnotes, code blocks and tables exempt)
  refs       every `ADR-NNNN` cited exists; every `ADR-NNNN Dn` names a decision
             the target record actually has; every record path in a footnote
             resolves on disk
  registry   the registry and the record files agree on number and status

Reports, without failing:

  uncited    a record that no other record and no source file names

The volatile check carries a baseline of figures that already exist in the
draft records, in `scripts/adr-volatile-baseline.txt`. A baselined figure does
not fail the check. The baseline is falsifiable: an entry that matches nothing
fails, so the list shrinks as the drafts are corrected and can never grow
stale. Do not add to it. Move the figure to the reference tables instead.

Exit 0 when every check passes, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-adrs.sh`.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR_DIRS = [ROOT / "docs" / "adrs" / "draft", ROOT / "docs" / "adrs" / "accepted"]
REGISTRY = ROOT / "docs" / "adrs" / "REGISTRY.md"
BASELINE = Path(__file__).resolve().parent / "adr-volatile-baseline.txt"

REQUIRED = ["## Context", "## Decision", "## Consequences", "## References"]

FILENAME = re.compile(r"^adr-(\d{4})-[a-z0-9-]+\.md$")
TITLE = re.compile(r"^#\s+ADR-(\d{4})\s*[:.]")
REGISTRY_ROW = re.compile(r"^\|\s*(\d{4})\s*\|\s*([^|]+?)\s*\|\s*(\w+)\s*\|", re.M)
DECISION = re.compile(r"^#{2,4}\s*(?:ADR-\d{4}\s+)?D(\d+)\b", re.M)
CITE = re.compile(r"\bADR-(\d{4})(?:\s+D(\d+))?")
FOOTNOTE_PATH = re.compile(r"`(docs/[^`]+\.md)`")

# Material a measurement or a release can change. Section 4.1 and 4.2 of the
# rule. Deliberately narrow: a platform constant such as a cache line size is
# not a budget, and flagging it would train everyone to ignore this check.
VOLATILE = [
    (re.compile(r"(?<![\w.])\d+\.\d+\.\d+(?![\w.])"), "a pinned release version"),
    (re.compile(r"\b(?:version|release)\s+\d+\.\d+"), "a pinned version"),
    (re.compile(r"(?:>=|==|~=|\^)\s*\d+\.\d+"), "a version constraint"),
    (re.compile(r"\b\d[\d_,.]*\s*(?:ns|µs|us|ms|milliseconds|microseconds)\b"), "a latency figure"),
    (re.compile(r"\b\d[\d_,.]*\s*(?:%|per cent|percent)"), "a percentage"),
    (re.compile(r"\b\d[\d_,.]*\s*(?:MB|GB|GiB|MiB|KB)\s+(?:budget|total|per\s+\w+)"), "a memory budget"),
    (re.compile(r"\b\d[\d_,.]*\s*(?:ops|events|ticks|frames)\s*/\s*s\b"), "a throughput figure"),
    (re.compile(r"\bbudget\s+(?:is|of)\s+\d"), "a budget value"),
]


def strip_exempt(text: str) -> str:
    """Remove the parts the documentation rule exempts: code blocks, tables,
    inline code, and the `## References` section."""
    text = text.split("\n## References", 1)[0]
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    text = re.sub(r"`[^`\n]*`", "", text)
    text = "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("|"))
    return text


def line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def main() -> int:
    failures: list[str] = []
    notes: list[str] = []

    baseline: set[str] = set()
    if BASELINE.exists():
        for line in BASELINE.read_text(encoding="utf-8").splitlines():
            line = line.split("#", 1)[0].rstrip()
            if line:
                baseline.add(line)
    matched_baseline: set[str] = set()

    files: dict[str, Path] = {}
    for d in ADR_DIRS:
        if not d.is_dir():
            continue
        for p in sorted(d.glob("*.md")):
            m = FILENAME.match(p.name)
            if not m:
                failures.append(f"{p.name}: file name must be adr-NNNN-slug.md")
                continue
            if m.group(1) in files:
                failures.append(f"{p.name}: number {m.group(1)} is used twice")
            files[m.group(1)] = p

    if not files:
        print("check_adrs: no records found", file=sys.stderr)
        return 1

    bodies = {n: p.read_text(encoding="utf-8") for n, p in files.items()}
    decisions = {n: {d for d in DECISION.findall(t)} for n, t in bodies.items()}

    registry = REGISTRY.read_text(encoding="utf-8") if REGISTRY.exists() else ""
    rows = {m.group(1): (m.group(2), m.group(3)) for m in REGISTRY_ROW.finditer(registry)}

    for num, path in sorted(files.items()):
        text = bodies[num]
        name = path.name

        # structure
        t = TITLE.match(text.lstrip().splitlines()[0] if text.strip() else "")
        if not t:
            failures.append(f"{name}: first line must be '# ADR-{num}: <claim>'")
        elif t.group(1) != num:
            failures.append(f"{name}: title says ADR-{t.group(1)}, file says {num}")

        seen = [s for s in REQUIRED if re.search(rf"^{re.escape(s)}\s*$", text, re.M)]
        for s in REQUIRED:
            if s not in seen:
                failures.append(f"{name}: missing required section '{s}'")
        if seen != [s for s in REQUIRED if s in seen]:
            failures.append(f"{name}: required sections are out of order")
        if seen and not text.rstrip().split("## References", 1)[-1:]:
            pass
        if "## References" in text:
            tail = text.split("## References", 1)[1]
            if re.search(r"^##\s+(?!References)", tail, re.M):
                failures.append(f"{name}: '## References' must be the last section")

        # volatile material
        body = strip_exempt(text)
        for pattern, what in VOLATILE:
            for m in pattern.finditer(body):
                key = f"{name}\t{' '.join(m.group(0).split())}"
                if key in baseline:
                    matched_baseline.add(key)
                    continue
                failures.append(
                    f"{name}: line {line_of(body, m.start())} holds {what} "
                    f"({' '.join(m.group(0).split())!r}). A record must not hold material that changes."
                )

        # references
        for m in CITE.finditer(text):
            target, dec = m.group(1), m.group(2)
            if target == num and dec is None:
                continue
            if target not in files and target not in rows:
                failures.append(f"{name}: cites ADR-{target}, which no record and no registry row has")
            elif dec is not None and target in files and dec not in decisions[target]:
                failures.append(f"{name}: cites ADR-{target} D{dec}, which ADR-{target} does not define")
        for m in FOOTNOTE_PATH.finditer(text):
            if not (ROOT / m.group(1)).exists():
                failures.append(f"{name}: footnote path does not exist: {m.group(1)}")

        # registry agreement
        if num not in rows:
            failures.append(f"{name}: no registry row for {num}. Allocate the number in the registry first.")
        elif rows[num][1] == "Proposed":
            failures.append(f"{name}: registry says Proposed, but the file exists. Set it to Draft.")

    for num, (title, status) in sorted(rows.items()):
        if status in {"Draft", "Accepted", "Superseded", "Rejected"} and num not in files:
            failures.append(f"registry: {num} '{title}' has status {status} but no file exists")

    for stale in sorted(baseline - matched_baseline):
        failures.append(
            f"baseline entry matches nothing and must be deleted: {stale!r}"
        )

    # uncited: a note, never a failure (rule section 6)
    code = [p for ext in ("*.rs", "*.py") for p in ROOT.rglob(ext) if ".git" not in p.parts]
    corpus = "\n".join(bodies[n] for n in files) + registry + "\n".join(
        p.read_text(encoding="utf-8", errors="ignore") for p in code
    )
    for num in sorted(files):
        others = "\n".join(bodies[n] for n in files if n != num)
        cited = re.search(rf"ADR-{num}\b", others) or re.search(
            rf"\|\s*\d{{4}}\s*\|[^|]*\|[^|]*\|[^|]*\b{num}\b", registry
        )
        if not cited and not any(re.search(rf"ADR-{num}\b", p.read_text(encoding="utf-8", errors="ignore")) for p in code):
            notes.append(f"ADR-{num} is cited by no other record and no source file. Is it a constraint, or a description?")

    for n in notes:
        print(f"note: {n}")
    for f in failures:
        print(f"FAIL: {f}", file=sys.stderr)

    print(f"\nchecked {len(files)} records: {len(failures)} failures, {len(notes)} notes")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
