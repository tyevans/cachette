#!/usr/bin/env python3
"""Check the product requirement records against the rules in docs/product/README.md.

Standard library only. Exits non-zero on any failure.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

DIR_STATUS = {
    "idea": {"Idea"},
    "shaped": {"Shaped"},
    "accepted": {"Accepted"},
    "shipped": {"Shipped", "Dropped"},
}

GATE_SECTIONS = [
    "Who this is for",
    "What the person cannot do today",
    "What good looks like",
    "What this does not do",
    "What it costs at the target scale",
    "Which blockers govern this",
]

NAME_RE = re.compile(r"^prd-(\d{4})-[a-z0-9-]+\.md$")
ROW_RE = re.compile(r"^\|\s*(\d{4})\s*\|([^|]*)\|\s*(\w+)\s*\|")


def read_registry(root: Path) -> tuple[dict[int, str], list[str]]:
    errors: list[str] = []
    rows: dict[int, str] = {}
    text = (root / "REGISTRY.md").read_text(encoding="utf-8")
    for line in text.splitlines():
        m = ROW_RE.match(line.strip())
        if not m:
            continue
        number = int(m.group(1))
        if number in rows:
            errors.append(f"registry: number {number:04d} occurs twice")
        rows[number] = m.group(3)
    return rows, errors


def check(root: Path) -> list[str]:
    errors: list[str] = []
    rows, errors_registry = read_registry(root)
    errors += errors_registry

    seen: dict[int, Path] = {}
    for directory, allowed in DIR_STATUS.items():
        d = root / directory
        if not d.is_dir():
            continue
        for path in sorted(d.glob("*.md")):
            m = NAME_RE.match(path.name)
            if not m:
                errors.append(f"{path}: name must match prd-NNNN-slug.md")
                continue
            number = int(m.group(1))
            if number in seen:
                errors.append(f"{path}: number {number:04d} also used by {seen[number]}")
            seen[number] = path

            status = rows.get(number)
            if status is None:
                errors.append(f"{path}: no row in the registry; allocate the number first")
            elif status not in allowed:
                errors.append(
                    f"{path}: registry says {status}, but the file sits in {directory}/"
                )
            errors += check_body(path, directory)

    for number, status in sorted(rows.items()):
        if status != "Idea" and number not in seen:
            errors.append(f"registry: {number:04d} has status {status} but no file exists")
    return errors


def check_body(path: Path, directory: str) -> list[str]:
    errors: list[str] = []
    text = path.read_text(encoding="utf-8")
    body, _, references = text.partition("\n## References")

    if directory != "idea":
        for heading in GATE_SECTIONS:
            pattern = re.compile(rf"^##\s+{re.escape(heading)}\s*$", re.MULTILINE)
            m = pattern.search(body)
            if not m:
                errors.append(f"{path}: missing the gate section '{heading}'")
                continue
            rest = body[m.end():]
            nxt = re.search(r"^##\s+", rest, re.MULTILINE)
            content = rest[: nxt.start()] if nxt else rest
            if not content.strip():
                errors.append(f"{path}: the gate section '{heading}' is empty")

    for n, line in enumerate(body.splitlines(), start=1):
        if line.lstrip().startswith("[^"):
            continue
        if re.search(r"\bADR-\d{4}\b", line) or "docs/adrs/" in line:
            errors.append(
                f"{path}:{n}: cites a decision record; a product record states a need, "
                "not a structure"
            )
    return errors


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "docs/product")
    if not root.is_dir():
        print(f"no such directory: {root}", file=sys.stderr)
        return 2
    errors = check(root)
    for e in errors:
        print(f"FAIL {e}")
    count = sum(1 for _ in root.glob("*/prd-*.md"))
    print(f"\nchecked {count} product records: {len(errors)} failures")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
