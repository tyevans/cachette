"""A stored policy survives a change to the action table, and this checks it.

A weight file used to carry one version integer for the whole action table.
The engine moves that integer whenever it moves a row, and it must, because a
caller decodes a row by arithmetic over the schema. So a change to one verb
retired every row of every stored policy, including the rows that meant what
they had always meant. That happened: a place argument reached the campaign
verb, the verb gained rows, every verb above it moved to a higher index, and
every stored policy of the project was refused and deleted.

A row is now named by its verb and its candidate coordinates, and the loader
carries each row onto the row of the same identity.[^1]

**No weight file of the narrower table survives in the tree.** The tests here
therefore construct one. The construction takes the table the engine publishes
now and puts the campaign verb back to no argument position, which is the
shape the engine published before the place argument. It states no verb name,
no bound and no stride of its own, because a table written by hand is a second
declaration of what the engine publishes.[^2]

References
----------
[^1]: ADR-0200, a stored policy names each row of the action table by its verb
and its candidate coordinates, decisions D1 to D5.
``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``

[^2]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import World
from cachette.learn.layout import ObservationLayout
from cachette.learn.policy import (
    ActionTable,
    LinearPolicy,
    PolicyFit,
    PolicyFitError,
    VerbBlock,
    VerbPosition,
    load_policy,
)
from cachette.learn.signals import SignalCatalogue
from cachette.learn.structured import StructuredPolicy

if TYPE_CHECKING:
    from pathlib import Path

# The world these tests read the action table from. It is small, because every
# assertion here is about the table and never about the play.
WIDTH = 24
HEIGHT = 24
FACTIONS = 3
SEED = 5

# The verb that gained an argument position, and the candidate that position
# holds. The tests name the verb because the change under test is the change
# that reached it. They name no bound and no row of it.
GAINED = "campaign"
PLACE = "place"

# The verb that moved to a higher index and changed no meaning. It sits above
# the verb that gained rows, so its index moved and its one row still means
# what it meant.
MOVED = "advertise"


def a_world() -> World:
    """Build the world these tests read the schemas of."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    world.seed_world()
    return world


def current_table(world: World) -> ActionTable:
    """Copy the action table the engine publishes for one world."""
    return ActionTable.of_schema(world.action_schema())


def relaid(verbs: list[VerbBlock]) -> ActionTable:
    """Lay a list of verb blocks out into a contiguous table.

    The first row of each block follows the last row of the block before it,
    and the row count of a block is the product of the bounds of its
    positions. That is the arithmetic the engine states, and a table built
    here must obey it or the coordinates decode to nothing.
    """
    laid = []
    first = 0
    for verb in verbs:
        rows = 1
        for position in verb.positions:
            rows *= position.bound
        laid.append(VerbBlock(verb.name, first, rows, verb.positions))
        first += rows
    return ActionTable(tuple(laid))


def without_position(table: ActionTable, name: str) -> ActionTable:
    """Return the table one verb held before it took an argument position.

    This is the narrower table the deleted weight files were written against.
    """
    return relaid(
        [
            VerbBlock(verb.name, verb.first, verb.rows, ())
            if verb.name == name
            else verb
            for verb in table.verbs
        ]
    )


def with_wider_bound(table: ActionTable, name: str, extra: int) -> ActionTable:
    """Return the table one verb holds after the bound of a position grows."""
    return relaid(
        [
            VerbBlock(
                verb.name,
                verb.first,
                verb.rows,
                tuple(
                    VerbPosition(
                        position.candidate, position.bound + extra, position.stride
                    )
                    for position in verb.positions
                ),
            )
            if verb.name == name
            else verb
            for verb in table.verbs
        ]
    )


def with_renamed_candidate(
    table: ActionTable, name: str, candidate: str
) -> ActionTable:
    """Return the table one verb holds after its candidate names another thing."""
    return relaid(
        [
            VerbBlock(
                verb.name,
                verb.first,
                verb.rows,
                tuple(
                    VerbPosition(candidate, position.bound, position.stride)
                    for position in verb.positions
                ),
            )
            if verb.name == name
            else verb
            for verb in table.verbs
        ]
    )


def with_extra_position(
    table: ActionTable, name: str, candidate: str, bound: int
) -> ActionTable:
    """Return the table one verb holds after it takes a further position.

    The engine builds the stride of a position from the product of the bounds
    after it, so a position added at the end multiplies every stride before it
    and takes a stride of one itself.
    """
    return relaid(
        [
            VerbBlock(
                verb.name,
                verb.first,
                verb.rows,
                (
                    *(
                        VerbPosition(
                            position.candidate, position.bound, position.stride * bound
                        )
                        for position in verb.positions
                    ),
                    VerbPosition(candidate, bound, 1),
                ),
            )
            if verb.name == name
            else verb
            for verb in table.verbs
        ]
    )


def without_verb(table: ActionTable, name: str) -> ActionTable:
    """Return the table the engine publishes after one verb leaves it."""
    return relaid([verb for verb in table.verbs if verb.name != name])


def fit_of(world: World, table: ActionTable) -> PolicyFit:
    """Build the fit of one world against a stated action table.

    Every entry but the table comes from the schemas the world publishes. The
    two action entries follow the stated table, because a file states the
    table it was written against and the row count of it.
    """
    held = PolicyFit.of_world(world)
    return PolicyFit(
        observation_version=held.observation_version,
        action_version=held.action_version,
        observation_length=held.observation_length,
        action_length=table.length,
        width=held.width,
        height=held.height,
        faction_count=held.faction_count,
        action_table=table,
    )


def ramp(table: ActionTable, features: int = 4) -> np.ndarray:
    """Build a readout whose every row differs from every other row.

    **A uniform readout hides every defect this file tests.** A rebuild that
    put the wrong stored row into a target row would give the same numbers as
    a rebuild that put the right one there.

    **The values rise as a square and not as a straight line.** The mean of an
    odd count of rows of a straight ramp is the middle row of them, so a test
    that asked whether a row took its own donor or the average of its verb
    would read the same answer either way for that row.
    """
    rows = table.length
    steps = np.arange(rows * features, dtype=np.float64) + 1.0
    return (steps * steps).reshape(rows, features)


def stored_linear(
    path: Path, world: World, table: ActionTable, weights: np.ndarray
) -> None:
    """Write a linear weight file against a stated action table."""
    LinearPolicy(weights).save(path, fit_of(world, table).as_meta())


def test_a_file_of_the_narrower_table_keeps_the_weights_of_the_verbs_that_moved(
    tmp_path: Path,
) -> None:
    """A verb that only moved keeps every weight, and the file is not refused.

    This is the case the project paid for. The campaign verb gained rows and
    every verb above it moved to a higher index. Nothing about the moved verbs
    changed meaning, so the loader carries each of their rows onto the row of
    the same identity.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    assert narrow.length < current.length, "the fixture states no narrower table"
    moved_before = narrow.named(MOVED)
    moved_now = current.named(MOVED)
    assert moved_before is not None and moved_now is not None
    assert moved_before.first != moved_now.first, (
        "the fixture does not renumber the table, so it tests nothing"
    )

    weights = ramp(narrow)
    path = tmp_path / "narrow.npz"
    stored_linear(path, world, narrow, weights)

    policy, meta = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    assert policy.weights.shape == (current.length, weights.shape[1])
    np.testing.assert_array_equal(
        policy.weights[moved_now.first], weights[moved_before.first]
    )
    for verb in ("gather", "build", "relation", "queue"):
        held = narrow.named(verb)
        now = current.named(verb)
        assert held is not None and now is not None
        for step in range(held.rows):
            np.testing.assert_array_equal(
                policy.weights[now.first + step], weights[held.first + step]
            )
    assert "rebuilt the readout" in str(meta["action_rebuild"])


def test_a_row_an_argument_added_takes_the_weight_of_the_row_it_narrows(
    tmp_path: Path,
) -> None:
    """A new row is not zero. It takes the whole-frame row it specialises.

    Index zero of a place enumeration names the whole frame, which is the
    answer the verb gave before the position existed. The stored row and the
    new rows are therefore one decision at two resolutions.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    gained_before = narrow.named(GAINED)
    gained_now = current.named(GAINED)
    assert gained_before is not None and gained_now is not None
    assert gained_now.candidates() == (PLACE,)
    assert gained_before.candidates() == ()

    weights = ramp(narrow)
    path = tmp_path / "narrow.npz"
    stored_linear(path, world, narrow, weights)

    policy, _ = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    donor = weights[gained_before.first]
    for step in range(gained_now.rows):
        np.testing.assert_array_equal(policy.weights[gained_now.first + step], donor)
    assert not np.allclose(donor, 0.0), "a zero donor would prove nothing"


def test_the_donor_of_a_new_row_is_its_own_coarse_row_and_not_the_verb_mean(
    tmp_path: Path,
) -> None:
    """The campaign verb cannot separate the donor rule from the mean rule.

    That verb held one row before it took a place, so the mean of its stored
    rows is that one row, and both rules give the same answer. A rule proved
    against it is proved against nothing.

    The gather verb holds one row for each resource kind, so the two rules
    disagree there. Every new row of one resource must take the stored row of
    that resource, and never the average over the kinds.
    """
    world = a_world()
    current = current_table(world)
    wider = with_extra_position(current, "gather", PLACE, bound=3)
    held = current.named("gather")
    now = wider.named("gather")
    assert held is not None and now is not None
    assert held.rows > 1, "a verb of one row cannot separate the two rules"

    weights = ramp(current)
    mean = weights[held.first : held.first + held.rows].mean(axis=0)
    stored = PolicyFit.read(dict(fit_of(world, current).as_meta()))
    assert stored is not None
    rebuild = stored.rebuild_for(fit_of(world, wider))
    assert rebuild is not None
    rebuilt = rebuild.apply(weights)

    for resource in range(held.rows):
        donor = weights[held.first + resource]
        assert not np.allclose(donor, mean), "the fixture cannot tell the rules apart"
        for place in range(3):
            np.testing.assert_array_equal(rebuilt[now.row_of((resource, place))], donor)


def test_a_row_a_grown_bound_added_takes_the_mean_of_its_verb(
    tmp_path: Path,
) -> None:
    """A grown bound leaves rows no stored coordinate reaches.

    No stored row specialises such a row, so it takes the mean of the stored
    rows of its verb. That is the closest statement the file holds of how
    much the verb appeals.
    """
    world = a_world()
    current = current_table(world)
    grown = with_wider_bound(current, "gather", extra=2)
    weights = ramp(current)
    path = tmp_path / "current.npz"
    stored_linear(path, world, current, weights)

    stored = PolicyFit.read(dict(fit_of(world, current).as_meta()))
    assert stored is not None
    rebuild = stored.rebuild_for(fit_of(world, grown))
    assert rebuild is not None
    rebuilt = rebuild.apply(weights)

    held = current.named("gather")
    now = grown.named("gather")
    assert held is not None and now is not None
    assert now.rows > held.rows
    mean = weights[held.first : held.first + held.rows].mean(axis=0)
    for step in range(held.rows):
        np.testing.assert_array_equal(
            rebuilt[now.first + step], weights[held.first + step]
        )
    for step in range(held.rows, now.rows):
        np.testing.assert_allclose(rebuilt[now.first + step], mean)


def test_a_verb_the_file_does_not_name_starts_at_zero(tmp_path: Path) -> None:
    """A new verb gets nothing, because the file states nothing about it."""
    world = a_world()
    current = current_table(world)
    shortened = without_verb(current, MOVED)
    weights = ramp(shortened)
    path = tmp_path / "short.npz"
    stored_linear(path, world, shortened, weights)

    policy, _ = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    fresh = current.named(MOVED)
    assert fresh is not None
    np.testing.assert_array_equal(
        policy.weights[fresh.first], np.zeros(weights.shape[1])
    )


def test_a_verb_the_current_table_does_not_name_drops(tmp_path: Path) -> None:
    """A verb that left the game takes its weights with it, and nothing raises."""
    world = a_world()
    current = current_table(world)
    extra = relaid([*current.verbs, VerbBlock("retired", 0, 1, ())])
    weights = ramp(extra)
    path = tmp_path / "extra.npz"
    stored_linear(path, world, extra, weights)

    policy, meta = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    assert policy.weights.shape[0] == current.length
    assert "retired" in str(meta["action_rebuild"])


def test_the_reader_refuses_a_verb_whose_candidate_names_another_thing(
    tmp_path: Path,
) -> None:
    """A renamed candidate is a different game, and the file is refused.

    The verb keeps its name and its row count, so nothing about the shape of
    the table says that anything changed. Only the candidate says it.
    """
    world = a_world()
    current = current_table(world)
    renamed = with_renamed_candidate(current, "gather", "terrain")
    weights = ramp(renamed)
    path = tmp_path / "renamed.npz"
    stored_linear(path, world, renamed, weights)

    with pytest.raises(PolicyFitError, match="does not begin with"):
        load_policy(path, PolicyFit.of_world(world))


def test_the_reader_refuses_a_verb_that_lost_an_argument_position(
    tmp_path: Path,
) -> None:
    """A removed position leaves most stored identities unnameable.

    The file names a coordinate for each row of the verb, and the current
    table names none. This is the reverse of the case the loader accepts, and
    it is a refusal.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    weights = ramp(current)
    path = tmp_path / "wide.npz"
    stored_linear(path, world, current, weights)

    stored = PolicyFit.read(dict(fit_of(world, current).as_meta()))
    assert stored is not None
    with pytest.raises(PolicyFitError, match="does not begin with"):
        stored.rebuild_for(fit_of(world, narrow))


def test_a_file_the_current_writer_wrote_loads_and_moves_nothing(
    tmp_path: Path,
) -> None:
    """The live run writes files at the table the engine publishes now.

    Those files must keep loading, and they must load verbatim. A reader that
    rebuilt them would move weights for no reason, and the run would resume
    from a centre it never wrote.
    """
    world = a_world()
    current = current_table(world)
    weights = ramp(current)
    path = tmp_path / "live.npz"
    stored_linear(path, world, current, weights)

    policy, meta = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    np.testing.assert_array_equal(policy.weights, weights)
    assert "action_rebuild" not in meta


def test_a_file_whose_table_disagrees_with_its_row_count_is_refused(
    tmp_path: Path,
) -> None:
    """The row count of the table is declared twice, so a check compares them.

    The fit states the row count and the table states the same number a second
    time. A writer that let the two drift would give a reader a table that
    places rows of a readout the file does not hold.
    """
    world = a_world()
    current = current_table(world)
    entries = dict(fit_of(world, current).as_meta())
    entries["action_length"] = current.length + 1
    path = tmp_path / "adrift.npz"
    LinearPolicy(ramp(current)).save(path, entries)

    with pytest.raises(PolicyFitError, match="not consistent with itself"):
        load_policy(path, PolicyFit.of_world(world))


def test_a_file_whose_readout_disagrees_with_its_table_is_refused(
    tmp_path: Path,
) -> None:
    """A readout of another row count was not written against the table.

    The rebuild reads a stored row by the index the stored table gives, so a
    readout of another row count would place weights that mean nothing. The
    check names both numbers rather than raising about an index.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    entries = dict(fit_of(world, narrow).as_meta())
    path = tmp_path / "short.npz"
    LinearPolicy(ramp(narrow)[:-1]).save(path, entries)

    with pytest.raises(PolicyFitError, match="not consistent with itself"):
        load_policy(path, PolicyFit.of_world(world))


def test_a_file_that_states_no_action_table_loads_by_the_version_integer(
    tmp_path: Path,
) -> None:
    """A file written before this table existed still loads.

    Such a file states no identity for any of its rows, so the reader has only
    the version integer and the row count. It compares both and refuses a
    mismatch, which is the rule every file loaded under before.
    """
    world = a_world()
    current = current_table(world)
    weights = ramp(current)
    path = tmp_path / "old.npz"
    plain = {
        key: value
        for key, value in fit_of(world, current).as_meta().items()
        if not key.startswith("action_verb") and not key.startswith("action_position")
    }
    LinearPolicy(weights).save(path, plain)

    policy, meta = load_policy(path, PolicyFit.of_world(world))
    assert isinstance(policy, LinearPolicy)
    np.testing.assert_array_equal(policy.weights, weights)
    assert "action_rebuild" not in meta
    assert "action_verb_names" not in meta


def test_a_file_that_states_no_action_table_is_refused_at_another_version(
    tmp_path: Path,
) -> None:
    """Without a table there is no identity, so the version integer decides."""
    world = a_world()
    current = current_table(world)
    held = PolicyFit.of_world(world)
    plain = {
        key: value
        for key, value in fit_of(world, current).as_meta().items()
        if not key.startswith("action_verb") and not key.startswith("action_position")
    }
    plain["action_version"] = int(held.action_version) + 1
    path = tmp_path / "stale.npz"
    LinearPolicy(ramp(current)).save(path, plain)

    with pytest.raises(PolicyFitError, match="action_version"):
        load_policy(path, PolicyFit.of_world(world))


def test_a_structured_file_of_the_narrower_table_rebuilds_its_readout(
    tmp_path: Path,
) -> None:
    """The structured policy stores its rows in the readout, and only there.

    Every tower and the trunk are functions of the observation, so a change to
    the action table reaches this policy through one layer. The rebuild moves
    that layer and leaves the rest where it was.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    layout = ObservationLayout.of_catalogue(SignalCatalogue.of_world(world))
    policy = StructuredPolicy.zeros(narrow.length, layout)
    policy = policy.with_readout(ramp(narrow, features=policy.trunk_width + 1))
    path = tmp_path / "structured.npz"
    policy.save(path, fit_of(world, narrow).as_meta())

    read, meta = load_policy(path, PolicyFit.of_world(world), layout)
    assert isinstance(read, StructuredPolicy)
    assert read.action_length == current.length
    np.testing.assert_array_equal(read.trunk, policy.trunk)
    moved_before = narrow.named(MOVED)
    moved_now = current.named(MOVED)
    assert moved_before is not None and moved_now is not None
    np.testing.assert_array_equal(
        read.readout[moved_now.first], policy.readout[moved_before.first]
    )
    assert "rebuilt the readout" in str(meta["action_rebuild"])


def test_a_rebuilt_structured_policy_still_scores_every_row(tmp_path: Path) -> None:
    """A rebuilt policy is a whole policy, and the trainer perturbs it.

    The flat vector of the policy carries every layer, and the trainer rebuilds
    a candidate from it. A readout of the wrong row count would give a vector
    the shell refuses, and the run would stop at the first generation.
    """
    world = a_world()
    current = current_table(world)
    narrow = without_position(current, GAINED)
    layout = ObservationLayout.of_catalogue(SignalCatalogue.of_world(world))
    policy = StructuredPolicy.zeros(narrow.length, layout)
    policy = policy.with_readout(ramp(narrow, features=policy.trunk_width + 1))
    path = tmp_path / "structured.npz"
    policy.save(path, fit_of(world, narrow).as_meta())

    read, _ = load_policy(path, PolicyFit.of_world(world), layout)
    assert isinstance(read, StructuredPolicy)
    shell = StructuredPolicy.zeros(current.length, layout)
    again = shell.rebuild(read.flat())
    assert again.action_length == current.length

    observations = np.zeros((2, layout.length), dtype=np.int64)
    masks = np.ones((2, current.length), dtype=np.uint8)
    chosen = again.choose_many(observations, masks)
    assert all(0 <= action < current.length for action in chosen)


def test_a_table_written_in_part_is_refused(tmp_path: Path) -> None:
    """A file that names some table keys and not others cannot be read.

    Every key is written in one call, so a file that holds a few of them was
    not written by this package. Reading the ones it holds would place rows by
    a table nobody stated.
    """
    world = a_world()
    current = current_table(world)
    partial = {
        key: value
        for key, value in fit_of(world, current).as_meta().items()
        if key != "action_position_bounds"
    }
    path = tmp_path / "partial.npz"
    LinearPolicy(ramp(current)).save(path, partial)

    with pytest.raises(PolicyFitError, match="part of an action table"):
        load_policy(path, PolicyFit.of_world(world))


def test_the_table_survives_one_round_trip_through_a_weight_file(
    tmp_path: Path,
) -> None:
    """What the file states is what the engine published, entry for entry.

    The table is stored as flat lists and one list of counts, because the
    archive holds arrays of one shape each. A round trip proves the ragged
    shape comes back.
    """
    world = a_world()
    current = current_table(world)
    path = tmp_path / "round.npz"
    stored_linear(path, world, current, ramp(current))

    _, meta = load_policy(path)
    assert ActionTable.read(meta) == current
