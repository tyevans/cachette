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
from .policy import LinearPolicy, MLPPolicy, load_policy
from .reward import Weighting

# A policy the trainer can perturb. Both kinds answer ``flat`` and
# ``rebuild``, so the trainer never asks which kind it holds.
Policy = LinearPolicy | MLPPolicy

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
    # Sigma is a relative size. The centre has unit length and each
    # perturbation has unit length, so sigma is the fraction of the centre
    # that one candidate moves. A measurement on real decisions of a trained
    # policy fixed the working range, and it is far above the value a run
    # would reach by analogy with a gradient method.
    sigma: float = 1.5
    # The fraction of the centre that one generation moves.
    learning_rate: float = 0.3
    workers: int = 4
    seed: int = 0


def field_starts(env: Env) -> dict[str, int]:
    """Return the start position of each field the report names."""
    schema = env.world.observation_schema()
    return {row["name"]: int(row["start"]) for row in schema["fields"]}


def run_population(
    config: EnvConfig,
    weighting: Weighting,
    policies: list[object],
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
        row["drawn"] = 1.0 if env.outcome == "drawn" else 0.0
        row["unresolved"] = 1.0 if env.outcome == "running" else 0.0
        row["end_tick"] = float(values[starts["tick"]])
        readings.append(row)
    return returns.reshape(len(policies), len(seeds)), readings


def unit(vector: np.ndarray) -> np.ndarray:
    """Return the vector scaled to unit length, or the vector when it is zero.

    **A policy chooses by the highest score, and that choice does not change
    when every weight is multiplied by one positive number.** A linear policy
    scores an action row as a weighted sum, and a network policy scores it
    from a fixed projection and a second layer. Scaling the trainable weights
    scales every score by the same factor, so the row that scores highest
    stays the row that scores highest.

    The trainer uses that freedom. It holds the centre at unit length, so a
    perturbation of a fixed size is always the same fraction of the centre.
    Without it the norm of the centre grows, the same perturbation becomes a
    smaller and smaller turn, and every candidate of a generation ends up
    choosing the same actions. The population then has no spread, the ranking
    has nothing to rank, and the update becomes a walk driven by noise.

    That failure is silent. The run keeps printing generations, and the best
    score equals the mean because every candidate is the same policy.
    """
    length = float(np.linalg.norm(vector))
    if length == 0.0:
        # The first generation starts from zero. A zero centre has no
        # direction to preserve, and the perturbations supply the first one.
        return vector
    return vector / length


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
    kind: str = "linear",
    hidden: int = 32,
    resume: bool = False,
    validation: list[int] | None = None,
    validate_every: int = 3,
) -> dict[str, object]:
    """Train one policy, and return what each generation scored.

    The kind entry names the policy the run trains. A linear policy scores
    each action row from a weighted sum of the features. A network policy
    puts one hidden layer between them, which lets it state a rule that two
    features must hold together.

    **The mean of a generation is not a learning curve.** The seed set moves
    at every generation, so a mean that rises may only mean that the new
    worlds are easier. Only the held-out measurement is evidence.
    """
    path = out_dir / f"{name}.npz"
    probe = Env(env_config, weighting)

    def store(current: Policy) -> None:
        """Write the weights of the run so far, with what they were trained on.

        The trainer writes after every generation. A run that takes hours
        therefore leaves a usable policy behind when it stops early.
        """
        current.save(
            path,
            {
                "action_version": 1,
                "observation_version": 1,
                "observation_length": probe.observation_length,
                "action_length": probe.action_length,
                "width": env_config.width,
                "height": env_config.height,
                "faction_count": env_config.faction_count,
                "seat": env_config.seat,
                "tick_limit": env_config.tick_limit,
                "horizon": env_config.horizon,
                "decision_interval": env_config.decision_interval,
                "hidden": hidden if kind == "mlp" else 0,
            },
        )

    policy: Policy
    if kind == "mlp":
        policy = MLPPolicy.zeros(probe.action_length, probe.observation_length, hidden)
    else:
        policy = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    if resume and path.exists():
        # A run that continues an earlier one starts from the weights that
        # run stored. The projection of a network is a function of one fixed
        # seed, so the stored network is the network this run would build.
        stored, _ = load_policy(path)
        policy = policy.rebuild(stored.flat())
        print(f"  {name} resumes from {path}", flush=True)
    rng = np.random.default_rng(train_config.seed)
    pairs = train_config.population // 2
    history: list[dict[str, float]] = []
    started = time.time()

    # **The last generation is not the best generation.** An evolution
    # strategy walks, and a walk can end downhill. The trainer therefore
    # plays the centre on a validation seed set every few generations and
    # keeps the centre that scored highest.
    #
    # The validation seeds belong to neither the training pool nor the
    # held-out set, so keeping the best of them takes nothing from the
    # held-out measurement that the report is judged on.
    best_policy = policy
    best_score = -np.inf
    best_generation = -1

    def validate(current: Policy, generation: int) -> float | None:
        """Play the centre on the validation seeds, and return what it scored."""
        nonlocal best_policy, best_score, best_generation
        if not validation:
            return None
        scored = float(
            run_population(
                env_config, weighting, [current], validation, train_config.workers
            )[0].mean()
        )
        if scored > best_score:
            best_score, best_policy, best_generation = scored, current, generation
            store(current)
        return scored

    for generation in range(train_config.generations):
        # A fresh seed set for each generation, taken from the pool in a
        # fixed order, so a repeat of this run takes the same worlds.
        offset = generation * train_config.seeds_per_generation
        seeds = [
            seed_pool[(offset + index) % len(seed_pool)]
            for index in range(train_config.seeds_per_generation)
        ]
        centre = unit(policy.flat())
        # Each row is one direction of unit length. **The size of a
        # perturbation must not depend on the dimension or on the norm the
        # centre happened to reach.** A raw normal vector of d entries has
        # length about the square root of d, so a fixed sigma means one thing
        # for a linear policy and another for a network.
        noise = rng.standard_normal((pairs, centre.size))
        noise = noise / np.linalg.norm(noise, axis=1, keepdims=True)
        # Antithetic sampling: each perturbation is tried in both directions,
        # so the estimate of the direction costs no extra variance from the
        # mean of the population.
        candidates = [
            policy.rebuild(centre + sign * train_config.sigma * noise[index])
            for index in range(pairs)
            for sign in (1.0, -1.0)
        ]
        returns, readings = run_population(
            env_config, weighting, candidates, seeds, train_config.workers
        )
        scores = returns.mean(axis=1)
        won = float(np.mean([row["won"] for row in readings]))
        shaped = rank_shape(scores)
        gradient = np.zeros_like(centre)
        for index in range(pairs):
            weight = shaped[2 * index] - shaped[2 * index + 1]
            gradient += weight * noise[index]
        # The rank shaping already threw away the scale of the reward, so the
        # length of this sum carries no information worth keeping. The
        # trainer therefore takes a step of a fixed size along the direction,
        # and the learning rate is the fraction of the centre that one
        # generation moves.
        policy = policy.rebuild(
            unit(centre + train_config.learning_rate * unit(gradient))
        )

        # The spread of a generation is what the ranking ranks. A spread of
        # zero means every candidate chose the same actions, and the update
        # that follows it carries no information. The run reports it, so the
        # failure that killed the first attempt is visible while it happens.
        spread = float(scores.max() - scores.min())
        last = generation == train_config.generations - 1
        checked = (
            validate(policy, generation)
            if last or generation % validate_every == validate_every - 1
            else None
        )
        history.append(
            {
                "generation": generation,
                "best": float(scores.max()),
                "mean": float(scores.mean()),
                "worst": float(scores.min()),
                "spread": spread,
                "won": won,
                "validation": checked,
                "seconds": round(time.time() - started, 1),
            }
        )
        print(
            f"  {name} generation {generation:2d} "
            f"mean {scores.mean():9.1f} best {scores.max():9.1f} "
            f"spread {spread:8.1f} won {won:5.2f} "
            f"valid {'-' if checked is None else f'{checked:9.1f}'} "
            f"[{history[-1]['seconds']:.0f}s]",
            flush=True,
        )

    # The stored file already holds the best centre, because validate wrote
    # it when it found it. A run with no validation seeds keeps the last.
    if not validation:
        best_policy = policy
        store(policy)
    return {
        "name": name,
        "kind": kind,
        "history": history,
        "weights": str(path),
        "parameters": int(best_policy.flat().size),
        "best_generation": best_generation,
        "best_validation": None if best_score == -np.inf else best_score,
        "validation_seeds": list(validation or []),
    }


OUTCOME_FIELDS = ("won", "lost", "drawn", "unresolved", "end_tick")


def evaluate(
    env_config: EnvConfig,
    weighting: Weighting,
    policy: object,
    seeds: list[int],
    workers: int,
    repeats: int = 1,
) -> dict[str, float]:
    """Play one policy on a seed set, and average what it ended with.

    The repeats entry plays the seed set more than once. The engine is
    deterministic, so a repeat only changes the answer for a policy that
    draws at random. A repeat therefore narrows the random baseline, which
    is the baseline that matters.
    """
    rows: list[dict[str, float]] = []
    values: list[float] = []
    for _ in range(max(1, repeats)):
        returns, readings = run_population(
            env_config, weighting, [policy], seeds, workers
        )
        values.append(float(returns.mean()))
        rows.extend(readings)
    summary = {"return": float(np.mean(values)), "episodes": float(len(rows))}
    for name in (*REPORT_FIELDS, *OUTCOME_FIELDS):
        summary[name] = float(np.mean([row[name] for row in rows]))
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
