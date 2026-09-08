"""The learner seat of the control plane.

A learner plays one faction while the built-in controllers play the rest.[^1]
This package holds the parts of that seat which the engine must not hold.

The reward is the first of them. What a faction should be rewarded for is a
rule of the downstream game, so the engine states none of it.[^1] A reward
inside the engine would enter the state hash, and one researcher's scoring
would then become a property of every world.

A run scores its seat in one of two ways. A weighting gives one weight to each
single-position field of the observation and collapses them into one scalar.
An objective vector gives one bounded number for each named objective, and a
play style weights that vector. **The second form trains several styles of
play from one mechanism**, because a style is a set of weights and a set of
weights is data.[^2]

References
----------
[^1]: PRD-0056, a learner plays one faction against the controllers.
``docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md``

[^2]: Report 42, what a policy should be able to see, sections 10.2 and 10.3.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
"""

from cachette.learn.env import (
    FACTION_SCOPED_READERS,
    Env,
    EnvConfig,
    StepResult,
    VectorEnv,
)
from cachette.learn.objective import (
    Measure,
    Objective,
    ObjectiveError,
    ObjectiveSet,
    ObjectiveVector,
    Term,
    TermKind,
)
from cachette.learn.presets import (
    LibraryError,
    ObjectiveSchedule,
    StyleLibrary,
    Variation,
    load_library,
    schedule_of,
)
from cachette.learn.reward import (
    OUTCOMES,
    RUNNING,
    SHAPED_ROWS,
    TERMINAL_ROWS,
    UNSET_WEIGHTING,
    Outcome,
    OutcomeReader,
    Reward,
    RewardStep,
    Scorer,
    Scoring,
    TermError,
    UnsetWeightError,
    Weighting,
)
from cachette.learn.signals import Aggregation, Signal, SignalCatalogue
from cachette.learn.style import (
    EpisodeScore,
    ObjectiveReward,
    ObjectiveScoring,
    Optimisation,
    PlayStyle,
    StyleError,
    rank_under,
)

__all__ = [
    "FACTION_SCOPED_READERS",
    "OUTCOMES",
    "RUNNING",
    "SHAPED_ROWS",
    "TERMINAL_ROWS",
    "UNSET_WEIGHTING",
    "Aggregation",
    "Env",
    "EnvConfig",
    "EpisodeScore",
    "LibraryError",
    "Measure",
    "Objective",
    "ObjectiveError",
    "ObjectiveReward",
    "ObjectiveSchedule",
    "ObjectiveScoring",
    "ObjectiveSet",
    "ObjectiveVector",
    "Optimisation",
    "Outcome",
    "OutcomeReader",
    "PlayStyle",
    "Reward",
    "RewardStep",
    "Scorer",
    "Scoring",
    "Signal",
    "SignalCatalogue",
    "StepResult",
    "StyleError",
    "StyleLibrary",
    "Term",
    "TermError",
    "TermKind",
    "UnsetWeightError",
    "Variation",
    "VectorEnv",
    "Weighting",
    "load_library",
    "rank_under",
    "schedule_of",
]
