"""The search that moves the centre, behind one door.

A training run holds a centre and asks a search for candidates. The search
proposes a population, the run plays it, and the search reads the scores and
gives back a new centre. **The run never reads how the search works.** It
calls two methods and logs what the second one answers.

This module holds that door and one search behind it. The search is an
evolution strategy: it perturbs the centre in both directions along a random
direction, ranks what the population scored, and steps along the ranked sum.
A policy-gradient method or a covariance strategy is a second class here, not
a rewrite of the run.

# The noise of a generation is a function of the generation

The perturbations come from a generator keyed on the run seed and the
generation number, and never from a stream that a generation advances. A
resumed run therefore draws the perturbations the run it continues drew. A
worker process draws them from the same two numbers, so a sharded generation
builds the candidates a single process would build.[^1]

# A perturbation stays the same fraction of the centre

A perturbation of a fixed length means one thing against a short centre and
another against a long one. The search holds the fraction rather than the
length, so the population of a generation keeps its spread however far the
centre has travelled.

**Only one kind of policy reaches that fraction by normalising the centre.** A
policy kind declares whether a positive scaling of its weight vector leaves
every choice where it was. A linear policy declares that it does, because one
matrix scales every action score by one factor. The search then holds its
centre at unit length, and sigma is the fraction. A structured policy declares
that it does not, because a ``tanh`` layer moves along its curve under a
scaling and a bias term does not scale with the weights beside it. The search
then leaves that centre where it is, and it takes the fraction from the length
of the centre.

# A generation of equal scores moves nothing

The rank of a score comes from the order of a stable sort, so a set of equal
scores ranks by candidate index. The search reports that a generation carried
no information, and it leaves the centre where it was.

# References

[^1]: ADR-0194, a generation is scored in shards and combined in candidate
order, decision D2.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

import numpy as np

from .policy import LinearPolicy
from .structured import STRUCTURED_KIND, StructuredPolicy

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from .env import Env

# A policy the search can perturb. Every kind answers ``flat`` and
# ``rebuild``, so the search never asks which kind it holds.
Trainable = LinearPolicy | StructuredPolicy


@dataclass(frozen=True)
class Update:
    """What the search did with one generation of scores.

    The centre entry is where the search now stands. The spread entry is the
    highest score minus the lowest one, which is what the ranking had to rank.
    The informative entry is false when the spread carried nothing and the
    centre did not move.

    The scores entry holds the score of each candidate, in candidate order,
    and the ranks entry holds what the ranking made of them. **A run that
    logged neither could not say afterwards why the centre moved**, so both
    travel back to the caller.
    """

    centre: np.ndarray
    spread: float
    informative: bool
    scores: np.ndarray
    ranks: np.ndarray


class Optimiser(Protocol):
    """What a training run needs of a search.

    A run builds one of these, then repeats three calls for each generation:
    it proposes a population, plays it, and updates the centre from the
    scores. The generation number reaches both calls, because the search draws
    from it rather than from a stream it advances.
    """

    @property
    def population(self) -> int:
        """How many candidates one generation holds."""

    def rebuild(self, centre: np.ndarray) -> Trainable:
        """Give back the policy that one flat centre names."""

    def start(self, centre: np.ndarray) -> np.ndarray:
        """Put one flat centre into the form the search holds it in."""

    def propose(self, centre: np.ndarray, generation: int) -> list[Trainable]:
        """Give back the candidates of one generation, in candidate order."""

    def update(self, centre: np.ndarray, generation: int, scores: np.ndarray) -> Update:
        """Read what the generation scored, and say where the centre goes."""


def unit(vector: np.ndarray) -> np.ndarray:
    """Return the vector scaled to unit length, or the vector when it is zero.

    A zero vector has no direction to keep, so it comes back as it went in.
    The untrained linear centre is such a vector, and the perturbations of the
    first generation supply the first direction.

    **This function states no claim about a policy.** It scales a vector. The
    search decides which vector it may scale, and it decides that from what
    each policy kind declares about a scaling of its weights.
    """
    length = float(np.linalg.norm(vector))
    if length == 0.0:
        return vector
    return vector / length


def choice_survives_scaling(policy: Trainable) -> bool:
    """Say whether the policy chooses the same action after a positive scaling.

    **Each policy kind declares this, and the search never derives it.** A
    kind that declares true promises that multiplying its whole weight vector
    by one positive number leaves every choice where it was. The search then
    holds the centre of that kind at unit length.

    A search that read the arithmetic of each kind instead would give a new
    kind an answer nobody checked for it. That failure is silent, because a
    normalised centre still plays and still scores.
    """
    return bool(policy.CHOICE_SURVIVES_SCALING)


def centre_scale(vector: np.ndarray) -> float:
    """Return the length of the centre, or one when the centre is zero.

    A perturbation and a step are both fractions of this number. A zero centre
    states no length of its own, so the search falls back to one. Sigma then
    means for a zero centre what it means for a centre of unit length.
    """
    length = float(np.linalg.norm(vector))
    return length if length > 0.0 else 1.0


def perturbation_scale(policy: Trainable, centre: np.ndarray, sigma: float) -> float:
    """Return the length of one perturbation of this centre.

    **Sigma is a fraction of the centre and never a length.** A perturbation
    that is too small for the centre it moves gives every candidate of a
    generation the same choices. The population then has no spread, the
    ranking has nothing to rank, and the update becomes a walk driven by
    noise. That failure is silent: the run keeps printing generations, and the
    best score equals the mean because every candidate is the same policy.

    A kind whose choice survives a scaling holds its centre at unit length, so
    sigma is already the fraction. A kind whose choice does not survive a
    scaling keeps its centre where it is, so the fraction comes from the
    length of the centre.

    **The trainer and every worker process call this with the same two
    arguments**, so a sharded generation builds the candidates a single
    process would build. The shell states the kind and the centre states the
    length, and neither reads a number this module holds.
    """
    if choice_survives_scaling(policy):
        return sigma
    return sigma * centre_scale(centre)


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

    Sigma is a fraction of the centre and never a length. The policy states
    its kind and the centre states its length, so the perturbation of a kind
    that keeps an unnormalised centre stays the same fraction of it.
    """
    step = perturbation_scale(policy, centre, sigma)
    return [
        policy.rebuild(centre + sign * step * noise[index])
        for index in range(first_pair, last_pair)
        for sign in (1.0, -1.0)
    ]


def rank_shape(scores: np.ndarray) -> np.ndarray:
    """Turn raw scores into centred ranks in the range minus a half to a half.

    A rank removes the scale of the reward from the update, so one lucky
    episode cannot move the weights further than the population is wide.
    """
    order = np.argsort(np.argsort(scores))
    return order / (len(scores) - 1) - 0.5


def carries_information(spread: float) -> bool:
    """Say whether a generation of this spread can move the centre.

    The spread is the highest score of the generation minus the lowest one. A
    spread of zero means the score did not depend on the candidate, so the
    ranking ranks a set of equal numbers and the update carries nothing.

    **A ranking of equal numbers is not a ranking of ties.** The rank of a
    score comes from the order of the sort, and the sort is stable, so an
    equal set ranks by candidate index. Every plus half then ranks below its
    own minus half by the same amount, and the update becomes a fixed step
    along a direction the noise alone chose.

    A world where no policy can matter produces exactly this. One faction of
    three holds a seat, or a seat reaches no food, and the game ends the same
    way whatever any candidate does.
    """
    return spread > 0.0


def shell_policy(kind: str, probe: Env) -> Trainable:
    """Build the untrained policy of one kind, sized from the world.

    **The trainer and a worker process both build this, and they must build
    the same thing.** A worker rebuilds its candidates from the centre, so it
    needs the shell the centre was taken from. Every fixed part of a shell
    comes from one fixed seed, so two processes build one shell.

    The lengths come from the engine schemas through the probe environment.
    This module states none of its own.

    The structured kind reads the whole layout and not only the length, so it
    takes the signal catalogue of the probe. That catalogue is the schema the
    engine published, and this module states no part of it.

    **A kind sizes itself from the probe and from nothing a caller passes.**
    The builder took a hidden width while one kind held a frozen projection
    into a fixed number of units. This project removed that kind, and the
    width went with it.
    """
    if kind == STRUCTURED_KIND:
        return StructuredPolicy.of_catalogue(probe.action_length, probe.signals)
    return LinearPolicy.zeros(probe.action_length, probe.observation_length)


@dataclass(frozen=True)
class EvolutionStrategy:
    """The search this project runs: antithetic perturbations, ranked.

    The shell entry is the untrained policy the centre is a flat view of. The
    search rebuilds a candidate through it, so it never states a weight
    layout of its own.

    An evolution strategy needs no gradient through the step. It scores a
    whole episode with one number, so a long run with a sparse reward costs it
    nothing.
    """

    shell: Trainable
    pairs: int
    sigma: float
    learning_rate: float
    seed: int

    @property
    def population(self) -> int:
        """How many candidates one generation holds."""
        return 2 * self.pairs

    def rebuild(self, centre: np.ndarray) -> Trainable:
        """Give back the policy that one flat centre names."""
        return self.shell.rebuild(centre)

    @property
    def holds_unit_centre(self) -> bool:
        """Say whether this search normalises the centre it holds.

        It normalises only when the policy kind declares that a positive
        scaling of the weight vector leaves every choice where it was.

        **Normalising the centre of a kind that declares otherwise changes the
        function the policy computes.** A ``tanh`` layer sits at another place
        on its curve once its weights are scaled, and a bias term does not
        scale with the weights beside it. The mapping is then not the old
        mapping times one positive number, so the chosen action can change.
        """
        return choice_survives_scaling(self.shell)

    def start(self, centre: np.ndarray) -> np.ndarray:
        """Put one flat centre into the form the search holds it in.

        The search scales the centre of a kind whose choice survives a
        scaling to unit length. It gives back the centre of every other kind
        as it stands, because scaling that one would change the policy.

        The search never writes to a centre, so it never copies one.
        """
        if self.holds_unit_centre:
            return unit(centre)
        return centre

    def noise(self, centre: np.ndarray, generation: int) -> np.ndarray:
        """Draw the perturbations of one generation, one row for each pair."""
        return generation_noise(self.seed, generation, self.pairs, centre.size)

    def propose(self, centre: np.ndarray, generation: int) -> list[Trainable]:
        """Build the whole population of one generation, in candidate order."""
        return pair_candidates(
            self.shell,
            centre,
            self.noise(centre, generation),
            self.sigma,
            0,
            self.pairs,
        )

    def step(self, centre: np.ndarray, gradient: np.ndarray) -> np.ndarray:
        """Move the centre one learning rate along the weighted sum.

        The rank shaping already threw away the scale of the reward, so the
        length of the weighted sum carries no information worth keeping. The
        search therefore takes a step of a fixed size along the direction, and
        the learning rate is the fraction of the centre that one generation
        moves.

        A kind whose choice survives a scaling holds its centre at unit
        length. The learning rate is then already that fraction, and the
        search normalises again after the step.

        A kind whose choice does not survive a scaling takes the fraction from
        the length of the centre, and the search never normalises it. **The
        length of such a centre is a trainable quantity and not a free one.**

        That length grows over a run. A step of many thousand dimensions sits
        near a right angle to the centre, so one generation multiplies the
        length by about the square root of one plus the learning rate squared.
        A run of the length this project takes pays that as a slowly rising
        saturation of the ``tanh`` layers, and a much longer run would pay it
        as a policy that saturates. A search that ran for thousands of
        generations needs a bound on the length, and this one states none.
        """
        direction = unit(gradient)
        if self.holds_unit_centre:
            return unit(centre + self.learning_rate * direction)
        return centre + self.learning_rate * centre_scale(centre) * direction

    def update(self, centre: np.ndarray, generation: int, scores: np.ndarray) -> Update:
        """Rank the scores, step along the ranked sum, and report both.

        A generation of equal scores moves nothing. The rank of an equal score
        is the index of the candidate, so the ranking would give every plus
        half a lower rank than its own minus half. The weighted sum of the
        perturbations is then a direction drawn from the noise alone, and the
        step would move the centre as far as an informed generation moves it,
        in a direction no episode chose.
        """
        spread = float(scores.max() - scores.min())
        informative = carries_information(spread)
        if not informative:
            return Update(
                centre=centre,
                spread=spread,
                informative=False,
                scores=scores,
                ranks=np.zeros_like(scores),
            )
        ranks = rank_shape(scores)
        noise = self.noise(centre, generation)
        gradient = np.zeros_like(centre)
        for index in range(self.pairs):
            weight = ranks[2 * index] - ranks[2 * index + 1]
            gradient += weight * noise[index]
        moved = self.step(centre, gradient)
        return Update(
            centre=moved,
            spread=spread,
            informative=True,
            scores=scores,
            ranks=ranks,
        )


__all__ = [
    "EvolutionStrategy",
    "Optimiser",
    "Trainable",
    "Update",
    "carries_information",
    "centre_scale",
    "choice_survives_scaling",
    "generation_noise",
    "pair_candidates",
    "perturbation_scale",
    "rank_shape",
    "shell_policy",
    "unit",
]
