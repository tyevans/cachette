"""The land-hold rows pay for held ground and for the clock together.

The ground rows of the strategy table pay for held tiles, for settlements and
for the readiness to found. They state no early weight, so a win at the first
tick and a win at the tick limit pay the same.

A measurement showed what that costs. A run that counted the tick limit as a
loss reached the limit in almost every game against a strong seat, and every
candidate paid the same flat loss. The search then ranked nothing.

The land-hold rows carry the ground scoring with the early weight added. A run
of these rows states no limit rule, so the engine decides a game on held ground
at the limit. The early weight then pays a fast win more than a slow one.

**The ground rows must not move.** A stored reference policy names the
land-structured strategy and was trained against the ground weighting. A change
to that weighting would change what the stored policy was measured against, and
nothing would fail.
"""

from __future__ import annotations

from dataclasses import replace

from cachette.learn.__main__ import EARLY, STRATEGIES
from cachette.learn.reward import Weighting
from cachette.learn.structured import STRUCTURED_KIND
from cachette.learn.train import first_scoring

# The new rows, and the row of the old pair each one answers to.
PAIRS = (("land", "land-hold"), ("land-structured", "land-hold-structured"))


def _weighting(name: str) -> Weighting:
    """Return the weighting of one row, and refuse a row that holds none."""
    assert name in STRATEGIES, f"the table holds no row named {name}"
    weighting = first_scoring(STRATEGIES[name][1])
    assert isinstance(weighting, Weighting), f"{name} states no weighting"
    return weighting


def test_the_table_holds_the_two_land_hold_rows() -> None:
    """The table names both rows, and each takes the kind of its sibling.

    The table pairs a linear row with a structured row for every scoring it
    holds. A new scoring that reached one kind alone could not say whether the
    policy or the scoring carried a result.
    """
    assert STRATEGIES["land-hold"][2] == "linear"
    assert STRATEGIES["land-hold-structured"][2] == STRUCTURED_KIND
    for old, new in PAIRS:
        assert STRATEGIES[new][2] == STRATEGIES[old][2], (
            f"{new} names a kind other than the kind of {old}"
        )
        assert STRATEGIES[new][0] == STRATEGIES[old][0], (
            f"{new} plays a world other than the world of {old}"
        )


def test_the_land_hold_rows_differ_from_the_ground_rows_by_the_early_weight() -> None:
    """Every level and every outcome weight is the weight the ground row states.

    This compares the whole weighting rather than the fields it names, so a
    weight added to one row and not to the other fails here.
    """
    for old, new in PAIRS:
        ground = _weighting(old)
        holding = _weighting(new)
        assert holding.won_early == EARLY, (
            f"{new} pays {holding.won_early} for the time left on the clock"
        )
        assert replace(holding, won_early=ground.won_early) == ground, (
            f"{new} differs from {old} by more than the early weight"
        )
        assert dict(holding.levels) == dict(ground.levels)
        assert dict(holding.terms) == dict(ground.terms)
        assert (holding.won, holding.lost, holding.drawn) == (
            ground.won,
            ground.lost,
            ground.drawn,
        )


def test_the_ground_rows_still_state_no_early_weight() -> None:
    """The row the stored reference trained against is unchanged.

    The published reference policy names the land-structured strategy. An early
    weight added to the ground weighting would change the scoring that policy
    was trained and measured under.
    """
    for old, _new in PAIRS:
        assert _weighting(old).won_early == 0.0, (
            f"{old} now pays for the time left on the clock, so the stored "
            f"reference was measured under a scoring the table no longer holds"
        )


def test_the_land_hold_rows_pay_the_early_weight_their_siblings_pay() -> None:
    """One early weight serves every row that states one.

    A second figure here would be a second declaration of the weight, and the
    two could part without anything failing.
    """
    for name in ("conquer", "wonder", "renown"):
        assert _weighting(name).won_early == EARLY
    for _old, new in PAIRS:
        assert _weighting(new).won_early == EARLY
