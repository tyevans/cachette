#!/usr/bin/env python3
"""Write the event column classes of the type stub from the compiled engine.

The engine declares the fields of an event in one place. The compiled module
reports that declaration through `cachette._core.event_schema`. This script
turns the report into the annotations of the matching class in the type stub,
so the stub holds no second copy of the field list.

The script rewrites one block inside each class. A marker comment opens the
block and another closes it. Everything else in the stub, including the
docstring of the class, stays as it is. A class that has no block yet gets one
straight after its docstring.

Run it with no argument to write the stub. Run it with `--check` to compare
without writing; it exits 1 and prints the differences when the stub and the
engine disagree. The test suite runs the check, so a field added to an event
fails a test until the stub follows.

The script needs the compiled module. Build it with `uv run maturin develop`
first.
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STUB = ROOT / "python" / "cachette" / "_core.pyi"

OPEN_MARKER = "    # Generated from the engine by scripts/generate_event_stubs.py."
CLOSE_MARKER = "    # End of the generated block."


def class_name(event: str) -> str:
    """Return the stub class that holds the columns of one event."""
    return "".join(part.capitalize() for part in event.split("_")) + "Columns"


def annotations(fields: list[tuple[str, str]]) -> list[str]:
    """Return one annotation line for each column of one event."""
    return [f"    {column}: npt.NDArray[np.{dtype}]" for column, dtype in fields]


def block(fields: list[tuple[str, str]]) -> list[str]:
    """Return the whole generated block of one class."""
    return [OPEN_MARKER, *annotations(fields), CLOSE_MARKER]


def docstring_end(lines: list[str], start: int) -> int:
    """Return the line after the docstring of the class that starts at `start`.

    The class body always opens with a docstring in this stub.
    """
    opened = False
    for index in range(start + 1, len(lines)):
        text = lines[index].strip()
        if not opened:
            if text.startswith('"""'):
                opened = True
                if text.endswith('"""') and len(text) > 3:
                    return index + 1
            continue
        if text.endswith('"""'):
            return index + 1
    raise SystemExit(f"the class at line {start + 1} has no docstring")


def rewrite(lines: list[str], event: str, fields: list[tuple[str, str]]) -> list[str]:
    """Return the stub lines with the block of one event written."""
    header = f"class {class_name(event)}(TypedDict):"
    start = next(
        (index for index, line in enumerate(lines) if line.startswith(header)), None
    )
    if start is None:
        raise SystemExit(f"the stub holds no class {class_name(event)}")
    stop = next(
        (
            index
            for index in range(start + 1, len(lines))
            if lines[index] and not lines[index].startswith((" ", ")", "]"))
        ),
        len(lines),
    )
    opened = next(
        (index for index in range(start, stop) if lines[index] == OPEN_MARKER),
        None,
    )
    if opened is None:
        at = docstring_end(lines, start)
        while at < len(lines) and not lines[at].strip():
            at += 1
        end = at
        while end < len(lines) and lines[end].startswith("    ") and ":" in lines[end]:
            end += 1
        return lines[:at] + block(fields) + lines[end:]
    closed = next(
        index for index in range(opened, len(lines)) if lines[index] == CLOSE_MARKER
    )
    return lines[:opened] + block(fields) + lines[closed + 1 :]


def main(argv: list[str]) -> int:
    """Write the stub, or check it, and return the exit status."""
    check = "--check" in argv[1:]
    from cachette import _core

    schema: dict[str, list[tuple[str, str]]] = _core.event_schema()
    before = STUB.read_text(encoding="utf-8")
    lines = before.splitlines()
    for event, fields in schema.items():
        lines = rewrite(lines, event, fields)
    after = "\n".join(lines) + "\n"
    if after == before:
        return 0
    if check:
        import difflib

        diff = difflib.unified_diff(
            before.splitlines(),
            after.splitlines(),
            fromfile="the stub",
            tofile="the engine",
            lineterm="",
        )
        print("\n".join(diff))
        print("\nRun scripts/generate_event_stubs.py to write the stub.")
        return 1
    STUB.write_text(after, encoding="utf-8")
    print(f"wrote {STUB.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
