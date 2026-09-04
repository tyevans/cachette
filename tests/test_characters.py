"""Black-box tests of the character and lineage interface.

Every test here starts at the Python boundary. The core held this whole
subsystem before any binding called it, and its own Rust tests passed the
whole time.[^1] A test that built the mechanism again would prove the same
thing again. Each test below drives the installed package.

References
----------
[^1]: Findings register, FND-470. ``docs/FINDINGS.md``

Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette

# The fixed-point scale of every value this project calls Q16.16.
SCALE = 65536

# The role values that the lineage answer carries in its role columns.
MOTHER = 0
FATHER = 1

# The sex values that the character read carries in its sex column.
FEMALE = 0
MALE = 1

# Food is the resource kind that `order_gather` takes. Gathering is what
# raises the deeds of a unit, and deeds are what make a unit eligible.
FOOD = 0

# A run that raises somebody stops as soon as it does. This many steps
# without a promotion is a failure of the fixture, not of the engine.
STEP_CEILING = 40


def _open_addresses(world: cachette.World, wanted: int) -> list[tuple[int, int]]:
    """Return addresses of ground that admits a unit."""
    found: list[tuple[int, int]] = []
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                found.append((q, r))
                if len(found) == wanted:
                    return found
    message = f"the world admits a unit at fewer than {wanted} addresses"
    raise AssertionError(message)


def _gathering_world(seed: int, units: int = 12) -> tuple[cachette.World, np.ndarray]:
    """Build a world whose units gather, and return it with its units."""
    world = cachette.World(width=24, height=24, seed=seed, faction_count=2)
    addresses = _open_addresses(world, units)
    identities = world.spawn_soldiers(addresses, faction=0)
    world.order_gather(identities, FOOD)
    return world, identities


def _run_until_somebody_is_raised(world: cachette.World, threads: int = 2) -> int:
    """Step until the engine raises somebody, and return the step count."""
    for step in range(1, STEP_CEILING + 1):
        world.step(threads=threads)
        if len(world.characters()["character"]) > 0:
            return step
    message = f"the engine raised nobody in {STEP_CEILING} steps"
    raise AssertionError(message)


def _family(world: cachette.World) -> tuple[int, int, int, int]:
    """Make two founders, a child and a grandchild, and return all four."""
    founders = world.create_characters(0, 3)
    mother, father, outsider = (int(one) for one in founders)
    child = int(world.bear_children([(mother, father)])[0])
    grandchild = int(world.bear_children([(child, outsider)])[0])
    return mother, father, child, grandchild


# The engine raises somebody, and the control plane reads them.


def test_a_run_raises_somebody_and_python_reads_their_parents(seed: int) -> None:
    # The whole path, from the boundary: spawn, order, step, read.
    world, _units = _gathering_world(seed)
    world.set_deed_threshold(1)
    world.set_character_schedule(1, 0)
    assert len(world.characters()["character"]) == 0

    _run_until_somebody_is_raised(world)

    people = world.characters()
    assert len(people["character"]) > 0
    raised = int(people["character"][0])

    lineage = world.character_lineage(raised)
    assert lineage["character"] == raised
    # A unit raised from the ranks receives no invented ancestry.
    assert len(lineage["parent"]) == 0
    assert len(lineage["ancestor"]) == 0
    assert len(lineage["descendant"]) == 0


def test_a_raised_unit_stays_a_unit_and_names_its_character(seed: int) -> None:
    world, units = _gathering_world(seed)
    world.set_deed_threshold(1)
    world.set_character_schedule(1, 0)
    _run_until_somebody_is_raised(world)

    named = world.unit_characters(units)
    raised = {int(one) for one in named if one != 0}
    assert raised == {int(one) for one in world.characters()["character"]}

    # The unit is still a unit. It did not become the character, and it still
    # stands on a tile.
    for unit in units:
        world.soldier_tile(int(unit))
    # The character is a character. It answers a lineage read.
    for character in named:
        if character != 0:
            assert world.character_lineage(int(character))["character"] == int(
                character
            )


def test_a_unit_identity_and_a_character_identity_share_one_number(
    seed: int,
) -> None:
    # The two arenas number their slots separately, so one number names a
    # unit in one arena and a person in the other. Nothing reports it. The
    # doc comments say so, and this test pins the sentence.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    addresses = _open_addresses(world, 2)
    units = world.spawn_soldiers(addresses, faction=0)
    people = world.create_characters(0, 2)
    assert units.tolist() == people.tolist()
    # Each call answers for its own arena, and neither refuses the other's
    # number.
    world.soldier_tile(int(people[0]))
    world.character_lineage(int(units[0]))


def test_a_unit_that_was_never_raised_names_no_character(seed: int) -> None:
    world, units = _gathering_world(seed)
    # No step has run, so the engine has raised nobody.
    assert world.unit_characters(units).tolist() == [0] * len(units)


def test_the_deed_threshold_decides_whether_anybody_is_raised(seed: int) -> None:
    # The answer must depend on the threshold, not merely repeat.
    low, _units = _gathering_world(seed)
    low.set_deed_threshold(1)
    low.set_character_schedule(1, 0)
    for _ in range(4):
        low.step(threads=2)

    high, _ = _gathering_world(seed)
    high.set_deed_threshold(2**60)
    high.set_character_schedule(1, 0)
    for _ in range(4):
        high.step(threads=2)

    assert len(low.characters()["character"]) > 0
    assert len(high.characters()["character"]) == 0


def test_the_threshold_reads_back_what_a_caller_wrote(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    before = world.deed_threshold()
    world.set_deed_threshold(before + 7)
    assert world.deed_threshold() == before + 7


def test_deeds_grow_while_a_unit_gathers(seed: int) -> None:
    world, units = _gathering_world(seed)
    start = world.unit_deeds(units)
    assert start.tolist() == [0] * len(units)
    for _ in range(30):
        world.step(threads=2)
    later = world.unit_deeds(units)
    assert later.sum() > 0
    assert all(later >= start)


# The lineage read.


def test_a_child_reads_both_parents_in_one_call(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, father, child, _ = _family(world)

    lineage = world.character_lineage(child)
    assert lineage["parent"].tolist() == [mother, father]
    assert lineage["parent_role"].tolist() == [MOTHER, FATHER]
    assert lineage["parent_alive"].tolist() == [1, 1]


def test_a_descendant_read_answers_with_the_descendants_and_not_the_ancestors(
    seed: int,
) -> None:
    # A swapped walk gives a list of the right type and the wrong content.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, father, child, grandchild = _family(world)

    of_mother = world.character_lineage(mother)
    assert sorted(of_mother["descendant"].tolist()) == sorted([child, grandchild])
    assert of_mother["ancestor"].tolist() == []

    of_grandchild = world.character_lineage(grandchild)
    assert grandchild not in of_grandchild["ancestor"].tolist()
    assert mother in of_grandchild["ancestor"].tolist()
    assert father in of_grandchild["ancestor"].tolist()
    assert child in of_grandchild["ancestor"].tolist()
    assert of_grandchild["descendant"].tolist() == []


def test_a_lineage_group_is_in_ascending_birth_order(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    _, _, _, grandchild = _family(world)
    order = world.character_lineage(grandchild)["ancestor_birth_order"].tolist()
    assert order == sorted(order)


def test_a_character_who_founds_a_line_has_no_parent(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    founder = int(world.create_characters(0, 1)[0])
    lineage = world.character_lineage(founder)
    assert lineage["parent"].tolist() == []
    assert lineage["ancestor"].tolist() == []
    # Its own birth order names its house, because it has no father.
    assert lineage["house"] == lineage["birth_order"]


def test_the_record_of_descent_outlives_a_removed_parent(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, father, child, _ = _family(world)
    world.remove_characters([mother])

    lineage = world.character_lineage(child)
    assert lineage["parent"].tolist() == [mother, father]
    assert lineage["parent_alive"].tolist() == [0, 1]

    # The identity of a removed character is refused everywhere.
    with pytest.raises(cachette.ViewError):
        world.character_lineage(mother)


def test_a_lineage_read_of_an_identity_the_world_does_not_hold_raises(
    seed: int,
) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    founder = int(world.create_characters(0, 1)[0])
    world.remove_characters([founder])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        world.character_lineage(founder)
    assert world.state_hash() == before


# The relation read.


def test_a_parent_and_a_child_are_related_by_one_half(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, father, child, _ = _family(world)
    values = world.character_relations(child, [mother, father])
    assert values.tolist() == [SCALE // 2, SCALE // 2]


def test_a_character_is_wholly_related_to_itself(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    founder = int(world.create_characters(0, 1)[0])
    assert world.character_relations(founder, [founder]).tolist() == [SCALE]


def test_two_founders_are_related_by_nothing(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 2)
    left, right = int(made[0]), int(made[1])
    assert world.character_relations(left, [right]).tolist() == [0]


def test_the_relation_answer_follows_the_order_of_the_set(seed: int) -> None:
    # The value must depend on which member it stands for.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, _, child, _ = _family(world)
    stranger = int(world.create_characters(0, 1)[0])
    forward = world.character_relations(child, [mother, stranger]).tolist()
    backward = world.character_relations(child, [stranger, mother]).tolist()
    assert forward == [SCALE // 2, 0]
    assert backward == [0, SCALE // 2]


def test_a_relation_that_names_a_dead_identity_answers_nothing(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 2)
    live, gone = int(made[0]), int(made[1])
    world.remove_characters([gone])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        world.character_relations(live, [live, gone])
    assert world.state_hash() == before


# The set-valued reads over units.


def test_a_deeds_read_that_names_a_dead_identity_answers_nothing(seed: int) -> None:
    world, units = _gathering_world(seed, units=2)
    world.despawn_soldiers([int(units[1])])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        world.unit_deeds(units)
    assert world.state_hash() == before


def test_a_character_link_read_that_names_a_dead_identity_answers_nothing(
    seed: int,
) -> None:
    world, units = _gathering_world(seed, units=2)
    world.despawn_soldiers([int(units[1])])
    with pytest.raises(cachette.ViewError):
        world.unit_characters(units)


# The writes.


def test_a_caller_makes_people_and_reads_them_back(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 4)
    assert len(made) == 4
    people = world.characters()
    assert people["character"].tolist() == made.tolist()
    assert people["faction"].tolist() == [0, 0, 0, 0]
    assert people["renown"].tolist() == [0, 0, 0, 0]
    assert set(people["sex"].tolist()) <= {FEMALE, MALE}


def test_the_character_read_scopes_to_one_faction(seed: int) -> None:
    # The answer must depend on the faction argument.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    first = world.create_characters(0, 2)
    second = world.create_characters(1, 3)
    assert world.characters(faction=0)["character"].tolist() == first.tolist()
    assert world.characters(faction=1)["character"].tolist() == second.tolist()
    assert len(world.characters()["character"]) == 5
    # A faction the world does not hold has nobody in it, which is an answer.
    assert world.characters(faction=9)["character"].tolist() == []


def test_making_people_in_a_faction_the_world_does_not_hold_makes_nobody(
    seed: int,
) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    before = world.state_hash()
    with pytest.raises(cachette.VerbError):
        world.create_characters(9, 3)
    assert world.state_hash() == before
    assert world.characters()["character"].tolist() == []


def test_bearing_a_child_of_a_dead_parent_bears_nobody(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 3)
    mother, father, gone = (int(one) for one in made)
    world.remove_characters([gone])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        # The first pair is good. The second names a character who is gone.
        world.bear_children([(mother, father), (mother, gone)])
    # Nothing was born, including the child of the good pair.
    assert world.state_hash() == before
    assert world.character_lineage(mother)["descendant"].tolist() == []


def test_a_child_of_one_parent_is_refused(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    founder = int(world.create_characters(0, 1)[0])
    before = world.state_hash()
    with pytest.raises(cachette.VerbError):
        world.bear_children([(founder, founder)])
    assert world.state_hash() == before


def test_a_child_takes_the_faction_of_its_mother(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother = int(world.create_characters(1, 1)[0])
    father = int(world.create_characters(0, 1)[0])
    child = int(world.bear_children([(mother, father)])[0])
    assert child in world.characters(faction=1)["character"].tolist()
    assert child not in world.characters(faction=0)["character"].tolist()


def test_removing_a_set_that_names_a_dead_identity_removes_nobody(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 3)
    world.remove_characters([int(made[2])])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        world.remove_characters([int(made[0]), int(made[2])])
    assert world.state_hash() == before
    assert world.characters()["character"].tolist() == made.tolist()[:2]


def test_a_caller_writes_renown_and_reads_it_back(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 3)
    world.set_character_renown([int(made[0]), int(made[2])], 3 * SCALE)
    assert world.characters()["renown"].tolist() == [3 * SCALE, 0, 3 * SCALE]


def test_a_write_of_zero_renown_is_a_write(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 1)
    world.set_character_renown(made, 5 * SCALE)
    assert world.characters()["renown"].tolist() == [5 * SCALE]
    world.set_character_renown(made, 0)
    assert world.characters()["renown"].tolist() == [0]


def test_renown_takes_a_negative_value(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 1)
    world.set_character_renown(made, -2 * SCALE)
    assert world.characters()["renown"].tolist() == [-2 * SCALE]


def test_writing_renown_to_a_set_that_names_a_dead_identity_writes_nothing(
    seed: int,
) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    made = world.create_characters(0, 2)
    live, gone = int(made[0]), int(made[1])
    world.remove_characters([gone])
    before = world.state_hash()
    with pytest.raises(cachette.ViewError):
        world.set_character_renown([live, gone], 4 * SCALE)
    assert world.state_hash() == before
    assert world.characters()["renown"].tolist() == [0]


def test_a_schedule_of_zero_is_refused(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    with pytest.raises(cachette.VerbError):
        world.set_character_schedule(0, 0)


def test_the_schedule_decides_which_frames_raise_somebody(seed: int) -> None:
    # The answer must depend on the schedule, not merely repeat.
    often, _ = _gathering_world(seed)
    often.set_deed_threshold(1)
    often.set_character_schedule(1, 0)

    seldom, _ = _gathering_world(seed)
    seldom.set_deed_threshold(1)
    seldom.set_character_schedule(1000, 999)

    for _ in range(4):
        often.step(threads=2)
        seldom.step(threads=2)

    assert len(often.characters()["character"]) > 0
    assert len(seldom.characters()["character"]) == 0


# The determinism of the answers.


@pytest.mark.parametrize("threads", [1, 2, 12])
def test_the_character_answers_do_not_depend_on_the_thread_count(
    seed: int, threads: int
) -> None:
    world, _ = _gathering_world(seed)
    world.set_deed_threshold(1)
    world.set_character_schedule(1, 0)
    for _ in range(6):
        world.step(threads=threads)
    people = world.characters()
    lineage = world.character_lineage(int(people["character"][0]))
    reading = (
        people["character"].tolist(),
        people["birth_order"].tolist(),
        people["sex"].tolist(),
        people["house"].tolist(),
        lineage["ancestor"].tolist(),
        lineage["descendant"].tolist(),
        world.state_hash(),
    )
    if threads == 1:
        pytest.character_reading = reading  # type: ignore[attr-defined]
    else:
        assert reading == pytest.character_reading  # type: ignore[attr-defined]


# The numbers that the prose states.


def test_the_documented_numbers_hold(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    mother, father, child, _ = _family(world)

    # A parent and a child give one half, as the raw Q16.16 integer.
    assert world.character_relations(child, [mother]).tolist() == [SCALE // 2]
    # Two children of one pair give one half as well.
    second = int(world.bear_children([(mother, father)])[0])
    assert world.character_relations(child, [second]).tolist() == [SCALE // 2]
    # Zero is female and one is male, and no third value occurs.
    assert set(world.characters()["sex"].tolist()) <= {FEMALE, MALE}
    # Zero is the mother role and one is the father role.
    assert world.character_lineage(child)["parent_role"].tolist() == [MOTHER, FATHER]
    # An ancestor and a descendant hold no role, so the column is zero.
    assert world.character_lineage(child)["ancestor_role"].tolist() == [MOTHER, MOTHER]
