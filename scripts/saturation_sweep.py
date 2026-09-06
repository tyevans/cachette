"""Play a seed set to the full tick limit and record the whole curve of it.

The committed sweep reports the end of a game and stops when the game ends.[^1]
This script answers a different question: **how much of a run is a subsystem
still moving in?** It therefore runs every seed to the tick limit whatever the
game end says, and it samples every quantity a watcher can read at a fixed
stride.

Every figure comes through the public Python interface, which is the interface
a player uses.[^2]

The companion script turns the samples into one table.[^3]

References
----------
The committed sweep. ``scripts/balance_sweep.py``

ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

The summary. ``scripts/saturation_summary.py``
"""

from __future__ import annotations

import argparse
import json
import multiprocessing
import pathlib
import sys

from cachette import World

BASE_SEED = 0x0123_4567_89AB_CDEF
SEED_STRIDE = 0x9E37_79B9_7F4A_7C15
SEED_MASK = (1 << 64) - 1


def seeds_for(count: int) -> list[int]:
    """Derive a seed set from the base seed by the harness rule."""
    return [(BASE_SEED + index * SEED_STRIDE) & SEED_MASK for index in range(count)]


def read(world: World, factions: int) -> dict[str, int]:
    """Read every scalar a watcher can see, at one tick."""
    standings = [world.standing(f) for f in range(factions)]
    population = list(world.faction_population())
    weather = dict(world.weather_totals())
    row: dict[str, int] = {
        "population_total": sum(population),
        "population_max": max(population),
        "population_min": min(population),
        "settlements": world.settlement_count,
        "held_total": sum(s["held_tiles"] for s in standings),
        "held_max": max(s["held_tiles"] for s in standings),
        "seats_total": sum(s["seats_held"] for s in standings),
        "store_max": max(s["store_total"] for s in standings),
        "store_min": min(s["store_total"] for s in standings),
        "wonder_max": max(s["wonder_progress"] for s in standings),
        "renown_max": max(s["best_renown"] for s in standings),
        "soldiers": world.soldier_count,
        "luxury_tiles": world.luxury_tile_count,
        "world_variety": world.world_variety,
        "weather_air": weather["air"],
        "weather_ground": weather["ground"],
        "weather_wet_cells": weather["wet_cells"],
        "weather_raised": weather["raised"],
        "weather_evaporated": weather["evaporated"],
    }
    relations = [
        world.relation(a, b) for a in range(factions) for b in range(factions) if a != b
    ]
    row["relation_max"] = max(relations)
    row["relation_min"] = min(relations)
    row["relation_spread"] = max(relations) - min(relations)
    for name, value in dict(world.subsystem_census()).items():
        row[f"census_{name}"] = int(value)
    return row


def play(job: tuple[int, int, int, int, int]) -> dict:
    """Play one seed to the tick limit and sample it at a fixed stride."""
    seed, extent, factions, limit, sample = job
    world = World(extent, extent, seed=seed, faction_count=factions)
    world.seed_world()
    world.set_tick_limit(limit)
    samples: list[dict[str, int]] = []
    end_tick: int | None = None
    end_path: str | None = None
    # Four readers count the last tick alone, so this loop adds them up. A
    # per-tick count is a rate and not a stock, and a curve of a rate cannot
    # be read the same way.
    run_total = {"fell": 0, "births": 0, "gathered": 0, "converted": 0}

    def snapshot() -> None:
        row = read(world, factions)
        row["tick"] = world.tick
        row.update(run_total)
        samples.append(row)

    snapshot()
    while world.tick < limit:
        world.step(1)
        run_total["fell"] += world.fell_count
        run_total["births"] += world.births
        run_total["gathered"] += world.gather_count
        run_total["converted"] += world.converted_count
        if end_tick is None:
            end = world.game_end()
            if end is not None:
                end_tick = int(end["tick"])
                end_path = str(end["path"])
        if world.tick % sample == 0:
            snapshot()
    if samples[-1]["tick"] != world.tick:
        snapshot()
    return {
        "seed": seed,
        "ticks": world.tick,
        "end_tick": end_tick,
        "end_path": end_path,
        "samples": samples,
    }


def main(argv: list[str] | None = None) -> int:
    """Run the sweep and write the JSON report."""
    parser = argparse.ArgumentParser(prog="saturation_sweep")
    parser.add_argument("--seeds", type=int, default=16)
    parser.add_argument("--extent", type=int, default=256)
    parser.add_argument("--factions", type=int, default=4)
    parser.add_argument("--tick-limit", type=int, default=20000)
    parser.add_argument("--sample", type=int, default=20)
    parser.add_argument("--workers", type=int, default=16)
    parser.add_argument(
        "--json", type=pathlib.Path, default=pathlib.Path("target/saturation.json")
    )
    args = parser.parse_args(argv)

    jobs = [
        (seed, args.extent, args.factions, args.tick_limit, args.sample)
        for seed in seeds_for(args.seeds)
    ]
    with multiprocessing.Pool(args.workers) as pool:
        runs = pool.map(play, jobs)
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(
        json.dumps(
            {
                "extent": args.extent,
                "factions": args.factions,
                "tick_limit": args.tick_limit,
                "sample": args.sample,
                "runs": runs,
            },
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    for run in runs:
        sys.stdout.write(
            f"{run['seed']:#018x} ticks={run['ticks']} "
            f"end={run['end_tick']} path={run['end_path']}\n"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
