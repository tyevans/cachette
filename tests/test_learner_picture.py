"""The drawing of an observation covers every position the schema declares.

The test that matters here is the coverage test. A field the engine adds must
appear in the picture, and the only way a reader finds out that it did not is
a check that compares the drawing against the schema. Every other test in this
module holds the geometry and the absence rule, which are the two things a
wrong picture gets wrong while still looking plausible.

The coverage test drives the real caller. It builds a world, reads the
observation the engine publishes, and asks the page what it drew. A test that
built a schema by hand would prove that the drawing covers a schema this file
wrote.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from cachette import World
from cachette.learn.picture import (
    Bar,
    Block,
    Lattice,
    Page,
    Panel,
    RingStack,
    picture_name,
    read_page,
    render,
    ring_stack_of,
    sequence,
)
from cachette.learn.signals import Signal, SignalCatalogue

# The engine states the spatial part of the layout in its own schema. The
# schema marks the ring stack with a space, names its channels, states the
# cells of each ring, states which axis runs first, and names the channel that
# gates a cell. The tests below therefore drive the engine and state none of
# those.
GATE = "area_inside_world"


def a_world(width: int = 24, height: int = 24, factions: int = 3) -> World:
    world = World(width=width, height=height, seed=11, faction_count=factions)
    world.seed_world()
    return world


def a_page(world: World, faction: int = 0, gate: str | None = None) -> Page:
    catalogue = SignalCatalogue.of_world(world)
    observation = np.asarray(world.faction_observation(faction))
    return read_page(catalogue, observation, "a caption", gate)


def an_ungated_page(world: World, faction: int = 0) -> Page:
    """Draw one page from a schema that names no gate.

    The engine names the gate, so this drops that one entry and keeps every
    other. The two pages then differ in the gate alone.
    """
    catalogue = SignalCatalogue.of_world(world)
    geometry = {
        key: value for key, value in catalogue.geometry.items() if key != "spatial_gate"
    }
    bare = SignalCatalogue(list(catalogue), catalogue.observation_length, geometry)
    observation = np.asarray(world.faction_observation(faction))
    return read_page(bare, observation, "a caption")


def test_the_page_draws_every_position_the_schema_declares() -> None:
    world = a_world()
    page = a_page(world)
    drawn = page.drawn()
    assert page.length == int(world.observation_schema()["length"])
    missing = [int(index) for index in np.flatnonzero(~drawn)]
    assert missing == []


def test_the_page_draws_every_position_of_every_declared_field() -> None:
    world = a_world()
    page = a_page(world)
    drawn = page.drawn()
    for row in world.observation_schema()["fields"]:
        start = int(row["start"])
        stop = start + int(row["positions"])
        undrawn = [index for index in range(start, stop) if not bool(drawn[index])]
        assert undrawn == [], f"{row['name']} is not in the picture"


def test_a_field_the_page_forgets_fails_the_coverage_check() -> None:
    """Prove that the coverage check can fail.

    The check reads the drawing of a page. A page built with one field left
    out must fail it, or the check proves nothing about a field the engine
    adds later.
    """
    world = a_world()
    page = a_page(world)
    forgetful = Page(
        caption=page.caption,
        notes=page.notes,
        panels=page.panels[:-1],
        blocks=page.blocks,
        geometry=page.geometry,
        length=page.length,
    )
    assert not bool(forgetful.drawn().all())


def test_the_page_draws_the_lattice_as_a_grid_when_no_field_states_a_space() -> None:
    """A schema that marks no space still draws, through the name prefix.

    The engine marks its spatial field today, so this states a schema of its
    own. The names are a fixture of this test and not fields of the engine,
    because the rule under test is the fallback and not the layout.
    """
    names = ("cell_seen_ever", "cell_units", "held_tiles")
    signals = [
        Signal(name, index * 4, 4 if name.startswith("cell_") else 1)
        for index, name in enumerate(names)
    ]
    catalogue = SignalCatalogue(signals, 9)
    page = read_page(catalogue, np.arange(9, dtype=np.int64), "a caption")
    assert isinstance(page.geometry, Lattice)
    assert page.geometry.cells == 4
    assert len(page.panels) == 2
    assert any("marks no field as spatial" in note for note in page.notes)


def test_a_gate_of_zero_reads_as_absent_and_not_as_zero() -> None:
    """The tool must not shade an empty cell the way it shades a zero.

    The ring frame reaches far past the edge of any world this test builds, so
    the outer cells lie outside the world and hold no value. A reader who sees
    the zero shade there reads a fact the engine never published.
    """
    world = a_world(width=48, height=48)
    gated = a_page(world)
    ungated = an_ungated_page(world)
    assert ungated.panels
    assert all(bool(panel.present.all()) for panel in ungated.panels)
    assert any(not bool(panel.present.all()) for panel in gated.panels)


def test_the_schema_names_the_gate_the_caller_would_have_to_name() -> None:
    """The engine states the gate, so a caller states nothing."""
    world = a_world()
    assert world.observation_schema()["spatial_gate"] == GATE
    stated = a_page(world, gate=GATE)
    declared = a_page(world)
    for first, second in zip(stated.panels, declared.panels, strict=True):
        assert np.array_equal(first.present, second.present)


def test_a_gate_name_the_layout_does_not_hold_names_the_channels() -> None:
    world = a_world()
    with pytest.raises(KeyError, match="names no field and no spatial channel"):
        a_page(world, gate="the_channel_of_a_typo")


def test_a_position_no_field_covers_draws_as_absent() -> None:
    catalogue = SignalCatalogue([Signal("only", 0, 2)], 5)
    page = read_page(catalogue, np.array([3, 4, 0, 0, 0]), "a caption")
    reserve = [block for block in page.blocks if not block.bars[0].present]
    assert len(reserve) == 1
    assert reserve[0].bars[0].positions == (2, 3, 4)
    assert bool(page.drawn().all())


def test_the_ring_geometry_comes_from_the_cell_list_of_the_schema() -> None:
    stack = ring_stack_of({"ring_cells": [1, 6, 12, 12]}, ["stack"])
    assert stack.ring_cells == (1, 6, 12, 12)
    assert stack.cells == 31
    assert stack.place(0) == (0, 0)
    assert stack.place(1) == (1, 0)
    assert stack.place(6) == (1, 5)
    assert stack.place(7) == (2, 0)
    assert stack.place(30) == (3, 11)


def test_the_ring_geometry_comes_from_a_ring_count_and_a_sector_count() -> None:
    stack = ring_stack_of({"rings": 8, "sectors": 12}, ["stack"])
    assert stack.ring_cells == (1, 6, 12, 12, 12, 12, 12, 12)
    assert stack.cells == 79


def test_a_ring_field_with_no_geometry_names_what_the_schema_must_add() -> None:
    with pytest.raises(ValueError, match="ring_cells"):
        ring_stack_of({}, ["ring_stack"])


def test_a_multi_channel_field_with_no_order_names_what_the_schema_must_add() -> None:
    catalogue = SignalCatalogue(
        [
            Signal(
                "stack",
                0,
                62,
                space="ring",
                channels=("fog", "own_ground"),
            )
        ],
        62,
        {"ring_cells": [1, 6, 12, 12]},
    )
    with pytest.raises(ValueError, match="channel_order"):
        read_page(catalogue, np.zeros(62, dtype=np.int64), "a caption")


def test_a_ring_page_draws_one_wedge_for_every_cell_of_every_channel() -> None:
    channels = ("fog", "own_ground", "food")
    stack = RingStack((1, 6, 12, 12))
    signals = [
        Signal(name, index * stack.cells, stack.cells, space="ring")
        for index, name in enumerate(channels)
    ]
    length = len(channels) * stack.cells
    catalogue = SignalCatalogue(signals, length, {"ring_cells": [1, 6, 12, 12]})
    values = np.arange(length, dtype=np.int64)
    page = read_page(catalogue, values, "a ring page")
    assert isinstance(page.geometry, RingStack)
    assert len(page.panels) == len(channels)
    assert bool(page.drawn().all())
    picture = render(page)
    assert picture.count("<path") == len(channels) * (stack.cells - 1)
    assert picture.count("<circle") == len(channels)


def test_a_signed_channel_draws_from_the_two_hue_ramp() -> None:
    panel = Panel(
        title="net",
        values=np.array([-4.0, 0.0, 4.0]),
        present=np.array([True, True, True]),
        positions=np.array([0, 1, 2]),
    )
    assert panel.signed
    assert panel.scale == 4.0


def test_a_channel_of_all_zeros_takes_a_scale_of_one() -> None:
    panel = Panel(
        title="empty",
        values=np.zeros(3),
        present=np.ones(3, dtype=bool),
        positions=np.array([0, 1, 2]),
    )
    assert panel.scale == 1.0
    assert not panel.signed


def test_the_block_of_a_bar_takes_the_scale_of_its_own_block() -> None:
    block = Block(
        name="self",
        bars=(
            Bar("held_tiles", 165.0, True, (0,)),
            Bar("store_total", 39338614.0, True, (1,)),
        ),
    )
    assert block.scale == 39338614.0


def test_the_picture_name_holds_the_world_the_faction_and_the_tick() -> None:
    assert (
        picture_name(48, 48, 3, 7, 1, 250)
        == "obs-48x48-f3-seed7-faction1-tick000250.svg"
    )


def test_the_rendered_picture_is_one_element_tree() -> None:
    world = a_world()
    picture = render(a_page(world))
    assert picture.startswith("<svg")
    assert picture.endswith("</svg>")
    assert "url(#absent)" in picture


def test_a_run_writes_one_picture_for_each_decision(tmp_path: Path) -> None:
    written = sequence(
        directory=tmp_path,
        width=24,
        height=24,
        factions=2,
        seed=3,
        faction=0,
        ticks=6,
        decision_interval=3,
        gate=GATE,
    )
    names = [path.name for path in written]
    assert "obs-24x24-f2-seed3-faction0-tick000000.svg" in names
    assert "obs-24x24-f2-seed3-faction0-sequence.html" in names
    svgs = [name for name in names if name.endswith(".svg")]
    assert len(svgs) >= 2


def test_a_prefix_one_signal_alone_carries_is_not_a_block() -> None:
    """A page of one-bar groups hides the groups that are real.

    A schema that publishes two fields sharing no prefix beside a set of
    fields that share one makes a group for each of the two, and a reader then
    hunts for the real group among them. The names below are a fixture of this
    test and not fields of the engine, because the rule under test is the
    grouping rule and not the layout.
    """
    catalogue = SignalCatalogue(
        [
            Signal("tick_limit", 0, 1),
            Signal("held_tiles", 1, 1),
            Signal("board_good", 2, 2),
            Signal("board_wants", 4, 2),
        ],
        6,
    )
    page = read_page(catalogue, np.arange(6, dtype=np.int64), "a caption")
    names = {block.name for block in page.blocks}
    assert names == {"self", "board"}
    board = next(block for block in page.blocks if block.name == "board")
    assert len(board.bars) == 4


def test_the_schema_wins_over_the_prefix_when_it_names_a_block() -> None:
    catalogue = SignalCatalogue(
        [
            Signal("board_good", 0, 1, block="trade"),
            Signal("board_wants", 1, 1, block="trade"),
        ],
        2,
    )
    page = read_page(catalogue, np.arange(2, dtype=np.int64), "a caption")
    assert [block.name for block in page.blocks] == ["trade"]
