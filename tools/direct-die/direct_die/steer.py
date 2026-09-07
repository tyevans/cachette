"""The steer that a person gives to the loop.

The front end writes one file into a round directory: `feedback.json`. The
generation loop is its only reader. This module holds the shape of that file
and the walk over a session that turns it into one record.

The reader is total. It never raises. A malformed field becomes an empty
field, because the loop must keep running when a person edits the file by
hand. The front end refuses bad input at the moment of the write.

## The old field

The file held a `choice` field, which named one variant. Nothing writes that
field now. A file that holds `choice` and no `likes` reads as one like and a
one-entry order, so every session already on disk still opens. This rule
lives in one function.
"""

from __future__ import annotations

from dataclasses import dataclass

from .session import VARIANT_LETTERS


@dataclass(frozen=True)
class Feedback:
    """What a person said about one round."""

    likes: tuple[str, ...] = ()
    denies: tuple[str, ...] = ()
    order: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


# The record of a round that holds no feedback.
EMPTY = Feedback()


def _letters(value: object) -> tuple[str, ...]:
    """Take the variant letters out of a value, once each, in order."""
    if not isinstance(value, list):
        return ()
    found: list[str] = []
    for item in value:
        if isinstance(item, str) and item in VARIANT_LETTERS and item not in found:
            found.append(item)
    return tuple(found)


def _text(value: object) -> str | None:
    """Take a text field, or give None when it holds nothing."""
    if not isinstance(value, str):
        return None
    stripped = value.strip()
    return stripped or None


def read_feedback(value: dict | None) -> Feedback:
    """Read one feedback object into a record.

    The function never raises. It drops what it cannot read.
    """
    if not isinstance(value, dict):
        return EMPTY

    denies = _letters(value.get("denies"))

    if "likes" in value:
        likes = _letters(value.get("likes"))
    else:
        choice = value.get("choice")
        likes = (choice,) if isinstance(choice, str) and choice in VARIANT_LETTERS else ()

    # A refusal beats a like. The two lists must not hold the same letter, and
    # the front end refuses that write, but a hand-edited file can hold it.
    likes = tuple(letter for letter in likes if letter not in denies)

    ranked = [letter for letter in _letters(value.get("order")) if letter in likes]
    ranked.extend(letter for letter in likes if letter not in ranked)

    return Feedback(
        likes=likes,
        denies=denies,
        order=tuple(ranked),
        note=_text(value.get("note")),
        text=_text(value.get("text")),
    )
