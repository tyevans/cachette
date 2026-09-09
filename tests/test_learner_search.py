"""The search must draw from the generation number, and must refuse a tie.

The search sits behind one door, so a run can hold another search without a
rewrite. These tests drive that door rather than the internals, and they check
the properties the run depends on: the perturbations are a function of the run
seed and the generation number, and a generation of equal scores moves the
centre nowhere.

# A policy kind declares whether a scaling moves its choice

The search once normalised every centre it held. It did that on the claim that
a policy chooses by the highest score, so a positive scaling of every weight
leaves the choice where it was. **That claim holds for a linear policy and
fails for a structured one.** A structured policy ends every tower in a
``tanh`` and holds bias arrays, so a scaling puts each saturating layer at
another place on its curve.

Each kind therefore declares the property, and the search normalises only the
centre of a kind that declares it. The tests below assert the declaration of
each kind against the choices that kind makes, and they assert that the search
holds the linear trajectory exactly where it was.[^1]

# A step is as long as the generation agreed

The search took a step of a fixed length before this, so a generation that
pointed a little of the way toward the truth moved the centre as far as one
that pointed perfectly. The step now scales by how far the two halves of each
antithetic pair sat apart in the ranking. The tests below build one generation
that agrees and one that agrees less, and they assert that the first moves the
centre further.

# A tie states no order

The rank of a tied score is the mean of the positions the tied scores occupy.
A stable sort would order the tied candidates by candidate index instead, and
that index is a number the search chose. The tests below build a partial tie
and assert that the ranking does not read the index. One of them puts the
stable sort back and asserts that the ranking then does read it.

References
----------
[^1]: Testing Rules, sections 1, 2 and 2a. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette._core import World
from cachette.learn import search as search_module
from cachette.learn.layout import ObservationLayout
from cachette.learn.policy import LinearPolicy
from cachette.learn.search import (
    EvolutionStrategy,
    Optimiser,
    Trainable,
    choice_survives_scaling,
    configuration_notes,
    generation_noise,
    generations_before_a_climb_beats_a_wander,
    layer_sizes,
    layer_weighting,
    rank_shape,
    step_alignment,
    unit,
)
from cachette.learn.signals import SignalCatalogue
from cachette.learn.structured import StructuredPolicy

ACTIONS = 6
FEATURES = 5

SPREAD_SCORES = np.array([5.0, 1.0, 4.0, 2.0, 3.0, 0.0])
"""The scores of the generation that the pins below act on.

Six scores are three pairs, and no two of them are equal, so the ranking
carries a direction.
"""

SPREAD_RANKS = np.array([5, 1, 4, 2, 3, 0]) / (SPREAD_SCORES.size - 1) - 0.5
"""The centred ranks of those scores, stated as data.

**Nothing computes this with the function under test.** The integers are the
ascending position of each score, and the division centres them on zero. A
rank shaping that returned a constant would satisfy an expectation it computed
for itself, because every pair difference of a constant is zero and the
gradient would then be zero as well.[^1]

References
----------
[^1]: Findings register, FND-709. ``docs/FINDINGS.md``
"""


SPREAD_AGREEMENT = 0.32 / 0.56
"""How far the generation above agreed, stated as arithmetic and not as a call.

**Nothing here calls the functions under test.** The pair weights of the ranks
above are 0.8, 0.4 and 0.6, so the sum of their squares is 1.16.

A ranking of pure noise reaches 0.84 over three pairs. The shaped ranks of six
candidates have a variance of seven sixtieths, two distinct positions of one
permutation correlate by minus a fifth, so a mean squared difference is 0.28,
and three pairs hold three of them.

A perfect split reaches 1.4 over three pairs. The pair weights are then a
fifth, three fifths and one, and the sum of their squares is 1.4.

The agreement is 1.16 less 0.84, over 1.4 less 0.84.
"""

AGREED_SCORES = np.array([5.0, 0.0, 4.0, 1.0, 3.0, 2.0])
"""The scores of a generation whose candidates agree as far as they can.

Each pair holds one candidate from the top of the order and one from the
bottom, which is what a ranking driven by one direction produces. The
agreement is therefore one, and this generation takes the whole learning rate.

**A generation of ascending scores is the opposite case and not a neutral
one.** Ascending scores put the two halves of every pair beside each other in
the order, so every pair weight is the smallest it can be. A fixture of
ascending scores therefore measures a generation that agreed on nothing, and
several tests here used one.
"""

DISAGREED_SCORES = np.array([5.0, 4.0, 3.0, 2.0, 1.0, 0.0])
"""The scores of a generation that agrees no better than noise agrees.

Every pair weight is a fifth, so the sum of their squares is 0.12 against the
0.84 that a ranking of pure noise reaches. The agreement is therefore zero and
the centre does not move.
"""

PARTIAL_TIE_SCORES = np.array([5.0, 1.0, 3.0, 2.0, 3.0, 0.0])
"""The scores of a generation where two candidates tie and four do not.

Candidate two and candidate four hold the same score. They sit in different
pairs, so a ranking that ordered them by candidate index would put a number
the search chose into two different pair weights.

**A measurement found this case twice in one small sweep**, so it is the
common case and not a corner.[^1]

References
----------
[^1]: Report on how sigma trades against the worlds each candidate plays,
section 5.
``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
"""

TIE_STEP = 1e-6
"""How far a test moves one tied score to break the tie.

A tie has two ways to break, and a neutral ranking sits between them. The step
is small, so the two broken rankings differ from the tied one in the tie alone.
"""


def index_order_ranks(scores: np.ndarray) -> np.ndarray:
    """Rank the scores the way a stable sort ranked them before this change.

    **This is the defect, restated so that a test can put it back.** A stable
    sort gives a tied score the position it occupies, so the candidate index
    decides which of two tied candidates ranks higher. A test that never sees
    this behaviour cannot say that the ranking under test avoids it.
    """
    order = np.argsort(np.argsort(scores, kind="stable"), kind="stable")
    return order / (len(scores) - 1) - 0.5


def normalised(vector: np.ndarray) -> np.ndarray:
    """Return the vector at unit length, without the function under test.

    The pins below assert an exact equality against an expression the tests
    build. An expression built from the search would hold whatever the search
    now does, so the tests state the arithmetic instead.
    """
    return vector / float(np.linalg.norm(vector))


def a_search(seed: int = 7, sigma: float = 0.5, rate: float = 0.3) -> Optimiser:
    """Build the search over a small linear policy, behind the door.

    Most tests here drive the door and never the class, so this states the
    protocol as its type. A test that pins the arithmetic of one search asks
    for that search by name instead.
    """
    return a_strategy(seed, sigma, rate)


def a_strategy(
    seed: int = 7, sigma: float = 0.5, rate: float = 0.3
) -> EvolutionStrategy:
    """Build the evolution strategy over a small linear policy, by name."""
    return EvolutionStrategy(
        shell=LinearPolicy.zeros(ACTIONS, FEATURES),
        pairs=3,
        sigma=sigma,
        learning_rate=rate,
        seed=seed,
    )


def a_centre(search: Optimiser) -> np.ndarray:
    """Build a centre of unit length for that search to move."""
    size = LinearPolicy.zeros(ACTIONS, FEATURES).flat().size
    draw = np.random.default_rng(11).standard_normal(size)
    return search.start(draw)


def test_the_population_is_the_pair_count_twice() -> None:
    """Each perturbation is tried in both directions, so a pair is two rows."""
    search = a_search()
    centre = a_centre(search)
    assert search.population == 6
    assert len(search.propose(centre, 0)) == search.population


def test_the_perturbations_are_a_function_of_the_generation() -> None:
    """A resumed run must draw what the run it continues drew.

    A single stream advanced by each generation would give a resumed run
    other perturbations, so the resume would silently be another experiment.
    """
    search = a_search()
    centre = a_centre(search)
    first = [row.flat() for row in search.propose(centre, 4)]
    again = [row.flat() for row in search.propose(centre, 4)]
    for one, other in zip(first, again, strict=True):
        assert np.array_equal(one, other)


def test_another_generation_draws_another_population() -> None:
    """A search that ignored the generation would repeat one population."""
    search = a_search()
    centre = a_centre(search)
    first = search.propose(centre, 4)[0].flat()
    later = search.propose(centre, 5)[0].flat()
    assert not np.allclose(first, later)


def test_another_run_seed_draws_another_population() -> None:
    """Two runs of one machine must not search the same neighbourhood."""
    centre = a_centre(a_search())
    first = a_search(seed=7).propose(centre, 0)[0].flat()
    other = a_search(seed=8).propose(centre, 0)[0].flat()
    assert not np.allclose(first, other)


def test_a_generation_of_equal_scores_moves_no_centre() -> None:
    """The rank of an equal score is the index of the candidate.

    The ranking would then give every plus half a lower rank than its own
    minus half, and the step would follow a direction the noise alone chose.
    """
    search = a_search()
    centre = a_centre(search)
    update = search.update(centre, 0, np.full(search.population, 3.0))
    assert not update.informative
    assert update.spread == 0.0
    assert np.array_equal(update.centre, centre)


def test_a_generation_with_a_spread_moves_the_centre_and_says_why() -> None:
    """The caller logs the spread, the scores and the ranks it acted on.

    The ranks come from a stated array and not from the shaping function, so a
    shaping that returned a constant fails here.
    """
    search = a_search()
    centre = a_centre(search)
    scores = SPREAD_SCORES
    update = search.update(centre, 0, scores)
    assert update.informative
    assert update.spread == pytest.approx(5.0)
    assert np.array_equal(update.scores, scores)
    assert np.array_equal(update.ranks, SPREAD_RANKS)
    assert not np.allclose(update.centre, centre)
    assert float(np.linalg.norm(update.centre)) == pytest.approx(1.0)


def test_the_centre_keeps_unit_length_over_many_generations() -> None:
    """A growing norm turns a fixed perturbation into a smaller and smaller turn.

    Every candidate then chooses the same actions, the population has no
    spread, and the update becomes a walk driven by noise. The failure is
    silent, so the length is asserted rather than watched.
    """
    search = a_search()
    centre = a_centre(search)
    for generation in range(6):
        update = search.update(centre, generation, AGREED_SCORES)
        assert update.agreement == pytest.approx(1.0), (
            "the fixture agreed on nothing, so the centre never moved"
        )
        centre = update.centre
        assert float(np.linalg.norm(centre)) == pytest.approx(1.0)


def test_a_zero_centre_keeps_its_zero() -> None:
    """The first generation starts from zero, and zero has no direction."""
    zero = np.zeros(4)
    assert np.array_equal(unit(zero), zero)


# The world the structured tests read a layout from. It is small, because
# every assertion below is about arithmetic over one weight vector and not
# about the play. **The layout comes from the schema the engine publishes**,
# so no test here states a ring count or a channel count.
LAYOUT_WIDTH = 24
LAYOUT_HEIGHT = 24
LAYOUT_FACTIONS = 3
LAYOUT_SEED = 7

# How many action rows the structured tests score. The number only has to be
# large enough that a change of the highest row is visible.
STRUCTURED_ACTIONS = 9

# How many decisions the choice tests measure over.
DECISIONS = 60

GROWTH_GENERATIONS = 20
"""How many generations the growth test runs."""

BOUND_GENERATIONS = 60
"""How many generations the norm bound test runs.

It must be more than the growth of one generation needs to reach the bound, or
the test would assert a bound the run never touched.
"""


def a_structured_shell() -> StructuredPolicy:
    """Build the untrained structured policy over the published layout."""
    world = World(
        width=LAYOUT_WIDTH,
        height=LAYOUT_HEIGHT,
        faction_count=LAYOUT_FACTIONS,
        seed=LAYOUT_SEED,
    )
    layout = ObservationLayout.of_catalogue(SignalCatalogue.of_world(world))
    return StructuredPolicy.zeros(STRUCTURED_ACTIONS, layout)


def a_trained_centre(shell: StructuredPolicy, seed: int = 3) -> np.ndarray:
    """Move the shell off its zero readout, the way a first generation does.

    **A shell scores every action row at zero, so it takes row zero whatever
    its other weights say.** A test that measured the shell would measure the
    no-op and never reach the arithmetic under test.
    """
    rng = np.random.default_rng(seed)
    flat = shell.flat()
    return flat + 0.05 * rng.standard_normal(flat.size)


def a_decision_set(length: int, seed: int = 5) -> tuple[np.ndarray, np.ndarray]:
    """Build observations that span many orders of magnitude, and their masks.

    **A uniform fixture would hide the defect under test.** The encoder takes
    the signed logarithm of each position, so a fixture of one magnitude puts
    every feature at one size and leaves the saturating layers in one place on
    their curves. These draws span several magnitudes and both signs.

    Row zero of every mask is legal, because the no-op always is. Each other
    row is legal at random, so the highest legal row is not always the highest
    row.
    """
    rng = np.random.default_rng(seed)
    magnitude = rng.uniform(0.0, 12.0, size=(DECISIONS, length))
    sign = rng.choice([-1.0, 1.0], size=(DECISIONS, length))
    observations = sign * np.expm1(magnitude)
    masks = rng.integers(0, 2, size=(DECISIONS, STRUCTURED_ACTIONS))
    masks[:, 0] = 1
    return observations, masks


def differing(first: list[int], second: list[int]) -> int:
    """Count the decisions where two policies chose different action rows."""
    return sum(one != other for one, other in zip(first, second, strict=True))


def test_a_linear_policy_keeps_its_choice_under_a_positive_scaling() -> None:
    """The linear kind declares this, and the declaration must be true.

    One matrix scores every action row as a weighted sum over one feature
    vector, so scaling the matrix scales every score by one factor.
    """
    rng = np.random.default_rng(19)
    policy = LinearPolicy(rng.standard_normal((STRUCTURED_ACTIONS, FEATURES + 1)))
    observations, masks = a_decision_set(FEATURES)
    base = policy.choose_many(observations, masks)
    assert choice_survives_scaling(policy)
    for factor in (0.1, 0.5, 2.0, 40.0):
        scaled = LinearPolicy(policy.weights * factor)
        assert differing(base, scaled.choose_many(observations, masks)) == 0


def test_a_structured_policy_loses_its_choice_under_a_positive_scaling() -> None:
    """The structured kind declares this, and the declaration must be true.

    **This test is the whole reason the declaration exists.** Every tower ends
    in a ``tanh`` and the trunk does too, so a scaling puts each one at
    another place on its curve. A bias meets a constant feature of value one,
    so it scales while what it is added to does not scale by the same factor.

    The search normalised every centre before this, on the claim that a
    scaling never moves a choice. The counts below are what that claim cost.
    """
    shell = a_structured_shell()
    centre = a_trained_centre(shell)
    policy = shell.rebuild(centre)
    observations, masks = a_decision_set(shell.layout.length)
    base = policy.choose_many(observations, masks)
    assert not choice_survives_scaling(policy)
    for factor in (0.5, 2.0, 10.0):
        scaled = shell.rebuild(centre * factor)
        assert differing(base, scaled.choose_many(observations, masks)) > 0


def test_the_search_leaves_a_structured_centre_choosing_what_it_chose() -> None:
    """The search must not change the function a policy computes.

    The second half of this test puts the old behaviour back and asserts that
    the centre it produced chose other rows. A test of an invariance that
    never sees the variant measures nothing.[^1]

    References
    ----------
    [^1]: Testing Rules, section 1. ``.agents/rules/testing.md``
    """
    shell = a_structured_shell()
    centre = a_trained_centre(shell)
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=1.5, learning_rate=0.3, seed=7
    )
    observations, masks = a_decision_set(shell.layout.length)
    base = shell.rebuild(centre).choose_many(observations, masks)

    held = search.start(centre)
    assert np.array_equal(held, centre)
    assert differing(base, shell.rebuild(held).choose_many(observations, masks)) == 0

    normalised = unit(centre)
    assert float(np.linalg.norm(centre)) > 2.0
    changed = differing(
        base, shell.rebuild(normalised).choose_many(observations, masks)
    )
    assert changed > 0


def test_a_structured_perturbation_stays_the_fraction_sigma_names() -> None:
    """Sigma is a fraction of the centre, whatever length the centre reached.

    A perturbation that keeps a fixed length becomes a smaller and smaller
    turn as the centre grows. Every candidate of a generation then chooses the
    same actions, the ranking has nothing to rank, and the update becomes a
    walk driven by noise. That failure is silent.
    """
    shell = a_structured_shell()
    sigma = 1.5
    centre = a_trained_centre(shell)
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=sigma, learning_rate=0.3, seed=7
    )
    lengths = []
    for generation in range(GROWTH_GENERATIONS):
        candidate = search.propose(centre, generation)[0]
        moved = candidate.flat() - centre
        fraction = float(np.linalg.norm(moved)) / float(np.linalg.norm(centre))
        assert fraction == pytest.approx(sigma)
        lengths.append(float(np.linalg.norm(centre)))
        update = search.update(centre, generation, AGREED_SCORES)
        assert update.agreement == pytest.approx(1.0), (
            "the fixture agreed on nothing, so the centre never moved"
        )
        centre = update.centre
    assert lengths[-1] > lengths[0]


def test_the_search_still_holds_a_linear_centre_at_unit_length() -> None:
    """The linear kind is the control every stored score compares against.

    A change to its trajectory invalidates every recorded comparison, so this
    asserts what the search declares and what it then does.
    """
    search = a_strategy()
    assert search.holds_unit_centre
    centre = a_centre(search)
    assert float(np.linalg.norm(centre)) == pytest.approx(1.0)


def test_the_linear_step_is_the_expression_it_always_was() -> None:
    """Pin the linear trajectory to the arithmetic it ran before this change.

    The test builds the old expression from stated ranks and from its own
    normalisation, and it asserts an exact equality. **An approximate equality
    would pass a step that moved in the last bits**, and a run that moved in
    the last bits no longer compares against a stored score.

    **The expectation reaches neither the shaping nor the normalisation of the
    search.** This pin once called both of them, so a shaping that returned a
    constant satisfied it: every pair difference would be zero, the gradient
    would be zero, and the expected centre would reduce to the centre that
    went in.[^1]

    The perturbation still comes from the search, because a draw from a
    generator is not an expression a test can restate. A test beside this one
    pins that draw on its own.

    **The expression gained one factor and the pin changed with it.** The step
    was the learning rate along the direction, whatever the generation scored.
    It is now the learning rate times how far the candidates agreed. The
    direction did not change, and neither did the normalisation on either side
    of the step.

    **The test takes the agreement from the search and pins it separately.**
    The agreement is a ratio of three float sums, so a test cannot restate it
    to the last bit, and an exact pin on the whole step needs the same bits.
    The separate assertion states the arithmetic and holds the value to six
    figures, so a search that answered a constant fails there. A search that
    answered zero fails on the exact pin as well, because a generation that
    moves nothing gives back the centre it was given and not a rescaled copy.

    References
    ----------
    [^1]: Findings register, FND-709. ``docs/FINDINGS.md``
    """
    search = a_strategy(seed=7, sigma=0.5, rate=0.3)
    centre = a_centre(search)
    noise = search.noise(centre, 0)
    gradient = np.zeros_like(centre)
    for index in range(3):
        step = SPREAD_RANKS[2 * index] - SPREAD_RANKS[2 * index + 1]
        gradient += step * noise[index]
    update = search.update(centre, 0, SPREAD_SCORES)
    assert update.agreement == pytest.approx(SPREAD_AGREEMENT)
    expected = normalised(centre + 0.3 * update.agreement * normalised(gradient))
    assert np.array_equal(update.centre, expected)


def test_the_linear_perturbation_is_sigma_and_not_a_scaled_sigma() -> None:
    """Pin the linear candidates to the arithmetic they ran before this change.

    The perturbation of a kind whose choice survives a scaling is sigma
    itself. A search that multiplied it by the length of a normalised centre
    would move every candidate in the last bits, because the length of a
    normalised vector is not exactly one.
    """
    search = a_strategy(seed=7, sigma=0.5, rate=0.3)
    centre = a_centre(search)
    noise = search.noise(centre, 0)
    candidates = search.propose(centre, 0)
    assert np.array_equal(candidates[0].flat(), centre + 0.5 * noise[0])
    assert np.array_equal(candidates[1].flat(), centre - 0.5 * noise[0])


def test_a_generation_that_agrees_moves_the_centre_further() -> None:
    """The step must be as long as the generation was worth.

    **This is the property that did not hold before this change.** The search
    normalised the summed direction and moved a fixed fraction of the centre
    along it, so a generation whose candidates barely separated moved the
    centre exactly as far as one that separated perfectly. An audit derived
    the cost: over twenty generations the centre wandered 4.7 times further
    than it climbed.[^1]

    The two generations here start from one centre at one generation number,
    so they draw the same perturbations. Only the scores differ.

    **The test measures the structured kind, because the linear kind hides the
    quantity.** A linear centre is normalised after the step, so the distance
    between the old centre and the new one depends on the angle between the
    centre and the direction as well as on the travel. Two generations that
    took one fixed step then landed different distances away, and an earlier
    version of this test passed on that difference alone. A structured centre
    keeps its length, so the distance is the travel.

    References
    ----------
    [^1]: Report on what is wrong with training and evaluation, items 5 and 13.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``
    """
    shell = a_structured_shell()
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=0.5, learning_rate=0.3, seed=7
    )
    centre = a_trained_centre(shell)
    length = float(np.linalg.norm(centre))
    assert length < search.norm_ceiling, (
        "the bound clipped the step, so the test measured the bound"
    )
    agreed = search.update(centre, 0, AGREED_SCORES)
    less = search.update(centre, 0, SPREAD_SCORES)
    assert agreed.agreement == pytest.approx(1.0)
    assert less.agreement == pytest.approx(SPREAD_AGREEMENT)
    travelled = float(np.linalg.norm(agreed.centre - centre))
    shorter = float(np.linalg.norm(less.centre - centre))
    assert travelled == pytest.approx(0.3 * length)
    assert shorter == pytest.approx(0.3 * SPREAD_AGREEMENT * length)
    assert travelled > shorter


def test_a_generation_that_agrees_no_better_than_noise_moves_nothing() -> None:
    """A ranking of noise gives the search no direction to step along.

    The pair weights of a descending generation are the smallest they can be,
    so the sum of their squares falls under what a random permutation reaches.
    The search reports that the generation carried no information, although its
    scores had a spread.
    """
    search = a_search()
    centre = a_centre(search)
    update = search.update(centre, 0, DISAGREED_SCORES)
    assert update.spread == pytest.approx(5.0)
    assert update.agreement == 0.0
    assert not update.informative
    assert np.array_equal(update.centre, centre)


def test_the_learning_rate_bounds_the_largest_step() -> None:
    """One lucky generation must not throw the centre.

    The agreement never passes one, so the learning rate keeps its meaning as
    the largest fraction of the centre that one generation moves. A search
    that scaled by an unbounded quantity would have no such bound, and the
    fixture here is the generation that reaches the largest step there is.
    """
    search = a_strategy(seed=7, sigma=0.5, rate=0.3)
    centre = a_centre(search)
    update = search.update(centre, 0, AGREED_SCORES)
    assert update.agreement == pytest.approx(1.0)
    turned = float(np.linalg.norm(update.centre - centre))
    assert turned <= 0.3


def test_a_tie_ranks_the_same_whichever_order_the_sort_visits() -> None:
    """A tie states that two candidates scored the same, and nothing more.

    The test reverses the generation and reverses the ranks back. A ranking
    that read the candidate index would answer differently, because the
    reversal moves each tied candidate to the other side of the other.

    **The second half puts the stable sort back and asserts that it does read
    the index.** A test of an invariance that never sees the variant measures
    nothing.[^1]

    References
    ----------
    [^1]: Testing Rules, section 1. ``.agents/rules/testing.md``
    """
    ranks = rank_shape(PARTIAL_TIE_SCORES)
    mirrored = rank_shape(PARTIAL_TIE_SCORES[::-1])[::-1]
    assert np.allclose(ranks, mirrored)
    assert ranks[2] == pytest.approx(ranks[4])

    old = index_order_ranks(PARTIAL_TIE_SCORES)
    old_mirrored = index_order_ranks(PARTIAL_TIE_SCORES[::-1])[::-1]
    assert not np.allclose(old, old_mirrored), (
        "the fixture holds no tie, so the test above proves nothing"
    )
    assert old[2] != old[4]


def test_a_partial_tie_ranks_between_the_two_ways_of_breaking_it(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A tie must not resolve toward either of the two orders it could take.

    The step is a function of the ranks, so the ranks are where the property
    is exact. The test breaks the tie both ways by a small amount, which gives
    the two rankings the search would have read if the tie had not happened.
    The ranks of the tied generation are the mean of those two, which is the
    answer that prefers neither candidate. The step therefore differs from
    both of them.

    **The second half puts the stable sort back.** A stable sort gives the
    earlier of two tied candidates the lower rank, so the tied generation
    reads as the generation where the later candidate scored higher. The
    ranking is that one bit for bit, and so is the step. The candidate index
    therefore chose the direction the centre moved.
    """
    search = a_strategy(seed=7, sigma=0.5, rate=0.3)
    centre = a_centre(search)
    lifted = PARTIAL_TIE_SCORES.copy()
    lifted[2] += TIE_STEP
    other = PARTIAL_TIE_SCORES.copy()
    other[4] += TIE_STEP

    tied = search.update(centre, 0, PARTIAL_TIE_SCORES)
    first = search.update(centre, 0, lifted)
    second = search.update(centre, 0, other)
    assert not np.allclose(first.ranks, second.ranks), (
        "the two ways of breaking the tie gave one ranking, so the tie is not real"
    )
    assert np.allclose(tied.ranks, (first.ranks + second.ranks) / 2.0)
    assert not np.array_equal(tied.centre, first.centre)
    assert not np.array_equal(tied.centre, second.centre)

    monkeypatch.setattr(search_module, "rank_shape", index_order_ranks)
    biased = search.update(centre, 0, PARTIAL_TIE_SCORES)
    old_first = search.update(centre, 0, lifted)
    old_second = search.update(centre, 0, other)
    assert np.array_equal(biased.ranks, old_second.ranks)
    assert np.array_equal(biased.centre, old_second.centre)
    assert not np.array_equal(biased.centre, old_first.centre)


def test_the_norm_of_a_structured_centre_stays_inside_its_bound() -> None:
    """A rising norm saturates the ``tanh`` layers of a structured policy.

    The structured kind declares that a positive scaling moves its choice, so
    the search never normalises that centre. Every step adds a near-orthogonal
    vector of a fixed fraction of the length, so the length rises over a run.

    The run here agrees at every generation, which is the fastest the length
    can rise. **The test asserts that the run reaches the bound as well as
    staying inside it**, because a run that never touched the bound would
    measure nothing.
    """
    shell = a_structured_shell()
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=0.5, learning_rate=0.3, seed=7
    )
    centre = a_trained_centre(shell)
    assert not search.holds_unit_centre
    for generation in range(BOUND_GENERATIONS):
        centre = search.update(centre, generation, AGREED_SCORES).centre
        assert float(np.linalg.norm(centre)) <= search.norm_ceiling * (1.0 + 1e-12)
    assert float(np.linalg.norm(centre)) == pytest.approx(search.norm_ceiling)


def test_the_bound_holds_a_resumed_centre_that_starts_above_it() -> None:
    """A centre already past the bound comes back to it on its first step.

    Such a centre is the saturated case the bound exists to prevent. The
    search shortens it and keeps its direction, because the direction is what
    the run trained.
    """
    shell = a_structured_shell()
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=0.5, learning_rate=0.3, seed=7
    )
    far = a_trained_centre(shell) * 10.0
    assert float(np.linalg.norm(far)) > search.norm_ceiling
    moved = search.update(far, 0, AGREED_SCORES).centre
    assert float(np.linalg.norm(moved)) == pytest.approx(search.norm_ceiling)


def test_the_search_leaves_a_linear_centre_out_of_the_norm_bound() -> None:
    """The bound governs no kind that holds its centre at unit length.

    A linear centre is normalised after every step, so its length is already
    fixed and a second bound on it would state the same thing twice.
    """
    search = a_strategy()
    assert search.holds_unit_centre
    centre = a_centre(search)
    for generation in range(6):
        centre = search.update(centre, generation, AGREED_SCORES).centre
    assert float(np.linalg.norm(centre)) == pytest.approx(1.0)


AUDITED_PAIRS = 12
"""How many antithetic pairs the run that paid for the audit of this path used.

The report states a population of 24 candidates, which is 12 pairs.
"""

AUDITED_TRAINABLE = 5354
"""How many weights the structured policy of that run trained."""

AUDITED_ALIGNMENT = 0.0473
"""The alignment the report derives from those two numbers."""

AUDITED_BREAK_EVEN = 447
"""The generations that alignment needs before the climb passes the wander.

The count is one over the square of the alignment, rounded up.
"""


def test_the_alignment_follows_the_law_the_project_measured() -> None:
    """The cosine is the square root of the pairs over the trainable count.

    One register measured that law over two orders of magnitude, and an audit
    applied it to the population and the policy shape of the run that paid for
    it.[^1] **The expected figures come from that report and not from the
    function under test.**

    The break-even count is the generations a run needs before the part of its
    travel that points at the truth passes the part that does not. The
    directed part grows with the generation count and the undirected part
    grows with its square root, so the two meet at one over the square of the
    alignment.

    References
    ----------
    [^1]: Report on what is wrong with training and evaluation, items 4 and 13.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``
    """
    alignment = step_alignment(AUDITED_PAIRS, AUDITED_TRAINABLE)
    assert alignment == pytest.approx(AUDITED_ALIGNMENT, abs=5e-5)
    assert generations_before_a_climb_beats_a_wander(alignment) == AUDITED_BREAK_EVEN


def test_the_search_reports_the_alignment_every_generation() -> None:
    """A run that never computes the figure cannot act on it.

    The figure follows from the pair count and the trainable count, so it is
    the same number at every generation of one run. It travels back with every
    update, including the update of a generation that carried nothing.
    """
    search = a_strategy()
    centre = a_centre(search)
    expected = step_alignment(3, centre.size)
    assert search.update(centre, 0, AGREED_SCORES).alignment == pytest.approx(expected)
    quiet = search.update(centre, 0, np.full(search.population, 3.0))
    assert quiet.alignment == pytest.approx(expected)


INCUMBENT_SIGMA = 1.5
"""The sigma the project ran before a measurement replaced it.

The measurement derives 6 worlds for each candidate at this sigma.
"""

RECOMMENDED_SIGMA = 0.5
"""The sigma the measurement recommends.

The measurement derives 3 worlds for each candidate at this sigma.
"""

CONFIGURED_WORLDS = 4
"""How many worlds the run gave each candidate, at either sigma."""


def test_the_report_says_when_a_run_is_under_sampled_for_its_sigma() -> None:
    """The incumbent pair was under-sampled by its own arithmetic.

    A ranking needs the spread between the candidates to exceed the noise on
    one candidate's score. The measurement derives 6 worlds for a sigma of
    1.5 and 3 for a sigma of 0.5, and the run gave four either way. **So the
    incumbent pair had two defects rather than one**, and nothing said so
    until the money was spent.[^1]

    References
    ----------
    [^1]: Report on how sigma trades against the worlds each candidate plays,
    section 7.
    ``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
    """
    incumbent = configuration_notes(INCUMBENT_SIGMA, CONFIGURED_WORLDS, 12, 5354, 20)
    assert any("under-sampled" in note for note in incumbent), incumbent
    assert any("needs 6 worlds" in note for note in incumbent), incumbent

    recommended = configuration_notes(
        RECOMMENDED_SIGMA, CONFIGURED_WORLDS, 12, 5354, 20
    )
    assert not any("under-sampled" in note for note in recommended), recommended
    assert any("needs 3 worlds" in note for note in recommended), recommended


def test_the_report_states_the_alignment_and_the_wander() -> None:
    """A run shorter than its break-even count is mostly wander, and says so.

    The note is a statement and not a refusal. A reader may want the run
    anyway, and a run that failed on this figure would train nothing at the
    population this project can afford.
    """
    short = configuration_notes(
        RECOMMENDED_SIGMA, CONFIGURED_WORLDS, AUDITED_PAIRS, AUDITED_TRAINABLE, 20
    )
    assert any("aligns 0.0473" in note for note in short), short
    assert any("mostly wander" in note for note in short), short

    long_enough = configuration_notes(
        RECOMMENDED_SIGMA,
        CONFIGURED_WORLDS,
        AUDITED_PAIRS,
        AUDITED_TRAINABLE,
        AUDITED_BREAK_EVEN,
    )
    assert not any("mostly wander" in note for note in long_enough), long_enough


# The blocks of the structured policy, in the order the flat vector holds
# them. **The policy states the count of each one**, so no test here states a
# weight count. The order is the order the policy lays the arrays out, and the
# reader that cuts a flat vector back into arrays reads the same order.
POLICY_BLOCKS = ("scalars", "ring", "tokens", "trunk", "readout")

TRAVEL_GENERATIONS = 14
"""How many generations the travel test runs.

The run must stay under the norm bound, because a clipped step is a step the
bound chose and not one the layers chose. Every generation of the fixture
agrees perfectly, which is the fastest the length can rise, and the test
asserts that the length stayed inside the bound at the end.
"""

TRAVEL_DEVIATIONS = 3.0
"""How many standard deviations the travel test allows each block.

**The band is derived and it is not a round number.** The test states the
derivation beside the function that computes it.
"""


def isotropic_weighting(policy: Trainable, centre: np.ndarray) -> np.ndarray:
    """Weight every coordinate the same, whatever layer it sits in.

    **This is the defect, restated so that a test can put it back.** The
    search drew an isotropic perturbation over the whole flat vector, so a
    layer took the share of the step that its weight count predicts and every
    weight of the policy moved the same distance. The initial scale of a layer
    then decided how far the search could revise it.[^1]

    References
    ----------
    [^1]: Findings register, FND-713. ``docs/FINDINGS.md``
    """
    return np.ones(centre.size)


def bare_layer_weighting(policy: Trainable, centre: np.ndarray) -> np.ndarray:
    """Weight each layer by its own scale, and give a layer of zeros nothing.

    **This is the second defect, restated so that a test can put it back.** A
    rule that reads the current scale of a layer and states no fallback gives
    a zero layer a zero perturbation. The layer stays zero, and the zero is a
    fixed point the search can never leave. The readout of the untrained
    structured policy is such a layer.
    """
    sizes = layer_sizes(policy)
    scales = np.empty(len(sizes))
    walked = 0
    for index, size in enumerate(sizes):
        scales[index] = float(np.linalg.norm(centre[walked : walked + size])) / np.sqrt(
            size
        )
        walked += size
    weighting = np.repeat(scales, sizes)
    return np.asarray(weighting / weighting.max())


def restated_weighting(shell: StructuredPolicy, centre: np.ndarray) -> np.ndarray:
    """Restate the layer weighting from the shapes and the centre.

    **Nothing here calls the function under test.** The rule is one line of
    arithmetic: a layer takes the root mean square of its own weights. The
    tolerance of the travel test comes from this array, and the expectation of
    that test does not.
    """
    sizes = [int(np.prod(shape)) for shape in shell.shapes]
    weighting = np.empty(centre.size)
    walked = 0
    for size in sizes:
        length = float(np.linalg.norm(centre[walked : walked + size]))
        weighting[walked : walked + size] = length / np.sqrt(size)
        walked += size
    return weighting


def walked_layers(shell: StructuredPolicy) -> list[tuple[int, int, int]]:
    """Give the index, first coordinate and last coordinate of each layer."""
    bounds = []
    walked = 0
    for index, size in enumerate(layer_sizes(shell)):
        bounds.append((index, walked, walked + size))
        walked += size
    return bounds


def block_bounds(shell: StructuredPolicy) -> list[tuple[str, int, int]]:
    """Give the first and last coordinate of each block of the flat vector."""
    counts = shell.counts()
    bounds = []
    walked = 0
    for name in POLICY_BLOCKS:
        bounds.append((name, walked, walked + counts[name]))
        walked += counts[name]
    return bounds


def effective_count(squared: np.ndarray) -> float:
    """Return how many equal coordinates a set of weighted ones counts as.

    A sum of squared normal draws with unequal weights has the variance of a
    smaller sum of equal ones. The count is the square of the sum of the
    weights over the sum of their squares, which is the usual answer for a
    weighted sum of squares.
    """
    return float(squared.sum() ** 2 / (squared**2).sum())


def travel_deviation(weighting: np.ndarray, first: int, last: int) -> float:
    """Return the standard deviation of one block's relative step, as a fraction.

    **The band of the travel test is derived here and it is not chosen.** One
    perturbation is a normal draw scaled by the weighting and then normalised,
    so the share a block takes of the squared step length is a ratio of two
    weighted sums of squared normal draws. The share has the mean the
    weighting predicts. Its variance follows from the effective coordinate
    count of the block, from the effective count of the rest, and from the
    share itself, and the step length is the square root of the share, which
    halves the fraction.

    The relative step of one generation is therefore the travel times one plus
    this fraction. A run of several generations adds the squared steps, so the
    fraction of the accumulated travel falls with the square root of the
    generation count. The caller divides by that.
    """
    squared = weighting**2
    block = squared[first:last]
    rest = np.concatenate([squared[:first], squared[last:]])
    share = float(block.sum() / squared.sum())
    spread = np.sqrt(2.0 / effective_count(block) + 2.0 / effective_count(rest))
    return float(0.5 * (1.0 - share) * spread)


def relative_travel(
    search: EvolutionStrategy, shell: StructuredPolicy, centre: np.ndarray, gens: int
) -> tuple[dict[str, float], np.ndarray]:
    """Run the search and give back how far each block travelled, over itself.

    The quantity is the length of each step of a block, divided by the length
    that block held when it took the step, accumulated in quadrature. Two
    steps in a space of this dimension are near orthogonal, so the quadrature
    sum is the travel of the block.

    **The quantity divides by the length the block holds at each generation
    and not by the length it started at.** A block grows as it travels, and
    every block grows by the same fraction under the rule under test, so the
    two forms differ by one factor that is common to every block. The
    generation form states the property directly.
    """
    travelled = {name: 0.0 for name in POLICY_BLOCKS}
    for generation in range(gens):
        update = search.update(centre, generation, AGREED_SCORES)
        assert update.agreement == pytest.approx(1.0), (
            "the fixture agreed on nothing, so the centre never moved"
        )
        step = update.centre - centre
        for name, first, last in block_bounds(shell):
            reach = float(np.linalg.norm(step[first:last]))
            held = float(np.linalg.norm(centre[first:last]))
            travelled[name] += (reach / held) ** 2
        centre = update.centre
    return {name: np.sqrt(value) for name, value in travelled.items()}, centre


def test_the_layers_of_a_policy_cover_its_flat_vector() -> None:
    """The layer boundaries and the flat vector are one statement, not two.

    The search cuts a perturbation into layers by the shapes the policy
    states, and the policy cuts a flat vector into arrays by the same shapes.
    A vector the layers do not cover is those two statements disagreeing, and
    a silent answer there would weight the wrong coordinates.
    """
    shell = a_structured_shell()
    assert sum(layer_sizes(shell)) == shell.flat().size
    linear = LinearPolicy.zeros(ACTIONS, FEATURES)
    assert sum(layer_sizes(linear)) == linear.flat().size
    with pytest.raises(ValueError, match="lays its weights out in layers"):
        layer_weighting(shell, shell.flat()[:-1])


def test_a_linear_policy_draws_the_isotropic_perturbation_it_always_drew() -> None:
    """The one-layer kind must not move, and it must not move in the last bits.

    A linear policy holds one layer, so the layer rule scales its whole vector
    by one number and the normalisation of each perturbation removes that
    number. The weighting is therefore exactly one at every coordinate, and
    the draw is the draw. **The expectation states the arithmetic rather than
    calling the search**, because an expectation taken from the search would
    hold whatever the search now does.
    """
    linear = LinearPolicy.zeros(ACTIONS, FEATURES)
    size = linear.flat().size
    drawn = np.random.default_rng(11).standard_normal(size)
    for centre in (linear.flat(), drawn):
        weighting = layer_weighting(linear, centre)
        assert np.all(weighting == 1.0)
    raw = np.random.default_rng([7, 4]).standard_normal((3, size))
    expected = raw / np.linalg.norm(raw, axis=1, keepdims=True)
    assert np.array_equal(generation_noise(7, 4, 3, linear, drawn), expected)


def test_a_layer_reads_its_own_weights_and_not_the_layers_beside_it() -> None:
    """The scale of a layer is a function of that layer alone.

    A rule that read the layers in the order a mapping happened to hold them,
    or that let one layer reach the scale of another, would put an order the
    search invented into every perturbation. The test moves one layer and
    asserts that the ratio between two others does not move at all.

    The weighting is also one number inside each layer, because a layer has
    one scale.
    """
    shell = a_structured_shell()
    centre = a_trained_centre(shell)
    sizes = layer_sizes(shell)
    before = layer_weighting(shell, centre)
    walked = 0
    for size in sizes:
        block = before[walked : walked + size]
        assert np.all(block == block[0])
        walked += size

    moved = centre.copy()
    first = sizes[0]
    moved[first : first + sizes[1]] *= 4.0
    after = layer_weighting(shell, moved)
    third = first + sizes[1]
    fourth = third + sizes[2]
    assert before[0] / before[third] == after[0] / after[third]
    assert before[third] / before[fourth] == after[third] / after[fourth]


def test_a_layer_of_zeros_still_receives_a_perturbation(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A zero layer takes the scale of the whole centre, so it can leave zero.

    The readout of the untrained structured policy is zero, and so is every
    bias array. A perturbation proportional to a zero scale is zero, the layer
    stays zero, and the zero is a fixed point the search can never leave.

    **The second half puts that rule back and asserts that the layer then
    stays zero.** A test of a fallback that never sees the case without it
    measures nothing.[^1]

    References
    ----------
    [^1]: Testing Rules, section 1. ``.agents/rules/testing.md``
    """
    shell = a_structured_shell()
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=0.5, learning_rate=0.3, seed=7
    )
    centre = search.start(shell.flat())
    zeros = [
        (first, last)
        for _, first, last in walked_layers(shell)
        if not np.any(centre[first:last])
    ]
    assert zeros, "the shell holds no zero layer, so this test measures nothing"

    moved = search.propose(centre, 0)[0].flat() - centre
    for first, last in zeros:
        assert np.all(moved[first:last] != 0.0)
    update = search.update(centre, 0, AGREED_SCORES)
    for first, last in zeros:
        assert np.any(update.centre[first:last] != 0.0)

    monkeypatch.setattr(search_module, "layer_weighting", bare_layer_weighting)
    bare = search.update(centre, 0, AGREED_SCORES)
    for first, last in zeros:
        assert np.all(bare.centre[first:last] == 0.0), (
            "the layer left zero without the fallback, so the fallback proves nothing"
        )


def test_every_block_travels_the_same_fraction_of_its_own_length(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The search must revise each block by the same fraction of itself.

    **This is the property that did not hold before this change.** The search
    drew an isotropic perturbation over the flat vector, so a block took the
    share of the step that its weight count predicts and every weight of the
    policy moved the same distance. The initial scale of a block then decided
    how far the search could revise it, and that scale spans a factor of seven
    across the blocks of this policy.[^1]

    The expectation is stated arithmetic and not a call. Every generation of
    the fixture agrees perfectly, so each step is the learning rate times the
    length of the centre, and a block that takes its own fraction of that step
    travels the learning rate times the square root of the generation count.

    **The band is derived from the fixture.** A perturbation is a normal draw,
    so the share a block takes of one step fluctuates, and the function beside
    this one states the closed form of that fluctuation.

    **The second half puts the isotropic draw back.** The blocks then miss the
    band by more than ten times its width, which is what the finding
    measured.

    References
    ----------
    [^1]: Findings register, FND-713. ``docs/FINDINGS.md``
    """
    shell = a_structured_shell()
    search = EvolutionStrategy(
        shell=shell, pairs=3, sigma=0.5, learning_rate=0.3, seed=7
    )
    start = a_trained_centre(shell)
    for _, first, last in walked_layers(shell):
        assert np.any(start[first:last]), (
            "a layer of the fixture is zero, so the test measures the fallback"
        )
    weighting = restated_weighting(shell, start)
    bands = {
        name: TRAVEL_DEVIATIONS
        * travel_deviation(weighting, first, last)
        / np.sqrt(TRAVEL_GENERATIONS)
        for name, first, last in block_bounds(shell)
    }
    expected = 0.3 * np.sqrt(TRAVEL_GENERATIONS)

    travelled, ended = relative_travel(search, shell, start, TRAVEL_GENERATIONS)
    assert float(np.linalg.norm(ended)) < search.norm_ceiling, (
        "the bound clipped a step, so the test measured the bound"
    )
    for name, reached in travelled.items():
        assert abs(reached / expected - 1.0) <= bands[name], (
            f"the {name} block travelled {reached:.4f} against {expected:.4f}"
        )

    monkeypatch.setattr(search_module, "layer_weighting", isotropic_weighting)
    isotropic, _ = relative_travel(search, shell, start, TRAVEL_GENERATIONS)
    missed = [
        name
        for name, reached in isotropic.items()
        if abs(reached / expected - 1.0) > bands[name]
    ]
    assert len(missed) >= len(POLICY_BLOCKS) - 1, (
        f"the isotropic draw held the band at {missed}, so the band is too wide"
    )
