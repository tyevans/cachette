"""Measure whether the weather separates the map and whether a storm travels.

The research report on the scale of the weather made its readings from scratch
scripts that nobody committed.[^1] This script is the committed replacement. It
reports the same three things, so a reader can compare a run before a change
against a run after it.

The three readings are:

1. **How many cells are wet, over time.** The report found every cell of the
   lattice above the wet mark at every tick of a long run. A weather system
   that still wets every cell has not separated the map.
2. **How far a storm centre travels.** The report found that the maximum of
   the air plane holds where it was raised for two ticks, and then stops
   being a direction at all. A front that a watcher can follow must move.
3. **How long a storm lives.** The report found the whole world back to its
   calm air total within twenty ticks.

Every figure comes through the public Python interface, which is the interface
a player uses.[^2]

Run it with ``uv run python scripts/weather_travel.py``.

References
----------
Research report 26, the scale of the weather, sections 2, 4 and 6.
``docs/research/reports/26-the-scale-of-the-weather.md``

ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``
"""

from __future__ import annotations

import argparse
import json

import numpy

from cachette import World
from cachette.demo.app import build_world

CALM_TICKS = 1000
SAMPLE_EVERY = 100
SETTLE_TICKS = 300
STORM_TICKS = 100
THREADS = 4
STOPS = (1, 2, 3, 5, 10, 20, 40, 60, 100)
# The storm counts as faded when its highest cell holds under this share of
# the water the raise put into the world.
FADED_UNDER_RAISED = 64


def world_for(factions: int, seed: int) -> World:
    """Build the demonstration world, on the default seed when none is named."""
    if seed:
        return build_world(factions=factions, seed=seed)
    return build_world(factions=factions)


def cell_row_and_column(index: int, wide: int) -> tuple[int, int]:
    """Return the row and the column of one cell index."""
    return index // wide, index % wide


def hex_distance(first: tuple[int, int], second: tuple[int, int]) -> int:
    """Return the distance between two cells, in cells.

    The lattice is a hex grid, so the distance is the third-axis form rather
    than the sum of the two differences.
    """
    first_row, first_column = first
    second_row, second_column = second
    down = first_row - second_row
    across = first_column - second_column
    return (abs(down) + abs(across) + abs(down + across)) // 2


def calm_run(factions: int, seed: int, ticks: int) -> dict:
    """Step a calm world and report the spread of the field at each stop."""
    world = world_for(factions, seed)
    world.seed_world()
    samples = []
    for tick in range(ticks):
        world.step(threads=THREADS)
        if (tick + 1) % SAMPLE_EVERY:
            continue
        ground = world.weather_ground()
        air = world.weather_air()
        temperature = world.weather_temperature()
        cells = int(temperature.size)
        wet = int((ground >= world.weather_wet_mark).sum()) if ground.size else 0
        samples.append(
            {
                "tick": world.tick,
                "cells": cells,
                "wet": wet,
                "ground_low": int(ground.min()) if ground.size else 0,
                "ground_median": int(numpy.median(ground)) if ground.size else 0,
                "ground_high": int(ground.max()) if ground.size else 0,
                "air_high": int(air.max()) if air.size else 0,
                "temperature_low": int(temperature.min()),
                "temperature_median": int(numpy.median(temperature)),
                "temperature_high": int(temperature.max()),
            }
        )
    return {"samples": samples}


def storm_run(factions: int, seed: int) -> dict:
    """Raise one storm in one of two equal worlds, and follow the difference.

    **The measurement runs two worlds and subtracts one from the other.** A
    reading that follows the maximum of the air plane alone follows the
    resting maximum of the calm world as soon as the storm sinks below it,
    and it then reports a travel that the storm did not make. The difference
    between the two planes is the water the storm put there, and nothing
    else.
    """
    stormy = world_for(factions, seed)
    calm = world_for(factions, seed)
    stormy.seed_world()
    calm.seed_world()
    for _ in range(SETTLE_TICKS):
        stormy.step(threads=THREADS)
        calm.step(threads=THREADS)

    wide = stormy.weather_cells_wide
    calm_air = int(stormy.weather_totals()["air"])

    # The storm gate refuses a place whose cell holds no ground of the
    # caller's faction, so the script takes a tile that a faction holds.
    holders = stormy.tile_holders()
    held = numpy.flatnonzero(holders != numpy.iinfo(holders.dtype).max)
    if not held.size:
        raise SystemExit("no faction holds a tile, so no storm can be raised")
    middle = int(held[held.size // 2])
    chosen = int(holders[middle])
    place = (int(middle % stormy.width), int(middle // stormy.width))

    report = stormy.inflict_weather(chosen, [place], stormy.weather_strength_ceiling)
    faded_under = report["drops"] // FADED_UNDER_RAISED

    raised_at = None
    stops = []
    lived = 0
    travelled = 0
    for tick in range(1, STORM_TICKS + 1):
        stormy.step(threads=THREADS)
        calm.step(threads=THREADS)
        here = stormy.weather_air().astype(numpy.int64)
        there = calm.weather_air().astype(numpy.int64)
        if not here.size or here.size != there.size:
            continue
        difference = here - there
        peak = int(difference.argmax())
        at = cell_row_and_column(peak, wide)
        if raised_at is None:
            raised_at = at
        away = hex_distance(at, raised_at)
        height = int(difference.max())
        if height > faded_under:
            lived = tick
            travelled = max(travelled, away)
        if tick in STOPS:
            stops.append(
                {
                    "tick_after_storm": tick,
                    "storm_water_left": int(difference[difference > 0].sum()),
                    "highest_cell": height,
                    "row": at[0],
                    "column": at[1],
                    "cells_from_the_start": away,
                }
            )
    return {
        "storm": report,
        "calm_air": calm_air,
        "faded_under": faded_under,
        "raised_over": raised_at,
        "stops": stops,
        "travelled_cells": travelled,
        "lived_ticks": lived,
    }


def main() -> None:
    """Run both measurements and print one JSON document."""
    parser = argparse.ArgumentParser(description="Measure the weather.")
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--factions", type=int, default=4)
    parser.add_argument("--ticks", type=int, default=CALM_TICKS)
    arguments = parser.parse_args()

    report = {
        "calm": calm_run(arguments.factions, arguments.seed, arguments.ticks),
        "travel": storm_run(arguments.factions, arguments.seed),
    }
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
