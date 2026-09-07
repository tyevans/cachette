"""A stored policy refuses a world it was not trained against.

**The case this file exists for is the silent one.** The observation length
counts the cells of the block lattice, and a block is a fixed number of tiles
on a side. Every world from one block to two blocks on each axis therefore
holds four cells and an observation length of 176 at three factions. A policy
trained on a world 48 tiles on a side loads on one 64 tiles on a side, reads an
array of the length it expects, and plays a world 78 per cent larger. Nothing
raised before this check existed, because no array had a shape to disagree
about.

A world of another band raises without this check, but it raises from inside a
matrix product and the message names neither world. That is the loud half of
the same defect.

The tests below drive the real callers. The trainer resumes, and the report
pass plays a stored file.

References
----------
[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables, decision D2.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^2]: Report 33, what a learner can see, say and be scored on, section 5.
``docs/research/reports/33-what-a-learner-can-see-and-say.md``
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig
from cachette.learn.policy import (
    LinearPolicy,
    MLPPolicy,
    PolicyFit,
    PolicyFitError,
    load_policy,
)
from cachette.learn.reward import Weighting

# A weighting with every weight set. The reward is not what these tests
# measure, so the values only have to be present.
WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=1.0, lost=-1.0, drawn=0.0)

# The world the training runs use.
TRAINED = EnvConfig(width=48, height=48, faction_count=3, tick_limit=40, horizon=4)

# A world of the same lattice band. It holds four cells and the same
# observation length, and it holds 4096 tiles against 2304.
SAME_BAND = EnvConfig(width=64, height=64, faction_count=3, tick_limit=40, horizon=4)

# A world of another lattice band. It holds nine cells, so its observation
# length differs and a matrix product would refuse it on its own.
OTHER_BAND = EnvConfig(width=96, height=96, faction_count=3, tick_limit=40, horizon=4)


def fit_of(config: EnvConfig) -> PolicyFit:
    """Return the fit of the world one configuration builds."""
    return PolicyFit.of_env(Env(config, WEIGHTING))


def store(path: Path, config: EnvConfig) -> None:
    """Write one linear policy trained against one world."""
    env = Env(config, WEIGHTING)
    policy = LinearPolicy.zeros(env.action_length, env.observation_length)
    policy.save(path, PolicyFit.of_env(env).as_meta())


def test_the_two_bands_are_the_case_this_check_exists_for() -> None:
    """The trained world and the same-band world share both lengths.

    **This test is the fixture, not the assertion.** It states the condition
    that makes the silent case possible. A change to the block edge or to the
    field list can end that condition, and this test then fails and says so,
    rather than letting the refusal test pass against a world that any matrix
    product would have refused anyway.
    """
    trained = fit_of(TRAINED)
    same = fit_of(SAME_BAND)
    other = fit_of(OTHER_BAND)

    assert trained.observation_length == same.observation_length
    assert trained.action_length == same.action_length
    assert (trained.width, trained.height) != (same.width, same.height)
    assert trained.observation_length != other.observation_length


def test_a_policy_plays_the_world_it_was_trained_against(tmp_path: Path) -> None:
    """A file whose fit matches the world loads and gives back a policy."""
    path = tmp_path / "trained.npz"
    store(path, TRAINED)

    policy, meta = load_policy(path, fit_of(TRAINED))

    assert isinstance(policy, LinearPolicy)
    assert meta["width"] == TRAINED.width


def test_a_policy_refuses_a_world_of_the_same_length(tmp_path: Path) -> None:
    """A file refuses a larger world that holds the same two lengths.

    This is the case that nothing caught. The message must name both sides,
    because the lengths agree and a reader has nothing else to go on.
    """
    path = tmp_path / "trained.npz"
    store(path, TRAINED)

    with pytest.raises(PolicyFitError) as caught:
        load_policy(path, fit_of(SAME_BAND))

    message = str(caught.value)
    assert "width" in message
    assert "48" in message
    assert "64" in message
    # The lengths agree, so the message must not claim they differ.
    assert "observation_length: the file says" not in message


def test_a_policy_refuses_a_world_of_another_length(tmp_path: Path) -> None:
    """A file refuses a world of another lattice band, and names both lengths."""
    path = tmp_path / "trained.npz"
    store(path, TRAINED)

    with pytest.raises(PolicyFitError) as caught:
        load_policy(path, fit_of(OTHER_BAND))

    message = str(caught.value)
    assert "observation_length" in message
    assert str(fit_of(TRAINED).observation_length) in message
    assert str(fit_of(OTHER_BAND).observation_length) in message


def test_a_policy_refuses_a_world_of_another_faction_count(tmp_path: Path) -> None:
    """A file refuses a world that seats a different number of factions."""
    path = tmp_path / "trained.npz"
    store(path, TRAINED)
    other = EnvConfig(width=48, height=48, faction_count=4, tick_limit=40, horizon=4)

    with pytest.raises(PolicyFitError) as caught:
        load_policy(path, fit_of(other))

    assert "faction_count" in str(caught.value)


def test_a_file_that_states_no_fit_cannot_be_placed(tmp_path: Path) -> None:
    """A file written before the fit existed refuses every world.

    A file that names none of the fit keys states nothing about the world it
    was trained against. It could be right and it could be wrong, and nothing
    can tell the two apart, so the reader refuses it.
    """
    path = tmp_path / "old.npz"
    env = Env(TRAINED, WEIGHTING)
    LinearPolicy.zeros(env.action_length, env.observation_length).save(
        path, {"generation": 3}
    )

    with pytest.raises(PolicyFitError) as caught:
        load_policy(path, fit_of(TRAINED))

    assert "states no fit" in str(caught.value)


def test_a_reader_that_asks_for_no_fit_still_reads_an_old_file(tmp_path: Path) -> None:
    """A caller that only reports what a file says takes it unchecked.

    The trainer reads a stored best score out of a file it never plays. That
    read must keep working against a file from an earlier run.
    """
    path = tmp_path / "old.npz"
    env = Env(TRAINED, WEIGHTING)
    LinearPolicy.zeros(env.action_length, env.observation_length).save(
        path, {"best_score": 1.5}
    )

    _, meta = load_policy(path)

    assert meta["best_score"] == 1.5


def test_a_network_policy_carries_the_same_fit(tmp_path: Path) -> None:
    """The check reads the file and not the kind of policy inside it."""
    path = tmp_path / "net.npz"
    env = Env(TRAINED, WEIGHTING)
    policy = MLPPolicy.zeros(env.action_length, env.observation_length, hidden=4)
    policy.save(path, PolicyFit.of_env(env).as_meta())

    loaded, _ = load_policy(path, fit_of(TRAINED))
    assert isinstance(loaded, MLPPolicy)

    with pytest.raises(PolicyFitError):
        load_policy(path, fit_of(SAME_BAND))


def test_the_engine_owns_the_version_the_file_states(tmp_path: Path) -> None:
    """The fit takes both versions from the schemas of the world.

    A constant in the learner package would be a second declaration of a
    number the engine owns, and a version bump would leave every new file
    stating the old number.
    """
    env = Env(TRAINED, WEIGHTING)
    # **The reset is asserted, not folded into the value.** Writing this as
    # `env.reset(0) is not None and env.world` gives a value that is either
    # False or a world, and a reader of it must then handle a boolean that
    # cannot happen.
    assert env.reset(0) is not None
    world = env.world
    fit = PolicyFit.of_env(env)

    assert fit.observation_version == int(world.observation_schema()["version"])
    assert fit.action_version == int(world.action_schema()["version"])


def test_a_policy_trained_on_one_world_misplays_the_other_without_the_check(
    tmp_path: Path,
) -> None:
    """The unchecked path runs on the wrong world and reports no fault.

    **This is the defect, put back.** The reader takes no fit, so it does not
    refuse. The policy then scores a world of another extent and gives back an
    action, which is the outcome the check exists to prevent.
    """
    path = tmp_path / "trained.npz"
    store(path, TRAINED)
    policy, _ = load_policy(path)

    wrong = Env(SAME_BAND, WEIGHTING)
    observation = wrong.reset(1)
    action = policy.choose(observation, wrong.action_mask())

    assert isinstance(action, int)
    assert len(observation) == Env(TRAINED, WEIGHTING).observation_length


def test_the_trainer_refuses_a_checkpoint_from_another_world(tmp_path: Path) -> None:
    """The resume path drives the check, and it is not a call in a test alone.

    A run that resumes takes the centre of the earlier run. A centre from a
    world of another extent is not the centre of this run.
    """
    from cachette.learn.train import TrainConfig, train

    out = tmp_path / "run"
    out.mkdir()
    store(out / "s-latest.npz", SAME_BAND)

    with pytest.raises(PolicyFitError) as caught:
        train(
            name="s",
            env_config=TRAINED,
            weighting=WEIGHTING,
            train_config=TrainConfig(generations=1, population=2),
            out_dir=out,
            seed_pool=[0, 1],
            resume=True,
        )

    assert "does not fit this world" in str(caught.value)


def test_the_fit_reads_back_from_the_file_it_wrote(tmp_path: Path) -> None:
    """A fit written into a file reads back as the same fit."""
    path = tmp_path / "trained.npz"
    store(path, TRAINED)

    _, meta = load_policy(path)

    assert PolicyFit.read(meta) == fit_of(TRAINED)


def test_a_partial_fit_is_no_fit(tmp_path: Path) -> None:
    """A file that names some keys and not others cannot be placed."""
    trained = fit_of(TRAINED)
    partial = trained.as_meta()
    del partial["faction_count"]

    assert PolicyFit.read(partial) is None
    assert np.isfinite(trained.observation_length)
