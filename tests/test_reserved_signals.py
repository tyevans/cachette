"""No reward of this project reads a field the engine holds back.

The engine declares a field of the observation before it writes one. A field
it has not claimed yet carries the reserved form, holds its positions so that
a later revision fills them without moving anything, and reads zero in every
one of them. **That zero states no quantity.**

A reward term that names such a field resolves, reads zero on every decision
of every episode, and scores a constant. Nothing fails. A generation costs the
same machine time whether the term reads a quantity or a zero, so a run trains
against nothing for as long as it is left alone.[^1]

The engine publishes the form of each field beside its position, so the
reserved form is readable from the control plane. This module reads it and
compares it against every name the project rewards: the objectives of the play
style table, and the terms of the strategy table the trainer ships.

**The refusal itself lives where a term binds, and not here.** An objective set
resolves every signal it names against the catalogue of the world it plays, so
a style file a researcher writes fails there with a message that says why.
This module covers the strategy table as well, which binds through another
path, and it states the whole rule in one place a reader can find.[^2]

References
----------
[^1]: Findings register, FND-694. ``docs/FINDINGS.md``

[^2]: The signal catalogue of the control plane.
``python/cachette/learn/signals.py``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from cachette.learn.__main__ import STRATEGIES
from cachette.learn.env import Env
from cachette.learn.objective import (
    Objective,
    ObjectiveError,
    ObjectiveSet,
    Term,
    TermKind,
)
from cachette.learn.presets import load_library
from cachette.learn.reward import Weighting
from cachette.learn.signals import RESERVED_FORM
from cachette.learn.train import first_scoring

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from cachette.learn.signals import SignalCatalogue


def _catalogue() -> SignalCatalogue:
    """Build the catalogue of the world the strategy table plays.

    Every row of the table states one world, so one probe answers for all of
    them and the test builds the world once.
    """
    configs = {config for config, _, _ in STRATEGIES.values()}
    assert len(configs) == 1, "the rows of the table state more than one world"
    config = next(iter(configs))
    scoring = first_scoring(next(iter(STRATEGIES.values()))[1])
    return Env(config, scoring).signals


def _named_by_the_play_styles() -> list[tuple[str, str]]:
    """Every signal the shipped play style table reads, with its objective."""
    named: list[tuple[str, str]] = []
    for objective in load_library().objectives:
        for term in objective.terms:
            named.append((objective.name, term.signal))
            if term.against is not None:
                named.append((objective.name, term.against))
    return named


def _named_by_the_strategies() -> list[tuple[str, str]]:
    """Every signal the shipped strategy table reads, with its strategy."""
    named: list[tuple[str, str]] = []
    for name, (_, scoring, _) in STRATEGIES.items():
        weighting = _weighting_of(scoring)
        if weighting is None:
            continue
        named.extend((name, term) for term in weighting.terms)
    return named


def _weighting_of(scoring: object) -> Weighting | None:
    """Return the weighting one row of the strategy table scores under.

    A row states a weighting or a schedule of objective scorings. A schedule
    reads its signals through an objective set, which refuses a reserved
    signal where it binds, so this returns nothing for one.
    """
    if isinstance(scoring, Weighting):
        return scoring
    return None


def test_the_layout_declares_fields_the_engine_holds_back() -> None:
    """The fixture reaches the case this module guards against.

    A test that walked the tables and found no reserved field would pass on a
    layout that declared none, and it would say nothing about either table.
    This asserts that the layout holds some, so the assertions below are
    answering a live question.
    """
    catalogue = _catalogue()
    held_back = catalogue.reserved()
    assert held_back, "the layout declares no reserved field, so nothing is at risk"
    assert all(signal.form is not None for signal in held_back)
    assert {signal.form.name for signal in held_back if signal.form} == {RESERVED_FORM}


def test_no_play_style_objective_reads_a_field_the_engine_holds_back() -> None:
    """Every objective of the shipped table names a field the engine writes."""
    catalogue = _catalogue()
    reading: list[str] = [
        f"the objective {objective!r} reads {name!r}"
        for objective, name in _named_by_the_play_styles()
        if catalogue.signal(name).reserved
    ]
    assert not reading, (
        f"these objectives score a field that reads zero in every position: {reading}"
    )


def test_no_strategy_of_the_table_reads_a_field_the_engine_holds_back() -> None:
    """Every term of the shipped strategy table names a field the engine writes."""
    catalogue = _catalogue()
    reading: list[str] = [
        f"the strategy {strategy!r} reads {name!r}"
        for strategy, name in _named_by_the_strategies()
        if catalogue.signal(name).reserved
    ]
    assert not reading, (
        f"these strategies weigh a field that reads zero always: {reading}"
    )


def test_an_objective_that_reads_a_held_back_field_is_refused_where_it_binds() -> None:
    """The refusal runs at the bind, so a style file fails before an episode.

    This drives the path a run drives. An objective set is what a play style
    binds through, and it is where a name a researcher wrote meets the layout
    of the world the run plays.
    """
    catalogue = _catalogue()
    held_back = catalogue.reserved()[0]
    objective = Objective(
        name="reads_a_reserved_field",
        terms=(Term(signal=held_back.name, kind=TermKind.FIXED_POINT),),
    )
    with pytest.raises(ObjectiveError) as refusal:
        ObjectiveSet([objective], catalogue)
    assert held_back.name in str(refusal.value)
    assert "zero" in str(refusal.value)


def test_the_check_sees_a_term_that_names_a_held_back_field() -> None:
    """Put the defect back, and watch the walk over a table find it.

    The two tests above pass because both shipped tables are clean today. A
    walk that could not see a bad term would pass for that reason instead, and
    the two results are indistinguishable from the output. This builds the
    table the project must never ship and asserts that the walk names it.
    """
    catalogue = _catalogue()
    held_back = catalogue.reserved()[0]
    weighting = Weighting(terms={held_back.name: 1.0}, won=1.0, lost=-1.0, drawn=0.0)
    reading = [name for name in weighting.terms if catalogue.signal(name).reserved]
    assert reading == [held_back.name]
