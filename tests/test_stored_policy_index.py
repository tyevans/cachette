"""The stored policy index says every file it lists loads, and this checks it.

The index carries one claim about the files beside it: a reader can load each
one and play it. **Nothing checked that claim, and it went false twice.** Four
files stated an observation version the engine had left behind, and the index
said so in prose while keeping them. Two more held a policy kind the project
later deleted, and the loader began refusing them the moment the kind went.

A file name is therefore declared twice, once on disk and once in the index
table. These tests derive one side from the tree and compare, rather than
asking a reader to sweep by hand.[^1]

The world here is the world the index names, because a weight file is a
function of one world and the loader refuses any other.[^2]

References
----------
[^1]: Recurring defect shapes, shape 1 and shape 2.
``.agents/rules/recurring-defects.md``

[^2]: Findings register, FND-686. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import re
import json
from pathlib import Path

import numpy as np
import pytest

from cachette import World
from cachette.learn.policy import PolicyFit, PolicyFitError, load_policy

# Where the stored policies live, found from this file and not from the
# directory a test runner happened to start in.
CHECKPOINTS = Path(__file__).resolve().parent.parent / "checkpoints"

INDEX = CHECKPOINTS / "README.md"

# The world every stored policy was trained against. The index states it in
# the line a reader copies to watch one play.
EXTENT = 48
FACTIONS = 3

# A row of a table in the index whose first cell names a stored file. A row of
# the section on what is gone names a file that must not be on disk, so the
# section heading separates the two rather than the table shape.
ROW = re.compile(r"^\|\s*`([a-z0-9]+/[a-z0-9-]+)`\s*\|", re.MULTILINE)

GONE_HEADING = "## What is gone"

KINDS = ("linear", "structured")
"""The kinds a builder knows.

A stored file naming another kind cannot be played, and the index promises
that every file it lists loads.
"""


def stored_files() -> list[Path]:
    """Return every weight file on disk, in a fixed order."""
    return sorted(CHECKPOINTS.rglob("*.npz"))


def index_names(kept: bool) -> set[str]:
    """Return the file names the index tables hold, above or below the break.

    The kept entry chooses the side. The index lists what a reader can load
    above the heading for what is gone, and the measurement of a removed file
    below it.
    """
    text = INDEX.read_text(encoding="utf-8")
    head, _, tail = text.partition(GONE_HEADING)
    assert tail, f"the index holds no {GONE_HEADING!r} heading"
    return {match.group(1) for match in ROW.finditer(head if kept else tail)}


def a_world() -> World:
    """Build the world the index names, seeded so it publishes its schemas."""
    world = World(width=EXTENT, height=EXTENT, seed=0, faction_count=FACTIONS)
    world.seed_world()
    return world


@pytest.mark.parametrize("path", stored_files(), ids=lambda path: path.stem)
def test_every_stored_policy_loads_against_the_world_the_index_names(
    path: Path,
) -> None:
    """The loader takes each file, which is the claim the index makes.

    This drives the reader a player drives. A test that only read the manifest
    beside a file would pass against a file whose weights the loader cannot
    place, because a manifest is prose that nothing derives.
    """
    policy, meta = load_policy(path, PolicyFit.of_world(a_world()))
    assert policy is not None
    kind = str(meta["kind"])
    assert kind in KINDS, f"{path.name} names the kind {kind!r}, which no builder knows"
    beside = json.loads(path.with_suffix(".json").read_text(encoding="utf-8"))
    assert beside["policy_kind"] == kind, (
        f"{path.name} holds the kind {kind!r} and its manifest says "
        f"{beside['policy_kind']!r}"
    )


def test_the_index_lists_every_file_on_disk_and_no_other() -> None:
    """The table above the break and the tree hold the same names.

    A file the index does not name is a file nobody finds. A name the index
    holds with no file behind it sends a reader looking for nothing.
    """
    on_disk = {f"{path.parent.name}/{path.stem}" for path in stored_files()}
    assert index_names(kept=True) == on_disk


def test_no_removed_policy_is_still_on_disk() -> None:
    """A file the index reports as gone is gone.

    The index keeps the measurement of a removed file so that a reader learns
    what the project tried. That section must never name a file that is still
    beside it, because a reader would then be offered a file the index says
    nothing can run.
    """
    on_disk = {f"{path.parent.name}/{path.stem}" for path in stored_files()}
    assert index_names(kept=False).isdisjoint(on_disk)


def test_every_stored_file_has_a_manifest() -> None:
    """Each weight file carries the manifest the index promises beside it."""
    for path in stored_files():
        manifest = path.with_suffix(".json")
        assert manifest.exists(), f"{path.name} has no manifest beside it"


def test_a_file_naming_a_deleted_kind_is_refused_by_name(tmp_path: Path) -> None:
    """A file of the removed kind says what is wrong with it.

    The project deleted one policy kind. A file written under it holds two
    layers and no single weight matrix, so the reader used to fall through to
    the linear branch and raise an error about a missing archive entry. That
    message named the storage and not the cause, and a reader who met it would
    look for a corrupt file.

    This builds the archive that kind wrote rather than keeping one, because
    the index removed every such file.
    """
    path = tmp_path / "deleted-kind.npz"
    np.savez(
        path,
        kind=np.array("mlp"),
        first=np.zeros((24, 8)),
        second=np.zeros((29, 24)),
    )
    with pytest.raises(PolicyFitError, match="does not build"):
        load_policy(path)
