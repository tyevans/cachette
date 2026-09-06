"""The clock states a phase and a speed, and every panel has a key.

The engine moves a unit one whole tile in one tick. At a speed below one tick
for each frame the unit stands still for several frames and then jumps. The
clock now says how far the wall clock is through the current tick, and the
frame draws a unit that moved between its two tiles at that share.

The frame also states the speed as a word beside the tick. The caller sends a
number, and the viewer holds the words, so no text crosses the boundary.[^1]

Nothing here loops over a tile or an entity. Each test drives the control
plane and reads a value the engine gave it.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D5.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

from cachette import World
from cachette.demo.app import LAST_FUNCTION_KEY, SETTINGS_KEYS, Demo, panel_keys
from cachette.demo.clock import SPEEDS, WHOLE_TICK, Clock, says

# A world small enough to step many times in a test.
WIDTH = 32
HEIGHT = 32
SEED = 0x0123_4567_89AB_CDEF
FACTIONS = 3


def a_demo() -> Demo:
    """Build a demonstration over a small world."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    return Demo(world, width=320, height=240, threads=1)


class Keys:
    """A stand-in for the key names the window library holds.

    The tests must not need a window library or a display, so this names the
    function keys and nothing else.
    """

    def __init__(self, highest: int) -> None:
        """Name every function key up to this number."""
        for number in range(1, highest + 1):
            setattr(self, f"F{number}", 1000 + number)


def test_the_clock_yields_the_phase_of_a_half_speed_run() -> None:
    """One tick over two frames leaves the first frame half way through it."""
    clock = Clock()
    clock.choose(SPEEDS.index(WHOLE_TICK // 2))
    assert clock.speed_milli == WHOLE_TICK // 2

    got = []
    for _ in range(4):
        got.append((clock.ticks_due(), clock.phase))
    assert got == [(0, 0.5), (1, 0.0), (0, 0.5), (1, 0.0)]


def test_the_clock_yields_the_phase_of_a_quarter_speed_run() -> None:
    """One tick over four frames passes three part frames on the way."""
    clock = Clock()
    clock.choose(SPEEDS.index(WHOLE_TICK // 4))

    got = [(clock.ticks_due(), clock.phase) for _ in range(4)]
    assert got == [(0, 0.25), (0, 0.5), (0, 0.75), (1, 0.0)]


def test_a_whole_speed_tweens_nothing() -> None:
    """At one tick for each frame, and above, no frame is part way through."""
    for speed in SPEEDS:
        if speed < WHOLE_TICK:
            continue
        clock = Clock()
        clock.choose(SPEEDS.index(speed))
        for _ in range(3):
            assert clock.ticks_due() == speed // WHOLE_TICK
            assert clock.phase == 0.0


def test_a_paused_clock_states_no_speed_and_no_phase() -> None:
    """A paused world is not moving toward a tick, so nothing tweens."""
    clock = Clock()
    clock.choose(SPEEDS.index(WHOLE_TICK // 2))
    clock.ticks_due()
    assert clock.phase == 0.5
    clock.pause()
    assert clock.speed_milli == 0
    assert clock.phase == 0.0
    assert clock.says() == "paused"


def test_the_frame_takes_the_phase_and_the_speed_from_the_clock() -> None:
    """The demonstration passes both on every frame, and the frame reports them."""
    demo = a_demo()
    demo.seed()
    # A seeded world holds a stale spatial structure until a step rebuilds
    # it, and the frame refuses to draw one it cannot trust.
    demo.advance()
    demo.clock.choose(SPEEDS.index(WHOLE_TICK // 2))

    reading = demo.advance()
    assert reading["speed_milli"] == WHOLE_TICK // 2
    assert reading["speed_says"] == "x1/2"

    demo.clock.pause()
    reading = demo.advance()
    assert reading["speed_milli"] == 0
    assert reading["speed_says"] == "paused"


def test_the_word_of_the_clock_and_the_word_of_the_frame_agree() -> None:
    """One speed has one word, and two sites must not disagree.

    The console line is printed before any frame is drawn, so the control
    plane holds the words as well as the viewer. This is the check that fails
    when the copies disagree.
    """
    demo = a_demo()
    demo.seed()
    demo.advance()
    for index, speed in enumerate(SPEEDS):
        demo.clock.choose(index)
        demo.clock.resume()
        reading = demo.advance()
        assert reading["speed_says"] == says(speed) == demo.clock.says()


def test_every_registered_panel_has_a_key() -> None:
    """A panel with no key is a capability nobody can invoke."""
    names = World.panel_names()
    keys = panel_keys(Keys(LAST_FUNCTION_KEY))
    assert len(keys) == len(names), (
        f"the engine registers {len(names)} panels and the mapping gives "
        f"{len(keys)} keys"
    )
    labels = [label for _, label in keys]
    assert len(set(labels)) == len(labels), "one key names one panel"
    for held in SETTINGS_KEYS:
        assert held not in labels, f"{held} already opens a setting"


def test_a_panel_key_puts_its_own_panel_on_the_frame() -> None:
    """The mapping must reach the panel, and not only name a key."""
    from cachette.demo.app import _toggle_panel_key

    demo = a_demo()
    keys = Keys(LAST_FUNCTION_KEY)
    names = World.panel_names()
    for at, (symbol, _) in enumerate(panel_keys(keys)):
        _toggle_panel_key(demo, symbol, keys)
        assert demo.panels == [names[at]]
        _toggle_panel_key(demo, symbol, keys)
        assert demo.panels == []
