"""Stand in for the generation tool, so a test can drive the whole front end.

The real tool asks a vision model to draw and to critique. One round takes
about seventy seconds and costs model time. A test of the interface must not
pay that, and it must not depend on an endpoint.

This script takes the same command line as the real tool, and writes the same
files in the same order: the SVG first, then the two renders, then the
critique, then the round metadata. It writes one file at a time, so a page
that reads the directory meets every part-way state.

The run manager calls this script when a test hands it this command. Nothing
else calls it, and the server never calls it.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import argparse
import os
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from make_fixtures import (  # noqa: E402
    LETTERS,
    critique,
    render,
    variant_svg,
    write_json,
)

# The environment variable that names the sessions root. The real tool takes
# the root from the directory beside its package. A test points this script
# at a scratch directory the same way it points the server.
ROOT_VARIABLE = "DIRECT_DIE_SESSIONS"

# The two render sizes of the contract.
DISPLAY_SIZE = 96
LARGE_SIZE = 512

# The wait between two files. It is short, and it exists so that a reader
# meets a directory that holds part of a round.
STEP_SECONDS = 0.02


def main(argv: list[str] | None = None) -> int:
    """Write the rounds that one run would write, with no model."""
    parser = argparse.ArgumentParser(prog="fake-direct-die")
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run")
    run.add_argument("--asset", required=True)
    run.add_argument("--subject", required=True)
    run.add_argument("--session", required=True)
    run.add_argument("--rounds", type=int, default=1)
    run.add_argument("--variants", type=int, default=4)
    run.add_argument("--exemplars", type=int, default=2)
    arguments = parser.parse_args(argv)

    root = Path(os.environ.get(ROOT_VARIABLE, "sessions"))
    directory = root / arguments.asset / arguments.session
    directory.mkdir(parents=True, exist_ok=True)
    write_json(
        directory / "session.json",
        {
            "asset": arguments.asset,
            "created": "2026-09-06T00:00:00Z",
            "model": "fake-model",
            "guide_version": "fake-guide",
        },
    )
    existing = [
        int(entry.name[len("round-") :])
        for entry in directory.iterdir()
        if entry.is_dir() and entry.name.startswith("round-")
    ]
    first = max(existing) + 1 if existing else 0

    for offset in range(arguments.rounds):
        number = first + offset
        round_directory = directory / f"round-{number:02d}"
        round_directory.mkdir(parents=True, exist_ok=True)
        print(f"round-{number:02d}", flush=True)
        for letter in LETTERS[: arguments.variants]:
            svg = variant_svg(letter, number)
            (round_directory / f"variant-{letter}.svg").write_text(
                svg, encoding="utf-8"
            )
            time.sleep(STEP_SECONDS)
            render(svg, round_directory / f"variant-{letter}.png", DISPLAY_SIZE)
            render(svg, round_directory / f"variant-{letter}.large.png", LARGE_SIZE)
            time.sleep(STEP_SECONDS)
            write_json(
                round_directory / f"variant-{letter}.critique.json",
                critique(letter, number),
            )
            print(f"  variant {letter}", flush=True)
        write_json(
            round_directory / "meta.json",
            {
                "round": number,
                "prompt_summary": f"{arguments.subject}; a fake round",
                "parent": None,
            },
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
