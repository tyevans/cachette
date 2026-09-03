"""The clock that separates the engine tick from the wall clock.

The demonstration used to step the engine once for each frame it drew, so the
two rates were one number. A watcher could not stop the world and could not
run it faster than the screen refreshes.

This module holds the rate and the state. It steps nothing itself. The caller
asks how many ticks this frame owes and runs that many.

**Nothing here names an entity.** The clock counts frames and ticks. It is a
control-plane value, and the engine holds no copy of it.[^1]

References
----------
ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

# The speeds a watcher chooses between, in engine ticks for each drawn frame.
#
# The set is small on purpose. A watcher picks one by number, and a number
# names one speed for the whole run.
SPEEDS: tuple[int, ...] = (1, 2, 4, 8)

# The speed a run opens at, as an index into the set above.
OPENING_SPEED = 0


class Clock:
    """How many engine ticks the demonstration owes each drawn frame.

    The window draws at its own rate. This says how far the world moves
    between two drawings. A paused world still draws, so the camera still
    moves and the panel still reads.
    """

    __slots__ = ("_paused", "_speed", "_stepped")

    def __init__(self, speed: int = OPENING_SPEED, paused: bool = False) -> None:
        """Build a clock at one of the speeds, running or paused."""
        self._speed = self._held(speed)
        self._paused = paused
        # A single step is one tick that outlives the pause. It is cleared as
        # soon as the caller takes it, so one press gives one tick.
        self._stepped = 0

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
        """Give back the ticks each frame runs while the world is not paused."""
        return SPEEDS[self._speed]

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
        """
        self._speed = self._held(speed)

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
        runs the world twice, because the single steps are cleared here.
        """
        due = self._stepped
        self._stepped = 0
        if not self._paused:
            due += self.speed
        return due

    def says(self) -> str:
        """Give back a short line that names the state and the speed."""
        if self._paused:
            return "paused"
        return f"x{self.speed}"
