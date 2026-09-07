"""The loop a learner drives, over one world and over many.

The parts of the learner surface each have a caller of their own. The fog
readers answer for one faction, the observation array flattens what that
faction sees, the action table bounds what it may do, the legality answer
says which rows it may take now, and the reward scores the change. **None of
those is a loop.** This module holds the loop.

# The learner sees only what a player sees

The environment calls the readers that answer for one faction, and it calls
no reader that answers from the truth of the whole world.[^1] The allowed set
is a constant of this module, and a test drives a whole episode through a
world that refuses every call outside it.

The rule matters more than it looks. A learner trained against unfogged state
learns a game nobody plays, and no result of that run says so.

# One episode is one world

A reset builds a new world. **It never reuses the world of the last
episode.** A world holds the state of its run, and a reset that reseeded it
would start the next episode inside the last one.

# The engine chooses the place, and the learner chooses the verb

An action is one integer over a bounded table.[^2] The learner sends one
integer for one faction, and the engine resolves what that verb names. The
learner never loops over units.[^3]

# References

[^1]: PRD-0001, a faction sees only what it observes.
``docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md``
[^2]: ADR-0176, an action integer is a mixed radix over the argument
positions each verb declares, decision D1.
``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
[^3]: ADR-0040, Python is a control plane, not a data plane, decision D1.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import numpy as np

from cachette._core import Batch, World

from .reward import Reward, RewardStep, Weighting

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

# The reader names the environment is allowed to call on a world.
#
# **Every name here answers for one faction, or it states a layout, or it
# states a public fact.** A name that answers from the truth of the whole
# world does not belong here, whatever it is called. The trap is
# `faction_units`, which reads like a faction reader and returns the truth.
#
# The environment calls nothing else, and a test proves it by driving a whole
# episode through a world that refuses every other name.
FACTION_SCOPED_READERS: frozenset[str] = frozenset(
    {
        # The faction reads its own observation, and the array is fog-scoped.
        "faction_observation",
        # A layout, and no state at all.
        "observation_schema",
        "action_schema",
        # The legality answer holds only what the faction observes.
        "legal_actions",
        # The verbs the faction may run. These write; they do not read out.
        "act",
        # The end of a game is a public fact, and it is recorded once.
        "game_end",
        # Construction, control and the clock.
        "seed_world",
        "set_externally_controlled",
        "set_tick_limit",
        "set_win_readers_enabled",
        "set_faction_weights",
        "step",
        # Parameters the caller itself supplied.
        "faction_count",
        "tick",
        "tick_limit",
        "width",
        "height",
        "seed",
    }
)


@dataclass(frozen=True)
class EnvConfig:
    """What one environment builds and how long it runs.

    The side entries give the world its extent. The faction count gives it
    its players, and the seat entry says which of them the learner plays. The
    tick limit ends an episode that no victory ended, and the horizon entry
    says how many decisions the learner takes in one episode.

    The decision interval is how many ticks the world runs between two
    decisions. A learner that decides on every tick spends its whole sample
    budget on ticks that changed nothing.

    The controlled entry says whether the learner takes the seat. It is true
    for a learner and for the two acting baselines. It is false for the one
    baseline that measures the built-in controller in the learner's own
    seat, which is the strongest baseline this project can state.

    **The tick limit is a rule of the game, not a truncation.** The engine
    compares held ground at the limit and records a winner, so every episode
    that reaches the limit still ends won or lost. A horizon shorter than
    the tick limit truncates instead, and a truncated episode can report no
    outcome at all.
    """

    width: int = 48
    height: int = 48
    faction_count: int = 3
    seat: int = 0
    tick_limit: int = 600
    horizon: int = 120
    decision_interval: int = 5
    threads: int = 1
    controlled: bool = True


@dataclass(frozen=True)
class StepResult:
    """What one decision returned.

    The observation entry is the flat array of the faction. The reward entry
    is what the decision earned. The terminated entry is true when the game
    ended, and the truncated entry is true when the horizon or the tick limit
    ended the episode instead. The info entry carries the reward breakdown.
    """

    observation: np.ndarray
    reward: float
    terminated: bool
    truncated: bool
    info: dict[str, Any]


class Env:
    """One world, one seat, and the loop between them.

    A caller builds the environment once, then calls ``reset`` for each
    episode and ``step`` for each decision. The shape of that loop is the
    shape a learning stack expects, so a stack drives it without an adapter
    that knows this project.
    """

    def __init__(self, config: EnvConfig, weighting: Weighting) -> None:
        """Build the environment. This builds no world; ``reset`` does that."""
        self._config = config
        self._weighting = weighting
        self._world: World | None = None
        self._reward: Reward | None = None
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        # The lengths come from the schemas, and both are functions of the
        # world parameters alone. A probe world answers them once, so a
        # caller sizes a network before it runs an episode.
        probe = self._build(0)
        self.observation_length: int = int(probe.observation_schema()["length"])
        self.action_length: int = int(probe.action_schema()["length"])

    @property
    def config(self) -> EnvConfig:
        """The configuration this environment runs."""
        return self._config

    @property
    def seat(self) -> int:
        """The faction the learner plays."""
        return self._config.seat

    def _build(self, seed: int) -> World:
        """Build one world, seed it, and give the seat to the caller."""
        config = self._config
        world = World(
            width=config.width,
            height=config.height,
            seed=seed,
            faction_count=config.faction_count,
        )
        world.seed_world()
        world.set_win_readers_enabled(True)
        world.set_tick_limit(config.tick_limit)
        # The built-in controller drives every other faction, and it leaves
        # the learner's seat alone. A configuration that is not controlled
        # leaves the seat to the built-in controller as well, which is the
        # baseline that measures the controller against itself.
        if config.controlled:
            world.set_externally_controlled(config.seat, True)
        return world

    def reset(self, seed: int) -> np.ndarray:
        """Start a new episode on a new world, and return the observation.

        **This builds a world. It never reuses the world of the last
        episode.** A reused world would carry the state of the run before it.
        """
        self._world = self._build(seed)
        self._reward = Reward(self._world, self._config.seat, self._weighting)
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        return self.observation()

    def _require_world(self) -> World:
        """Return the world of the episode, or refuse."""
        if self._world is None:
            message = "the environment has no episode. Call reset first."
            raise RuntimeError(message)
        return self._world

    @property
    def world(self) -> World:
        """The world of the episode.

        **A caller that reads this holds the whole truth of the world.** The
        environment itself calls no reader outside the faction-scoped set, and
        a caller that reaches through this property leaves that guarantee
        behind. It exists for a test and for a report written after a run.
        """
        return self._require_world()

    def adopt(self, world: World) -> np.ndarray:
        """Take a world the caller built, and start an episode on it.

        A test uses this to hand the environment a world that watches which
        readers the environment calls. The world must already be seeded and
        must already give the seat to the caller.
        """
        self._world = world
        self._reward = Reward(world, self._config.seat, self._weighting)
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        return self.observation()

    def observation(self) -> np.ndarray:
        """Return the flat observation array of the seat, as ``int64``."""
        world = self._require_world()
        return world.faction_observation(self._config.seat)

    def action_mask(self) -> np.ndarray:
        """Return one byte for each action row, one where the row is legal.

        Row zero is the no-op and it is always legal, so the mask is never
        empty and a learner never learns legality by trial.
        """
        world = self._require_world()
        return world.legal_actions(self._config.seat)

    @property
    def done(self) -> bool:
        """Whether the episode has ended, for any reason."""
        return self._terminated or self._truncated

    def step(self, action: int) -> StepResult:
        """Take one decision, run the world, and score the change.

        The environment applies the action, then runs the world for the
        decision interval, then reads the reward. An action the verb refuses
        is not an error: the engine reports the refusal and the world runs
        anyway, in the way it runs for a controller whose choice fell
        through.
        """
        world = self._require_world()
        if self._reward is None:  # pragma: no cover - reset builds both
            message = "the environment has no reward. Call reset first."
            raise RuntimeError(message)
        if self.done:
            message = "the episode has ended. Call reset before stepping again."
            raise RuntimeError(message)

        applied = (
            world.act(self._config.seat, int(action))
            if self._config.controlled
            else None
        )
        for _ in range(self._config.decision_interval):
            world.step(self._config.threads)
        self._decisions += 1

        reading = self._reward.read(world)
        self._settle(reading)
        return StepResult(
            observation=self.observation(),
            reward=reading.value,
            terminated=self._terminated,
            truncated=self._truncated,
            info={
                "applied": applied,
                "outcome": reading.outcome,
                "shaped": reading.shaped,
                "terminal": reading.terminal,
                "terms": dict(reading.terms),
                "changes": dict(reading.changes),
                "decisions": self._decisions,
            },
        )

    def _settle(self, reading: RewardStep) -> None:
        """Decide whether the episode ended, and how it ended."""
        if reading.outcome in ("won", "lost"):
            self._terminated = True
            return
        if reading.done or self._decisions >= self._config.horizon:
            self._truncated = True

    @property
    def outcome(self) -> str:
        """Name the state of the run: running, won, lost or drawn."""
        if self._reward is None:
            return "running"
        return self._reward.outcome


def viable_seeds(config: EnvConfig, count: int, start: int = 0) -> list[int]:
    """Return the first seeds that build a world every faction can sit in.

    Not every seed gives a world with a place for each faction. A seed that
    seats nobody raises, and a training run that met one mid-generation would
    lose the generation. A caller therefore takes its seeds from here.

    The search walks upward from the start, so the same start and the same
    count always give the same seeds.
    """
    found: list[int] = []
    seed = start
    while len(found) < count:
        world = World(
            width=config.width,
            height=config.height,
            seed=seed,
            faction_count=config.faction_count,
        )
        try:
            world.seed_world()
        except Exception:
            seed += 1
            continue
        found.append(seed)
        seed += 1
    return found


class VectorEnv:
    """Many worlds, one crossing of the boundary for each decision.

    The population of an evolution strategy is many episodes that differ only
    in the policy that drives them. Stepping them one at a time crosses the
    boundary once for each world, and the crossing is the part a training run
    pays for over and over. This type crosses once for the whole set.[^1]

    **The results keep the index order of the worlds.** The batch sorts by
    that index, and it never reports in the order a worker finished.

    References
    ----------
    [^1]: ADR-0155, a batch of worlds steps in one call, in index order.
    ``docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md``
    """

    def __init__(
        self,
        config: EnvConfig,
        weighting: Weighting,
        count: int,
        workers: int = 1,
    ) -> None:
        """Build a vector of environments over one configuration."""
        if count < 1:
            message = "a vector holds at least one environment"
            raise ValueError(message)
        self._config = config
        self._weighting = weighting
        self._count = count
        self._workers = max(1, workers)
        self._envs = [Env(config, weighting) for _ in range(count)]
        self._batch: Batch | None = None
        self._live: list[int] = []
        self.observation_length = self._envs[0].observation_length
        self.action_length = self._envs[0].action_length

    def __len__(self) -> int:
        """Return how many environments the vector holds."""
        return self._count

    @property
    def envs(self) -> Sequence[Env]:
        """The environments, in index order."""
        return self._envs

    def reset(self, seeds: Sequence[int]) -> np.ndarray:
        """Start one episode in each environment, and stack the observations.

        The seeds name one world for each index. A caller that scores many
        policies on one set of worlds passes the same seeds to each.
        """
        if len(seeds) != self._count:
            message = (
                f"the vector holds {self._count} environments and {len(seeds)} seeds"
            )
            raise ValueError(message)
        rows = [
            env.reset(int(seed)) for env, seed in zip(self._envs, seeds, strict=True)
        ]
        self._live = list(range(self._count))
        self._batch = Batch([env._require_world() for env in self._envs])
        return np.stack(rows)

    def action_masks(self) -> np.ndarray:
        """Return one mask for each environment, stacked in index order."""
        return np.stack([env.action_mask() for env in self._envs])

    def step(self, actions: Sequence[int]) -> list[StepResult]:
        """Take one decision in each environment, in one crossing.

        Every environment applies its own action first. The batch then runs
        every world, and each environment reads its own reward. An
        environment whose episode has ended takes the no-op and stays where
        it is.
        """
        if self._batch is None:
            message = "the vector has no episode. Call reset first."
            raise RuntimeError(message)
        if len(actions) != self._count:
            message = (
                f"the vector holds {self._count} environments "
                f"and {len(actions)} actions"
            )
            raise ValueError(message)

        live = [index for index, env in enumerate(self._envs) if not env.done]
        if self._config.controlled:
            for index in live:
                env = self._envs[index]
                env._require_world().act(env.seat, int(actions[index]))

        # One crossing for every world that is still running. **A world whose
        # episode has ended leaves the batch**, because a game resolves after
        # anything from two hundred to several thousand ticks, and a batch
        # that carried the finished worlds would spend most of a generation
        # stepping games that were already decided.
        #
        # The live list is ascending, so the batch keeps the index order of
        # the vector. The order of a result never comes from which worker
        # finished.
        if live != self._live:
            self._batch = Batch([self._envs[index]._require_world() for index in live])
            self._live = live
        for _ in range(self._config.decision_interval):
            rows = self._batch.step(self._workers, self._config.threads)
            for row in rows:
                if row.error is not None:
                    failed = live[row.index]
                    message = f"the world at index {failed} refused: {row.error}"
                    raise RuntimeError(message)

        results: list[StepResult] = []
        for index, env in enumerate(self._envs):
            if index not in live:
                results.append(
                    StepResult(
                        observation=env.observation(),
                        reward=0.0,
                        terminated=env._terminated,
                        truncated=env._truncated,
                        info={"skipped": True},
                    )
                )
                continue
            env._decisions += 1
            reading = env._reward.read(env._require_world())  # type: ignore[union-attr]
            env._settle(reading)
            results.append(
                StepResult(
                    observation=env.observation(),
                    reward=reading.value,
                    terminated=env._terminated,
                    truncated=env._truncated,
                    info={
                        "outcome": reading.outcome,
                        "shaped": reading.shaped,
                        "terminal": reading.terminal,
                        "changes": dict(reading.changes),
                    },
                )
            )
        return results

    @property
    def done(self) -> bool:
        """Whether every episode of the vector has ended."""
        return all(env.done for env in self._envs)
