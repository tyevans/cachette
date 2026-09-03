"""Checks that each priority index lists every open thing exactly once.

A priority index states an order. It must not go stale by omission, because a
row that is missing is work nobody sees. This check derives the open set from
the tree and from the registry, and compares.

It does not check the order. The order is a judgement.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ROW = re.compile(r"^\| (\d{4}) \|", re.M)


def listed(path):
    return ROW.findall(path.read_text(encoding="utf-8"))


def repeated(listed_ids):
    """Return every number the index lists more than once, in order.

    A merge that keeps both sides of a conflicted index produces exactly this,
    and the merge check calls this function rather than restating the rule.[^1]

    # References

    [^1]: The merge defect check. `scripts/check_merge_defects.py`
    """
    seen = set()
    out = []
    for number in listed_ids:
        if number in seen:
            out.append(number)
        seen.add(number)
    return out


def report(name, listed_ids, open_ids, failures, known=None):
    """Compares a listed set against the open set.

    ``known`` is the set a row may name. It is wider than ``open_ids`` for the
    records index, because that index also names a row the project has chosen
    to write next. Completeness is required of ``open_ids`` only.
    """
    for number in repeated(listed_ids):
        failures.append(f"{name}: {number} is listed more than once")
    seen = set(listed_ids)
    for number in sorted(seen - (known if known is not None else open_ids)):
        failures.append(f"{name}: {number} is listed but does not exist")
    for number in sorted(open_ids - seen):
        failures.append(f"{name}: {number} is open but is not listed")
    return len(open_ids)


def main():
    failures = []
    total = 0

    backlog = ROOT / "docs/backlog"
    open_items = {
        path.name[:4]
        for directory in ("proposed", "refined")
        for path in (backlog / directory).glob("[0-9][0-9][0-9][0-9]-*.md")
    }
    total += report("backlog", listed(backlog / "PRIORITY.md"), open_items, failures)

    product = ROOT / "docs/product"
    open_prds = {
        path.name[4:8]
        for directory in ("idea", "shaped", "accepted")
        if (product / directory).is_dir()
        for path in (product / directory).glob("prd-*.md")
    }
    total += report("product", listed(product / "PRIORITY.md"), open_prds, failures)

    registry = (ROOT / "docs/adrs/REGISTRY.md").read_text(encoding="utf-8")
    open_adrs = set()
    every_adr = set()
    for line in registry.splitlines():
        cells = [cell.strip() for cell in line.split("|")]
        if len(cells) > 4 and re.fullmatch(r"\d{4}", cells[1]):
            every_adr.add(cells[1])
            # A draft is written and binds nothing until a reviewer accepts it,
            # so the index must account for every one. A proposed row is a
            # number the project may never spend, and the scope rule says most
            # should stay unwritten, so the index names only the ones it means
            # to write next.
            if cells[3] in ("Draft", "Reserved"):
                open_adrs.add(cells[1])
    total += report(
        "records",
        listed(ROOT / "docs/adrs/PRIORITY.md"),
        open_adrs,
        failures,
        known=every_adr,
    )

    for failure in failures:
        print(f"FAIL: {failure}")
    print(f"\nchecked {total} priority rows: {len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
