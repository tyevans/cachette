"""Train a policy by an evolution strategy, and measure what it learned.

An evolution strategy needs no gradient through the step. It scores a whole
episode with one number, so a long run with a sparse reward costs it nothing.
That fits this engine: one crossing of the boundary steps a whole population,
and each world returns one score.

# One generation is one crossing of the batch

A generation holds a population of candidate policies and a set of seeds. The
trainer builds one world for each pair of the two, steps them all through one
batch, and scores each. The batch orders its results by the index of the
world, so the score of a candidate never depends on which worker finished
first.

# The seed set is fixed inside a generation and moves between generations

Every candidate of one generation plays the same worlds. That removes the
variance that would otherwise drown a small population. The set moves at the
next generation, so a policy cannot learn one map.

# The learner-side arithmetic is float, and the engine's is not

The weights, the scores and the update are floating point. None of them
enters the world. The engine holds integers, and the only thing this module
sends it is one action integer.
"""

from __future__ import annotations

import json
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np

from .env import Env, EnvConfig, VectorEnv, viable_seeds
from .policy import LinearPolicy
from .reward import Weighting

# The observation fields a report names. Each holds one position, and each
# says something a reader of the report recognises.
REPORT_FIELDS = (
    "held_tiles",
    "seats_held",
    "live_units",
    "population",
    "store_total",
    "best_renown",
    "wonder_progress",
)


@dataclass(frozen=True)
class TrainConfig:
    """How long the training runs and how wide each generation is."""

    generations: int = 12
    population: int = 16
    seeds_per_generation: int = 4
    sigma: float = 0.08
    learning_rate: float = 0.06
    workers: int = 4
    seed: int = 0


def field_starts(env: Env) -> dict[str, int]:
    """Return the start position of each field the report names."""
    schema = env.world.observation_schema()
    return {row["name"]: int(row["start"]) for row in schema["fields"]}


def run_population(
    config: EnvConfig,
    weighting: Weighting,
    policies: list[LinearPolicy],
    seeds: list[int],
    workers: int,
) -> tuple[np.ndarray, list[dict[str, float]]]:
    """Play every policy on every seed, and return the returns and the readings.

    The world at index ``candidate * len(seeds) + seed`` belongs to that pair.
    The batch reports in index order, so the mapping holds for every step.
    """
    pairs = [(c, s) for c in range(len(policies)) for s in range(len(seeds))]
    vector = VectorEnv(config, weighting, count=len(pairs), workers=workers)
    vector.reset([seeds[s] for _, s in pairs])

    returns = np.zeros(len(pairs))
    while not vector.done:
        observations = np.stack([env.observation() for env in vector.envs])
        masks = vector.action_masks()
        actions = [0] * len(pairs)
        # Each candidate scores its own worlds. The rows of one candidate are
        # contiguous, so one matrix product answers for all of them.
        for candidate, policy in enumerate(policies):
            first = candidate * len(seeds)
            last = first + len(seeds)
            chosen = policy.choose_many(observations[first:last], masks[first:last])
            actions[first:last] = chosen
        for index, result in enumerate(vector.step(actions)):
            returns[index] += result.reward

    starts = field_starts(vector.envs[0])
    readings = []
    for env in vector.envs:
        values = env.observation()
        row = {name: float(values[starts[name]]) for name in REPORT_FIELDS}
        row["won"] = 1.0 if env.outcome == "won" else 0.0
        row["lost"] = 1.0 if env.outcome == "lost" else 0.0
        readings.append(row)
    return returns.reshape(len(policies), len(seeds)), readings


def rank_shape(scores: np.ndarray) -> np.ndarray:
    """Turn raw scores into centred ranks in the range minus a half to a half.

    A rank removes the scale of the reward from the update, so one lucky
    episode cannot move the weights further than the population is wide.
    """
    order = np.argsort(np.argsort(scores))
    return order / (len(scores) - 1) - 0.5


def train(
    name: str,
    env_config: EnvConfig,
    weighting: Weighting,
    train_config: TrainConfig,
    out_dir: Path,
    seed_pool: list[int],
) -> dict[str, object]:
    """Train one policy, and return what each generation scored."""
    probe = Env(env_config, weighting)
    policy = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    rng = np.random.default_rng(train_config.seed)
    pairs = train_config.population // 2
    history: list[dict[str, float]] = []
    started = time.time()

    for generation in range(train_config.generations):
        # A fresh seed set for each generation, taken from the pool in a
        # fixed order, so a repeat of this run takes the same worlds.
        offset = generation * train_config.seeds_per_generation
        seeds = [
            seed_pool[(offset + index) % len(seed_pool)]
            for index in range(train_config.seeds_per_generation)
        ]
        noise = rng.standard_normal((pairs, *policy.shape))
        # Antithetic sampling: each perturbation is tried in both directions,
        # so the estimate of the direction costs no extra variance from the
        # mean of the population.
        candidates = [
            LinearPolicy(policy.weights + sign * train_config.sigma * noise[index])
            for index in range(pairs)
            for sign in (1.0, -1.0)
        ]
        returns, _ = run_population(
            env_config, weighting, candidates, seeds, train_config.workers
        )
        scores = returns.mean(axis=1)
        shaped = rank_shape(scores)
        gradient = np.zeros_like(policy.weights)
        for index in range(pairs):
            weight = shaped[2 * index] - shaped[2 * index + 1]
            gradient += weight * noise[index]
        step = train_config.learning_rate / (
            train_config.population * train_config.sigma
        )
        policy = LinearPolicy(policy.weights + step * gradient)
        history.append(
            {
                "generation": generation,
                "best": float(scores.max()),
                "mean": float(scores.mean()),
                "worst": float(scores.min()),
                "seconds": round(time.time() - started, 1),
            }
        )
        print(
            f"  {name} generation {generation:2d} "
            f"mean {scores.mean():9.1f} best {scores.max():9.1f} "
            f"[{history[-1]['seconds']:.0f}s]",
            flush=True,
        )

    path = out_dir / f"{name}.npz"
    probe_world = probe
    policy.save(
        path,
        {
            "action_version": 1,
            "observation_version": 1,
            "observation_length": probe_world.observation_length,
            "action_length": probe_world.action_length,
            "width": env_config.width,
            "height": env_config.height,
            "faction_count": env_config.faction_count,
            "seat": env_config.seat,
        },
    )
    return {"name": name, "history": history, "weights": str(path)}


def evaluate(
    env_config: EnvConfig,
    weighting: Weighting,
    policy: LinearPolicy,
    seeds: list[int],
    workers: int,
) -> dict[str, float]:
    """Play one policy on a seed set, and average what it ended with."""
    returns, readings = run_population(env_config, weighting, [policy], seeds, workers)
    summary = {"return": float(returns.mean())}
    for name in (*REPORT_FIELDS, "won", "lost"):
        summary[name] = float(np.mean([row[name] for row in readings]))
    return summary


def write_report(path: Path, payload: dict[str, object]) -> None:
    """Write the run report as one JSON file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")


__all__ = [
    "REPORT_FIELDS",
    "TrainConfig",
    "asdict",
    "evaluate",
    "field_starts",
    "rank_shape",
    "run_population",
    "train",
    "viable_seeds",
    "write_report",
]
