#!/usr/bin/env python3
"""Measure the perturbation size an evolution strategy needs, on real decisions.

**Sigma is not portable and it must not be guessed.** The trainer perturbs a
policy centre and ranks the results, so it needs a sigma large enough that
candidates make different choices. A sigma too small gives every candidate the
same policy, the ranking has nothing to rank, and the search stops without
saying so.

The right value depends on the dimension, and the dependence is not the one
intuition gives. A random direction in several thousand dimensions is nearly
orthogonal to the feature vector, so most of a perturbation does nothing to the
scores. The first run of this trainer used a sigma about six times too small,
chosen by analogy with a gradient method, and it stopped learning part way
through without any output saying so.

This tool answers one question: **what fraction of a policy's real choices does
a perturbation of a given size change?** Aim for a sigma that moves a useful
fraction. A value near zero means the search is dead. A value near one means
the candidates are noise rather than neighbours of the centre.

# Run this again on new hardware and on a new policy shape

The number is a property of the policy shape and the observation, not of the
machine, so a port to another platform does not change it by itself. A change
to the observation schema, to the action table, or to the hidden width does
change it. Re-run this after any of those, and after any change that alters how
many action rows are legal at once.

# Usage

Give it a stored policy and the world that policy plays.

    uv run python scripts/calibrate_sigma.py runs/learn/night2/conquer.npz

The centre is scaled to unit length first, because the trainer holds it there.
A measurement against an unnormalised centre answers a question nobody asks.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import numpy as np

from cachette.learn.env import EnvConfig, VectorEnv, viable_seeds
from cachette.learn.policy import load_policy
from cachette.learn.reward import Weighting
from cachette.learn.train import unit

# The sizes to report. The range spans a perturbation that does nothing and one
# that replaces the policy, so a reader sees where the useful band sits rather
# than one number with no scale beside it.
SIZES = (0.1, 0.25, 0.5, 1.0, 1.5, 2.0, 4.0)

# How many random directions to average over at each size. A single direction
# is one sample of a quantity with real variance.
DIRECTIONS = 8

# A weighting that lets the environment run. **It states no rule of the
# downstream game.** This tool reads choices and never reads the reward, so the
# weights here change nothing it reports.
PROBE_WEIGHTING = Weighting(
    terms={"held_tiles": 1.0}, won=1.0, lost=-1.0, drawn=0.0
)


def collect(
    config: EnvConfig, policy: object, seeds: list[int], workers: int
) -> tuple[np.ndarray, np.ndarray]:
    """Play the policy and return the observations and masks it really met.

    **A random observation array is not a substitute.** The legality mask
    decides before the weights do, and a real decision offers a fraction of
    the action table rather than all of it. A measurement over unmasked
    random inputs answers about a policy nobody runs.
    """
    vector = VectorEnv(config, PROBE_WEIGHTING, count=len(seeds), workers=workers)
    vector.reset(seeds)
    rows: list[np.ndarray] = []
    masks: list[np.ndarray] = []
    while not vector.done:
        observations = np.stack([env.observation() for env in vector.envs])
        mask = vector.action_masks()
        rows.append(observations)
        masks.append(mask)
        vector.step(policy.choose_many(observations, mask))  # type: ignore[attr-defined]
    return np.concatenate(rows), np.concatenate(masks)


def main() -> int:
    """Measure and print the choice change rate at each perturbation size."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("weights", type=Path)
    parser.add_argument("--seeds", type=int, default=6)
    parser.add_argument("--seed-start", type=int, default=50_000)
    # **Nothing here bakes in a worker count.** The knee is a property of the
    # machine, and this project targets a platform the development boxes are
    # not.
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--tick-limit", type=int, default=800)
    parser.add_argument("--decision-interval", type=int, default=10)
    arguments = parser.parse_args()

    policy, meta = load_policy(arguments.weights)
    config = EnvConfig(
        width=int(meta["width"]),
        height=int(meta["height"]),
        faction_count=int(meta["faction_count"]),
        seat=int(meta["seat"]),
        tick_limit=arguments.tick_limit,
        horizon=arguments.tick_limit // arguments.decision_interval,
        decision_interval=arguments.decision_interval,
    )
    seeds = viable_seeds(config, arguments.seeds, arguments.seed_start)
    observations, masks = collect(config, policy, seeds, arguments.workers)

    centre = unit(np.asarray(policy.flat()))
    base = np.array(policy.rebuild(centre).choose_many(observations, masks))
    legal = float(masks.sum(axis=1).mean())
    taken, counts = np.unique(base, return_counts=True)

    print(f"policy      {arguments.weights} ({meta.get('kind')})")
    print(f"parameters  {centre.size}")
    print(f"decisions   {len(observations)} over seeds {seeds}")
    print(f"legal rows  {legal:.1f} of {masks.shape[1]} on average")
    print(f"actions     {dict(zip(taken.tolist(), counts.tolist(), strict=True))}")
    if len(taken) == 1:
        print(
            "            the policy takes one action everywhere, so it reads "
            "nothing from the observation"
        )
    print()
    print("  sigma   choices changed")
    rng = np.random.default_rng(0)
    for size in SIZES:
        changed = []
        for _ in range(DIRECTIONS):
            direction = rng.standard_normal(centre.size)
            direction /= np.linalg.norm(direction)
            moved = policy.rebuild(centre + size * direction)
            changed.append(
                float(
                    np.mean(np.array(moved.choose_many(observations, masks)) != base)
                )
            )
        print(f"  {size:5.2f}   {np.mean(changed):.3f}")
    print()
    print("A sigma whose rate is near zero leaves the search with no spread.")
    print("A sigma whose rate is near one makes every candidate noise.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
