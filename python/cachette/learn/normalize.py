"""The reference sample one feature normalizer comes from.

The encoder standardizes each squashed position against a centre and a scale,
and both come from a sample of played observations.[^1] This module derives
that sample and caches it.

# The derivation is a function of the world and of nothing else

A normalizer is a function of the world configuration and of the observation
version. Two derivations must therefore give the same two arrays, byte for
byte, so every input of this module is fixed: the seed list, the generator
that draws the action of each decision, and the decision count.

**A run derives it once and gives the same one to every worker process and
to every generation.** Two candidates of one generation that read two
normalizers are not comparable, so a rank over them says nothing about
either policy. The worker takes the arrays in its task rather than deriving
them again.

# The seat plays at random

The sample must cover what a policy meets, so the seat takes a legal action
drawn uniformly at each decision. A seat that took the no-op would supply the
observations of a faction that does nothing, and the distribution of a
faction that does nothing is not the distribution a policy is trained on.[^2]

The draw of one episode is keyed on the seed of that episode, so a seed that
the engine refuses drops out of the sample and moves no other episode.

# The scoring reaches nothing here

The derivation builds an environment, and an environment needs a scoring.
**The scoring reaches no choice of the world**: the world comes from the
configuration and the seed, and the action comes from the generator. The
arrays therefore do not depend on what the run rewards, and the cache is
keyed on the configuration alone.

References
----------
[^1]: The feature normalizer. ``python/cachette/learn/policy.py``

[^2]: Testing rules, a fixture supplies the input.
``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np

from .env import Env, viable_seeds
from .policy import FeatureNormalizer

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from .env import EnvConfig
    from .reward import Scoring

# How many episodes the reference sample plays. Each one is a world of its
# own, so this is the count of worlds the centre and the scale average over.
REFERENCE_EPISODES = 12

# How many decisions the sample takes in one episode. An episode that ends
# earlier contributes what it reached.
REFERENCE_DECISIONS = 60

# The seed the action draw of each episode is keyed on. The key is this
# number beside the seed of the episode, so one refused seed moves no other
# episode of the sample.
REFERENCE_DRAW_SEED = 20260909

# Where the search for viable seeds starts. The search walks upward, so this
# and the episode count name one seed list for one world.
REFERENCE_SEED_START = 0

# The normalizers this process has derived, keyed on every input of the
# derivation. **A worker process takes the arrays in its task and never
# reaches this**, so the cache holds one entry for one run.
_HELD: dict[tuple[object, ...], FeatureNormalizer] = {}


def reference_observations(
    config: EnvConfig,
    scoring: Scoring,
    episodes: int = REFERENCE_EPISODES,
    decisions: int = REFERENCE_DECISIONS,
    draw_seed: int = REFERENCE_DRAW_SEED,
    seed_start: int = REFERENCE_SEED_START,
) -> np.ndarray:
    """Play the reference sample, and return one raw observation row for each decision.

    The seeds are the first viable seeds above the start, so the same
    configuration always gives the same worlds. The seat takes a legal action
    drawn uniformly at each decision, and the draw of one episode is keyed on
    the seed of that episode.

    An episode whose game ends before the decision count contributes the rows
    it reached. A seed the engine refuses to generate a world for contributes
    nothing and moves nothing else.

    Raises ``ValueError`` when the whole sample reached no decision. A
    normalizer over no rows states nothing.
    """
    seeds = viable_seeds(config, episodes, seed_start)
    env = Env(config, scoring)
    rows: list[np.ndarray] = []
    for seed in seeds:
        draws = np.random.default_rng([draw_seed, seed])
        try:
            env.reset(seed)
        except Exception:
            continue
        for _ in range(decisions):
            if env.done:
                break
            rows.append(np.asarray(env.observation(), dtype=np.int64))
            legal = np.flatnonzero(env.action_mask())
            env.step(int(draws.choice(legal)))
    if not rows:
        message = (
            f"the reference sample of {episodes} episodes above seed "
            f"{seed_start} reached no decision, so nothing can be derived "
            "from it"
        )
        raise ValueError(message)
    return np.stack(rows)


def derive_normalizer(
    config: EnvConfig,
    scoring: Scoring,
    episodes: int = REFERENCE_EPISODES,
    decisions: int = REFERENCE_DECISIONS,
    draw_seed: int = REFERENCE_DRAW_SEED,
    seed_start: int = REFERENCE_SEED_START,
) -> FeatureNormalizer:
    """Derive the normalizer of one world from the reference sample.

    Two calls with one configuration give the same two arrays, byte for byte.
    **This plays episodes, so it costs what those episodes cost.** A caller
    that needs it more than once takes the cached door instead.
    """
    return FeatureNormalizer.of_observations(
        reference_observations(
            config, scoring, episodes, decisions, draw_seed, seed_start
        )
    )


def reference_normalizer(
    config: EnvConfig,
    scoring: Scoring,
    episodes: int = REFERENCE_EPISODES,
    decisions: int = REFERENCE_DECISIONS,
    draw_seed: int = REFERENCE_DRAW_SEED,
    seed_start: int = REFERENCE_SEED_START,
) -> FeatureNormalizer:
    """Give the normalizer of one world, deriving it once for this process.

    The cache is keyed on the configuration and on every constant of the
    derivation. **The scoring is not in the key**, because a scoring reaches
    no choice of the world and therefore no observation the sample holds.
    """
    key: tuple[object, ...] = (config, episodes, decisions, draw_seed, seed_start)
    held = _HELD.get(key)
    if held is None:
        held = derive_normalizer(
            config, scoring, episodes, decisions, draw_seed, seed_start
        )
        _HELD[key] = held
    return held


def forget_normalizers() -> None:
    """Empty the cache of this process.

    A test that measures the cost of a derivation needs the state a fresh
    process starts in.
    """
    _HELD.clear()


__all__ = [
    "REFERENCE_DECISIONS",
    "REFERENCE_DRAW_SEED",
    "REFERENCE_EPISODES",
    "REFERENCE_SEED_START",
    "derive_normalizer",
    "forget_normalizers",
    "reference_normalizer",
    "reference_observations",
]
