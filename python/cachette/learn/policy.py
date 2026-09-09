"""A linear policy over the observation, and the file that stores it.

The policy is learner-side arithmetic. **It is not simulated state and it
never enters the world**, so it may hold a floating point number.[^1] The
engine stores integers, and the boundary is the action integer this module
returns.

# The features squash a wide range into a narrow one

The observation holds a tick beside a raw Q16.16 store total beside a count
of cells. The ranges differ by many orders of magnitude, so a linear policy
over the raw array would be driven by one field. The encoder therefore takes
the signed logarithm of each position, which keeps the sign and the order and
throws away the scale.

# The squash alone leaves a constant subspace, and a search finds it first

A squashed position is not centred. Most positions of the observation never
change over a run, so a weight over one of them can only add a fixed offset
to the score of an action row. A measurement over a reference sample found
that the constant part of the feature body carried several times the length
of the part that varies within an episode, and that every policy trained
under the plain squash chose almost one single action row. The commit that
added the normalizer holds the counts, because a count belongs to one moment
of the tree.

An evolution strategy finds that constant part first. A per-row offset pays
the same amount at every decision of every episode, while a state-dependent
weight has to correlate with a small wobble through the reward noise of a
whole episode. The legality mask then supplies what looks like situational
play.

**A stored normalizer removes the constant part.** The encoder subtracts a
per-position centre and divides by a per-position scale, and both come from
one fixed reference sample of the world. A position that never changes reads
exactly zero after the subtraction, so its weight reaches no score at all.

# The mask decides before the weights do

The policy scores every row of the action table, then it removes the rows the
engine says are illegal, then it takes the highest of what is left. Row zero
is the no-op and it is always legal, so the choice is never empty.[^2]

# References

[^1]: ADR-0002, state holds no floating point number, decision D1.
``docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md``
[^2]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables, decision D5.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Protocol

import numpy as np

from .layout import ObservationLayout

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from cachette._core import ActionSchema, ObservationSchema

    from .structured import StructuredPolicy

# The divisor that brings the signed logarithm into roughly one unit. A store
# total of a raw Q16.16 quantity reaches about twenty in the logarithm, so
# this puts the widest field near one.
FEATURE_SCALE = 20.0


# The smallest divisor a normalizer uses, declared once. Section 3 of the
# ``FeatureNormalizer`` docstring holds why it is this number.
FEATURE_SCALE_FLOOR = 0.10

# The keys a weight file stores the normalizer under. Both carry the
# ``normalizer_`` prefix, so neither collides with a key of the fit, of the
# action table or of the structured layout.
NORMALIZER_KEYS = ("normalizer_centre", "normalizer_scale")

# How many hexadecimal characters a normalizer digest holds. The digest names
# one derivation in a refusal message, and nothing reads it back as data.
DIGEST_WIDTH = 12


def squash(values: np.ndarray) -> np.ndarray:
    """Take the signed logarithm of every entry, scaled into roughly one unit.

    This is the transform the whole package reads the observation through, and
    **it is declared here once**. The encoder of one row, the encoder of a
    stack and the derivation of a normalizer all call it, so no caller applies
    a second copy of it.
    """
    held = np.asarray(values, dtype=np.float64)
    return np.sign(held) * np.log1p(np.abs(held)) / FEATURE_SCALE


@dataclass(frozen=True, eq=False)
class FeatureNormalizer:
    """The per-position centre and scale that standardize the squashed features.

    # 1. What it does

    The encoder squashes the observation, then this subtracts the centre and
    divides by the scale. **The trailing bias entry of one is never centred
    and never scaled**, because it is the one feature that is a constant on
    purpose.

    # 2. The centre applies to every position

    Nothing is dropped. A measurement over the reference sample found no
    plateau in the count of positions that ever move: the count climbed with
    every seed added, so a position that stayed still over the sample may move
    on the next world. Dropping a position is therefore unsafe.

    Centring one is safe, and it is the part that removes the constant
    subspace. **A position that never changes reads exactly zero after the
    subtraction**, so its weight becomes inert rather than a per-row bias.

    # 3. The scale takes a floor

    The scale of a position is its standard deviation over the reference
    sample, or the floor when the deviation is under it.

    The floor exists because a per-position scale divides the residual mean
    shift as well as the spread. A small floor therefore amplifies the offset
    that the centring just removed, and the amplified offset returns as the
    per-row bias this transform exists to remove.

    The floor caps that amplification at ten times, and it holds most of the
    positions that move at unit variance. **The measured optimum is flat over
    a wide band around it**, so the value is not delicate. The commit that
    added this type holds the measurement and the band.

    # 4. A stored policy is meaningless without its normalizer

    A weight file writes both arrays beside the weights. A reader refuses a
    file whose normalizer is not the normalizer of the world it is asked to
    play, and it names the digest of each side.

    **A file that stores no normalizer gives a policy that holds none, and a
    policy that holds none reads the plain squash.** That is what the identity
    of this type computes, so a policy published before this type existed
    still runs.
    """

    centre: np.ndarray
    scale: np.ndarray

    def __post_init__(self) -> None:
        """Hold both arrays as one flat ``float64`` each, and refuse a bad one."""
        object.__setattr__(self, "centre", np.asarray(self.centre, dtype=np.float64))
        object.__setattr__(self, "scale", np.asarray(self.scale, dtype=np.float64))
        if self.centre.ndim != 1 or self.scale.ndim != 1:
            message = (
                "a normalizer holds one centre and one scale for each position, "
                f"so each array has one axis, and these hold {self.centre.ndim} "
                f"and {self.scale.ndim}"
            )
            raise PolicyFitError(message)
        if self.centre.shape != self.scale.shape:
            message = (
                f"a normalizer holds {self.centre.size} centres and "
                f"{self.scale.size} scales. The two are one for each position, "
                "so they hold the same count."
            )
            raise PolicyFitError(message)
        if self.scale.size and float(self.scale.min()) <= 0.0:
            message = (
                "a normalizer divides by its scale, so every scale is above "
                f"zero, and the smallest here is {float(self.scale.min())}"
            )
            raise PolicyFitError(message)

    @classmethod
    def identity(cls, length: int) -> FeatureNormalizer:
        """Return the normalizer that changes nothing.

        The centre is zero and the scale is one, so the standardized features
        are the squashed features. A policy that holds no normalizer computes
        the same thing, and a caller that wants to state the transform rather
        than leave it absent asks for this.
        """
        return cls(np.zeros(int(length)), np.ones(int(length)))

    @classmethod
    def of_observations(cls, observations: np.ndarray) -> FeatureNormalizer:
        """Derive the normalizer from a stack of raw observation rows.

        The rows are the reference sample. This squashes them, takes the mean
        of each position as the centre, and takes the standard deviation of
        each position under the floor as the scale.
        """
        rows = np.asarray(observations)
        if rows.ndim != 2 or rows.shape[0] < 1:
            message = (
                "a normalizer comes from a stack of observation rows, so the "
                f"sample has two axes and at least one row, and this holds "
                f"shape {rows.shape}"
            )
            raise PolicyFitError(message)
        squashed = squash(rows)
        return cls(
            squashed.mean(axis=0), np.maximum(squashed.std(axis=0), FEATURE_SCALE_FLOOR)
        )

    @property
    def length(self) -> int:
        """How many observation positions this standardizes."""
        return int(self.centre.size)

    @property
    def is_identity(self) -> bool:
        """Say whether this changes nothing at all."""
        return bool(
            np.array_equal(self.centre, np.zeros(self.length))
            and np.array_equal(self.scale, np.ones(self.length))
        )

    def apply(self, squashed: np.ndarray) -> np.ndarray:
        """Standardize the squashed body of one row or of a stack of rows.

        The argument holds no bias entry. The caller appends that after this,
        because the bias is never centred and never scaled.
        """
        held = np.asarray(squashed, dtype=np.float64)
        if held.shape[-1] != self.length:
            message = (
                f"this normalizer standardizes {self.length} positions and was "
                f"given {held.shape[-1]}. A normalizer is a function of one "
                "world, so train a policy against this world."
            )
            raise PolicyFitError(message)
        return np.asarray((held - self.centre) / self.scale, dtype=np.float64)

    def digest(self) -> str:
        """Return a short digest of both arrays, for a refusal message.

        The digest comes from the arrays every time a caller asks for it.
        **Nothing stores it**, because a stored digest beside the arrays it
        describes is a second declaration of one thing, and nothing would fail
        when the two disagreed.[^1]

        References
        ----------
        [^1]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
        """
        held = hashlib.sha256()
        held.update(np.ascontiguousarray(self.centre).tobytes())
        held.update(np.ascontiguousarray(self.scale).tobytes())
        return held.hexdigest()[:DIGEST_WIDTH]

    def describe(self) -> str:
        """Return one line that names the length and the digest."""
        kind = "identity" if self.is_identity else self.digest()
        return f"{self.length} positions, {kind}"

    def as_arrays(self) -> dict[str, np.ndarray]:
        """Return the entries a weight file stores this under."""
        return {"normalizer_centre": self.centre, "normalizer_scale": self.scale}

    @classmethod
    def read(cls, stored: Mapping[str, np.ndarray]) -> FeatureNormalizer | None:
        """Return the normalizer a weight file holds, or nothing when it holds none.

        Raises ``PolicyFitError`` when a file names one of the two arrays and
        not the other. A normalizer is written in one piece, so such a file is
        not consistent with itself.
        """
        present = [key for key in NORMALIZER_KEYS if key in stored]
        if not present:
            return None
        if len(present) != len(NORMALIZER_KEYS):
            missing = [key for key in NORMALIZER_KEYS if key not in stored]
            message = (
                "the stored policy states part of a feature normalizer. It "
                f"names {', '.join(present)} and it does not name "
                f"{', '.join(missing)}. A normalizer is written in one piece, "
                "so this file is not consistent with itself."
            )
            raise PolicyFitError(message)
        return cls(
            np.asarray(stored["normalizer_centre"], dtype=np.float64).reshape(-1),
            np.asarray(stored["normalizer_scale"], dtype=np.float64).reshape(-1),
        )


def encode(
    observation: np.ndarray, normalizer: FeatureNormalizer | None = None
) -> np.ndarray:
    """Turn one observation array into the feature vector of the policy.

    The result holds one entry for each position of the observation, and one
    trailing entry of one for the bias.

    A normalizer standardizes the body and leaves the bias entry alone. A
    caller that passes none gets the plain squash, which is what the identity
    normalizer computes.
    """
    squashed = squash(observation)
    if normalizer is not None:
        squashed = normalizer.apply(squashed)
    return np.concatenate([squashed, np.ones(1)])


def encode_many(
    observations: np.ndarray, normalizer: FeatureNormalizer | None = None
) -> np.ndarray:
    """Encode a stack of observations, one for each row.

    A normalizer standardizes the body of every row and leaves the bias entry
    of every row alone.
    """
    squashed = squash(observations)
    if normalizer is not None:
        squashed = normalizer.apply(squashed)
    ones = np.ones((squashed.shape[0], 1))
    return np.concatenate([squashed, ones], axis=1)


def masked_choices(scores: np.ndarray, masks: np.ndarray) -> list[int]:
    """Return the highest-scoring legal row of each row of a score stack.

    **This is the one declaration of how a mask meets a score.** Both policy
    kinds choose this way, and the instrument that reads an unmasked
    preference reads the same score matrix. A second copy of the masking
    would be one rule stored twice, with nothing that fails when the copies
    disagree.[^1]

    References
    ----------
    [^1]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    legal = np.where(masks > 0, scores, -np.inf)
    return [int(value) for value in np.argmax(legal, axis=1)]


def preferred_rows(scores: np.ndarray) -> list[int]:
    """Return the highest-scoring row of each row of a score stack, unmasked.

    **The engine's legality answer is out of this.** A policy that holds one
    fixed preference order over the action rows still emits many different
    actions, because the mask removes the rows it cannot take. The unmasked
    argmax is what separates a preference order from a policy: it changes
    inside an episode only when the observation moved the scores.[^1]

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    return [int(value) for value in np.argmax(scores, axis=1)]


class PolicyFitError(ValueError):
    """A stored policy does not fit the world a caller asked it to play.

    A weight file is a function of one observation layout and one action
    layout. Both layouts are functions of the world parameters, so a file
    written against one world states nothing about another.[^1]

    **The lengths alone do not separate two worlds.** The observation length
    counts the cells of the block lattice, and a block is a fixed number of
    tiles on a side. Every world from one block to two blocks on each axis
    therefore holds the same cell count and the same observation length. A
    policy trained on the smallest of those loads on the largest, reads an
    array of the length it expects, and plays a world it never saw. Nothing
    raises, because nothing has a shape to disagree about.

    The fit therefore carries the world extent and the faction count beside
    the two lengths and the two versions.

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables, decision D2.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """


# The keys a weight file stores the action table under. Every one carries the
# ``action_`` prefix, so none of them collides with a key of the fit or of the
# structured layout.
ACTION_TABLE_KEYS = (
    "action_verb_names",
    "action_verb_first",
    "action_verb_rows",
    "action_verb_position_counts",
    "action_position_candidates",
    "action_position_bounds",
    "action_position_strides",
)


@dataclass(frozen=True)
class VerbPosition:
    """One argument position of one verb, as the engine published it.

    The candidate names what the position chooses. The bound is how many
    choices it holds, and the stride is what one step of it adds to the
    action integer.[^1]

    References
    ----------
    [^1]: ADR-0176, an action integer is a mixed radix over the argument
    positions each verb declares, decision D1.
    ``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
    """

    candidate: str
    bound: int
    stride: int


@dataclass(frozen=True)
class VerbBlock:
    """One verb of the action table, and the block of rows it holds.

    The block is contiguous, and the action integer of a row is a mixed radix
    over the positions of the verb.[^1] A verb the engine resolves by itself
    declares no position and holds one row.

    References
    ----------
    [^1]: ADR-0176, an action integer is a mixed radix over the argument
    positions each verb declares, decision D1.
    ``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
    """

    name: str
    first: int
    rows: int
    positions: tuple[VerbPosition, ...]

    def coordinates(self, row: int) -> tuple[int, ...]:
        """Give the candidate coordinates one row of this block names.

        The coordinates are the mixed-radix digits of the row inside the
        block. **They are the identity of the row, and the row index is
        not.**[^1] A change to the table moves the index and leaves the
        coordinates where they were.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D2.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        offset = row - self.first
        return tuple(
            (offset // position.stride) % position.bound for position in self.positions
        )

    def row_of(self, coordinates: Sequence[int]) -> int:
        """Give the row of this block that names the leading coordinates.

        A coordinate the caller does not give reads as zero, which is the
        first value of the candidate list of that position.
        """
        row = self.first
        for index, position in enumerate(self.positions):
            if index < len(coordinates):
                row += int(coordinates[index]) * position.stride
        return row

    def holds(self, coordinates: Sequence[int]) -> bool:
        """Say whether every coordinate the caller gives is inside its bound."""
        return all(
            int(value) < self.positions[index].bound
            for index, value in enumerate(coordinates)
            if index < len(self.positions)
        )

    def candidates(self) -> tuple[str, ...]:
        """Name the candidate of each position, in the order the verb declares."""
        return tuple(position.candidate for position in self.positions)

    def describe(self) -> str:
        """Return one line that names the verb and its positions."""
        if not self.positions:
            return f"{self.name}()"
        arguments = ", ".join(
            f"{position.candidate}<{position.bound}>" for position in self.positions
        )
        return f"{self.name}({arguments})"


@dataclass(frozen=True)
class ActionTable:
    """The action table one weight file was trained against.

    The engine publishes this table beside every world, and this type is a
    copy of one publication of it.[^1] **Nothing in this package states a
    verb, a bound or a stride as a literal.** A hand-written verb list would
    be a second declaration of what the engine publishes, and nothing fails
    when two declarations disagree.[^2] [^3]

    A stored table lets a reader place a row of an older file by what the row
    means rather than by where it sat. The meaning is the verb and the
    candidate coordinates, and both survive a change that renumbers the
    table.[^3]

    References
    ----------
    [^1]: ADR-0176, an action integer is a mixed radix over the argument
    positions each verb declares, decision D1.
    ``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``

    [^2]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``

    [^3]: ADR-0200, a stored policy names each row of the action table by its
    verb and its candidate coordinates, decisions D1 and D2.
    ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
    """

    verbs: tuple[VerbBlock, ...]

    @classmethod
    def of_schema(cls, schema: ActionSchema) -> ActionTable:
        """Copy the table out of the schema the engine publishes."""
        blocks = []
        for verb in schema["verbs"]:
            positions = tuple(
                VerbPosition(
                    candidate=str(position["candidate"]),
                    bound=int(position["bound"]),
                    stride=int(position["stride"]),
                )
                for position in verb["positions"]
            )
            blocks.append(
                VerbBlock(
                    name=str(verb["name"]),
                    first=int(verb["first"]),
                    rows=int(verb["rows"]),
                    positions=positions,
                )
            )
        return cls(verbs=tuple(blocks))

    @property
    def length(self) -> int:
        """How many rows the whole table holds."""
        return int(sum(verb.rows for verb in self.verbs))

    def named(self, name: str) -> VerbBlock | None:
        """Give the block of one verb, by name, or nothing when it has none.

        **The match is by name and never by index.** An index is a position in
        a layout, and a change to one verb moves the index of every verb after
        it.
        """
        for verb in self.verbs:
            if verb.name == name:
                return verb
        return None

    def as_meta(self) -> dict[str, list[object]]:
        """Return the table as the entries a weight file stores.

        The positions of every verb lie in three flat lists, and one list of
        counts says how many belong to each verb. A ragged table therefore
        stores as arrays of one shape each, which is what the archive holds.
        """
        names: list[object] = []
        first: list[object] = []
        rows: list[object] = []
        counts: list[object] = []
        candidates: list[object] = []
        bounds: list[object] = []
        strides: list[object] = []
        for verb in self.verbs:
            names.append(verb.name)
            first.append(verb.first)
            rows.append(verb.rows)
            counts.append(len(verb.positions))
            for position in verb.positions:
                candidates.append(position.candidate)
                bounds.append(position.bound)
                strides.append(position.stride)
        return {
            "action_verb_names": names,
            "action_verb_first": first,
            "action_verb_rows": rows,
            "action_verb_position_counts": counts,
            "action_position_candidates": candidates,
            "action_position_bounds": bounds,
            "action_position_strides": strides,
        }

    @classmethod
    def read(cls, meta: Mapping[str, object]) -> ActionTable | None:
        """Return the table a weight file states, or nothing when it states none.

        A file written before this package stored a table names none of the
        keys. Such a file has no identity for any of its rows, so the caller
        falls back to the version integer of the layout.

        Raises ``PolicyFitError`` when a file names some of the keys and not
        others, or when the counts do not add up to the positions. Both mean
        the writer of the file was inconsistent, and neither can be read.
        """
        present = [key for key in ACTION_TABLE_KEYS if key in meta]
        if not present:
            return None
        if len(present) != len(ACTION_TABLE_KEYS):
            missing = [key for key in ACTION_TABLE_KEYS if key not in meta]
            message = (
                "the stored policy states part of an action table. It names "
                f"{', '.join(present)} and it does not name "
                f"{', '.join(missing)}. A table is written in one piece, so "
                "this file is not consistent with itself."
            )
            raise PolicyFitError(message)
        names = _as_names(meta["action_verb_names"])
        first = _as_whole(meta["action_verb_first"])
        rows = _as_whole(meta["action_verb_rows"])
        counts = _as_whole(meta["action_verb_position_counts"])
        candidates = _as_names(meta["action_position_candidates"])
        bounds = _as_whole(meta["action_position_bounds"])
        strides = _as_whole(meta["action_position_strides"])
        lengths = {len(names), len(first), len(rows), len(counts)}
        position_lengths = {len(candidates), len(bounds), len(strides)}
        wanted = sum(counts)
        if (
            len(lengths) != 1
            or len(position_lengths) != 1
            or wanted not in position_lengths
        ):
            message = (
                "the stored policy states an action table whose lists do not "
                f"agree. It names {len(names)} verb names, {len(first)} first "
                f"rows, {len(rows)} row counts and {len(counts)} position "
                f"counts, which sum to {wanted}, against {len(candidates)} "
                f"candidates, {len(bounds)} bounds and {len(strides)} strides. "
                "The file is not consistent with itself."
            )
            raise PolicyFitError(message)
        blocks = []
        walked = 0
        for index, name in enumerate(names):
            held = counts[index]
            positions = tuple(
                VerbPosition(
                    candidate=candidates[walked + offset],
                    bound=bounds[walked + offset],
                    stride=strides[walked + offset],
                )
                for offset in range(held)
            )
            walked += held
            blocks.append(
                VerbBlock(
                    name=name,
                    first=first[index],
                    rows=rows[index],
                    positions=positions,
                )
            )
        return cls(verbs=tuple(blocks))

    def rebuild_from(self, stored: ActionTable) -> ActionRebuild:
        """Say where each row of this table takes its weight from.

        This table is the one the engine publishes now. The argument is the
        one a weight file states. The result names, for each row of this
        table, the stored row that holds the same identity, or the stored
        block whose mean stands in for a row the file never held.[^1]

        Raises ``PolicyFitError`` when a verb of both tables declares a
        position sequence that this table does not extend. A reordering, a
        renamed candidate and a removed position are all refusals, because
        the verb no longer means what the file meant by it.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decisions D3 and D5.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        length = self.length
        source = [-1] * length
        average_first = [0] * length
        average_rows = [0] * length
        kept = 0
        donated = 0
        for verb in self.verbs:
            held = stored.named(verb.name)
            if held is None:
                continue
            _refuse_narrowed(verb, held)
            for row in range(verb.first, verb.first + verb.rows):
                coordinates = verb.coordinates(row)
                leading = coordinates[: len(held.positions)]
                if held.holds(leading):
                    source[row] = held.row_of(leading)
                    if all(value == 0 for value in coordinates[len(held.positions) :]):
                        kept += 1
                    else:
                        donated += 1
                else:
                    average_first[row] = held.first
                    average_rows[row] = held.rows
                    donated += 1
        return ActionRebuild(
            length=length,
            stored_length=stored.length,
            source=tuple(source),
            average_first=tuple(average_first),
            average_rows=tuple(average_rows),
            kept=kept,
            donated=donated,
            dropped=tuple(
                verb.name for verb in stored.verbs if self.named(verb.name) is None
            ),
            added=tuple(
                verb.name for verb in self.verbs if stored.named(verb.name) is None
            ),
        )


def _as_entries(value: object) -> list[object]:
    """Read one entry of a weight file as a list, whatever shape it holds.

    An archive of one entry reads back as a scalar rather than as a list of
    one, so a table of one verb would otherwise refuse to be read.
    """
    if isinstance(value, (list, tuple)):
        return list(value)
    return [value]


def _as_whole(value: object) -> list[int]:
    """Read one entry of a weight file as a list of whole numbers."""
    return [int(entry) for entry in _as_entries(value)]  # type: ignore[call-overload]


def _as_names(value: object) -> list[str]:
    """Read one entry of a weight file as a list of names."""
    return [str(entry) for entry in _as_entries(value)]


def _refuse_narrowed(current: VerbBlock, stored: VerbBlock) -> None:
    """Refuse a verb whose stored positions the current verb does not extend.

    The current sequence must begin with the stored one. A position added
    after the stored ones narrows what the verb already meant, so every stored
    identity keeps a home. Any other change moves what a coordinate names, and
    no mapping recovers it.[^1]

    References
    ----------
    [^1]: ADR-0200, a stored policy names each row of the action table by its
    verb and its candidate coordinates, decision D3.
    ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
    """
    held = stored.candidates()
    now = current.candidates()
    if now[: len(held)] == held:
        return
    message = (
        f"the stored policy states the verb {stored.name!r} as "
        f"{stored.describe()} and this world states it as "
        f"{current.describe()}. The current verb does not begin with the "
        "stored argument positions, so a row of the file names something "
        "this world does not hold. Train a policy against this world."
    )
    raise PolicyFitError(message)


@dataclass(frozen=True)
class ActionRebuild:
    """Where each row of the current action table takes its weight from.

    A readout holds one weight vector for each row of the table, and this
    states how to carry one readout onto another table.[^1]

    ``source`` names the stored row of the same identity, or minus one when
    the file held no row of that identity. ``average_first`` and
    ``average_rows`` name the stored block whose mean stands in for such a
    row, and both read zero when nothing stands in and the row starts at zero.

    ``stored_length`` is how many rows the table of the file held. A readout
    of another row count than that was not written against the table the file
    states, so the rebuild would read rows that mean nothing.

    References
    ----------
    [^1]: ADR-0200, a stored policy names each row of the action table by its
    verb and its candidate coordinates, decisions D3 and D5.
    ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
    """

    length: int
    stored_length: int
    source: tuple[int, ...]
    average_first: tuple[int, ...]
    average_rows: tuple[int, ...]
    kept: int
    donated: int
    dropped: tuple[str, ...]
    added: tuple[str, ...]

    @property
    def moved(self) -> bool:
        """Say whether this carries a readout onto a table of another shape.

        A rebuild that keeps every row where it was is the identity, and a
        caller that reads this reports a load rather than a rebuild.
        """
        return bool(
            self.donated
            or self.dropped
            or self.added
            or self.kept != self.length
            or any(row != index for index, row in enumerate(self.source))
        )

    def apply(self, readout: np.ndarray) -> np.ndarray:
        """Carry one stored readout onto the current table.

        The result holds one row for each row of the current table. A row of
        a verb the file does not name stays at zero, because the file states
        nothing about it.

        Raises ``PolicyFitError`` when the readout holds another row count
        than the table of the file. **The row count is declared twice in one
        file**, once by the readout and once by the table, and a check that
        fails is what a second declaration site needs.[^1]

        References
        ----------
        [^1]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
        """
        held = np.asarray(readout, dtype=np.float64)
        if held.ndim != 2:
            message = (
                f"a readout holds one row for each action row, so it has two "
                f"axes, and this holds {held.ndim}"
            )
            raise PolicyFitError(message)
        if held.shape[0] != self.stored_length:
            message = (
                f"the stored policy holds a readout of {held.shape[0]} rows "
                f"and states an action table of {self.stored_length} rows. "
                "The file is not consistent with itself."
            )
            raise PolicyFitError(message)
        rebuilt = np.zeros((self.length, held.shape[1]), dtype=np.float64)
        for target in range(self.length):
            row = self.source[target]
            if row >= 0:
                rebuilt[target] = held[row]
                continue
            rows = self.average_rows[target]
            if rows > 0:
                first = self.average_first[target]
                rebuilt[target] = held[first : first + rows].mean(axis=0)
        return rebuilt

    def describe(self) -> str:
        """Return one line that says what this rebuild moved.

        A resumed run prints this, because a run that rebuilt a checkpoint
        and a run that loaded one are different states, and a reader who
        cannot tell them apart cannot read a score.
        """
        parts = [
            f"{self.kept} rows kept",
            f"{self.donated} rows donated",
        ]
        if self.dropped:
            parts.append(f"dropped {', '.join(self.dropped)}")
        if self.added:
            parts.append(f"added {', '.join(self.added)}")
        return f"rebuilt the readout onto {self.length} rows: " + ", ".join(parts)


@dataclass(frozen=True)
class PolicyFit:
    """What one weight file was trained against.

    Every entry is a function of the world parameters and never of the
    population.[^1] A file states its fit, and a caller that plays the file
    states the fit of its own world. The two must agree.

    The two version entries come from the engine schemas and never from a
    constant in this package. A version written by hand is a second
    declaration of a number the engine owns, and nothing fails when the two
    disagree.[^2]

    **The action table decides the action half of the fit, and the action
    version integer does not.** A fit that holds a table compares verb by
    verb, so a change that renumbers the table is not a refusal.[^3] A fit
    that holds no table falls back to the version integer and the row count,
    which is what a file written before this type existed can be read by.

    The table is out of the comparison of two fits, because two tables of one
    world are always the same table and a fit that a test writes by hand
    states none. The check reads the table separately.

    **The feature normalizer is the same shape of entry as the table.** It is
    a function of the world and of the observation version, and no integer of
    the fit separates two of them, so the fit carries it and the check
    compares the digests. A side that states none falls back to what the
    integers say, which is what a file written before the normalizer existed
    can be read by.

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables, decision D2.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

    [^2]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``

    [^3]: ADR-0200, a stored policy names each row of the action table by its
    verb and its candidate coordinates, decision D4.
    ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
    """

    observation_version: int
    action_version: int
    observation_length: int
    action_length: int
    width: int
    height: int
    faction_count: int
    action_table: ActionTable | None = field(default=None, compare=False)
    normalizer: FeatureNormalizer | None = field(default=None, compare=False)

    # The keys a weight file stores the fit under. The names are the ones
    # the trainer already wrote, so a file written before this type existed
    # still reads back as a fit.
    KEYS = (
        "observation_version",
        "action_version",
        "observation_length",
        "action_length",
        "width",
        "height",
        "faction_count",
    )

    # The entries of the fit that the action table decides once both sides
    # hold one. They stay in the file and in the message, and they stop
    # deciding a refusal.
    ACTION_KEYS = ("action_version", "action_length")

    @classmethod
    def of_env(
        cls, env: EnvLike, normalizer: FeatureNormalizer | None = None
    ) -> PolicyFit:
        """Return the fit of the world one environment builds.

        **The normalizer is an argument and never a derivation here.** It
        comes from a reference sample of played episodes, and a fit is built
        many times in one run. A caller derives it once and passes the same
        one every time.
        """
        config = env.config
        return cls(
            observation_version=int(env.observation_version),
            action_version=int(env.action_version),
            observation_length=int(env.observation_length),
            action_length=int(env.action_length),
            width=int(config.width),
            height=int(config.height),
            faction_count=int(config.faction_count),
            action_table=env.action_table,
            normalizer=normalizer,
        )

    @classmethod
    def of_world(
        cls, world: WorldLike, normalizer: FeatureNormalizer | None = None
    ) -> PolicyFit:
        """Return the fit of one world, read from the schemas it publishes.

        A caller that holds a world and no environment reads the fit here.
        The demonstration is such a caller: it builds a world of its own and
        seats a stored policy on one faction of it.

        **The two versions and the two lengths come from the engine
        schemas**, in the way they do for an environment. Nothing here holds
        a constant, so a change in the engine reaches both callers at once.
        """
        observation = world.observation_schema()
        action = world.action_schema()
        return cls(
            observation_version=int(observation["version"]),
            action_version=int(action["version"]),
            observation_length=int(observation["length"]),
            action_length=int(action["length"]),
            width=int(world.width),
            height=int(world.height),
            faction_count=int(world.faction_count),
            action_table=ActionTable.of_schema(action),
            normalizer=normalizer,
        )

    @classmethod
    def read(
        cls,
        meta: Mapping[str, object],
        normalizer: FeatureNormalizer | None = None,
    ) -> PolicyFit | None:
        """Return the fit a weight file states, or nothing when it states none.

        A file written before this package stored a fit names some of the
        keys and not others. Such a file cannot be placed, so this returns
        nothing and the caller refuses it.

        A file that also states an action table reads it back here, and a
        file that states none reads back a fit whose table is nothing.

        **The normalizer is an argument and not an entry of the reported
        keys.** It is two arrays and the reported keys are what a caller
        prints, so the reader of the file passes it in.

        Raises ``PolicyFitError`` when the row count of the stated table is
        not the row count the fit states. **The file declares that number
        twice**, and a second declaration site needs a check that fails when
        the copies disagree.[^1]

        References
        ----------
        [^1]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
        """
        values: dict[str, int] = {}
        for key in cls.KEYS:
            value = meta.get(key)
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                return None
            values[key] = int(value)
        table = ActionTable.read(meta)
        if table is not None and table.length != values["action_length"]:
            message = (
                f"the stored policy states an action table of {table.length} "
                f"rows and an action length of {values['action_length']}. The "
                "file is not consistent with itself."
            )
            raise PolicyFitError(message)
        return cls(**values, action_table=table, normalizer=normalizer)

    def as_meta(self) -> dict[str, object]:
        """Return the fit as the entries a weight file stores.

        A fit that holds an action table writes it beside the seven integers.
        A fit that holds none writes the seven integers alone, which is what
        every file held before this table existed.
        """
        entries: dict[str, object] = {key: int(getattr(self, key)) for key in self.KEYS}
        if self.action_table is not None:
            entries.update(self.action_table.as_meta())
        return entries

    def describe(self) -> str:
        """Return one line that names every entry of the fit."""
        return ", ".join(f"{key}={getattr(self, key)}" for key in self.KEYS)

    def rebuild_for(self, wanted: PolicyFit) -> ActionRebuild | None:
        """Say how to carry this file's readout onto the table a caller wants.

        Returns nothing when either side states no action table. The caller
        then places the rows by index, which is all the version integer
        supports.

        Raises ``PolicyFitError`` when a verb of both tables changed what its
        coordinates name.
        """
        if self.action_table is None or wanted.action_table is None:
            return None
        return wanted.action_table.rebuild_from(self.action_table)

    def check(self, wanted: PolicyFit, path: Path | None = None) -> None:
        """Refuse when this fit is not the fit a caller asked for.

        **The action table decides the action half when both sides state
        one.** A change to one verb moves every row above it, so the version
        integer and the row count of the table then say only that the table
        moved. The rebuild says which rows changed meaning, and it raises for
        those.[^1]

        **A normalizer that differs is a refusal.** Every weight of the file
        scores a standardized feature, so a file played under another
        standardization scores a different quantity at every position. The
        message names the digest of each side.

        Raises ``PolicyFitError`` naming both sides, so a reader sees which
        entry differs without opening the file.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D4.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        by_table = self.action_table is not None and wanted.action_table is not None
        keys = [key for key in self.KEYS if not (by_table and key in self.ACTION_KEYS)]
        differ = [
            f"{key}: the file says {getattr(self, key)} "
            f"and the world says {getattr(wanted, key)}"
            for key in keys
            if getattr(self, key) != getattr(wanted, key)
        ]
        differ.extend(self._normalizer_difference(wanted))
        if not differ:
            return
        where = f" at {path}" if path is not None else ""
        message = (
            f"the stored policy{where} does not fit this world. "
            + "; ".join(differ)
            + f". The file states {self.describe()}. "
            f"The world states {wanted.describe()}. "
            "Train a policy against this world, or play the policy on the "
            "world it was trained against."
        )
        raise PolicyFitError(message)

    def _normalizer_difference(self, wanted: PolicyFit) -> list[str]:
        """Say how the two normalizers differ, or say nothing when they agree.

        **The comparison happens only when both sides state a normalizer.** A
        file written before the normalizer existed states none, and the reader
        accepts such a file and plays it through the plain squash. A world
        that states none asks for whatever the file holds, which is what a
        reader that only reports a file needs.

        The digest names each side. Two arrays of a few thousand entries do
        not belong in a message, and a digest separates two derivations.
        """
        held = self.normalizer
        asked = wanted.normalizer
        if held is None or asked is None:
            return []
        if np.array_equal(held.centre, asked.centre) and np.array_equal(
            held.scale, asked.scale
        ):
            return []
        return [
            f"normalizer: the file says {held.describe()} "
            f"and the world says {asked.describe()}"
        ]


class EnvLike(Protocol):
    """What a fit reads from an environment.

    The fit needs the two schema versions, the two lengths, the action table
    and the world parameters. Naming them here keeps this module free of an
    import from the environment, which imports this one.
    """

    observation_version: int
    action_version: int
    observation_length: int
    action_length: int
    action_table: ActionTable

    @property
    def config(self) -> ConfigLike:
        """The configuration the environment runs."""


class ConfigLike(Protocol):
    """The world parameters a fit reads from an environment configuration.

    **The three are read-only, because the fit only reads them.** A protocol
    that declares a plain attribute asks for one that can be written, and a
    frozen configuration cannot answer that. Declaring what this actually
    needs lets a frozen dataclass satisfy it.
    """

    @property
    def width(self) -> int:
        """How many tiles the world holds across."""

    @property
    def height(self) -> int:
        """How many tiles the world holds down."""

    @property
    def faction_count(self) -> int:
        """How many factions play the world."""


class WorldLike(Protocol):
    """What a fit reads from a world it did not build.

    The two schemas state the layout, and the three parameters state the
    world. Naming them here keeps this module free of an import from the
    engine binding, which the type checker reads from a stub.

    **The three parameters are read-only, because the fit only reads them.**
    A protocol that declares a plain attribute asks for one that can be
    written, and the world publishes them as properties.
    """

    @property
    def width(self) -> int:
        """How many tiles the world holds across."""

    @property
    def height(self) -> int:
        """How many tiles the world holds down."""

    @property
    def faction_count(self) -> int:
        """How many factions play the world."""

    def observation_schema(self) -> ObservationSchema:
        """Give back the layout of the observation array."""

    def action_schema(self) -> ActionSchema:
        """Give back the layout of the action table."""


class Policy(Protocol):
    """What a caller needs of a policy to play it on a batch of worlds.

    The training loop and the holdout measurement both score a whole batch at
    once, so the one thing either of them asks of a policy is a choice for
    every row. The linear policy and the two baselines all answer it, and a
    signature that named one of them would refuse the other two.
    """

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action integer for each row of a stack of observations."""


class LinearPolicy:
    """One weight matrix over the features, and one score for each action."""

    # **A positive scaling of the weights leaves every choice where it was.**
    # One matrix scores every action row as a weighted sum over one feature
    # vector, so scaling the matrix scales every score by one factor. The row
    # that scores highest stays the row that scores highest. The bias is a
    # trailing feature of value one, and its weight scales with every other
    # weight, so it carries no exception.
    #
    # The search reads this and holds the centre of this kind at unit length.
    CHOICE_SURVIVES_SCALING = True

    def __init__(
        self, weights: np.ndarray, normalizer: FeatureNormalizer | None = None
    ) -> None:
        """Take the weight matrix and the normalizer it reads features through.

        The shape of the matrix is (actions, features).

        A policy that holds no normalizer reads the plain squash. That is what
        a file written before the normalizer existed loads as, and it is what
        a caller who scores a hand-written matrix asks for.
        """
        self.weights = np.asarray(weights, dtype=np.float64)
        self.normalizer = normalizer

    @classmethod
    def zeros(
        cls,
        action_length: int,
        observation_length: int,
        normalizer: FeatureNormalizer | None = None,
    ) -> LinearPolicy:
        """Build the untrained policy. Every score is zero.

        A zero policy takes the first legal row of the table at every
        decision, which is the no-op. It is the baseline every trained model
        is measured against.
        """
        return cls(np.zeros((action_length, observation_length + 1)), normalizer)

    @property
    def shape(self) -> tuple[int, int]:
        """The shape of the weight matrix."""
        actions, features = self.weights.shape
        return actions, features

    def choose(self, observation: np.ndarray, mask: np.ndarray) -> int:
        """Return the action integer of the highest-scoring legal row."""
        scores = self.weights @ encode(observation, self.normalizer)
        return masked_choices(scores[None, :], np.asarray(mask)[None, :])[0]

    def scores_many(self, observations: np.ndarray) -> np.ndarray:
        """Return one unmasked score for each action row of each observation.

        **This is the one place the linear score is computed for a stack.**
        The choice masks the result, and the instrument that reads the
        unmasked preference reads the same matrix.
        """
        return encode_many(observations, self.normalizer) @ self.weights.T

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action for each row of a stack of observations."""
        return masked_choices(self.scores_many(observations), masks)

    def flat(self) -> np.ndarray:
        """Return every trainable weight as one vector."""
        return self.weights.reshape(-1)

    def rebuild(self, flat: np.ndarray) -> LinearPolicy:
        """Return a policy of this shape with the given weights.

        **The normalizer travels with the rebuilt policy.** The search
        rebuilds every candidate of every generation through the shell, so a
        normalizer that stopped here would reach no candidate the run scored.
        """
        return LinearPolicy(flat.reshape(self.weights.shape), self.normalizer)

    def save(self, path: Path, meta: Mapping[str, object]) -> None:
        """Write the weights and what they were trained against.

        The two schema versions go into the file, and so does the action
        table when the caller states one. **The table is what places a row of
        this file in a later table**, because a row is named by its verb and
        its candidate coordinates and not by its index.[^1]

        **The normalizer goes into the file beside the weights.** A weight of
        this file scores a standardized feature, and nothing outside the file
        says which standardization that was, so a file without its normalizer
        states nothing a reader can play.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D1.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        normalizer = self.normalizer.as_arrays() if self.normalizer is not None else {}
        # numpy declares ``allow_pickle`` as a keyword before its own
        # ``**kwds``, so a mapping keyed on ``str`` can never unpack cleanly.
        np.savez(
            path,
            weights=self.weights,
            kind=np.array("linear"),
            **normalizer,  # type: ignore[arg-type]
            **{key: np.array(value) for key, value in meta.items()},  # type: ignore[arg-type]
        )

    @classmethod
    def load(cls, path: Path) -> tuple[LinearPolicy, dict[str, object]]:
        """Read a weight file, and return the policy and what it names.

        A file that states no normalizer gives a policy that reads the plain
        squash, so a policy published before the normalizer existed still
        runs.
        """
        stored = np.load(path, allow_pickle=False)
        skip = {"weights", *NORMALIZER_KEYS}
        meta = {key: stored[key].tolist() for key in stored.files if key not in skip}
        return cls(stored["weights"], FeatureNormalizer.read(stored)), meta


def load_policy(
    path: Path,
    wanted: PolicyFit | None = None,
    layout: ObservationLayout | None = None,
) -> tuple[LinearPolicy | StructuredPolicy, dict[str, object]]:
    """Read a weight file, and return the policy it holds and what it names.

    The file states its own kind. A file that names none holds a linear
    policy, which is what the runs of the first night wrote.

    **Pass the fit of the world the policy will play.** The reader then
    refuses a file that was trained against another world, and it names both
    sides in the message. A caller that passes nothing takes whatever the
    file holds, which is correct for a reader that only reports what a file
    says.

    **Pass the layout when the policy is a structured one.** The two schema
    versions catch a change the engine made to the observation. They do not
    catch a change a caller made to the geometry it states over one version,
    and a stack of 151 cells by 25 channels holds the same positions as a
    stack of 25 cells by 151 channels. The layout separates the two.

    Raises ``PolicyFitError`` when a fit is asked for and the file does not
    match it, when a fit is asked for and the file states none, and when a
    layout is asked for and the file reads another one.

    **A file that names a kind this package no longer builds is refused by
    name.** The project deleted one kind, and a file written under it holds
    its two layers and no single weight matrix. Reading it as a linear policy
    raised an error about a missing archive entry, which named the storage
    and not the cause.[^1]

    **A file that states no normalizer still runs.** The published style
    files were written before the normalizer existed, so they hold no centre
    and no scale. The policy this reader gives back for one of them holds
    none, and a policy that holds none reads the plain squash, which is what
    the identity of the normalizer computes. Nothing refuses such a file,
    because the fit compares two normalizers only when both sides state one.

    **The readout is rebuilt row by row when the file and the world state
    two different action tables.** A row keeps its weight when its verb and
    its candidate coordinates survive, and a row an argument added takes the
    weight of the stored row it narrows.[^2] The reader states what it moved
    under the key ``action_rebuild``, and it states nothing when it moved
    nothing.

    References
    ----------
    [^1]: Findings register, FND-686. ``docs/FINDINGS.md``

    [^2]: ADR-0200, a stored policy names each row of the action table by its
    verb and its candidate coordinates, decisions D3 and D5.
    ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
    """
    stored = np.load(path, allow_pickle=False)
    kind = str(stored["kind"]) if "kind" in stored.files else "linear"
    skip = {"weights", "kind", "flat", "architecture", *NORMALIZER_KEYS}
    meta = {
        key: stored[key].tolist()
        for key in stored.files
        if key not in skip and not key.startswith("layout_")
    }
    meta["kind"] = kind
    # The two normalizer arrays stay out of the reported entries. They run to
    # a few thousand numbers each, and every caller of this reader prints or
    # stores what it gets back.
    normalizer = FeatureNormalizer.read(stored)
    rebuild: ActionRebuild | None = None
    if wanted is not None:
        held = PolicyFit.read(meta, normalizer)
        if held is None:
            message = (
                f"the stored policy at {path} states no fit, so nothing can "
                "place it. The file must name every one of "
                f"{', '.join(PolicyFit.KEYS)}. The world states "
                f"{wanted.describe()}. Train a policy against this world."
            )
            raise PolicyFitError(message)
        held.check(wanted, path)
        rebuild = held.rebuild_for(wanted)
        if rebuild is not None and not rebuild.moved:
            rebuild = None
        if rebuild is not None:
            meta["action_rebuild"] = rebuild.describe()
    if kind == "structured":
        # The structured policy reads this module for the encoder and the
        # fit, so the import sits here and the two modules do not form a
        # cycle at import time.
        from .structured import StructuredPolicy

        policy = StructuredPolicy.restore(stored)
        if layout is not None:
            policy.check_layout(layout, path)
        if rebuild is not None:
            policy = policy.with_readout(rebuild.apply(policy.readout))
        return policy, meta
    if "weights" not in stored.files:
        message = (
            f"the stored policy at {path} names the kind {kind!r}, which this "
            "package does not build. The kinds it builds are 'linear' and "
            "'structured'. Train a policy against this world."
        )
        raise PolicyFitError(message)
    weights = np.asarray(stored["weights"], dtype=np.float64)
    if rebuild is not None:
        weights = rebuild.apply(weights)
    return LinearPolicy(weights, normalizer), meta


class RandomPolicy:
    """A policy that takes one legal row at random.

    This is the second baseline. The untrained linear policy scores every row
    at zero and therefore always takes the no-op, which measures a faction
    that does nothing. A random policy measures a faction that acts without
    reading the world, and it is the harder of the two to beat.
    """

    def __init__(self, seed: int = 0) -> None:
        """Build the policy over one stream of draws."""
        self._rng = np.random.default_rng(seed)

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one legal action for each row, drawn uniformly."""
        del observations
        chosen = []
        for mask in masks:
            legal = np.flatnonzero(mask)
            chosen.append(int(self._rng.choice(legal)))
        return chosen
