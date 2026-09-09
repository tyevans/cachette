"""Tests of the objective vector and the play styles that weight it.

One reward mechanism must train several styles of play. A style is a set of
weights over a vector of named objectives, and the scalar a learner receives
is the weighted combination of that vector.[^1]

Every test that scores a run drives a real world. It seeds the world, plays
it, and reads the objective vector through the public interface. A test that
fed the arithmetic a table of numbers would prove that the arithmetic works
and not that anything reaches the engine.[^2]

Two tests do build an observation array by hand, and they say so in their
names. They test the arithmetic of a term over a signal of many positions,
and the current world publishes zero in every such signal. A test that read
those positions from a live world would assert on nothing.[^3]

Every weight in these tests is a fixture value. One blocker holds the rules
of the downstream game, so no weight here is a proposal about the game.[^4]

References
----------
[^1]: Report 42, what a policy should be able to see, sections 10.2 and 10.3.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^2]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``

[^3]: Testing Rules, a uniform input hides a defect.
``.agents/rules/testing.md``

[^4]: Blockers register, BLK-050. ``docs/BLOCKERS.md``

[^5]: Findings register, FND-701. ``docs/FINDINGS.md``

[^6]: Recurring defect shapes, shape 1.
``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import math
from functools import cache

import numpy as np
import pytest

from cachette import World
from cachette.learn import (
    Aggregation,
    Env,
    EnvConfig,
    LibraryError,
    Measure,
    Objective,
    ObjectiveError,
    ObjectiveSchedule,
    ObjectiveScoring,
    ObjectiveSet,
    ObjectiveVector,
    Optimisation,
    PlayStyle,
    Signal,
    SignalCatalogue,
    StyleError,
    Term,
    TermKind,
    Variation,
    Weighting,
    load_library,
    rank_under,
    schedule_of,
)
from cachette.learn.policy import RandomPolicy
from cachette.learn.record import OBJECTIVE_PREFIX
from cachette.learn.rollout import run_population
from cachette.learn.style import EpisodeScore

# A world small enough to play inside a test, and wide enough that the
# seeding seats a faction in it.
# The horizon ends every episode before the game ends, so every candidate
# reports the running outcome. **That is what the ranking test needs.** A
# terminal weight of a hundred against a dense score of a few units decides
# the whole ranking, so a fixture in which one candidate loses measures the
# outcome and not the objective vector.
CONFIG = EnvConfig(
    width=24,
    height=24,
    faction_count=2,
    seat=0,
    tick_limit=600,
    horizon=45,
    decision_interval=4,
)

# A scoring that states nothing, for a probe that reads only the layout.
PROBE = Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0)

# The seeds the ranking test plays. Two seeds give each candidate two worlds,
# which is what a mean over a seed set needs in order to mean anything.
SEEDS = (11, 19)


def a_catalogue() -> SignalCatalogue:
    """Return the signal catalogue of the world these tests play."""
    world = World(
        width=CONFIG.width,
        height=CONFIG.height,
        seed=0,
        faction_count=CONFIG.faction_count,
    )
    world.seed_world()
    return SignalCatalogue.of_world(world)


def a_library():  # noqa: ANN201
    """Return the play style table this package ships."""
    return load_library()


@cache
def played_episodes(
    style: str, candidates: int = 4
) -> tuple[tuple[EpisodeScore, ...], ...]:
    """Play a population under one style, and return what each episode scored.

    The population is a set of random policies over different draw streams.
    Each one acts without reading the world, so each one drives the world
    somewhere else and the objective vector of each one differs.

    **The scoring of the batch is what a run uses.** The episode record
    carries the objective vector of each episode, and a later reader scores
    the same episodes under another style without playing them again. The
    combination is linear in the vector, so the sum over the decisions of the
    combination equals the combination of the sum.

    The result is cached, because playing a population costs several seconds
    and several tests read the same one.
    """
    scoring = a_library().scoring(style, a_catalogue())
    policies = [RandomPolicy(seed=index) for index in range(candidates)]
    record = run_population(CONFIG, scoring, policies, SEEDS, workers=1)
    grouped: list[list[EpisodeScore]] = [[] for _ in range(candidates)]
    for row in record.episodes:
        grouped[row.candidate].append(
            EpisodeScore(
                objectives=ObjectiveVector(dict(row.objectives)),
                outcome=row.outcome,
            )
        )
    return tuple(tuple(rows) for rows in grouped)


def spread(scores: tuple[float, ...]) -> float:
    """Return the highest score minus the lowest one."""
    return max(scores) - min(scores)


def test_two_play_styles_rank_the_same_candidates_differently() -> None:
    # This is the whole point of the objective vector. A run that ranked by
    # one scalar could not tell a candidate that led by fighting from one
    # that led by trading, so every style trained the same policy.
    played = played_episodes("aggressive")
    library = a_library()
    army = library.style("aggressive")
    trade = library.style("trade_led")

    by_army = rank_under(army, played)
    by_trade = rank_under(trade, played)

    # **The fixture must separate the candidates before the assertion means
    # anything.** A population that scored one number under a style would
    # rank in index order under it, and two such styles would agree by
    # accident.
    assert spread(by_army) > 0.0, f"the fixture is uniform under {army.name}"
    assert spread(by_trade) > 0.0, f"the fixture is uniform under {trade.name}"

    assert np.argsort(by_army).tolist() != np.argsort(by_trade).tolist(), (
        "two styles that rank the same episodes in the same order are "
        f"cosmetic. The army style scored {by_army} and the trade style "
        f"scored {by_trade}."
    )


def test_a_refusal_changes_the_ranking() -> None:
    # A refusal must reach the arithmetic. A style whose refusals changed
    # nothing would rank exactly as a style that weighted everything, and
    # every style of the table would then train one policy.
    #
    # **The objective this test adds must be one the fixture separates.** The
    # world here is small and the horizon is short, so most objectives read
    # nearly one number over the whole episode. Adding such an objective
    # shifts every score by nearly the same amount and preserves the order,
    # and the test then measures the fixture rather than the refusal. The
    # army strength is the objective that separates the candidates of this
    # fixture furthest, so this test adds that one.
    played = played_episodes("aggressive")
    library = a_library()
    trade = library.style("trade_led")
    assert "military" in trade.refuses

    also_army = PlayStyle(
        name="trade-and-army",
        weights={**trade.weights, "military": 3.0},
        refuses=tuple(name for name in trade.refuses if name != "military"),
        won=trade.won,
        lost=trade.lost,
        drawn=trade.drawn,
    )

    by_trade = rank_under(trade, played)
    by_both = rank_under(also_army, played)

    assert spread(by_trade) > 0.0
    assert spread(by_both) > 0.0
    assert np.argsort(by_trade).tolist() != np.argsort(by_both).tolist(), (
        "the refusal of an objective changed no ranking, so the refusal is "
        f"decoration. Trade scored {by_trade} and trade with the army "
        f"scored {by_both}."
    )


def test_every_episode_of_the_fixture_reports_the_same_outcome() -> None:
    # The ranking tests measure the objective vector. A terminal weight of a
    # hundred against a dense score of a few units decides the whole
    # ranking, so a fixture in which the outcomes differ measures the
    # outcome instead.
    played = played_episodes("aggressive")
    outcomes = {row.outcome for rows in played for row in rows}

    assert outcomes == {"running"}, (
        f"the fixture ends its episodes {sorted(outcomes)}, so a terminal "
        "weight decides the ranking rather than the objective vector"
    )


def test_a_run_records_the_objective_vector_of_every_episode() -> None:
    # The engine is obligated to reach the objective vector through the
    # environment. A test that built a scorer and read it would prove that
    # the scorer works and not that a run stores what it measured.
    played = played_episodes("aggressive", candidates=1)
    assert played[0], "the batch played no episode"

    library = a_library()
    declared = tuple(objective.name for objective in library.objectives)
    for row in played[0]:
        assert tuple(row.objectives.values) == declared

    scoring = library.scoring("aggressive", a_catalogue())
    record = run_population(CONFIG, scoring, [RandomPolicy(seed=0)], SEEDS, workers=1)
    row = record.episodes[0].as_row()
    for name in declared:
        assert f"{OBJECTIVE_PREFIX}{name}" in row
    assert record.objectives


def test_a_term_that_names_an_unknown_signal_fails_and_lists_the_signals() -> None:
    catalogue = a_catalogue()
    absent = Objective(
        name="absent",
        terms=(Term(signal="no_such_field", kind=TermKind.MAGNITUDE),),
    )

    with pytest.raises(ObjectiveError) as caught:
        ObjectiveSet([absent], catalogue)

    message = str(caught.value)
    assert "absent" in message
    assert "no_such_field" in message
    for signal in catalogue:
        assert signal.name in message


def test_a_style_that_names_an_unknown_objective_fails_and_lists_them() -> None:
    catalogue = a_catalogue()
    library = a_library()
    held = [objective.name for objective in library.objectives]
    style = PlayStyle(
        name="fixture",
        weights={"no_such_objective": 1.0},
        refuses=tuple(held),
    )

    with pytest.raises(StyleError) as caught:
        ObjectiveScoring(objectives=library.objective_set(catalogue), style=style)

    message = str(caught.value)
    assert "no_such_objective" in message
    for name in held:
        assert name in message


def test_a_style_that_says_nothing_about_an_objective_is_refused() -> None:
    # A style that weights everything trains what every other style trains.
    # Silence about an objective is therefore refused, and a refusal by name
    # is accepted.
    catalogue = a_catalogue()
    library = a_library()
    held = [objective.name for objective in library.objectives]
    silent = PlayStyle(name="silent", weights={held[0]: 1.0})

    with pytest.raises(StyleError) as caught:
        ObjectiveScoring(objectives=library.objective_set(catalogue), style=silent)

    assert "says nothing about" in str(caught.value)
    for name in held[1:]:
        assert name in str(caught.value)


def test_a_style_that_weights_an_objective_at_zero_is_refused() -> None:
    with pytest.raises(StyleError) as caught:
        PlayStyle(name="fixture", weights={"territory": 0.0})

    assert "zero" in str(caught.value)


def test_a_style_cannot_both_weight_and_refuse_one_objective() -> None:
    with pytest.raises(StyleError) as caught:
        PlayStyle(name="fixture", weights={"territory": 1.0}, refuses=("territory",))

    assert "both weights and refuses" in str(caught.value)


def test_a_term_over_a_signal_of_many_positions_reads_every_position() -> None:
    # The reward this replaces could reach no such signal at all. It refused
    # any field of more than one position, so no weighting could score trade
    # and no weighting could score diplomacy.
    catalogue = a_catalogue()
    compound = catalogue.compound()
    assert compound, "the fixture needs a signal of several positions"
    signal = compound[0]

    total = Objective(
        name="total",
        terms=(
            Term(
                signal=signal.name,
                kind=TermKind.MAGNITUDE,
                aggregation=Aggregation.SUM,
            ),
        ),
    )
    highest = Objective(
        name="highest",
        terms=(
            Term(
                signal=signal.name,
                kind=TermKind.MAGNITUDE,
                aggregation=Aggregation.HIGHEST,
            ),
        ),
    )
    objectives = ObjectiveSet([total, highest], catalogue)

    observation = np.zeros(catalogue.observation_length, dtype=np.int64)
    observation[signal.start : signal.start + signal.positions] = 7
    read = objectives.read(observation)

    # The sum over the positions is above the highest of them, and both are
    # above nothing. A term that read only the first position would score the
    # two objectives at the same number.
    assert read.values["total"] > read.values["highest"] > 0.0


def test_a_term_over_a_signal_of_many_positions_needs_an_aggregation() -> None:
    catalogue = a_catalogue()
    signal = catalogue.compound()[0]
    objectives = ObjectiveSet(
        [
            Objective(
                name="unaggregated",
                terms=(Term(signal=signal.name, kind=TermKind.MAGNITUDE),),
            )
        ],
        catalogue,
    )
    observation = np.zeros(catalogue.observation_length, dtype=np.int64)

    with pytest.raises(ValueError, match="needs an aggregation"):
        objectives.read(observation)


def test_a_share_term_stays_inside_its_bounds_on_any_world() -> None:
    # A term over a raw count means different things on two worlds. Every
    # term is therefore bounded, and a share that exceeds its denominator is
    # held at one rather than allowed to grow.
    catalogue = a_catalogue()
    scalars = catalogue.scalars()
    numerator = scalars[0]
    denominator = scalars[1]
    objectives = ObjectiveSet(
        [
            Objective(
                name="share",
                terms=(
                    Term(
                        signal=numerator.name,
                        kind=TermKind.SHARE,
                        against=denominator.name,
                    ),
                ),
            )
        ],
        catalogue,
    )
    observation = np.zeros(catalogue.observation_length, dtype=np.int64)
    observation[numerator.start] = 1_000_000
    observation[denominator.start] = 1

    assert objectives.read(observation).values["share"] == 1.0


def test_a_share_term_needs_a_denominator_and_a_magnitude_refuses_one() -> None:
    with pytest.raises(ObjectiveError, match="needs a second signal"):
        Term(signal="held_tiles", kind=TermKind.SHARE)
    with pytest.raises(ObjectiveError, match="bounds itself"):
        Term(signal="held_tiles", kind=TermKind.MAGNITUDE, against="world_tiles")


def test_a_run_cannot_be_configured_to_vary_the_objective_by_candidate() -> None:
    # An evolution strategy ranks the candidates of one generation against
    # each other. Two candidates scored under two objectives give a rank that
    # says nothing about either policy, so the update follows the noise.
    with pytest.raises(LibraryError) as caught:
        Variation.of("candidate")

    message = str(caught.value)
    assert "ranks the candidates" in message
    for entry in Variation:
        assert entry.value in message

    assert "candidate" not in {entry.value for entry in Variation}

    library = a_library()
    with pytest.raises(LibraryError, match="ranks the candidates"):
        schedule_of(
            library,
            ["aggressive", "trade_led"],
            a_catalogue(),
            variation="candidate",
        )


def test_a_schedule_gives_every_candidate_the_same_objective_per_position() -> None:
    # The safe form of variation is a pattern shared by every candidate. The
    # schedule answers for a generation and for a seed position, and it holds
    # no argument for a candidate.
    library = a_library()
    catalogue = a_catalogue()
    schedule = schedule_of(
        library, ["aggressive", "trade_led"], catalogue, variation="episode"
    )
    positions = schedule.for_generation(generation=0, episodes=4)

    assert [held.style.name for held in positions] == [
        "aggressive",
        "trade_led",
        "aggressive",
        "trade_led",
    ]
    assert schedule.for_generation(generation=1, episodes=4) == positions


def test_a_generation_schedule_holds_one_objective_for_the_whole_generation() -> None:
    library = a_library()
    catalogue = a_catalogue()
    schedule = schedule_of(
        library, ["aggressive", "trade_led"], catalogue, variation="generation"
    )

    first = schedule.for_generation(generation=0, episodes=3)
    second = schedule.for_generation(generation=1, episodes=3)

    assert {held.style.name for held in first} == {"aggressive"}
    assert {held.style.name for held in second} == {"trade_led"}


def test_a_fixed_schedule_holds_one_style() -> None:
    library = a_library()
    catalogue = a_catalogue()
    with pytest.raises(LibraryError, match="fixed schedule holds one scoring"):
        ObjectiveSchedule(
            scorings=(
                library.scoring("aggressive", catalogue),
                library.scoring("trade_led", catalogue),
            ),
            variation=Variation.FIXED,
        )


def test_a_change_term_is_refused_under_an_evolution_strategy() -> None:
    # An evolution strategy sums the reward over the whole episode with no
    # discount inside it. A term over a change then telescopes, so the whole
    # episode pays the level at the end minus the level at the start and the
    # shaping gives no signal inside the episode.
    catalogue = a_catalogue()
    scalar = catalogue.scalars()[0]
    changing = Objective(
        name="changing",
        terms=(
            Term(
                signal=scalar.name,
                kind=TermKind.MAGNITUDE,
                measure=Measure.DIFFERENCE,
            ),
        ),
    )
    objectives = ObjectiveSet([changing], catalogue)
    style = PlayStyle(name="fixture", weights={"changing": 1.0})

    assert objectives.telescoping() == ("changing",)

    with pytest.raises(StyleError, match="gives no signal inside the episode"):
        ObjectiveScoring(objectives=objectives, style=style)

    admitted = ObjectiveScoring(
        objectives=objectives, style=style, optimisation=Optimisation.GRADIENT
    )
    assert admitted.style.name == "fixture"


def test_the_shipped_table_binds_to_the_world_and_every_style_is_complete() -> None:
    # A style is data, and data that names a signal the world does not
    # publish must fail loudly. This binds every shipped style to the world
    # the tests play.
    library = a_library()
    catalogue = a_catalogue()
    assert library.names
    for name in library.names:
        scoring = library.scoring(name, catalogue)
        style = scoring.style
        assert set(style.weights) | set(style.refuses) == set(scoring.objectives.names)
        assert style.description


def test_the_combination_stays_inside_the_range_of_one_objective() -> None:
    # The weighted combination divides by the sum of the absolute weights, so
    # a fixed step size means the same thing under every style however many
    # objectives carry a weight.
    vector = ObjectiveVector({"a": 1.0, "b": 1.0, "c": -1.0})

    assert vector.combine({"a": 1.0}) == 1.0
    assert vector.combine({"a": 1.0, "b": 1.0}) == 1.0
    assert vector.combine({"a": 1.0, "c": 1.0}) == 0.0
    assert vector.combine({}) == 0.0


def test_a_weight_that_names_no_objective_fails_and_lists_the_objectives() -> None:
    vector = ObjectiveVector({"a": 1.0})

    with pytest.raises(ObjectiveError) as caught:
        vector.combine({"b": 1.0})

    assert "b" in str(caught.value)
    assert "a" in str(caught.value)


def test_an_environment_reports_no_objective_under_a_weighting() -> None:
    # A weighting collapses every term into one scalar, so it holds no
    # objective vector. The environment says so rather than inventing one.
    env = Env(
        CONFIG, Weighting(terms={"held_tiles": 1.0}, won=0.0, lost=0.0, drawn=0.0)
    )
    env.reset(SEEDS[0])

    assert env.objectives == {}


def test_an_environment_reports_the_objective_vector_under_a_style() -> None:
    scoring = a_library().scoring("aggressive", a_catalogue())
    env = Env(CONFIG, scoring)
    env.reset(SEEDS[0])
    mask = env.action_mask()
    env.step(int(np.flatnonzero(mask)[0]))

    assert set(env.objectives) == set(scoring.objectives.names)


def test_a_probe_scoring_states_nothing() -> None:
    # The probe of these tests must score every reading at zero, so that a
    # test which uses it measures the layout and not the probe.
    env = Env(CONFIG, PROBE)
    env.reset(SEEDS[0])
    mask = env.action_mask()
    result = env.step(int(np.flatnonzero(mask)[0]))

    assert result.reward == 0.0


def test_a_compressed_magnitude_term_takes_no_second_logarithm() -> None:
    """A term over a count the engine compressed reads the published value.

    The engine compresses every count before it publishes one. It divides the
    logarithm of the count by a divisor it states, and it scales the answer by
    a unit it states. A term that took its own logarithm of that value
    measured the logarithm of a logarithm, which is nearly flat: eight times
    the count moved such a term by a few hundredths of its range.

    This asserts the shape of the term rather than a number. A count and
    eight times that count must differ by three parts in the divisor, because
    the compression is a base-two logarithm and eight is three doublings.[^5]

    References
    ----------
    [^5]: Findings register, FND-701. ``docs/FINDINGS.md``
    """
    catalogue = a_catalogue()
    signal = catalogue.signal("population")
    form = signal.form
    assert form is not None, "the schema must publish a form for the population"
    assert form.log_base == 2, "this test assumes a base-two compression"
    divisor = form.divisor_bits
    assert divisor is not None

    objectives = ObjectiveSet(
        [
            Objective(
                name="people",
                terms=(Term(signal=signal.name, kind=TermKind.MAGNITUDE),),
            )
        ],
        catalogue,
    )

    def read(count: int) -> float:
        observation = np.zeros(catalogue.observation_length, dtype=np.int64)
        published = form.unit * math.log2(1.0 + count) / divisor
        observation[signal.start] = round(published)
        return objectives.read(observation).values["people"]

    small = read(1_000)
    large = read(8_000)
    step = 3.0 / float(divisor)

    assert large - small == pytest.approx(step, rel=0.01), (
        "three doublings of the count must move the term by three parts in the divisor"
    )


def test_a_bounded_term_reads_its_unit_and_its_divisor_from_the_schema() -> None:
    """Nothing in the objective module states the unit or the divisor.

    The engine owns both, and it publishes them in the value form table of
    the schema. A number written in the module would be a second declaration
    of an engine rule, and nothing would fail when the engine moved and the
    module did not.[^6]

    This builds a catalogue whose signals carry no form at all, and asserts
    that a bounded term over it refuses rather than falls back to a number of
    its own.

    References
    ----------
    [^6]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    bare = SignalCatalogue([Signal("count", 0, 1)], 4)
    for kind in (TermKind.MAGNITUDE, TermKind.FIXED_POINT):
        with pytest.raises(ObjectiveError, match="no unit and no divisor"):
            ObjectiveSet(
                [
                    Objective(
                        name="bounded",
                        terms=(Term(signal="count", kind=kind),),
                    )
                ],
                bare,
            )


def test_a_fixed_point_term_divides_by_the_published_unit() -> None:
    """A share reads one at the top of the range the engine published.

    The engine writes a share against a unit it states, and a term over such
    a field divides by that unit. This reads the unit from the schema and
    asserts that a value at the unit scores one.
    """
    catalogue = a_catalogue()
    signal = catalogue.signal("held_share_world")
    form = signal.form
    assert form is not None
    objectives = ObjectiveSet(
        [
            Objective(
                name="ground",
                terms=(Term(signal=signal.name, kind=TermKind.FIXED_POINT),),
            )
        ],
        catalogue,
    )
    observation = np.zeros(catalogue.observation_length, dtype=np.int64)
    observation[signal.start] = form.unit
    assert objectives.read(observation).values["ground"] == 1.0
    observation[signal.start] = form.unit // 4
    assert objectives.read(observation).values["ground"] == pytest.approx(0.25)


def test_a_compressed_magnitude_term_refuses_a_field_of_another_form() -> None:
    """The kind entry of a term is a check of a belief and not a label.

    The engine writes each field of the observation under one form and
    publishes that form in the schema. A compressed magnitude term over a
    field of another form takes a base-two logarithm of a value the engine
    already wrote against the unit. The answer stays inside the bounds, so
    nothing fails, and the term reads a quarter of its range for one part in
    a hundred of the quantity.

    Two objectives of the shipped table held that shape. This asserts that a
    term over a share and a term over a group of mixed forms are both
    refused.[^7]

    References
    ----------
    [^7]: Recurring defect shapes, shape 1, two copies that agree are still
    two copies. ``.agents/rules/recurring-defects.md``
    """
    catalogue = a_catalogue()
    share = catalogue.signal("wonder_progress")
    assert share.form is not None
    assert share.form.name == "share"

    with pytest.raises(ObjectiveError, match="as a compressed magnitude") as caught:
        ObjectiveSet(
            [
                Objective(
                    name="work",
                    terms=(Term(signal=share.name, kind=TermKind.MAGNITUDE),),
                )
            ],
            catalogue,
        )
    message = str(caught.value)
    assert "share" in message
    assert TermKind.FIXED_POINT.value in message

    mixed = catalogue.signal("trade_board")
    assert mixed.form is not None
    assert not mixed.form.uniform

    with pytest.raises(ObjectiveError, match="as a compressed magnitude"):
        ObjectiveSet(
            [
                Objective(
                    name="board",
                    terms=(
                        Term(
                            signal=mixed.name,
                            kind=TermKind.MAGNITUDE,
                            aggregation=Aggregation.SUM,
                        ),
                    ),
                )
            ],
            catalogue,
        )


def test_every_shipped_objective_names_the_kind_the_engine_wrote() -> None:
    """No objective of the table reads a field under the wrong rule.

    The refusal above covers one term at a time. This drives the shipped
    table itself, because a table that binds is the only proof that every
    term of it agrees with the schema of the world the run plays.
    """
    library = a_library()
    catalogue = a_catalogue()
    objectives = library.objective_set(catalogue)

    for objective in objectives:
        for term in objective.terms:
            form = catalogue.signal(term.signal).form
            assert form is not None, f"{term.signal} carries no form"
            compressed = form.invertible and form.uniform
            assert (term.kind is TermKind.MAGNITUDE) == compressed, (
                f"the objective {objective.name!r} reads {term.signal!r} as "
                f"{term.kind.value!r} and the engine wrote it as {form.name!r}"
            )


def test_the_renown_objective_reads_the_target_and_not_the_lead() -> None:
    """A renown win reads the target, so the objective reads the target.

    The engine publishes two shares of renown. One divides the best renown of
    the faction by the best renown of every faction, so it reads one whole as
    soon as the faction leads, at any renown. The other divides the best
    renown by the target a win needs, so it reads one whole only at the win.

    A style led by renown must climb toward the win and not toward the lead.
    This builds a state in which the faction leads on renown at a quarter of
    the target, and asserts that the objective reads the quarter.
    """
    catalogue = a_catalogue()
    objectives = a_library().objective_set(catalogue)
    lead = catalogue.signal("best_renown_share")
    target = catalogue.signal("renown_progress")
    form = target.form
    assert form is not None

    observation = np.zeros(catalogue.observation_length, dtype=np.int64)
    observation[lead.start] = form.unit
    observation[target.start] = form.unit // 4

    assert objectives.read(observation).values["renown"] == pytest.approx(0.25)


def test_the_trade_objective_separates_two_states_of_a_real_episode() -> None:
    """The trade objective must read two numbers on two board states.

    A bounded term that pins at one value over a whole episode ranks every
    candidate the same, and a style led by it trains nothing. The engine
    publishes a board of market statistics whose positions carry a price, a
    spread and a depth under three different rules, and no aggregation over
    them combines one rule.

    This plays a real episode and collects what the objective read at each
    decision. A fixture that never puts a good on the board would give one
    value and would measure the fixture, so the assertion names the count it
    saw.[^3]
    """
    library = a_library()
    catalogue = a_catalogue()
    objectives = library.objective_set(catalogue)
    env = Env(CONFIG, library.scoring("trade_led", catalogue))
    policy = RandomPolicy(seed=0)

    observation = env.reset(SEEDS[0])
    seen = set()
    while not env.done:
        seen.add(round(objectives.read(np.asarray(observation)).values["trade"], 9))
        mask = env.action_mask()
        action = policy.choose_many(observation[None, :], mask[None, :])[0]
        observation = env.step(action).observation

    assert len(seen) > 1, (
        f"the trade objective read one value, {seen}, over the whole episode. "
        "A style led by it would rank every candidate the same."
    )


def test_the_military_objective_reads_the_strength_and_not_the_head_count() -> None:
    """A faction of workers fields no army, and the objective must say so.

    A worker carries no attack and no armour, and an attacker whose attack
    does not exceed the armour of the defender inflicts nothing. A faction of
    workers can therefore take no casualty from anybody, however many workers
    it holds. The engine also holds no person that is not a unit, so the live
    unit count and the people are one number.

    A military objective over the unit count rewards bodies under the name of
    an army. This builds a state with many units and no strength, and asserts
    that the military objective reads nothing while the population objective
    reads the bodies.
    """
    catalogue = a_catalogue()
    objectives = a_library().objective_set(catalogue)
    strength = catalogue.signal("military_strength")
    people = catalogue.signal("population")
    units = catalogue.signal("live_units")
    form = people.form
    assert form is not None

    observation = np.zeros(catalogue.observation_length, dtype=np.int64)
    crowd = form.unit // 8
    observation[people.start] = crowd
    observation[units.start] = crowd
    observation[strength.start] = 0

    read = objectives.read(observation)
    assert read.values["military"] == 0.0, (
        "the military objective read a strength on a faction of workers, so "
        "it reads the head count and not the army"
    )
    assert read.values["population"] > 0.0


def test_the_renown_style_leads_on_renown_and_says_nothing_twice() -> None:
    """The renown style rewards two objectives and refuses the other seven.

    A contest raises the renown of the champion that fells a unit, so the
    renown objective pays for spending an army. The military objective pays
    for fielding one, and the style weights it below the renown because it is
    the means and not the end. The style pays for a win, because the renown
    path is a win path.
    """
    library = a_library()
    champion = library.style("renown_champion")
    objectives = tuple(objective.name for objective in library.objectives)

    assert set(champion.weights) == {"renown", "military"}
    assert set(champion.weights) | set(champion.refuses) == set(objectives)
    assert champion.won > 0.0
    assert champion.lost < 0.0
    assert "renown" in library.style("aggressive").refuses
