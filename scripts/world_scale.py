#!/usr/bin/env python3
"""Measure what a larger training world buys, and what it costs.

Every reinforcement learning run this project has done trains on a world of
extent 48 by 48 with three factions. This probe measures the quantities a
decision to train on a larger world needs, over a range of extents.

**This probe runs on a development machine.** The development machines are
x86-64 and Apple Silicon. The engine targets AWS Graviton, which has a
64-byte cache line where Apple Silicon has a 128-byte line, so a local
measurement misleads on false sharing and on alignment.[^1] The throughput
rows below are therefore local figures and are not target figures. The
register of target costs holds the target rows, and it says which figures
are measured and which are derived.[^2] [^3]

# Two of the four sections measure the engine, not the machine

The ring section and the episode section read the engine. The engine is
deterministic, so one binary gives one answer at any thread count and on any
machine.[^4] A ring count and an end tick are therefore facts about the
world and the play, and they carry from a development machine to the target
unchanged.

The throughput section and the generation section read the machine. Neither
carries.

# What each section answers

**Rings.** The observation of a faction is a fixed-width table over an
egocentric log-polar frame. The ring index of a tile is the bit length of its
hex distance from the centre, so the frame reaches distance 8191 over 14
rings. A world that is too small leaves the outer rings empty, and a policy
then has no input where the frame states map-scale strategy. This section
reports the rings the frame can reach in a world of each extent, and the
rings a played episode actually filled.

**Episodes.** The point of a larger world is that a faction has room to grow
into it. A larger world that still yields one settlement has bought nothing.
This section plays whole episodes and reads the settlement count and the held
tile count as the episode runs, so a reader sees whether a faction grew.

**End ticks.** This section plays under a generous tick limit rather than the
limit a training run uses, so the end tick it reads is the tick the game
resolved at and not the tick a limit cut it off at. The quantiles of that
distribution are what a tick limit should be set from.

**Throughput.** One tick is one step of one world. The batch steps every
world that is still running, so one crossing of the boundary advances the
decision interval in each live world. The tick counting lives in the
throughput probe, which this module calls rather than counting again.[^5]

# The measured shape must be the shape a training run uses

An earlier probe gave each worker one world. **The trainer never runs that
shape.** A generation holds one world for every pair of a candidate and a
seed, so a generation of twenty-four candidates over six seeds holds one
hundred and forty-four worlds whatever the worker count is. A figure taken at
one world for each worker describes the probe. This probe therefore takes the
world count as an argument and defaults it to the trainer's product.

# Two arms bracket the tick limit

The controller arm gives every seat to the built-in controller. The idle arm
takes the learner seat and plays the no-op at every decision, which is what
an untrained policy of the first generation does. A game with a passive seat
resolves at a different tick from a game with none, and a tick limit has to
cover both, so the probe plays both and reports each.

# References

[^1]: Project orientation, the target platform. ``CLAUDE.md``
[^2]: Target platform costs. ``docs/reference/graviton-costs.md``
[^3]: Blockers register, BLK-007. ``docs/BLOCKERS.md``
[^4]: ADR-0001, one binary gives one answer at any thread count.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
[^5]: The learner throughput probe. ``scripts/train_throughput.py``
[^6]: Reinforcement learning parameters. ``docs/reference/rl-costs.md``
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import sys
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import numpy as np

from cachette.learn.env import Env, EnvConfig, VectorEnv, viable_seeds
from cachette.learn.record import end_tick_of
from cachette.learn.reward import Weighting

sys.path.insert(0, str(Path(__file__).resolve().parent))

from train_throughput import Measurement, measure

# The extents the probe sweeps by default. The lowest is the world every run
# so far has trained on, and it is here so that every other row has something
# to be compared with.
DEFAULT_EXTENTS = (48, 96, 128, 192, 256)

# The tick limit the episode section plays under. It is far above the limit a
# training run uses, because this section measures the tick a game resolves
# at. A limit that cuts a game off replaces that measurement with itself.
DEFAULT_TICK_CEILING = 12_000

# How many ticks one decision covers, matching the trainer's default.
DEFAULT_DECISION_INTERVAL = 10

# The world count of the throughput shape. The trainer's default population
# is twenty-four candidates over six seeds, and it puts the whole product in
# one batch.
DEFAULT_THROUGHPUT_WORLDS = 144

# How many crossings of the boundary the throughput section times.
DEFAULT_THROUGHPUT_DECISIONS = 20

# The scoring every section runs under. The reward never changes what the
# engine does and never ends an episode, so any weighting reads the same
# ticks and the same settlements.
PROBE_WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=0.0, lost=0.0, drawn=0.0)

# The two signals the episode section reads. The engine owns both names, and
# the catalogue of the world resolves them, so this probe states no position.
SETTLEMENT_SIGNAL = "settlements"
GROUND_SIGNAL = "held_tiles"

# The name of the arm that gives every seat to the built-in controller, and
# the name of the arm that holds the learner seat and plays the no-op.
CONTROLLER_ARM = "controller"
IDLE_ARM = "idle"

# The settlement counts the episode section reports a reaching tick for. A
# faction that never passes one has no room, and that is the reading this
# whole probe exists to take.
SETTLEMENT_MARKS = (2, 3, 4)

# The sections the probe runs. A caller names a subset, and a name outside
# this set is a typo that would otherwise run nothing and report nothing.
SECTIONS = frozenset({"rings", "episodes", "throughput"})

# The quantile the tick limit recommendation reads. A limit at this quantile
# lets nine games in ten resolve on their own terms, and truncates the tenth.
LIMIT_QUANTILE = 0.9

# What the recommendation rounds the limit up to. A round figure is easier to
# carry between a register row, a launcher and a command line than a quantile
# to the tick.
LIMIT_STEP = 500


def ring_bound(width: int, height: int) -> int:
    """Return how many rings of the frame a world of this extent can fill.

    The world is a rhombus of ``width`` columns by ``height`` rows in axial
    addresses, so the greatest hex distance between two of its tiles is
    ``width`` plus ``height`` less two. The ring index of a tile is the bit
    length of its hex distance from the centre, so the highest index a world
    can reach is the bit length of that greatest distance, and the ring count
    is one more than the index because ring zero is the centre.

    **Each doubling of the extent buys exactly one ring.** The bit length of
    a doubled distance is one greater, so the frame gains one ring for each
    doubling and no more. That is the whole of the relation between the world
    size and the spatial input of a policy.
    """
    if width < 1 or height < 1:
        message = "a world holds at least one column and one row"
        raise ValueError(message)
    return max(width + height - 2, 0).bit_length() + 1


@dataclass(frozen=True)
class RingReading:
    """What one extent reached in the ring frame.

    The bound entry is what the extent can fill, from the geometry alone. The
    reached entry is what a played episode filled, read from the channel that
    says how much of a cell lies inside the world. The two differ because the
    frame is egocentric: a faction near the middle of the world reaches less
    far than a faction in a corner.
    """

    extent: int
    ring_count: int
    bound: int
    reached_at_start: int
    reached_at_end: int

    def as_row(self) -> dict[str, object]:
        """Return the reading as one mapping, for the JSON output."""
        return {
            "extent": self.extent,
            "ring_count": self.ring_count,
            "bound": self.bound,
            "reached_at_start": self.reached_at_start,
            "reached_at_end": self.reached_at_end,
        }


def ring_cell_counts(env: Env) -> list[int]:
    """Return how many cells each ring of the frame holds.

    The schema of the world publishes the list, so no reader of an
    observation states a cell count of its own.
    """
    counts = env.signals.geometry["ring_cells"]
    return [int(count) for count in cast("Sequence[int]", counts)]


def rings_filled(env: Env) -> int:
    """Return how many rings of one observation hold ground inside the world.

    The schema names the channel that gates the rest of a spatial cell, and
    it names how many cells each ring holds, so this reads the layout and
    states no position of its own. A ring holds ground when any cell of it
    reads above zero in the gate channel.
    """
    signal = env.signals.signal("ring_stack")
    channels = list(signal.channels)
    gate = str(env.signals.geometry["spatial_gate"])
    if gate not in channels:
        message = f"the ring stack holds no {gate!r} channel"
        raise RuntimeError(message)
    observation = np.asarray(env.observation())
    window = observation[signal.start : signal.start + signal.positions]
    grid = window.reshape(-1, len(channels))[:, channels.index(gate)]
    counts = ring_cell_counts(env)
    filled = 0
    at = 0
    for count in counts:
        band = grid[at : at + count]
        at += count
        if bool((band > 0).any()):
            filled += 1
    return filled


@dataclass(frozen=True)
class EpisodeReading:
    """What one played episode ended at, and how it grew on the way.

    The settlement entries hold the count at the end and the highest count
    the episode reached. The reaching entries hold the tick at which the
    settlement count first passed each mark, and an absent mark holds no
    entry.
    """

    arm: str
    extent: int
    seed: int
    outcome: str
    end_tick: int
    reached_ceiling: bool
    settlements_at_end: int
    settlements_at_most: int
    held_tiles_at_end: int
    reaching_ticks: dict[int, int]
    rings_at_end: int

    def as_row(self) -> dict[str, object]:
        """Return the reading as one mapping, for the JSON output."""
        return {
            "arm": self.arm,
            "extent": self.extent,
            "seed": self.seed,
            "outcome": self.outcome,
            "end_tick": self.end_tick,
            "reached_ceiling": self.reached_ceiling,
            "settlements_at_end": self.settlements_at_end,
            "settlements_at_most": self.settlements_at_most,
            "held_tiles_at_end": self.held_tiles_at_end,
            "reaching_ticks": {
                str(mark): tick for mark, tick in self.reaching_ticks.items()
            },
            "rings_at_end": self.rings_at_end,
        }


def world_of(
    extent: int, factions: int, tick_limit: int, interval: int, controlled: bool
) -> EnvConfig:
    """Return the world of one extent, with the horizon derived once.

    **The horizon is derived here and nowhere else in this module.** A
    horizon shorter than the tick limit ends an episode before the game ends,
    and a truncated episode reports no outcome, so the two cannot be allowed
    to disagree.
    """
    return EnvConfig(
        width=extent,
        height=extent,
        faction_count=factions,
        seat=0,
        tick_limit=tick_limit,
        horizon=tick_limit // interval,
        decision_interval=interval,
        controlled=controlled,
    )


def read_count(env: Env, observation: np.ndarray, name: str) -> int:
    """Return one published count of an observation as the quantity behind it.

    The engine compresses a count before it publishes one, and the schema
    publishes the rule, so the catalogue inverts it. A caller that read the
    published value would read a logarithm and call it a settlement count.
    """
    signal = env.signals.signal(name)
    return round(float(signal.quantities(observation)[0]))


def play_arm(
    arm: str,
    extent: int,
    factions: int,
    ceiling: int,
    interval: int,
    seeds: Sequence[int],
    workers: int,
) -> list[EpisodeReading]:
    """Play one arm over the seed set, and read what each episode did.

    The controller arm gives every seat to the built-in controller, so no
    action reaches the world. The idle arm holds the learner seat and plays
    the no-op, which is action zero and is legal in every world. Both arms
    step the same batch, so a difference between them comes from the seat and
    from nothing else.

    **The batch drops an episode that has ended.** The probe therefore reads
    the growth of an episode from the results of the crossings the episode
    was still live for, and reads its end from the game end record of its own
    world.
    """
    config = world_of(extent, factions, ceiling, interval, arm == IDLE_ARM)
    vector = VectorEnv(config, PROBE_WEIGHTING, count=len(seeds), workers=workers)
    observations = vector.reset(list(seeds))
    highest = [0] * len(seeds)
    reaching: list[dict[int, int]] = [{} for _ in seeds]
    for index, env in enumerate(vector.envs):
        count = read_count(env, observations[index], SETTLEMENT_SIGNAL)
        highest[index] = count
        for mark in SETTLEMENT_MARKS:
            if count >= mark and mark not in reaching[index]:
                reaching[index][mark] = 0
    while not vector.done:
        results = vector.step([0] * len(seeds))
        for index, (env, result) in enumerate(zip(vector.envs, results, strict=True)):
            observation = np.asarray(result.observation)
            count = read_count(env, observation, SETTLEMENT_SIGNAL)
            highest[index] = max(highest[index], count)
            for mark in SETTLEMENT_MARKS:
                if count >= mark and mark not in reaching[index]:
                    reaching[index][mark] = int(env.world.tick)
        observations = np.stack([result.observation for result in results])
    readings = []
    for index, (seed, env) in enumerate(zip(seeds, vector.envs, strict=True)):
        observation = np.asarray(env.observation())
        end = end_tick_of(env.world)
        readings.append(
            EpisodeReading(
                arm=arm,
                extent=extent,
                seed=int(seed),
                outcome=env.outcome,
                end_tick=end,
                reached_ceiling=end >= ceiling,
                settlements_at_end=read_count(env, observation, SETTLEMENT_SIGNAL),
                settlements_at_most=highest[index],
                held_tiles_at_end=read_count(env, observation, GROUND_SIGNAL),
                reaching_ticks=dict(sorted(reaching[index].items())),
                rings_at_end=rings_filled(env),
            )
        )
    return readings


def quantile(values: Sequence[float], share: float) -> float:
    """Return one quantile of a sample, or zero for an empty sample."""
    if not values:
        return 0.0
    return float(np.quantile(np.asarray(values, dtype=float), share))


def recommended_limit(end_ticks: Sequence[int]) -> int:
    """Return the tick limit that lets the stated share of games resolve.

    The limit is the quantile of the natural end tick, rounded up to a round
    figure. **A limit below this quantile truncates a game that had not
    resolved**, and a truncated game reports a winner by a comparison of held
    ground rather than by anything a faction achieved.
    """
    if not end_ticks:
        return 0
    at = quantile([float(tick) for tick in end_ticks], LIMIT_QUANTILE)
    steps = int(at // LIMIT_STEP) + (1 if at % LIMIT_STEP else 0)
    return max(steps, 1) * LIMIT_STEP


@dataclass(frozen=True)
class ArmSummary:
    """What one arm of one extent reached, over its whole seed set."""

    arm: str
    extent: int
    episodes: int
    settlements_median: float
    settlements_at_most_median: float
    grew_share: float
    held_tiles_median: float
    end_tick_median: float
    end_tick_upper_quartile: float
    end_tick_quantile: float
    end_tick_highest: int
    ceiling_share: float
    recommended_limit: int
    reaching_median: dict[int, float]
    reaching_share: dict[int, float]

    def as_row(self) -> dict[str, object]:
        """Return the summary as one mapping, for the JSON output."""
        return {
            "arm": self.arm,
            "extent": self.extent,
            "episodes": self.episodes,
            "settlements_median": self.settlements_median,
            "settlements_at_most_median": self.settlements_at_most_median,
            "grew_share": self.grew_share,
            "held_tiles_median": self.held_tiles_median,
            "end_tick_median": self.end_tick_median,
            "end_tick_upper_quartile": self.end_tick_upper_quartile,
            "end_tick_quantile": self.end_tick_quantile,
            "end_tick_highest": self.end_tick_highest,
            "ceiling_share": self.ceiling_share,
            "recommended_limit": self.recommended_limit,
            "reaching_median": {
                str(mark): value for mark, value in self.reaching_median.items()
            },
            "reaching_share": {
                str(mark): value for mark, value in self.reaching_share.items()
            },
        }


def summarise(readings: Sequence[EpisodeReading]) -> ArmSummary:
    """Reduce the episodes of one arm and one extent to one row."""
    if not readings:
        message = "an arm summary needs at least one episode"
        raise ValueError(message)
    first = readings[0]
    ends = [reading.end_tick for reading in readings]
    reaching_median: dict[int, float] = {}
    reaching_share: dict[int, float] = {}
    for mark in SETTLEMENT_MARKS:
        ticks = [
            float(reading.reaching_ticks[mark])
            for reading in readings
            if mark in reading.reaching_ticks
        ]
        reaching_share[mark] = len(ticks) / len(readings)
        reaching_median[mark] = quantile(ticks, 0.5) if ticks else 0.0
    return ArmSummary(
        arm=first.arm,
        extent=first.extent,
        episodes=len(readings),
        settlements_median=quantile(
            [float(reading.settlements_at_end) for reading in readings], 0.5
        ),
        settlements_at_most_median=quantile(
            [float(reading.settlements_at_most) for reading in readings], 0.5
        ),
        grew_share=sum(1 for reading in readings if reading.settlements_at_most > 1)
        / len(readings),
        held_tiles_median=quantile(
            [float(reading.held_tiles_at_end) for reading in readings], 0.5
        ),
        end_tick_median=quantile([float(tick) for tick in ends], 0.5),
        end_tick_upper_quartile=quantile([float(tick) for tick in ends], 0.75),
        end_tick_quantile=quantile([float(tick) for tick in ends], LIMIT_QUANTILE),
        end_tick_highest=max(ends),
        ceiling_share=sum(1 for reading in readings if reading.reached_ceiling)
        / len(readings),
        recommended_limit=recommended_limit(ends),
        reaching_median=reaching_median,
        reaching_share=reaching_share,
    )


def generation_seconds(
    worlds: int, mean_episode_ticks: float, ticks_per_second: float
) -> float:
    """Return how long one generation takes, derived from two measurements.

    **This figure is derived and not measured.** It multiplies the world
    count of a generation by the ticks one episode runs, and divides by the
    ticks a second the batch reached. A run pays more than this, because the
    batch loses parallelism as episodes end and the remaining worlds no
    longer fill every worker.
    """
    if ticks_per_second <= 0.0:
        return 0.0
    return worlds * mean_episode_ticks / ticks_per_second


def machine_facts() -> dict[str, object]:
    """Return what the machine is, so a row says where it was taken."""
    facts: dict[str, object] = {
        "machine": platform.machine(),
        "processor": platform.processor(),
        "system": platform.system(),
        "cores": os.cpu_count() or 0,
        "development_machine": True,
    }
    try:
        facts["load_average"] = [round(value, 2) for value in os.getloadavg()]
    except OSError:
        facts["load_average"] = []
    return facts


def ring_table(readings: Sequence[RingReading]) -> str:
    """Render the ring section as a table."""
    lines = [
        "# rings of the observation frame",
        "extent\trings\tcan fill\tfilled at start\tfilled at end\tempty",
    ]
    for row in readings:
        lines.append(
            f"{row.extent}\t{row.ring_count}\t{row.bound}\t"
            f"{row.reached_at_start}\t{row.reached_at_end}\t"
            f"{row.ring_count - row.reached_at_end}"
        )
    return "\n".join(lines)


def episode_table(summaries: Sequence[ArmSummary]) -> str:
    """Render the episode section as a table."""
    lines = [
        "# episodes, at a generous tick limit",
        "arm\textent\tgames\tsettlements\tmost\tgrew\ttiles\t"
        "end p50\tend p75\tend p90\tend max\tat ceiling\tlimit",
    ]
    for row in summaries:
        lines.append(
            f"{row.arm}\t{row.extent}\t{row.episodes}\t"
            f"{row.settlements_median:.1f}\t{row.settlements_at_most_median:.1f}\t"
            f"{row.grew_share:.2f}\t{row.held_tiles_median:.0f}\t"
            f"{row.end_tick_median:.0f}\t{row.end_tick_upper_quartile:.0f}\t"
            f"{row.end_tick_quantile:.0f}\t{row.end_tick_highest}\t"
            f"{row.ceiling_share:.2f}\t{row.recommended_limit}"
        )
    lines.append("")
    lines.append("# tick at which the settlement count first passed each mark")
    lines.append("arm\textent\tto 2\tshare\tto 3\tshare\tto 4\tshare")
    for row in summaries:
        cells = []
        for mark in SETTLEMENT_MARKS:
            cells.append(f"{row.reaching_median[mark]:.0f}")
            cells.append(f"{row.reaching_share[mark]:.2f}")
        lines.append(f"{row.arm}\t{row.extent}\t" + "\t".join(cells))
    return "\n".join(lines)


def sweep_throughput(
    extents: Sequence[int],
    factions: int,
    ceiling: int,
    interval: int,
    worlds: int,
    workers: int,
    decisions: int,
    repeats: int,
) -> dict[int, list[Measurement]]:
    """Measure the throughput of each extent, one round at a time.

    **The rounds interleave the extents rather than finishing one extent
    before it starts the next.** A development machine carries other work, and
    that work comes and goes. A sweep that measured every round of one extent
    together would give each extent a different machine, and the difference
    between two extents would then hold the difference between two machines.
    Interleaving spreads any load over every row, so the ratio between two
    extents survives it even when neither absolute figure does.

    The caller reads the ratio and not the absolute. One finding in this
    project records a training run cut to a fourteenth of its rate by
    contention, so an absolute figure taken beside other work states nothing.
    """
    rows: dict[int, list[Measurement]] = {extent: [] for extent in extents}
    for round_number in range(repeats):
        for extent in extents:
            config = world_of(extent, factions, ceiling, interval, True)
            print(
                f"  round {round_number + 1} of {repeats}: extent {extent} "
                f"over {worlds} worlds and {workers} workers",
                file=sys.stderr,
            )
            rows[extent].append(measure(config, worlds, workers, decisions))
    return rows


def best_of(rows: Sequence[Measurement]) -> Measurement:
    """Return the round that reached the highest rate.

    **The highest round is the least contended round.** Other work on the
    machine can only take time away, so the fastest of several rounds is the
    closest to what an idle machine would give. A mean would average in every
    interruption and report the load rather than the engine.
    """
    return max(rows, key=lambda row: row.ticks_per_second)


def throughput_table(
    rows: Sequence[tuple[int, Measurement]], price: float, worlds: int
) -> str:
    """Render the throughput section as a table.

    The price is the dollars an hour the machine costs. A price of zero leaves
    the cost column at zero, because a probe on a development machine has no
    price to state.

    **The last column is the figure to read.** It is the tick cost of the
    extent divided by the tick cost of the smallest extent of the sweep. A
    ratio survives a loaded machine, because the load falls on both ends of
    it. An absolute figure does not, and it belongs to the target platform in
    any case.
    """
    lines = [
        f"# throughput, {worlds} worlds in one process, a development machine",
        "extent\tworkers\tworlds\tticks\tseconds\tticks a second\t"
        "for each worker\tdollars a million ticks\ttick cost against the first",
    ]
    first = rows[0][1].ticks_per_second if rows else 0.0
    for extent, row in rows:
        rate = row.ticks_per_second
        cost = first / rate if rate > 0.0 else 0.0
        lines.append(
            f"{extent}\t{row.total_workers}\t{row.worlds}\t{row.ticks}\t"
            f"{row.seconds:.1f}\t{rate:.0f}\t"
            f"{row.ticks_per_second_per_worker:.1f}\t"
            f"{row.dollars_per_million_ticks(price):.4f}\t{cost:.2f}"
        )
    return "\n".join(lines)


def generation_table(
    rows: Sequence[tuple[int, Measurement]],
    summaries: Sequence[ArmSummary],
    worlds: int,
) -> str:
    """Render the derived generation cost as a table.

    The episode length comes from the controller arm of the same extent, if
    the run measured one. An extent with no episode reading holds no row,
    because the derivation has no episode length to multiply.
    """
    lengths = {
        row.extent: row.end_tick_median
        for row in summaries
        if row.arm == CONTROLLER_ARM
    }
    lines = [
        f"# one generation of {worlds} worlds, derived, a development machine",
        "extent\tticks a second\tmedian episode ticks\tgeneration seconds\t"
        "generation minutes",
    ]
    for extent, row in rows:
        length = lengths.get(extent)
        if length is None:
            continue
        seconds = generation_seconds(worlds, length, row.ticks_per_second)
        lines.append(
            f"{extent}\t{row.ticks_per_second:.0f}\t{length:.0f}\t"
            f"{seconds:.0f}\t{seconds / 60.0:.1f}"
        )
    return "\n".join(lines)


def measure_rings(
    extents: Sequence[int],
    factions: int,
    ceiling: int,
    interval: int,
    seed: int,
    decisions: int,
) -> list[RingReading]:
    """Read the ring reach of each extent, at the start and after some play.

    The reach at the start is what the frame holds at the first decision. The
    reach after some play can differ, because the frame is egocentric and its
    centre moves with the faction. **The gate channel is geometry and not
    fog**, so neither reading depends on what the faction has observed. Both
    come from one seed, so a row states a world and not a distribution.
    """
    readings = []
    for extent in extents:
        config = world_of(extent, factions, ceiling, interval, False)
        env = Env(config, PROBE_WEIGHTING)
        env.reset(seed)
        at_start = rings_filled(env)
        for _ in range(decisions):
            if env.done:
                break
            env.step(0)
        counts = ring_cell_counts(env)
        readings.append(
            RingReading(
                extent=extent,
                ring_count=len(counts),
                bound=ring_bound(extent, extent),
                reached_at_start=at_start,
                reached_at_end=rings_filled(env),
            )
        )
    return readings


def main() -> int:
    """Run the sections the caller asked for, and print what they measured."""
    parser = argparse.ArgumentParser(description="Measure the cost of a larger world.")
    parser.add_argument(
        "--extents",
        type=str,
        default=",".join(str(extent) for extent in DEFAULT_EXTENTS),
        help="the square world extents to sweep, separated by commas",
    )
    parser.add_argument("--factions", type=int, default=3)
    parser.add_argument(
        "--decision-interval", type=int, default=DEFAULT_DECISION_INTERVAL
    )
    parser.add_argument(
        "--tick-ceiling",
        type=int,
        default=DEFAULT_TICK_CEILING,
        help=(
            "the tick limit the episode section plays under. It is far above "
            "the limit a training run uses, because the section measures the "
            "tick a game resolves at"
        ),
    )
    parser.add_argument(
        "--seeds",
        type=int,
        default=16,
        help="how many episodes each arm of each extent plays",
    )
    parser.add_argument(
        "--arms",
        type=str,
        default=f"{CONTROLLER_ARM},{IDLE_ARM}",
        help=(
            "which arms to play. The controller arm gives every seat to the "
            "built-in controller, and the idle arm holds the learner seat and "
            "plays the no-op"
        ),
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=0,
        help="how many engine workers one batch holds. Zero takes half the cores",
    )
    parser.add_argument(
        "--worlds",
        type=int,
        default=DEFAULT_THROUGHPUT_WORLDS,
        help=(
            "how many worlds the throughput batch holds. The default is the "
            "product of the trainer's population and its seed count, because "
            "that is the shape a generation runs"
        ),
    )
    parser.add_argument(
        "--decisions",
        type=int,
        default=DEFAULT_THROUGHPUT_DECISIONS,
        help="how many crossings of the boundary the throughput section times",
    )
    parser.add_argument(
        "--repeats",
        type=int,
        default=3,
        help=(
            "how many rounds the throughput sweep runs. The rounds interleave "
            "the extents, and the sweep reports the fastest round of each "
            "extent, because other work on the machine can only take time away"
        ),
    )
    parser.add_argument(
        "--price", type=float, default=0.0, help="the dollars an hour the machine costs"
    )
    parser.add_argument(
        "--sections",
        type=str,
        default="rings,episodes,throughput",
        help="which sections to run, separated by commas",
    )
    parser.add_argument("--out", type=Path, help="write the text output here as well")
    parser.add_argument(
        "--json", action="store_true", help="write the rows as JSON and no table"
    )
    arguments = parser.parse_args()

    extents = [int(part) for part in arguments.extents.split(",") if part.strip()]
    if not extents:
        message = "the sweep holds no extent"
        raise ValueError(message)
    arms = [part.strip() for part in arguments.arms.split(",") if part.strip()]
    for arm in arms:
        if arm not in (CONTROLLER_ARM, IDLE_ARM):
            message = (
                f"{arm!r} names no arm. The probe plays the controller arm "
                "and the idle arm."
            )
            raise ValueError(message)
    sections = {part.strip() for part in arguments.sections.split(",") if part.strip()}
    unknown = sections - SECTIONS
    if unknown:
        message = (
            f"{sorted(unknown)} names no section. The probe runs {sorted(SECTIONS)}."
        )
        raise ValueError(message)
    cores = os.cpu_count() or 1
    workers = arguments.workers or max(cores // 2, 1)

    blocks: list[str] = []
    payload: dict[str, object] = {
        "facts": machine_facts(),
        "factions": arguments.factions,
        "decision_interval": arguments.decision_interval,
        "tick_ceiling": arguments.tick_ceiling,
        "workers": workers,
        "price_per_hour": arguments.price,
    }

    if "rings" in sections:
        print("  measuring the ring reach", file=sys.stderr)
        rings = measure_rings(
            extents,
            arguments.factions,
            arguments.tick_ceiling,
            arguments.decision_interval,
            seed=1,
            decisions=40,
        )
        payload["rings"] = [row.as_row() for row in rings]
        blocks.append(ring_table(rings))

    summaries: list[ArmSummary] = []
    if "episodes" in sections:
        episodes: list[EpisodeReading] = []
        for arm in arms:
            for extent in extents:
                config = world_of(
                    extent,
                    arguments.factions,
                    arguments.tick_ceiling,
                    arguments.decision_interval,
                    arm == IDLE_ARM,
                )
                seeds = viable_seeds(config, arguments.seeds, 1000)
                print(
                    f"  playing {arm} at {extent} over {len(seeds)} seeds",
                    file=sys.stderr,
                )
                readings = play_arm(
                    arm,
                    extent,
                    arguments.factions,
                    arguments.tick_ceiling,
                    arguments.decision_interval,
                    seeds,
                    workers,
                )
                episodes.extend(readings)
                summaries.append(summarise(readings))
        payload["episodes"] = [row.as_row() for row in episodes]
        payload["arms"] = [row.as_row() for row in summaries]
        blocks.append(episode_table(summaries))

    if "throughput" in sections:
        rounds = sweep_throughput(
            extents,
            arguments.factions,
            arguments.tick_ceiling,
            arguments.decision_interval,
            arguments.worlds,
            workers,
            arguments.decisions,
            arguments.repeats,
        )
        rows: list[tuple[int, Measurement]] = [
            (extent, best_of(rounds[extent])) for extent in extents
        ]
        payload["throughput"] = [
            {
                "extent": extent,
                "workers": row.total_workers,
                "worlds": row.worlds,
                "ticks": row.ticks,
                "seconds": row.seconds,
                "ticks_per_second": row.ticks_per_second,
                "ticks_per_second_per_worker": row.ticks_per_second_per_worker,
                "dollars_per_million_ticks": row.dollars_per_million_ticks(
                    arguments.price
                ),
                "every_round": [
                    round_row.ticks_per_second for round_row in rounds[extent]
                ],
            }
            for extent, row in rows
        ]
        blocks.append(throughput_table(rows, arguments.price, arguments.worlds))
        if summaries:
            blocks.append(generation_table(rows, summaries, arguments.worlds))

    if arguments.json:
        rendered = json.dumps(payload, indent=2)
        print(rendered)
        if arguments.out:
            arguments.out.parent.mkdir(parents=True, exist_ok=True)
            arguments.out.write_text(rendered + "\n", encoding="utf-8")
        return 0

    facts = payload["facts"]
    rendered = "\n\n".join(blocks)
    rendered += f"\n\n# machine {facts}"
    rendered += (
        "\n# a development machine on x86-64. The ring rows and the episode "
        "rows read the engine and carry to the target. The throughput rows "
        "and the generation rows read the machine and do not."
    )
    print(rendered)
    if arguments.out:
        arguments.out.parent.mkdir(parents=True, exist_ok=True)
        arguments.out.write_text(rendered + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
