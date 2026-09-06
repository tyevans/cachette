"""Play a seed set of the demonstration world to the full tick limit and report.

The balance harness reports the end of a game.[^1] This script reports the
whole run: the population of each faction over time, the counts of the
subsystems as they rise, and the running value of each faction on each win
path and on the path that no reader watches. It
answers whether the world reaches a win condition, and when.

Every figure comes through the public Python interface, which is the interface
a player uses.[^2]

References
----------
Balance harness. ``python/cachette/balance/__init__.py``

ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``
"""

from __future__ import annotations

import argparse
import json
import multiprocessing
import pathlib
import sys

from cachette import World, stock_target

BASE_SEED = 0x0123_4567_89AB_CDEF
SEED_STRIDE = 0x9E37_79B9_7F4A_7C15
SEED_MASK = (1 << 64) - 1

# The engine states the wealth bar, and this script reads it. A copy here
# reported a share against a bar the engine no longer held.[^1]
#
# [^1]: Findings register, FND-551. ``docs/FINDINGS.md``
STOCK_TARGET_RAW = stock_target()


def seeds_for(count: int) -> list[int]:
    """Derive a seed set from the base seed by the harness rule."""
    return [(BASE_SEED + index * SEED_STRIDE) & SEED_MASK for index in range(count)]


def play(job: tuple[int, int, int, int, int]) -> dict:
    """Play one seed and record a sample every `sample` ticks."""
    seed, extent, factions, limit, sample = job
    world = World(extent, extent, seed=seed, faction_count=factions)
    world.seed_world()
    world.set_tick_limit(limit)
    fell = 0
    samples: list[dict] = []

    def snapshot() -> None:
        standings = [world.standing(f) for f in range(factions)]
        samples.append(
            {
                "tick": world.tick,
                "population": list(world.faction_population()),
                "settlements": world.settlement_count,
                "held": [s["held_tiles"] for s in standings],
                "seats": [s["seats_held"] for s in standings],
                "store": [s["store_total"] for s in standings],
                "wonder": [s["wonder_progress"] for s in standings],
                "fell": fell,
                "census": dict(world.subsystem_census()),
            }
        )

    snapshot()
    end = world.game_end()
    while end is None and world.tick < limit:
        world.step(1)
        fell += world.fell_count
        end = world.game_end()
        if world.tick % sample == 0:
            snapshot()
    if not samples or samples[-1]["tick"] != world.tick:
        snapshot()
    return {
        "seed": seed,
        "tick": world.tick,
        "winner": None if end is None else end["winner"],
        "path": None if end is None else end["path"],
        "fell": fell,
        "samples": samples,
    }


def render(games: list[dict]) -> str:
    """Write one line for each seed."""
    lines = [
        f"{'seed':>18} {'end':>6} {'path':<10} {'fell':>5} {'pop':>16} "
        f"{'sites':>5} {'held':>18} {'wonder':>7} {'wealth':>8}"
    ]
    for game in games:
        last = game["samples"][-1]
        pop = ",".join(str(p) for p in last["population"])
        held = ",".join(str(h) for h in last["held"])
        share = max(last["store"]) * 100 // STOCK_TARGET_RAW
        lines.append(
            f"{game['seed']:#018x} {game['tick']:>6} {game['path'] or '-'!s:<10} "
            f"{game['fell']:>5} {pop:>16} {last['settlements']:>5} {held:>18} "
            f"{max(last['wonder']):>7} {share:>7}%"
        )
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    """Run the sweep and write the JSON report."""
    parser = argparse.ArgumentParser(prog="balance_sweep")
    parser.add_argument("--seeds", type=int, default=24)
    parser.add_argument("--extent", type=int, default=256)
    parser.add_argument("--factions", type=int, default=4)
    parser.add_argument("--tick-limit", type=int, default=20000)
    parser.add_argument("--sample", type=int, default=500)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument(
        "--json", type=pathlib.Path, default=pathlib.Path("target/sweep.json")
    )
    args = parser.parse_args(argv)

    jobs = [
        (seed, args.extent, args.factions, args.tick_limit, args.sample)
        for seed in seeds_for(args.seeds)
    ]
    with multiprocessing.Pool(args.workers) as pool:
        games = pool.map(play, jobs)
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps({"games": games}, sort_keys=True), encoding="utf-8")
    sys.stdout.write(render(games))
    return 0


if __name__ == "__main__":
    sys.exit(main())
