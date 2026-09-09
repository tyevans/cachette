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

from collections.abc import Sequence
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Protocol, cast

import numpy as np

from cachette._core import Batch, World

from .policy import ActionTable
from .reward import RewardStep, Scorer, Scoring
from .signals import SignalCatalogue

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Callable, Mapping

    import numpy.typing as npt

    from cachette._core import FoundingReport, GameEnd, ObservationSchema

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
        # What the built-in controller did in one seat on the last tick. It
        # answers for one faction and reads no other.
        "controller_actions",
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


class SeatWorld(Protocol):
    """What an environment calls on a world it did not build itself.

    An environment builds its own world for a normal episode. A test hands it
    a proxy instead, to record which readers the episode reaches for, and the
    proxy is not a ``World``. This states the six calls the episode makes, so
    that the door describes what it takes.
    """

    def faction_observation(self, faction: int) -> npt.NDArray[np.int64]:
        """Give back the flat observation array of one faction."""

    def observation_schema(self) -> ObservationSchema:
        """Give back the layout of the observation array."""

    def legal_actions(self, faction: int) -> npt.NDArray[np.uint8]:
        """Give back one byte for each action row of one faction."""

    def act(self, faction: int, action: int) -> bool:
        """Run one action for one faction, and say whether the verb took it."""

    def step(self, threads: int) -> int:
        """Run the world for one tick, and give back the event count."""

    def game_end(self) -> GameEnd | None:
        """Give back how the game ended, or nothing while it runs."""


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

    def __init__(
        self,
        config: EnvConfig,
        scoring: Scoring,
        also: Mapping[str, Scoring] | None = None,
    ) -> None:
        """Build the environment. This builds no world; ``reset`` does that.

        The scoring entry is what the seat is rewarded for. A weighting over
        single fields of the observation is one such thing, and an objective
        vector under a play style is the other. **The environment tells them
        apart by nothing.** It asks either one for the scorer of a world, so
        a new way to score a run needs no change here.

        The signal catalogue names every quantity the engine publishes about
        the seat, and it comes from the schema of a probe world. **A caller
        reads an observation through it and never through a position of its
        own.** A tuple of names written by hand is a second declaration of
        what the engine publishes, and nothing fails when the two disagree.

        The action table names each verb, its block of rows and the bound of
        each argument position, and it also comes from the schema of the probe
        world. **A stored policy carries a copy of it, so that a later table
        places each row by its verb and its candidate coordinates rather than
        by its index.**[^1]

        The also entry names further scorings that read the same episode. **A
        scoring reaches no choice of the world.** The world comes from the
        configuration and the seed, the action comes from the caller, and the
        end of an episode comes from the outcome reader, which reads the
        observation of the faction and the recorded end of the game. A scoring
        therefore weights the readings of an episode and never moves it, so
        one play answers for every scoring at once.

        Each companion scorer keeps its own running state, and it reads the
        world once for each decision the primary scorer reads it. The names
        keep the order the caller gave, so a caller that combines the results
        sorts by a key it stated and never by what finished first.

        The held array is the observation the last decision built. A reset
        clears it, a decision sets it, and a finished episode answers every
        later row with it.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D1.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        self._config = config
        self._scoring = scoring
        self._also_scorings: dict[str, Scoring] = dict(also or {})
        self._also: dict[str, Scorer] = {}
        self._world: World | None = None
        self._reward: Scorer | None = None
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        self._held: np.ndarray | None = None
        # The lengths come from the schemas, and both are functions of the
        # world parameters alone. A probe world answers them once, so a
        # caller sizes a network before it runs an episode.
        #
        # **The two versions come from the same schemas.** A stored policy
        # states the version it was trained under, and the engine owns that
        # number. A constant here would be a second declaration of it, and
        # nothing would fail when the engine moved and this did not.
        probe = self._build(0)
        observation = probe.observation_schema()
        action = probe.action_schema()
        self.observation_length: int = int(observation["length"])
        self.action_length: int = int(action["length"])
        self.observation_version: int = int(observation["version"])
        self.action_version: int = int(action["version"])
        self.action_table: ActionTable = ActionTable.of_schema(action)
        self.signals: SignalCatalogue = SignalCatalogue.of_world(probe)

    @property
    def config(self) -> EnvConfig:
        """The configuration this environment runs."""
        return self._config

    @property
    def seat(self) -> int:
        """The faction the learner plays."""
        return self._config.seat

    @property
    def decisions(self) -> int:
        """How many decisions this episode has taken."""
        return self._decisions

    @property
    def terminated(self) -> bool:
        """Whether the game ended this episode."""
        return self._terminated

    @property
    def truncated(self) -> bool:
        """Whether the horizon or the tick limit ended this episode."""
        return self._truncated

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
        self._reward = self._scoring.scorer(self._world, self._config.seat)
        self._also = self._companions()
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        self._held = None
        return self.observation()

    def _companions(self) -> dict[str, Scorer]:
        """Build one companion scorer for each further scoring, in name order."""
        world = self._require_world()
        return {
            name: scoring.scorer(world, self._config.seat)
            for name, scoring in self._also_scorings.items()
        }

    @property
    def also_names(self) -> tuple[str, ...]:
        """The further scorings this environment reads, in the order given."""
        return tuple(self._also_scorings)

    def objectives_under(self, name: str | None = None) -> Mapping[str, float]:
        """Return what each objective scored under one scoring of this episode.

        A name of ``None`` asks the primary scoring. Any other name asks one
        companion, and a name the environment does not hold is refused with
        the list of the names it does hold.
        """
        if name is None:
            return self.objectives
        scorer = self._also.get(name)
        if scorer is None:
            message = (
                f"{name!r} names no scoring of this environment. It holds "
                f"{sorted(self._also_scorings)}."
            )
            raise KeyError(message)
        return scorer.objectives

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
        behind. It exists for a test, for a report written after a run, and
        for the batch, which needs the handle of every world it steps in one
        crossing.
        """
        return self._require_world()

    def adopt(self, world: SeatWorld) -> np.ndarray:
        """Take a world the caller built, and start an episode on it.

        A test uses this to hand the environment a world that watches which
        readers the environment calls. The world must already be seeded and
        must already give the seat to the caller.

        **A vector environment cannot hold what this door takes.** A batch
        steps real worlds, so the environment holds the argument as one, and
        this is the only place that widens it.
        """
        self._world = cast(World, world)
        self._reward = self._scoring.scorer(self._world, self._config.seat)
        self._also = self._companions()
        self._decisions = 0
        self._terminated = False
        self._truncated = False
        self._held = None
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

    def step(
        self, action: int, on_tick: Callable[[World], None] | None = None
    ) -> StepResult:
        """Take one decision, run the world, and score the change.

        The environment applies the action, then runs the world for the
        decision interval, then reads the reward. An action the verb refuses
        is not an error: the engine reports the refusal and the world runs
        anyway, in the way it runs for a controller whose choice fell
        through.

        The tick reader runs once after each tick of the interval, at the
        frame barrier. A caller that records what the built-in controller
        did needs it, because the engine empties the command log at the
        start of each tick. **The reader must not write the world.** It runs
        between two ticks of one decision, and a write there would land in
        the middle of a decision the caller thinks is one step.
        """
        world = self._require_world()
        if self.done:
            message = "the episode has ended. Call reset before stepping again."
            raise RuntimeError(message)

        applied = self.apply(action)
        for _ in range(self._config.decision_interval):
            world.step(self._config.threads)
            if on_tick is not None:
                on_tick(world)
        return self.settle(applied)

    def apply(self, action: int) -> bool | None:
        """Send one action to the verb, and say whether the verb took it.

        The answer is true when the verb accepted the action, and false when
        the verb refused it. The answer is absent when the seat is not
        controlled, because nothing here chose an action.

        A refusal is not an error. The engine reports it and the world runs
        anyway. A caller that measures a policy counts the refusals, because a
        policy whose actions the engine mostly refuses is close to a no-op
        whatever it chooses.
        """
        world = self._require_world()
        if not self._config.controlled:
            return None
        return world.act(self._config.seat, int(action))

    def settle(self, applied: bool | None = None) -> StepResult:
        """Close one decision: count it, read the reward, and score the change.

        The environment counts the decision, reads the reward of the seat, and
        decides whether the episode ended. It runs no tick of its own, so a
        caller that ran the world itself calls this to finish the decision.
        The batch does that: it applies every action, steps every world in one
        crossing, then closes each decision here.

        The applied entry is what ``apply`` answered. It travels into the
        result, so a caller reads the refusal of the decision it just took.

        Each companion scorer reads the same decision, and the also entry of
        the info carries what each one paid. **Only the primary scoring
        decides that the episode ended**, because the end of an episode is a
        property of the world and every scorer reads the same answer for it.
        **One decision builds the observation of its state once.** The reward
        reads the array, the outcome reader reads the array, and the result
        carries the array. Three readers of one state read one build, because
        the world does not change between them and a second build gives the
        same numbers at the same cost again. The array of a decision was
        built four times before this, and a training run spent about a fifth
        of every decision on the three builds it threw away.[^1]

        **The array in the result is the array the reward kept.** A caller
        must not write it. A caller that needs a writable array copies it,
        and stacking a batch of them copies.

        References
        ----------
        [^1]: Report 38, where the training time goes, section 10.2.
        ``docs/research/reports/38-where-the-training-time-goes.md``
        """
        world = self._require_world()
        if self._reward is None:  # pragma: no cover - reset builds both
            message = "the environment has no reward. Call reset first."
            raise RuntimeError(message)
        self._decisions += 1
        held = self.observation()
        self._held = held
        reading = self._reward.read(world, held)
        also = {
            name: scorer.read(world, held).value for name, scorer in self._also.items()
        }
        self._record_end(reading)
        return StepResult(
            observation=held,
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
                "objectives": dict(reading.objectives),
                "also": also,
                "decisions": self._decisions,
            },
        )

    def idle(self) -> StepResult:
        """Return the result of an environment whose episode already ended.

        The batch keeps every environment in index order, so a finished
        episode still reports a row. The row earns nothing and changes
        nothing, and the skipped entry of the info says so.

        No scorer reads a finished episode, so every companion earns nothing
        here as well. The also entry still holds one zero for each name, so
        a caller reads the same set of names on every row.
        **A finished episode leaves the batch, so its world stands still.**
        The array of the last decision is therefore the array of every row
        after it, and this returns that array rather than building it again.
        A pass that started 512 worlds runs its last decisions with most of
        them finished, and each of those built an array nothing read.
        """
        if self._held is None:
            self._held = self.observation()
        return StepResult(
            observation=self._held,
            reward=0.0,
            terminated=self._terminated,
            truncated=self._truncated,
            info={
                "skipped": True,
                "also": dict.fromkeys(self._also_scorings, 0.0),
            },
        )

    def _record_end(self, reading: RewardStep) -> None:
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

    @property
    def objectives(self) -> Mapping[str, float]:
        """What each objective scored over the episode so far.

        A run that scores by a weighting over single fields holds no
        objective vector, and this is then empty. A run that scores by an
        objective vector holds one value for each objective, so a report says
        which objective moved and which did not.
        """
        if self._reward is None:
            return {}
        return self._reward.objectives


def _each_scoring(
    scoring: Scoring | Sequence[Scoring], count: int
) -> tuple[Scoring, ...]:
    """Return one scoring for each index of a batch of this size.

    One scoring answers for every index. A sequence answers for one index
    each, and it must hold exactly one entry for each environment.
    """
    if isinstance(scoring, Sequence):
        held = tuple(scoring)
        if len(held) != count:
            message = f"the vector holds {count} environments and {len(held)} scorings"
            raise ValueError(message)
        return held
    return (scoring,) * count


def seats_every_faction(reports: Sequence[FoundingReport], factions: int) -> bool:
    """Say whether a founding report gives every faction a seat it can hold.

    The engine seats each faction in turn and reports the outcome of each
    one. **It refuses the whole world only when it seats nobody.** A world
    that seats one faction of three is therefore a world the engine accepts,
    and that world is over on the first tick: the seated faction holds every
    settlement, so the engine records a domination win.

    A seat also has to feed the group that takes it. The engine states that
    for each seat it filled, and the answer is a property of the site rather
    than a threshold this module invented. A faction whose site reaches no
    food starves within about a hundred ticks, whatever it does.

    Both refusals name a world where no policy can matter. The score of such
    a world is the same for every candidate of a generation, so the
    generation ranks a set of equal numbers.
    """
    if len(reports) != factions:
        return False
    if not all(bool(report.get("seated")) for report in reports):
        return False
    return all(bool(report.get("carries_its_group")) for report in reports)


def viable_seeds(config: EnvConfig, count: int, start: int = 0) -> list[int]:
    """Return the first seeds that build a world a policy can decide.

    Not every seed gives a world with a place for each faction. The engine
    refuses a world that seats nobody, and a training run that met one
    mid-generation would lose the generation.

    **A seat that the engine filled is not always a seat that can play.** The
    engine accepts a world that seats one faction of three, and that world
    ends on the first tick by domination. It also accepts a seat whose site
    reaches no food, and that faction starves within about a hundred ticks.
    In both worlds every candidate of a generation scores the same number, so
    the generation carries no information. This function therefore takes the
    founding report of each world and keeps only the seeds where every
    faction holds a seat that feeds it.

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
            reports = world.seed_world()
        except Exception:
            seed += 1
            continue
        if seats_every_faction(reports, config.faction_count):
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
        scoring: Scoring | Sequence[Scoring],
        count: int,
        workers: int = 1,
        also: Mapping[str, Scoring] | None = None,
    ) -> None:
        """Build a vector of environments over one configuration.

        The scoring entry is one scoring for every environment, or one
        scoring for each environment in index order. **A caller that varies
        the scoring gives one entry for each index and never one entry for
        each candidate.** Two candidates scored under two objectives are not
        comparable, so a rank over them carries no information.

        The also entry names further scorings that every environment reads
        beside its own. A scoring reaches no choice of the world, so one play
        answers for every scoring at once, and a caller that needs one number
        for each of several objectives plays the episodes once.
        """
        if count < 1:
            message = "a vector holds at least one environment"
            raise ValueError(message)
        scorings = _each_scoring(scoring, count)
        self._config = config
        self._scorings = scorings
        self._count = count
        self._workers = max(1, workers)
        self._envs = [Env(config, held, also) for held in scorings]
        self._batch: Batch | None = None
        self._live: list[int] = []
        # How many world-ticks this vector has run. One world stepped one
        # tick is one. **A throughput figure derived from an assumed episode
        # length misleads on a machine nobody has measured**, and this
        # project targets a platform the development boxes are not.
        self.world_ticks = 0
        self.observation_length = self._envs[0].observation_length
        self.action_length = self._envs[0].action_length
        self.observation_version = self._envs[0].observation_version
        self.action_version = self._envs[0].action_version

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
        self.world_ticks = 0
        self._batch = Batch([env.world for env in self._envs], self._workers)
        return np.stack(rows)

    def action_masks(self) -> np.ndarray:
        """Return one mask for each environment, stacked in index order."""
        return np.stack([env.action_mask() for env in self._envs])

    def step(self, actions: Sequence[int]) -> list[StepResult]:
        """Take one decision in each environment, in one crossing.

        Every environment applies its own action first and reports whether the
        verb took it. The batch then runs every world, and each environment
        reads its own reward. An environment whose episode has ended takes the
        no-op and stays where it is.

        The answer of each verb travels into the result of its own
        environment, so a caller measures how much of what a policy chose the
        engine carried out.
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
        applied = {
            index: self._envs[index].apply(int(actions[index])) for index in live
        }

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
            self._batch = Batch(
                [self._envs[index].world for index in live], self._workers
            )
            self._live = live
        self.world_ticks += len(live) * self._config.decision_interval
        for _ in range(self._config.decision_interval):
            rows = self._batch.step(self._config.threads)
            for row in rows:
                if row.error is not None:
                    failed = live[row.index]
                    message = f"the world at index {failed} refused: {row.error}"
                    raise RuntimeError(message)

        results: list[StepResult] = []
        for index, env in enumerate(self._envs):
            if index in applied:
                results.append(env.settle(applied[index]))
            else:
                results.append(env.idle())
        return results

    @property
    def done(self) -> bool:
        """Whether every episode of the vector has ended."""
        return all(env.done for env in self._envs)
