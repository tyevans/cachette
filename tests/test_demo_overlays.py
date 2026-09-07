"""The watcher switches the map between the overlays the engine registers.

An overlay is one quantity of the world, drawn over every tile of the map as a
strength of one colour. The map carries one at a time, and the watcher chooses
which. The engine holds the list of overlays, and the demonstration takes its
keys from that list, so an overlay that joins the deck gets a key with no edit
to the control plane.

The engine refuses a name it did not publish. A presenter chooses among the
names the one renderer published, so a presenter never draws a layer of its
own.[^1]

Nothing here loops over a tile or an entity. Each test drives the control plane
and reads a value the engine gave it.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D5.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

from string import ascii_uppercase

import pytest

from cachette import FrameError, World
from cachette.demo.app import (
    OVERLAY_OFF_KEY,
    Demo,
    _choose_overlay_key,
    overlay_keys,
)
from cachette.names import Names

# A world small enough to step many times in a test.
WIDTH = 32
HEIGHT = 32
SEED = 0x0123_4567_89AB_CDEF
FACTIONS = 3

# The digit keys a keyboard carries.
DIGIT_KEYS = 10


def a_demo() -> Demo:
    """Build a demonstration over a small world."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    return Demo(world, Names(world.seed), width=320, height=240, threads=1)


class Keys:
    """A stand-in for the key names the window library holds.

    The tests must not need a window library or a display. Importing the real
    key names opens a window, and that fails on a machine with no display, so
    this names the keys itself.

    **This names a whole keyboard, and not the keys the mapping uses today.**
    A stand-in that named only the digits made every overlay past the ninth
    look like an overlay with no key, because the mapping spills onto letters
    once the digits run out. That is a defect of the stand-in and not of the
    mapping.

    The library names a digit with a leading underscore, names a letter by its
    capital, and gives each one its ASCII code. This does the same, so a
    symbol here is the symbol a keyboard sends.
    """

    def __init__(self) -> None:
        """Name every digit key and every letter key."""
        for number in range(DIGIT_KEYS):
            setattr(self, f"_{number}", ord(str(number)))
        for letter in ascii_uppercase:
            setattr(self, letter, ord(letter))

    def number(self, digit: int) -> int:
        """Give back the symbol of one number key."""
        return int(getattr(self, f"_{digit}"))


def test_the_engine_names_the_overlays_it_can_draw() -> None:
    """The list is the registration, and it holds no repeat."""
    names = World.overlay_names()
    assert names, "the engine registers no overlay"
    assert len(set(names)) == len(names), "one name names one overlay"


def test_every_registered_overlay_has_a_key() -> None:
    """An overlay with no key is a capability nobody can invoke."""
    names = World.overlay_names()
    keys = overlay_keys(Keys())
    assert len(keys) == len(names), (
        f"the engine registers {len(names)} overlays and the mapping gives "
        f"{len(keys)} keys"
    )
    labels = [label for _, label in keys]
    assert len(set(labels)) == len(labels), "one key names one overlay"
    assert OVERLAY_OFF_KEY not in labels, (
        f"{OVERLAY_OFF_KEY} turns the overlay off, so no overlay may take it"
    )


def test_an_overlay_key_puts_its_own_overlay_on_the_map() -> None:
    """The mapping must reach the overlay, and not only name a key."""
    demo = a_demo()
    keys = Keys()
    names = World.overlay_names()
    for at, (symbol, _) in enumerate(overlay_keys(keys)):
        _choose_overlay_key(demo, symbol, keys)
        assert demo.overlay == names[at]
    _choose_overlay_key(demo, keys.number(0), keys)
    assert demo.overlay is None


def test_the_frame_reports_the_overlay_it_drew() -> None:
    """A caller reads the scale the picture was drawn at."""
    demo = a_demo()
    demo.seed()
    for name in World.overlay_names():
        demo.choose_overlay(name)
        reading = demo.advance()
        drawn = reading["overlay"]
        assert drawn is not None, f"the frame drew no overlay for {name}"
        assert drawn[0] == name
        assert drawn[2] > drawn[1], f"the {name} overlay declared a span of no width"


def test_a_frame_with_no_overlay_names_none() -> None:
    """A caller cannot read a scale that no picture used."""
    demo = a_demo()
    demo.seed()
    demo.choose_overlay(None)
    assert demo.advance()["overlay"] is None


def test_the_engine_refuses_a_name_it_did_not_publish() -> None:
    """A frame with no wash looks the same as a frame the caller mistyped."""
    demo = a_demo()
    demo.seed()
    with pytest.raises(FrameError) as refused:
        demo.world.draw(
            demo.camera,
            demo.surface.width,
            demo.surface.height,
            demo.surface.pixels,
            overlay="nonsense",
        )
    said = str(refused.value)
    assert "nonsense" in said
    for name in World.overlay_names():
        assert name in said, f"the refusal did not name the {name} overlay"


def test_the_control_plane_refuses_a_name_it_did_not_publish() -> None:
    """The demonstration refuses at the choice rather than at the drawing."""
    demo = a_demo()
    with pytest.raises(ValueError, match="nonsense"):
        demo.choose_overlay("nonsense")
    assert demo.overlay is None
