"""The demonstration drives the tick apart from the frame, and names its panels.

The demonstration used to step the engine once for each frame it drew, so a
watcher could not stop the world and could not read one tick.[^1] A clock now
says how many ticks a frame runs.

Nothing here loops over a tile or an entity. Each test drives the control plane
and reads a count the engine gave it.

References
----------
Findings register, FND-322. ``docs/FINDINGS.md``

ADR-0040, Python is a control plane, not a data plane, decision D1.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``
"""

from __future__ import annotations

import pytest

from cachette import World
from cachette.demo.app import Demo
from cachette.demo.clock import SPEEDS, Clock
from cachette.demo.settings import SIZES, Settings

# A world small enough to step many times in a test.
WIDTH = 32
HEIGHT = 32
SEED = 0x0123_4567_89AB_CDEF
FACTIONS = 3


def a_demo() -> Demo:
    """Build a demonstration over a small world."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    return Demo(world, width=320, height=240, threads=1)


def test_a_paused_world_draws_and_does_not_step() -> None:
    """A paused world still draws, so the camera moves and the panel reads."""
    demo = a_demo()
    demo.clock.pause()
    before = demo.world.tick
    reading = demo.advance()
    assert demo.world.tick == before
    assert reading["tick"] == before


def test_one_step_runs_exactly_one_tick_while_paused() -> None:
    """The engine keeps its logs for one tick, so a watcher asks for one."""
    demo = a_demo()
    demo.clock.pause()
    before = demo.world.tick
    demo.clock.step_once()
    demo.advance()
    assert demo.world.tick == before + 1
    # The step is taken, not held. A second frame must run no further tick.
    demo.advance()
    assert demo.world.tick == before + 1


@pytest.mark.parametrize("index", range(len(SPEEDS)))
def test_each_speed_runs_that_many_ticks_in_one_frame(index: int) -> None:
    """A speed is the number of ticks a drawn frame runs."""
    demo = a_demo()
    demo.clock.choose(index)
    before = demo.world.tick
    demo.advance()
    assert demo.world.tick == before + SPEEDS[index]


def test_a_speed_outside_the_set_is_held_to_the_nearest_end() -> None:
    """A keyboard cannot ask for a speed that does not exist."""
    clock = Clock()
    clock.choose(-5)
    assert clock.speed == SPEEDS[0]
    clock.choose(len(SPEEDS) + 5)
    assert clock.speed == SPEEDS[-1]


def test_the_engine_names_the_panels_it_can_draw() -> None:
    """The names come from the viewer's own registration."""
    names = World.panel_names()
    assert names
    assert len(names) == len(set(names)), "a name must select one panel"
    for name in names:
        assert name == name.lower()
        assert " " not in name


def test_a_panel_the_engine_does_not_hold_is_refused() -> None:
    """A frame with nothing on it looks the same as a name that was mistyped."""
    demo = a_demo()
    with pytest.raises(ValueError, match="no panel is called"):
        demo.toggle_panel("no such panel")


def test_a_named_panel_changes_the_frame() -> None:
    """The deck must reach the pixels, not only the selection list."""
    demo = a_demo()
    demo.advance()
    bare = bytes(demo.surface.to_bytes())

    demo.toggle_panel(World.panel_names()[0])
    demo.advance()
    assert bytes(demo.surface.to_bytes()) != bare

    demo.toggle_panel(World.panel_names()[0])
    assert demo.panels == []


def test_the_faction_population_answers_once_for_every_faction() -> None:
    """A faction that dies is invisible until a count of the world says so."""
    demo = a_demo()
    counts = demo.world.faction_population()
    assert len(counts) == FACTIONS
    assert all(count == 0 for count in counts)

    demo.found()
    counts = demo.world.faction_population()
    assert sum(counts) == demo.world.soldier_count
    assert any(count > 0 for count in counts)


def test_every_video_setting_names_a_method_a_window_has() -> None:
    """A control that changes nothing is a capability nobody invokes."""

    class Window:
        """A window that records what the settings asked of it."""

        def __init__(self) -> None:
            self.asked: list[str] = []

        def set_size(self, width: int, height: int) -> None:
            self.asked.append(f"size {width} {height}")

        def set_fullscreen(self, on: bool) -> None:
            self.asked.append(f"fullscreen {on}")

        def set_vsync(self, on: bool) -> None:
            self.asked.append(f"vsync {on}")

    settings = Settings()
    window = Window()
    assert settings.apply_to(window) == []
    width, height = SIZES[settings.video.size_index]
    assert window.asked == [
        f"size {width} {height}",
        "fullscreen False",
        "vsync True",
    ]


def test_a_window_that_cannot_take_a_setting_is_named() -> None:
    """A silent refusal looks the same as a setting that did nothing."""

    class Window:
        """A window that takes a size and nothing else."""

        def set_size(self, width: int, height: int) -> None:
            """Take a size and record nothing."""

    refused = Settings().apply_to(Window())
    assert refused == ["fullscreen", "vertical sync"]
