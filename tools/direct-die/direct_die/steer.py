"""The steer that a person gives to the loop.

The front end writes one file into a round directory: `feedback.json`. The
generation loop is its only reader. This module holds the shape of that file
and the walk over a session that turns it into one record.

The reader is total. It never raises. A malformed field becomes an empty
field, because the loop must keep running when a person edits the file by
hand. The front end refuses bad input at the moment of the write.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from . import session as session_module
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

    likes = _letters(value.get("likes"))

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


@dataclass(frozen=True)
class Steer:
    """What every round below one index says about the next round."""

    parents: tuple[str, ...] = ()
    faults: dict[str, list[str]] = field(default_factory=dict)
    denied: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


def reference(index: int, letter: str) -> str:
    """Name one variant of one round, as the round metadata names it."""
    return f"{session_module.round_name(index)}/variant-{letter}"


def split_reference(value: str) -> tuple[int, str]:
    """Take the round index and the variant letter out of a reference.

    Raise ValueError when the value is not a reference.
    """
    head, _, tail = value.partition("/")
    if not head.startswith("round-") or not tail.startswith("variant-"):
        raise ValueError(f"not a variant reference: {value!r}")
    return int(head[len("round-") :]), tail[len("variant-") :]


def collect(store, index: int) -> Steer:
    """Read every round below one index, and give the steer for that index.

    A refusal holds for the whole session. A like holds for the next round
    only, so the round directly below the index decides the parents. The
    standing note is the newest note that any round holds.
    """
    if index <= 0:
        return Steer()

    below = [number for number in store.existing_rounds() if number < index]
    previous = index - 1

    denied: list[str] = []
    note: str | None = None
    text: str | None = None
    parents: tuple[str, ...] = ()

    for number in below:
        found = read_feedback(store.feedback(number))
        denied.extend(reference(number, letter) for letter in found.denies)
        if found.note is not None:
            note = found.note
        if number == previous:
            text = found.text
            parents = tuple(reference(number, letter) for letter in found.order)

    if not parents:
        best_key: tuple[int, int, str] | None = None
        best: str | None = None
        for number in below:
            for letter in VARIANT_LETTERS:
                critique = store.critique(number, letter)
                if not isinstance(critique, dict):
                    continue
                score = critique.get("score")
                if isinstance(score, bool) or not isinstance(score, int):
                    continue
                candidate = reference(number, letter)
                if candidate in denied:
                    continue
                key = (score, number, letter)
                if best_key is None or key >= best_key:
                    best_key = key
                    best = candidate
        parents = (best,) if best else ()

    faults: dict[str, list[str]] = {}
    for candidate in parents:
        number, letter = split_reference(candidate)
        critique = store.critique(number, letter) or {}
        listed = critique.get("faults")
        faults[candidate] = (
            [item for item in listed if isinstance(item, str)]
            if isinstance(listed, list)
            else []
        )

    return Steer(
        parents=parents,
        faults=faults,
        denied=tuple(denied),
        note=note,
        text=text,
    )
