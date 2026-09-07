"""Choose one drawing for each asset, and export a style as a pack.

A session holds many drawings. A pack holds one drawing for each asset name
that the engine uses. This module decides which drawing that is, and writes
the chosen ones to disk in a layout that something else can load.

## Which drawing wins

The person outranks the model, as the tool guide states.[^1] The rule
therefore reads the sessions of one style and one asset, newest session
first, and takes the drawing that the person put first. When no person
chose, it takes the highest score of every round of every session. A refused
drawing never wins, whatever the model scored it. A later round wins a tie,
so the pack follows the work.

## Which session draws which asset

The tool records no asset name in the session manifest. The server names a
session after the slug when it starts a run, so the directory name carries
the asset name. A session that a person started at the command line has no
such name. The rule then reads the subject text out of the round metadata
and looks for an asset name in it. Nothing stores the asset name twice.

## The pack on disk

    packs/<style>/
      pack.json
      <slug>.svg
      <slug>.png        the display size render

A pack with gaps is legal and normal. `pack.json` names what is present and
nothing else.

## References

[^1]: The tool guide. `tools/direct-die/README.md`
"""

from __future__ import annotations

import shutil
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path

import slugs as slug_table
from store import (
    Round,
    Session,
    SessionStore,
    Variant,
    read_json,
    safe_name,
    write_json_atomically,
)

# The name of the manifest inside one pack directory.
PACK_MANIFEST = "pack.json"


def slug_of(session: Session) -> str | None:
    """Give the engine asset name that one session draws, if it has one.

    The session identifier carries the name when the server started the run.
    A session from the command line does not, so this reads the subject out
    of the round metadata and looks for a name in the words.
    """
    parts = session.session_id.split("-")
    for part in reversed(parts):
        if slug_table.is_slug(part):
            return part
    subject = session.subject
    if subject:
        words = subject.lower().replace(",", " ").replace(".", " ").split()
        for slug in slug_table.SLUGS:
            if slug in words or f"{slug}s" in words:
                return slug
    return None


@dataclass(frozen=True)
class Pick:
    """The one drawing that stands for a style and an asset name."""

    style: str
    slug: str
    session_id: str
    round_name: str
    letter: str
    score: int | None
    source: str
    svg_name: str | None
    png_name: str | None

    @property
    def complete(self) -> bool:
        """Report whether this pick holds both files that a pack needs."""
        return bool(self.svg_name and self.png_name)

    @property
    def chosen_by_person(self) -> bool:
        """Report whether a person picked this drawing."""
        return self.source == "human"


def sessions_of(
    store: SessionStore, style: str, sessions: list[Session] | None = None
) -> list[Session]:
    """List the sessions of one style, oldest first.

    Pass a session list to read the disk once for a whole page. The matrix
    view does that, because it asks about every style and every asset name.
    """
    found = store.list_sessions() if sessions is None else sessions
    return [item for item in found if item.asset == style]


def sessions_for(
    store: SessionStore,
    style: str,
    slug: str,
    sessions: list[Session] | None = None,
) -> list[Session]:
    """List the sessions of one style that draw one asset name."""
    return [
        item for item in sessions_of(store, style, sessions) if slug_of(item) == slug
    ]


def pick_for(
    store: SessionStore,
    style: str,
    slug: str,
    sessions: list[Session] | None = None,
) -> Pick | None:
    """Give the drawing that stands for one style and one asset name.

    Give `None` when no session of that style drew that asset, and when
    every session that did holds no drawing yet.
    """
    found = sessions_for(store, style, slug, sessions)
    for session in reversed(found):
        for entry in reversed(session.rounds):
            letter = entry.winner
            if letter is None:
                continue
            variant = _variant(entry, letter)
            if variant is None or variant.svg is None:
                continue
            return Pick(
                style=style,
                slug=slug,
                session_id=session.session_id,
                round_name=entry.name,
                letter=letter,
                score=variant.score,
                source="human",
                svg_name=variant.svg,
                png_name=variant.display_png,
            )

    best: Pick | None = None
    best_key: tuple[int, str, int, str] | None = None
    for session in found:
        for entry in session.rounds:
            for variant in entry.present_variants:
                if variant.svg is None:
                    continue
                if variant.letter in entry.denies:
                    # A person refused this drawing. It never stands for the
                    # asset, whatever the model scored it.
                    continue
                # A drawing with no critique still counts. The critique
                # arrives after the picture, and a person may want the
                # picture before the model says anything about it.
                rank = variant.score if variant.score is not None else -1
                key = (rank, session.session_id, entry.number, variant.letter)
                if best_key is None or key > best_key:
                    best_key = key
                    best = Pick(
                        style=style,
                        slug=slug,
                        session_id=session.session_id,
                        round_name=entry.name,
                        letter=variant.letter,
                        score=variant.score,
                        source="score",
                        svg_name=variant.svg,
                        png_name=variant.display_png,
                    )
    return best


def _variant(entry: Round, letter: str) -> Variant | None:
    for variant in entry.variants:
        if variant.letter == letter:
            return variant
    return None


@dataclass
class ExportReport:
    """What one export wrote, and what it could not write."""

    style: str
    directory: Path
    written: list[str] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)
    removed: list[str] = field(default_factory=list)


def pack_directory(packs_root: Path, style: str) -> Path:
    """Give the directory of one pack."""
    return Path(packs_root) / safe_name(style)


def read_pack(packs_root: Path, style: str) -> dict | None:
    """Read the manifest of one pack, or give `None` when there is none."""
    return read_json(pack_directory(packs_root, style) / PACK_MANIFEST)


def pack_assets(packs_root: Path, style: str) -> dict:
    """Give the asset table of one pack. Give an empty table for a gap."""
    manifest = read_pack(packs_root, style)
    if manifest is None:
        return {}
    assets = manifest.get("assets")
    return assets if isinstance(assets, dict) else {}


def export_pack(
    store: SessionStore,
    packs_root: Path,
    style: str,
    only: list[str] | None = None,
    sessions: list[Session] | None = None,
) -> ExportReport:
    """Write the chosen drawings of one style as a pack, and report.

    An asset with no drawing, and an asset whose render is not on disk yet,
    is a gap. The manifest names neither. The report says why each one is
    absent, so a person can see what to draw next.
    """
    directory = pack_directory(packs_root, style)
    directory.mkdir(parents=True, exist_ok=True)
    report = ExportReport(style=style, directory=directory)
    wanted = list(only) if only else list(slug_table.SLUGS)
    assets: dict[str, dict] = {}

    known = store.list_sessions() if sessions is None else sessions
    for slug in wanted:
        pick = pick_for(store, style, slug, known)
        if pick is None:
            report.skipped.append((slug, "nothing is drawn for it yet"))
            continue
        if not pick.complete:
            report.skipped.append((slug, "the display render is not on disk yet"))
            continue
        try:
            svg_path = store.asset_file(
                style, pick.session_id, pick.round_name, pick.svg_name or ""
            )
            png_path = store.asset_file(
                style, pick.session_id, pick.round_name, pick.png_name or ""
            )
        except ValueError as error:
            report.skipped.append((slug, str(error)))
            continue
        if svg_path is None or png_path is None:
            report.skipped.append((slug, "a file went away during the export"))
            continue
        shutil.copyfile(svg_path, directory / f"{slug}.svg")
        shutil.copyfile(png_path, directory / f"{slug}.png")
        assets[slug] = {
            "svg": f"{slug}.svg",
            "png": f"{slug}.png",
            "score": pick.score,
            "session": pick.session_id,
            "round": pick.round_name,
            "variant": pick.letter,
        }
        report.written.append(slug)

    write_json_atomically(
        directory / PACK_MANIFEST,
        {
            "style": style,
            "created": datetime.now(UTC)
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
            "assets": assets,
        },
    )

    # Remove a drawing that an earlier export wrote and this one did not.
    # A file that the manifest does not name is a lie to the next reader.
    for slug in slug_table.SLUGS:
        if slug in assets:
            continue
        for suffix in (".svg", ".png"):
            stale = directory / f"{slug}{suffix}"
            if stale.is_file():
                stale.unlink()
                report.removed.append(stale.name)
    return report


def pack_file(packs_root: Path, style: str, file_name: str) -> Path | None:
    """Give the path of one file in a pack, or `None` when it is absent.

    The name must be an asset name with a permitted suffix, or the manifest.
    This refuses any other name, so a request cannot read outside the pack.
    """
    safe_name(file_name)
    permitted = {PACK_MANIFEST}
    for slug in slug_table.SLUGS:
        permitted.update({f"{slug}.svg", f"{slug}.png"})
    if file_name not in permitted:
        raise ValueError(f"not a pack file name: {file_name!r}")
    path = pack_directory(packs_root, style) / file_name
    return path if path.is_file() else None
