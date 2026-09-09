"""The stored feature normalizer, and what it does to the features.

A policy read the observation through a signed logarithm and nothing else.
Most positions of the observation never change over a run, so a weight over
one of them could only add a fixed offset to the score of an action row. The
constant part of the feature body outweighed the part that varies within an
episode, and an evolution strategy finds a per-row offset before it finds a
state-dependent rule, because the offset pays the same amount at every
decision of every episode.

These tests hold the transform that removes it. The encoder subtracts a
per-position centre and divides by a per-position scale, and both come from
one fixed reference sample of played episodes.

# What each test measures

The first two are the behavioural claims. **The reference distribution test
asserts an inequality that is reversed today**: the part of the feature body
that varies must outweigh the constant part plus the bias entry. The choice
test draws whole weight matrices and asks how much of the state a drawn
policy reads, which is what the defect showed as a single action row on
almost every decision.

The rest hold the storage: a normalizer derived twice is identical, a saved
policy round-trips it, a policy under another one is refused, and a file that
holds none still runs.

# The fixture supplies a held-out sample

The reference sample and the sample the assertions read come from two
disjoint seed ranges. A centre subtracted from the sample it was derived
from is exactly zero by construction, so an assertion over that sample would
measure the arithmetic and not the transform.[^1]

References
----------
[^1]: Testing Rules, sections 1, 2a and 6. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig
from cachette.learn.layout import ObservationLayout, RingBlock, token_blocks
from cachette.learn.normalize import (
    derive_normalizer,
    forget_normalizers,
    reference_normalizer,
    reference_observations,
)
from cachette.learn.picture import RingStack
from cachette.learn.policy import (
    FEATURE_SCALE_FLOOR,
    FeatureNormalizer,
    LinearPolicy,
    PolicyFit,
    PolicyFitError,
    encode,
    encode_many,
    load_policy,
    squash,
)
from cachette.learn.reward import Weighting
from cachette.learn.search import shell_policy
from cachette.learn.structured import StructuredPolicy

if TYPE_CHECKING:
    from pathlib import Path

# A weighting with every weight set. What the seat is rewarded for reaches no
# observation of the sample, so the values only have to be present.
WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=1.0, lost=-1.0, drawn=0.0)

# The world these tests derive a normalizer from. It is small and its
# episodes are short, because the cost of the fixture is the cost of the
# suite. Every assertion here is about the transform and not about the play.
CONFIG = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=12,
    decision_interval=4,
)

# A second world of the same two lengths. Nothing of the observation layout
# follows the world extent, so a policy of one loads into the other, and the
# normalizer is what separates them.
OTHER = EnvConfig(
    width=64,
    height=64,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=12,
    decision_interval=4,
)

# How many episodes and decisions the reference sample of these tests plays.
# The package holds larger defaults for a training run, and these tests state
# their own so that the suite is not the cost of a run.
EPISODES = 6
DECISIONS = 12

# Where the held-out sample starts. It is far above the reference range, so no
# seed of the reference sample reaches the assertions.
HELD_OUT_START = 500

# How many action rows the drawn weight matrices of the choice test score, and
# how many matrices it draws.
DRAWN_ROWS = 40
DRAWN_WEIGHTS = 24


def a_normalizer(config: EnvConfig = CONFIG) -> FeatureNormalizer:
    """Derive the normalizer of one world from the reference sample."""
    return derive_normalizer(config, WEIGHTING, episodes=EPISODES, decisions=DECISIONS)


def held_out(config: EnvConfig = CONFIG) -> np.ndarray:
    """Play a sample of worlds that the reference sample never reached."""
    return reference_observations(
        config,
        WEIGHTING,
        episodes=EPISODES,
        decisions=DECISIONS,
        seed_start=HELD_OUT_START,
    )


def constant_and_varying(
    rows: np.ndarray, normalizer: FeatureNormalizer
) -> tuple[float, float]:
    """Split one sample of features into its constant part and its varying part.

    The constant part is the length of the mean over the sample, which is the
    offset a weight can earn at every decision at once. The varying part is
    the root of the summed variance, which is what a weight has to read the
    state to earn.
    """
    body = normalizer.apply(squash(rows))
    return (
        float(np.linalg.norm(body.mean(axis=0))),
        float(np.sqrt(np.var(body, axis=0).sum())),
    )


def modal_share(
    rows: np.ndarray, normalizer: FeatureNormalizer, weights: np.ndarray
) -> float:
    """Say what share of the decisions one drawn policy spends on one action row.

    A policy that reads nothing scores every row by a fixed offset, so one row
    wins every decision and the share is one. A policy that reads the state
    spreads its choices.
    """
    masks = np.ones((rows.shape[0], DRAWN_ROWS), dtype=np.uint8)
    chosen = LinearPolicy(weights, normalizer).choose_many(rows, masks)
    counts = np.bincount(chosen, minlength=DRAWN_ROWS)
    return float(counts.max()) / float(len(chosen))


def drawn_weights(features: int) -> list[np.ndarray]:
    """Draw the weight matrices both arms of the choice test score.

    **Both arms score the same matrices.** A separate draw for each arm would
    let the difference between two samples of weights stand in for the
    difference between two feature transforms.
    """
    rng = np.random.default_rng(7)
    return [
        rng.standard_normal((DRAWN_ROWS, features + 1)) for _ in range(DRAWN_WEIGHTS)
    ]


def a_small_layout() -> ObservationLayout:
    """Build a layout that no engine publishes, for the storage tests.

    The rings hold 1, 6 and 12 cells, which is the shape of the real stack in
    miniature. A storage test that used the real layout would pay for a world
    it never reads.
    """
    stack = RingStack((1, 6, 12))
    ring = RingBlock.contiguous(0, stack, 4)
    tokens = token_blocks("tokens", ring.slots, (("first", 3, 5), ("second", 2, 7)))
    length = ring.slots + sum(block.slots for block in tokens) + 11
    return ObservationLayout(length=length, ring=ring, tokens=tokens)


def a_drawn_normalizer(length: int, seed: int = 5) -> FeatureNormalizer:
    """Build a normalizer of one length that no derivation would give.

    A storage test asks whether the two arrays survive a write and a read. It
    does not ask what they mean, so a draw separates them from the identity
    more cheaply than a played sample does.
    """
    rng = np.random.default_rng(seed)
    return FeatureNormalizer(rng.standard_normal(length), rng.uniform(0.2, 2.0, length))


def test_the_varying_part_outweighs_the_constant_part_and_the_bias() -> None:
    """The standardized features carry more signal than offset. Today they do not.

    The bias entry is a constant of one, and a weight over a position that
    never changes is a second bias. The quantity an evolution strategy trades
    is therefore the varying part against the constant part plus that one, and
    **the inequality runs the wrong way under the plain squash**.

    The sample is held out from the derivation. A centre subtracted from its
    own sample gives a mean of exactly zero, so the reference sample would
    make the first half of this assertion true by arithmetic.

    **The two constant parts are not comparable on their own.** A scale
    divides the residual mean shift as well as the spread, so the standardized
    constant part is the larger of the two in absolute terms. The ratio is the
    quantity the search trades, and it is the one that moves.
    """
    rows = held_out()
    identity = FeatureNormalizer.identity(rows.shape[1])
    normalizer = a_normalizer()

    plain_constant, plain_varying = constant_and_varying(rows, identity)
    constant, varying = constant_and_varying(rows, normalizer)

    assert plain_varying < plain_constant + 1.0
    assert varying > constant + 1.0
    assert varying / (constant + 1.0) > 2.0 * plain_varying / (plain_constant + 1.0)


def test_a_drawn_policy_reads_more_of_the_state_under_standardized_features() -> None:
    """A drawn weight matrix spreads its choices once the constant part is gone.

    This is the behavioural claim, so it is a property over drawn weights
    rather than one example. Under the plain squash at least one draw takes
    one action row on almost every decision, which is the shape the trained
    policies showed. Under the standardized features no draw takes one row on
    more than half of them.

    **Putting the defect back fails this test.** An identity normalizer makes
    the second arm the first arm, and the first arm reaches above nine tenths.
    """
    rows = held_out()
    identity = FeatureNormalizer.identity(rows.shape[1])
    normalizer = a_normalizer()
    draws = drawn_weights(rows.shape[1])

    plain = np.asarray([modal_share(rows, identity, held) for held in draws])
    standard = np.asarray([modal_share(rows, normalizer, held) for held in draws])

    assert plain.max() > 0.9
    assert standard.max() < 0.5
    assert standard.mean() < plain.mean()


def test_a_normalizer_derived_twice_from_one_configuration_is_identical() -> None:
    """Two derivations of one world give the same two arrays, byte for byte.

    Every input of the derivation is fixed: the seed list, the generator that
    draws the action of each decision, and the decision count. A normalizer
    that moved between two derivations would make the candidates of two
    generations incomparable.
    """
    first = a_normalizer()
    second = a_normalizer()

    assert np.array_equal(first.centre, second.centre)
    assert np.array_equal(first.scale, second.scale)
    assert first.digest() == second.digest()


def test_two_worlds_of_one_length_give_two_normalizers() -> None:
    """The normalizer separates two worlds that both lengths agree about.

    **This test is the fixture of the refusal test, not an assertion about
    it.** Nothing of the observation layout follows the world extent, so a
    policy of one world loads into the other and no length disagrees. The
    normalizer is the entry that differs.
    """
    here = PolicyFit.of_env(Env(CONFIG, WEIGHTING))
    there = PolicyFit.of_env(Env(OTHER, WEIGHTING))

    assert here.observation_length == there.observation_length
    assert a_normalizer(CONFIG).digest() != a_normalizer(OTHER).digest()


def test_the_cache_gives_one_derivation_to_every_caller() -> None:
    """The cached door answers with the object it derived the first time.

    A training run derives the normalizer once and hands the same one to every
    generation and every worker process. Two candidates that read two feature
    transforms are not comparable, so a rank over them says nothing.
    """
    forget_normalizers()
    first = reference_normalizer(
        CONFIG, WEIGHTING, episodes=EPISODES, decisions=DECISIONS
    )
    second = reference_normalizer(
        CONFIG, WEIGHTING, episodes=EPISODES, decisions=DECISIONS
    )

    assert first is second


def test_the_scale_takes_the_floor_where_a_position_barely_moves() -> None:
    """No scale falls under the floor, and a moving position keeps its own.

    A per-position scale divides the residual mean shift as well as the
    spread, so a scale near zero would amplify the offset that the centring
    removed. The floor caps that amplification.
    """
    rows = np.zeros((8, 4), dtype=np.int64)
    rows[:, 1] = np.arange(8)
    rows[:, 2] = np.arange(8) * 5000
    rows[4:, 3] = 1

    normalizer = FeatureNormalizer.of_observations(rows)
    spread = squash(rows).std(axis=0)

    assert normalizer.scale[0] == FEATURE_SCALE_FLOOR
    assert spread[1] < FEATURE_SCALE_FLOOR
    assert normalizer.scale[1] == FEATURE_SCALE_FLOOR
    assert normalizer.scale[2] == pytest.approx(spread[2])
    assert normalizer.scale.min() >= FEATURE_SCALE_FLOOR


def test_a_position_that_never_moves_reads_exactly_zero() -> None:
    """A centred constant position is zero, so its weight reaches no score.

    This is why the centre applies to every position and nothing is dropped.
    A dropped position cannot come back, and a measurement found no plateau in
    the count of positions that ever move. A centred one is inert while it
    stays still and live the moment it moves.
    """
    rows = np.zeros((6, 3), dtype=np.int64)
    rows[:, 0] = 41
    rows[:, 1] = np.arange(6) * 1000

    normalizer = FeatureNormalizer.of_observations(rows)
    features = encode_many(rows, normalizer)

    assert features[:, 0].tolist() == [0.0] * 6
    assert np.any(features[:, 1] != 0.0)


def test_the_bias_entry_is_never_centred_or_scaled() -> None:
    """The trailing feature stays one under every normalizer.

    It is the one feature that is a constant on purpose, so the transform that
    removes the accidental constants must leave it alone.
    """
    rows = np.arange(12, dtype=np.int64).reshape(3, 4) * 700
    normalizer = a_drawn_normalizer(4)

    one = encode(rows[0], normalizer)
    many = encode_many(rows, normalizer)

    assert one[-1] == 1.0
    assert many[:, -1].tolist() == [1.0, 1.0, 1.0]
    assert one[:-1].tolist() == many[0, :-1].tolist()


def test_no_normalizer_computes_the_plain_squash() -> None:
    """The identity reproduces the encoder this project shipped before it.

    A policy file written before the normalizer existed loads holding none, so
    the two must agree to the last bit. A difference here would move every
    published policy.
    """
    rows = np.arange(12, dtype=np.int64).reshape(3, 4) * 300
    identity = FeatureNormalizer.identity(4)

    assert encode_many(rows).tolist() == encode_many(rows, identity).tolist()
    assert encode(rows[0]).tolist() == encode(rows[0], identity).tolist()
    assert identity.is_identity


def test_a_saved_linear_policy_round_trips_its_normalizer(tmp_path: Path) -> None:
    """The two arrays come back from the file with the weights.

    A weight of the file scores a standardized feature, and nothing outside
    the file says which standardization that was.
    """
    path = tmp_path / "linear.npz"
    env = Env(CONFIG, WEIGHTING)
    normalizer = a_normalizer()
    policy = LinearPolicy.zeros(env.action_length, env.observation_length, normalizer)
    policy.save(path, PolicyFit.of_env(env, normalizer).as_meta())

    read, meta = load_policy(path, PolicyFit.of_env(env, normalizer))

    assert isinstance(read, LinearPolicy)
    assert read.normalizer is not None
    assert np.array_equal(read.normalizer.centre, normalizer.centre)
    assert np.array_equal(read.normalizer.scale, normalizer.scale)
    assert "normalizer_centre" not in meta


def test_a_saved_structured_policy_round_trips_its_normalizer(tmp_path: Path) -> None:
    """Every tower reads a standardized feature, so the file carries the arrays."""
    path = tmp_path / "structured.npz"
    layout = a_small_layout()
    normalizer = a_drawn_normalizer(layout.length)
    policy = StructuredPolicy.zeros(9, layout, normalizer=normalizer)
    policy.save(path, {})

    read, _ = load_policy(path)

    assert isinstance(read, StructuredPolicy)
    assert read.normalizer is not None
    assert np.array_equal(read.normalizer.centre, normalizer.centre)
    assert np.array_equal(read.normalizer.scale, normalizer.scale)


def test_a_policy_whose_normalizer_does_not_match_the_world_is_refused(
    tmp_path: Path,
) -> None:
    """A file trained under another standardization means something else.

    Both lengths agree, both versions agree and the action table agrees, so
    nothing else in the fit separates the two sides. The message names the
    digest of each one, because two arrays of a few thousand entries do not
    belong in a message.
    """
    path = tmp_path / "elsewhere.npz"
    env = Env(CONFIG, WEIGHTING)
    trained = a_normalizer()
    LinearPolicy.zeros(env.action_length, env.observation_length, trained).save(
        path, PolicyFit.of_env(env, trained).as_meta()
    )
    asked = a_drawn_normalizer(env.observation_length)

    with pytest.raises(PolicyFitError) as raised:
        load_policy(path, PolicyFit.of_env(env, asked))

    message = str(raised.value)
    assert "normalizer" in message
    assert trained.digest() in message
    assert asked.digest() in message


def test_a_matching_normalizer_is_not_a_refusal(tmp_path: Path) -> None:
    """The same two arrays on both sides load without a word."""
    path = tmp_path / "same.npz"
    env = Env(CONFIG, WEIGHTING)
    normalizer = a_normalizer()
    LinearPolicy.zeros(env.action_length, env.observation_length, normalizer).save(
        path, PolicyFit.of_env(env, normalizer).as_meta()
    )

    read, _ = load_policy(path, PolicyFit.of_env(env, a_normalizer()))

    assert isinstance(read, LinearPolicy)


def test_a_file_with_no_normalizer_loads_and_runs(tmp_path: Path) -> None:
    """A published file holds no normalizer, and it still plays.

    The four style files of this repository were written before the normalizer
    existed. A file that states none reads the plain squash, which is what the
    identity computes, and the fit compares two normalizers only when both
    sides state one.
    """
    path = tmp_path / "published.npz"
    env = Env(CONFIG, WEIGHTING)
    rng = np.random.default_rng(3)
    weights = rng.standard_normal((env.action_length, env.observation_length + 1))
    LinearPolicy(weights).save(path, PolicyFit.of_env(env).as_meta())

    read, _ = load_policy(path, PolicyFit.of_env(env, a_normalizer()))

    assert isinstance(read, LinearPolicy)
    assert read.normalizer is None
    rows = held_out()
    masks = np.ones((rows.shape[0], env.action_length), dtype=np.uint8)
    explicit = LinearPolicy(weights, FeatureNormalizer.identity(env.observation_length))
    assert read.choose_many(rows, masks) == explicit.choose_many(rows, masks)


def test_a_half_written_normalizer_is_refused(tmp_path: Path) -> None:
    """A file that names one array and not the other is not consistent with itself.

    A normalizer is written in one piece, so a file that holds half of one was
    written by something that disagreed with itself about what it was writing.
    """
    path = tmp_path / "half.npz"
    np.savez(
        path,
        weights=np.zeros((3, 5)),
        kind=np.array("linear"),
        normalizer_centre=np.zeros(4),
    )

    with pytest.raises(PolicyFitError) as raised:
        load_policy(path)

    assert "normalizer_scale" in str(raised.value)


def test_a_rebuilt_candidate_carries_the_normalizer() -> None:
    """Every candidate of every generation reads the normalizer of the run.

    The search rebuilds each candidate through the shell, so a normalizer that
    stopped at the shell would reach nothing the run scored. This drives the
    shell builder that the trainer and every worker process both call.
    """
    probe = Env(CONFIG, WEIGHTING)
    normalizer = a_normalizer()
    for kind in ("linear", "structured"):
        shell = shell_policy(kind, probe, normalizer)
        rebuilt = shell.rebuild(np.zeros(shell.flat().size))

        assert shell.normalizer is normalizer
        assert rebuilt.normalizer is normalizer


def test_a_normalizer_refuses_a_sample_of_another_length() -> None:
    """A normalizer is a function of one world, so it states its own length."""
    normalizer = a_drawn_normalizer(6)

    with pytest.raises(PolicyFitError) as raised:
        normalizer.apply(np.zeros((2, 7)))

    assert "6 positions" in str(raised.value)
