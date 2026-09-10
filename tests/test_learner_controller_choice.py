"""A pass whose seat the built-in controller holds asks for no choice.

The controller baseline plays the learner's own seat with the built-in
controller. The environment of such a world discards any action a caller
sends, so the score matrix that a caller builds for it reaches nothing.

That matrix is not free. A measurement on a world of 128 columns timed the
sections of the loop and found the score matrix at about a twelfth of the
wall clock of the pass, and every engine worker of the process waits while
one interpreter builds it.

These tests hold two things. A controller pass never asks a policy for a
choice, and a learner pass still does. The second is what proves that the
fixture reaches the case: the same policy refuses both calls, so a pass that
asks it fails loudly.

References
----------
[^1]: Testing Rules, sections 1 and 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from dataclasses import replace

import numpy as np
import pytest

from cachette.learn import Env, EnvConfig, Weighting
from cachette.learn.policy import LinearPolicy, RandomPolicy
from cachette.learn.rollout import run_population

# A weighting with every weight set, so the reward runs. The values are the
# test's own and they state no rule of the downstream game.
WEIGHTING = Weighting(
    terms={"held_tiles": 1.0, "population": 0.5},
    levels={"settlements": 2.0, "live_units": 0.25},
    won=100.0,
    lost=-100.0,
    drawn=0.0,
)

# A small world with a short episode. The tests drive whole decisions, so the
# cost of the fixture is the cost of the suite.
LEARNER = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=6,
    decision_interval=4,
)

# The same world with the seat given back to the built-in controller.
CONTROLLER = replace(LEARNER, controlled=False)

SEEDS = [11, 12]


class Refusing:
    """A policy that refuses to answer. Nothing may ask it for a choice.

    **Both doors raise.** The loop asks for a score matrix first and falls
    back to the choice, so a policy that answered either one would let a pass
    through the door the test means to close.
    """

    def scores_many(self, observations: np.ndarray) -> np.ndarray:
        """Refuse to score, and say who asked."""
        del observations
        message = "the pass asked this policy to score a batch"
        raise AssertionError(message)

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Refuse to choose, and say who asked."""
        del observations, masks
        message = "the pass asked this policy to choose a batch"
        raise AssertionError(message)


def test_a_controller_pass_asks_no_policy_for_a_choice() -> None:
    """The built-in controller holds the seat, so nothing needs an action."""
    played = run_population(CONTROLLER, WEIGHTING, [Refusing()], SEEDS, workers=1)

    assert played.returns.shape == (1, len(SEEDS))
    assert len(played.episodes) == len(SEEDS)


def test_a_learner_pass_still_asks_the_policy() -> None:
    """The fixture reaches the case: a learner seat consults the policy.

    Without this, a pass that asked no policy under any configuration would
    pass the test above and nothing would say so.
    """
    with pytest.raises(AssertionError, match="asked this policy"):
        run_population(LEARNER, WEIGHTING, [Refusing()], SEEDS, workers=1)


def test_a_controller_episode_does_not_depend_on_the_policy() -> None:
    """Two policies that differ everywhere play one controller episode.

    This is the property that lets the pass skip the choice. The seat belongs
    to the built-in controller, so the policy reaches no verb, no reward and
    no outcome. Every reading must agree, not only the return.
    """
    probe = Env(CONTROLLER, WEIGHTING)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    drawing = RandomPolicy(seed=7)
    silent = run_population(CONTROLLER, WEIGHTING, [zeros], SEEDS, workers=1)
    drawn = run_population(CONTROLLER, WEIGHTING, [drawing], SEEDS, workers=1)

    assert silent.ticks == drawn.ticks
    assert np.array_equal(silent.returns, drawn.returns)
    assert [row.as_row() for row in silent.episodes] == [
        row.as_row() for row in drawn.episodes
    ]
