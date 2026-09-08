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

# A candidate is scored against the seats it played, not against a constant

A generation may put more than one candidate in one world. Two candidates in
one game share the map, the weather and the opponents, so the difference
between their returns holds almost none of the variance that either return
holds on its own. The trainer then ranks that difference rather than the raw
return.

**A relative score cannot say whether the population improved.** It is zero
on average by construction, so a population that got worse together reads the
same as one that got better together. The trainer therefore plays the centre
against the built-in controller on a held-out seed set as well, and reports
that absolute number beside the relative one.

**The yardstick is measured in the single-seat world, whatever the generation
played.** A candidate in a league meets another candidate and one built-in
controller. The centre on the validation seeds meets two built-in controllers,
which is the game the run is judged on. A run that measured its centre inside
its own league would move the opponent and the policy together, and no number
of that run could be compared with a number of another.

# A generation may be scored in several processes

The episodes of one generation are independent, so they could use the whole
machine. One interpreter cannot use it, because the section between two
decisions runs in one process and every engine worker of that process waits
for it. A run therefore splits the candidates across worker processes, and
each process scores the pairs it owns.

**A worker never receives a candidate.** It receives the centre, the
generation number and the pair range, and it draws the same perturbations
this module draws. It sends back the scores.

**The combination is ordered by the candidate index.** A sharded run and a
single-process run give the same weights for the same seed, at every shard
count.[^1]

# The learner-side arithmetic is float, and the engine's is not

The weights, the scores and the update are floating point. None of them
enters the world. The engine holds integers, and the only thing this module
sends it is one action integer.

# References

[^1]: ADR-0192, a generation is scored in shards and combined in candidate
order. ``docs/adrs/draft/adr-0192-a-generation-is-scored-in-shards.md``
"""

from __future__ import annotations

import json
import math
import time
from collections.abc import Mapping, Sequence
from contextlib import AbstractContextManager, nullcontext
from dataclasses import asdict, dataclass, replace
from pathlib import Path
from typing import NotRequired, TypedDict

import numpy as np

from .env import Env, EnvConfig, VectorEnv, viable_seeds
from .league import run_seated_population
from .policy import LinearPolicy, MLPPolicy, Policy, PolicyFit, load_policy
from .reward import Weighting

# How often a long call says that it is still working. A generation of the
# usual size takes several minutes, so a reader needs a line inside it. Thirty
# seconds is often enough to tell a slow generation from a stopped one, and
# rare enough to leave the log readable.
HEARTBEAT_SECONDS = 30.0

# A policy the trainer can perturb. Both kinds answer ``flat`` and
# ``rebuild``, so the trainer never asks which kind it holds.
Trainable = LinearPolicy | MLPPolicy

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
    # How many engine workers one process gives its batch. **This is a per
    # process count, whatever the shard count is.** A run of five processes
    # with this at twelve asks for sixty workers on the machine.
    workers: int = 4
    # How many worker processes score one generation. One process scores the
    # whole generation in the process that asked for it, and starts nothing.
    # **The caller states this. Nothing derives it from the core count**, and
    # a second declaration site that silently disagreed with the worker count
    # is the defect shape this project names first.
    shards: int = 1
    seed: int = 0
    # The seats a candidate may take. An empty list puts one candidate in one
    # world, in the seat the environment names, and every other seat keeps the
    # built-in controller. Two or more seats put that many candidates in one
    # world, and the trainer then scores each of them against the others.
    learner_seats: tuple[int, ...] = ()
    # Whether a seated generation ranks the margin against the other seats of
    # one world, or the raw return. This has no meaning without seated play.
    relative: bool = True
    # Whether a validating generation also plays the highest candidate on the
    # validation worlds. The centre is what the run keeps, so this measures
    # nothing the run acts on. It answers one question: whether the highest
    # of many draws on a few worlds is a real gain or the luckiest draw. That
    # question has been answered, so the pass is off unless a caller asks.
    validate_candidate: bool = False


class TrainResult(TypedDict):
    """What one training run reports when it ends.

    **The shape is declared here and nowhere else.** A caller that read this
    from a mapping of loose values would state the shape a second time, and
    the two statements would part company at the first change.

    The holdout entry is absent when the run ends. The report writer plays
    the stored centre against the held-out seeds and adds it, because only a
    held-out measurement is evidence of what the run learned.
    """

    name: str
    kind: str
    history: list[dict[str, float | None]]
    weights: str
    latest_weights: str
    parameters: int
    best_generation: int
    best_validation: float | None
    validation_seeds: list[int]
    holdout: NotRequired[dict[str, dict[str, float]]]


def field_starts(env: Env) -> dict[str, int]:
    """Return the start position of each field the report names."""
    schema = env.world.observation_schema()
    return {row["name"]: int(row["start"]) for row in schema["fields"]}


def run_population(
    config: EnvConfig,
    weighting: Weighting,
    policies: Sequence[Policy],
    seeds: list[int],
    workers: int,
    label: str = "",
) -> tuple[np.ndarray, list[dict[str, float]], int]:
    """Play every policy on every seed, and return the returns and the readings.

    The label names what is playing, for example ``conquer generation  3``. A
    call that gives one reports progress while it runs. A call that gives none
    stays silent, which is what a short call wants.

    The world at index ``candidate * len(seeds) + seed`` belongs to that pair.
    The batch reports in index order, so the mapping holds for every step.

    The third value is how many world ticks the batch ran, which the trainer
    reports as the sample cost of a generation.
    """
    pairs = [(c, s) for c in range(len(policies)) for s in range(len(seeds))]
    vector = VectorEnv(config, weighting, count=len(pairs), workers=workers)
    vector.reset([seeds[s] for _, s in pairs])

    returns = np.zeros(len(pairs))
    started = time.perf_counter()
    spoke = started
    told = 0
    decisions = 0
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
        decisions += 1

        # **A generation says it is working while it works.** A generation of
        # this size takes several minutes, and the row that reports it comes
        # only at the end. Without this line, a slow generation and a stopped
        # one look the same from outside, and a reader can only guess from the
        # load of the machine.
        #
        # **The clock decides when to print, and nothing else.** No simulated
        # value reads it, and the printing changes no state, so this cannot
        # move a result. It is not a time budget and it ends nothing.
        now = time.perf_counter()
        if label and now - spoke >= HEARTBEAT_SECONDS:
            # **The rate is what happened since the last line, not since the
            # start.** A world leaves the batch when its episode ends, so the
            # live count falls through a generation and a rate taken over the
            # whole elapsed time falls with it. That reads as a machine
            # slowing down when it is only running fewer worlds.
            window = now - spoke
            since = vector.world_ticks - told
            spoke = now
            told = vector.world_ticks
            live = sum(1 for env in vector.envs if not env.done)
            elapsed = now - started
            rate = since / window if window else 0.0
            print(
                f"  {label} working  decisions {decisions:5d} "
                f"live {live:4d}/{len(pairs):<4d} "
                f"ticks {vector.world_ticks:9d} rate {rate:8.1f} t/s "
                f"[{elapsed:.0f}s]",
                flush=True,
            )

    ticks = vector.world_ticks
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
    return returns.reshape(len(policies), len(seeds)), readings, ticks


@dataclass(frozen=True)
class Generation:
    """What one generation of candidates scored.

    The ranked entry is the quantity the update ranks. The absolute entry is
    the mean return of each candidate, which is reported whatever the update
    ranks, so that a reader sees both instruments on every generation.
    """

    ranked: np.ndarray
    absolute: np.ndarray
    won: float
    ticks: int


def score_generation(
    env_config: EnvConfig,
    weighting: Weighting,
    candidates: Sequence[Policy],
    seeds: list[int],
    train_config: TrainConfig,
    label: str = "",
) -> Generation:
    """Play one generation, and return the score the update ranks.

    A run with no learner seats plays one candidate in one world, and the
    score is the mean return over the seeds. A run with two or more learner
    seats puts that many candidates in one world, and the score is the mean
    margin against the other seats of the same world.
    """
    if not train_config.learner_seats:
        returns, readings, ticks = run_population(
            env_config, weighting, candidates, seeds, train_config.workers, label
        )
        absolute = returns.mean(axis=1)
        return Generation(
            ranked=absolute,
            absolute=absolute,
            won=float(np.mean([row["won"] for row in readings])),
            ticks=ticks,
        )
    result = run_seated_population(
        env_config,
        weighting,
        candidates,
        seeds,
        train_config.learner_seats,
        train_config.workers,
    )
    absolute = result.absolute.mean(axis=1)
    ranked = result.relative.mean(axis=1) if train_config.relative else absolute
    return Generation(
        ranked=ranked, absolute=absolute, won=result.won, ticks=result.ticks
    )


def shell_policy(kind: str, probe: Env, hidden: int) -> Trainable:
    """Build the untrained policy of one kind, sized from the world.

    **The trainer and a worker process both build this, and they must build
    the same thing.** A worker rebuilds its candidates from the centre, so it
    needs the shell the centre was taken from. The projection of a network
    comes from one fixed seed, so two processes build one projection.

    The lengths come from the engine schemas through the probe environment.
    This module states none of its own.
    """
    if kind == "mlp":
        return MLPPolicy.zeros(probe.action_length, probe.observation_length, hidden)
    return LinearPolicy.zeros(probe.action_length, probe.observation_length)


def generation_noise(seed: int, generation: int, pairs: int, size: int) -> np.ndarray:
    """Draw the perturbation of every pair of one generation.

    **The noise of a generation is a function of the generation.** A single
    stream advanced by each generation would give a resumed run different
    perturbations from the run it continues, so a resume would silently be a
    different experiment. It also lets a worker process draw the same
    perturbations the trainer draws, from the two numbers alone.

    Each row is one direction of unit length. **The size of a perturbation
    must not depend on the dimension or on the norm the centre happened to
    reach.** A raw normal vector of many entries has a length near the square
    root of that count, so a fixed sigma would mean one thing for a linear
    policy and another for a network.
    """
    rng = np.random.default_rng([seed, generation])
    noise = rng.standard_normal((pairs, size))
    return noise / np.linalg.norm(noise, axis=1, keepdims=True)


def pair_candidates(
    policy: Trainable,
    centre: np.ndarray,
    noise: np.ndarray,
    sigma: float,
    first_pair: int,
    last_pair: int,
) -> list[Trainable]:
    """Build the candidates of a range of pairs, in candidate index order.

    Antithetic sampling: each perturbation is tried in both directions, so
    the estimate of the direction costs no extra variance from the mean of
    the population. Candidate ``2 * pair`` is the plus half and
    ``2 * pair + 1`` is the minus half.

    **A worker process calls this with the pairs of its own shard.** The
    noise it passes is the whole generation's noise, so the row of a pair is
    the row that pair has in every process.
    """
    return [
        policy.rebuild(centre + sign * sigma * noise[index])
        for index in range(first_pair, last_pair)
        for sign in (1.0, -1.0)
    ]


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
) -> TrainResult:
    """Train one policy, and return what each generation scored.

    The kind entry names the policy the run trains. A linear policy scores
    each action row from a weighted sum of the features. A network policy
    puts one hidden layer between them, which lets it state a rule that two
    features must hold together.

    **The mean of a generation is not a learning curve.** The seed set moves
    at every generation, so a mean that rises may only mean that the new
    worlds are easier. Only the held-out measurement is evidence.
    """
    # **The latest centre and the best centre are two different things, and
    # one file cannot be both.** The latest is the resume point and it must
    # exist after every generation, whatever it scored. The best is what a
    # reader loads to play or to measure, and it only moves when a validation
    # pass finds something better.
    #
    # Conflating them cost this project twice. Writing only the best meant a
    # run with no validation seeds wrote nothing at all until it ended, so an
    # early stop lost everything. Writing only the latest meant the first full
    # run stored a centre taken from inside a collapsed region.
    # The shard module calls this one, so this import sits here rather than at
    # the top of the file. A module-level import would be a cycle.
    from .shard import ShardPool, run_sharded_generation

    best_path = out_dir / f"{name}.npz"
    latest_path = out_dir / f"{name}-latest.npz"
    probe = Env(env_config, weighting)

    def store(
        current: Trainable,
        target: Path,
        generation: int,
        spread: float,
        validated: float,
        best: float,
    ) -> None:
        """Write one centre, and everything needed to reason about it later.

        The generation entry says where the centre came from, so a file found
        after a crash can be placed. The spread entry says whether the search
        still had a population to rank when it stopped, which is the signal
        that the first full run lost silently.

        A weight that is absent is written as a quiet value rather than left
        out, because a file that loads with a missing key fails somewhere
        further away than the file.
        """
        current.save(
            target,
            {
                "generation": generation,
                "spread": spread,
                "validation_score": validated,
                "best_score": best,
                # **The fit states the world this file was trained against,
                # and the engine owns every number in it.** A reader refuses
                # a file whose fit is not the fit of the world it is asked
                # to play, so a policy never plays a world it never saw.
                **PolicyFit.of_env(probe).as_meta(),
                "seat": env_config.seat,
                "tick_limit": env_config.tick_limit,
                "horizon": env_config.horizon,
                "decision_interval": env_config.decision_interval,
                "hidden": hidden if kind == "mlp" else 0,
            },
        )

    policy: Trainable = shell_policy(kind, probe, hidden)
    # The generation a resumed run starts at. A run that starts fresh starts
    # at zero.
    first_generation = 0
    resumed_best = -np.inf
    if resume and latest_path.exists():
        # **A resumed run continues the run. It is not a fresh run wearing an
        # old centre.** It takes the centre, the generation counter and the
        # best score the earlier run reached, so it neither repeats the
        # generations already paid for nor overwrites a better checkpoint
        # with a worse one.
        #
        # The projection of a network is a function of one fixed seed, so the
        # stored network is the network this run would build.
        # **A resumed run must refuse a checkpoint from another world.** The
        # centre of a run is a function of one observation layout, and a
        # world of another extent can hold the same layout length while
        # meaning something else by every position of it.
        stored, meta = load_policy(latest_path, PolicyFit.of_env(probe))
        policy = policy.rebuild(np.asarray(stored.flat()))
        # A weight file states what it holds, and the reader gives back what
        # the file held. A file written by an older run can therefore be
        # missing a key, so each read names the type it needs and falls back
        # to the value a fresh run would start at.
        written = meta.get("generation")
        if isinstance(written, (int, float)):
            first_generation = int(written) + 1
        if best_path.exists():
            # **A stored best score that is not a real number means that no
            # centre has been chosen yet.** That is the state a fresh run
            # starts in, so it reads back as the value a fresh run starts
            # from. A run with no validation seeds stores the quiet value for
            # every generation, and a resumed run that took it as it stands
            # would compare every later score against a quantity that no
            # score is greater than. The best centre would then never move
            # again, the run would keep printing generations, and nothing
            # would say that the search had stopped choosing.
            score = load_policy(best_path)[1].get("best_score")
            if isinstance(score, (int, float)) and math.isfinite(score):
                resumed_best = float(score)
        print(
            f"  {name} resumes from {latest_path} at generation {first_generation}",
            flush=True,
        )
    pairs = train_config.population // 2
    history: list[dict[str, float | None]] = []
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
    best_score = resumed_best
    best_generation = -1

    # **The absolute yardstick.** A relative score is zero on average by
    # construction, so it cannot tell a population that improved from one that
    # got worse together. The built-in controller plays the learner's own seat
    # on the validation seeds, and the run reports every validation score
    # against that number. The controller does not learn, so the yardstick is
    # measured once and holds for the whole run.
    yardstick: float | None = None
    if validation:
        yardstick = float(
            run_population(
                replace(env_config, controlled=False),
                weighting,
                [policy],
                validation,
                train_config.workers,
                f"{name} yardstick",
            )[0].mean()
        )
        print(f"  {name} controller yardstick {yardstick:9.1f}", flush=True)

    def score_on_validation(current: Trainable, label: str, seeds: list[int]) -> float:
        """Return what one policy scores on the validation seeds.

        **This chooses nothing.** It plays the seeds and reports. The caller
        that keeps the best centre is `validate` below, and it is the only
        one that may move the best. A candidate must never replace the centre,
        because a candidate is the highest of many draws on a few seeds and
        the highest draw is usually the luckiest one.

        The caller passes the seeds, because the only caller has already
        checked that there are some.
        """
        return float(
            run_population(
                env_config,
                weighting,
                [current],
                seeds,
                train_config.workers,
                label,
            )[0].mean()
        )

    def validate(current: Trainable, generation: int) -> float | None:
        """Play the centre on the validation seeds, and return what it scored."""
        nonlocal best_policy, best_score, best_generation
        if not validation:
            return None
        scored = float(
            run_population(
                env_config,
                weighting,
                [current],
                validation,
                train_config.workers,
                f"{name} validation {generation:2d}",
            )[0].mean()
        )
        if scored > best_score:
            best_score, best_policy, best_generation = scored, current, generation
        return scored

    # **The pool decides whether a generation is sharded, and the shard count
    # decides whether there is a pool.** One process opens nothing and scores
    # the generation here, which is the path every earlier run took.
    #
    # The pool stays open for the whole run, so a generation pays no process
    # start cost, and the matrix thread variables it sets hold for as long as
    # a worker might start.
    opened: AbstractContextManager[ShardPool | None]
    if train_config.shards > 1:
        opened = ShardPool(train_config.shards)
        print(
            f"  {name} scores each generation in {train_config.shards} processes "
            f"of {train_config.workers} workers",
            flush=True,
        )
    else:
        opened = nullcontext(None)

    with opened as pool:
        for generation in range(first_generation, train_config.generations):
            # A fresh seed set for each generation, taken from the pool in a
            # fixed order, so a repeat of this run takes the same worlds.
            offset = generation * train_config.seeds_per_generation
            seeds = [
                seed_pool[(offset + index) % len(seed_pool)]
                for index in range(train_config.seeds_per_generation)
            ]
            centre = unit(policy.flat())
            noise = generation_noise(train_config.seed, generation, pairs, centre.size)
            label = f"{name} generation {generation:2d}"
            # **A worker process builds its own candidates from the centre and
            # the generation number.** Only the centre crosses to it, so this
            # process builds the population only when it plays the population
            # itself.
            if pool is None:
                played = score_generation(
                    env_config,
                    weighting,
                    pair_candidates(
                        policy, centre, noise, train_config.sigma, 0, pairs
                    ),
                    seeds,
                    train_config,
                    label,
                )
            else:
                played = run_sharded_generation(
                    env_config,
                    weighting,
                    train_config,
                    seeds,
                    generation,
                    centre,
                    pool,
                    kind,
                    hidden,
                    label,
                )
            scores, ticks, won = played.ranked, played.ticks, played.won
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
            # The spread of the raw return is reported beside the spread of the
            # ranked score, because the two answer different questions and a run
            # that reported one of them could not be compared with the other.
            absolute_spread = float(played.absolute.max() - played.absolute.min())
            last = generation == train_config.generations - 1
            validating = last or generation % validate_every == validate_every - 1
            checked = validate(policy, generation) if validating else None

            # **The highest candidate of a generation is the highest of many
            # draws on a few seeds, so it is usually the luckiest and not the
            # best.** The reported best therefore says nothing about whether
            # the population found a policy the centre should move toward.
            # Playing that candidate on the validation seeds says it: a
            # candidate that holds its score there is a real gain the centre
            # is not taking, and one that falls back to the score of the
            # centre was luck.
            #
            # This chooses nothing. The stored best centre is decided by
            # `validate` above and by nothing here.
            candidate_checked: float | None = None
            if (
                train_config.validate_candidate
                and validating
                and validation
                and checked is not None
            ):
                highest = int(np.argmax(played.absolute))
                candidate_checked = score_on_validation(
                    pair_candidates(
                        policy, centre, noise, train_config.sigma, 0, pairs
                    )[highest],
                    f"{name} candidate {generation:2d}",
                    list(validation),
                )
                print(
                    f"  {name} generation {generation:2d} "
                    f"candidate {highest:4d} scored "
                    f"{played.absolute[highest]:9.1f} on its own seeds and "
                    f"{candidate_checked:9.1f} on the validation seeds, "
                    f"where the centre scored {checked:9.1f}",
                    flush=True,
                )

            # **The latest centre is written every generation, unconditionally.**
            # No validation gate and no improvement gate. This is the resume
            # point, and a run that stops between two validation passes must
            # still leave one behind.
            quiet = float("nan")
            store(
                policy,
                latest_path,
                generation,
                spread,
                quiet if checked is None else checked,
                quiet if best_score == -np.inf else best_score,
            )
            # The best centre moves only when a validation pass finds something
            # better. A run with no validation seeds has no way to tell one
            # centre from another, so its latest centre is also its best.
            if not validation or best_generation == generation:
                store(
                    best_policy,
                    best_path,
                    best_generation if validation else generation,
                    spread,
                    quiet if checked is None else checked,
                    quiet if best_score == -np.inf else best_score,
                )
            history.append(
                {
                    "generation": generation,
                    "best": float(scores.max()),
                    "mean": float(scores.mean()),
                    "worst": float(scores.min()),
                    "spread": spread,
                    "absolute_spread": absolute_spread,
                    "absolute_mean": float(played.absolute.mean()),
                    "world_ticks": ticks,
                    "won": won,
                    "validation": checked,  # may be None on a generation that skips it
                    # What the centre scored above the built-in controller on the
                    # same seeds. This is the number that says whether the whole
                    # population improved, and a relative score cannot say it.
                    "yardstick": yardstick,
                    "above_controller": (
                        None
                        if checked is None or yardstick is None
                        else checked - yardstick
                    ),
                    "seconds": round(time.time() - started, 1),
                }
            )
            print(
                f"  {name} generation {generation:2d} "
                f"mean {scores.mean():9.1f} best {scores.max():9.1f} "
                f"spread {spread:8.1f} abs-spread {absolute_spread:8.1f} "
                f"won {won:5.2f} "
                f"ticks {ticks} "
                f"valid {'-' if checked is None else f'{checked:9.1f}'} "
                f"[{history[-1]['seconds']:.0f}s]",
                flush=True,
            )

    return {
        "name": name,
        "kind": kind,
        "history": history,
        "weights": str(best_path),
        "latest_weights": str(latest_path),
        "parameters": int(best_policy.flat().size),
        "best_generation": best_generation,
        "best_validation": None if best_score == -np.inf else best_score,
        "validation_seeds": list(validation or []),
    }


OUTCOME_FIELDS = ("won", "lost", "drawn", "unresolved", "end_tick")


def evaluate(
    env_config: EnvConfig,
    weighting: Weighting,
    policy: Policy,
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
        returns, readings, _ = run_population(
            env_config, weighting, [policy], seeds, workers
        )
        values.append(float(returns.mean()))
        rows.extend(readings)
    summary = {"return": float(np.mean(values)), "episodes": float(len(rows))}
    for name in (*REPORT_FIELDS, *OUTCOME_FIELDS):
        summary[name] = float(np.mean([row[name] for row in rows]))
    return summary


def write_report(path: Path, payload: Mapping[str, object]) -> None:
    """Write the run report as one JSON file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")


__all__ = [
    "REPORT_FIELDS",
    "Generation",
    "TrainConfig",
    "TrainResult",
    "asdict",
    "evaluate",
    "field_starts",
    "generation_noise",
    "pair_candidates",
    "rank_shape",
    "run_population",
    "score_generation",
    "shell_policy",
    "train",
    "viable_seeds",
    "write_report",
]
