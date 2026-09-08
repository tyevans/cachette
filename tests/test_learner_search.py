"""The search must draw from the generation number, and must refuse a tie.

The search sits behind one door, so a run can hold another search without a
rewrite. These tests drive that door rather than the internals, and they check
the two properties the run depends on: the perturbations are a function of the
run seed and the generation number, and a generation of equal scores moves the
centre nowhere.
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette.learn.policy import LinearPolicy
from cachette.learn.search import EvolutionStrategy, Optimiser, rank_shape, unit

ACTIONS = 6
FEATURES = 5


def a_search(seed: int = 7, sigma: float = 0.5, rate: float = 0.3) -> Optimiser:
    """Build the search over a small linear policy."""
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
    """The caller logs the spread, the scores and the ranks it acted on."""
    search = a_search()
    centre = a_centre(search)
    scores = np.array([5.0, 1.0, 4.0, 2.0, 3.0, 0.0])
    update = search.update(centre, 0, scores)
    assert update.informative
    assert update.spread == pytest.approx(5.0)
    assert np.array_equal(update.scores, scores)
    assert np.array_equal(update.ranks, rank_shape(scores))
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
