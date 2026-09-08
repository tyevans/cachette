"""The play styles a run may choose, read from a file a human edits.

A play style is data. A researcher who wants an aggressive policy beside a
trading one edits a table, and touches no code. This module reads that table
and gives back the objectives and the styles it declares.

The table is a document in the format the standard library reads, so the
package needs no dependency to read it and a reader needs no tool to edit
it.[^1] The format carries nested tables and inline tables, which is the
shape of an objective and its terms.

# Nothing here names a field of the engine

The engine owns the layout of the observation and states it in a schema.[^2]
A term of the table names the signal it reads, and the run resolves that name
against the schema of the world it plays. A term that names a signal the
world does not publish fails when the run binds the library to the world, and
the message lists what the world does publish.

**This module therefore carries no field name.** A field name written in
Python would be a second declaration of what the engine publishes, and
nothing would fail when the engine moved and this did not. The table is data,
and a name in it is a statement about the world the run plays.

# Varying the objective across a run

An evolution strategy ranks the candidates of one generation against each
other. Two candidates scored under two objectives are not comparable, so the
rank is noise and the update follows the noise. **A schedule therefore cannot
address a candidate.** It answers for a generation and for the position of an
episode inside a generation, and the run gives every candidate the same
answer.

The unsafe form is not available to configure. A schedule of the candidate
form has no name in this module, and the loader refuses the word with a
message that says why.

# References

[^1]: The `tomllib` module of the Python standard library, from version 3.11.

[^2]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, Any, Final

from .objective import Measure, Objective, ObjectiveSet, Term, TermKind
from .signals import Aggregation
from .style import (
    ObjectiveScoring,
    Optimisation,
    PlayStyle,
    StyleError,
    require_complete,
)

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from .reward import Scoring
    from .signals import SignalCatalogue

# The table this package ships. A caller that gives no path reads this one.
DEFAULT_LIBRARY: Final[Path] = Path(__file__).with_name("play_styles.toml")

# The keys a term of the table may hold. A key outside this set is a spelling
# mistake, and a spelling mistake that reads as a default is silent.
_TERM_KEYS: Final[frozenset[str]] = frozenset(
    {
        "signal",
        "kind",
        "aggregation",
        "against",
        "against_aggregation",
        "measure",
        "coefficient",
    }
)

# The keys a style of the table may hold.
_STYLE_KEYS: Final[frozenset[str]] = frozenset(
    {"description", "rewards", "refuses", "won", "lost", "drawn"}
)

# The keys an objective of the table may hold.
_OBJECTIVE_KEYS: Final[frozenset[str]] = frozenset({"description", "terms"})


class LibraryError(ValueError):
    """The table of play styles states something this module cannot read."""


class Variation(Enum):
    """How the objective of a run changes while the run proceeds.

    Three forms are safe, and one form is absent on purpose.

    The fixed form holds one style for the whole run. The generation form
    moves to the next style at each generation, and every candidate of one
    generation plays the same style. The episode form gives each position of
    the seed set its own style, in a pattern every candidate shares.

    **There is no candidate form.** An evolution strategy ranks the
    candidates of one generation against each other, so two candidates scored
    under two objectives give a rank that carries no information about either
    policy. The update then follows the noise. No member of this enumeration
    addresses a candidate, and the loader refuses the word.
    """

    FIXED = "fixed"
    GENERATION = "generation"
    EPISODE = "episode"

    @classmethod
    def of(cls, name: str) -> Variation:
        """Read one variation by name, and refuse the unsafe one by name.

        The candidate form gets its own message, because a reader who asks
        for it has a reason and needs the reason answered.
        """
        if name == "candidate":
            message = (
                "a run cannot vary the objective between the candidates of one "
                "generation. An evolution strategy ranks the candidates of a "
                "generation against each other, so two candidates scored under "
                "two objectives give a rank that says nothing about either "
                "policy, and the update follows the noise. Vary the objective "
                "between generations or between the episodes of every "
                f"candidate instead: {sorted(entry.value for entry in cls)}."
            )
            raise LibraryError(message)
        for entry in cls:
            if entry.value == name:
                return entry
        message = (
            f"{name!r} names no variation. The forms are "
            f"{sorted(entry.value for entry in cls)}."
        )
        raise LibraryError(message)


@dataclass(frozen=True)
class StyleLibrary:
    """Every objective and every play style one run may choose from.

    A library holds the objectives as declarations and the styles as
    weightings over them. It binds neither to a world, because the shape of a
    world decides which signals exist and a library outlives one world.
    """

    objectives: tuple[Objective, ...]
    styles: Mapping[str, PlayStyle]

    @property
    def names(self) -> tuple[str, ...]:
        """The styles this library holds, in the order the table declares."""
        return tuple(self.styles)

    def style(self, name: str) -> PlayStyle:
        """Return one style by name, and name the alternatives when absent."""
        found = self.styles.get(name)
        if found is None:
            message = (
                f"{name!r} names no play style. The library holds "
                f"{sorted(self.styles)}."
            )
            raise StyleError(message)
        return found

    def objective_set(self, catalogue: SignalCatalogue) -> ObjectiveSet:
        """Bind every objective to the layout of one world."""
        return ObjectiveSet(self.objectives, catalogue)

    def scoring(
        self,
        name: str,
        catalogue: SignalCatalogue,
        optimisation: Optimisation = Optimisation.EVOLUTION_STRATEGY,
    ) -> ObjectiveScoring:
        """Return the scoring of one style over the layout of one world."""
        return ObjectiveScoring(
            objectives=self.objective_set(catalogue),
            style=self.style(name),
            optimisation=optimisation,
        )


@dataclass(frozen=True)
class ObjectiveSchedule:
    """Which objective a run scores each generation and each episode with.

    The scorings entry holds the styles the run cycles through, in the order
    the caller gives. The variation entry says what moves between them.

    **No method of this type takes a candidate.** That is the guarantee, and
    it is structural rather than written down: a caller cannot ask for the
    scoring of one candidate, because there is no argument for one.
    """

    scorings: tuple[ObjectiveScoring, ...]
    variation: Variation = Variation.FIXED

    def __post_init__(self) -> None:
        """Refuse an empty schedule, and a fixed schedule of several styles."""
        if not self.scorings:
            message = "a schedule holds at least one scoring"
            raise LibraryError(message)
        if self.variation is Variation.FIXED and len(self.scorings) != 1:
            message = (
                f"a fixed schedule holds one scoring and this one holds "
                f"{len(self.scorings)}. Name the generation form or the "
                "episode form to cycle through several."
            )
            raise LibraryError(message)

    def for_generation(self, generation: int, episodes: int) -> tuple[Scoring, ...]:
        """Return the scoring of each episode position of one generation.

        The result holds one entry for each position of the seed set of the
        generation. **A run gives every candidate the same tuple**, so two
        candidates of one generation are always compared under the same
        objective at the same position.

        A negative generation or a run of no episodes is refused, because
        both would silently return nothing.
        """
        if generation < 0:
            message = f"a generation counts from zero and this one is {generation}"
            raise LibraryError(message)
        if episodes < 1:
            message = "a generation plays at least one episode"
            raise LibraryError(message)
        if self.variation is Variation.EPISODE:
            return tuple(
                self.scorings[position % len(self.scorings)]
                for position in range(episodes)
            )
        chosen = self.scorings[generation % len(self.scorings)]
        return (chosen,) * episodes

    def styles(self) -> tuple[str, ...]:
        """Return the styles this schedule cycles through, in order."""
        return tuple(scoring.style.name for scoring in self.scorings)


def load_library(path: Path | None = None) -> StyleLibrary:
    """Read the objectives and the play styles of one table.

    A caller that gives no path reads the table this package ships.
    """
    source = path or DEFAULT_LIBRARY
    with source.open("rb") as handle:
        document = tomllib.load(handle)
    objectives = _objectives_of(document.get("objective", {}))
    declared = tuple(objective.name for objective in objectives)
    styles = _styles_of(document.get("style", {}), declared)
    return StyleLibrary(objectives=objectives, styles=styles)


def schedule_of(
    library: StyleLibrary,
    names: Sequence[str],
    catalogue: SignalCatalogue,
    variation: str = "fixed",
    optimisation: Optimisation = Optimisation.EVOLUTION_STRATEGY,
) -> ObjectiveSchedule:
    """Build the schedule of one run from the names of its styles.

    The variation entry names one of the safe forms. The word for the unsafe
    form is refused here, so a run cannot be configured into it.
    """
    if not names:
        message = "a run scores under at least one play style"
        raise LibraryError(message)
    return ObjectiveSchedule(
        scorings=tuple(
            library.scoring(name, catalogue, optimisation) for name in names
        ),
        variation=Variation.of(variation),
    )


def _objectives_of(tables: Mapping[str, Any]) -> tuple[Objective, ...]:
    """Read every objective of the table, in declaration order."""
    if not tables:
        message = "the table declares no objective"
        raise LibraryError(message)
    objectives = []
    for name, body in tables.items():
        _refuse_keys(body, _OBJECTIVE_KEYS, f"the objective {name!r}")
        objectives.append(
            Objective(
                name=name,
                terms=tuple(_term_of(row, name) for row in _terms_of(body, name)),
                description=str(body.get("description", "")).strip(),
            )
        )
    return tuple(objectives)


def _terms_of(body: Mapping[str, Any], name: str) -> Sequence[Mapping[str, Any]]:
    """Return the term rows of one objective, and refuse a body without any."""
    rows = body.get("terms")
    if not isinstance(rows, list) or not rows:
        message = f"the objective {name!r} declares no term"
        raise LibraryError(message)
    return rows


def _term_of(row: Mapping[str, Any], objective: str) -> Term:
    """Read one term of one objective."""
    _refuse_keys(row, _TERM_KEYS, f"a term of the objective {objective!r}")
    signal = row.get("signal")
    if not isinstance(signal, str):
        message = f"a term of the objective {objective!r} names no signal"
        raise LibraryError(message)
    return Term(
        signal=signal,
        kind=_kind_of(row.get("kind"), objective),
        aggregation=_aggregation_of(row.get("aggregation"), objective),
        against=None if row.get("against") is None else str(row["against"]),
        against_aggregation=_aggregation_of(row.get("against_aggregation"), objective),
        measure=_measure_of(row.get("measure"), objective),
        coefficient=float(row.get("coefficient", 1.0)),
    )


def _kind_of(value: object, objective: str) -> TermKind:
    """Read the kind of one term, and name the alternatives when it is absent."""
    if not isinstance(value, str):
        message = (
            f"a term of the objective {objective!r} states no kind. The kinds "
            f"are {sorted(entry.value for entry in TermKind)}."
        )
        raise LibraryError(message)
    for entry in TermKind:
        if entry.value == value:
            return entry
    message = (
        f"{value!r} names no term kind. The kinds are "
        f"{sorted(entry.value for entry in TermKind)}."
    )
    raise LibraryError(message)


def _measure_of(value: object, objective: str) -> Measure:
    """Read the measure of one term. A term that states none reads a level."""
    if value is None:
        return Measure.LEVEL
    for entry in Measure:
        if entry.value == value:
            return entry
    message = (
        f"{value!r} names no measure of a term of the objective {objective!r}. "
        f"The measures are {sorted(entry.value for entry in Measure)}."
    )
    raise LibraryError(message)


def _aggregation_of(value: object, objective: str) -> Aggregation | None:
    """Read the aggregation of one term, which a scalar signal leaves out."""
    if value is None:
        return None
    for entry in Aggregation:
        if entry.value == value:
            return entry
    message = (
        f"{value!r} names no aggregation of a term of the objective "
        f"{objective!r}. The aggregations are "
        f"{sorted(entry.value for entry in Aggregation)}."
    )
    raise LibraryError(message)


def _styles_of(
    tables: Mapping[str, Any], declared: Sequence[str]
) -> dict[str, PlayStyle]:
    """Read every play style of the table, in declaration order."""
    if not tables:
        message = "the table declares no play style"
        raise LibraryError(message)
    styles: dict[str, PlayStyle] = {}
    for name, body in tables.items():
        _refuse_keys(body, _STYLE_KEYS, f"the style {name!r}")
        rewards = body.get("rewards")
        if not isinstance(rewards, dict) or not rewards:
            message = f"the style {name!r} rewards no objective"
            raise LibraryError(message)
        refuses = tuple(str(entry) for entry in body.get("refuses", ()))
        styles[name] = PlayStyle(
            name=name,
            weights={key: float(value) for key, value in rewards.items()},
            refuses=refuses,
            won=float(body.get("won", 0.0)),
            lost=float(body.get("lost", 0.0)),
            drawn=float(body.get("drawn", 0.0)),
            description=str(body.get("description", "")).strip(),
        )
        require_complete(styles[name], declared)
    return styles


def _refuse_keys(body: Mapping[str, Any], allowed: frozenset[str], what: str) -> None:
    """Refuse a table that holds a key this module does not read.

    A key outside the allowed set is a spelling mistake, and a spelling
    mistake that reads as a default changes the objective in silence.
    """
    extra = sorted(set(body) - allowed)
    if extra:
        message = (
            f"{what} holds {extra}, which this module does not read. The keys "
            f"are {sorted(allowed)}."
        )
        raise LibraryError(message)


__all__ = [
    "DEFAULT_LIBRARY",
    "LibraryError",
    "ObjectiveSchedule",
    "StyleLibrary",
    "Variation",
    "load_library",
    "schedule_of",
]
