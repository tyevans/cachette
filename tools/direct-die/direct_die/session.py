"""The session store on disk.

One session holds the rounds of one asset. The review interface reads the
same directory while the loop writes it. Every write therefore goes to a
temporary name in the same directory, and then renames. A rename inside
one directory is atomic, so the interface never reads half a file.

The layout is fixed. Do not change a name here without changing the
review interface too.

    sessions/<asset>/<session-id>/
      session.json
      round-00/
        meta.json
        variant-a.svg
        variant-a.png
        variant-a.large.png
        variant-a.critique.json
        feedback.json     the review interface writes this file

The critique file holds a score. The score runs from 0 to 100, and it
means the same thing in every round. The loop clamps the value that the
model gives into that range.

The feedback file names the round that it belongs to, the chosen
variant, the text of the human direction, and the time. The loop reads
this file. The loop never writes it.
"""

from __future__ import annotations

import json
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path

# The root of the session store. It sits beside the package.
SESSION_ROOT = Path(__file__).resolve().parent.parent / "sessions"

# The variant letters, in order.
VARIANT_LETTERS = ["a", "b", "c", "d"]


def now_iso() -> str:
    """Give the present time as an ISO 8601 string in UTC."""
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def new_session_id() -> str:
    """Give a session identifier that sorts by time."""
    return datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")


def write_bytes(path: Path, data: bytes) -> None:
    """Write bytes to the path atomically."""
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(dir=str(path.parent), prefix=".tmp-")
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


def write_text(path: Path, text: str) -> None:
    """Write text to the path atomically."""
    write_bytes(path, text.encode("utf-8"))


def write_json(path: Path, value: object) -> None:
    """Write a JSON document to the path atomically."""
    write_text(path, json.dumps(value, indent=2, sort_keys=True) + "\n")


def read_json(path: Path) -> dict | None:
    """Read a JSON document, or give None when it is absent or broken."""
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def round_name(index: int) -> str:
    """Give the directory name of one round."""
    return f"round-{index:02d}"


class Session:
    """One session directory, and the paths inside it."""

    def __init__(self, asset: str, session_id: str, root: Path = SESSION_ROOT):
        self.asset = asset
        self.session_id = session_id
        self.path = root / asset / session_id

    def round_path(self, index: int) -> Path:
        """Give the directory of one round."""
        return self.path / round_name(index)

    def existing_rounds(self) -> list[int]:
        """Give the indices of the rounds that are already on disk."""
        if not self.path.is_dir():
            return []
        found = []
        for entry in self.path.iterdir():
            if entry.is_dir() and entry.name.startswith("round-"):
                try:
                    found.append(int(entry.name[len("round-") :]))
                except ValueError:
                    continue
        return sorted(found)

    def write_header(self, model: str, guide_version: str) -> None:
        """Write session.json.

        The header holds no round count. The round directories are the
        record of how many rounds ran. A second copy of that number would
        disagree with the directories every time a round is part way
        through, and nothing would fail.
        """
        existing = read_json(self.path / "session.json") or {}
        created = existing.get("created") or now_iso()
        write_json(
            self.path / "session.json",
            {
                "asset": self.asset,
                "created": created,
                "model": model,
                "guide_version": guide_version,
            },
        )

    def feedback(self, index: int) -> dict | None:
        """Read the feedback of one round.

        The review interface writes this file. The loop only reads it.

        The file names the round that it belongs to. This function
        refuses the file when that name does not match the directory,
        because a mismatch means one of the two is wrong and neither is
        safe to guess from.
        """
        value = read_json(self.round_path(index) / "feedback.json")
        if not isinstance(value, dict):
            return None
        named = value.get("round")
        if named is not None and named != index:
            return None
        return value

    def critique(self, index: int, letter: str) -> dict | None:
        """Read the critique of one variant."""
        value = read_json(self.round_path(index) / f"variant-{letter}.critique.json")
        if not isinstance(value, dict):
            return None
        return value

    def svg(self, index: int, letter: str) -> str | None:
        """Read the SVG source of one variant."""
        path = self.round_path(index) / f"variant-{letter}.svg"
        try:
            return path.read_text(encoding="utf-8")
        except OSError:
            return None
