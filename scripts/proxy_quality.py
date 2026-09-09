"""Rank every field a reward could target by how well it predicts winning.

**The shaped term is the objective for most of a run, not a hint toward it.**
A generation ranks its candidates against each other, and in over half of the
generations measured no candidate won or almost none did, so the outcome term
separated nothing and the shaped term decided the whole ranking. Whatever field
the shaped term weighs is therefore what the search optimises.

This measures, for each field, whether ordering candidates by that field orders
them the way winning does. **The comparison is inside one world**, because a
generation plays one world and ranks the candidates of that world. A field that
predicts winning across worlds but not within one is no use to the search.

The measure is the share of winner and loser pairs of one world where the
winner holds the higher value. A field at 0.5 orders candidates no better than
a coin. A field at 1.0 orders them exactly as winning does. **A field below 0.5
orders them backwards, and weighing it teaches the search to lose.**

The candidates are a random projection network with random weights at the search
width, drawn on the unit sphere. **That is the neighbourhood a run starts from
and not the neighbourhood a trained centre sits in.** A generation late in a run
perturbs a centre that already plays, so its spread is narrower and its ordering
may differ. Read this as the answer for the reward a run should start with.

**A field that accumulates is confounded with how long the episode ran.** A
candidate that lost at tick 500 held ground for a fifth as long as one that lost
at the limit, and a winner ends its episode the moment it wins. The raw share
therefore reads a short win as a low value on every accumulating field, which
pushes such a field below a coin for a reason that is not about play. The table
reports a second share beside the first: the same measure over the value divided
by the end tick, which is the rate rather than the total. **Read the two
together.** A field whose total orders badly and whose rate orders well is a
field the episode length is hiding.

Run it with the candidate count and the world count:

    uv run python scripts/proxy_quality.py --policies 24 --seeds 8
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from cachette.learn.env import Env
from cachette.learn.policy import Policy, RandomPolicy
from cachette.learn.reward import Weighting
from cachette.learn.structured import StructuredPolicy
from cachette.learn.train import run_population

# Fields that cannot inform a ranking, and why each is left out. A constant
# cannot order anything, and the outcome fields are the thing being predicted
# rather than a predictor of it.
IGNORED: frozenset[str] = frozenset(
    {
        "world_tiles",  # the same number in every episode
        "world_passable_tiles",  # the same number in every episode
        "seated_faction_share",  # the same number in every episode
        "objective_weight",  # the style the caller chose, not a proxy
        "wonder_track_progress",  # a win condition, so it is the outcome
        "domination_progress",  # a win condition, so it is the outcome
        "renown_progress",  # a win condition, so it is the outcome
        "ground_progress",  # a win condition, so it is the outcome
        "tick_share",  # the position in the episode, which every row shares
        "remaining_ticks",  # the same quantity as the tick share
    }
)

# What the reward would have to weigh to target each reported field. The
# engine publishes no observation field that carries the tick of the end, so a
# reward cannot weigh the reported name at all and must weigh the remaining
# ticks instead.
WEIGHABLE_AS: dict[str, str] = {"end_tick": "remaining_ticks"}


@dataclass
class Ranking:
    """How well one field orders the candidates of a world."""

    field: str
    auc: float
    rate_auc: float
    pairs: int
    spread: float


def candidate_fields(probe: Env) -> list[str]:
    """Return the fields this measurement can read for one episode.

    **The set comes from the schema of the engine and not from a tuple.** The
    catalogue of the environment names every one-position quantity the engine
    publishes, and a reading of an episode carries all of them. A tuple
    written here would be a second declaration of that set, and a name the
    schema stopped carrying would read as a missing value rather than fail.

    The engine publishes no field that carries the tick of the end. A reading
    carries it as ``end_tick``, read from the world rather than from a
    signal, so this adds that one name to the candidates.
    """
    named = [signal.name for signal in probe.signals.scalars()]
    return [name for name in (*named, "end_tick") if name not in IGNORED]


def within_world_auc(
    seeds: list[int], values: np.ndarray, won: np.ndarray
) -> tuple[float, int]:
    """Return the share of same-world winner and loser pairs ordered right.

    The arrays hold one row for each candidate and one column for each seed.
    """
    better = 0.0
    pairs = 0
    for column in range(len(seeds)):
        winners = np.flatnonzero(won[:, column])
        losers = np.flatnonzero(~won[:, column])
        for winner in winners:
            for loser in losers:
                pairs += 1
                high = values[winner, column]
                low = values[loser, column]
                if high > low:
                    better += 1.0
                elif high == low:
                    better += 0.5
    return (better / pairs if pairs else float("nan"), pairs)


def main() -> None:
    """Play a spread of candidates and rank every field by what it predicts."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policies", type=int, default=24)
    parser.add_argument("--seeds", type=int, default=8)
    parser.add_argument("--seed-start", type=int, default=60_000)
    parser.add_argument("--sigma", type=float, default=1.0)
    parser.add_argument("--workers", type=int, default=12)
    parser.add_argument("--out", type=Path, default=None)
    arguments = parser.parse_args()

    from cachette.learn.__main__ import WORLD
    from cachette.learn.env import viable_seeds

    seeds = viable_seeds(WORLD, arguments.seeds, arguments.seed_start)

    # **The weighting shapes nothing.** The reading reports its fixed set
    # whatever the weighting says, so no term is needed to read a field. The
    # outcome carries the only weight, so the return of an episode is the win
    # itself and the ranking here is by outcome alone.
    weighting = Weighting(terms={}, won=1.0, lost=0.0, drawn=0.0)

    # The candidates are drawn on the unit sphere, which is the neighbourhood
    # generation zero draws from. One random policy sits among them as a
    # control.
    generator = np.random.default_rng(20260908)
    # **The two lengths come from the engine and never from a constant here.**
    # A length written by hand is a second declaration of a number the engine
    # owns, and nothing fails when the two disagree.
    probe = Env(WORLD, weighting)
    fields = candidate_fields(probe)
    zero = StructuredPolicy.of_catalogue(probe.action_length, probe.signals)
    size = zero.flat().size
    policies: list[Policy] = []
    for _ in range(arguments.policies - 1):
        draw = generator.standard_normal(size)
        policies.append(zero.rebuild(arguments.sigma * draw / np.linalg.norm(draw)))
    policies.append(RandomPolicy(seed=7))

    print(
        f"playing {len(policies)} candidates over {len(seeds)} worlds, "
        f"{len(policies) * len(seeds)} episodes, reading {len(fields)} fields"
    )
    readings = run_population(
        WORLD, weighting, policies, seeds, arguments.workers, label="proxy"
    ).rows()

    # The readings arrive flat, one for each candidate and seed together, at
    # index candidate times seed count plus seed.
    shape = (len(policies), len(seeds))
    won = np.zeros(shape, dtype=bool)
    columns = {field: np.zeros(shape) for field in fields}
    # **A field the reading does not report must fail, not default.** A
    # default of zero makes every candidate tie on that field, which scores as
    # a coin at 0.5 and reads as a useless proxy rather than as a field this
    # measurement never saw. This caught the tick of the end scoring as a coin
    # when it was never read at all.
    missing = sorted(set(fields) - set(readings[0]))
    if missing:
        message = (
            f"the reading reports no value for {missing}, so this run cannot "
            f"say whether those fields order anything"
        )
        raise AssertionError(message)
    for index, reading in enumerate(readings):
        row, column = divmod(index, len(seeds))
        won[row, column] = reading["won"] > 0.5
        for field in fields:
            columns[field][row, column] = float(reading[field])

    print(
        f"\n{int(won.sum())} of {won.size} episodes were won, and "
        f"{int((won.any(axis=0) & ~won.all(axis=0)).sum())} of {len(seeds)} "
        f"worlds hold both a winner and a loser and so can order anything\n"
    )

    # The end tick of each episode, which the rate of every field divides by.
    # **A zero here destroys the whole rate column and not one row of it.** The
    # check above refuses a field the reading leaves out, and it cannot see a
    # field the reading reports as zero. The end tick read zero for every
    # episode once, and this run then divided every field by nothing.
    ticks = columns["end_tick"]
    if not ticks.all():
        message = (
            "the end tick reads zero for at least one episode, so the rate of "
            "every field divides by nothing. An episode takes at least one "
            "tick, so a zero here is a defect in the reading and not a game"
        )
        raise AssertionError(message)

    rankings: list[Ranking] = []
    for field in fields:
        auc, pairs = within_world_auc(seeds, columns[field], won)
        rate_auc, _ = within_world_auc(seeds, columns[field] / ticks, won)
        rankings.append(
            Ranking(
                field=field,
                auc=auc,
                rate_auc=rate_auc,
                pairs=pairs,
                spread=float(np.nanmean(np.nanstd(columns[field], axis=0))),
            )
        )
    # A field with no ordered pair reads back as not a number. It sorts last,
    # because a field this run could not measure is not a field it ranked
    # above the fields it could.
    rankings.sort(
        key=lambda entry: (
            entry.auc != entry.auc,
            -entry.auc if entry.auc == entry.auc else 0.0,
        )
    )

    print(f"{'field':>22s} {'weigh as':>16s} {'total':>9s} {'rate':>9s} {'pairs':>7s}")
    for entry in rankings:
        weigh = WEIGHABLE_AS.get(entry.field, entry.field)
        print(
            f"{entry.field:>22s} {weigh:>16s} {entry.auc:9.3f} "
            f"{entry.rate_auc:9.3f} {entry.pairs:7d}"
        )
    print(
        "\n0.5 is a coin. Above 0.5 the field orders candidates the way "
        "winning does.\nBelow 0.5 it orders them backwards, and weighing it "
        "teaches the search to lose.\nThe total column reads the value at the "
        "end. The rate column divides it by the end tick,\nso a field whose "
        "total is low only because the episode was short shows the "
        "difference."
    )

    if arguments.out is not None:
        arguments.out.write_text(
            json.dumps(
                {
                    "seeds": seeds,
                    "candidates": len(policies),
                    "won": int(won.sum()),
                    "fields": [vars(entry) for entry in rankings],
                },
                indent=2,
            )
            + "\n"
        )
        print(f"\nwrote {arguments.out}")


if __name__ == "__main__":
    main()
