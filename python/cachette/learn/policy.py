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


def encode(observation: np.ndarray) -> np.ndarray:
    """Turn one observation array into the feature vector of the policy.

    The result holds one entry for each position of the observation, and one
    trailing entry of one for the bias.
    """
    values = observation.astype(np.float64)
    squashed = np.sign(values) * np.log1p(np.abs(values)) / FEATURE_SCALE
    return np.concatenate([squashed, np.ones(1)])


def encode_many(observations: np.ndarray) -> np.ndarray:
    """Encode a stack of observations, one for each row."""
    values = observations.astype(np.float64)
    squashed = np.sign(values) * np.log1p(np.abs(values)) / FEATURE_SCALE
    ones = np.ones((values.shape[0], 1))
    return np.concatenate([squashed, ones], axis=1)


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
    def of_env(cls, env: EnvLike) -> PolicyFit:
        """Return the fit of the world one environment builds."""
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
        )

    @classmethod
    def of_world(cls, world: WorldLike) -> PolicyFit:
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
        )

    @classmethod
    def read(cls, meta: Mapping[str, object]) -> PolicyFit | None:
        """Return the fit a weight file states, or nothing when it states none.

        A file written before this package stored a fit names some of the
        keys and not others. Such a file cannot be placed, so this returns
        nothing and the caller refuses it.

        A file that also states an action table reads it back here, and a
        file that states none reads back a fit whose table is nothing.

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
        return cls(**values, action_table=table)

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

    def __init__(self, weights: np.ndarray) -> None:
        """Take the weight matrix. Its shape is (actions, features)."""
        self.weights = np.asarray(weights, dtype=np.float64)

    @classmethod
    def zeros(cls, action_length: int, observation_length: int) -> LinearPolicy:
        """Build the untrained policy. Every score is zero.

        A zero policy takes the first legal row of the table at every
        decision, which is the no-op. It is the baseline every trained model
        is measured against.
        """
        return cls(np.zeros((action_length, observation_length + 1)))

    @property
    def shape(self) -> tuple[int, int]:
        """The shape of the weight matrix."""
        actions, features = self.weights.shape
        return actions, features

    def choose(self, observation: np.ndarray, mask: np.ndarray) -> int:
        """Return the action integer of the highest-scoring legal row."""
        scores = self.weights @ encode(observation)
        scores = np.where(mask > 0, scores, -np.inf)
        return int(np.argmax(scores))

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action for each row of a stack of observations."""
        scores = encode_many(observations) @ self.weights.T
        scores = np.where(masks > 0, scores, -np.inf)
        return [int(value) for value in np.argmax(scores, axis=1)]

    def flat(self) -> np.ndarray:
        """Return every trainable weight as one vector."""
        return self.weights.reshape(-1)

    def rebuild(self, flat: np.ndarray) -> LinearPolicy:
        """Return a policy of this shape with the given weights."""
        return LinearPolicy(flat.reshape(self.weights.shape))

    def save(self, path: Path, meta: Mapping[str, object]) -> None:
        """Write the weights and what they were trained against.

        The two schema versions go into the file, and so does the action
        table when the caller states one. **The table is what places a row of
        this file in a later table**, because a row is named by its verb and
        its candidate coordinates and not by its index.[^1]

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D1.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        # numpy declares ``allow_pickle`` as a keyword before its own
        # ``**kwds``, so a mapping keyed on ``str`` can never unpack cleanly.
        np.savez(
            path,
            weights=self.weights,
            kind=np.array("linear"),
            **{key: np.array(value) for key, value in meta.items()},  # type: ignore[arg-type]
        )

    @classmethod
    def load(cls, path: Path) -> tuple[LinearPolicy, dict[str, object]]:
        """Read a weight file, and return the policy and what it names."""
        stored = np.load(path, allow_pickle=False)
        meta = {key: stored[key].tolist() for key in stored.files if key != "weights"}
        return cls(stored["weights"]), meta


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
    skip = {"weights", "kind", "flat", "architecture"}
    meta = {
        key: stored[key].tolist()
        for key in stored.files
        if key not in skip and not key.startswith("layout_")
    }
    meta["kind"] = kind
    rebuild: ActionRebuild | None = None
    if wanted is not None:
        held = PolicyFit.read(meta)
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
    return LinearPolicy(weights), meta


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
