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

# A perturbation is the same fraction of every layer, and not of the whole

A policy of one layer is a flat vector and nothing more. A policy of many
layers holds each of them at its own scale, because each is drawn against its
own fan-in, and that scale spans a factor of seven in the structured kind.

An isotropic perturbation over the flat vector moves every weight the same
distance, whatever layer it sits in. The initial scale of a layer then decides
how far the search can revise it: a layer drawn small for each weight is
rewritten, and a layer drawn large for each weight barely turns. A measurement
found the two geometric towers at the large end, and those towers are the part
of the design the structured shape exists for.[^2]

The search therefore scales the perturbation of each layer by the scale of
that layer. **Sigma keeps the meaning it had.** The whole perturbation still
has the length sigma names, and sigma is now the fraction of each layer rather
than the fraction of the flat vector alone.

**A figure measured against the old draw does not transfer.** One measurement
picked the radius this module runs by reading the spread the candidate scores
reached at four radii, and it read them under an isotropic draw.[^3] The length
of a perturbation is what it was, and the neighbourhood the population covers
is not. Measure the spread again before anyone reads that table as current.

# A tie states no order, and the candidate index is not a neutral order

The rank of a tied score is the mean of the positions the tied scores occupy.
Two candidates that scored the same number therefore reach the step with the
same rank. A stable sort would have ranked them by candidate index instead,
which puts an ordering the search invented into the direction it steps along.

A generation whose candidates all tie reports that it carried no information,
and it leaves the centre where it was.

# The length of a step is what the generation agreed on

Rank shaping throws away the scale of the reward. It does not follow that the
search should throw away the agreement between the candidates as well. The
search reads that agreement out of the ranks, and it moves the centre by the
learning rate times the agreement.

# References

[^1]: ADR-0194, a generation is scored one episode at a time, and
combined in candidate order, decision D2.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
[^2]: Findings register, FND-713. ``docs/FINDINGS.md``
[^3]: Findings register, FND-711. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

import numpy as np

from .policy import FeatureNormalizer, LinearPolicy
from .structured import STRUCTURED_KIND, StructuredPolicy

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from .env import Env

# A policy the search can perturb. Every kind answers ``flat``, ``rebuild``
# and ``shapes``, so the search never asks which kind it holds. The shapes
# state where each layer of the flat vector sits.
Trainable = LinearPolicy | StructuredPolicy

NORM_CEILING_OVER_SHELL = 2.0
"""The largest length an unnormalised centre may reach, over the shell length.

The search states the reasoning where it applies the bound.
"""


@dataclass(frozen=True)
class Update:
    """What the search did with one generation of scores.

    The centre entry is where the search now stands. The spread entry is the
    highest score minus the lowest one, which is what the ranking had to rank.
    The informative entry is false when the centre did not move.

    The scores entry holds the score of each candidate, in candidate order,
    and the ranks entry holds what the ranking made of them. **A run that
    logged neither could not say afterwards why the centre moved**, so both
    travel back to the caller.

    The agreement entry is the fraction of the learning rate that this
    generation moved the centre by, and it is the quantity a reader watches to
    see whether a run is climbing or wandering. The alignment entry is the
    cosine between the step and the direction the search is trying to find. It
    follows from the pair count and the trainable count alone, so it is the
    same number every generation of one run, and the run reports it beside the
    agreement because the product of the two is what the centre gains.
    """

    centre: np.ndarray
    spread: float
    informative: bool
    scores: np.ndarray
    ranks: np.ndarray
    agreement: float
    alignment: float


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


def layer_sizes(policy: Trainable) -> tuple[int, ...]:
    """How many weights each layer holds, in the order the flat vector holds them.

    **The boundaries come from the policy and never from this module.** The
    policy states the shape of every trainable array, in the order it lays
    them out, and that statement is the one the policy itself reads to cut a
    flat vector back into arrays. A second statement here would agree on the
    day it was written, and nothing would fail on the day a layer moved.

    The order is the order of the arrays and never the order of a mapping, so
    two processes that hold one shell answer this identically.
    """
    return tuple(int(np.prod(shape)) for shape in policy.shapes)


def layer_scale(block: np.ndarray, fallback: float) -> float:
    """Return the scale of one layer, as its root mean square for each weight.

    **The scale is the one the layer holds now and not the one it started
    at.** A layer that grew holds more of what the run learned, and the search
    revises it by the same fraction of what it now is. That rule is the rule
    this module already applies to the whole centre, at the granularity of one
    layer: a perturbation stays the same fraction of what it moves. An initial
    scale would state a second quantity, taken from a shell, that no later
    generation reads.

    **A layer of zeros states no scale of its own, so it takes the
    fallback.** The readout of the untrained structured policy is such a
    layer, and so is every bias array. A perturbation proportional to a zero
    scale is zero, the layer stays zero, and the zero is then a fixed point
    the search can never leave.
    """
    length = float(np.linalg.norm(block))
    if length == 0.0:
        return fallback
    return length / math.sqrt(block.size)


def layer_weighting(policy: Trainable, centre: np.ndarray) -> np.ndarray:
    """Return the multiplier each coordinate of a perturbation carries.

    The search draws a perturbation isotropically and multiplies it by this,
    so a layer takes a step in proportion to its own scale rather than in
    proportion to its weight count. **That is the property the search
    lacked.** A measurement found each layer taking the share of the squared
    step length that its weight count predicts, to three parts in a thousand,
    and it found every weight of the policy moving the same distance. The
    initial scale of a layer then decided how far the search could revise it,
    and that scale spans a factor of seven.[^1]

    Fan-in initialisation is not the defect, and this changes nothing about
    it. A layer drawn against its fan-in holds the activations of a ``tanh``
    in range, and one initial scale over every layer would push the early
    layers off their curves. The quantity to equalise is the fraction of
    itself that the search can revise a layer by, and that is what this
    equalises.

    A layer of zeros takes the root mean square for each weight of the whole
    centre. **The absolute scale of the readout carries no behaviour**: it is
    the last layer, so multiplying it scales every action score by one factor
    and the highest legal row stays the highest. What matters in the readout
    is the direction, and the direction is what the fallback lets the search
    find.

    **A layer that starts small stays small beside the layers around it.**
    Every layer grows by the same fraction under this rule, so the ratio
    between two layers holds over a run. That is what the rule is for, and it
    is a cost for a bias array that starts at zero: such an array reaches the
    scale of the weights it is added to more slowly than an isotropic step
    would take it there. The bias arrays of this policy hold a few weights
    each, and no measurement covers the trade.

    The result is divided by its largest entry. The caller normalises each
    perturbation to unit length, so any overall factor here is removed there.
    **The divisor exists so that a policy of one layer takes exactly one.**
    Such a kind then multiplies its draw by 1.0 and keeps the perturbation it
    drew before this rule existed, bit for bit.

    Raises ``ValueError`` when the layers do not cover the centre. That is the
    layout of the policy disagreeing with the vector the search holds, and a
    silent answer there would weight the wrong coordinates.

    References
    ----------
    [^1]: Findings register, FND-713. ``docs/FINDINGS.md``
    """
    sizes = layer_sizes(policy)
    covered = int(sum(sizes))
    if covered != centre.size:
        message = (
            f"the policy lays its weights out in layers of {covered} and the "
            f"search holds a centre of {centre.size}"
        )
        raise ValueError(message)
    fallback = centre_scale(centre) / math.sqrt(centre.size)
    scales = np.empty(len(sizes), dtype=np.float64)
    walked = 0
    for index, size in enumerate(sizes):
        scales[index] = layer_scale(centre[walked : walked + size], fallback)
        walked += size
    weighting = np.repeat(scales, sizes)
    return np.asarray(weighting / weighting.max())


def generation_noise(
    seed: int, generation: int, pairs: int, policy: Trainable, centre: np.ndarray
) -> np.ndarray:
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

    **The draw is isotropic, and the layers weight it afterwards.** The policy
    and the centre reach this function rather than a length, so no caller can
    draw a perturbation that ignores the layers. The generator reads the run
    seed and the generation alone, so the weighting changes where a
    perturbation points and never which numbers the generator produced.
    """
    rng = np.random.default_rng([seed, generation])
    weighting = layer_weighting(policy, centre)
    noise = rng.standard_normal((pairs, weighting.size)) * weighting
    return np.asarray(noise / np.linalg.norm(noise, axis=1, keepdims=True))


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

    **A set of tied scores takes the mean of the positions it occupies.** A
    tie states that the two candidates scored the same, and nothing more. The
    order of a stable sort states more than that: it orders the tied
    candidates by candidate index, and the candidate index is a number the
    search chose when it drew the perturbations. That number then reaches the
    direction the centre steps along.

    **A partial tie is the common case, and it is the expensive one.** A
    measurement of one generation of eight candidates found two of them
    scoring the same number to six decimal places, at two of the four sigmas
    it tried.[^1] The whole-generation guard passed each of those
    generations, so nothing saw the tie, and the index order reached the step
    every time.

    A mean rank is the neutral answer. Two tied candidates get one rank, so
    the pair of each of them carries the same weight whichever slot it sits
    in. A tie inside one pair then gives that pair a weight of zero, which
    says that the pair separated nothing.

    References
    ----------
    [^1]: Report on how sigma trades against the worlds each candidate plays,
    section 5. ``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
    """
    count = len(scores)
    order = np.argsort(scores, kind="stable")
    position = np.empty(count, dtype=np.float64)
    position[order] = np.arange(count, dtype=np.float64)
    _, group = np.unique(scores, return_inverse=True)
    group = group.reshape(-1)
    total = np.bincount(group, weights=position)
    members = np.bincount(group)
    return (total / members)[group] / (count - 1) - 0.5


def pair_weights(ranks: np.ndarray) -> np.ndarray:
    """Return what each antithetic pair says, as the plus rank less the minus.

    Candidate ``2 * pair`` is the plus half and ``2 * pair + 1`` is the minus
    half, so the difference is positive when the plus direction scored higher.
    The magnitude says how far apart the ranking put the two halves of one
    perturbation.
    """
    return np.asarray(ranks[0::2] - ranks[1::2])


def noise_agreement_sum(pairs: int) -> float:
    """Return the squared pair-weight sum a ranking of pure noise reaches.

    A generation whose scores carry nothing ranks its candidates in an order
    the noise chose, so the ranks are a random permutation of the shaped
    values. The mean of a squared rank difference over two distinct positions
    of that permutation is a closed form in the population size, so this
    function measures nothing and states no figure of its own.

    The shaped ranks are the positions divided by one less than the
    population, less a half. Their variance is the population plus one over
    twelve times the population less one. Two distinct positions of one
    permutation correlate by minus one over the population less one, so the
    mean squared difference is twice the variance times the population over
    the population less one. One generation holds that many pairs.
    """
    population = 2 * pairs
    variance = (population + 1) / (12.0 * (population - 1))
    difference = 2.0 * variance * population / (population - 1)
    return pairs * difference


def perfect_agreement_sum(pairs: int) -> float:
    """Return the squared pair-weight sum a perfectly split ranking reaches.

    A generation carries the most a ranking can carry when the two halves of
    every pair sit at opposite ends of the order. The pair weights are then
    the odd numbers up to one less than the population, divided by one less
    than the population, and the sum of their squares is a closed form.
    """
    population = 2 * pairs
    odd = np.arange(1, population, 2, dtype=np.float64) / (population - 1)
    return float(odd @ odd)


def generation_agreement(ranks: np.ndarray) -> float:
    """Return how far the candidates of one generation agreed, from zero to one.

    The value is one when the two halves of every pair sit at opposite ends of
    the ranking, which is what a ranking driven by one direction produces. It
    is zero when the pair weights reach only what a ranking of pure noise
    reaches. The search multiplies the learning rate by this number, so a
    generation whose candidates disagree moves the centre less than one whose
    candidates agree.

    **The statistic is the squared sum of the pair weights, and it is a
    statistic of the ranks alone.** One register measured the length of the
    summed perturbation against the floor that near-orthogonal directions put
    under it, and found that ratio to be one within a few parts in a hundred
    for pure noise, for perfect signal and for a whole generation of ties.
    **That measurement rules out the geometric length as a scale**, and it
    says nothing against the floor itself.[^1] The floor is the squared sum of
    the pair weights, which is a rank statistic, and this function compares it
    against the two constants that bound it.

    Both bounds follow from the population size and from nothing a generation
    scored, so neither is a measured figure. The value is clipped, because a
    ranking of noise falls below the noise expectation about half the time.

    **One pair states no agreement.** A single pair always reaches both bounds
    at once, because its two halves are the whole population and its weight is
    fixed. Such a generation therefore takes the whole learning rate.

    References
    ----------
    [^1]: Findings register, FND-668. ``docs/FINDINGS.md``
    """
    pairs = ranks.size // 2
    if pairs < 2:
        return 1.0
    weights = pair_weights(ranks)
    floor = noise_agreement_sum(pairs)
    ceiling = perfect_agreement_sum(pairs)
    reached = float(weights @ weights)
    return float(np.clip((reached - floor) / (ceiling - floor), 0.0, 1.0))


def step_alignment(pairs: int, trainable: int) -> float:
    """Return the cosine between the step of one generation and the truth.

    Sampling a fixed number of directions in a space of higher dimension
    estimates the direction of steepest ascent to an accuracy that falls as
    the dimension rises. One register measured the law over two orders of
    magnitude: **the cosine is near the square root of the pair count divided
    by the trainable count.** The same measurement found that the noise of
    scoring on one world halves it.[^1]

    The figure follows from the population and the policy shape alone. No
    other knob of a run changes it, and doubling it needs four times the
    population. A run that reports it can see before it spends whether its
    steps point anywhere.

    References
    ----------
    [^1]: Findings register, FND-668. ``docs/FINDINGS.md``
    """
    if pairs <= 0 or trainable <= 0:
        return 0.0
    return math.sqrt(pairs / trainable)


def generations_before_a_climb_beats_a_wander(alignment: float) -> int:
    """Return the generations a run needs before the climb passes the wander.

    Each step turns the centre by one angle. The part of that turn which
    points at the truth accumulates, so it grows with the generation count
    times the alignment. The part which does not point at the truth
    accumulates as a random walk, so it grows with the square root of the
    generation count. The two are equal when the generation count reaches one
    over the square of the alignment.

    A run shorter than that answer spends more of its travel on wander than on
    climb. **The answer is optimistic by a factor of four**, because the noise
    of scoring halves the alignment and this function takes the alignment it is
    given.
    """
    if alignment <= 0.0:
        return 0
    return math.ceil(1.0 / (alignment * alignment))


WORLDS_A_SIGMA_NEEDS = {0.1: 35, 0.25: 6, 0.5: 3, 1.5: 6}
"""How many worlds one candidate needs, for each sigma a measurement covered.

A ranking can separate a candidate from its neighbours when the spread between
the candidates exceeds the noise on one candidate's score. The figure is the
square of the residual standard deviation over the square of the signal
standard deviation, rounded up, and one measurement supplied both.[^1]

References
----------
[^1]: Report on how sigma trades against the worlds each candidate plays,
sections 4 and 6.
``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
"""


def worlds_a_sigma_needs(sigma: float) -> tuple[float, int]:
    """Return the measured sigma nearest this one, and the worlds it needs.

    A ranking needs the spread between the candidates to exceed the noise on
    the score of one candidate. More worlds for each candidate lower that
    noise, and a larger sigma raises both the spread and the noise. The number
    of worlds a sigma needs is therefore a measured quantity and not a derived
    one.

    **The need does not rise with sigma.** It falls and then rises again,
    because the spread stops growing above the middle of the measured range
    while the noise keeps growing. So no interpolation between two measured
    sigmas is safe, and this function answers with the nearest measured sigma
    on a logarithmic scale. It gives that sigma back beside the count, so a
    caller can say which measurement it read. The table beside this function
    holds the counts.

    A sigma of zero or below has no logarithm and no meaning, so it takes the
    smallest sigma the measurement covered.
    """
    if sigma <= 0.0:
        nearest = min(WORLDS_A_SIGMA_NEEDS)
        return nearest, WORLDS_A_SIGMA_NEEDS[nearest]
    nearest = min(
        WORLDS_A_SIGMA_NEEDS, key=lambda measured: abs(math.log(sigma / measured))
    )
    return nearest, WORLDS_A_SIGMA_NEEDS[nearest]


def configuration_notes(
    sigma: float, worlds: int, pairs: int, trainable: int, generations: int
) -> list[str]:
    """Say what this configuration can and cannot reach, before a run spends.

    **A run that is under-sampled for its sigma must say so on its first
    lines.** The configuration the project ran before this gave each candidate
    fewer worlds than its sigma needed, so every generation of it ranked
    candidates on too few worlds. Nothing said so, and the run finished before
    anyone derived the number.[^1]

    The notes also state the alignment of one step and the generations the run
    needs before its climb passes its wander. **Neither note fails a run.** A
    figure a reader can act on is worth more than a refusal, because the
    reader may want the run anyway.

    References
    ----------
    [^1]: Report on how sigma trades against the worlds each candidate plays,
    section 7.
    ``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
    """
    notes = []
    nearest, needed = worlds_a_sigma_needs(sigma)
    given = "1 world" if worlds == 1 else f"{worlds} worlds"
    notes.append(
        f"sigma {sigma:g} needs {needed} worlds for each candidate, "
        f"measured at sigma {nearest:g}, and this run gives {worlds}"
    )
    if worlds < needed:
        notes.append(
            f"this run is under-sampled for its sigma: the spread between "
            f"the candidates does not exceed the noise on one candidate at "
            f"{given}"
        )
    alignment = step_alignment(pairs, trainable)
    breaks = generations_before_a_climb_beats_a_wander(alignment)
    notes.append(
        f"one step of {pairs} pairs over {trainable} trainable weights aligns "
        f"{alignment:.4f} with the truth, and scoring noise halves that"
    )
    if generations < breaks:
        notes.append(
            f"this run is mostly wander: the climb passes the wander after "
            f"{breaks} generations and this run asks for {generations}"
        )
    return notes


def carries_information(spread: float) -> bool:
    """Say whether a generation of this spread can move the centre.

    The spread is the highest score of the generation minus the lowest one. A
    spread of zero means the score did not depend on the candidate, so the
    ranking ranks a set of equal numbers and the update carries nothing.

    A world where no policy can matter produces exactly this. One faction of
    three holds a seat, or a seat reaches no food, and the game ends the same
    way whatever any candidate does.

    **This guard is no longer the only defence, and it is still worth having.**
    A ranking that gives a tied score the mean of its positions gives every
    candidate of an equal generation one rank, so every pair weight is zero
    and the agreement is zero as well. The guard saves the draw of the
    perturbations, and it names the generation in the log, which the agreement
    alone would not.
    """
    return spread > 0.0


def shell_policy(
    kind: str, probe: Env, normalizer: FeatureNormalizer | None = None
) -> Trainable:
    """Build the untrained policy of one kind, sized from the world.

    **The trainer and a worker process both build this, and they must build
    the same thing.** A worker rebuilds its candidates from the centre, so it
    needs the shell the centre was taken from. Every fixed part of a shell
    comes from one fixed seed, so two processes build one shell.

    **The normalizer is an argument and never a derivation here.** It comes
    from a reference sample of played episodes, and a worker process must
    read the one the trainer derived rather than one of its own. A shell that
    holds none reads the plain squash.

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
        return StructuredPolicy.of_catalogue(
            probe.action_length, probe.signals, normalizer=normalizer
        )
    return LinearPolicy.zeros(probe.action_length, probe.observation_length, normalizer)


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
        return generation_noise(self.seed, generation, self.pairs, self.shell, centre)

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

    @property
    def norm_ceiling(self) -> float:
        """Return the largest length the search lets an unnormalised centre reach.

        A kind whose choice survives a scaling holds its centre at unit length,
        so this bound governs no such kind and the search never applies it
        there.

        A kind whose choice does not survive a scaling keeps the length its
        centre reached, and that length rises over a run. A step of many
        thousand dimensions sits near a right angle to the centre, so one
        generation multiplies the length by about the square root of one plus
        the learning rate squared. **A rising length saturates the ``tanh``
        layers**, so a late generation reads a coarser function of the
        observation than an early one, and that works against what the run is
        trying to learn.

        The bound is a multiple of the length of the untrained shell. The
        shell comes from one fixed seed, so the trainer and every worker
        process derive the same bound from the same shell, and no second
        declaration site holds it.

        **The multiple is the growth of a run whose cost the project has
        seen.** One audit measured a run of twenty generations at a learning
        rate of 0.3 and found the length grew by 2.37, and it called the
        saturation of that run modest.[^1] A bound of twice the shell keeps
        every run inside the range that audit covered. A larger bound would
        let a long run reach a saturation nobody has measured.

        References
        ----------
        [^1]: Report on what is wrong with training and evaluation, item 8.
        ``docs/research/what-is-wrong-with-training-and-evaluation.md``
        """
        return NORM_CEILING_OVER_SHELL * centre_scale(self.shell.flat())

    def bounded(self, centre: np.ndarray) -> np.ndarray:
        """Scale the centre back to the ceiling when the step took it past.

        The direction of the centre is what the run trained, so the bound
        keeps it and changes the length alone.

        **A resumed centre longer than the ceiling comes back to the ceiling
        on its first step.** That changes the function the policy computes,
        which is the price of any bound on a kind whose choice does not
        survive a scaling. Such a centre is the saturated case this bound
        exists to prevent, so the search shortens it rather than leaving it.
        """
        length = float(np.linalg.norm(centre))
        ceiling = self.norm_ceiling
        if length <= ceiling:
            return centre
        return centre * (ceiling / length)

    def step(
        self, centre: np.ndarray, gradient: np.ndarray, agreement: float
    ) -> np.ndarray:
        """Move the centre along the summed direction, as far as the agreement says.

        The rank shaping throws away the scale of the reward, and that is
        correct: one lucky episode must not move the weights further than the
        population is wide. **It does not follow that the search should throw
        away how far the candidates agreed.** The search took a step of a fixed
        length before this, so a generation that pointed a little of the way
        toward the truth moved the centre exactly as far as one that pointed
        perfectly. An audit derived what that cost: over twenty generations at
        a learning rate of 0.3 the directed part of the travel was 15.8 degrees
        and the undirected part was 74.7, so the centre wandered 4.7 times
        further than it climbed.[^1]

        The agreement is a number from zero to one, and the search multiplies
        the learning rate by it. **The learning rate therefore keeps its
        meaning and becomes a bound**: it is the largest fraction of the
        centre that one generation may move, and a generation reaches it only
        by splitting every pair to the ends of the ranking. One lucky
        generation cannot throw the centre further than that.

        A kind whose choice survives a scaling holds its centre at unit
        length. The learning rate is then already the fraction, and the search
        normalises again after the step.

        A kind whose choice does not survive a scaling takes the fraction from
        the length of the centre, and the search holds that length under a
        ceiling rather than letting it rise over a run.

        References
        ----------
        [^1]: Report on what is wrong with training and evaluation, items 5 and
        13. ``docs/research/what-is-wrong-with-training-and-evaluation.md``
        """
        direction = unit(gradient)
        travel = self.learning_rate * agreement
        if self.holds_unit_centre:
            return unit(centre + travel * direction)
        return self.bounded(centre + travel * centre_scale(centre) * direction)

    def update(self, centre: np.ndarray, generation: int, scores: np.ndarray) -> Update:
        """Rank the scores, step as far as they agreed, and report both.

        A generation of equal scores moves nothing. The spread guard stops it
        first, and the ranking would stop it as well: every candidate takes
        the mean of the positions the tied scores occupy, so every pair weight
        is zero and the generation agreed on nothing.

        The step is the learning rate times the agreement, so a generation
        whose candidates disagree moves the centre less than one whose
        candidates agree. A generation that reaches only what pure noise
        reaches moves the centre nowhere, and the report says so.

        **A generation that moves nothing gives back the centre it was
        given**, and never a rescaled copy of it. A rescaled copy differs in
        the last bits, and a run that moved in the last bits no longer
        compares against a stored score.
        """
        alignment = step_alignment(self.pairs, centre.size)
        spread = float(scores.max() - scores.min())
        if not carries_information(spread):
            return Update(
                centre=centre,
                spread=spread,
                informative=False,
                scores=scores,
                ranks=np.zeros_like(scores),
                agreement=0.0,
                alignment=alignment,
            )
        ranks = rank_shape(scores)
        agreement = generation_agreement(ranks)
        if agreement <= 0.0:
            return Update(
                centre=centre,
                spread=spread,
                informative=False,
                scores=scores,
                ranks=ranks,
                agreement=0.0,
                alignment=alignment,
            )
        noise = self.noise(centre, generation)
        gradient = np.zeros_like(centre)
        for index in range(self.pairs):
            weight = ranks[2 * index] - ranks[2 * index + 1]
            gradient += weight * noise[index]
        moved = self.step(centre, gradient, agreement)
        return Update(
            centre=moved,
            spread=spread,
            informative=True,
            scores=scores,
            ranks=ranks,
            agreement=agreement,
            alignment=alignment,
        )


__all__ = [
    "NORM_CEILING_OVER_SHELL",
    "WORLDS_A_SIGMA_NEEDS",
    "EvolutionStrategy",
    "Optimiser",
    "Trainable",
    "Update",
    "carries_information",
    "centre_scale",
    "choice_survives_scaling",
    "configuration_notes",
    "generation_agreement",
    "generation_noise",
    "generations_before_a_climb_beats_a_wander",
    "layer_scale",
    "layer_sizes",
    "layer_weighting",
    "noise_agreement_sum",
    "pair_candidates",
    "pair_weights",
    "perfect_agreement_sum",
    "perturbation_scale",
    "rank_shape",
    "shell_policy",
    "step_alignment",
    "unit",
    "worlds_a_sigma_needs",
]
