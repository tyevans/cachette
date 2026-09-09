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

References
----------
[^1]: Testing Rules, sections 1, 2 and 2a. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette._core import World
from cachette.learn.layout import ObservationLayout
from cachette.learn.policy import LinearPolicy
from cachette.learn.search import (
    EvolutionStrategy,
    Optimiser,
    choice_survives_scaling,
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
        scores = np.arange(search.population, dtype=float)
        centre = search.update(centre, generation, scores).centre
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

# How many generations the growth test runs, and the multiple of the starting
# length that the centre must stay under. **The search states no bound on the
# length of an unnormalised centre**, so this test is the only thing that
# would notice a step that ran away.
GROWTH_GENERATIONS = 20
GROWTH_CEILING = 4.0


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
        scores = np.arange(search.population, dtype=float)
        centre = search.update(centre, generation, scores).centre
    assert lengths[-1] > lengths[0]
    assert float(np.linalg.norm(centre)) < GROWTH_CEILING * lengths[0]


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
    expected = normalised(centre + 0.3 * normalised(gradient))
    assert np.array_equal(search.update(centre, 0, SPREAD_SCORES).centre, expected)


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
