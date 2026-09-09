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

from cachette.learn.__main__ import STRATEGIES
from cachette.learn.env import Env
from cachette.learn.policy import LinearPolicy
from cachette.learn.search import shell_policy
from cachette.learn.structured import STRUCTURED_KIND, StructuredPolicy
from cachette.learn.train import first_scoring

# What the builder answers for each kind the table may name. A kind outside
# this mapping is a kind no run can train.
BUILT: dict[str, type] = {
    "linear": LinearPolicy,
    STRUCTURED_KIND: StructuredPolicy,
}


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
    """
    probe = _probe()
    linear = shell_policy("linear", probe)
    structured = shell_policy(STRUCTURED_KIND, probe)
    assert structured.flat().size < linear.flat().size
    assert structured.flat().size < probe.observation_length
