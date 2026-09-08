"""A play style, which is a weighting over the objective vector of a run.

One reward mechanism trains one behaviour. The observed policy of this
project grows and then sits still, because one weighting rewards growth and
nothing else. A project that wants an aggressive policy beside a trading one
needs several weightings over one mechanism, not several mechanisms.

A play style is that weighting. It names a weight for each objective it
rewards, and it names every objective it refuses to reward. The scalar a
learner receives is the weighted combination of the objective vector, plus
the weight of the terminal outcome.[^1]

# A style must refuse something

A style that weights every objective trains the same policy as every other
style. This module therefore refuses a style that leaves an objective
unmentioned. Each objective of the vector appears in the weights or in the
refusals, and never in both. A refusal is a statement, and silence is not.

A weight of zero is refused as well. A zero weight and a refusal mean the
same thing to the arithmetic and different things to a reader, so this module
admits one of the two spellings.

# Floating point is allowed here

The engine holds no floating point value in simulated or aggregated state,
and this module is on the other side of that boundary.[^2] It reads the
engine through the objective set and writes nothing back.

# References

[^1]: Report 42, what a policy should be able to see, sections 10.2 and 10.3.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^2]: ADR-0002, state holds no floating point number, decision D4.
``docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import TYPE_CHECKING

import numpy as np

from .objective import ObjectiveSet, ObjectiveVector
from .reward import RUNNING, TERMINAL_ROWS, OutcomeReader, RewardStep

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from cachette import World


class StyleError(ValueError):
    """A play style states a weighting the objective vector cannot carry."""


class Optimisation(Enum):
    """Which optimiser reads the reward.

    The choice changes which shaping terms carry a signal. An evolution
    strategy sums the reward over the whole episode with no discount inside
    it, so a term over a change telescopes into one number for the episode. A
    gradient method discounts inside the episode, so the same term keeps its
    guarantee and gives a signal at every decision.[^1]

    References
    ----------
    [^1]: Report 42, what a policy should be able to see, section 10.4.
    ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
    """

    EVOLUTION_STRATEGY = "evolution_strategy"
    GRADIENT = "gradient"


@dataclass(frozen=True)
class PlayStyle:
    """What one style of play rewards, and what it refuses to reward.

    The weights entry gives one weight for each objective the style rewards.
    The refuses entry names every objective the style leaves at nothing. Every
    objective of the vector appears in exactly one of the two.

    The three outcome entries give what each terminal outcome is worth. They
    are the sparse part of the reward, and the objective vector is the dense
    part.

    The description entry says what the style trains. It is prose for a reader
    of a report and nothing computes with it.
    """

    name: str
    weights: Mapping[str, float]
    refuses: tuple[str, ...] = ()
    won: float = 0.0
    lost: float = 0.0
    drawn: float = 0.0
    description: str = ""

    def __post_init__(self) -> None:
        """Refuse a style that rewards nothing, or that says nothing twice."""
        if not self.weights:
            message = f"the style {self.name!r} rewards no objective"
            raise StyleError(message)
        zero = sorted(name for name, weight in self.weights.items() if weight == 0.0)
        if zero:
            message = (
                f"the style {self.name!r} weights {zero} at zero. A zero weight "
                "and a refusal mean the same thing, so name these in the "
                "refuses entry instead."
            )
            raise StyleError(message)
        both = sorted(set(self.weights) & set(self.refuses))
        if both:
            message = (
                f"the style {self.name!r} both weights and refuses {both}. "
                "An objective belongs to one of the two."
            )
            raise StyleError(message)

    def terminal(self, outcome: str) -> float:
        """Return what one outcome is worth under this style.

        Returns zero for the running outcome, which ends nothing.
        """
        if outcome == RUNNING:
            return 0.0
        if outcome not in TERMINAL_ROWS:
            message = f"{outcome!r} names no outcome. The outcomes are {TERMINAL_ROWS}."
            raise StyleError(message)
        weight: float = getattr(self, outcome)
        return weight

    def score(self, objectives: ObjectiveVector, outcome: str) -> float:
        """Return the scalar this style pays for one objective vector."""
        return objectives.combine(self.weights) + self.terminal(outcome)


def require_complete(style: PlayStyle, objectives: Sequence[str]) -> None:
    """Refuse a style that does not say something about every objective.

    **This is the one declaration of that rule.** The loader of the style
    table calls it against the objectives the table declares, and the scoring
    calls it against the objectives one world supplies. A second copy of the
    rule would be one fact stored twice, with nothing that fails when the
    copies disagree.

    A style that names an objective the caller does not hold is refused as
    well, and the message lists what the caller does hold.
    """
    held = set(objectives)
    named = set(style.weights) | set(style.refuses)
    unknown = sorted(named - held)
    if unknown:
        message = (
            f"the style {style.name!r} names {unknown}, which is no objective "
            f"here. The objectives are {sorted(held)}."
        )
        raise StyleError(message)
    silent = sorted(held - named)
    if silent:
        message = (
            f"the style {style.name!r} says nothing about {silent}. A style "
            "must weight every objective or refuse it by name, because a "
            "style that rewards everything trains what every other style "
            "trains."
        )
        raise StyleError(message)


@dataclass(frozen=True)
class EpisodeScore:
    """What one episode scored on each objective, and how it ended.

    A run stores this for each episode, so a later reader scores the same
    episodes under another style without playing them again. The combination
    is linear in the vector, so the sum of the combination over the decisions
    equals the combination of the sum.
    """

    objectives: ObjectiveVector
    outcome: str

    def under(self, style: PlayStyle) -> float:
        """Return what one style pays for this episode."""
        return style.score(self.objectives, self.outcome)


def rank_under(
    style: PlayStyle, episodes: Sequence[Sequence[EpisodeScore]]
) -> tuple[float, ...]:
    """Return the mean score of each candidate under one style.

    The episodes entry holds one sequence for each candidate, in candidate
    order, and each sequence holds the episodes that candidate played. **The
    order is the order the caller gives, and never the order a worker
    finished.**[^1]

    References
    ----------
    [^1]: ADR-0155, a batch of worlds steps in one call, in index order.
    ``docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md``
    """
    scores = []
    for played in episodes:
        if not played:
            message = "a candidate with no episode has no score"
            raise StyleError(message)
        scores.append(float(np.mean([row.under(style) for row in played])))
    return tuple(scores)


@dataclass(frozen=True)
class ObjectiveScoring:
    """An objective vector and the play style that weights it.

    A run holds one of these and asks it for the scorer of each episode. It
    is the second of the two things a run may be scored by, and a weighting
    over single fields is the first.

    The optimisation entry names the optimiser that will read the reward. It
    decides whether a shaping term over a change is admitted, because such a
    term gives no signal inside an episode under an evolution strategy.
    """

    objectives: ObjectiveSet
    style: PlayStyle
    optimisation: Optimisation = Optimisation.EVOLUTION_STRATEGY

    def __post_init__(self) -> None:
        """Refuse a style the vector cannot carry, and a term that telescopes.

        The vector must hold every objective the style names, and the style
        must name every objective the vector holds. A style written against
        one world and run against another then fails here, rather than
        rewarding a subset of what its author intended.

        A weighted objective whose every term reads a change contributes one
        number for the whole episode under an evolution strategy. That is not
        shaping, and a run that believed it was shaping would measure a flat
        generation and blame the policy. A gradient run admits the same
        objective, because it discounts inside the episode.
        """
        require_complete(self.style, self.objectives.names)
        if self.optimisation is not Optimisation.EVOLUTION_STRATEGY:
            return
        telescoping = sorted(
            set(self.objectives.telescoping()) & set(self.style.weights)
        )
        if telescoping:
            message = (
                f"the style {self.style.name!r} weights {telescoping}, and every "
                "term of those objectives reads a change. An evolution "
                "strategy sums the reward over the whole episode, so the sum "
                "of a change is the level at the end minus the level at the "
                "start. Such an objective gives no signal inside the episode. "
                "Use the level of the same signal, or name the gradient "
                "optimisation."
            )
            raise StyleError(message)

    def scorer(self, world: World, faction: int) -> ObjectiveReward:
        """Build the scorer of one faction of one world under this style."""
        return ObjectiveReward(world, faction, self.objectives, self.style)


class ObjectiveReward:
    """The objective vector and the reward of one faction over one run.

    A caller builds one of these for one faction of one world, steps the
    world, and reads the reward of each decision. Each reading carries the
    whole objective vector beside the scalar, so a report says which
    objective moved.

    The reader reads the observation array of the faction, and the winner of
    a game that has already ended. It reads nothing a player of that faction
    could not see.
    """

    def __init__(
        self,
        world: World,
        faction: int,
        objectives: ObjectiveSet,
        style: PlayStyle,
    ) -> None:
        """Build the scorer of one faction, and take the first reading."""
        self._faction = faction
        self._objectives = objectives
        self._style = style
        self._outcomes = OutcomeReader(world, faction)
        self._previous: np.ndarray | None = None
        self._total = objectives.zero()
        self.reset(world)

    @property
    def faction(self) -> int:
        """Return the faction this scorer answers for."""
        return self._faction

    @property
    def outcome(self) -> str:
        """Return the outcome the last reading reported."""
        return self._outcomes.outcome

    @property
    def done(self) -> bool:
        """Return whether the run has ended."""
        return self._outcomes.done

    @property
    def objectives(self) -> Mapping[str, float]:
        """Return what each objective scored over the episode so far."""
        return self._total.as_dict()

    @property
    def vector(self) -> ObjectiveVector:
        """Return the objective vector of the episode so far."""
        return self._total

    def score(self) -> EpisodeScore:
        """Return what the episode scored, for a later reader of the run."""
        return EpisodeScore(objectives=self._total, outcome=self.outcome)

    def reset(self, world: World) -> None:
        """Take the first reading of a run, and pay nothing for it.

        A caller resets before the first decision. The first reading is the
        baseline of every term that reads a change, so it earns nothing.
        """
        self._previous = np.asarray(world.faction_observation(self._faction))
        self._total = self._objectives.zero()
        self._outcomes.reset()

    def read(self, world: World) -> RewardStep:
        """Return what the decision before this reading earned."""
        ended = self.done
        observation = np.asarray(world.faction_observation(self._faction))
        state = self._outcomes.read(world)
        if ended:
            return RewardStep(
                value=0.0,
                shaped=0.0,
                terminal=0.0,
                outcome=state.name,
                done=True,
                alive=state.alive,
                terms={},
                changes={},
                objectives=self._objectives.zero().as_dict(),
            )
        reading = self._objectives.read(observation, self._previous)
        self._previous = observation
        self._total = self._total.plus(reading)
        shaped = reading.combine(self._style.weights)
        terminal = self._style.terminal(state.name)
        return RewardStep(
            value=shaped + terminal,
            shaped=shaped,
            terminal=terminal,
            outcome=state.name,
            done=state.done,
            alive=state.alive,
            terms=dict(reading.values),
            changes={},
            objectives=reading.as_dict(),
        )


__all__ = [
    "EpisodeScore",
    "ObjectiveReward",
    "ObjectiveScoring",
    "Optimisation",
    "PlayStyle",
    "StyleError",
    "rank_under",
    "require_complete",
]
