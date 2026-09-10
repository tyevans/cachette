"""A run keeps every generation, and no reader ever sees half of a weight file.

The training launcher copies the weight files of a run off a spot instance
every two minutes, while the trainer writes them.[^1] Two defects lost weights
on that path. The trainer kept only the newest resume point, so each
generation overwrote the one before it. It also saved each file straight onto
its final path, so a copy could take half of an archive and move it over a
whole one.

These tests drive the trainer and the public save of each policy kind, and
then read the files from disk.

References
----------
[^1]: Findings register, FND-746 and FND-760. ``docs/FINDINGS.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, NoReturn

import pytest

from cachette.learn.env import EnvConfig, viable_seeds
from cachette.learn.layout import ObservationLayout, RingBlock, token_blocks
from cachette.learn.picture import RingStack
from cachette.learn.policy import LinearPolicy, load_policy
from cachette.learn.reward import Weighting
from cachette.learn.structured import StructuredPolicy
from cachette.learn.train import TrainConfig, train

if TYPE_CHECKING:
    from collections.abc import Callable
    from pathlib import Path

WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

TRAIN = TrainConfig(
    generations=2, population=4, seeds_per_generation=1, workers=4, seed=0
)


class WriteFailed(RuntimeError):
    """The failure that an unwritable entry raises in the middle of a save."""


class Unwritable:
    """A metadata entry that stops a save after the weights are written.

    numpy writes the entries of an archive in order, and it pickles an entry
    that holds an object. The pickle asks this entry for its state. The entry
    then records the names in the directory and raises. The record shows what
    a fetch that ran at that moment would find.
    """

    def __init__(self, directory: Path) -> None:
        """Watch one directory."""
        self.directory = directory
        self.seen: list[str] = []

    def __reduce__(self) -> NoReturn:
        """Record the directory, then stop the save."""
        self.seen.extend(sorted(path.name for path in self.directory.iterdir()))
        message = "the save stopped half way"
        raise WriteFailed(message)


def a_linear_policy() -> LinearPolicy:
    """Build a small linear policy."""
    return LinearPolicy.zeros(9, 13)


def a_structured_policy() -> StructuredPolicy:
    """Build a small structured policy over a layout that no engine publishes."""
    stack = RingStack((1, 6, 12))
    ring = RingBlock.contiguous(0, stack, 4)
    tokens = token_blocks("tokens", ring.slots, (("first", 3, 5), ("second", 2, 7)))
    length = ring.slots + sum(block.slots for block in tokens) + 11
    layout = ObservationLayout(length=length, ring=ring, tokens=tokens)
    return StructuredPolicy.zeros(9, layout)


def test_a_run_keeps_one_weight_file_for_each_generation(tmp_path: Path) -> None:
    """Each generation leaves its own file, and the newest equals the resume point.

    The resume point holds the newest generation only. A reader who wants the
    centre of an earlier generation reads the file that names it.
    """
    pool = viable_seeds(WORLD, 6, 900)
    train("t", WORLD, WEIGHTING, TRAIN, tmp_path, pool, validation=[])

    kept = sorted(path.name for path in tmp_path.glob("t-gen*.npz"))
    assert kept == [f"t-gen{index:03d}.npz" for index in range(TRAIN.generations)]
    for index, name in enumerate(kept):
        _, meta = load_policy(tmp_path / name)
        assert meta["generation"] == index, f"{name} holds another generation"
    newest = (tmp_path / kept[-1]).read_bytes()
    assert newest == (tmp_path / "t-latest.npz").read_bytes()

    leftover = [path.name for path in tmp_path.iterdir() if ".npz." in path.name]
    assert leftover == [], "the run left a temporary weight file behind"


@pytest.mark.parametrize(
    "build", [a_linear_policy, a_structured_policy], ids=["linear", "structured"]
)
def test_a_save_that_fails_half_way_leaves_the_earlier_file_whole(
    tmp_path: Path, build: Callable[[], LinearPolicy | StructuredPolicy]
) -> None:
    """A save that stops half way changes nothing at the final path.

    The entry that stops the save also records what a fetch would find in the
    middle of the write. A fetch takes the pattern ``*.npz``, so the only
    name that matches it must be the whole file from before.
    """
    path = tmp_path / "t.npz"
    policy = build()
    policy.save(path, {"generation": 1})
    before = path.read_bytes()

    entry = Unwritable(tmp_path)
    with pytest.raises(WriteFailed):
        policy.save(path, {"generation": 2, "broken": entry})

    assert path.read_bytes() == before, "a failed save changed the file"
    _, meta = load_policy(path)
    assert meta["generation"] == 1
    assert len(entry.seen) == 2, (
        f"the save wrote no temporary file beside the final one: {entry.seen}"
    )
    matched = [name for name in entry.seen if name.endswith(".npz")]
    assert matched == ["t.npz"], f"a fetch would take a partial file: {matched}"
    assert sorted(item.name for item in tmp_path.iterdir()) == ["t.npz"], (
        "a failed save left its temporary file behind"
    )
