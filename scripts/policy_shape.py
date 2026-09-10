#!/usr/bin/env python3
"""Print the shape of the structured policy over the world a run trains in.

The document that describes the reinforcement learning architecture states
figures. Every one of those figures is a property of the schema the engine
publishes and of the default widths, so no document may state one from
memory. This probe reads both and prints them.

The engine is deterministic, so one binary gives one answer at any thread
count and on any machine.[^1] Every figure here counts positions or weights,
so it carries from a development machine to the target unchanged. Nothing
here measures the machine.

The world extent, the faction count and the tick limit come from the trainer,
which is the only declaration of the world a run plays.[^2]

References
----------
[^1]: ADR-0001, one binary gives one answer at any thread count.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``

[^2]: The training entry point, the world every strategy plays.
``python/cachette/learn/__main__.py``
"""

from __future__ import annotations

import json
import sys

from cachette._core import World
from cachette.learn.__main__ import WORLD
from cachette.learn.config import TrainConfig
from cachette.learn.layout import ObservationLayout
from cachette.learn.search import (
    generations_before_a_climb_beats_a_wander,
    step_alignment,
)
from cachette.learn.signals import SignalCatalogue
from cachette.learn.structured import (
    POOL_STATISTICS,
    StructuredPolicy,
    StructuredShape,
)


def report(populations: list[int]) -> dict[str, object]:
    """Read the schema and give every figure the architecture document states."""
    world = World(
        width=WORLD.width,
        height=WORLD.height,
        faction_count=WORLD.faction_count,
        seed=1,
    )
    catalogue = SignalCatalogue.of_world(world)
    layout = ObservationLayout.of_catalogue(catalogue)
    actions = int(world.action_schema()["length"])
    shape = StructuredShape()
    policy = StructuredPolicy.zeros(actions, layout, shape)
    verbs = [
        {"name": str(row["name"]), "rows": int(row["rows"])}
        for row in world.action_schema()["verbs"]
    ]
    steps = []
    for population in populations:
        pairs = population // 2
        alignment = step_alignment(pairs, policy.parameter_count)
        steps.append(
            {
                "population": population,
                "pairs": pairs,
                "alignment": alignment,
                "generations_before_a_climb_beats_a_wander": (
                    generations_before_a_climb_beats_a_wander(alignment)
                ),
            }
        )
    return {
        "world": {
            "extent": WORLD.width,
            "factions": WORLD.faction_count,
            "tick_limit": WORLD.tick_limit,
            "decision_interval": WORLD.decision_interval,
            "horizon": WORLD.horizon,
        },
        "observation": {
            "length": layout.length,
            "ring_slots": layout.ring.slots,
            "rings": layout.ring.rings,
            "sectors": layout.ring.sectors,
            "ring_channels": layout.ring.channels,
            "ring_cells": layout.ring.cells,
            "token_sets": [
                {
                    "name": block.name,
                    "tokens": block.tokens,
                    "channels": block.channels,
                    "slots": block.slots,
                }
                for block in layout.tokens
            ],
            "scalar_slots": layout.scalar_slots,
        },
        "actions": {"length": actions, "verbs": verbs},
        "widths": {
            "scalar_width": shape.scalar_width,
            "ring_width": shape.ring_width,
            "ring_bands": shape.ring_bands,
            "sector_kernel": shape.sector_kernel,
            "token_width": shape.token_width,
            "trunk_width": shape.trunk_width,
            "pool_statistics": POOL_STATISTICS,
        },
        "trainable": policy.counts(),
        "trunk_input_width": policy.feature_width,
        "dense_bound": layout.length * actions,
        "steps": steps,
    }


if __name__ == "__main__":
    asked = [int(value) for value in sys.argv[1:]]
    print(json.dumps(report(asked or [TrainConfig().population]), indent=2))
