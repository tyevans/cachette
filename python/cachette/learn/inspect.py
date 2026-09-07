"""Say what a trained policy does, verb by verb.

A score says that a policy is better. It does not say what the policy does.
This module plays a policy on a seed set and counts which verb it chose at
each decision, so a reader sees the behaviour and not only the number.

The verb of an action integer comes from the action schema of the world. The
schema is the only declaration of that layout, so nothing here holds a table
of verb numbers.
"""

from __future__ import annotations

from collections import Counter
from typing import TYPE_CHECKING

import numpy as np

from .env import Env, EnvConfig, VectorEnv
from .reward import Weighting

if TYPE_CHECKING:
    # The shape of the dictionary the engine returns. It lives in the stub
    # beside the compiled module, so importing it at run time would fail.
    from cachette._core import ActionSchema


def verb_of(schema: ActionSchema, action: int) -> str:
    """Name the verb of one action integer, through the schema."""
    for row in schema["verbs"]:
        if row["first"] <= action < row["first"] + row["rows"]:
            return str(row["name"])
    return "unknown"


def behaviour(
    config: EnvConfig,
    weighting: Weighting,
    policy: object,
    seeds: list[int],
    workers: int = 1,
) -> dict[str, object]:
    """Play the policy and report which verbs it chose, and how often.

    The report gives the share of decisions each verb took, the share the
    engine actually applied, and the outcome of each episode.
    """
    vector = VectorEnv(config, weighting, count=len(seeds), workers=workers)
    vector.reset(seeds)
    schema = vector.envs[0].world.action_schema()

    chosen: Counter[str] = Counter()
    decisions = 0
    while not vector.done:
        observations = np.stack([env.observation() for env in vector.envs])
        masks = vector.action_masks()
        actions = policy.choose_many(observations, masks)  # type: ignore[attr-defined]
        for index, action in enumerate(actions):
            if not vector.envs[index].done:
                chosen[verb_of(schema, int(action))] += 1
                decisions += 1
        vector.step(actions)

    starts = {
        row["name"]: int(row["start"])
        for row in vector.envs[0].world.observation_schema()["fields"]
    }
    return {
        "decisions": decisions,
        "verbs": {
            name: round(count / max(decisions, 1), 3)
            for name, count in chosen.most_common()
        },
        "outcomes": [env.outcome for env in vector.envs],
        "held_tiles": [
            int(env.observation()[starts["held_tiles"]]) for env in vector.envs
        ],
        "population": [
            int(env.observation()[starts["population"]]) for env in vector.envs
        ],
        "live_units": [
            int(env.observation()[starts["live_units"]]) for env in vector.envs
        ],
        "seats_held": [
            int(env.observation()[starts["seats_held"]]) for env in vector.envs
        ],
    }


__all__ = ["Env", "behaviour", "verb_of"]
