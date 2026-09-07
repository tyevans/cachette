"""The recorder reads the controller, and it reads it at the right moment.

A supervised dataset of controller play is worth nothing if the observation
of a sample is not the array the learner would have read. The tests here
drive the environment twice: once through the recorder, and once through a
plain loop that reads the same environment. The two must agree.

**Both assertions can fail, and both were made to fail before they were
trusted.** The first goes red when the recorder reads the observation after
the window rather than before it. The second goes red when the recorder loses
one tick of a window.[^1]

References
----------
[^1]: Testing Rules, sections 1 and 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.imitate import Dataset, fit_linear, fit_mlp, record, score
from cachette.learn.reward import Weighting

# A weighting with every weight set, so the reward runs. The values are the
# test's own and they state no rule of the downstream game.
WEIGHTING = Weighting(
    terms={"held_tiles": 1.0},
    won=100.0,
    lost=-100.0,
    drawn=0.0,
)

# A small world that runs quickly. The seat stays with the built-in
# controller, because that is the only configuration the recorder takes.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=120,
    horizon=12,
    decision_interval=5,
    controlled=False,
)


def a_seed() -> int:
    """Return one seed that seats every faction."""
    return viable_seeds(WORLD, 1, 900)[0]


def test_the_recorder_refuses_a_seat_it_would_hold_itself() -> None:
    """A configuration that takes the seat records nothing, so it is refused.

    The built-in controller emits no command for a faction under external
    control, so a recorder that took the seat would read an empty log for
    every window and label every one of them the no-op.
    """
    taken = EnvConfig(**{**WORLD.__dict__, "controlled": True})
    with pytest.raises(ValueError, match="built-in controller"):
        record(taken, WEIGHTING, [a_seed()])


def test_the_observation_of_a_window_is_the_one_the_window_starts_with() -> None:
    """The recorder reads the observation before the ticks of the window.

    A second loop over the same environment reads the observation at each
    decision boundary. The two sequences must be the same array for the same
    window. **The recorder must not read the array the window ends with**,
    because the learner acts on what it saw before it acted.
    """
    seed = a_seed()
    data = record(WORLD, WEIGHTING, [seed])

    env = Env(WORLD, WEIGHTING)
    observation = env.reset(seed)
    expected = []
    while not env.done:
        expected.append(observation)
        observation = env.step(0).observation
    stacked = np.stack(expected)

    assert data.observations.shape == stacked.shape
    assert np.array_equal(data.observations, stacked)
    # The rows must differ from one window to the next, or the assertion
    # above would hold for a recorder that read the wrong end of the window.
    assert not np.array_equal(stacked[:-1], stacked[1:])


def test_the_label_of_a_window_holds_every_command_of_that_window() -> None:
    """The label counts each command the controller gave inside the window.

    A second loop reads the raw command columns of the same environment, one
    tick at a time, and builds the same share for each action row. **A
    recorder that dropped one tick of the window would disagree here**, and
    nothing else in this file would notice.
    """
    seed = a_seed()
    data = record(WORLD, WEIGHTING, [seed])

    env = Env(WORLD, WEIGHTING)
    env.reset(seed)
    rows: list[np.ndarray] = []
    while not env.done:
        seen: list[np.ndarray] = []

        def collect(world: object, seen: list[np.ndarray] = seen) -> None:
            """Read the raw command columns of one tick."""
            columns = world.controller_actions(WORLD.seat)  # type: ignore[attr-defined]
            encoded = np.asarray(columns["encoded"])
            action = np.asarray(columns["action"], dtype=np.int64)
            seen.append(action[encoded > 0])

        env.step(0, on_tick=collect)
        taken = np.concatenate(seen) if seen else np.zeros(0, dtype=np.int64)
        row = np.zeros(env.action_length)
        if taken.size:
            row = np.bincount(taken, minlength=env.action_length).astype(np.float64)
            row /= float(taken.size)
        else:
            row[0] = 1.0
        rows.append(row)

    assert np.allclose(data.targets, np.stack(rows))
    # A window that holds several different rows is what makes this test
    # sharper than a count. A set of one-hot labels would pass a recorder
    # that kept only the last tick.
    assert float(np.count_nonzero(data.targets, axis=1).mean()) > 1.0


def test_the_same_seeds_give_the_same_dataset() -> None:
    """The recorder writes nothing, so a repeat gives the same arrays."""
    seed = a_seed()
    first = record(WORLD, WEIGHTING, [seed])
    second = record(WORLD, WEIGHTING, [seed])
    assert np.array_equal(first.observations, second.observations)
    assert np.array_equal(first.targets, second.targets)
    assert first.commands == second.commands


def test_the_recording_states_the_share_the_table_could_not_express() -> None:
    """Every command of a recorded window states whether it is an encoding.

    The engine emits no choice the table cannot express today, so the count
    is zero. **The count is the thing under test and not the zero**: a
    recorder that never read the column would report zero as well, and this
    test would then pass for the wrong reason. The window count below is what
    separates the two, because it proves the recorder saw commands at all.
    """
    data = record(WORLD, WEIGHTING, [a_seed()])
    assert data.commands > len(data)
    assert data.unencodable == 0
    assert data.lost_windows == 0


def test_a_fit_loads_in_the_shapes_the_trainer_plays() -> None:
    """Both fitted policies have the shape the trainer builds and plays.

    A fit that returned a matrix of another shape would fail only when a
    training run loaded it, which is hours later.
    """
    data = record(WORLD, WEIGHTING, viable_seeds(WORLD, 2, 900))
    env = Env(WORLD, WEIGHTING)
    linear = fit_linear(data)
    assert linear.shape == (env.action_length, env.observation_length + 1)
    network = fit_mlp(data, hidden=8)
    first, second = network.shapes
    assert first == (8, env.observation_length + 1)
    assert second == (env.action_length, 8)
    for policy in (linear, network):
        reading = score(policy, data)
        assert 0.0 <= reading["command_accuracy"] <= 1.0
        assert reading["windows"] == float(len(data))


def test_a_saved_dataset_reads_back_as_the_one_that_was_written(
    tmp_path: Path,
) -> None:
    """The recording survives a write and a read, with its counts.

    The recording is the expensive half of the module, so a caller writes it
    once and fits many times. **The counts are the part this test protects**,
    because an array round trip is obvious and a scalar dropped from the file
    would give a report a silent zero.
    """
    data = record(WORLD, WEIGHTING, [a_seed()])
    path = tmp_path / "dataset.npz"
    data.save(path)
    read = Dataset.load(path)
    assert np.array_equal(read.observations, data.observations)
    assert np.array_equal(read.targets, data.targets)
    assert np.array_equal(read.masks, data.masks)
    assert np.array_equal(read.episodes, data.episodes)
    assert np.array_equal(read.sizes, data.sizes)
    assert read.commands == data.commands
    assert read.silent_windows == data.silent_windows
    assert read.lost_windows == data.lost_windows
    assert read.unencodable == data.unencodable
