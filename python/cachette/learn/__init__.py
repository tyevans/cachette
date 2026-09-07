"""The learner seat of the control plane.

A learner plays one faction while the built-in controllers play the rest.[^1]
This package holds the parts of that seat which the engine must not hold.

The reward is the first of them. What a faction should be rewarded for is a
rule of the downstream game, so the engine states none of it.[^1] A reward
inside the engine would enter the state hash, and one researcher's scoring
would then become a property of every world.

References
----------
[^1]: PRD-0056, a learner plays one faction against the controllers.
``docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md``
"""

from cachette.learn.env import (
    FACTION_SCOPED_READERS,
    Env,
    EnvConfig,
    StepResult,
    VectorEnv,
)
from cachette.learn.reward import (
    OUTCOMES,
    RUNNING,
    SHAPED_ROWS,
    TERMINAL_ROWS,
    UNSET_WEIGHTING,
    Reward,
    RewardStep,
    TermError,
    UnsetWeightError,
    Weighting,
)

__all__ = [
    "FACTION_SCOPED_READERS",
    "OUTCOMES",
    "RUNNING",
    "SHAPED_ROWS",
    "TERMINAL_ROWS",
    "UNSET_WEIGHTING",
    "Env",
    "EnvConfig",
    "Reward",
    "RewardStep",
    "StepResult",
    "TermError",
    "UnsetWeightError",
    "VectorEnv",
    "Weighting",
]
