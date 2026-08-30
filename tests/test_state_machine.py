"""The property-based state machine that drives the public API.

ADR-0006 D11 names this the highest-value harness for a stateful engine.
It belongs on the Python side, because the properties that matter are
properties of the boundary.

The machine generates a sequence of commands, applies them, and calls the
invariant check after every rule. The engine exposes that check as a
method for this purpose.

To add a property, add a rule or an invariant to the class below. A rule
issues a command. An invariant states something that must hold after every
rule.

The properties in this file are the ones that the stub engine can support.
Add these when the engine supports them:

- A selector and its negation partition the world, and their counts sum to
  the total.
- A command on an empty selector leaves the state hash unchanged.
- The length of a returned array equals the reported count.
- A scope that raises still closes and still invalidates its views.

References
----------
ADR-0006, The Python boundary, decision D11.
``docs/adrs/draft/adr-0006-python-is-a-control-plane.md``
"""

from __future__ import annotations

from hypothesis import HealthCheck, settings
from hypothesis import strategies as st
from hypothesis.stateful import (
    RuleBasedStateMachine,
    initialize,
    invariant,
    precondition,
    rule,
)

import cachette

# A stateful run builds and steps real worlds, so the deadline is generous
# and the health check that measures fixture time is off.
STATEFUL_SETTINGS = settings(
    max_examples=25,
    stateful_step_count=20,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow],
)


class WorldMachine(RuleBasedStateMachine):
    """Drive one world through a generated sequence of commands."""

    def __init__(self) -> None:
        """Build an empty machine. The initialize rule builds the world."""
        super().__init__()
        self.world: cachette.World | None = None
        self.expected_tick = 0
        self.hashes: list[int] = []

    @initialize(
        tile_count=st.integers(min_value=1, max_value=2048),
        seed=st.integers(min_value=0, max_value=2**64 - 1),
        faction_count=st.integers(min_value=1, max_value=8),
    )
    def build(self, tile_count: int, seed: int, faction_count: int) -> None:
        """Build the world under test."""
        self.world = cachette.World(
            tile_count=tile_count, seed=seed, faction_count=faction_count
        )
        self.expected_tick = 0
        self.hashes = [self.world.state_hash()]

    @precondition(lambda self: self.world is not None)
    @rule(threads=st.integers(min_value=1, max_value=12))
    def step(self, threads: int) -> None:
        """Run one frame."""
        assert self.world is not None
        self.world.step(threads=threads)
        self.expected_tick += 1
        self.hashes.append(self.world.state_hash())

    @precondition(lambda self: self.world is not None)
    @rule()
    def read_the_tile_column(self) -> None:
        """Read the whole tile column and check its length."""
        assert self.world is not None
        values = self.world.tile_values()
        assert len(values) == self.world.tile_count

    @precondition(lambda self: self.world is not None)
    @rule()
    def read_the_event_log(self) -> None:
        """Read the event log and check that it holds whole events."""
        assert self.world is not None
        raw = self.world.event_log_bytes()
        assert len(raw) == self.world.event_count * 24

    @invariant()
    def the_engine_holds_its_invariants(self) -> None:
        """Ask the engine to report its own invariants after every rule."""
        if self.world is None:
            return
        assert self.world.check_invariants()

    @invariant()
    def the_tick_counts_the_steps(self) -> None:
        """Check that the tick equals the number of steps that ran."""
        if self.world is None:
            return
        assert self.world.tick == self.expected_tick

    @invariant()
    def a_read_does_not_change_the_state(self) -> None:
        """Check that a read leaves the state hash unchanged."""
        if self.world is None:
            return
        assert self.world.state_hash() == self.hashes[-1]


TestWorldMachine = WorldMachine.TestCase
TestWorldMachine.settings = STATEFUL_SETTINGS
