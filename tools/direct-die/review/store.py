"""The read side of the direct-die session directory, and the one write.

The generation loop writes every file under a session directory. This module
writes one file only: `feedback.json` in a round directory.

The loop runs while a person reads the pages. A directory is therefore
incomplete much of the time. Every function here returns a partial record
instead of raising. A missing file becomes `None`. A bad JSON file becomes
`None`. The page shows a gap.

This module caches nothing. Each call reads the disk again.
"""

from __future__ import annotations

import json
import os
import re
import tempfile
from collections.abc import Sequence
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path

# The four variant letters of a round. The contract fixes this set.
VARIANT_LETTERS = ("a", "b", "c", "d")

# A round directory is `round-NN`. The number holds two digits or more.
ROUND_DIRECTORY = re.compile(r"^round-(\d+)$")


class ContractError(ValueError):
    """A caller gave a name that the on-disk contract does not permit."""


def read_json(path: Path) -> dict | None:
    """Read one JSON object from the path.

    Return `None` when the file is absent, when it is half-written, or when
    it holds something other than a JSON object. The generation loop writes
    these files while a person reads them, so a partial read is normal.
    """
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None
    try:
        value = json.loads(text)
    except json.JSONDecodeError:
        return None
    if not isinstance(value, dict):
        return None
    return value


@dataclass(frozen=True)
class Variant:
    """One variant of one round.

    Each field is `None` when the file behind it is absent. The loop writes
    the SVG, the two renders, and the critique at different moments.
    """

    letter: str
    svg: str | None = None
    display_png: str | None = None
    large_png: str | None = None
    critique: dict | None = None

    @property
    def present(self) -> bool:
        """Report whether the loop wrote anything at all for this variant."""
        return any(
            (self.svg, self.display_png, self.large_png, self.critique is not None)
        )

    @property
    def verdict(self) -> str | None:
        """Give the model verdict, or `None` when the critique is absent."""
        if self.critique is None:
            return None
        value = self.critique.get("verdict")
        return value if isinstance(value, str) else None

    @property
    def score(self) -> int | None:
        """Give the model score, or `None` when the critique is absent."""
        if self.critique is None:
            return None
        value = self.critique.get("score")
        return value if isinstance(value, int) else None

    @property
    def faults(self) -> list[str]:
        """List the model faults. Give an empty list when there are none."""
        if self.critique is None:
            return []
        value = self.critique.get("faults")
        if not isinstance(value, list):
            return []
        return [item for item in value if isinstance(item, str)]


def _letters(value: object) -> tuple[str, ...]:
    """Take the variant letters out of a value, once each, in order."""
    if not isinstance(value, list):
        return ()
    found: list[str] = []
    for item in value:
        if isinstance(item, str) and item in VARIANT_LETTERS and item not in found:
            found.append(item)
    return tuple(found)


def read_feedback(
    value: dict | None,
) -> tuple[tuple[str, ...], tuple[str, ...], tuple[str, ...], str, str]:
    """Read one feedback object into likes, refusals, order, note and text.

    The function never raises. It drops what it cannot read, because a person
    can edit the file by hand and a page must still render.

    The file held a `choice` field, which named one variant. Nothing writes
    that field now. A file that holds `choice` and no `likes` reads as one
    like, so every session already on disk still opens. The generation loop
    holds the same rule in its own reader, and one test compares the two.[^1]

    ## References

    [^1]: The reader of the loop. `tools/direct-die/direct_die/steer.py`
    """
    if not isinstance(value, dict):
        return (), (), (), "", ""
    denies = _letters(value.get("denies"))
    if "likes" in value:
        likes = _letters(value.get("likes"))
    else:
        choice = value.get("choice")
        likes = (
            (choice,) if isinstance(choice, str) and choice in VARIANT_LETTERS else ()
        )
    likes = tuple(letter for letter in likes if letter not in denies)
    ranked = [letter for letter in _letters(value.get("order")) if letter in likes]
    ranked.extend(letter for letter in likes if letter not in ranked)
    note = value.get("note")
    text = value.get("text")
    return (
        likes,
        denies,
        tuple(ranked),
        note.strip() if isinstance(note, str) else "",
        text if isinstance(text, str) else "",
    )


@dataclass(frozen=True)
class Round:
    """One round of one session."""

    number: int
    name: str
    meta: dict | None
    variants: list[Variant] = field(default_factory=list)
    feedback: dict | None = None
    analysis: dict | None = None

    @property
    def prompt_summary(self) -> str | None:
        """Give the prompt summary that the loop recorded for this round."""
        if self.meta is None:
            return None
        value = self.meta.get("prompt_summary")
        return value if isinstance(value, str) else None

    @property
    def present_variants(self) -> list[Variant]:
        """List the variants that hold at least one file."""
        return [variant for variant in self.variants if variant.present]

    @property
    def _read(self) -> tuple:
        """Read the feedback once, for the properties below."""
        return read_feedback(self.feedback)

    @property
    def likes(self) -> tuple[str, ...]:
        """Give the letters that the person accepted."""
        return self._read[0]

    @property
    def denies(self) -> tuple[str, ...]:
        """Give the letters that the person refused."""
        return self._read[1]

    @property
    def order(self) -> tuple[str, ...]:
        """Give the accepted letters, best first."""
        return self._read[2]

    @property
    def note(self) -> str:
        """Give the standing note. Give "" when there is none."""
        return self._read[3]

    @property
    def winner(self) -> str | None:
        """Give the letter that stands for this round, or `None`.

        It is the first of the order, which is the drawing the person put
        first. A refused drawing is never the winner, because the reader
        drops a refused letter from the likes.
        """
        found = self.order
        return found[0] if found else None

    @property
    def parents(self) -> dict[str, str]:
        """Give the parent that each variant of this round revised.

        The loop wrote one `parent` string before it could branch. A round
        that holds the old field gives that one value to every variant.
        """
        if self.meta is None:
            return {}
        found = self.meta.get("parents")
        if isinstance(found, dict):
            return {
                letter: value
                for letter, value in found.items()
                if letter in VARIANT_LETTERS and isinstance(value, str)
            }
        single = self.meta.get("parent")
        if isinstance(single, str):
            return {variant.letter: single for variant in self.present_variants}
        return {}

    @property
    def feedback_text(self) -> str:
        """Give the note for this round only. Give "" when there is none."""
        return self._read[4]


@dataclass(frozen=True)
class Session:
    """One session of one asset."""

    asset: str
    session_id: str
    manifest: dict | None
    rounds: list[Round] = field(default_factory=list)

    @property
    def key(self) -> str:
        """Give the two-part name that a URL uses for this session."""
        return f"{self.asset}/{self.session_id}"

    @property
    def subject(self) -> str | None:
        """Give the subject that the loop drew, as the round metadata says.

        The manifest holds no subject. The loop writes the subject into the
        prompt summary of each round, before a semicolon. This reads the
        first round that has one. Give `None` when no round has one.
        """
        for entry in self.rounds:
            summary = entry.prompt_summary
            if summary:
                first = summary.split(";")[0].strip()
                if first:
                    return first
        return None

    @property
    def created(self) -> str | None:
        """Give the creation time that the loop recorded."""
        return self._manifest_string("created")

    @property
    def model(self) -> str | None:
        """Give the model name that the loop recorded."""
        return self._manifest_string("model")

    @property
    def guide_version(self) -> str | None:
        """Give the style guide version that the loop recorded."""
        return self._manifest_string("guide_version")

    @property
    def declared_rounds(self) -> int | None:
        """Give the round count that the manifest declares.

        The count can disagree with the directory, because the loop writes a
        round directory before it updates the manifest. The pages show both
        and trust the directory.
        """
        if self.manifest is None:
            return None
        value = self.manifest.get("rounds")
        return value if isinstance(value, int) else None

    def _manifest_string(self, key: str) -> str | None:
        if self.manifest is None:
            return None
        value = self.manifest.get(key)
        return value if isinstance(value, str) else None


def safe_name(name: str) -> str:
    """Reject a path segment that could leave the directory it names."""
    if not name or name in (".", "..") or "/" in name or "\\" in name:
        raise ContractError(f"unsafe path segment: {name!r}")
    return name


# The old private name. The run manager and the pack export call the public
# one. This alias keeps one definition of the rule.
_safe_name = safe_name


def write_json_atomically(path: Path, payload: object) -> Path:
    """Write one JSON document, and give the path.

    The write goes to a temporary name in the same directory, and then
    renames. A rename inside one directory is atomic, so a reader never sees
    half a file. Every write in this tool takes this form.
    """
    directory = path.parent
    directory.mkdir(parents=True, exist_ok=True)
    handle, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}-", suffix=".tmp", dir=directory
    )
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as stream:
            json.dump(payload, stream, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, path)
    except BaseException:
        Path(temporary_name).unlink(missing_ok=True)
        raise
    return path


class SessionStore:
    """Read sessions from one root directory, and write feedback into them."""

    def __init__(self, root: Path) -> None:
        """Hold the sessions root that every method reads."""
        self.root = Path(root)

    # -- discovery ---------------------------------------------------------

    def list_sessions(self) -> list[Session]:
        """List every session under the root, newest asset name last.

        Each session carries its manifest and its rounds. The root itself can
        be absent, which gives an empty list.
        """
        sessions: list[Session] = []
        for asset_directory in self._sorted_directories(self.root):
            for session_directory in self._sorted_directories(asset_directory):
                sessions.append(
                    self.load_session(asset_directory.name, session_directory.name)
                )
        return sessions

    @staticmethod
    def _sorted_directories(parent: Path) -> list[Path]:
        try:
            entries = list(parent.iterdir())
        except OSError:
            return []
        return sorted(
            (entry for entry in entries if entry.is_dir()), key=lambda p: p.name
        )

    def session_directory(self, asset: str, session_id: str) -> Path:
        """Give the directory of one session."""
        return self.root / _safe_name(asset) / _safe_name(session_id)

    def round_directory(self, asset: str, session_id: str, round_name: str) -> Path:
        """Give the directory of one round."""
        if not ROUND_DIRECTORY.match(_safe_name(round_name)):
            raise ContractError(f"not a round directory name: {round_name!r}")
        return self.session_directory(asset, session_id) / round_name

    # -- reading -----------------------------------------------------------

    def load_session(self, asset: str, session_id: str) -> Session:
        """Read one session and every round in it.

        Return a session with an empty round list when the directory holds no
        round. Return a session with no manifest when `session.json` is
        absent. Neither case raises.
        """
        directory = self.session_directory(asset, session_id)
        manifest = read_json(directory / "session.json")
        rounds = [
            self.load_round(asset, session_id, name)
            for name in self.round_names(asset, session_id)
        ]
        return Session(
            asset=asset, session_id=session_id, manifest=manifest, rounds=rounds
        )

    def round_names(self, asset: str, session_id: str) -> list[str]:
        """List the round directory names of one session, in round order."""
        directory = self.session_directory(asset, session_id)
        names: list[tuple[int, str]] = []
        for entry in self._sorted_directories(directory):
            match = ROUND_DIRECTORY.match(entry.name)
            if match is not None:
                names.append((int(match.group(1)), entry.name))
        names.sort()
        return [name for _, name in names]

    def load_round(self, asset: str, session_id: str, round_name: str) -> Round:
        """Read one round, with every variant that has a file on disk."""
        directory = self.round_directory(asset, session_id, round_name)
        match = ROUND_DIRECTORY.match(round_name)
        number = int(match.group(1)) if match else -1
        meta = read_json(directory / "meta.json")
        feedback = read_json(directory / "feedback.json")
        analysis = read_json(directory / "analysis.json")
        variants = [self._load_variant(directory, letter) for letter in VARIANT_LETTERS]
        return Round(
            number=number,
            name=round_name,
            meta=meta,
            variants=variants,
            feedback=feedback,
            analysis=analysis,
        )

    @staticmethod
    def _load_variant(directory: Path, letter: str) -> Variant:
        stem = f"variant-{letter}"

        def name_if_present(suffix: str) -> str | None:
            candidate = directory / f"{stem}{suffix}"
            return candidate.name if candidate.is_file() else None

        return Variant(
            letter=letter,
            svg=name_if_present(".svg"),
            display_png=name_if_present(".png"),
            large_png=name_if_present(".large.png"),
            critique=read_json(directory / f"{stem}.critique.json"),
        )

    def asset_file(
        self, asset: str, session_id: str, round_name: str, file_name: str
    ) -> Path | None:
        """Give the path of one file in a round, or `None` when it is absent.

        The file name must belong to the contract. This rejects any other
        name, so a request cannot read outside the round directory.
        """
        _safe_name(file_name)
        permitted = set()
        for letter in VARIANT_LETTERS:
            permitted.update(
                {
                    f"variant-{letter}.svg",
                    f"variant-{letter}.png",
                    f"variant-{letter}.large.png",
                }
            )
        if file_name not in permitted:
            raise ContractError(f"not a variant file name: {file_name!r}")
        path = self.round_directory(asset, session_id, round_name) / file_name
        return path if path.is_file() else None

    @staticmethod
    def round_index(round_name: str) -> int:
        """Give the index that a round directory name holds.

        Raise `ContractError` when the name is not a round directory name.
        """
        if not round_name.startswith("round-"):
            raise ContractError(f"not a round name: {round_name!r}")
        try:
            return int(round_name[len("round-") :])
        except ValueError as error:
            raise ContractError(f"not a round name: {round_name!r}") from error

    # -- the one write -----------------------------------------------------

    def write_feedback(
        self,
        asset: str,
        session_id: str,
        round_name: str,
        likes: Sequence[str],
        denies: Sequence[str],
        order: Sequence[str],
        note: str,
        text: str,
    ) -> Path:
        """Write `feedback.json` into a round directory, and give its path.

        The write is atomic. It writes a temporary file in the same directory
        and then renames it, so the generation loop never reads half a file.

        Raise `ContractError` when a letter is not a variant letter, when a
        letter is in both lists, when the order is not the same set as the
        likes, and when the round directory does not exist. The loop owns
        that directory, so this module does not create one.
        """
        index = self.round_index(round_name)
        for group in (likes, denies, order):
            for letter in group:
                if letter not in VARIANT_LETTERS:
                    raise ContractError(f"not a variant letter: {letter!r}")
        both = sorted(set(likes) & set(denies))
        if both:
            raise ContractError(
                f"a letter cannot be liked and refused: {', '.join(both)}"
            )
        if sorted(set(order)) != sorted(set(likes)):
            raise ContractError("the order must hold every liked letter once")
        directory = self.round_directory(asset, session_id, round_name)
        if not directory.is_dir():
            raise ContractError(f"no such round: {asset}/{session_id}/{round_name}")
        payload = {
            "round": index,
            "likes": list(dict.fromkeys(likes)),
            "denies": list(dict.fromkeys(denies)),
            "order": list(dict.fromkeys(order)),
            "note": note.strip(),
            "text": text,
            "at": datetime.now(UTC)
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        }
        return write_json_atomically(directory / "feedback.json", payload)
