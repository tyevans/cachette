"""A stored policy may hold a seat of every world a training run plays.

The learner takes one seat and the built-in controller takes the rest. An
owner who wants the learner to meet a policy rather than the controller seats
a stored weight file in one of those seats.

**The opponent holds its seat in every pass of the run**: the training
episodes, the validation, the held-out measurement and the controller bar.
The bar gives the learner seat back to the controller and leaves the opponent
where it is, so the bar measures the controller against the same opponent.

**The built-in controller keeps at least one seat.** It is the yardstick the
run is measured against, and a run that seated a policy in every seat would
report a number that no policy could move.

The worlds below are small and the episodes are short, because a world of the
training extent takes minutes for each episode.

References
----------
[^1]: The learner environment, the loop a learner drives.
``python/cachette/learn/env.py``
"""

from __future__ import annotations

import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest

from cachette.learn.baseline import BaselineCache
from cachette.learn.env import (
    Env,
    EnvConfig,
    Opponent,
    VectorEnv,
    file_sha256,
    seat_opponents,
    viable_seeds,
)
from cachette.learn.league import SeatedGame
from cachette.learn.policy import LinearPolicy, PolicyFitError, load_policy
from cachette.learn.reward import Weighting
from cachette.learn.train import Checkpoint
from test_launcher_trainer_calls import STAND_IN_TRAINER

# A world that a test can play to the end of a few decisions in seconds. It is
# the world the start file tests play.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

# The command line arguments that name the world above.
WORLD_ARGUMENTS = ("--world-extent", "24", "--tick-limit", "200")

# A reward that scores held ground. The tests read the play and not the score,
# and every environment needs one.
REWARD = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

# How many decisions a trace covers. A difference between two plays shows in
# the first few decisions, and each decision costs ten ticks of the world.
DECISIONS = 8

# The seat the first opponent takes. The learner holds seat 0, so the first
# free seat is seat 1.
OPPONENT_SEAT = 1


def write_policy(path: Path, world: EnvConfig, weights: np.ndarray | None) -> Path:
    """Write a weight file of one world, and return where it went.

    A weight entry of ``None`` writes the untrained centre, which scores every
    row at zero and therefore takes the no-op at every decision. Any other
    entry writes those weights, which choose a row that is not the no-op.
    """
    probe = Env(world, REWARD)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    policy = zeros if weights is None else zeros.rebuild(weights)
    checkpoint = Checkpoint(
        name="opponent",
        out_dir=path.parent,
        env_config=world,
        probe=probe,
        kind="linear",
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    checkpoint.write(policy, path, 0, 0.0, None, None)
    return path


def acting_weights(world: EnvConfig, seed: int = 7) -> np.ndarray:
    """Return weights that do not score every action row the same.

    The untrained centre scores every row at zero and takes the no-op. A
    drawn matrix scores the rows apart, so the policy takes a row the engine
    can carry out.
    """
    probe = Env(world, REWARD)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    generator = np.random.default_rng(seed)
    return generator.standard_normal(zeros.flat().shape)


def seat_one_trace(config: EnvConfig, seed: int) -> np.ndarray:
    """Play a few decisions and return what the seat one faction saw.

    The learner takes the no-op at every decision, so the only thing that
    moves between two calls of this is what holds seat one.

    **The pass goes through the vector the trainer plays.** A test that drove
    one environment by hand would prove nothing about the path a generation
    takes.
    """
    vector = VectorEnv(config, REWARD, count=1)
    vector.reset([seed])
    rows: list[np.ndarray] = []
    for _ in range(DECISIONS):
        if vector.done:
            break
        vector.step([0])
        rows.append(np.asarray(vector.envs[0].world.faction_observation(1)).copy())
    return np.concatenate(rows)


@pytest.fixture
def seed() -> int:
    """Return the first seed that builds a world every faction can play."""
    return viable_seeds(WORLD, 1)[0]


@pytest.fixture
def idle_opponent(tmp_path: Path) -> EnvConfig:
    """Return a world whose seat one holds a policy that takes the no-op."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    return seat_opponents(WORLD, [path])


@pytest.fixture
def acting_opponent(tmp_path: Path) -> EnvConfig:
    """Return a world whose seat one holds a policy that chooses real rows."""
    path = write_policy(tmp_path / "acting-latest.npz", WORLD, acting_weights(WORLD))
    return seat_opponents(WORLD, [path])


def test_a_stored_policy_takes_the_seat_from_the_built_in_controller(
    seed: int, idle_opponent: EnvConfig
) -> None:
    """Seat one plays differently once a stored policy holds it.

    The stored policy here takes the no-op at every decision, and the
    built-in controller does not. **A seat that fell back to the controller
    would give the same play**, so this fails when the opponent never takes
    its seat.
    """
    controller = seat_one_trace(WORLD, seed)
    stored = seat_one_trace(idle_opponent, seed)
    assert not np.array_equal(controller, stored)


def test_the_actions_the_stored_policy_chooses_reach_the_engine(
    seed: int, idle_opponent: EnvConfig, acting_opponent: EnvConfig
) -> None:
    """Two stored policies of one seat play two different games.

    Both worlds take the seat from the built-in controller, so the only
    difference between them is the row each policy chooses. **A seat whose
    chosen action never reached the engine would give the same play**,
    whatever the weights said.
    """
    idle = seat_one_trace(idle_opponent, seed)
    acting = seat_one_trace(acting_opponent, seed)
    assert not np.array_equal(idle, acting)


def test_the_controller_bar_keeps_the_opponent_seated(
    seed: int, acting_opponent: EnvConfig
) -> None:
    """The bar gives back the learner seat and holds the opponent seat.

    The bar measures the built-in controller in the learner's own seat. A bar
    that dropped the opponent would measure a game the learner never played.
    """
    plain = seat_one_trace(replace(WORLD, controlled=False), seed)
    against = seat_one_trace(replace(acting_opponent, controlled=False), seed)
    assert not np.array_equal(plain, against)


def test_a_list_that_fills_every_seat_but_the_learner_seat_is_refused(
    tmp_path: Path,
) -> None:
    """The built-in controller keeps a seat, so two opponents are too many."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    with pytest.raises(ValueError, match="every seat"):
        seat_opponents(WORLD, [path, path])


def test_an_opponent_cannot_hold_the_learner_seat(tmp_path: Path) -> None:
    """A world that seats an opponent where the learner sits is refused."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    opponent = Opponent.of_file(path, WORLD.seat)
    with pytest.raises(ValueError, match="learner seat"):
        Env(replace(WORLD, opponents=(opponent,)), REWARD)


def test_an_opponent_of_another_world_is_refused_before_any_episode(
    tmp_path: Path,
) -> None:
    """A file trained against another world never plays this one."""
    other = replace(WORLD, width=32, height=32)
    path = write_policy(tmp_path / "other-latest.npz", other, None)
    with pytest.raises(PolicyFitError):
        Env(seat_opponents(WORLD, [path]), REWARD)


def test_a_file_that_changed_after_the_run_recorded_it_is_refused(
    tmp_path: Path,
) -> None:
    """The digest is the identity of an opponent, and a changed file is refused."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    config = seat_opponents(WORLD, [path])
    write_policy(path, WORLD, acting_weights(WORLD))
    with pytest.raises(ValueError, match="changed after the run started"):
        Env(config, REWARD)


def test_a_seated_game_refuses_a_stored_opponent(idle_opponent: EnvConfig) -> None:
    """A league game drives no opponent, so it refuses one rather than dropping it."""
    with pytest.raises(ValueError, match="drives no stored opponent"):
        SeatedGame(idle_opponent, REWARD, [0, 1])


def test_the_baseline_key_names_the_opponent_by_seat_and_digest(
    tmp_path: Path, idle_opponent: EnvConfig, acting_opponent: EnvConfig
) -> None:
    """A bar measured against one opponent never answers for another.

    **A world with no opponent states no opponent entry at all**, so a bar
    that a run stored before this option existed still answers for it.
    """
    cache = BaselineCache(tmp_path, "engine")
    plain = cache.inputs(replace(WORLD, controlled=False), REWARD, [1, 2], 3)
    idle = cache.inputs(replace(idle_opponent, controlled=False), REWARD, [1, 2], 3)
    acting = cache.inputs(replace(acting_opponent, controlled=False), REWARD, [1, 2], 3)
    assert plain is not None
    assert idle is not None
    assert acting is not None
    assert "opponents" not in plain["world"]
    assert idle["world"]["opponents"] == [
        {
            "seat": OPPONENT_SEAT,
            "sha256": idle_opponent.opponents[0].sha256,
        }
    ]
    assert "path" not in idle["world"]["opponents"][0]
    assert cache.path_of(idle) != cache.path_of(plain)
    assert cache.path_of(idle) != cache.path_of(acting)


def test_the_written_checkpoint_names_each_opponent(
    tmp_path: Path, acting_opponent: EnvConfig
) -> None:
    """A weight file states the file name and the digest of every opponent."""
    probe = Env(acting_opponent, REWARD)
    checkpoint = Checkpoint(
        name="conquer",
        out_dir=tmp_path,
        env_config=acting_opponent,
        probe=probe,
        kind="linear",
    )
    target = tmp_path / "conquer-latest.npz"
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    checkpoint.write(zeros, target, 0, 0.0, None, None)
    _, meta = load_policy(target)
    assert meta["opponent_seats"] == [OPPONENT_SEAT]
    assert meta["opponent_files"] == ["acting-latest.npz"]
    assert meta["opponent_sha256"] == [file_sha256(tmp_path / "acting-latest.npz")]


def run_the_trainer(
    tmp_path: Path, *arguments: str
) -> subprocess.CompletedProcess[str]:
    """Run the trainer from its command line, up to its first seed search."""
    stand_in = tmp_path / "trainer.py"
    stand_in.write_text(STAND_IN_TRAINER, encoding="utf-8")
    return subprocess.run(
        [
            sys.executable,
            str(stand_in),
            *WORLD_ARGUMENTS,
            *arguments,
            "--out",
            str(tmp_path / "out"),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


def test_the_command_line_refuses_an_opponent_in_every_seat(tmp_path: Path) -> None:
    """Two opponents fill a world of three factions, so the run ends at once."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    finished = run_the_trainer(
        tmp_path, "--only", "conquer", "--opponent", str(path), "--opponent", str(path)
    )
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert "every seat" in finished.stderr


def test_the_command_line_refuses_an_opponent_beside_a_league(tmp_path: Path) -> None:
    """A league seats the candidates, and an opponent seats a stored file."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    finished = run_the_trainer(
        tmp_path, "--only", "conquer", "--opponent", str(path), "--league", "0,1"
    )
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert "Name one of them" in finished.stderr


def test_the_command_line_refuses_an_opponent_of_another_world(tmp_path: Path) -> None:
    """The fit is read before the rental, and a file of another world fails."""
    other = replace(WORLD, width=32, height=32)
    path = write_policy(tmp_path / "other-latest.npz", other, None)
    finished = run_the_trainer(tmp_path, "--only", "conquer", "--opponent", str(path))
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert "--opponent cannot seat" in finished.stderr


def test_the_command_line_takes_an_opponent_that_fits(tmp_path: Path) -> None:
    """A file of this world passes every check and the run reaches its seeds."""
    path = write_policy(tmp_path / "idle-latest.npz", WORLD, None)
    finished = run_the_trainer(tmp_path, "--only", "conquer", "--opponent", str(path))
    assert finished.returncode == 0, finished.stderr[-2000:]
    assert f"seat {OPPONENT_SEAT} holds the stored policy idle-latest.npz" in (
        finished.stdout
    )
