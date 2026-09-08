"""If the legality check allows an action, the verb must accept it.

Two parts of the engine decide what a faction may do. `legal_actions` returns a
mask of allowed rows. The verbs then accept or reject the row that arrives. The
mask is supposed to read the same rules the verbs read.

When the mask allows a row and the verb rejects it, that is an engine bug. This
test looks for it. ADR-0154 decision D5 asks for the test and nobody wrote
it.[^1]

Nothing could have caught it before. `World.act` already returns whether the
verb accepted the action. The single-world path keeps that answer. The batched
path, which every training run uses, throws it away.

A failure here is a bug in the engine, not in the test. The message gives the
seed, the decision number and the row, so the case can be repeated.

# References

[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D5.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.reward import Weighting

SEEDS = 6
DECISIONS = 40
NO_OP = 0

WORLD = EnvConfig(
    width=48,
    height=48,
    faction_count=3,
    seat=0,
    tick_limit=2500,
    horizon=250,
    decision_interval=10,
)

OUTCOME_ONLY = Weighting(terms={}, won=1.0, lost=0.0, drawn=0.0)


def test_the_no_op_row_is_always_legal_and_always_accepted() -> None:
    """Row zero does nothing, and both parts must always allow it."""
    env = Env(WORLD, OUTCOME_ONLY)
    for seed in viable_seeds(WORLD, 2, 70_000):
        env.reset(seed)
        for _ in range(5):
            if env.done:
                break
            assert bool(env.action_mask()[NO_OP]), "row zero must be legal"
            result = env.step(NO_OP)
            assert result.info["applied"] is True, (
                f"the verb rejected row zero on seed {seed}"
            )


def test_every_allowed_row_is_accepted() -> None:
    """Apply allowed rows across a seed set and check the verb accepts each.

    Applying a row moves the world on, so this samples states rather than
    covering them. It cycles through the allowed rows so the sample spreads
    across the table instead of repeating one row.
    """
    env = Env(WORLD, OUTCOME_ONLY)
    rejected: list[str] = []
    applied = 0
    for seed in viable_seeds(WORLD, SEEDS, 70_000):
        env.reset(seed)
        for decision in range(DECISIONS):
            if env.done:
                break
            mask = np.asarray(env.action_mask(), dtype=bool)
            allowed = np.flatnonzero(mask)
            assert allowed.size, "row zero is always legal, so the mask is never empty"
            row = int(allowed[decision % allowed.size])
            result = env.step(row)
            applied += 1
            if result.info["applied"] is not True:
                rejected.append(f"seed {seed}, decision {decision}, row {row}")
    assert applied, "the sweep applied nothing, so it compared nothing"
    assert not rejected, (
        f"the mask allowed {len(rejected)} rows of {applied} that the verb "
        f"rejected. The first few:\n" + "\n".join(rejected[:8])
    )


def test_the_check_can_fail() -> None:
    """Check the other direction, so the test can tell agreement from anything.

    A test that only applies allowed rows would pass against an engine that
    accepts every row. This finds a row the mask forbids and checks the verb
    rejects it too. If every row is legal everywhere sampled, the test proves
    nothing and says so.
    """
    env = Env(WORLD, OUTCOME_ONLY)
    for seed in viable_seeds(WORLD, SEEDS, 70_000):
        env.reset(seed)
        for _ in range(DECISIONS):
            if env.done:
                break
            mask = np.asarray(env.action_mask(), dtype=bool)
            forbidden = np.flatnonzero(~mask)
            if forbidden.size:
                row = int(forbidden[0])
                result = env.step(row)
                assert result.info["applied"] is False, (
                    f"the mask forbade row {row} on seed {seed} and the verb "
                    f"accepted it"
                )
                return
            env.step(NO_OP)
    pytest.skip("every row was legal in every state sampled, so nothing was proved")
