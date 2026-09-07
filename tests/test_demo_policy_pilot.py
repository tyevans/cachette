"""A stored policy holds a faction of the demonstration, and the frames drive it.

**These tests drive the frame, not the pilot.** The demonstration is obliged
to give a policy a decision for every tick the world ran. A test that called
the pilot directly would prove that the pilot works and prove nothing about
whether the loop reaches it.[^1]

Every test below builds a weight file for the world it runs, so no test reads
a checkpoint from the repository. A committed checkpoint states the world it
was trained against, and a test that depended on one would fail the day
somebody retrained it.[^2]

References
----------
[^1]: Testing Rules, section 5. ``.agents/rules/testing.md``

[^2]: The stored policy and what it was trained against.
``python/cachette/learn/policy.py``
"""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pytest

from cachette import World
from cachette.demo.app import Demo, main
from cachette.demo.clock import SPEEDS
from cachette.demo.pilot import PolicyChoiceError, chosen_policies, seat_policies
from cachette.demo.player import read_table
from cachette.demo.ui import status
from cachette.learn.policy import LinearPolicy, PolicyFit, PolicyFitError
from cachette.names import Names

# A world small enough to step many times in a test, and large enough for the
# founding survey to seat every faction. These are the numbers the seat tests
# use, so the two suites drive one world.
SIDE = 64
SEED = 0x0CAC_4E77_0472
FACTIONS = 3
WIDTH = 420
HEIGHT = 520

# How many ticks the world runs between two decisions, in these tests.
#
# **It is not the interval of any checkpoint in the repository.** A test that
# used the same number could not tell a pilot that reads the file from a pilot
# that holds a constant.
INTERVAL = 7

# How many frames a test draws while it waits for the world to move.
#
# The clock reads the wall clock, and a test must not assert on it. Every test
# below therefore drives a fixed number of frames and asserts on the tick.
FRAMES = 12


def build(ticks: int = 20) -> Demo:
    """Give back a seeded, stepped demonstration on the engine renderer."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=FACTIONS)
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.foundings = demo.seed()
    for _ in range(ticks):
        world.step(1)
    demo.open_on((SIDE // 2, SIDE // 2))
    # The fastest speed runs several ticks for each frame, so a test reaches
    # several decisions in a few frames.
    demo.clock.choose(len(SPEEDS) - 1)
    return demo


def weights_for(action: int, world: World) -> LinearPolicy:
    """Give back a policy that scores one row above every other row.

    The score of a row is a weighted sum over the features, and the bias is
    the last feature. A bias of one on the wanted row and zero elsewhere
    therefore names that row, whatever the observation holds.
    """
    schema = world.action_schema()
    policy = LinearPolicy.zeros(
        int(schema["length"]), int(world.observation_schema()["length"])
    )
    policy.weights[action, -1] = 1.0
    return policy


def write_policy(
    path: Path,
    world: World,
    action: int = 0,
    interval: int = INTERVAL,
    fit: PolicyFit | None = None,
    manifest: dict[str, object] | None = None,
) -> Path:
    """Write a weight file that fits this world, and give back its path."""
    held = fit if fit is not None else PolicyFit.of_world(world)
    meta: dict[str, object] = dict(held.as_meta())
    if interval > 0:
        meta["decision_interval"] = interval
    weights_for(action, world).save(path, meta)
    written = path.with_suffix(".npz")
    if manifest is not None:
        written.with_suffix(".json").write_text(json.dumps(manifest), encoding="utf-8")
    return written


def verb_named(world: World, name: str) -> int:
    """Give back the first action row of one verb, through the schema."""
    for verb in world.action_schema()["verbs"]:
        if str(verb["name"]) == name:
            return int(verb["first"])
    message = f"no verb is called {name!r}"
    raise AssertionError(message)


def test_the_frames_give_a_policy_one_decision_for_each_interval(
    tmp_path: Path,
) -> None:
    """The loop drives the policy, at the cadence the weight file names.

    This is the test that proves the wiring. The pilot is built by the
    demonstration and driven by the frames, and nothing here calls it.
    """
    demo = build()
    path = write_policy(tmp_path / "held", demo.world)
    demo.pilots = seat_policies(demo.world, [(1, path)])
    pilot = demo.pilots[0]

    # A pilot decides as it takes the seat, so one decision is already taken
    # and the world has not moved.
    assert pilot.decisions == 1
    assert pilot.interval == INTERVAL, "the cadence did not come from the file"
    assert demo.world.is_externally_controlled(1)

    was = demo.world.tick
    for _ in range(FRAMES):
        demo.advance()
    ran = demo.world.tick - was
    assert ran > 0, "the frames did not step the world"
    # One decision at the seat, and one for each whole interval the world ran.
    assert pilot.decisions == 1 + ran // INTERVAL, (
        f"the world ran {ran} ticks at an interval of {INTERVAL}, so the "
        f"policy owed {1 + ran // INTERVAL} decisions and took {pilot.decisions}"
    )


def test_the_policy_sends_the_action_it_chose_through_the_seat(
    tmp_path: Path,
) -> None:
    """The row the weights name is the row the engine receives.

    A count of decisions cannot tell a pilot that chooses from a pilot that
    sends the no-op every time. This names the row and reads it back from the
    words the seat recorded.
    """
    demo = build()
    wanted = verb_named(demo.world, "gather")
    path = write_policy(tmp_path / "gathers", demo.world, action=wanted)
    demo.pilots = seat_policies(demo.world, [(1, path)])
    pilot = demo.pilots[0]
    for _ in range(FRAMES):
        demo.advance()

    said = [choice for choice in read_table(demo.world) if choice.action == wanted]
    assert said, "the action table holds no row for the chosen verb"
    assert pilot.taken, "the policy sent no action through the seat"
    assert pilot.taken[-1].startswith(said[0].says()), (
        f"the policy sent {pilot.taken[-1]!r} and the weights named {said[0].says()!r}"
    )


def test_two_policies_hold_two_factions_of_one_world(tmp_path: Path) -> None:
    """A watcher compares two checkpoints by seating both at once.

    Each pilot keeps the cadence of its own file, so a comparison between two
    policies fitted at two intervals is still a fair one.
    """
    demo = build()
    first = write_policy(tmp_path / "first", demo.world, interval=INTERVAL)
    second = write_policy(tmp_path / "second", demo.world, interval=1)
    demo.pilots = seat_policies(demo.world, [(0, first), (2, second)])
    assert [pilot.faction for pilot in demo.pilots] == [0, 2]
    assert [pilot.interval for pilot in demo.pilots] == [INTERVAL, 1]
    assert demo.world.is_externally_controlled(0)
    assert demo.world.is_externally_controlled(2)
    assert not demo.world.is_externally_controlled(1), (
        "a faction nobody named left the hands of the engine controller"
    )

    was = demo.world.tick
    for _ in range(FRAMES):
        demo.advance()
    ran = demo.world.tick - was
    assert demo.pilots[1].decisions > demo.pilots[0].decisions, (
        "the two pilots kept one cadence, so the cadence is not read per file"
    )
    assert demo.pilots[1].decisions == 1 + ran, (
        f"the pilot at an interval of one owed {1 + ran} decisions over "
        f"{ran} ticks and took {demo.pilots[1].decisions}"
    )


def test_a_frozen_turn_holds_every_policy_as_well(tmp_path: Path) -> None:
    """A person who is choosing stops the world, so no policy decides.

    The cadence counts ticks. A frame that ran no tick therefore owes no
    decision, and a policy cannot act while the world is frozen.
    """
    demo = build()
    path = write_policy(tmp_path / "held", demo.world)
    demo.pilots = seat_policies(demo.world, [(1, path)])
    demo.take_seat(0)
    seat = demo.seat
    assert seat is not None
    assert seat.frozen

    was = demo.pilots[0].decisions
    hashed = demo.world.state_hash()
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.state_hash() == hashed, "the world moved while time was frozen"
    assert demo.pilots[0].decisions == was, (
        "a policy acted while the person at the keyboard was choosing"
    )


def test_the_title_block_names_the_file_that_holds_each_faction(
    tmp_path: Path,
) -> None:
    """A watcher comparing three checkpoints reads which one is playing.

    The card grows by one row for each policy, and each row names the file.
    """
    demo = build()
    first = write_policy(tmp_path / "gen9", demo.world)
    second = write_policy(tmp_path / "gen11", demo.world)
    bare = status.height_of(demo)
    demo.pilots = seat_policies(demo.world, [(0, first), (2, second)])
    assert [pilot.name for pilot in demo.pilots] == ["gen9", "gen11"]
    assert status.height_of(demo) > bare, "the card left no room for the policy rows"
    # The card draws into the frame the demonstration owns. A draw that raised
    # would leave the watcher with no card at all.
    status.paint(demo.surface, demo)


def test_a_checkpoint_of_another_world_stops_the_run(tmp_path: Path) -> None:
    """A file trained elsewhere is refused, and the message names the entry.

    The observation length counts lattice cells, so a world of another extent
    can hold the same length. The refusal must therefore name the extent.
    """
    demo = build()
    honest = PolicyFit.of_world(demo.world)
    elsewhere = PolicyFit(
        observation_version=honest.observation_version,
        action_version=honest.action_version,
        observation_length=honest.observation_length,
        action_length=honest.action_length,
        width=honest.width + 1,
        height=honest.height,
        faction_count=honest.faction_count,
    )
    path = write_policy(tmp_path / "elsewhere", demo.world, fit=elsewhere)
    with pytest.raises(PolicyFitError) as refusal:
        seat_policies(demo.world, [(1, path)])
    said = str(refusal.value)
    assert "width" in said, said
    assert str(honest.width) in said, said
    for faction in range(FACTIONS):
        assert not demo.world.is_externally_controlled(faction), (
            "a refused run left a faction out of the hands of the controller"
        )


def test_a_checkpoint_that_names_no_cadence_stops_the_run(tmp_path: Path) -> None:
    """A guessed interval would play a game the policy never learned."""
    demo = build()
    path = write_policy(tmp_path / "silent", demo.world, interval=0)
    with pytest.raises(PolicyChoiceError, match="decision interval"):
        seat_policies(demo.world, [(1, path)])


def test_a_cadence_the_manifest_states_alone_still_places_the_policy(
    tmp_path: Path,
) -> None:
    """A checkpoint may name its cadence in the manifest and not the weights."""
    demo = build()
    path = write_policy(
        tmp_path / "manifested",
        demo.world,
        interval=0,
        manifest={"decision_interval": 3},
    )
    pilots = seat_policies(demo.world, [(1, path)])
    assert pilots[0].interval == 3
    for pilot in pilots:
        pilot.release()


def test_two_cadences_that_disagree_stop_the_run(tmp_path: Path) -> None:
    """A checkpoint that states two intervals is refused, not resolved.

    One value declared in two places with no check is the defect shape this
    project names first, so the check is the point of this test.
    """
    demo = build()
    path = write_policy(
        tmp_path / "split",
        demo.world,
        interval=INTERVAL,
        manifest={"decision_interval": INTERVAL + 1},
    )
    with pytest.raises(PolicyChoiceError, match="two cadences"):
        seat_policies(demo.world, [(1, path)])


def test_two_policies_cannot_hold_one_faction(tmp_path: Path) -> None:
    """A faction holds one policy, and the second is refused by name."""
    demo = build()
    first = write_policy(tmp_path / "first", demo.world)
    second = write_policy(tmp_path / "second", demo.world)
    with pytest.raises(PolicyChoiceError, match="already holds a policy"):
        seat_policies(demo.world, [(1, first), (1, second)])
    for faction in range(FACTIONS):
        assert not demo.world.is_externally_controlled(faction)


def test_the_command_line_reads_a_faction_and_a_file(tmp_path: Path) -> None:
    """The option takes a path, and a leading number names the faction."""
    first = tmp_path / "a.npz"
    second = tmp_path / "b.npz"
    first.write_bytes(b"")
    second.write_bytes(b"")
    assert chosen_policies([str(first)]) == [(None, first)]
    assert chosen_policies([f"2={first}"]) == [(2, first)]
    assert chosen_policies([f"0={first}", f"1={second}"]) == [(0, first), (1, second)]
    with pytest.raises(PolicyChoiceError, match="one faction"):
        chosen_policies([f"1={first}", f"1={second}"])
    with pytest.raises(PolicyChoiceError, match="no weight file"):
        chosen_policies(["3="])


def test_the_run_refuses_a_weight_file_that_is_not_there(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A mistyped path stops the run before it builds a world.

    This drives the command line, which is the way a watcher turns the option
    on. The run must print one sentence and give back a refusal code.
    """
    missing = tmp_path / "nothing.npz"
    assert main(["--policy", str(missing)]) == 2
    said = capsys.readouterr().out
    assert str(missing) in said, said
    assert "seed" not in said, "the run built a world before it read the policy"


def test_the_run_refuses_a_policy_beside_the_headless_mode(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A policy plays through the frames, and the headless mode draws none."""
    held = tmp_path / "weights.npz"
    held.write_bytes(b"")
    assert main(["--policy", str(held), "--run-to-end"]) == 2
    assert "--run-to-end" in capsys.readouterr().out


def test_the_zero_policy_takes_the_no_op(tmp_path: Path) -> None:
    """The untrained policy scores every row at zero and takes row zero.

    This is the control for the choice test above. A pilot that reported an
    action for any weight set would pass that test on an accident.
    """
    demo = build()
    world = demo.world
    zero = LinearPolicy.zeros(
        int(world.action_schema()["length"]),
        int(world.observation_schema()["length"]),
    )
    observation = world.faction_observation(1)
    mask = world.legal_actions(1)
    assert zero.choose(observation, mask) == 0
    assert np.asarray(mask)[0] == 1, "the no-op row is not legal"
