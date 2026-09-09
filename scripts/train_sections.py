#!/usr/bin/env python3
"""Time each phase of one decision, and say how many cores a process uses.

A training process crosses into the engine once for each decision. Around
that crossing one interpreter does everything else: it stacks the
observations, it builds the action masks, it runs the policy, it sends each
action to its verb, and it reads the reward of each world. **Every engine
worker of the process waits through all of that**, because one interpreter
runs it.

The throughput probe beside this one measures ticks a second.[^1] It cannot
say where the time went. This script says where.

# What the last line means

The last line divides the batch phase by the whole decision and multiplies by
the worker count. That is how many cores the process actually keeps busy. A
process that asks for ten workers and gives four cores is spending the rest of
its decision in the interpreter.

# Why the block count matters

An early tick is cheap and a late tick is dear, because a game grows. A
measurement over the first few decisions therefore overstates the interpreter.
Run enough decisions that the world has developed.

# Why a reuse switch

The loop of a training run takes the observation of the next decision from the
result of this one, because the reward of a decision already read the state
after the ticks. The switch drives the loop the other way, so a reader can
measure what the extra build costs.

# References

[^1]: The throughput probe. `scripts/train_throughput.py`
"""

from __future__ import annotations

import argparse
import time

import numpy as np

from cachette.learn.env import Batch, EnvConfig, VectorEnv, viable_seeds
from cachette.learn.policy import LinearPolicy
from cachette.learn.reward import Weighting

# The scoring the probe runs under. The reward never changes what the engine
# does, so any weighting measures the same phases. This one is the cheapest.
PROBE_WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=0.0, lost=0.0, drawn=0.0)

# The world the learner trains on, so that a figure here answers what a
# training run costs.
WIDTH = 48
HEIGHT = 48
FACTIONS = 3
TICK_LIMIT = 2500

PHASES = ("observe", "mask", "policy", "apply", "batch", "settle")


def parse() -> argparse.Namespace:
    """Read what shape to measure."""
    parser = argparse.ArgumentParser(description="Time each phase of a decision.")
    parser.add_argument("--worlds", type=int, default=64)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--decisions", type=int, default=60)
    parser.add_argument("--interval", type=int, default=10)
    parser.add_argument(
        "--reuse",
        action="store_true",
        help="take the observation from the result of the last decision, "
        "which is what the training loop does",
    )
    return parser.parse_args()


def main() -> None:
    """Run one shape and print one row for each phase."""
    arguments = parse()
    config = EnvConfig(
        width=WIDTH,
        height=HEIGHT,
        faction_count=FACTIONS,
        seat=0,
        tick_limit=TICK_LIMIT,
        horizon=TICK_LIMIT // arguments.interval,
        decision_interval=arguments.interval,
    )
    seeds = viable_seeds(config, arguments.worlds, start=0)
    vector = VectorEnv(
        config, PROBE_WEIGHTING, count=arguments.worlds, workers=arguments.workers
    )
    vector.reset(seeds)
    envs = list(vector.envs)
    policy = LinearPolicy.zeros(envs[0].action_length, envs[0].observation_length)
    batch = Batch([env.world for env in envs], arguments.workers)

    spent = dict.fromkeys(PHASES, 0.0)
    results: list = []
    for _ in range(arguments.decisions):
        mark = time.perf_counter()
        if arguments.reuse and results:
            observations = np.stack([result.observation for result in results])
        else:
            observations = np.stack([env.observation() for env in envs])
        spent["observe"] += time.perf_counter() - mark

        mark = time.perf_counter()
        masks = np.stack([env.action_mask() for env in envs])
        spent["mask"] += time.perf_counter() - mark

        mark = time.perf_counter()
        actions = policy.choose_many(observations, masks)
        spent["policy"] += time.perf_counter() - mark

        mark = time.perf_counter()
        pairs = zip(envs, actions, strict=True)
        applied = [env.apply(int(action)) for env, action in pairs]
        spent["apply"] += time.perf_counter() - mark

        mark = time.perf_counter()
        for _ in range(arguments.interval):
            batch.step(config.threads)
        spent["batch"] += time.perf_counter() - mark

        mark = time.perf_counter()
        settled = zip(envs, applied, strict=True)
        results = [env.settle(answer) for env, answer in settled]
        spent["settle"] += time.perf_counter() - mark

    report(arguments, spent)


def report(arguments: argparse.Namespace, spent: dict[str, float]) -> None:
    """Print one row for each phase, and the cores the process kept busy."""
    whole = sum(spent.values())
    print(f"# worlds\t{arguments.worlds}")
    print(f"# workers\t{arguments.workers}")
    print(f"# decisions\t{arguments.decisions}")
    print(f"# decision_interval\t{arguments.interval}")
    for name in PHASES:
        each = 1000.0 * spent[name] / arguments.decisions
        print(f"  {name:8s} {each:9.2f} ms a decision  {spent[name] / whole:6.1%}")
    interpreter = (whole - spent["batch"]) / whole
    cores = arguments.workers * spent["batch"] / whole
    print(f"  the interpreter holds {interpreter:.1%} of a decision")
    print(f"  {arguments.workers} workers therefore give {cores:.2f} cores")


if __name__ == "__main__":
    main()
