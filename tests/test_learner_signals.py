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

# One unit of the fixed-point scale of the engine.
UNIT = 65536

# A tile count one below a power of two, which a compressed magnitude inverts
# exactly. The world of the inversion test holds this many tiles.
EXACT_TILE_COUNT = 1023


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
    statistics = catalogue.signal("power_held_tiles")
    assert not statistics.scalar
    with pytest.raises(ValueError, match="needs an aggregation"):
        statistics.read(observation)


def test_every_aggregation_reduces_a_compound_signal(
    world: World, catalogue: SignalCatalogue
) -> None:
    """Each aggregation must answer, and the answers must bracket each other."""
    observation = np.asarray(world.faction_observation(0))
    statistics = catalogue.signal("power_held_tiles")
    window = observation[statistics.start : statistics.start + statistics.positions]
    lowest = statistics.read(observation, Aggregation.LOWEST)
    highest = statistics.read(observation, Aggregation.HIGHEST)
    mean = statistics.read(observation, Aggregation.MEAN)
    total = statistics.read(observation, Aggregation.SUM)
    assert lowest == float(window.min())
    assert highest == float(window.max())
    assert lowest <= mean <= highest
    assert total == pytest.approx(mean * statistics.positions)


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
    values = catalogue.read(observation, {"power_held_tiles": Aggregation.SUM})
    assert "power_held_tiles" in values
    assert "trade_board" not in values
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


def test_the_engine_publishes_a_value_form_for_every_field(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A field with no form is a value nothing outside the engine can read back.

    The engine names the form of each field and publishes what an inversion of
    that form needs. A reader that met a field with no form would hold the
    compression itself, which puts one engine rule in two places.
    """
    published = world.observation_schema()["value_forms"]
    assert published
    for signal in catalogue:
        assert signal.form is not None, signal.name
        assert signal.form.name in published
        assert signal.form.unit == UNIT


def test_a_known_count_inverts_from_the_value_the_engine_published() -> None:
    """A count crosses as a compressed magnitude, and the signal inverts it.

    **This is the round trip the published form exists for.** The world holds
    one tile fewer than a power of two, so the inversion is exact and the
    assertion needs no tolerance beyond the float arithmetic.

    The test drives the engine rather than a hand-built schema, so it proves
    that the form reaches a real observation.[^1]

    # References

    [^1]: Testing rules, section 5. ``.agents/rules/testing.md``
    """
    built = World(width=31, height=33, seed=2, faction_count=FACTIONS)
    built.seed_world()
    catalogue = SignalCatalogue.of_world(built)
    signal = catalogue.signal("world_tiles")
    assert signal.form is not None
    assert signal.form.name == "magnitude"
    assert signal.invertible

    observation = np.asarray(built.faction_observation(0))
    published = signal.read(observation)
    assert published != EXACT_TILE_COUNT
    assert signal.quantities(observation) == pytest.approx(EXACT_TILE_COUNT)


def test_a_compound_magnitude_inverts_every_one_of_its_positions(
    world: World, catalogue: SignalCatalogue
) -> None:
    """A caller that wants a count reads one quantity for each position.

    The stock of each good class is a compressed magnitude of a store total, so
    every position of it inverts. A stock the world does not hold reads zero,
    and zero inverts to zero.
    """
    signal = catalogue.signal("stock_of_class")
    assert signal.form is not None
    assert signal.form.name == "magnitude"
    observation = np.asarray(world.faction_observation(0))
    recovered = signal.quantities(observation)
    assert recovered.shape == (signal.positions,)
    assert np.all(recovered >= 0.0)


def test_a_share_refuses_an_inversion_and_names_its_denominator(
    catalogue: SignalCatalogue,
) -> None:
    """A share divides by a whole that no position of the array carries.

    A reader that inverted a share anyway would report a count the engine never
    published. The refusal names the denominator convention, so the caller
    learns what the value would need.
    """
    signal = catalogue.signal("held_share_world")
    assert signal.form is not None
    assert signal.form.name == "share"
    assert not signal.invertible
    with pytest.raises(ValueError, match="per_field"):
        signal.invert(1000.0)


def test_a_statistic_refuses_an_inversion_and_sends_the_reader_to_the_channels(
    catalogue: SignalCatalogue,
) -> None:
    """A statistic groups several forms over one quantity.

    The seven order statistics of a power quantity hold shares and signed
    relations together, so the positions do not share one rule. **This is the
    field class the layout cannot classify at the field level**, and the schema
    says so rather than naming a form the positions do not all hold.
    """
    signal = catalogue.signal("power_held_tiles")
    assert signal.form is not None
    assert signal.form.name == "statistic"
    assert not signal.form.uniform
    assert not signal.invertible
    with pytest.raises(ValueError, match="channels"):
        signal.invert(1000.0)


def test_only_a_magnitude_carries_an_inversion_error(
    catalogue: SignalCatalogue,
) -> None:
    """A form that truncates a logarithm cannot recover a count exactly.

    A caller that reports a recovered count must report this bound beside it. A
    difference smaller than the bound is not a difference the observation
    carries. A form that takes no logarithm has no such error.
    """
    magnitude = catalogue.signal("population")
    share = catalogue.signal("held_share_world")
    assert magnitude.form is not None
    assert share.form is not None
    assert magnitude.form.relative_precision > 0.0
    assert magnitude.form.relative_precision < 0.001
    assert share.form.relative_precision == 0.0


def test_a_field_that_names_an_unpublished_form_is_refused() -> None:
    """Prove the resolution can fail rather than leave a field without a form.

    The engine derives the form of a field and the form table from one
    declaration, so it cannot publish a name the table lacks. A reader that
    took the absence for a field with no form would invert nothing and report
    no error, so the catalogue refuses instead.

    This builds a layout by hand, because the engine cannot produce the case.
    """

    class Broken:
        """A world whose schema names a form it does not publish."""

        def observation_schema(self) -> dict[str, object]:
            """Return a layout of one field that names a missing form."""
            return {
                "version": 1,
                "length": 1,
                "value_forms": {},
                "fields": [
                    {"name": "count", "start": 0, "positions": 1, "form": "magnitude"}
                ],
            }

    with pytest.raises(KeyError, match="magnitude"):
        SignalCatalogue.of_world(Broken())


def test_a_catalogue_of_a_schema_with_no_form_table_still_reads(
    world: World,
) -> None:
    """A schema that states no form gives the catalogue it gave before forms.

    The form entry defaults to absent, so a caller that built a catalogue by
    hand builds the same one it built before. Such a signal refuses an
    inversion and says that the schema stated no form.
    """
    schema = dict(world.observation_schema())
    del schema["value_forms"]
    for row in schema["fields"]:
        row.pop("form", None)

    class Formless:
        """A world whose schema states no value form."""

        def observation_schema(self) -> dict[str, object]:
            """Return the layout with every form entry removed."""
            return schema

    catalogue = SignalCatalogue.of_world(Formless())
    signal = catalogue.signal("population")
    assert signal.form is None
    assert not signal.invertible
    with pytest.raises(ValueError, match="states no value form"):
        signal.invert(1000.0)


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
