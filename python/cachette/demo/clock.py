"""The clock that separates the engine tick from the wall clock.

The demonstration used to step the engine once for each frame it drew, so the
two rates were one number. A watcher could not stop the world and could not
run it faster than the screen refreshes.

This module holds the rate and the state. It steps nothing itself. The caller
asks how many ticks this frame owes and runs that many.

**A speed is in thousandths of a tick for each frame.** A speed below one
thousand runs a tick over several frames, so the world moves more slowly than
the screen refreshes. The clock keeps what the frames owe and hands over a
whole tick when the total reaches one.

The share of the current tick that has elapsed is the phase. The caller passes
it to the frame command, and the frame draws a unit that moved between its two
tiles at that share. At one tick for each frame or more the phase is zero,
because nothing is part way through.

**Nothing here names an entity.** The clock counts frames and ticks. It is a
control-plane value, and the engine holds no copy of it.[^1]

References
----------
ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

# One whole tick, in the thousandths the speeds are stated in.
#
# The viewer holds the same scale, because the frame command takes the speed
# as a number in it. A test asserts that the two agree, so neither can move
# alone.
WHOLE_TICK = 1000

# The speeds a watcher chooses between, in thousandths of a tick for each
# drawn frame.
#
# The set is small on purpose. A watcher picks one by number, and a number
# names one speed for the whole run. The two slowest run one tick over four
# frames and over two frames, and the frame tweens the units between the
# tiles while they run.
SPEEDS: tuple[int, ...] = (250, 500, WHOLE_TICK, 2000, 4000, 8000)

# The speed a run opens at, as an index into the set above.
OPENING_SPEED = 2


def says(speed_milli: int) -> str:
    """Give back the word for a speed in thousandths of a tick each frame.

    Zero is ``paused``. A whole number of ticks reads ``x2``. A unit fraction
    reads ``x1/2``.

    **The viewer holds the same words, and the frame states them.** This is
    the console line, which is printed before any frame is drawn. A test
    compares the two, so neither can move alone.
    """
    if speed_milli == 0:
        return "paused"
    if speed_milli % WHOLE_TICK == 0:
        return f"x{speed_milli // WHOLE_TICK}"
    if WHOLE_TICK % speed_milli == 0:
        return f"x1/{WHOLE_TICK // speed_milli}"
    whole, part = divmod(speed_milli, WHOLE_TICK)
    return f"x{whole}.{part:03d}"


class Clock:
    """How many engine ticks the demonstration owes each drawn frame.

    The window draws at its own rate. This says how far the world moves
    between two drawings. A paused world still draws, so the camera still
    moves and the panel still reads.
    """

    __slots__ = ("_owed", "_paused", "_speed", "_stepped")

    def __init__(self, speed: int = OPENING_SPEED, paused: bool = False) -> None:
        """Build a clock at one of the speeds, running or paused."""
        self._speed = self._held(speed)
        self._paused = paused
        # A single step is one tick that outlives the pause. It is cleared as
        # soon as the caller takes it, so one press gives one tick.
        self._stepped = 0
        # What the frames owe toward the next tick, in thousandths. It is
        # always below one whole tick, because a whole tick is handed over as
        # soon as it is owed.
        self._owed = 0

    @staticmethod
    def _held(speed: int) -> int:
        """Give back a speed index inside the set."""
        if speed < 0:
            return 0
        if speed >= len(SPEEDS):
            return len(SPEEDS) - 1
        return speed

    @property
    def paused(self) -> bool:
        """Say whether the world is stopped."""
        return self._paused

    @property
    def speed(self) -> int:
        """Give back the ticks each frame runs, in thousandths of a tick."""
        return SPEEDS[self._speed]

    @property
    def speed_milli(self) -> int:
        """Give back the speed the frame command takes.

        A paused world runs no tick, so its speed is zero. The frame states
        the word for it.
        """
        if self._paused:
            return 0
        return self.speed

    @property
    def phase(self) -> float:
        """Give back the share of the current tick that has elapsed.

        The share is the frames since the last tick divided by the frames each
        tick takes. It is zero at one tick for each frame and above, because
        no frame is then part way through a tick. It is zero while the world
        is paused, because a paused world is not moving toward a tick.
        """
        if self._paused:
            return 0.0
        return self._owed / WHOLE_TICK

    @property
    def speed_index(self) -> int:
        """Give back which of the speeds the clock holds."""
        return self._speed

    def pause(self) -> None:
        """Stop the world. The window keeps drawing."""
        self._paused = True

    def resume(self) -> None:
        """Start the world again at the speed the clock holds."""
        self._paused = False

    def toggle(self) -> None:
        """Stop a running world, or start a stopped one."""
        self._paused = not self._paused

    def choose(self, speed: int) -> None:
        """Choose one of the speeds by its number.

        A number outside the set is held to the nearest end, so a keyboard
        cannot ask for a speed that does not exist.

        The part tick the frames already owe is dropped, so a change of speed
        starts the next tick from a whole frame.
        """
        self._speed = self._held(speed)
        self._owed = 0

    def faster(self) -> None:
        """Choose the next speed up, or keep the fastest."""
        self.choose(self._speed + 1)

    def slower(self) -> None:
        """Choose the next speed down, or keep the slowest."""
        self.choose(self._speed - 1)

    def step_once(self) -> None:
        """Ask for exactly one tick on the next frame.

        The tick runs whether or not the world is paused. This is how a
        watcher reads one tick of the logs, which the engine keeps for one
        tick only.
        """
        self._stepped += 1

    def ticks_due(self) -> int:
        """Give back how many ticks this frame owes, and take the single steps.

        Call this once for each drawn frame. Calling it twice for one frame
        runs the world twice, because the single steps are cleared here and
        the part tick moves on.
        """
        due = self._stepped
        self._stepped = 0
        if not self._paused:
            self._owed += self.speed
            due += self._owed // WHOLE_TICK
            self._owed %= WHOLE_TICK
        return due

    def says(self) -> str:
        """Give back a short line that names the state and the speed."""
        return says(self.speed_milli)
