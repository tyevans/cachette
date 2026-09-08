"""The objective vector of a run, built from the signals a world publishes.

A learner needs one number on every decision. A researcher needs more than
one. The two needs are not the same, and a reward that holds one scalar
serves only the first. A run that ranks candidates by one number cannot say
whether the leader won by taking ground or by killing units, and it cannot
say what a generation traded away.

This module holds the middle term. An objective is a named, bounded quantity
built from the signals of one observation. The objective vector is the set of
them. A play style is a weighting over that vector, and the scalar a learner
receives is the weighted combination.[^1]

# Every term is bounded, because a raw count is not comparable

A term over a raw count means different things on two worlds. Half of a 24 by
24 world and half of a 512 by 512 world are the same play and a different
number, so a weight tuned on one misreads the other.[^2] This module
therefore admits no raw term. Every term states one of four kinds, and each
kind maps its signal into the closed interval from minus one to one. A
decision record binds that rule and the normalisation of the combination.[^6]

- A share divides the signal by a named denominator signal.
- A signed relation divides the difference of two signals by the sum of their
  magnitudes, which bounds it without a chosen denominator.
- A compressed magnitude maps the signal through a base-two logarithm against
  a fixed 40-bit cap. It needs no denominator and it holds the sign.
- A fixed-point value divides by the Q16.16 unit, for a signal the engine
  already publishes as a share.

The research report prefers a share to a compressed magnitude for a reward
term, because the derivative of a compressed magnitude falls with the
quantity, so an early gain outweighs a late gain of the same size.[^3] A
share needs a denominator the engine publishes, and this module refuses to
invent one. A caller that has no denominator uses a compressed magnitude and
accepts the bias.

# Floating point is allowed here, and reaches nothing the engine stores

The engine holds no floating point value in simulated or aggregated state,
and this module is on the other side of that boundary.[^4] It reads the
engine and writes nothing back, so no value computed here enters a state hash
and no golden file moves when a researcher changes a weight.

# Names come from the schema, never from this module

The engine owns the layout of the observation and states it in a schema.[^5]
The signal catalogue is the one reader of that schema, and this module reads
every signal through it.[^6] **No field name appears in this module.** A name
written here would be a second declaration of what the engine publishes, and
nothing would fail when the engine moved and this did not. A term therefore
names its signal as data, and a term that names a signal the world does not
publish fails with the list of what the world does publish.

# References

[^1]: Report 42, what a policy should be able to see, section 10.2.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^2]: Report 42, what a policy should be able to see, section 8.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^3]: Report 42, what a policy should be able to see, section 10.1.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^4]: ADR-0002, state holds no floating point number, decision D4.
``docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md``

[^5]: ADR-0195, the observation of a faction is a fixed-width scale-free
table.
``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``

[^6]: ADR-0196, a reward is a bounded weighted objective vector, decisions D1
and D2.
``docs/adrs/draft/adr-0196-a-reward-is-a-bounded-weighted-objective-vector.md``

[^6]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING, Final

from .signals import Aggregation

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterator, Mapping, Sequence

    import numpy as np

    from .signals import Signal, SignalCatalogue

# The Q16.16 unit. A signal the engine publishes as a share carries this as
# its full value, and the engine owns that choice.
FIXED_POINT_UNIT: Final = 65536.0

# The structural cap of a compressed magnitude, in bits. It admits any
# quantity below 1.1 times 10 to the twelfth, which is above every total the
# engine can reach at the target scale. This is a property of the compression
# and not a budget, so it is stated here rather than in a register.
MAGNITUDE_CAP_BITS: Final = 40.0

# The bounds every term obeys. A share never goes below zero, and the other
# three kinds hold the sign of the quantity they read.
UPPER: Final = 1.0
LOWER: Final = -1.0


class ObjectiveError(ValueError):
    """A term or an objective named something the world cannot supply.

    The message lists what the world does publish, because the failure this
    replaces was a caller carrying a name from one part of the project into
    another that spells it differently.
    """


class TermKind(Enum):
    """How one term maps a raw signal into a bounded number.

    Each kind mirrors one of the value kinds the observation design states,
    and each lands inside the closed interval from minus one to one.[^1]

    References
    ----------
    [^1]: Report 42, what a policy should be able to see, section 4.
    ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
    """

    SHARE = "share"
    RELATION = "relation"
    MAGNITUDE = "magnitude"
    FIXED_POINT = "fixed_point"

    @property
    def needs_denominator(self) -> bool:
        """Whether this kind reads a second signal to bound the first."""
        return self in (TermKind.SHARE, TermKind.RELATION)

    def apply(self, numerator: float, denominator: float | None) -> float:
        """Map one raw reading into the bounded range of this kind."""
        if self is TermKind.SHARE:
            return _clamp(
                max(numerator, 0.0) / max(denominator or 0.0, 1.0), 0.0, UPPER
            )
        if self is TermKind.RELATION:
            other = denominator or 0.0
            scale = max(abs(numerator) + abs(other), 1.0)
            return _clamp((numerator - other) / scale, LOWER, UPPER)
        if self is TermKind.MAGNITUDE:
            bits = math.log2(1.0 + abs(numerator))
            size = min(bits / MAGNITUDE_CAP_BITS, UPPER)
            return -size if numerator < 0.0 else size
        return _clamp(numerator / FIXED_POINT_UNIT, LOWER, UPPER)


class Measure(Enum):
    """Whether a term reads the level of a signal or its change.

    Potential-based shaping adds the change of a potential to the reward, and
    it leaves the optimal policy unchanged under a gradient method.[^1] Under
    an evolution strategy the same term telescopes. The strategy optimises the
    sum of the reward over the whole episode with no discount inside it, so
    the sum of a change is the level at the end minus the level at the start.
    A difference term then contributes one number for the whole episode and
    gives no signal inside it.[^2]

    A level term integrated over the episode gives a dense signal and biases
    the optimum. That is the trade an evolution strategy takes. A run that
    moves to a gradient method switches the same terms to the difference form
    and recovers the guarantee, and this module supports both.

    References
    ----------
    [^1]: Ng, Harada and Russell, policy invariance under reward
    transformations, 1999.

    [^2]: Report 42, what a policy should be able to see, section 10.4.
    ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
    """

    LEVEL = "level"
    DIFFERENCE = "difference"


@dataclass(frozen=True)
class Term:
    """One bounded reading of one named signal.

    The signal entry names a field the engine publishes. The kind entry says
    how the raw value becomes bounded. The aggregation entry reduces a signal
    of several positions to one number, and a signal of one position refuses
    one.

    The against entry names the denominator of a share or the other side of a
    signed relation. The two kinds that need it refuse a term without it, and
    the two kinds that do not need it refuse a term with it.

    The coefficient entry is how much this term contributes to its objective.
    It is a shape of the objective and not a play style, so a style never
    changes it.
    """

    signal: str
    kind: TermKind
    aggregation: Aggregation | None = None
    against: str | None = None
    against_aggregation: Aggregation | None = None
    measure: Measure = Measure.LEVEL
    coefficient: float = 1.0

    def __post_init__(self) -> None:
        """Refuse a term whose kind and denominator disagree."""
        if self.kind.needs_denominator and self.against is None:
            message = (
                f"a {self.kind.value} term over {self.signal!r} needs a second "
                "signal to bound it. Name one in the against entry."
            )
            raise ObjectiveError(message)
        if not self.kind.needs_denominator and self.against is not None:
            message = (
                f"a {self.kind.value} term over {self.signal!r} bounds itself, "
                f"so it takes no second signal. Remove {self.against!r}."
            )
            raise ObjectiveError(message)


@dataclass(frozen=True)
class Objective:
    """One named element of the objective vector.

    An objective is the weighted sum of its terms. Every term is bounded, so
    an objective is bounded by the sum of the absolute coefficients of its
    terms.

    The description entry says what the objective measures. It is prose for a
    reader of a report and nothing computes with it.
    """

    name: str
    terms: tuple[Term, ...]
    description: str = ""

    def __post_init__(self) -> None:
        """Refuse an objective that reads nothing."""
        if not self.terms:
            message = f"the objective {self.name!r} holds no term"
            raise ObjectiveError(message)

    @property
    def telescopes(self) -> bool:
        """Whether every term of this objective reads a change.

        Such an objective contributes one number for the whole episode under
        an evolution strategy, whatever its weight.
        """
        return all(term.measure is Measure.DIFFERENCE for term in self.terms)


@dataclass(frozen=True)
class _BoundTerm:
    """One term with its signals resolved against one layout."""

    term: Term
    signal: Signal
    against: Signal | None

    def level(self, observation: np.ndarray) -> float:
        """Return the bounded value of this term at one observation."""
        numerator = self.signal.read(observation, self.term.aggregation)
        denominator = (
            None
            if self.against is None
            else self.against.read(observation, self.term.against_aggregation)
        )
        return self.term.kind.apply(numerator, denominator)

    def read(self, observation: np.ndarray, previous: np.ndarray | None) -> float:
        """Return what this term contributes at one observation.

        A level term contributes its own value. A difference term contributes
        the change since the previous observation, and it contributes nothing
        at the first reading of an episode, because there is no change yet.
        """
        current = self.level(observation)
        if self.term.measure is Measure.LEVEL:
            return self.term.coefficient * current
        if previous is None:
            return 0.0
        return self.term.coefficient * (current - self.level(previous))


@dataclass(frozen=True)
class ObjectiveVector:
    """What one reading, or one whole episode, scored on each objective.

    The values entry holds one number for each objective, under the name of
    the objective. A reading holds the bounded value of that decision. An
    episode holds the sum over its decisions, which is the level integrated
    over time for a level term.
    """

    values: Mapping[str, float]

    @property
    def names(self) -> tuple[str, ...]:
        """The objectives this vector holds, in the order it was built."""
        return tuple(self.values)

    def plus(self, other: ObjectiveVector) -> ObjectiveVector:
        """Return the sum of this vector and another over the same objectives.

        Two vectors over different objectives cannot be added, because the
        sum would silently drop the objectives one of them holds.
        """
        if self.names != other.names:
            message = (
                f"these vectors hold different objectives: {self.names} "
                f"against {other.names}"
            )
            raise ObjectiveError(message)
        return ObjectiveVector(
            {name: value + other.values[name] for name, value in self.values.items()}
        )

    def combine(self, weights: Mapping[str, float]) -> float:
        """Return the weighted combination of this vector.

        The result divides by the sum of the absolute weights, so it stays
        inside the range of one objective however many objectives carry a
        weight. A fixed step size then means the same thing under every play
        style.[^1]

        An objective the weights leave out contributes nothing, and a weight
        that names no objective fails with the list of the objectives.

        References
        ----------
        [^1]: Report 42, what a policy should be able to see, section 10.2.
        ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
        """
        unknown = sorted(set(weights) - set(self.values))
        if unknown:
            message = (
                f"{unknown} names no objective of this vector. It holds "
                f"{sorted(self.values)}."
            )
            raise ObjectiveError(message)
        scale = sum(abs(weight) for weight in weights.values())
        if scale == 0.0:
            return 0.0
        total = sum(weight * self.values[name] for name, weight in weights.items())
        return total / scale

    def as_dict(self) -> dict[str, float]:
        """Return this vector as plain values, for a report file."""
        return dict(self.values)


class ObjectiveSet:
    """The objective vector of one run, bound to the layout of one world.

    A caller builds this once for a world shape and reads every observation
    of that shape through it. The build resolves every signal a term names,
    so a term that names a signal the world does not publish fails here and
    not inside an episode.
    """

    def __init__(
        self, objectives: Sequence[Objective], catalogue: SignalCatalogue
    ) -> None:
        """Bind each term of each objective to the layout of one world."""
        if not objectives:
            message = "an objective set holds at least one objective"
            raise ObjectiveError(message)
        seen: dict[str, Objective] = {}
        for objective in objectives:
            if objective.name in seen:
                message = f"two objectives are named {objective.name!r}"
                raise ObjectiveError(message)
            seen[objective.name] = objective
        self._objectives = tuple(objectives)
        self._bound = {
            objective.name: tuple(
                _bind(term, catalogue, objective.name) for term in objective.terms
            )
            for objective in self._objectives
        }
        self._length = catalogue.observation_length

    def __len__(self) -> int:
        """How many objectives the vector holds."""
        return len(self._objectives)

    def __iter__(self) -> Iterator[Objective]:
        """Walk every objective in the order the caller declared."""
        return iter(self._objectives)

    def __contains__(self, name: object) -> bool:
        """Whether the vector holds an objective of this name."""
        return name in self._bound

    @property
    def names(self) -> tuple[str, ...]:
        """The objectives of the vector, in declaration order."""
        return tuple(objective.name for objective in self._objectives)

    def objective(self, name: str) -> Objective:
        """Return one objective by name, and name the alternatives when absent."""
        for objective in self._objectives:
            if objective.name == name:
                return objective
        message = f"{name!r} names no objective. The vector holds {sorted(self.names)}."
        raise ObjectiveError(message)

    def telescoping(self) -> tuple[str, ...]:
        """Return the objectives whose every term reads a change.

        Such an objective gives no signal inside an episode under an
        evolution strategy, because the sum of a change over the episode is
        the level at the end minus the level at the start.
        """
        return tuple(
            objective.name for objective in self._objectives if objective.telescopes
        )

    def read(
        self, observation: np.ndarray, previous: np.ndarray | None = None
    ) -> ObjectiveVector:
        """Return the objective vector of one observation.

        The previous entry is the observation of the decision before this
        one. A level term ignores it. A difference term reads the change
        against it, and contributes nothing when it is absent.
        """
        self._require_length(observation)
        if previous is not None:
            self._require_length(previous)
        return ObjectiveVector(
            {
                name: sum(term.read(observation, previous) for term in terms)
                for name, terms in self._bound.items()
            }
        )

    def zero(self) -> ObjectiveVector:
        """Return the vector that scored nothing on every objective."""
        return ObjectiveVector(dict.fromkeys(self.names, 0.0))

    def _require_length(self, observation: np.ndarray) -> None:
        """Refuse an observation of another layout."""
        if observation.shape[-1] != self._length:
            message = (
                f"this objective set reads an observation of {self._length} "
                f"positions and was given one of {observation.shape[-1]}"
            )
            raise ObjectiveError(message)


def _bind(term: Term, catalogue: SignalCatalogue, objective: str) -> _BoundTerm:
    """Resolve the signals of one term, and say what is on offer when it fails."""
    return _BoundTerm(
        term=term,
        signal=_signal(catalogue, term.signal, objective),
        against=(
            None
            if term.against is None
            else _signal(catalogue, term.against, objective)
        ),
    )


def _signal(catalogue: SignalCatalogue, name: str, objective: str) -> Signal:
    """Return one signal, and raise an objective error when it is absent.

    The catalogue already names the alternatives. This wraps its refusal so
    that a caller catches one error type for every failure of an objective
    set, and so that the message says which objective asked.
    """
    try:
        return catalogue.signal(name)
    except KeyError as absent:
        message = f"the objective {objective!r} asks for a signal that is absent. "
        raise ObjectiveError(message + str(absent.args[0])) from absent


def _clamp(value: float, lower: float, upper: float) -> float:
    """Hold one value inside a closed interval."""
    return max(lower, min(upper, value))


__all__ = [
    "FIXED_POINT_UNIT",
    "LOWER",
    "MAGNITUDE_CAP_BITS",
    "UPPER",
    "Measure",
    "Objective",
    "ObjectiveError",
    "ObjectiveSet",
    "ObjectiveVector",
    "Term",
    "TermKind",
]
