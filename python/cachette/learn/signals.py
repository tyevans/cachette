"""Every quantity a faction can read about itself, declared once.

A signal is one named quantity the engine publishes in the observation of a
faction. The engine owns the layout and states it in a schema, so this module
reads the schema and states no position of its own.[^1]

# One declaration, because three declarations disagreed

The project held the set of readable quantities in three places. The reward
accepted a term for any field of one position and refused the rest. The
trainer reported a fixed tuple of seven names. The schema declared thirty
fields of which twelve hold one position. Nothing compared the three, and they
disagreed in two ways that cost a measurement each.

The trainer reports the tick of the end under the name ``end_tick`` while a
reward must ask for ``tick``, so a caller that carried a reported name into a
weighting got an error, and a caller that carried a weighed name into a
reading got silence. The silence is the worse of the two: a measurement of
which quantity predicts winning read a missing name as zero for every
candidate, scored the resulting tie as a coin, and reported that the tick of
the end predicts nothing when it had never been read.[^2]

This module is the one declaration. A caller asks it what the engine
publishes, and reads a value through it.

# A quantity of many positions needs an aggregation

Twelve of the thirty fields hold one position. The relation of a faction holds
one position for each faction, the trade board holds one for each row it
carries, and each summary of the block lattice holds one for each cell. A
reward that accepts one position alone can therefore not score trade and can
not score diplomacy, and no weighting reaches those fields however it is
written.

A signal of many positions becomes a scalar through an aggregation, and the
caller chooses which. The aggregation is part of the objective and not a
property of the engine, so it belongs to the caller that states the weighting.

# References

[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^2]: Findings register, FND-670. ``docs/FINDINGS.md``
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
class Signal:
    """One named quantity of the observation, and where it sits."""

    name: str
    start: int
    positions: int

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


class SignalCatalogue:
    """Every signal one world shape publishes, read from its own schema.

    A caller builds this once for a world shape and reads every observation of
    that shape through it. Two worlds of one shape publish one layout, so the
    catalogue of either answers for both.
    """

    def __init__(self, signals: Sequence[Signal], length: int) -> None:
        """Hold the signals of one layout, in the order the schema gives."""
        self._signals = tuple(signals)
        self._by_name = {signal.name: signal for signal in self._signals}
        self._length = length

    @classmethod
    def of_world(cls, world: WorldLike) -> SignalCatalogue:
        """Read the catalogue from the schema a world publishes."""
        schema = world.observation_schema()
        fields = schema["fields"]
        if not isinstance(fields, list):  # pragma: no cover - schema contract
            message = "the schema of the observation holds no field list"
            raise TypeError(message)
        signals = [
            Signal(
                name=str(row["name"]),
                start=int(row["start"]),
                positions=int(row["positions"]),
            )
            for row in fields
        ]
        length = schema["length"]
        if not isinstance(length, int):  # pragma: no cover - schema contract
            message = "the schema of the observation states no length"
            raise TypeError(message)
        return cls(signals, length)

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
