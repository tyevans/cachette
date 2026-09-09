"""Every strategy of a training run names a policy the trainer can build.

A training run reads a table of strategies. Each row names the world, what
the seat is rewarded for, and which policy shape the run trains. **The kind
in that row is the only thing that decides what a run trains**, so a row that
names a shape the builder does not know trains nothing, and a row that names
a shape nobody wants wastes the machine for hours.

The project trains two shapes. The linear policy scores each action row from
a weighted sum of every position of the observation. The structured policy
reads the ring stack and the entity token sets through shared weights, and it
trains every layer it holds.

The cosine between the step one generation takes and the direction it looks
for is near the square root of the pair count divided by the trainable
count.[^1] The linear policy is therefore the control with the most trainable
weights and the worst aligned step, and both shapes must stay in the table so
that a run can compare them.

References
----------
[^1]: Findings register, FND-668. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import math
from dataclasses import replace

import pytest

from cachette.learn.__main__ import (
    FOUND_READY,
    SETTLEMENT_TARGET,
    STRATEGIES,
    WORLD,
    strategy_table,
)
from cachette.learn.env import Env
from cachette.learn.policy import LinearPolicy
from cachette.learn.presets import ObjectiveSchedule
from cachette.learn.reward import Reward, Scoring, Weighting
from cachette.learn.search import shell_policy
from cachette.learn.structured import STRUCTURED_KIND, StructuredPolicy
from cachette.learn.train import first_scoring

# The two fields the founding chain of the table reads. The engine owns both
# names, and a name written twice here would be a second declaration of what
# the engine publishes.
SETTLEMENT_FIELD = "settlements"
READY_FIELD = "may_found"

# What the builder answers for each kind the table may name. A kind outside
# this mapping is a kind no run can train.
BUILT: dict[str, type] = {
    "linear": LinearPolicy,
    STRUCTURED_KIND: StructuredPolicy,
}


def _weighting(scoring: Scoring | ObjectiveSchedule) -> Weighting:
    """Return the weighting of one row, and refuse a row that holds none.

    Every row of the strategy table states a weighting. A row that stated an
    objective vector instead would hold no level weight, so the assertions
    below would read nothing rather than fail.
    """
    weighting = first_scoring(scoring)
    assert isinstance(weighting, Weighting), (
        "every row of the strategy table states a weighting"
    )
    return weighting


def _level_of(weighting: Weighting, field: str) -> float:
    """Return the level weight of one field, and refuse an unset weight."""
    weight = weighting.levels.get(field)
    assert weight is not None, f"the weighting states no level weight for {field}"
    return float(weight)


def _probe() -> Env:
    """Build the probe environment the table's own world states.

    Every row states one world, so this asserts that and then builds it once.
    A probe for each row would build the same world eight times.
    """
    configs = {config for config, _, _ in STRATEGIES.values()}
    assert len(configs) == 1, "the rows of the table state more than one world"
    config = next(iter(configs))
    scoring = first_scoring(next(iter(STRATEGIES.values()))[1])
    return Env(config, scoring)


def test_every_strategy_names_a_kind_the_trainer_can_build() -> None:
    """The table and the builder agree, and the builder is the one that runs.

    The trainer and every worker process call this builder, so a kind it does
    not know would train nothing. This drives the builder itself rather than
    reading the table alone.
    """
    probe = _probe()
    for name, (_, _, kind) in STRATEGIES.items():
        assert kind in BUILT, f"{name} names the kind {kind}, which nothing builds"
    for kind, wanted in BUILT.items():
        built = shell_policy(kind, probe)
        assert isinstance(built, wanted), f"the builder gave {type(built)} for {kind}"


def test_every_objective_of_the_table_has_a_structured_strategy() -> None:
    """Each reward the run trains against reaches the structured policy.

    A reward that only ever reached the linear policy would leave the
    structured policy untested against it, and the run could not say whether
    the reward or the policy carried a result.
    """
    linear = {
        name: scoring
        for name, (_, scoring, kind) in STRATEGIES.items()
        if kind == "linear"
    }
    structured = [
        scoring
        for _, (_, scoring, kind) in STRATEGIES.items()
        if kind == STRUCTURED_KIND
    ]
    assert linear, "the table holds no linear control"
    assert structured, "the table holds no structured strategy"
    for name, scoring in linear.items():
        assert scoring in structured, f"no structured strategy scores like {name}"


def test_the_structured_strategies_train_fewer_weights_than_the_linear_ones() -> None:
    """The structured policy is the smaller search shape, and this states it.

    A generation of an evolution strategy points the right way in proportion
    to the square root of the pair count divided by the trainable count, so
    the smaller shape is the better aligned one. **The linear policy stays in
    the table as the control**, and this asserts which of the two is which
    rather than assuming it.

    **The observation length is not the bound to compare against.** It was,
    while the action table held one row for each verb. A place argument gives
    the table one row for each cell of the frame, and the readout of a policy
    carries one weight for each row it scores, so the readout grew with the
    table while the observation did not move. The dense control grew with it
    and the structured shape did not, which is the claim this holds: the
    structured shape stays smaller by an order of magnitude, whatever the
    table costs.
    """
    probe = _probe()
    linear = shell_policy("linear", probe)
    structured = shell_policy(STRUCTURED_KIND, probe)
    assert structured.flat().size < linear.flat().size
    assert structured.flat().size * 10 < linear.flat().size


def test_no_row_of_the_table_scores_by_its_endpoints_alone() -> None:
    """Every row holds a shaped weight the optimiser sees inside the episode.

    An evolution strategy sums the reward of every decision of the episode
    with no discount, so a sum of first differences collapses to the last
    reading less the first. A row whose shaped weights are all change weights
    therefore trains against a terminal reward, whatever its weights look
    like, and nothing fails.[^2] [^3]

    Every row of this table held change weights alone once. This is the check
    that fails when one does again.

    References
    ----------
    [^2]: Findings register, FND-679. ``docs/FINDINGS.md``
    [^3]: Findings register, FND-700. ``docs/FINDINGS.md``
    """
    for name, (_, scoring, _) in STRATEGIES.items():
        weighting = _weighting(scoring)
        assert not weighting.telescopes, (
            f"{name} holds change weights alone, so its shaping pays one "
            f"number for the whole episode"
        )


def test_the_rows_that_play_for_ground_reward_the_settlement_chain() -> None:
    """Founding a settlement is several decisions long, and it must pay.

    A faction founds a city by queueing a settler, waiting for the settler,
    and settling with it. No shaped weight of this table paid anything at any
    step of that chain, and no trained policy has founded a city.

    This asserts that the two rows where founding is the point pay for the
    settlement and for the readiness that leads to it, and that both weights
    read a level. A change weight on either would collapse to the endpoints
    and give nothing to climb.
    """
    for name in ("conquer", "land"):
        weighting = _weighting(STRATEGIES[name][1])
        assert _level_of(weighting, SETTLEMENT_FIELD) > 0.0, (
            f"{name} pays nothing for a settlement"
        )
        assert _level_of(weighting, READY_FIELD) > 0.0, (
            f"{name} pays nothing for the readiness to found"
        )
        assert SETTLEMENT_FIELD not in weighting.terms
        assert READY_FIELD not in weighting.terms


def test_the_founding_ladder_never_outpays_the_founding_it_leads_to() -> None:
    """The readiness weight stays under the rise of one more settlement.

    The engine sets the readiness field while the settle verb is legal, and
    founding the settlement spends the settler and clears it. **A readiness
    weight above the rise of the settlement level therefore pays a faction to
    hold the settler and never found.**

    The bound is the rise of the settlement level for one more settlement at
    the target count, and this derives it from the schema of the world rather
    than restating it. The settlement field crosses as a compressed
    magnitude, so the rise falls as the count rises and the tightest bound is
    at the highest count the table means to reach.[^4]

    References
    ----------
    [^4]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decision D1.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """
    catalogue = _probe().signals
    form = catalogue.signal(SETTLEMENT_FIELD).form
    assert form is not None, "the schema must publish a form for the settlements"
    assert form.log_base == 2, "this bound assumes a base-two compression"
    assert form.divisor_bits is not None

    # The published value of a count is the unit times the logarithm of one
    # plus the count, over the divisor. The rise for one more settlement at
    # the target count is therefore the logarithm of the ratio, over the
    # divisor.
    rise = math.log2((SETTLEMENT_TARGET + 2) / (SETTLEMENT_TARGET + 1)) / float(
        form.divisor_bits
    )
    assert FOUND_READY < rise, (
        f"a readiness ratio of {FOUND_READY} outpays the rise of {rise} for "
        f"settlement {SETTLEMENT_TARGET + 1}, so a faction is paid to hold "
        f"the settler and never found"
    )

    for name in ("conquer", "land"):
        weighting = _weighting(STRATEGIES[name][1])
        ready = _level_of(weighting, READY_FIELD)
        settlement = _level_of(weighting, SETTLEMENT_FIELD)
        assert ready < settlement * rise, (
            f"the readiness weight of {name} outpays founding at settlement "
            f"{SETTLEMENT_TARGET + 1}"
        )


def test_every_level_weight_names_a_field_the_world_publishes() -> None:
    """A weight over a name the engine does not publish reaches nothing.

    The reward refuses such a name, so this builds the reward of every row
    against the world the row states. A row that named a field of another
    world would fail here rather than inside a run.
    """
    probe = _probe()
    probe.reset(0)
    world = probe.world
    for name, (_, scoring, _) in STRATEGIES.items():
        weighting = _weighting(scoring)
        assert Reward(world, probe.seat, weighting).faction == probe.seat, (
            f"{name} named a field the world does not publish"
        )


def test_a_wider_decision_interval_raises_every_level_weight() -> None:
    """The weights are a function of the world, and one function derives both.

    A level weight is paid on every decision, so a wider interval gives fewer
    decisions and each one must pay more. A table that kept its weights and
    took a new horizon would pay a fraction of its shaping, and nothing would
    fail.
    """
    narrow = strategy_table(replace(WORLD, decision_interval=10, horizon=250))
    wide = strategy_table(replace(WORLD, decision_interval=50, horizon=50))
    for name in narrow:
        thin_row = _weighting(narrow[name][1])
        thick_row = _weighting(wide[name][1])
        thin = thin_row.levels
        assert set(thin) == set(thick_row.levels)
        for field in thin:
            assert _level_of(thick_row, field) == pytest.approx(
                _level_of(thin_row, field) * 5.0
            )
