"""Every quantity a faction can read about itself, declared once.

A signal is one named quantity the engine publishes in the observation of a
faction. The engine owns the layout and states it in a schema, so this module
reads the schema and states no position of its own.[^1]

# One declaration, because three declarations disagreed

The project held the set of readable quantities in three places. The reward
accepted a term for any field of one position and refused the rest. The
trainer reported a fixed tuple of names. The schema declared the whole set,
and how many of its fields hold one position depends on the world. Nothing
compared the three, and they disagreed in two ways that cost a measurement
each.

The trainer reports the tick of the end under the name ``end_tick`` while a
reward must ask for a field of the schema, so a caller that carried a reported
name into a weighting got an error, and a caller that carried a weighed name
into a reading got silence. The silence is the worse of the two: a measurement
of which quantity predicts winning read a missing name as zero for every
candidate, scored the resulting tie as a coin, and reported that the tick of
the end predicts nothing when it had never been read.[^2]

**The engine publishes no signal that carries the tick of the end.** The
project believed it published one called ``tick``, and a record read that name
out of the signals with a default of zero. The name is absent from every
schema, so the column held zero for every episode the project recorded. A
caller reads the end tick from the world and never from this catalogue.[^4]

This module is the one declaration. A caller asks it what the engine
publishes, and reads a value through it.

# A quantity of many positions needs an aggregation

Some fields hold one position and the rest hold several. The relation of a
faction holds one position for each faction, the trade board holds one for
each row it carries, and each summary of the block lattice holds one for each
cell. A reward that accepts one position alone can therefore not score trade
and can not score diplomacy, and no weighting reaches those fields however it
is written.

**Which fields hold one position is a property of the world and not of the
engine.** A lattice of one cell makes every summary of that lattice a scalar,
and a world of two factions makes the relation a scalar. A caller that holds a
count of the scalars is therefore right for one world shape and wrong for the
next, and this module states the rule rather than the count.[^3]

A signal of many positions becomes a scalar through an aggregation, and the
caller chooses which. The aggregation is part of the objective and not a
property of the engine, so it belongs to the caller that states the weighting.

# A published value is not the quantity, and the engine says which form it is

The engine publishes every quantity in one of several forms. A count such as a
population, a settlement total, a held tile total or a store total crosses as a
compressed magnitude. Others are shares, signed relations, or groups of the
three over one quantity.

The schema states the form of each field and the parameters an inversion of
that form needs, so this module inverts a value and states no compression rule
of its own. A rule declared twice fails silently when one copy moves, and that
is the defect shape this project names first.[^5]

**The inversion belongs on the signal and not at each call site.** A
measurement of a population against a baseline compares two published values
exactly, because the compression is monotone. It cannot state the difference in
people until it inverts them, and a tool that held the compression would hold a
second copy of an engine rule.

A share and a signed relation are not invertible. Neither value carries the
denominator it divided by, so the signal refuses and the message says what the
value would need.

# References

[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^2]: Findings register, FND-669. ``docs/FINDINGS.md``
[^3]: Findings register, FND-670. ``docs/FINDINGS.md``
[^4]: Findings register, FND-689. ``docs/FINDINGS.md``
[^5]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING, Protocol

import numpy as np

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterator, Mapping, Sequence


class WorldLike(Protocol):
    """What this module needs of a world: the schema it publishes."""

    def observation_schema(self) -> Mapping[str, object]:
        """Return the layout of the observation of one faction."""


class Aggregation(Enum):
    """How a signal of many positions becomes one number.

    The choice belongs to the objective that reads the signal and never to the
    engine that publishes it. A caller that weighs the relation of a faction
    may want the total standing of its rivals, or the standing of its worst
    rival, and neither is more correct than the other.
    """

    SUM = "sum"
    MEAN = "mean"
    HIGHEST = "highest"
    LOWEST = "lowest"

    def apply(self, values: np.ndarray) -> float:
        """Reduce the positions of one signal to one number."""
        if values.size == 0:
            message = "an aggregation needs at least one position"
            raise ValueError(message)
        if self is Aggregation.SUM:
            return float(values.sum())
        if self is Aggregation.MEAN:
            return float(values.mean())
        if self is Aggregation.HIGHEST:
            return float(values.max())
        return float(values.min())


@dataclass(frozen=True)
class ValueForm:
    """One value form of the observation, and how to invert it.

    The engine publishes every quantity in one of several forms. A count such
    as a population, a settlement total or a store total crosses as a
    compressed magnitude, and a compressed magnitude hides the count it came
    from. A reader that wanted the count therefore had to restate the
    compression, which put an engine rule in two places with nothing that
    fails when the two disagree.[^5]

    This class holds the parameters the engine publishes for one form. It
    states no parameter of its own, so a change to the compression reaches
    every reader through the schema.
    """

    name: str
    low: int
    high: int
    unit: int
    uniform: bool
    invertible: bool
    denominator: str | None = None
    log_base: int | None = None
    log_offset: int | None = None
    divisor_bits: int | None = None

    @classmethod
    def of_row(cls, name: str, row: Mapping[str, object]) -> ValueForm:
        """Read one form from the entry the schema publishes for it."""
        return cls(
            name=str(row.get("name", name)),
            low=int(row["low"]),  # type: ignore[arg-type]
            high=int(row["high"]),  # type: ignore[arg-type]
            unit=int(row["unit"]),  # type: ignore[arg-type]
            uniform=bool(row["uniform"]),
            invertible=bool(row["invertible"]),
            denominator=_text_or_none(row.get("denominator")),
            log_base=_int_or_none(row.get("log_base")),
            log_offset=_int_or_none(row.get("log_offset")),
            divisor_bits=_int_or_none(row.get("divisor_bits")),
        )

    @property
    def relative_precision(self) -> float:
        """The relative error the divisor puts on an inversion of this form.

        The compression divides a logarithm by the divisor and truncates, so
        the recovered quantity is near the true one and rarely equal to it. A
        caller that reports a recovered count must report this figure beside
        it. A difference smaller than this figure is not a difference the
        observation carries.

        **Read this as the scale of the error and not as a strict cap.** The
        engine truncates the logarithm as well, by a much smaller amount, and
        this figure does not add that term.

        A form that takes no logarithm carries no such error and answers zero.
        """
        if self.divisor_bits is None or self.log_base is None:
            return 0.0
        return float(self.log_base ** (self.divisor_bits / self.unit) - 1.0)

    def invert(self, value: float | np.ndarray) -> np.ndarray:
        """Recover the quantity behind one published value, or an array of them.

        The inversion raises the published base to the recovered exponent and
        subtracts the published offset. It restores the sign of the value,
        because the compression keeps it.

        A form the engine does not call invertible refuses, and the message
        says what the value would need. A share divides by a whole that the
        doc of each field names and that no position carries. A signed
        relation divides by the sum of the two magnitudes it compares, and one
        value cannot give both back.
        """
        if not self.invertible:
            message = (
                f"the {self.name!r} form is not invertible. "
                f"Its denominator is {self.denominator!r}, and no position of "
                f"the observation carries it."
            )
            raise ValueError(message)
        if self.log_base is None or self.log_offset is None:
            message = (  # pragma: no cover - schema contract
                f"the {self.name!r} form claims to be invertible and "
                f"publishes no logarithm"
            )
            raise ValueError(message)
        if self.divisor_bits is None:
            message = (  # pragma: no cover - schema contract
                f"the {self.name!r} form claims to be invertible and "
                f"publishes no divisor"
            )
            raise ValueError(message)
        magnitude = np.abs(np.asarray(value, dtype=np.float64))
        exponent = magnitude * self.divisor_bits / self.unit
        recovered = np.float_power(self.log_base, exponent) - self.log_offset
        return np.sign(np.asarray(value, dtype=np.float64)) * recovered


@dataclass(frozen=True)
class Signal:
    """One named quantity of the observation, and where it sits.

    The name, the start and the position count are the reading contract, and
    every schema states all three. The four entries after them describe the
    shape of the quantity, and a schema may state none of them.

    The space entry says that the positions of the signal are places in the
    world rather than separate quantities. A drawing of the observation needs
    that to lay the positions out, and a reader of one value does not. The
    block entry names the group the layout puts the signal in. The gate entry
    names another signal that says which positions carry a value at all, so
    that a reader tells an empty position from a position that holds zero. The
    channels entry names the quantities of a signal that holds several of them
    over one set of places.

    The form entry names how the engine wrote every position of the signal,
    and it carries the parameters an inversion of that form needs. A reader
    that wants the count behind a compressed magnitude asks the signal for it,
    and holds no compression rule of its own.

    Each of the five defaults to absent, so a schema that states none of them
    gives the same catalogue it gave before they existed.
    """

    name: str
    start: int
    positions: int
    space: str | None = None
    block: str | None = None
    gate: str | None = None
    channels: tuple[str, ...] = ()
    form: ValueForm | None = None

    @property
    def scalar(self) -> bool:
        """Whether this signal holds one position and needs no aggregation."""
        return self.positions == 1

    def read(
        self, observation: np.ndarray, aggregation: Aggregation | None = None
    ) -> float:
        """Read this signal from one observation array.

        A signal of one position needs no aggregation and refuses one, because
        an aggregation over one position hides which signal the caller thought
        it was reading. A signal of many positions requires one.
        """
        window = observation[self.start : self.start + self.positions]
        if self.scalar:
            if aggregation is not None:
                message = (
                    f"{self.name!r} holds one position, so it takes no "
                    f"aggregation. Remove the {aggregation.value!r}."
                )
                raise ValueError(message)
            return float(window[0])
        if aggregation is None:
            message = (
                f"{self.name!r} holds {self.positions} positions, so it needs "
                f"an aggregation. Name one of "
                f"{sorted(entry.value for entry in Aggregation)}."
            )
            raise ValueError(message)
        return aggregation.apply(window)

    def invert(self, value: float | np.ndarray) -> np.ndarray:
        """Recover the quantity behind one published value of this signal.

        The engine names the form of the signal and publishes the parameters
        of that form, so the arithmetic lives here once rather than at each
        call site.

        A signal whose schema states no form refuses, and so does a signal of
        a form the engine does not call invertible. A signal whose form groups
        several forms over one quantity also refuses, because its positions do
        not share one rule and a reader must go by channel.
        """
        form = self._invertible_form()
        return form.invert(value)

    def quantities(self, observation: np.ndarray) -> np.ndarray:
        """Recover the quantity behind every position of this signal.

        This is what a caller that wants a count reads. A measurement of a
        population against a baseline can compare two published values
        exactly, because the compression is monotone, and it cannot state the
        difference in people until it inverts them.
        """
        window = observation[self.start : self.start + self.positions]
        return self.invert(window)

    @property
    def invertible(self) -> bool:
        """Whether this signal recovers its quantity from a published value."""
        form = self.form
        return form is not None and form.invertible and form.uniform

    def _invertible_form(self) -> ValueForm:
        """Return the form of this signal, or say why it cannot be inverted."""
        form = self.form
        if form is None:
            message = (
                f"the schema states no value form for {self.name!r}, so "
                f"nothing here knows how the engine wrote it"
            )
            raise ValueError(message)
        if not form.uniform:
            message = (
                f"{self.name!r} holds the {form.name!r} form, which groups "
                f"several forms over one quantity. Read its channels, because "
                f"its positions do not share one rule."
            )
            raise ValueError(message)
        return form


def _text_or_none(value: object) -> str | None:
    """Read one optional text entry of a schema row."""
    if value is None:
        return None
    return str(value)


def _int_or_none(value: object) -> int | None:
    """Read one optional integer entry of a schema row."""
    if value is None:
        return None
    return int(value)  # type: ignore[call-overload]


def _forms_of(published: object) -> dict[str, ValueForm]:
    """Read the value form table of a schema, which may state none."""
    if published is None:
        return {}
    if not isinstance(published, dict):  # pragma: no cover - schema contract
        message = (
            f"a value form table must be a dict, and this schema holds {published!r}"
        )
        raise TypeError(message)
    return {
        str(name): ValueForm.of_row(str(name), row) for name, row in published.items()
    }


def _form_of(
    row: Mapping[str, object], forms: Mapping[str, ValueForm]
) -> ValueForm | None:
    """Resolve the value form one field names, against the table of forms.

    A field that names a form the table does not hold is refused rather than
    left without one. The two come from one declaration in the engine, so a
    name with no entry means the schema disagrees with itself, and a reader
    that took the absence for a field with no form would invert nothing and
    report no error.
    """
    named = _text_or_none(row.get("form"))
    if named is None:
        return None
    found = forms.get(named)
    if found is None:
        message = (
            f"the field {row.get('name')!r} names the value form {named!r}, "
            f"and the schema publishes {sorted(forms)}"
        )
        raise KeyError(message)
    return found


def _names(value: object) -> tuple[str, ...]:
    """Read one optional list of names of a schema row."""
    if value is None:
        return ()
    if isinstance(value, str):  # pragma: no cover - schema contract
        message = "a channel list holds names, and this row holds one string"
        raise TypeError(message)
    if not isinstance(value, list | tuple):  # pragma: no cover - schema contract
        message = f"a channel list must be a list, and this row holds {value!r}"
        raise TypeError(message)
    return tuple(str(entry) for entry in value)


class SignalCatalogue:
    """Every signal one world shape publishes, read from its own schema.

    A caller builds this once for a world shape and reads every observation of
    that shape through it. Two worlds of one shape publish one layout, so the
    catalogue of either answers for both.
    """

    def __init__(
        self,
        signals: Sequence[Signal],
        length: int,
        geometry: Mapping[str, object] | None = None,
    ) -> None:
        """Hold the signals of one layout, in the order the schema gives.

        The geometry argument carries every entry of the schema that is not a
        field and is not the length. A drawing of the spatial part of the
        observation needs the shape of that part, and the schema is the only
        place that can state it. The argument defaults to empty, so a caller
        that builds a catalogue by hand builds the same one it built before.
        """
        self._signals = tuple(signals)
        self._by_name = {signal.name: signal for signal in self._signals}
        self._length = length
        self._geometry = dict(geometry or {})

    @classmethod
    def of_world(cls, world: WorldLike) -> SignalCatalogue:
        """Read the catalogue from the schema a world publishes."""
        schema = world.observation_schema()
        fields = schema["fields"]
        if not isinstance(fields, list):  # pragma: no cover - schema contract
            message = "the schema of the observation holds no field list"
            raise TypeError(message)
        forms = _forms_of(schema.get("value_forms"))
        signals = [
            Signal(
                name=str(row["name"]),
                start=int(row["start"]),
                positions=int(row["positions"]),
                space=_text_or_none(row.get("space")),
                block=_text_or_none(row.get("block")),
                gate=_text_or_none(row.get("gate")),
                channels=_names(row.get("channels")),
                form=_form_of(row, forms),
            )
            for row in fields
        ]
        length = schema["length"]
        if not isinstance(length, int):  # pragma: no cover - schema contract
            message = "the schema of the observation states no length"
            raise TypeError(message)
        geometry = {
            key: value
            for key, value in schema.items()
            if key not in {"fields", "length"}
        }
        return cls(signals, length, geometry)

    @property
    def geometry(self) -> Mapping[str, object]:
        """Every schema entry that is not a field and is not the length."""
        return dict(self._geometry)

    @property
    def value_forms(self) -> Mapping[str, ValueForm]:
        """Every value form the layout uses, keyed by the name of the form.

        A signal carries its own form, so a caller that reads one value needs
        this table for nothing. A caller that reports what the whole layout
        publishes reads it, because it names every rule the engine wrote a
        value under.
        """
        return {
            signal.form.name: signal.form
            for signal in self._signals
            if signal.form is not None
        }

    def invertible(self) -> tuple[Signal, ...]:
        """Every signal whose quantity a reader recovers from its value.

        A tool that reports a count reads this. A signal left out either
        divides by a whole that no position carries, or holds a form that
        groups several rules over one quantity.
        """
        return tuple(signal for signal in self._signals if signal.invertible)

    def __len__(self) -> int:
        """How many signals the layout holds."""
        return len(self._signals)

    def __iter__(self) -> Iterator[Signal]:
        """Walk every signal in the order the schema declares."""
        return iter(self._signals)

    def __contains__(self, name: object) -> bool:
        """Whether the layout holds a signal of this name."""
        return name in self._by_name

    @property
    def observation_length(self) -> int:
        """How long one observation of this layout is."""
        return self._length

    def signal(self, name: str) -> Signal:
        """Return one signal by name, and name the alternatives when it is absent.

        The message lists what the layout does hold, because the failure this
        replaces was a caller carrying a name from one part of the project into
        another that spells it differently.
        """
        found = self._by_name.get(name)
        if found is None:
            message = (
                f"{name!r} names no signal of this world. The layout holds "
                f"{sorted(self._by_name)}."
            )
            raise KeyError(message)
        return found

    def scalars(self) -> tuple[Signal, ...]:
        """Every signal that holds one position, so needs no aggregation."""
        return tuple(signal for signal in self._signals if signal.scalar)

    def compound(self) -> tuple[Signal, ...]:
        """Every signal that holds several positions, so needs an aggregation."""
        return tuple(signal for signal in self._signals if not signal.scalar)

    def read_scalars(self, observation: np.ndarray) -> dict[str, float]:
        """Read every one-position signal of one observation.

        This is what a record of an episode carries, so that a later reader
        holds every quantity the engine published rather than the subset one
        caller thought to name.
        """
        self._require_length(observation)
        return {signal.name: signal.read(observation) for signal in self.scalars()}

    def read(
        self,
        observation: np.ndarray,
        aggregations: Mapping[str, Aggregation] | None = None,
    ) -> dict[str, float]:
        """Read every signal, aggregating the compound ones as the caller says.

        A compound signal with no named aggregation is left out rather than
        guessed at, because a guess here becomes an objective nobody chose.
        """
        self._require_length(observation)
        chosen = dict(aggregations or {})
        values: dict[str, float] = {}
        for signal in self._signals:
            if signal.scalar:
                values[signal.name] = signal.read(observation)
                continue
            aggregation = chosen.get(signal.name)
            if aggregation is not None:
                values[signal.name] = signal.read(observation, aggregation)
        return values

    def _require_length(self, observation: np.ndarray) -> None:
        """Refuse an observation of another layout."""
        if observation.shape[-1] != self._length:
            message = (
                f"this catalogue reads an observation of {self._length} "
                f"positions and was given one of {observation.shape[-1]}"
            )
            raise ValueError(message)
