"""Fabricate a direct-die session tree, so that the review server has data.

The generation loop writes the real tree. This script writes a fake one with
the same names and the same shapes. It costs no model time, so anyone can
test the review server with it.

The script writes the broken cases on purpose, because a half-written
directory is the normal case while the loop runs:

- A round that holds one variant, not four.
- A round whose variants have no critique.
- A round that holds an SVG and no render.
- A session that holds no round at all.
- A session that holds no session.json.
- A round whose critique file holds broken JSON.

Run it, then start the server against the directory it wrote.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
from datetime import UTC, datetime, timedelta
from pathlib import Path

import cairosvg

# The two render sizes of the contract. A hex tile lives at the display size.
DISPLAY_SIZE = 96
LARGE_SIZE = 512

# The four variant letters of the contract.
LETTERS = ("a", "b", "c", "d")

PALETTES = {
    "a": ("#3d7a4b", "#2a5a36", "#8fbf7a"),
    "b": ("#4a6fa5", "#31507d", "#93b4dd"),
    "c": ("#a5824a", "#7d6131", "#ddc193"),
    "d": ("#7a4a7a", "#5a315a", "#c193dd"),
}

VERDICTS = ("keep", "revise", "reject")

FAULTS = [
    "The outline is too heavy for the display size.",
    "The interior detail disappears below 32 pixels.",
    "The colour value sits too near the terrain behind it.",
    "The silhouette does not read as one shape.",
    "The highlight direction disagrees with the style guide.",
    "The shape leaves the hex boundary on two edges.",
]


def hexagon_points(centre: float, radius: float) -> str:
    """Give the six points of a flat-top hexagon, as an SVG point list."""
    points = []
    for index in range(6):
        angle = math.radians(60 * index)
        points.append(
            f"{centre + radius * math.cos(angle):.2f},"
            f"{centre + radius * math.sin(angle):.2f}"
        )
    return " ".join(points)


def variant_svg(letter: str, round_number: int) -> str:
    """Draw a plain hex tile in the palette of one variant letter."""
    fill, edge, light = PALETTES[letter]
    outline = 2 + round_number % 3
    inner = 16 + 4 * (round_number % 4)
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128"'
        ' width="128" height="128">\n'
        f'  <polygon points="{hexagon_points(64, 60)}" fill="{fill}"'
        f' stroke="{edge}" stroke-width="{outline}"/>\n'
        f'  <circle cx="64" cy="{56 + round_number}" r="{inner}"'
        f' fill="{light}" opacity="0.75"/>\n'
        f'  <path d="M 32 84 Q 64 {70 - round_number * 2} 96 84" fill="none"'
        f' stroke="{edge}" stroke-width="3"/>\n'
        '  <text x="64" y="118" font-family="sans-serif" font-size="14"'
        f' fill="{edge}" text-anchor="middle">{letter}{round_number}</text>\n'
        "</svg>\n"
    )


def render(svg: str, target: Path, size: int) -> None:
    """Rasterize one SVG into a square PNG of the given size."""
    cairosvg.svg2png(
        bytestring=svg.encode("utf-8"),
        write_to=str(target),
        output_width=size,
        output_height=size,
    )


def critique(letter: str, round_number: int) -> dict:
    """Fabricate a critique that looks like one the model would write."""
    score = 40 + (round_number * 13 + LETTERS.index(letter) * 7) % 55
    verdict = VERDICTS[(round_number + LETTERS.index(letter)) % len(VERDICTS)]
    start = (round_number + LETTERS.index(letter)) % len(FAULTS)
    count = 1 + (round_number + LETTERS.index(letter)) % 3
    faults = [FAULTS[(start + step) % len(FAULTS)] for step in range(count)]
    return {"verdict": verdict, "faults": faults, "score": score}


def write_json(path: Path, payload: object) -> None:
    """Write one JSON document with a trailing newline."""
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def write_variant(
    directory: Path,
    letter: str,
    round_number: int,
    with_renders: bool = True,
    with_critique: bool = True,
    broken_critique: bool = False,
) -> None:
    """Write the files of one variant into a round directory."""
    svg = variant_svg(letter, round_number)
    stem = directory / f"variant-{letter}"
    stem.with_suffix(".svg").write_text(svg, encoding="utf-8")
    if with_renders:
        render(svg, directory / f"variant-{letter}.png", DISPLAY_SIZE)
        render(svg, directory / f"variant-{letter}.large.png", LARGE_SIZE)
    if broken_critique:
        (directory / f"variant-{letter}.critique.json").write_text(
            '{"verdict": "keep", "faults": [', encoding="utf-8"
        )
    elif with_critique:
        write_json(
            directory / f"variant-{letter}.critique.json",
            critique(letter, round_number),
        )


def write_round(
    session_directory: Path,
    number: int,
    letters: tuple[str, ...] = LETTERS,
    parent: str | None = None,
    summary: str = "",
    with_meta: bool = True,
    with_renders: bool = True,
    with_critique: bool = True,
    broken_critique: bool = False,
) -> Path:
    """Write one round directory, and give its path."""
    directory = session_directory / f"round-{number:02d}"
    directory.mkdir(parents=True, exist_ok=True)
    if with_meta:
        write_json(
            directory / "meta.json",
            {
                "round": number,
                "prompt_summary": summary or f"round {number} of the hex tile",
                "parent": parent,
            },
        )
    for letter in letters:
        write_variant(
            directory,
            letter,
            number,
            with_renders=with_renders,
            with_critique=with_critique,
            broken_critique=broken_critique,
        )
    return directory


def write_manifest(
    session_directory: Path, asset: str, created: datetime, rounds: int
) -> None:
    """Write the session manifest of the contract."""
    write_json(
        session_directory / "session.json",
        {
            "asset": asset,
            "created": created.isoformat(timespec="seconds").replace("+00:00", "Z"),
            "model": "fixture-model-1",
            "guide_version": "guide-2026-09-01",
            "rounds": rounds,
        },
    )


def build(root: Path, clean: bool = True) -> Path:
    """Write the whole fixture tree under the root, and give the root."""
    if clean and root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True, exist_ok=True)
    created = datetime(2026, 9, 1, 10, 0, tzinfo=UTC)

    # A healthy session: three complete rounds, and feedback on the first two.
    healthy = root / "hex-forest" / "sess-20260901-1000"
    healthy.mkdir(parents=True, exist_ok=True)
    write_manifest(healthy, "hex-forest", created, 3)
    write_round(healthy, 0, summary="first pass at the forest hex")
    write_round(
        healthy, 1, parent="round-00/variant-b", summary="heavier canopy, softer edge"
    )
    write_round(
        healthy, 2, parent="round-01/variant-c", summary="lift the value against grass"
    )
    write_json(
        healthy / "round-00" / "feedback.json",
        {
            "choice": "b",
            "text": "B reads best at tile size. The outline of A is too heavy.",
            "at": (created + timedelta(minutes=6))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )
    write_json(
        healthy / "round-01" / "feedback.json",
        {
            "choice": None,
            "text": "None of these. The canopy lost its silhouette. Go back to B.",
            "at": (created + timedelta(minutes=14))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )

    # A session that the loop is part way through writing.
    partial = root / "hex-mountain" / "sess-20260902-0930"
    partial.mkdir(parents=True, exist_ok=True)
    write_manifest(partial, "hex-mountain", created + timedelta(days=1), 4)
    # A complete round.
    write_round(partial, 0, summary="first pass at the mountain hex")
    # A round that holds one variant only.
    write_round(partial, 1, letters=("a",), summary="only variant a is written")
    # A round whose variants have no critique.
    write_round(partial, 2, with_critique=False, summary="the critique is not written")
    # A round with an SVG and no render, and no meta.json.
    write_round(partial, 3, letters=("a", "b"), with_renders=False, with_meta=False)
    # A round whose critique file holds broken JSON.
    write_round(partial, 4, letters=("a", "b"), broken_critique=True)

    # A session that holds no round at all.
    empty = root / "hex-water" / "sess-20260903-0800"
    empty.mkdir(parents=True, exist_ok=True)
    write_manifest(empty, "hex-water", created + timedelta(days=2), 0)

    # A session directory with no session.json and one round.
    nameless = root / "unit-scout" / "sess-20260903-1100"
    nameless.mkdir(parents=True, exist_ok=True)
    write_round(nameless, 0, letters=("a", "b", "c"), summary="no manifest yet")

    return root


def main() -> None:
    """Write the fixture tree at the path the caller names."""
    default = Path(__file__).resolve().parent.parent / "fixtures" / "sessions"
    parser = argparse.ArgumentParser(description="write a fake direct-die session tree")
    parser.add_argument(
        "root", nargs="?", type=Path, default=default, help="the sessions root to write"
    )
    parser.add_argument(
        "--keep",
        action="store_true",
        help="add to the tree instead of removing it first",
    )
    arguments = parser.parse_args()
    root = build(arguments.root, clean=not arguments.keep)
    print(f"wrote the fixture sessions to {root}")


if __name__ == "__main__":
    main()
