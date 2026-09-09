"""The reward of one faction, computed in the control plane.

A learner needs a scalar on every decision. The engine holds no reward, and
that is a decision rather than a gap. What a faction should be rewarded for
is a rule of the downstream game, so a product record refuses to state
it.[^1] A reward inside the engine would enter the state hash, and every
golden file would move when a researcher changed their mind.[^2]

This module is therefore Python, and it changes nothing in the engine.
Floating point is allowed here, because the simulation holds none and the
learner's side of the boundary is the exception the record names.[^3]

The terms of the reward are data. A caller states a weighting, and this
module applies it. **This module states no weight.** One blocker holds the
rules of the downstream game, so it governs every term.[^4] A register holds
one row for each term, and every row is unset.[^5]

What a caller must supply
-------------------------

The reward means nothing until a caller supplies all of these.

- **A weight for each shaped term it names.** A term with a weight of zero
  contributes nothing, which is a statement and not an omission.
- **A weight for each of the three terminal outcomes.** A weighting with
  three zero terminal weights rewards no outcome.
- **The unit of each weight.** A change weight multiplies the raw published
  value, and the engine compresses every count before it publishes one. Two
  counts of very different sizes therefore cross in the same range, and a
  weight sized for a raw integer reaches almost nothing. A level weight
  multiplies a value the unit of the field has already bounded, so every
  level weight carries one unit.

**A weighting whose shaped weights are all zero is the terminal reward.**
This module therefore needs no mode selection. The choice between a shaped
reward and a terminal reward is a choice of weights, which keeps the terms
data rather than code.

What this reward reads
----------------------

**It reads the observation array of the faction, and one public fact.** The
array holds what that faction observes and nothing else.[^6] The public fact
is the winner of a game that has already ended, which every player learns
when the game ends. This module reads that fact through the game end record,
and only after the array says the game is over.[^7]

The alternative was a reward computed from the truth of the world. That was
refused. The first checkable statement of the product record is that the
harness never shows a learner anything a player of that faction could not
see.[^1] A reward is not an observation, but a reward reaches the policy
through the gradient, so a term the observation never held still teaches the
policy something a player could not know. The cost of the refusal is a
weaker signal: this module cannot weigh a rival's standing, and it cannot
weigh ground the faction has not seen. A researcher who wants those terms
must first ask the engine for a reader that answers for one faction.

Why the array and not the standing
----------------------------------

The standing of a faction reports the work toward a victory claim. The
wonder reader compares the claim itself, and the two are not the same
quantity.[^8] The observation array carries both, and it carries the
population, the tick, the tick limit and the game end flag as well. The
array is therefore a superset of the standing, and this module reads the
array.

**The array still omits one quantity a win reader compares.** The renown
reader compares the best renown against the renown target, and the array
carries no renown target. The array carries the tick limit, which is the
matching threshold of the tick limit path, so the omission is an asymmetry
and not a rule.[^9]

What a faction gets when it loses every unit
--------------------------------------------

**There is no elimination outcome, and that answer is measured.** A faction
that loses every unit and every person keeps its held ground and its seat.
The territory reader compares held ground at the tick limit, so such a
faction has not lost and may still win.[^10] A terminal that fired on the
loss of the last unit would end an episode the faction could still win.

This module therefore reports one boolean beside the outcome. The alive
entry is true while the faction holds a unit or a person, which is what a
faction needs in order to act at all. A caller that wants to stop a run
early reads it. A caller that wants the whole run ignores it.

A faction the seeding never seated reads false as well, and it reads no
terminal until the game ends. A never-seated faction and a spent faction
therefore differ in nothing this module reports, because they differ in
nothing the observation reports.

What a decisive game is worth
-----------------------------

**A game that runs to the tick limit is a signal of indecisive play.** Four
paths end a game. Domination, a finished wonder and a renown target all fire
inside the run. The territory path fires at the tick limit and compares held
ground, so an episode that runs out the clock still ends won or lost. Weaker
play shifts a game toward the limit, and a policy that does nothing at all
guarantees it.

This module therefore pays one term for the time a win left on the clock.
The term is the early weight times the share of the tick limit that was still
to run when the game ended. A win at the limit leaves nothing on the clock
and pays nothing here.

**The term is terminal, and it is a level rather than a difference.** An
undiscounted episode return sums the reward of every decision, so a sum of
first differences collapses to the last reading less the first. A term that
weighs a change therefore contributes the same amount whatever the policy did
in between, and the optimiser sees nothing.[^11]

**Only a win pays the term.** A loss pays the loss weight and nothing else,
whether it comes at tick 300 or at the tick limit. A term that paid the time
left on any outcome would pay a faction for losing quickly, and a faction
that gives up early would then outscore a faction that held on and lost
narrowly at the limit. A flat penalty for reaching the limit has the same
defect with the sign reversed: it makes a fast loss cheaper than a slow one.

**The early weight is zero unless a caller sets it.** A weight of zero pays
nothing, so a run under a weighting that states no early weight scores what
it scored before this term existed. A register holds the row, and one finding
holds the reasoning.[^12]

Two shaped forms
----------------

**A caller states a change weight, a level weight, or both.** A change weight
telescopes, so it pays one number for the whole episode. A level weight is
paid on every decision, so the episode pays the area under the curve of the
field. A policy that reaches a level sooner scores more, which is the signal a
multi-step chain needs.

A level divides the published value by the unit the engine published for that
field, so every level lies inside the closed interval from minus one to one. A
weighting states no bound of its own, because the engine owns the bound of
each field and publishes it.[^6]

**Every shaped weight of this module was a change weight, and the whole
strategy table was one.** A run therefore trained against a terminal reward
under weights that read as dense. One finding holds the measurement.[^13]

Determinism
-----------

This module reads the engine and writes nothing to it. It adds no value to
the state hash, and two researchers train two policies on one engine with
two weightings.

References
----------
[^1]: PRD-0056, a learner plays one faction against the controllers.
``docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md``

[^2]: Design, a learner plays one faction against the controllers, section 3.
``docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md``

[^3]: ADR-0002, state holds no floating point number, decision D4.
``docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md``

[^4]: Blockers register, BLK-050. ``docs/BLOCKERS.md``

[^5]: Reinforcement learning parameters, the reward rows.
``docs/reference/rl-costs.md``

[^6]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D3.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^7]: ADR-0148, a game end is recorded once and stops the controllers,
decisions D1 and D2.
``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``

[^8]: Findings register, FND-568. ``docs/FINDINGS.md``

[^9]: Findings register, FND-582. ``docs/FINDINGS.md``

[^10]: Findings register, FND-583. ``docs/FINDINGS.md``

[^11]: Findings register, FND-679. ``docs/FINDINGS.md``

[^12]: Findings register, FND-692. ``docs/FINDINGS.md``

[^13]: Findings register, FND-700. ``docs/FINDINGS.md``
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Final, Protocol

from .objective import Scale
from .signals import SignalCatalogue

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

    import numpy as np

    from cachette import World

    from .signals import Signal

# The shaped terms the register holds one row for. A caller may weigh any
# other field of the schema that holds one position, and this tuple is what a
# caller gets when it asks for the rows the project reserved.
#
# The register mirrors this tuple, and one test compares the two, because a
# second declaration site with nothing to fail is the defect shape this
# project names first.
SHAPED_ROWS: Final[tuple[str, ...]] = (
    "held_tiles",
    "domination_progress",
    "live_units",
    "population",
    "store_total",
    "best_renown",
    "wonder_progress",
    "wonder_track_progress",
)

# The outcome the timing term pays on. It is named once, because the term
# and the outcome reader both act on it.
WON: Final = "won"

# The terminal outcomes the register holds one row for.
TERMINAL_ROWS: Final[tuple[str, ...]] = (WON, "lost", "drawn")

# The terminal timing terms the register holds one row for. A timing term is
# paid once, at the end, and it weighs a level rather than a change.
#
# The register mirrors this tuple as well, and one test compares the two.
TIMING_ROWS: Final[tuple[str, ...]] = ("won_early",)

# Every outcome this module reports, including the one that ends nothing.
RUNNING: Final = "running"
OUTCOMES: Final[tuple[str, ...]] = (RUNNING, *TERMINAL_ROWS)

# The field of the observation array that decides the outcome.
#
# The array carries no game-over flag and no raw tick, because every position
# of it is a bounded value. The record of the end is a public fact the world
# reports directly, and the ticks left before the limit fires answer the draw.
_REMAINING_TICKS: Final = "remaining_ticks"

# The field that states how far into the episode the reading stands. It is
# the elapsed tick count over the tick limit, so it rises from zero to one
# and the engine clamps it at one.
#
# The ticks left field cannot answer this. The engine writes it as a
# compressed magnitude, and a logarithm of a tick count divided by a tick
# limit is not a share of the clock.
_TICK_SHARE: Final = "tick_share"

# The fields that state whether the faction can act. A faction that reads
# zero in both holds nothing that takes a decision.
_ACTING_FIELDS: Final[tuple[str, ...]] = ("live_units", "population")


class UnsetWeightError(ValueError):
    """A weighting left a weight unset, so the reward has no value.

    Every weight of this module is unset until the rules of the downstream
    game are written down. One blocker holds those rules, and a register
    holds one unset row for each weight.[^1] [^2]

    References
    ----------
    [^1]: Blockers register, BLK-050. ``docs/BLOCKERS.md``

    [^2]: Reinforcement learning parameters, the reward rows.
    ``docs/reference/rl-costs.md``
    """


class TermError(ValueError):
    """A weighting named a term the observation schema cannot supply.

    A term must name a field of the schema, and that field must hold exactly
    one position. A weight over many positions states nothing about which
    position it weighs.
    """


@dataclass(frozen=True)
class Weighting:
    """What each term of the reward is worth.

    The terms entry maps the name of an observation field to the weight of
    its change since the previous decision. The levels entry maps the name of
    an observation field to the weight of its own value. The three outcome
    entries give what each terminal outcome is worth.

    **A term entry telescopes and a level entry does not.** An evolution
    strategy sums the reward of every decision of the episode with no
    discount, so a sum of changes collapses to the last reading less the
    first. A term entry therefore pays one number for the whole episode,
    whatever the policy did in between, and no weight on it makes the signal
    dense. A level entry is paid on every decision, so the episode pays the
    area under the curve of the field and a policy that reaches a level
    sooner scores more.[^3]

    A level entry reads the published value divided by the unit the engine
    published for that field, so every level lies inside the closed interval
    from minus one to one. **A level entry therefore states no bound of its
    own.** The engine owns the bound of each field and publishes it in the
    schema.[^4]

    One field belongs in one entry. A field named in both would pay for its
    level and for its change at once, and no reader of the two numbers could
    say which weight moved the reward.

    The early entry is what the time left on the clock pays on a win. It is
    zero unless a caller sets it, and a zero pays nothing. A weighting that
    states no early weight therefore scores a run exactly as it scored the
    run before the entry existed.[^2]

    A weight of ``None`` is unset. A reward refuses to run while any weight
    a caller asked for is unset, because a guessed weight is a rule of the
    downstream game that nobody has written down.[^1]

    **The early entry is the one weight that defaults rather than refuses.**
    It defaults so that every stored score stays comparable, and a default
    of zero states a rule of the downstream game no more than an absent term
    does.

    References
    ----------
    [^1]: Blockers register, BLK-050. ``docs/BLOCKERS.md``

    [^2]: Findings register, FND-692. ``docs/FINDINGS.md``

    [^3]: Findings register, FND-679. ``docs/FINDINGS.md``

    [^4]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decision D1.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """

    terms: Mapping[str, float | None] = field(default_factory=dict)
    levels: Mapping[str, float | None] = field(default_factory=dict)
    won: float | None = None
    lost: float | None = None
    drawn: float | None = None
    won_early: float = 0.0

    def __post_init__(self) -> None:
        """Refuse a weighting that reads one field as a level and as a change."""
        both = sorted(set(self.terms) & set(self.levels))
        if both:
            message = (
                f"{both} name a field this weighting reads as a level and as "
                f"a change. A field belongs in one entry, because a reader of "
                f"the two numbers cannot say which weight moved the reward."
            )
            raise TermError(message)

    @property
    def telescopes(self) -> bool:
        """Whether every shaped weight of this weighting reads a change.

        Such a weighting gives no signal inside an episode under an evolution
        strategy, because the sum of a change over the episode is the level
        at the end minus the level at the start. A shaped weight then pays
        one number for the whole episode, and the optimiser sees a terminal
        reward however dense the weights look.[^1]

        A weighting with no shaped weight at all is the terminal reward, and
        it does not telescope: it states that nothing but the outcome scores.

        References
        ----------
        [^1]: Findings register, FND-679. ``docs/FINDINGS.md``
        """
        return bool(self.terms) and not self.levels

    def terminal(self, outcome: str) -> float:
        """Return the weight of one outcome.

        Returns zero for the running outcome, which ends nothing. Raises
        ``ValueError`` when the outcome names none of the outcomes, and
        ``UnsetWeightError`` when the weight of the outcome is unset.
        """
        if outcome == RUNNING:
            return 0.0
        if outcome not in TERMINAL_ROWS:
            message = f"{outcome!r} names no outcome. The outcomes are {OUTCOMES}."
            raise ValueError(message)
        weight: float | None = getattr(self, outcome)
        if weight is None:  # pragma: no cover - the constructor refuses this
            message = f"the weight of the {outcome} outcome is unset"
            raise UnsetWeightError(message)
        return weight

    def early(self, outcome: str, remaining_share: float) -> float:
        """Return what the time left before the tick limit pays on one outcome.

        The remaining share entry is the part of the tick limit that had not
        run when the reading was taken. It is one at the first tick and zero
        at the limit.

        **Only a win pays.** A win with most of the clock left pays the whole
        early weight. A win at the limit pays nothing, because the territory
        path fired and the faction left no time on the clock. Every other
        outcome pays nothing at all, so a faction gains nothing by losing
        quickly.
        """
        if outcome != WON:
            return 0.0
        return self.won_early * remaining_share

    def unset_names(self) -> tuple[str, ...]:
        """Return the name of every weight this weighting left unset."""
        unset = [name for name, weight in self.terms.items() if weight is None]
        unset.extend(name for name, weight in self.levels.items() if weight is None)
        unset.extend(name for name in TERMINAL_ROWS if getattr(self, name) is None)
        return tuple(unset)

    def scorer(self, world: World, faction: int) -> Reward:
        """Build the scorer of one faction of one world under this weighting.

        This makes a weighting one of the two things a run may be scored by.
        The other is an objective vector under a play style, and an
        environment tells them apart by nothing: it asks either one for a
        scorer.
        """
        return Reward(world, faction, self)


# The weighting the register states. Every row is present and every row is
# unset, so a caller that takes it and builds a reward is told which weights
# it must supply, by name.
UNSET_WEIGHTING: Final[Weighting] = Weighting(terms=dict.fromkeys(SHAPED_ROWS))


class Scorer(Protocol):
    """What an environment needs of the thing that scores its seat.

    Two scorers satisfy this. The weighted reward of this module gives one
    scalar and no objective vector. The objective scorer gives both.[^1]

    An environment holds one of these for one episode. It resets before the
    first decision and reads after each one.

    References
    ----------
    [^1]: Report 42, what a policy should be able to see, section 10.2.
    ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
    """

    @property
    def outcome(self) -> str:
        """Name the state of the run: running, won, lost or drawn."""

    @property
    def done(self) -> bool:
        """Whether the run has ended."""

    @property
    def objectives(self) -> Mapping[str, float]:
        """What each objective scored over the episode so far."""

    def reset(self, world: World) -> None:
        """Take the first reading of a run, and pay nothing for it."""

    def read(self, world: World, observation: np.ndarray | None = None) -> RewardStep:
        """Return what the decision before this reading earned.

        The observation entry is the array of the seat at this state. A
        caller that already holds it passes it, and the scorer then builds
        no array of its own.
        """


class Scoring(Protocol):
    """How a run builds the scorer of one faction of one world.

    An environment builds a new world for each episode, so it needs a way to
    build a new scorer for it. This is that way, and it is the one seam
    between a run and what the run rewards.
    """

    def scorer(self, world: World, faction: int) -> Scorer:
        """Build the scorer of one faction of one world."""


@dataclass(frozen=True)
class Outcome:
    """The state of one run, read from the observation of one faction.

    The name entry is one of the outcomes this module declares. The done
    entry is true once the run has ended. The alive entry is true while the
    faction holds a unit or a person, which is what a faction needs in order
    to act at all.

    The remaining share entry is the part of the tick limit that had not run
    when the reading was taken. It is one at the first tick and zero at the
    limit. A world with no tick limit reads zero, because a run with no end
    leaves no time on a clock it does not hold.
    """

    name: str
    done: bool
    alive: bool
    remaining_share: float = 0.0


class _ElapsedShare:
    """How much of the tick limit a reading still has to run.

    The engine writes the elapsed part of the episode as a bounded share of
    the tick limit, and it publishes the bound that the share was written
    against. This class reads the field through the signal catalogue of the
    world and divides by the published bound, so it holds no bound of its
    own.[^1]

    A bound written here would be a second declaration of an engine rule,
    and nothing would fail when the two disagreed.

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decision D1.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """

    def __init__(self, signal: Signal, bound: float) -> None:
        """Hold the field of one layout and the bound the engine published."""
        self._signal = signal
        self._bound = bound

    @classmethod
    def of_world(cls, world: World) -> _ElapsedShare:
        """Read the field and its published bound from the schema of a world."""
        signal = SignalCatalogue.of_world(world).signal(_TICK_SHARE)
        form = signal.form
        if form is None or form.high <= 0:  # pragma: no cover - schema contract
            message = (
                f"the schema states no upper bound for {_TICK_SHARE!r}, so "
                f"nothing here knows what one whole episode reads as"
            )
            raise ValueError(message)
        return cls(signal, float(form.high))

    def remaining_share(self, world: World, values: np.ndarray) -> float:
        """Return the part of the tick limit that has not run at this reading.

        A world with no tick limit answers zero. The field reads zero in such
        a world, because a run with no end holds no position inside an
        episode, and a zero there means the first tick rather than the last.
        """
        if world.tick_limit <= 0:
            return 0.0
        elapsed = self._signal.read(values) / self._bound
        return max(0.0, 1.0 - elapsed)


class OutcomeReader:
    """Whether the run has ended, how it ended, and whether the faction acts.

    **This is the one declaration of how a run ends.** Two scorers need the
    answer, and a second copy of the rule would be one fact stored twice with
    nothing that fails when the copies disagree.

    The reader reads the observation array of the faction, and it reads the
    winner of a game that has already ended.[^1] It reads nothing a player of
    that faction could not see.

    References
    ----------
    [^1]: ADR-0148, a game end is recorded once and stops the controllers,
    decision D1.
    ``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``
    """

    def __init__(self, world: World, faction: int) -> None:
        """Find the positions this reader needs in the layout of one world.

        The elapsed share comes through the signal catalogue of the world
        rather than through a position alone. The catalogue carries the value
        form the engine published for the field, and that form states the
        bound the share was written against. A divisor written here would be
        a second declaration of an engine rule.
        """
        self._faction = faction
        self._starts = _field_starts(world, (_REMAINING_TICKS, *_ACTING_FIELDS))
        self._elapsed = _ElapsedShare.of_world(world)
        self._outcome = RUNNING
        self._done = False

    @property
    def outcome(self) -> str:
        """Return the outcome the last reading reported."""
        return self._outcome

    @property
    def done(self) -> bool:
        """Return whether the run has ended."""
        return self._done

    def reset(self) -> None:
        """Forget the end of the previous run."""
        self._outcome = RUNNING
        self._done = False

    def read(self, world: World, observation: np.ndarray | None = None) -> Outcome:
        """Name the state of the run, and keep it.

        A run that has already ended keeps the outcome it ended with. The
        first terminal reading is the one that pays, and every later reading
        reports the same end.

        The observation entry is the array of the faction at this state. The
        reader reads four positions of it and never writes it, so a caller
        that already holds the array passes it and the reader builds none.
        """
        values = (
            world.faction_observation(self._faction)
            if observation is None
            else observation
        )
        reading = {name: int(values[start]) for name, start in self._starts.items()}
        alive = any(reading[name] > 0 for name in _ACTING_FIELDS)
        remaining = self._elapsed.remaining_share(world, values)
        if self._done:
            return Outcome(
                name=self._outcome,
                done=True,
                alive=alive,
                remaining_share=remaining,
            )
        self._outcome = self._name_of(world, reading)
        self._done = self._outcome != RUNNING
        return Outcome(
            name=self._outcome,
            done=self._done,
            alive=alive,
            remaining_share=remaining,
        )

    def _name_of(self, world: World, reading: Mapping[str, int]) -> str:
        """Name the state of the run after one reading."""
        end = world.game_end()
        if end is not None:
            return WON if end["winner"] == self._faction else "lost"
        if world.tick_limit > 0 and reading[_REMAINING_TICKS] == 0:
            return "drawn"
        return RUNNING


@dataclass(frozen=True)
class RewardStep:
    """What one decision earned.

    The value entry is the reward the learner receives. It is the shaped
    entry plus the terminal entry plus the early entry.

    The early entry is what the time left on the clock paid on a win. It is
    zero on every other outcome, and it is zero on a win under a weighting
    that states no early weight. The remaining share entry is the part of the
    tick limit that had not run at this reading, and a caller reads it to see
    the early entry hold at zero while the clock moves.

    The terms entry gives what each change term contributed, so a caller sees
    which term moved. The changes entry gives the raw change of each term
    before its weight, so a caller sees a term move while its weight holds
    the contribution at zero.

    The levels entry gives what each level term contributed, and the shares
    entry gives the bounded level each one read before its weight. The pair
    answers the same question for a level term that the terms and the changes
    pair answers for a change term.

    The outcome entry names the state of the run. The done entry is true once
    the run has ended. The alive entry is true while the faction holds a unit
    or a person.

    The objectives entry gives what each named objective scored on this
    decision. It is empty for a scorer that holds no objective vector, which
    is the single-scalar weighting of this module.
    """

    value: float
    shaped: float
    terminal: float
    outcome: str
    done: bool
    alive: bool
    terms: Mapping[str, float]
    changes: Mapping[str, int]
    objectives: Mapping[str, float] = field(default_factory=dict)
    levels: Mapping[str, float] = field(default_factory=dict)
    shares: Mapping[str, float] = field(default_factory=dict)
    early: float = 0.0
    remaining_share: float = 0.0


class Reward:
    """The reward of one faction over one run.

    A caller builds one of these for one faction of one world, steps the
    world, and reads the reward of each decision.

    The reward reads the observation array of the faction, and it reads the
    winner of a game that has already ended.[^1] It reads nothing else, so no
    term of it weighs a quantity the faction has not observed.

    References
    ----------
    [^1]: ADR-0148, a game end is recorded once and stops the controllers,
    decision D1.
    ``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``
    """

    def __init__(self, world: World, faction: int, weighting: Weighting) -> None:
        """Build the reward of one faction, and take the first reading.

        Raises ``TermError`` when a term names no single-position field of
        the schema of this world, or when a level term names a field whose
        form the schema does not state. Raises ``UnsetWeightError`` when the
        weighting leaves a weight unset.
        """
        self._faction = faction
        self._weighting = weighting
        self._starts = _field_starts(world, tuple(weighting.terms))
        self._level_starts = _field_starts(world, tuple(weighting.levels))
        self._units = _field_units(world, tuple(weighting.levels))
        self._outcomes = OutcomeReader(world, faction)
        unset = weighting.unset_names()
        if unset:
            message = (
                "the reward has no value while a weight is unset. "
                f"These weights are unset: {', '.join(unset)}. "
                "BLK-050 holds the rules of the downstream game, and "
                "docs/reference/rl-costs.md holds one unset row for each "
                "weight. Supply a weight for each name above."
            )
            raise UnsetWeightError(message)
        self._previous: dict[str, int] = {}
        self.reset(world)

    @property
    def faction(self) -> int:
        """Return the faction this reward answers for."""
        return self._faction

    @property
    def outcome(self) -> str:
        """Return the outcome the last reading reported."""
        return self._outcomes.outcome

    @property
    def objectives(self) -> Mapping[str, float]:
        """Return no objective, because this scorer holds no objective vector.

        A weighting collapses every term into one scalar, so there is nothing
        for a report to break down. An objective scorer answers this with one
        value for each objective it holds.
        """
        return {}

    @property
    def done(self) -> bool:
        """Return whether the run has ended."""
        return self._outcomes.done

    def reset(self, world: World) -> None:
        """Take the first reading of a run, and pay nothing for it.

        A caller resets before the first decision. A change weight reads the
        movement since the previous reading, so the first reading is a
        baseline and not a reward.

        **A level weight keeps no state, so this resets none for it.** A
        level is a property of one reading, and the reset therefore holds
        the baseline of the change weights alone.
        """
        self._previous = self._read(world)
        self._outcomes.reset()

    def read(self, world: World, observation: np.ndarray | None = None) -> RewardStep:
        """Return what the decision before this reading earned.

        A caller steps the world and then calls this. The shaped part is the
        weighted change of each change term since the previous reading, plus
        the weighted bounded level of each level term at this reading. The
        terminal part is the weight of the outcome, and it is paid once. The
        early part is what the time left on the clock pays on a win, and it
        is paid once as well.

        **The observation array of one state serves every reader of it.** The
        weighted terms and the outcome both come from the array of the seat
        at this state, and building it twice gives the same numbers at twice
        the cost. A caller that already holds the array passes it here, and
        this reads no array of its own.
        """
        ended = self.done
        held = self._array(world, observation)
        reading = self._read(world, held)
        state = self._outcomes.read(world, held)
        if ended:
            return RewardStep(
                value=0.0,
                shaped=0.0,
                terminal=0.0,
                outcome=state.name,
                done=True,
                alive=state.alive,
                terms=dict.fromkeys(self._weighting.terms, 0.0),
                changes=dict.fromkeys(self._weighting.terms, 0),
                levels=dict.fromkeys(self._weighting.levels, 0.0),
                shares=dict.fromkeys(self._weighting.levels, 0.0),
                remaining_share=state.remaining_share,
            )

        changes = {
            name: reading[name] - self._previous[name] for name in self._weighting.terms
        }
        terms = {
            name: _weight_of(self._weighting.terms, name) * float(change)
            for name, change in changes.items()
        }
        shares = self._read_levels(world, held)
        levels = {
            name: _weight_of(self._weighting.levels, name) * share
            for name, share in shares.items()
        }
        shaped = sum(terms.values()) + sum(levels.values())
        terminal = self._weighting.terminal(state.name)
        early = self._weighting.early(state.name, state.remaining_share)

        self._previous = reading

        return RewardStep(
            value=shaped + terminal + early,
            shaped=shaped,
            terminal=terminal,
            outcome=state.name,
            done=state.done,
            alive=state.alive,
            terms=terms,
            changes=changes,
            levels=levels,
            shares=shares,
            early=early,
            remaining_share=state.remaining_share,
        )

    def _array(self, world: World, observation: np.ndarray | None = None) -> np.ndarray:
        """Return the observation array of the seat, building at most one.

        **One decision builds the array of its state once.** Three readers of
        this module read it, and a fourth reads it in the outcome reader, so
        one build answers all of them. A caller that already holds the array
        passes it and this builds none.[^1]

        References
        ----------
        [^1]: Report 38, where the training time goes, section 10.2.
        ``docs/research/reports/38-where-the-training-time-goes.md``
        """
        if observation is not None:
            return observation
        return world.faction_observation(self._faction)

    def _read(
        self, world: World, observation: np.ndarray | None = None
    ) -> dict[str, int]:
        """Read every position this reward needs from the observation array."""
        values = self._array(world, observation)
        return {name: int(values[start]) for name, start in self._starts.items()}

    def _read_levels(
        self, world: World, observation: np.ndarray | None = None
    ) -> dict[str, float]:
        """Read the bounded level of every level term of this weighting.

        A level divides the published value by the unit the engine published
        for the field, so it lies inside the closed interval from minus one
        to one. **The unit comes from the schema.** A divisor written here
        would be a second declaration of an engine rule, and nothing would
        fail when the engine moved and this did not.
        """
        if not self._level_starts:
            return {}
        values = self._array(world, observation)
        return {
            name: self._units[name].fraction(float(values[start]))
            for name, start in self._level_starts.items()
        }


def _weight_of(weights: Mapping[str, float | None], name: str) -> float:
    """Return the weight of one shaped term, which the constructor set."""
    weight = weights[name]
    if weight is None:  # pragma: no cover - the constructor refuses this
        message = f"the weight of {name!r} is unset"
        raise UnsetWeightError(message)
    return float(weight)


def _field_units(world: World, names: Sequence[str]) -> dict[str, Scale]:
    """Return the published scale of every named field of one layout.

    A level term divides by the unit the engine wrote the field against, and
    the schema is the only declaration of that unit. A field whose form the
    schema does not state is refused rather than given a unit of one, because
    a level with no bound is a raw count and a raw count means different
    things on two worlds.

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decision D1.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """
    if not names:
        return {}
    catalogue = SignalCatalogue.of_world(world)
    units: dict[str, Scale] = {}
    for name in names:
        form = catalogue.signal(name).form
        if form is None or form.unit <= 0:
            message = (
                f"{name!r} carries no published unit, so nothing here knows "
                f"what one whole reads as. A level term needs one. The "
                f"layout publishes the forms "
                f"{sorted(catalogue.value_forms)}."
            )
            raise TermError(message)
        units[name] = Scale(
            unit=float(form.unit),
            compressed=form.invertible and form.uniform,
        )
    return units


def _field_starts(world: World, names: Sequence[str]) -> dict[str, int]:
    """Return the array position of every named field of one layout.

    The schema of the world is the only declaration of the layout, so this
    function states no position of its own.[^1]

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decision D1.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """
    schema = world.observation_schema()
    rows = {row["name"]: row for row in schema["fields"]}
    starts: dict[str, int] = {}
    for name in names:
        row = rows.get(name)
        if row is None:
            message = (
                f"{name!r} names no field of the observation schema. "
                f"The schema holds {sorted(rows)}."
            )
            raise TermError(message)
        if row["positions"] != 1:
            message = (
                f"{name!r} holds {row['positions']} positions. A term must "
                "name a field that holds exactly one position, because a "
                "weight over many positions states no position."
            )
            raise TermError(message)
        starts[name] = int(row["start"])
    return starts
