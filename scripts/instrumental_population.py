"""Ask whether population rises where nothing rewards it.

The project owner proposes that population is the universal instrumental
good: the quantity every training objective has to learn, whether or not its
reward names it. Three of the four objectives of this run reward held ground
and one of them rewards people, so the claim is testable. If population rises
under the three objectives that never pay for it, the claim holds. If
population rises only under the objective that pays for it, each objective is
finding its own road and the claim is wrong.

# Two engine facts make the claim plausible, and both were read

The engine eliminates a faction when it holds zero settlements and zero
population, and only after that faction took a seat.[^1] Population therefore
buys survival directly, under every objective.

The engine holds four win paths: domination, territory at the tick limit, a
finished wonder, and a renown target.[^2] **Population is none of them.** So
a rise in population under an objective that does not reward it is
instrumental and never direct.

# A rise is measured against a baseline, and the baseline is the controller

An absolute population is not evidence. Every faction that survives holds
people, so a policy that merely has population has shown nothing. The
evidence is a policy that reaches more population than the built-in
controller reached in the same seat on the same seeds. This tool therefore
plays the controller over the held-out seed set as the yardstick, in the way
the training run already does, and reports every arm as a difference against
it.[^3]

The seeds are shared, so the comparison is paired. The tool reports the mean
difference and the count of seeds on which the arm exceeded the controller.
A paired count over a small seed set says more than a mean difference does,
because one runaway seed cannot carry it.

# Every quantity is read through the schema

The engine owns the layout of the observation and states it in a schema. The
signal catalogue is the one reader of that schema, and this tool reads every
quantity through it.[^4] The report carries every one-position signal the
world published, so a later reader holds what the engine published rather
than the subset one caller thought to name.

The tool names three quantities in its printed table: the two the survival
rule names, and held ground, which separates land from people. Each of the
three is resolved against the catalogue, so a name the engine stopped
publishing fails here with the list of what the engine does publish. It is
not read by position and it is not read from a stored list.

The quantity each objective rewards comes from the objective itself. A
weighting names its terms, and this tool reads those names, so a rise in a
rewarded quantity can be told apart from a rise in an unrewarded one.

# A published value is bounded, so a difference is not a count

**The observation publishes no raw count.** Every position of it is a bounded
number in the closed interval from minus one to one, held as a Q16.16
integer. A quantity with a named denominator crosses as a share. A quantity
with no denominator crosses as a compressed magnitude, which is the
base-two logarithm of one plus the count, over a fixed bit cap. Population,
settlements and held ground each cross in the second form.

**The schema states the bounds of each field and not the form.** It gives a
low and a high, and it names neither the share nor the compressed magnitude.
A reader therefore cannot turn a published value back into a count without a
second declaration of the form, and this tool refuses to hold one.

The compression is monotone in the count, so it keeps the order. **The count
of the seeds on which an arm read higher than the controller is therefore
exact, and the size of a mean difference is not in people.** Read the paired
count as the evidence, and read a mean as the published value it is.

# The end of a game comes from the game end record

The observation of a faction publishes no tick and no win path. It publishes
the ticks that remain before the limit fires, which is a bounded value. The
tick of the end and the path that ended the game are public facts the world
records once, and this tool reads them from that record.[^5]

**A report row spelled the tick of the end from a signal named ``tick``, and
no world publishes a signal of that name.** Such a row reads zero for every
episode. This tool takes the tick of the end from the record, and it takes
the tick of an unresolved game from the clock of the world.

# What the win path distribution is worth on its own

Nobody has measured the win path distribution of the world this run plays.
The balance harness measured it at extent 256 with four factions at tick
limits of 5000 and 20000, and it found that a territory win at 5000 ticks is
a truncated renown win.[^6] The training run plays extent 48 with three
factions at a limit of 2500, and no measurement covers that world. If most
games end at the limit on territory rather than by domination, that reframes
what every objective competes for.

The end tick distribution answers a separate question. The run sets a limit
of 2500 ticks, and a shorter limit buys samples if games end before it. A
distribution of where games actually end answers whether a shorter limit
loses anything.

# What this tool cannot run today

The action table gained a place argument, so the engine now writes action
version 2. Every weight file this project holds states version 1, and the
checkpoint directory holds no weight file at all. A stored policy that does
not fit the world is refused by name, so the measurement cannot run against
a trained policy until a run writes weights at the current version.

What runs today is the controller and an untrained policy of each kind. That
proves the reading path end to end, and it measures the win path
distribution and the end tick distribution, which need no trained policy.

# References

[^1]: The elimination rule. ``crates/cachette-core/src/world/sites.rs``
[^2]: The win paths. ``crates/cachette-core/src/controller.rs``
[^3]: The controller baseline. ``python/cachette/learn/baseline.py``
[^4]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^5]: ADR-0148, a game end is recorded once and stops the controllers,
decision D1.
``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``
[^6]: Budgets and costs, the win-path share row. ``docs/reference/balance.md``
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import TYPE_CHECKING, Any

import numpy as np

from cachette.learn.env import Env, EnvConfig, VectorEnv, viable_seeds
from cachette.learn.policy import (
    LinearPolicy,
    Policy,
    PolicyFit,
    RandomPolicy,
    load_policy,
)
from cachette.learn.record import end_tick_of
from cachette.learn.reward import Weighting
from cachette.learn.structured import STRUCTURED_KIND, StructuredPolicy

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterable, Mapping, Sequence

    from cachette.learn.reward import Scoring
    from cachette.learn.signals import SignalCatalogue

# The quantities the printed table always shows. The first two are the pair
# the elimination rule names, and the third separates ground from people.
# **Each name is resolved against the schema before it is read**, so a name
# the engine stopped publishing fails with the list of what it does publish.
POPULATION_SIGNAL = "population"
SETTLEMENT_SIGNAL = "settlements"
GROUND_SIGNAL = "held_tiles"
SURVIVAL_SIGNALS: tuple[str, ...] = (POPULATION_SIGNAL, SETTLEMENT_SIGNAL)

# The name the report gives an episode that no win reader ended. The engine
# records no path for such a game, and the report needs a column for it.
NO_PATH = "none"

# Where the held-out seed search starts. The behaviour report of the training
# run reads from the same start, so the two measurements share worlds.
HELDOUT_START = 50_000


@dataclass(frozen=True)
class Reading:
    """What one episode of one arm ended at.

    The signals entry holds every one-position quantity the engine published
    about the seat at the last decision, under the name the engine gives it.
    The path entry names the win reader that ended the game, and it is the
    absent-path name when no reader fired. The end tick is the tick the
    record states, or the clock of the world when no record was written.
    """

    arm: str
    seed: int
    outcome: str
    decisions: int
    path: str
    end_tick: int
    winner: int | None
    reached_limit: bool
    signals: Mapping[str, float]

    def as_dict(self) -> dict[str, Any]:
        """Return this reading as plain values, for a report file."""
        return {
            "arm": self.arm,
            "seed": self.seed,
            "outcome": self.outcome,
            "decisions": self.decisions,
            "path": self.path,
            "end_tick": self.end_tick,
            "winner": self.winner,
            "reached_limit": self.reached_limit,
            "signals": dict(self.signals),
        }


@dataclass(frozen=True)
class Arm:
    """One thing that plays the seat, and the objective that shaped it.

    The objective entry names the row of the strategy table the policy was
    trained under, or nothing for a baseline that no objective shaped. The
    rewarded entry holds the signals that objective pays for, so a report
    tells a rise in a rewarded quantity from a rise in an unrewarded one.
    """

    name: str
    config: EnvConfig
    policy: Policy
    objective: str | None = None
    rewarded: tuple[str, ...] = ()


@dataclass(frozen=True)
class Spread:
    """Where a set of numbers sits, without assuming a shape for it.

    A mean alone hides a distribution that piles up at one end, and the end
    tick of a game does exactly that when most games reach the limit. The
    quartiles say so and the mean does not.
    """

    count: int
    lowest: float
    lower_quarter: float
    middle: float
    upper_quarter: float
    highest: float
    mean: float

    @classmethod
    def of(cls, values: Sequence[float]) -> Spread:
        """Return the spread of a set of numbers, or refuse an empty set."""
        if not values:
            message = "a spread needs at least one number"
            raise ValueError(message)
        held = np.asarray(values, dtype=float)
        return cls(
            count=len(values),
            lowest=float(held.min()),
            lower_quarter=float(np.quantile(held, 0.25)),
            middle=float(np.quantile(held, 0.5)),
            upper_quarter=float(np.quantile(held, 0.75)),
            highest=float(held.max()),
            mean=float(held.mean()),
        )

    def as_dict(self) -> dict[str, float]:
        """Return this spread as plain values, for a report file."""
        return {
            "count": float(self.count),
            "lowest": self.lowest,
            "lower_quarter": self.lower_quarter,
            "middle": self.middle,
            "upper_quarter": self.upper_quarter,
            "highest": self.highest,
            "mean": self.mean,
        }


@dataclass(frozen=True)
class Rise:
    """How one quantity of one arm sits against the same quantity of a baseline.

    **Both means are of the published value and never of a count.** The
    observation publishes a quantity with no denominator as a compressed
    magnitude, and the schema does not state which form a field takes. The
    compression keeps the order, so the above entry is exact and the size of
    the difference is not in the unit of the quantity.

    The above entry counts the seeds on which the arm read higher than the
    baseline, out of the seeds both played. **The count is the stronger
    statement over a small seed set**, because one runaway seed cannot carry
    it, and it is the one statement the compression cannot distort.

    The rewarded entry says whether the objective of the arm pays for this
    quantity. A rise in a quantity nothing rewards is the instrumental
    reading the measurement exists to find.
    """

    signal: str
    arm_mean: float
    baseline_mean: float
    above: int
    paired: int
    rewarded: bool

    @property
    def difference(self) -> float:
        """How much higher the arm read than the baseline, on average."""
        return self.arm_mean - self.baseline_mean

    @property
    def above_share(self) -> float:
        """The share of the paired seeds on which the arm read higher."""
        if self.paired == 0:
            return 0.0
        return self.above / self.paired

    def as_dict(self) -> dict[str, Any]:
        """Return this comparison as plain values, for a report file."""
        return {
            "signal": self.signal,
            "arm_mean": self.arm_mean,
            "baseline_mean": self.baseline_mean,
            "difference": self.difference,
            "above": self.above,
            "paired": self.paired,
            "above_share": self.above_share,
            "rewarded": self.rewarded,
        }


def rewarded_signals(scoring: Scoring) -> tuple[str, ...]:
    """Return the signals one objective pays for, in the order it names them.

    A weighting over single fields names its signals directly, and every one
    of them with a weight that is neither absent nor zero is a signal the
    objective pays for. An objective vector under a play style names its
    signals through the terms of the objectives the style weights, so this
    walks those terms and keeps the denominator of a bounded term as well.

    **The names come from the objective and never from a list written here.**
    A list here would be a second declaration of what a run rewards, and
    nothing would fail when a researcher changed the strategy table and this
    did not.

    An objective that states itself in neither form gives no names, because
    nothing here can read it.
    """
    terms = getattr(scoring, "terms", None)
    if isinstance(terms, dict):
        return tuple(
            name for name, weight in terms.items() if weight is not None and weight
        )
    objectives = getattr(scoring, "objectives", None)
    style = getattr(scoring, "style", None)
    if objectives is None or style is None:
        return ()
    weights = getattr(style, "weights", {})
    found: list[str] = []
    for objective in objectives:
        if not weights.get(objective.name):
            continue
        for term in objective.terms:
            for name in (term.signal, term.against):
                if name is not None and name not in found:
                    found.append(name)
    return tuple(found)


def require_signals(
    catalogue: SignalCatalogue, names: Iterable[str]
) -> tuple[str, ...]:
    """Resolve each name against the schema, and drop nothing silently.

    A name the world does not publish raises here, with the list of what the
    world does publish. A name the world publishes over several positions
    raises as well, because this report reads one number for each quantity
    and a compound signal states nothing about which position it means.
    """
    resolved: list[str] = []
    for name in names:
        signal = catalogue.signal(name)
        if not signal.scalar:
            message = (
                f"{name!r} holds {signal.positions} positions, and this report "
                "reads one number for each quantity. Name a signal of one "
                "position."
            )
            raise ValueError(message)
        if name not in resolved:
            resolved.append(name)
    return tuple(resolved)


def play(
    arm: Arm, seeds: Sequence[int], workers: int, scoring: Scoring
) -> list[Reading]:
    """Play one arm over the seed set, and read what each episode ended at.

    **This drives the batch itself rather than calling the population pass.**
    The population pass builds its environments, plays them and discards
    them, and it returns no handle on the worlds. The win path and the tick
    of the end come from the game end record of a world, so a caller that
    needs them has to hold the world after the last decision. The pass could
    carry the record instead, and that is a change to a module another agent
    holds.

    The scoring entry decides nothing about the episodes. The world comes
    from the configuration and the seed, the action comes from the policy,
    and a world whose seat the built-in controller holds takes no action at
    all. A scoring weights the readings, and this report weights nothing, so
    the scoring only has to be one the environment accepts.
    """
    vector = VectorEnv(arm.config, scoring, count=len(seeds), workers=workers)
    observations = vector.reset(list(seeds))
    while not vector.done:
        masks = vector.action_masks()
        actions = arm.policy.choose_many(observations, masks)
        results = vector.step(actions)
        observations = np.stack([result.observation for result in results])
    return [
        read_episode(arm.name, int(seed), env)
        for seed, env in zip(seeds, vector.envs, strict=True)
    ]


def read_episode(arm: str, seed: int, env: Env) -> Reading:
    """Read one finished episode: its signals, its path and its end tick.

    The signals come from the catalogue the environment built out of the
    schema of the engine, so this names no position and no field.

    The path comes from the game end record. A game that no reader ended
    holds no record, so the path is the absent-path name.

    **The end tick comes from the one reader the record module holds for
    it.** The rule was declared twice, and the two copies disagreed until one
    of them was removed.[^1]

    References
    ----------
    [^1]: Findings register, FND-689. ``docs/FINDINGS.md``
    """
    world = env.world
    end = world.game_end()
    limit = world.tick_limit
    end_tick = end_tick_of(world)
    return Reading(
        arm=arm,
        seed=seed,
        outcome=env.outcome,
        decisions=env.decisions,
        path=NO_PATH if end is None else str(end["path"]),
        end_tick=end_tick,
        winner=None if end is None else int(end["winner"]),
        reached_limit=limit > 0 and end_tick >= limit,
        signals=env.signals.read_scalars(np.asarray(env.observation())),
    )


def path_counts(readings: Sequence[Reading]) -> dict[str, int]:
    """Count how many episodes each win path ended, in path name order.

    **The paths come from the readings and never from a list written here.**
    The engine owns the names, and a list here would read a name the engine
    stopped writing as a zero rather than fail.
    """
    counts: dict[str, int] = {}
    for reading in readings:
        counts[reading.path] = counts.get(reading.path, 0) + 1
    return {name: counts[name] for name in sorted(counts)}


def outcome_counts(readings: Sequence[Reading]) -> dict[str, int]:
    """Count how many episodes ended in each outcome, in outcome name order."""
    counts: dict[str, int] = {}
    for reading in readings:
        counts[reading.outcome] = counts.get(reading.outcome, 0) + 1
    return {name: counts[name] for name in sorted(counts)}


def compare(
    arm: Sequence[Reading],
    baseline: Sequence[Reading],
    names: Sequence[str],
    rewarded: Sequence[str],
) -> list[Rise]:
    """Return how each named quantity of an arm sits against the baseline.

    Both sides are grouped by seed, so the comparison is paired. A seed only
    one side played is left out of the count, and each mean is taken over the
    readings that side holds.
    """
    held = {reading.seed: reading for reading in arm}
    yardstick = {reading.seed: reading for reading in baseline}
    shared = sorted(set(held) & set(yardstick))
    return [
        Rise(
            signal=name,
            arm_mean=_mean(arm, name),
            baseline_mean=_mean(baseline, name),
            above=sum(
                1
                for seed in shared
                if held[seed].signals[name] > yardstick[seed].signals[name]
            ),
            paired=len(shared),
            rewarded=name in rewarded,
        )
        for name in names
    ]


def _mean(readings: Sequence[Reading], name: str) -> float:
    """Return the mean of one signal over a set of readings."""
    if not readings:
        return 0.0
    return float(np.mean([reading.signals[name] for reading in readings]))


class IdleSeat:
    """Take the no-op at every decision.

    The baseline gives the seat back to the built-in controller, so the
    environment sends no action at all and this policy never reaches the
    engine. It exists because the batch asks every arm for one action for
    each world.
    """

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return the no-op for every world of the batch."""
        del masks
        return [0] * len(observations)


def build_arms(
    world: EnvConfig,
    controller_world: EnvConfig,
    strategies: Mapping[str, tuple[EnvConfig, Scoring, str]],
    stored: Mapping[str, Path],
    probe: Env,
) -> list[Arm]:
    """Return the arms to play: the baseline first, then everything else.

    The baseline is the built-in controller in the learner's own seat. It is
    first because every other arm is reported against it, so a stopped run
    holds the yardstick rather than a set of numbers with nothing to compare.

    The two untrained arms take the no-op at every decision, one under each
    policy kind this package builds. They are here to prove the reading path,
    and a pair that reads alike is the evidence that it works.

    A stored arm names a row of the strategy table, so its world and its
    rewarded quantities come from that row and never from the command line.
    """
    arms: list[Arm] = [
        Arm(name="controller", config=controller_world, policy=IdleSeat()),
        Arm(
            name="untrained-linear",
            config=world,
            policy=LinearPolicy.zeros(probe.action_length, probe.observation_length),
        ),
        Arm(
            name=f"untrained-{STRUCTURED_KIND}",
            config=world,
            policy=StructuredPolicy.of_catalogue(probe.action_length, probe.signals),
        ),
        Arm(name="random", config=world, policy=RandomPolicy(seed=0)),
    ]
    for objective, path in stored.items():
        config, scoring, _kind = strategies[objective]
        policy, _meta = load_policy(path, PolicyFit.of_env(Env(config, scoring)))
        arms.append(
            Arm(
                name=f"{objective} ({path.stem})",
                config=config,
                policy=policy,
                objective=objective,
                rewarded=rewarded_signals(scoring),
            )
        )
    return arms


def focus_signals(catalogue: SignalCatalogue, arms: Sequence[Arm]) -> tuple[str, ...]:
    """Return the quantities the printed table shows, resolved against the schema.

    The survival pair and held ground are always shown. The quantity each
    objective rewards is shown as well, so a reader tells a rise in a
    rewarded quantity from a rise in an unrewarded one.
    """
    wanted = [*SURVIVAL_SIGNALS, GROUND_SIGNAL]
    for arm in arms:
        wanted.extend(arm.rewarded)
    return require_signals(catalogue, wanted)


def _say_paths(readings: Sequence[Reading], label: str) -> None:
    """Print the win path share and the end tick spread of one arm."""
    total = len(readings)
    print(f"  {label}")
    for name, count in path_counts(readings).items():
        print(f"    path {name:>12s} {count:5d}/{total:<5d} {count / total:7.3f}")
    for name, count in outcome_counts(readings).items():
        print(f"    seat {name:>12s} {count:5d}/{total:<5d} {count / total:7.3f}")
    spread = Spread.of([float(reading.end_tick) for reading in readings])
    reached = sum(1 for reading in readings if reading.reached_limit)
    print(
        f"    end tick  lowest {spread.lowest:7.0f} quarter "
        f"{spread.lower_quarter:7.0f} middle {spread.middle:7.0f} quarter "
        f"{spread.upper_quarter:7.0f} highest {spread.highest:7.0f} "
        f"mean {spread.mean:8.1f}"
    )
    print(f"    reached the tick limit {reached:5d}/{total:<5d} {reached / total:7.3f}")


def _say_rises(rises: Sequence[Rise], label: str) -> None:
    """Print how each quantity of one arm sits against the baseline.

    The three value columns hold the published value of the observation,
    which is bounded and not a count. The seeds column is the paired count,
    and it is the column to read.
    """
    print(f"  {label}")
    print(
        f"    {'quantity':>22s} {'arm value':>12s} {'controller':>12s} "
        f"{'difference':>12s} {'seeds above':>12s} {'share':>7s} "
        f"{'rewarded':>9s}"
    )
    for rise in rises:
        mark = "yes" if rise.rewarded else "no"
        print(
            f"    {rise.signal:>22s} {rise.arm_mean:12.1f} "
            f"{rise.baseline_mean:12.1f} {rise.difference:+12.1f} "
            f"{rise.above:6d}/{rise.paired:<5d} {rise.above_share:7.3f} "
            f"{mark:>9s}"
        )


def _say_verdict(arms: Sequence[Arm], rises: Mapping[str, Sequence[Rise]]) -> None:
    """Print the population row of each arm, and say whether it was rewarded.

    This is the line the question turns on. An arm whose objective never pays
    for population, and which still reads higher than the controller on most
    of the paired seeds, is the instrumental reading. An arm that reads higher
    only where population is rewarded is the other answer.

    The line reports the paired count and never the mean difference, because
    the published value is compressed and the count is not.
    """
    print("\nthe population row of each arm\n")
    for arm in arms:
        for rise in rises.get(arm.name, ()):
            if rise.signal != POPULATION_SIGNAL:
                continue
            pays = "rewards it" if rise.rewarded else "rewards it not"
            print(
                f"  {arm.name:>28s} {pays:>16s} above the controller on "
                f"{rise.above}/{rise.paired} seeds ({rise.above_share:.3f})"
            )


def probe_scoring() -> Weighting:
    """Return a scoring that states nothing, for a run that weights nothing.

    This report reads signals and never a reward, so the scoring only has to
    be one the environment accepts. A weighting with no term and no outcome
    weight scores every reading at zero, which is what a reader wants: it
    reads the layout and pays for nothing.
    """
    return Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0)


def at_tick_limit(
    world: EnvConfig,
    controller_world: EnvConfig,
    strategies: Mapping[str, tuple[EnvConfig, Scoring, str]],
    limit: int,
) -> tuple[EnvConfig, EnvConfig, dict[str, tuple[EnvConfig, Scoring, str]]]:
    """Move the tick limit of every world, and carry the horizon with it.

    **The horizon is derived from the limit and never set beside it.** A
    horizon shorter than the limit truncates an episode, and a truncated
    episode reports no outcome, so a caller that moved one and not the other
    would measure a game that never ended.
    """
    if limit < 1:
        message = "the tick limit must be one tick or more"
        raise ValueError(message)

    def moved(config: EnvConfig) -> EnvConfig:
        return replace(
            config, tick_limit=limit, horizon=limit // config.decision_interval
        )

    return (
        moved(world),
        moved(controller_world),
        {
            name: (moved(config), scoring, kind)
            for name, (config, scoring, kind) in strategies.items()
        },
    )


def stored_policies(
    named: Sequence[str], strategies: Mapping[str, tuple[EnvConfig, Scoring, str]]
) -> dict[str, Path]:
    """Read the stored policies the caller named, and refuse an unknown row.

    A policy is named by the row of the strategy table it was trained under,
    so the world it plays and the quantities it was rewarded for come from
    the table. A row this package does not hold is refused with the list of
    the rows it does hold.
    """
    found: dict[str, Path] = {}
    for entry in named:
        objective, _, path = entry.partition("=")
        if not path:
            message = (
                f"{entry!r} names no path. Write it as OBJECTIVE=PATH, for "
                "example people=runs/people.npz."
            )
            raise ValueError(message)
        if objective not in strategies:
            message = (
                f"{objective!r} names no strategy. The table holds "
                f"{sorted(strategies)}."
            )
            raise ValueError(message)
        found[objective] = Path(path)
    return found


def report(
    world: EnvConfig,
    seeds: Sequence[int],
    seed_start: int,
    played: Mapping[str, Sequence[Reading]],
    rises: Mapping[str, Sequence[Rise]],
    focus: Sequence[str],
) -> dict[str, Any]:
    """Return every reading, so that a later reader asks a new question of it.

    The report holds one row for each episode with every one-position signal
    the engine published, not the subset the printed table shows. A run that
    kept only the printed table would have to be played again for the next
    question.
    """
    return {
        "world": {
            "width": world.width,
            "height": world.height,
            "faction_count": world.faction_count,
            "tick_limit": world.tick_limit,
            "horizon": world.horizon,
            "decision_interval": world.decision_interval,
            "seat": world.seat,
        },
        "seed_start": seed_start,
        "seeds": [int(seed) for seed in seeds],
        "focus": list(focus),
        "arms": {
            name: {
                "paths": path_counts(list(readings)),
                "outcomes": outcome_counts(list(readings)),
                "end_tick": Spread.of(
                    [float(row.end_tick) for row in readings]
                ).as_dict(),
                "reached_limit": sum(1 for row in readings if row.reached_limit),
                "episodes": [row.as_dict() for row in readings],
                "against_controller": [rise.as_dict() for rise in rises.get(name, ())],
            }
            for name, readings in played.items()
        },
    }


def main() -> None:
    """Play every arm over the held-out seeds, and print what each ended at."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--seeds",
        type=int,
        default=12,
        help=(
            "how many held-out worlds each arm plays. Keep it small while a "
            "training run holds the machine."
        ),
    )
    parser.add_argument("--seed-start", type=int, default=HELDOUT_START)
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument(
        "--tick-limit",
        type=int,
        default=None,
        help=(
            "override the tick limit of the run, to measure where games end "
            "under a shorter horizon. The horizon follows it."
        ),
    )
    parser.add_argument(
        "--policy",
        action="append",
        default=[],
        metavar="OBJECTIVE=PATH",
        help=(
            "a stored policy, named by the row of the strategy table it was "
            "trained under. Repeat it. The world and the rewarded quantities "
            "come from that row."
        ),
    )
    parser.add_argument("--out", type=Path, default=None)
    arguments = parser.parse_args()

    from cachette.learn.__main__ import CONTROLLER_WORLD, STRATEGIES, WORLD
    from cachette.learn.train import first_scoring

    world = WORLD
    controller_world = CONTROLLER_WORLD
    strategies = {
        name: (config, first_scoring(scoring), kind)
        for name, (config, scoring, kind) in STRATEGIES.items()
    }
    if arguments.tick_limit is not None:
        world, controller_world, strategies = at_tick_limit(
            world, controller_world, strategies, arguments.tick_limit
        )
    world = replace(world, threads=arguments.threads)
    controller_world = replace(controller_world, threads=arguments.threads)

    stored = stored_policies(arguments.policy, strategies)
    scoring = probe_scoring()
    probe = Env(world, scoring)
    seeds = viable_seeds(world, arguments.seeds, arguments.seed_start)
    arms = build_arms(world, controller_world, strategies, stored, probe)
    focus = focus_signals(probe.signals, arms)

    print(
        f"world {world.width} by {world.height}, {world.faction_count} "
        f"factions, tick limit {world.tick_limit}, horizon {world.horizon} "
        f"decisions of {world.decision_interval} ticks"
    )
    print(
        f"observation version {probe.observation_version}, action version "
        f"{probe.action_version}"
    )
    print(f"{len(seeds)} held-out seeds from {arguments.seed_start}: {seeds}")
    print(f"{len(arms)} arms, {arguments.workers} workers\n")

    played: dict[str, list[Reading]] = {}
    for arm in arms:
        print(f"playing {arm.name}", flush=True)
        played[arm.name] = play(arm, seeds, arguments.workers, scoring)

    print("\nwin paths and where games end\n")
    for arm in arms:
        _say_paths(played[arm.name], arm.name)

    baseline = played[arms[0].name]
    print("\npopulation and the quantities beside it, against the controller")
    print(
        "the value columns hold the bounded value the observation publishes, "
        "not a count.\nthe compression keeps the order, so read the paired "
        "count of the seeds above.\n"
    )
    rises: dict[str, list[Rise]] = {}
    for arm in arms[1:]:
        rises[arm.name] = compare(played[arm.name], baseline, focus, arm.rewarded)
        _say_rises(rises[arm.name], arm.name)

    _say_verdict(arms[1:], rises)

    if arguments.out is not None:
        payload = report(world, seeds, arguments.seed_start, played, rises, focus)
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        print(f"\nwrote {arguments.out}")


if __name__ == "__main__":
    main()
