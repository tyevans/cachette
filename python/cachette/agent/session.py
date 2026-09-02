"""The worlds that one agent session holds between tool calls.

An agent builds a world, steps it, and looks at the result. Each of those is
a separate tool call. The store keeps the world between the calls, so the
agent does not rebuild the world to inspect it.

The store holds a name for each world. The name is the only handle an agent
needs, and every tool takes it.

This module holds no simulation logic. It holds the engine's ``World`` and
the settings that built it, because the settings are not readable back from
the world.[^1]

References
----------
[^1]: The compiled module. ``crates/cachette-py/src/lib.rs``
"""

from __future__ import annotations

from dataclasses import dataclass

from cachette._core import World

__all__ = ["SessionStore", "UnknownWorldError", "WorldSession", "WorldSettings"]


class UnknownWorldError(LookupError):
    """The store holds no world under the given name."""


@dataclass(frozen=True)
class WorldSettings:
    """The four values that build a world.

    The engine does not report the seed or the faction count back, so the
    store keeps them. An agent that compares two runs needs the seed.
    """

    width: int
    height: int
    seed: int
    faction_count: int


@dataclass(frozen=True)
class WorldSession:
    """One world and the settings that built it."""

    name: str
    settings: WorldSettings
    world: World


class SessionStore:
    """Holds every world of one server process.

    The store names each world in the order it was built. A name is stable
    for the life of the process, and the store never reuses a name.
    """

    def __init__(self) -> None:
        """Build an empty store."""
        self._sessions: dict[str, WorldSession] = {}
        self._built = 0

    def create(self, settings: WorldSettings) -> WorldSession:
        """Build a world from the settings and keep it under a new name.

        Raises ``ConfigError`` when the extent does not describe a world.
        """
        world = World(
            width=settings.width,
            height=settings.height,
            seed=settings.seed,
            faction_count=settings.faction_count,
        )
        self._built += 1
        name = f"world-{self._built}"
        session = WorldSession(name=name, settings=settings, world=world)
        self._sessions[name] = session
        return session

    def get(self, name: str) -> WorldSession:
        """Return the world under the name.

        Raises ``UnknownWorldError`` when no world holds that name.
        """
        try:
            return self._sessions[name]
        except KeyError:
            known = ", ".join(sorted(self._sessions)) or "none"
            message = f"no world is named {name!r}; the store holds: {known}"
            raise UnknownWorldError(message) from None
