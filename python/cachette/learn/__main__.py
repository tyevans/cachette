"""Train several policies, each against a different weighting, and report.

Each strategy below names what its reward is worth. **The weights are the
strategy.** A policy trained against territory and a policy trained against
people play the same game under different scoring, and the point of the run
is to see whether they end up playing it differently.

Every weight here belongs to this run. None of them states a rule of the
downstream game, and one blocker holds that question open.[^1]

References
----------
[^1]: Blockers register, BLK-050. ``docs/BLOCKERS.md``
"""

from __future__ import annotations

import argparse
import time
from dataclasses import asdict
from pathlib import Path

from .env import EnvConfig, viable_seeds
from .policy import LinearPolicy, RandomPolicy
from .reward import Weighting
from .train import TrainConfig, evaluate, train, write_report

# The world every strategy plays. A side of 48 with three factions resolves
# in a few hundred ticks, and a decision every five ticks keeps the sample
# budget on ticks that changed something.
WORLD = EnvConfig(
    width=48,
    height=48,
    faction_count=3,
    seat=0,
    tick_limit=4000,
    horizon=60,
    decision_interval=5,
)

# A longer horizon, for the strategy that is allowed more time.
LONG_WORLD = EnvConfig(**{**asdict(WORLD), "horizon": 120})

# The store total crosses as a raw Q16.16 integer, so its numbers are about
# five orders of magnitude above a tile count. This weight brings one store
# into the range of one territory.
STORE_SCALE = 1.0e-5

STRATEGIES: dict[str, tuple[EnvConfig, Weighting]] = {
    # Take ground and hold it. Nothing else scores.
    "land": (
        WORLD,
        Weighting(terms={"held_tiles": 1.0}, won=500.0, lost=-500.0, drawn=0.0),
    ),
    # Grow the people. Ground scores a little, because a faction with no
    # ground grows nobody.
    "people": (
        WORLD,
        Weighting(
            terms={"population": 3.0, "held_tiles": 0.25},
            won=500.0,
            lost=-500.0,
            drawn=0.0,
        ),
    ),
    # Raise an army and use it. A loss costs more here than anywhere else.
    "war": (
        WORLD,
        Weighting(
            terms={"live_units": 4.0, "held_tiles": 0.5, "seats_held": 50.0},
            won=1000.0,
            lost=-1000.0,
            drawn=0.0,
        ),
    ),
    # Fill the stores. Ground scores a little, for the same reason.
    "wealth": (
        WORLD,
        Weighting(
            terms={"store_total": STORE_SCALE, "held_tiles": 0.5},
            won=500.0,
            lost=-500.0,
            drawn=0.0,
        ),
    ),
    # The same scoring as the land strategy, over twice the horizon. This
    # varies the time the policy has rather than what it is paid for.
    "land-long": (
        LONG_WORLD,
        Weighting(terms={"held_tiles": 1.0}, won=500.0, lost=-500.0, drawn=0.0),
    ),
}


def main() -> int:
    """Train each named strategy, measure it against the baselines, report."""
    parser = argparse.ArgumentParser(description="Train the learner seat.")
    parser.add_argument("--out", type=Path, default=Path("runs/learn"))
    parser.add_argument("--generations", type=int, default=12)
    parser.add_argument("--population", type=int, default=16)
    parser.add_argument("--seeds", type=int, default=4)
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--holdout", type=int, default=8)
    parser.add_argument("--only", type=str, default="")
    arguments = parser.parse_args()

    names = [n for n in arguments.only.split(",") if n] or list(STRATEGIES)
    out = arguments.out
    out.mkdir(parents=True, exist_ok=True)

    # The training pool and the holdout share no seed, so a reported figure
    # comes from a world the policy never trained on.
    pool = viable_seeds(WORLD, arguments.generations * arguments.seeds + 4, 1000)
    holdout = viable_seeds(WORLD, arguments.holdout, 50_000)
    print(f"training seeds {len(pool)}, holdout seeds {holdout}", flush=True)

    report: dict[str, object] = {
        "holdout": holdout,
        "generations": arguments.generations,
        "population": arguments.population,
        "seeds_per_generation": arguments.seeds,
        "strategies": {},
    }
    started = time.time()

    for index, name in enumerate(names):
        env_config, weighting = STRATEGIES[name]
        print(f"\n=== {name} ===", flush=True)
        train_config = TrainConfig(
            generations=arguments.generations,
            population=arguments.population,
            seeds_per_generation=arguments.seeds,
            workers=arguments.workers,
            seed=index,
        )
        result = train(name, env_config, weighting, train_config, out, pool)
        trained, _ = LinearPolicy.load(Path(result["weights"]))  # type: ignore[arg-type]
        untrained = LinearPolicy.zeros(trained.shape[0], trained.shape[1] - 1)
        measured = {
            "trained": evaluate(
                env_config, weighting, trained, holdout, arguments.workers
            ),
            "untrained": evaluate(
                env_config, weighting, untrained, holdout, arguments.workers
            ),
            "random": evaluate(
                env_config,
                weighting,
                RandomPolicy(seed=index),
                holdout,
                arguments.workers,
            ),
        }
        result["holdout"] = measured
        report["strategies"][name] = result  # type: ignore[index]
        for label in ("trained", "untrained", "random"):
            print(f"  {label:9s} {measured[label]}", flush=True)
        write_report(out / "report.json", report)

    report["seconds"] = round(time.time() - started, 1)
    write_report(out / "report.json", report)
    print(f"\nwrote {out / 'report.json'} in {report['seconds']}s", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
