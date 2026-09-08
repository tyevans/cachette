"""The survival term must not make a slow loss beat a quick win.

**The search ranks candidates and never scores them**, so what matters is
whether a losing candidate can ever sort above a winning one. The outcome term
separates the two classes by twice the win weight. Every shaped term the
strategy carries pays into that gap, and once the shaped terms can cover it the
order inverts and the run learns to lose slowly.

The bound is arithmetic and it is checkable, so it is checked here rather than
stated in a comment beside the weight.
"""

from __future__ import annotations

import pytest

from cachette.learn.__main__ import (
    STRATEGIES,
    SURVIVAL,
    TICK_LIMIT,
    WIN,
    WORLD,
)

# The strategies this rule governs, and the ceiling of each shaped term over
# one episode. A term whose ceiling is not stated here is not covered, so a new
# term in one of these strategies fails the completeness test below.
CEILINGS: dict[str, float] = {
    # The ground term pays its weight for each tile the seat holds, and the
    # world holds this many tiles in total.
    "held_tiles": float(WORLD.width * WORLD.height),
    # The survival term pays its weight for each tick, up to the limit.
    "tick": float(TICK_LIMIT),
}


def outcome_gap() -> float:
    """Return the distance the outcome term puts between a win and a loss."""
    return 2.0 * WIN


@pytest.mark.parametrize("name", ["land-hold-net"])
def test_the_shaped_terms_cannot_cover_the_outcome_gap(name: str) -> None:
    """A candidate that lost must not be able to outrank one that won."""
    _, weighting, _ = STRATEGIES[name]
    highest = sum(
        abs(weight) * CEILINGS[term]
        for term, weight in weighting.terms.items()
        if weight is not None
    )
    assert highest < outcome_gap(), (
        f"the shaped terms of {name!r} pay up to {highest} over one episode "
        f"and a win stands only {outcome_gap()} above a loss, so a candidate "
        f"that lost slowly can outrank one that won"
    )


@pytest.mark.parametrize("name", ["land-hold-net"])
def test_every_shaped_term_states_a_ceiling(name: str) -> None:
    """A term with no stated ceiling is a term this rule does not check."""
    _, weighting, _ = STRATEGIES[name]
    unstated = sorted(set(weighting.terms) - set(CEILINGS))
    assert not unstated, (
        f"{name!r} weighs {unstated}, and this test states no ceiling for "
        f"them, so it cannot say whether the outcome still decides the order"
    )


def test_the_bound_can_fail() -> None:
    """Prove the check above can fail, by giving it a weight that crosses.

    A bound that no input violates is decoration. The weight below is the
    smallest multiple of the shipped one that covers the gap.
    """
    ground = CEILINGS["held_tiles"]
    crossing = (outcome_gap() - ground) / CEILINGS["tick"]
    highest = ground + crossing * CEILINGS["tick"]
    assert highest == pytest.approx(outcome_gap())
    assert SURVIVAL < crossing, (
        f"the shipped survival weight is {SURVIVAL} and the weight that "
        f"covers the gap is {crossing}, so the shipped one must be smaller"
    )
