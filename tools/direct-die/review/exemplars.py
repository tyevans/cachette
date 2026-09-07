"""Promote a drawing to an exemplar of its style, and list what is accepted.

The critic judges a candidate against pictures, not only against adjectives.
Those pictures are the exemplars of the style. An exemplar directory starts
empty, so the first run of a new style has nothing to aim at. Promotion is
the step that fills it. It is how the next run gets better.[^1]

## What promotion does

It copies the SVG source of the chosen drawing into the exemplar directory
of the style, under the engine asset name. The next run reads the directory
again, so the drawing becomes part of the guide with no other step.

The guide version is a digest of the rules and of every exemplar file. A
promotion therefore changes the guide version, and each session records the
version it ran against.

## What this module does not touch

Another person owns the rules of each style. This module writes into the
exemplar directory of a style and nowhere else. It reads the rules files
only to learn which styles exist.

## The limit that a person must know

The critic attaches a fixed number of exemplars, sorted by file name. A
directory with many exemplars therefore sends the first few only. Promote
the drawings that state the style, not every drawing that a person likes.

## References

[^1]: The tool guide. `tools/direct-die/README.md`
"""

from __future__ import annotations

import shutil
from dataclasses import dataclass
from pathlib import Path

from store import safe_name

# The file that states the score scale. It is not a style.
SCORE_FILE = "score.md"

# The suffixes that the guide loader reads as an exemplar picture.
EXEMPLAR_SUFFIXES = (".svg", ".png")


@dataclass(frozen=True)
class Exemplar:
    """One accepted picture of one style."""

    name: str

    @property
    def slug(self) -> str:
        """Give the file name without its suffix."""
        return Path(self.name).stem


def styles(styleguide_root: Path) -> list[str]:
    """List the styles that the guide covers, by their rules files.

    The guide holds one markdown file for each style, and one file for the
    score scale. The score scale is not a style, so this drops it.

    The tool answers the same question in its own loader. This reads the
    directory instead of importing the tool, so that a change inside the tool
    cannot stop the server. The directory is the one declaration of the list.
    """
    try:
        entries = list(Path(styleguide_root).glob("*.md"))
    except OSError:
        return []
    return sorted(path.stem for path in entries if path.name != SCORE_FILE)


def exemplar_directory(styleguide_root: Path, style: str) -> Path:
    """Give the exemplar directory of one style."""
    return Path(styleguide_root) / "exemplars" / safe_name(style)


def list_exemplars(styleguide_root: Path, style: str) -> list[Exemplar]:
    """List the accepted pictures of one style, in the order the critic reads.

    Give an empty list when the directory is absent. A new style has no
    exemplar, and that is the normal first state.
    """
    directory = exemplar_directory(styleguide_root, style)
    try:
        entries = list(directory.iterdir())
    except OSError:
        return []
    found = [
        entry
        for entry in entries
        if entry.is_file() and entry.suffix.lower() in EXEMPLAR_SUFFIXES
    ]
    return [
        Exemplar(name=path.name) for path in sorted(found, key=lambda path: path.name)
    ]


def has_exemplar(styleguide_root: Path, style: str, slug: str) -> bool:
    """Report whether the style holds an exemplar under one asset name."""
    return any(
        item.slug == safe_name(slug) for item in list_exemplars(styleguide_root, style)
    )


def promote(styleguide_root: Path, style: str, slug: str, source: Path) -> Path:
    """Copy one SVG drawing into the exemplar directory, and give its path.

    The exemplar takes the engine asset name, so a second promotion of the
    same asset replaces the first. One asset of one style therefore has one
    exemplar, and the directory cannot fill with near copies.
    """
    safe_name(slug)
    source = Path(source)
    if source.suffix.lower() != ".svg":
        raise ValueError(f"an exemplar comes from an SVG file: {source.name!r}")
    if not source.is_file():
        raise ValueError(f"no such drawing: {source}")
    directory = exemplar_directory(styleguide_root, style)
    directory.mkdir(parents=True, exist_ok=True)
    target = directory / f"{slug}.svg"
    shutil.copyfile(source, target)
    return target


def withdraw(styleguide_root: Path, style: str, name: str) -> bool:
    """Remove one exemplar, and report whether a file went away.

    A person needs this when a promotion pulls the style the wrong way. The
    call removes a picture inside the exemplar directory of one style, and
    refuses every other name.
    """
    safe_name(name)
    if Path(name).suffix.lower() not in EXEMPLAR_SUFFIXES:
        raise ValueError(f"not an exemplar file name: {name!r}")
    target = exemplar_directory(styleguide_root, style) / name
    if not target.is_file():
        return False
    target.unlink()
    return True


def exemplar_file(styleguide_root: Path, style: str, name: str) -> Path | None:
    """Give the path of one exemplar picture, or `None` when it is absent."""
    safe_name(name)
    if Path(name).suffix.lower() not in EXEMPLAR_SUFFIXES:
        raise ValueError(f"not an exemplar file name: {name!r}")
    target = exemplar_directory(styleguide_root, style) / name
    return target if target.is_file() else None
