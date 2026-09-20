"""Every action row encodes and decodes back to its verb and its arguments.

An accepted record says that an action is one integer that indexes a bounded
table the engine declares, that the engine answers which rows are legal at this
tick, and that one log records the choice of the built-in controller and the
action of a learner in one encoding.[^1] [^2] [^3] A second record says that the
integer is a mixed radix over the argument positions each verb declares.[^4]

The tests here verify that the Python bindings expose the schema, action
encoding, action decoding, and legality answers, and that the Python
``ActionTable`` matches the engine's encoding and decoding arithmetic.

References
----------
[^1]: ADR-0154, the observation and the action of a faction are schema-declared
bounded tables the engine owns, decision D4.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^2]: ADR-0154, the observation and the action of a faction are schema-declared
bounded tables the engine owns, decision D5.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^3]: ADR-0154, the observation and the action of a faction are schema-declared
bounded tables the engine owns, decision D6.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^4]: ADR-0176, an action integer is a mixed radix over the argument positions
each verb declares, decisions D1 to D4.
``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import VerbError, World
from cachette.learn.policy import ActionTable


def make_world(factions: int = 3, seed: int = 17) -> World:
    """Build a small world for action table tests."""
    return World(width=24, height=24, faction_count=factions, seed=seed)


def test_action_schema_structure() -> None:
    """The action schema declares a version, a length and a list of verbs."""
    world = make_world(factions=4)
    schema = world.action_schema()
    assert schema["version"] > 0
    assert schema["length"] > 0
    assert len(schema["verbs"]) > 0

    no_op = schema["verbs"][0]
    assert no_op["name"] == "no_op"
    assert no_op["first"] == 0
    assert no_op["rows"] == 1
    assert len(no_op["positions"]) == 0


def test_every_action_encodes_and_decodes_round_trip() -> None:
    """Every action integer decodes and encodes back to the same integer."""
    for factions in [1, 2, 4]:
        world = make_world(factions=factions)
        schema = world.action_schema()
        length = schema["length"]
        for action in range(length):
            verb, arguments = world.decode_action(action)
            assert isinstance(verb, str)
            assert isinstance(arguments, list)
            encoded = world.encode_action(verb, arguments)
            assert encoded == action, (
                f"action {action} decoded to ({verb}, {arguments}) "
                f"but encoded to {encoded}"
            )


def test_encode_and_decode_error_handling() -> None:
    """Decoding past length or encoding with bad arguments raises VerbError."""
    world = make_world(factions=3)
    schema = world.action_schema()
    length = schema["length"]

    with pytest.raises(VerbError, match="at or above the length"):
        world.decode_action(length)

    with pytest.raises(VerbError, match="not a known verb"):
        world.encode_action("not_a_verb", [])

    with pytest.raises(VerbError, match="invalid for the verb"):
        world.encode_action("no_op", [1])

    with pytest.raises(VerbError, match="invalid for the verb"):
        world.encode_action("gather", [])


def test_action_table_agrees_with_world_encoding_and_decoding() -> None:
    """ActionTable methods agree with the engine's encode_action and decode_action."""
    world = make_world(factions=3)
    schema = world.action_schema()
    table = ActionTable.of_schema(schema)
    assert table.length == schema["length"]

    for action in range(table.length):
        engine_verb, engine_args = world.decode_action(action)
        table_verb, table_coords = table.decode(action)
        assert table_verb == engine_verb
        assert list(table_coords) == engine_args

        engine_encoded = world.encode_action(table_verb, list(table_coords))
        table_encoded = table.encode(table_verb, table_coords)
        assert engine_encoded == action
        assert table_encoded == action

    with pytest.raises(ValueError, match="at or above the length"):
        table.decode(table.length)

    with pytest.raises(ValueError, match="not a known verb"):
        table.encode("invalid_verb", ())


def test_legal_actions_and_no_op_legality() -> None:
    """The legal_actions mask covers every action row and marks no-op legal."""
    world = make_world(factions=3)
    schema = world.action_schema()
    length = schema["length"]

    for faction in range(3):
        mask = world.legal_actions(faction)
        assert isinstance(mask, np.ndarray)
        assert mask.shape == (length,)
        assert mask.dtype == np.uint8
        assert int(mask[0]) == 1, "no-op must always be legal"

    with pytest.raises(VerbError, match="names no faction"):
        world.legal_actions(99)

    with pytest.raises(VerbError, match="names no faction"):
        world.act(99, 0)

    with pytest.raises(VerbError, match="at or above the length"):
        world.act(0, length)
