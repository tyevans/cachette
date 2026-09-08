"""The catalogue must agree with the engine, and refuse what it cannot read.

These tests drive the engine rather than a hand-built schema. A catalogue built
from a fixture proves that the catalogue reads a fixture.
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette._core import World
from cachette.learn.signals import Aggregation, Signal, SignalCatalogue

WIDTH = 48
HEIGHT = 48
FACTIONS = 3


@pytest.fixture(scope="module")
def world() -> World:
    """One seeded world of the shape the training run plays."""
    built = World(width=WIDTH, height=HEIGHT, seed=1, faction_count=FACTIONS)
    built.seed_world()
    return built


@pytest.fixture(scope="module")
def catalogue(world: World) -> SignalCatalogue:
    """Build the catalogue of that world."""
    return SignalCatalogue.of_world(world)


def test_the_catalogue_holds_every_field_the_schema_declares(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A signal the catalogue drops is a quantity no caller can reach."""
    declared = {row["name"] for row in world.observation_schema()["fields"]}
    held = {signal.name for signal in catalogue}
    assert held == declared


def test_the_catalogue_states_the_length_the_schema_states(
    world: World, catalogue: SignalCatalogue
) -> None:
    """The length decides which observations this catalogue may read."""
    assert catalogue.observation_length == world.observation_schema()["length"]


def test_the_positions_of_every_signal_cover_the_observation(
    catalogue: SignalCatalogue,
) -> None:
    """Every position of the array belongs to exactly one signal.

    A gap is a quantity nothing names. An overlap is two names for one number,
    which is the defect shape this project records first.
    """
    covered: list[int] = []
    for signal in catalogue:
        covered.extend(range(signal.start, signal.start + signal.positions))
    assert sorted(covered) == list(range(catalogue.observation_length))


def test_a_scalar_and_a_compound_signal_both_exist(
    catalogue: SignalCatalogue,
) -> None:
    """The distinction is the reason this module exists, so it must be real."""
    assert catalogue.scalars(), "no signal holds one position"
    assert catalogue.compound(), "no signal holds several positions"
    assert len(catalogue.scalars()) + len(catalogue.compound()) == len(catalogue)


def test_reading_a_scalar_needs_no_aggregation(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A one-position signal reads straight from its own place."""
    observation = np.asarray(world.faction_observation(0))
    held = catalogue.signal("held_tiles")
    assert held.scalar
    assert held.read(observation) == observation[held.start]


def test_a_scalar_refuses_an_aggregation(
    world: World, catalogue: SignalCatalogue
) -> None:
    """An aggregation over one position hides which signal was read."""
    observation = np.asarray(world.faction_observation(0))
    with pytest.raises(ValueError, match="takes no aggregation"):
        catalogue.signal("held_tiles").read(observation, Aggregation.SUM)


def test_a_compound_signal_refuses_to_be_read_without_an_aggregation(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A guess here becomes an objective nobody chose."""
    observation = np.asarray(world.faction_observation(0))
    relation = catalogue.signal("relation")
    assert not relation.scalar
    with pytest.raises(ValueError, match="needs an aggregation"):
        relation.read(observation)


def test_every_aggregation_reduces_a_compound_signal(
    world: World, catalogue: SignalCatalogue
) -> None:
    """Each aggregation must answer, and the answers must bracket each other."""
    observation = np.asarray(world.faction_observation(0))
    relation = catalogue.signal("relation")
    window = observation[relation.start : relation.start + relation.positions]
    lowest = relation.read(observation, Aggregation.LOWEST)
    highest = relation.read(observation, Aggregation.HIGHEST)
    mean = relation.read(observation, Aggregation.MEAN)
    total = relation.read(observation, Aggregation.SUM)
    assert lowest == float(window.min())
    assert highest == float(window.max())
    assert lowest <= mean <= highest
    assert total == pytest.approx(mean * relation.positions)


def test_the_catalogue_names_the_alternatives_when_a_name_is_absent(
    catalogue: SignalCatalogue,
) -> None:
    """The failure this replaces was a caller carrying a name across a boundary.

    The trainer reports the tick of the end as ``end_tick`` and a reward must
    ask for ``tick``. A reader who carries the reported name here must be told
    what the layout does hold.
    """
    with pytest.raises(KeyError, match="names no signal") as raised:
        catalogue.signal("end_tick")
    assert "tick" in str(raised.value)


def test_reading_an_observation_of_another_layout_fails(
    catalogue: SignalCatalogue,
) -> None:
    """A shorter array would read a signal from another quantity's place."""
    with pytest.raises(ValueError, match="positions and was given one of"):
        catalogue.read_scalars(np.zeros(catalogue.observation_length - 1))


def test_reading_leaves_out_a_compound_signal_the_caller_did_not_aggregate(
    world: World, catalogue: SignalCatalogue
) -> None:
    """An unnamed compound signal is absent rather than guessed at."""
    observation = np.asarray(world.faction_observation(0))
    values = catalogue.read(observation, {"relation": Aggregation.SUM})
    assert "relation" in values
    assert "board_good" not in values
    assert set(catalogue.read_scalars(observation)) <= set(values)


def test_the_scalars_of_a_reading_are_every_one_position_signal(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A record carries every published quantity, not a subset someone named."""
    observation = np.asarray(world.faction_observation(0))
    values = catalogue.read_scalars(observation)
    assert set(values) == {signal.name for signal in catalogue.scalars()}


def test_an_aggregation_over_no_position_fails() -> None:
    """A signal of zero positions is a schema this module cannot serve."""
    with pytest.raises(ValueError, match="at least one position"):
        Aggregation.SUM.apply(np.zeros(0))


def test_a_signal_reads_from_its_own_window_and_not_its_neighbour() -> None:
    """Prove the read can fail. A start that is off by one reads the neighbour.

    The catalogue takes its positions from the schema, so an error here would
    come from the arithmetic rather than from the layout. This builds a layout
    by hand for that reason, which is the one place a fixture is correct.
    """
    array = np.arange(10, dtype=float)
    assert Signal(name="third", start=2, positions=1).read(array) == 2.0
    middle = Signal(name="middle", start=3, positions=4)
    assert middle.read(array, Aggregation.SUM) == 3.0 + 4.0 + 5.0 + 6.0
    assert middle.read(array, Aggregation.LOWEST) == 3.0
    assert middle.read(array, Aggregation.HIGHEST) == 6.0
