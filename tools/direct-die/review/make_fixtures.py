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
        parents = {letter: parent for letter in letters} if parent is not None else {}
        write_json(
            directory / "meta.json",
            {
                "round": number,
                "prompt_summary": summary or f"round {number} of the hex tile",
                "parents": parents,
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
    for round_name in ("round-01", "round-02"):
        meta_path = healthy / round_name / "meta.json"
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        meta["parents"] = {
            "a": "round-00/variant-b",
            "b": "round-00/variant-d",
            "c": "round-00/variant-b",
            "d": "round-00/variant-d",
        }
        write_json(meta_path, meta)
    write_json(
        healthy / "round-00" / "feedback.json",
        {
            "round": 0,
            "likes": ["b"],
            "denies": [],
            "order": ["b"],
            "note": "",
            "text": "B reads best at tile size. The outline of A is too heavy.",
            "at": (created + timedelta(minutes=6))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )
    write_json(
        healthy / "round-01" / "feedback.json",
        {
            "round": 1,
            "likes": ["b", "d"],
            "denies": ["a"],
            "order": ["d", "b"],
            "note": "keep the palette flat",
            "text": "raise the contrast of the base",
            "at": "2026-09-01T10:20:00Z",
        },
    )
    write_json(
        healthy / "round-01" / "analysis.json",
        {
            "round": 1,
            "preference": "The accepted drawings hold three shapes and one flat fill.",
            "order": ["d", "b"],
            "reasons": {"d": "the silhouette reads at 64 pixels", "b": ""},
            "guide_edit": {
                "section": "Shape language",
                "rule": "Use three shapes or fewer inside the hexagon.",
            },
            "at": "2026-09-01T10:25:00Z",
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
    # A round whose variants have no critique. Every letter gets the same
    # parent, as the regression proof that one parent covers every variant.
    write_round(
        partial,
        2,
        parent="round-00/variant-a",
        with_critique=False,
        summary="the critique is not written",
    )
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

    build_style_sessions(root, created)
    return root


# The styles that the fixture guide names. The real guide names its own.
FIXTURE_STYLES = ("cartoon", "pencil")


def build_style_sessions(root: Path, created: datetime) -> None:
    """Write sessions that a style run writes, so the grid has cells.

    The server names a session after the engine asset name when it starts a
    run. These names follow that shape, so the grid reads the asset name out
    of the directory name.

    The tree holds one complete asset with a human choice, one asset that is
    part way through, one asset whose name only the subject text gives, and
    one session with no round at all.
    """
    # A cartoon forest that a person chose in the second round.
    chosen = root / "cartoon" / "20260904-090000-forest"
    chosen.mkdir(parents=True, exist_ok=True)
    write_manifest(chosen, "cartoon", created + timedelta(days=3), 2)
    write_round(chosen, 0, summary="a dense stand of trees")
    write_round(
        chosen, 1, parent="round-00/variant-b", summary="a dense stand of trees"
    )
    write_json(
        chosen / "round-01" / "feedback.json",
        {
            "round": 1,
            "likes": ["b", "d"],
            "denies": ["a"],
            "order": ["b", "d"],
            "note": "Keep the canopy shape.",
            "text": "B reads at tile size.",
            "at": (created + timedelta(days=3, minutes=5))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )
    write_json(
        chosen / "round-01" / "analysis.json",
        {
            "round": 1,
            "preference": "The accepted drawings hold three shapes and one flat fill.",
            "order": ["d", "b"],
            "reasons": {"d": "the silhouette reads at 64 pixels", "b": ""},
            "guide_edit": {
                "section": "Shape language",
                "rule": "Use three shapes or fewer inside the hexagon.",
            },
            "at": (created + timedelta(days=3, minutes=8))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )

    # A cartoon water that the loop is part way through. One variant is
    # drawn and has no critique. This is the live case.
    live = root / "cartoon" / "20260904-091000-water"
    live.mkdir(parents=True, exist_ok=True)
    write_manifest(live, "cartoon", created + timedelta(days=3, minutes=10), 1)
    write_round(live, 0, letters=("a",), with_critique=False, summary="open water")

    # A cartoon hill whose name only the subject text gives. The session
    # identifier carries no asset name, as a command line run does not.
    unnamed = root / "cartoon" / "20260904-092000"
    unnamed.mkdir(parents=True, exist_ok=True)
    write_manifest(unnamed, "cartoon", created + timedelta(days=3, minutes=20), 1)
    write_round(unnamed, 0, letters=("a", "b"), summary="a low rolling hill")

    # A pencil session that holds no round, and no manifest.
    empty = root / "pencil" / "20260904-093000-mountain"
    empty.mkdir(parents=True, exist_ok=True)


def build_styleguide(root: Path, clean: bool = True) -> Path:
    """Write a fake style guide, with two styles and no exemplar.

    An exemplar directory starts empty. That is the state a person meets
    before the first promotion, and the grid must render it.
    """
    root = Path(root)
    if clean and root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True, exist_ok=True)
    (root / "score.md").write_text(
        "# The score scale\n\nThe fixture score scale.\n", encoding="utf-8"
    )
    for style in FIXTURE_STYLES:
        (root / f"{style}.md").write_text(
            f"# The {style} style\n\nThe fixture rules of the {style} style.\n",
            encoding="utf-8",
        )
        (root / "exemplars" / style).mkdir(parents=True, exist_ok=True)
    return root


def build_workspace(root: Path, clean: bool = True) -> dict[str, Path]:
    """Write every directory that the front end reads, and give the paths.

    The front end reads four roots: the sessions, the style guide, the packs
    and the runs. This writes the first two with data, and makes the other
    two empty. An empty packs directory and an empty runs directory are the
    first state a person meets.
    """
    root = Path(root)
    paths = {
        "sessions": root / "sessions",
        "styleguide": root / "styleguide",
        "packs": root / "packs",
        "runs": root / "runs",
    }
    build(paths["sessions"], clean=clean)
    build_styleguide(paths["styleguide"], clean=clean)
    paths["packs"].mkdir(parents=True, exist_ok=True)
    paths["runs"].mkdir(parents=True, exist_ok=True)
    return paths


def main() -> None:
    """Write the fixture workspace at the path the caller names."""
    default = Path(__file__).resolve().parent.parent / "fixtures"
    parser = argparse.ArgumentParser(
        description="write a fake direct-die workspace for the front end"
    )
    parser.add_argument(
        "root",
        nargs="?",
        type=Path,
        default=default,
        help="the workspace root to write",
    )
    parser.add_argument(
        "--keep",
        action="store_true",
        help="add to the tree instead of removing it first",
    )
    arguments = parser.parse_args()
    paths = build_workspace(arguments.root, clean=not arguments.keep)
    print(f"wrote the fixture workspace to {arguments.root}")
    print(
        "start the server with: python app.py"
        f" --sessions {paths['sessions']}"
        f" --styleguide {paths['styleguide']}"
        f" --packs {paths['packs']}"
        f" --runs {paths['runs']}"
    )


if __name__ == "__main__":
    main()
