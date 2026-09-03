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
  status     no record file states a status of its own. The registry holds the
             status of a record, and it is the only document that does

Reports, without failing:

  uncited    a record that no other record and no source file names

The volatile check carries a baseline of figures that already exist in the
draft records, in `scripts/adr-volatile-baseline.txt`. A baselined figure does
not fail the check. The baseline is falsifiable: an entry that matches nothing
fails, so the list shrinks as the drafts are corrected and can never grow
stale. Do not add to it. Move the figure to the reference tables instead.

Exit 0 when every check passes, 1 otherwise. No dependencies beyond the
standard library. Run it with `scripts/check-adrs.sh`.

The script reads `docs/adrs` by default. Give it another directory to check a
broken fixture instead, which is how the probe recipe proves the checks fail.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADR_ROOT = ROOT / "docs" / "adrs"
BASELINE = Path(__file__).resolve().parent / "adr-volatile-baseline.txt"

REQUIRED = ["## Context", "## Decision", "## Consequences", "## References"]

FILENAME = re.compile(r"^adr-(\d{4})-[a-z0-9-]+\.md$")
TITLE = re.compile(r"^#\s+ADR-(\d{4})\s*[:.]")
REGISTRY_ROW = re.compile(r"^\|\s*(\d{4})\s*\|\s*([^|]+?)\s*\|\s*(\w+)\s*\|", re.M)
DECISION = re.compile(r"^#{2,4}\s*(?:ADR-\d{4}\s+)?D(\d+)\b", re.M)
CITE = re.compile(r"\bADR-(\d{4})(?:\s+D(\d+))?")
FOOTNOTE_PATH = re.compile(r"`(docs/[^`]+\.md)`")
# The status of a record lives in the registry and nowhere else. A record file
# that declares one declares a second copy, and nothing fails when the two
# disagree. This is the check that fails instead. The pattern reads the status
# vocabulary of the registry, so it catches a line copied from an old record
# and leaves ordinary prose alone.
STATUS_LINE = re.compile(
    r"^\s*(?:\*\*|__|_|\*)?\s*status\s*(?:\*\*|__|_|\*)?\s*[:=]\s*"
    r"(?:\*\*|_)?\s*(proposed|draft|accepted|superseded|rejected)\b",
    re.I,
)

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


def status_failures(name: str, text: str) -> list[str]:
    """Report every line of a record that states a status.

    A fenced code block and a table row are exempt. A record may quote a
    registry row, and the documentation rule exempts both.
    """
    out: list[str] = []
    fenced = False
    for number, line in enumerate(text.splitlines(), start=1):
        if line.lstrip().startswith("```"):
            fenced = not fenced
            continue
        if fenced or line.lstrip().startswith("|"):
            continue
        if STATUS_LINE.match(line):
            out.append(
                f"{name}: line {number} states a status ({line.strip()!r}). "
                f"The registry holds the status of a record, and it is the only "
                f"document that does."
            )
    return out


def main(argv: list[str]) -> int:
    failures: list[str] = []
    notes: list[str] = []

    adr_root = Path(argv[0]).resolve() if argv else ADR_ROOT
    adr_dirs = [adr_root / "draft", adr_root / "accepted"]
    registry_path = adr_root / "REGISTRY.md"

    baseline: set[str] = set()
    if BASELINE.exists():
        for line in BASELINE.read_text(encoding="utf-8").splitlines():
            line = line.split("#", 1)[0].rstrip()
            if line:
                baseline.add(line)
    matched_baseline: set[str] = set()

    files: dict[str, Path] = {}
    for d in adr_dirs:
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

    registry = registry_path.read_text(encoding="utf-8") if registry_path.exists() else ""
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

        # one status site
        failures += status_failures(name, text)

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
    # A worktree holds another checkout of this repository. Its files belong to
    # the run that owns them, they change under this one, and a file deleted
    # mid-scan raises rather than reporting. The citation check skips them for
    # the same reason.
    # The skip names paths and not path components. A component named
    # `worktrees` matches every file of a checkout that is itself inside
    # `.claude/worktrees`, so the component form silently scanned nothing when
    # this script ran from a worktree, and reported every record as cited by no
    # source file.[^1] The three sibling checks already name paths.
    #
    # [^1]: Findings register, FND-305. `docs/FINDINGS.md`
    skip = (ROOT / ".git", ROOT / "target", ROOT / ".claude" / "worktrees")
    code = [
        p
        for ext in ("*.rs", "*.py")
        for p in ROOT.rglob(ext)
        if not any(p.is_relative_to(d) for d in skip)
    ]
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
    raise SystemExit(main(sys.argv[1:]))
