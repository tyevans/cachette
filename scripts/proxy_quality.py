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

The candidates are drawn the way a generation draws them: a random projection
network with random weights at the search width. So the spread this measures is
the spread the search actually sees.

Run it with the candidate count and the world count:

    uv run python scripts/proxy_quality.py --policies 24 --seeds 8
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from cachette.learn.policy import MLPPolicy, Policy, RandomPolicy
from cachette.learn.reward import Weighting
from cachette.learn.train import REPORT_FIELDS, run_population

# Fields that cannot inform a ranking, and why each is left out. A constant
# cannot order anything, and the outcome fields are the thing being predicted
# rather than a predictor of it.
IGNORED: frozenset[str] = frozenset(
    {
        "tick_limit",  # the same number in every episode
        "faction",  # the seat, which is the same in every episode
        "game_over",  # true at the end of every episode
        "weight",  # a scaling constant of the array
        "wonder_claim",  # a win condition, so it is the outcome and not a proxy
    }
)

# What the reward would have to weigh to target each reported field. The
# reading names the tick of the end differently from the schema, and a reader
# who takes the reported name into a weighting gets an error rather than a
# reward.
WEIGHABLE_AS: dict[str, str] = {"end_tick": "tick"}


@dataclass
class Ranking:
    """How well one field orders the candidates of a world."""

    field: str
    auc: float
    pairs: int
    spread: float


def candidate_fields() -> list[str]:
    """Return the fields this measurement can read for one episode.

    **The reading reports a fixed set and not the terms of the weighting.**
    The trainer names that set, and it holds the tick of the end under another
    name, so a caller that asks the schema instead gets fields no reading
    carries. The set below is what an episode actually reports.
    """
    return [name for name in (*REPORT_FIELDS, "end_tick") if name not in IGNORED]


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
    parser.add_argument("--hidden", type=int, default=24)
    parser.add_argument("--sigma", type=float, default=1.0)
    parser.add_argument("--workers", type=int, default=12)
    parser.add_argument("--out", type=Path, default=None)
    arguments = parser.parse_args()

    from cachette.learn.__main__ import WORLD
    from cachette.learn.env import viable_seeds

    seeds = viable_seeds(WORLD, arguments.seeds, arguments.seed_start)
    fields = candidate_fields()

    # **The weighting shapes nothing.** The reading reports its fixed set
    # whatever the weighting says, so no term is needed to read a field. The
    # outcome carries the only weight, so the return of an episode is the win
    # itself and the ranking here is by outcome alone.
    weighting = Weighting(terms={}, won=1.0, lost=0.0, drawn=0.0)

    # The candidates are drawn as a generation draws them, so the spread is the
    # spread the search sees. One random policy sits among them as a control.
    generator = np.random.default_rng(20260908)
    zero = MLPPolicy.zeros(29, 184, arguments.hidden)
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
    _, readings, _ = run_population(
        WORLD, weighting, policies, seeds, arguments.workers, label="proxy"
    )

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

    rankings: list[Ranking] = []
    for field in fields:
        auc, pairs = within_world_auc(seeds, columns[field], won)
        rankings.append(
            Ranking(
                field=field,
                auc=auc,
                pairs=pairs,
                spread=float(np.nanmean(np.nanstd(columns[field], axis=0))),
            )
        )
    rankings.sort(key=lambda entry: -entry.auc if entry.auc == entry.auc else 0.0)

    print(
        f"{'field':>22s} {'weigh as':>16s} {'orders like winning':>20s} {'pairs':>7s}"
    )
    for entry in rankings:
        weigh = WEIGHABLE_AS.get(entry.field, entry.field)
        print(f"{entry.field:>22s} {weigh:>16s} {entry.auc:20.3f} {entry.pairs:7d}")
    print(
        "\n0.5 is a coin. Above 0.5 the field orders candidates the way "
        "winning does.\nBelow 0.5 it orders them backwards, and weighing it "
        "teaches the search to lose."
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
