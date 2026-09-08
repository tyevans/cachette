"""Play every constant-preference policy, and report the win share of each.

A constant-preference policy reads nothing. It names one action row and takes
that row whenever the row is legal, and the no-op otherwise. There is one such
policy for each row of the action table.

**This measures what search must beat.** A trained policy reads the
observation and chooses. If the best policy that reads nothing wins as often
as the best trained policy, then the search bought nothing that an enumeration
of the action table gives free, and the observation and the action table are
the binding constraint rather than the optimiser.[^1]

The win share is reported against the exact chance line. One seat of a
symmetric game of three factions takes one third of the wins whatever the
players do, so the chance line is one third and it needs no measurement.

Run it with the seed count and the worker count:

    uv run python scripts/constant_policy_baseline.py --seeds 512 --workers 30

## References

[^1]: Report 40, what a well-trained policy needs, section 11, experiment 1.
``docs/research/reports/40-what-a-well-trained-policy-needs.md``
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from cachette._core import World
from cachette.learn.env import EnvConfig, viable_seeds
from cachette.learn.policy import Policy, PolicyFit, load_policy
from cachette.learn.reward import Weighting
from cachette.learn.train import run_population


@dataclass
class Result:
    """What one policy scored over the seed set."""

    name: str
    won: float
    standard_errors: float
    stored: bool


class ConstantPreference:
    """Take one row of the action table whenever that row is legal.

    The row is fixed for the life of the policy. Row zero is the no-op and it
    is always legal, so a policy whose row is illegal on a decision falls back
    to it and the episode still advances.
    """

    def __init__(self, row: int) -> None:
        """Take the row this policy will prefer for its whole life."""
        self._row = row

    @property
    def row(self) -> int:
        """The row this policy prefers."""
        return self._row

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return the preferred row where it is legal, and the no-op elsewhere."""
        legal = masks[:, self._row].astype(bool)
        return [int(self._row) if ok else 0 for ok in legal]


def main() -> None:
    """Play every constant-preference policy and print a table."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seeds", type=int, default=512)
    parser.add_argument("--seed-start", type=int, default=50_000)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--out", type=Path, default=None)
    parser.add_argument(
        "--batch",
        type=int,
        default=4,
        help=(
            "how many policies to score at once. A batch is written before "
            "the next one starts, so a stopped run loses at most one batch."
        ),
    )
    parser.add_argument(
        "--policy",
        action="append",
        default=[],
        type=Path,
        help=(
            "a stored policy to play beside the constant ones. Repeat it. "
            "Every policy plays the same worlds under the same scoring, so "
            "the shares in one table compare and a share from another run "
            "does not."
        ),
    )
    arguments = parser.parse_args()

    # The import sits here so that the module states the world it plays in one
    # place, and so that a reader of the table above needs no second file.
    from cachette.learn.__main__ import WORLD

    seeds = viable_seeds(WORLD, arguments.seeds, arguments.seed_start)
    sample = _sample_world(WORLD)
    schema_rows = int(sample.action_schema()["length"])
    policies: list[Policy] = [ConstantPreference(row) for row in range(schema_rows)]
    names = [f"row {row}" for row in range(schema_rows)]

    # **A stored policy must play the same worlds as the constant ones.** A
    # share measured on another seed set under another weighting is a different
    # quantity, and the register already holds one comparison that failed for
    # that reason.
    fit = PolicyFit.of_world(sample)
    for path in arguments.policy:
        weights, meta = load_policy(path, fit)
        policies.append(weights)
        generation = meta.get("generation", -1)
        stated = int(generation) if isinstance(generation, (int, float)) else -1
        names.append(f"{path.stem} (gen {stated})")

    # **The weighting must not change the ranking of a win share.** A win share
    # counts outcomes, so the shaped terms are set to nothing and only the
    # terminal terms carry weight.
    weighting = Weighting(terms={}, won=1.0, lost=0.0, drawn=0.0)

    chance = 1.0 / WORLD.faction_count
    error = math.sqrt(chance * (1.0 - chance) / len(seeds))

    print(
        f"playing {len(policies)} policies over {len(seeds)} worlds of "
        f"{WORLD.width} by {WORLD.height} with {WORLD.faction_count} factions, "
        f"{len(policies) * len(seeds)} episodes in batches of {arguments.batch}"
    )

    # **A long run must survive being stopped.** One batch of every policy at
    # once loses everything when the machine is interrupted, and this run takes
    # hours on one development machine. Each batch is scored on its own and its
    # result is written before the next one starts, so a second call continues
    # where the first stopped.
    done: dict[str, float] = {}
    if arguments.out is not None and arguments.out.exists():
        stored_run = json.loads(arguments.out.read_text())
        if (
            stored_run.get("seeds") == len(seeds)
            and stored_run.get("seed_start") == arguments.seed_start
        ):
            done = {entry["name"]: entry["won"] for entry in stored_run["policies"]}
            print(f"continuing a run that already scored {len(done)} policies")
        else:
            print("the stored run played other worlds, so it is ignored")

    table: list[Result] = [
        Result(
            name=name,
            won=share,
            standard_errors=(share - chance) / error,
            stored=index >= schema_rows,
        )
        for index, name in enumerate(names)
        if (share := done.get(name)) is not None
    ]

    # **The decisive comparison runs first.** The question this script exists
    # to answer is whether a stored policy beats the best policy that reads
    # nothing, and a run ordered by array index answers it last. A stopped run
    # then holds every constant score and none of the comparison. The stored
    # policies sit at the end of the list because they are appended to it, so
    # the order of the list is not the order of the work.
    pending = sorted(
        ((index, name) for index, name in enumerate(names) if name not in done),
        key=lambda entry: (entry[0] < schema_rows, entry[0]),
    )
    for start in range(0, len(pending), arguments.batch):
        batch = pending[start : start + arguments.batch]
        returns, readings, _ = run_population(
            WORLD,
            weighting,
            [policies[index] for index, _ in batch],
            seeds,
            arguments.workers,
            label=f"batch {start // arguments.batch + 1}",
        )
        # **The returns arrive shaped one row for each policy, and the readings
        # arrive flat with one entry for each policy and seed together.** Under
        # this weighting an episode returns one for a win and nothing
        # otherwise, so the mean of a row is the win share of that policy. The
        # readings are averaged the same way and the two must agree, because
        # reading the flat list as one entry for each policy is the mistake
        # this comment exists to prevent.
        shares = returns.mean(axis=1)
        for position, (index, name) in enumerate(batch):
            share = float(shares[position])
            first = position * len(seeds)
            counted = [
                float(row["won"]) for row in readings[first : first + len(seeds)]
            ]
            if abs(sum(counted) / len(counted) - share) > 1e-9:
                message = (
                    f"the return of {name} says {share:.4f} and its readings "
                    f"say {sum(counted) / len(counted):.4f}"
                )
                raise AssertionError(message)
            table.append(
                Result(
                    name=name,
                    won=share,
                    standard_errors=(share - chance) / error,
                    stored=index >= schema_rows,
                )
            )
            print(f"  {name:>33s} {share:7.4f}")
        _write(arguments.out, seeds, arguments.seed_start, chance, error, table)

    table.sort(key=lambda entry: entry.won, reverse=True)

    print(
        f"\nchance is {chance:.4f} and the standard error over "
        f"{len(seeds)} worlds is {error:.4f}. A share needs to beat "
        f"{chance + 2 * error:.4f} to stand two errors above chance.\n"
    )
    print(f"{'policy':>34s} {'wins':>7s} {'errors from chance':>19s}")
    for entry in table:
        mark = "*" if entry.stored else " "
        print(
            f"{entry.name:>33s}{mark} {entry.won:7.4f} {entry.standard_errors:+18.1f}"
        )
    print("\n* a stored policy. Every other row reads nothing at all.")

    constants = [entry for entry in table if not entry.stored]
    trained = [entry for entry in table if entry.stored]
    best_constant = constants[0]
    print(
        f"\nthe best policy that reads nothing is {best_constant.name} "
        f"at {best_constant.won:.4f}"
    )
    for entry in trained:
        verdict = "does not beat it" if entry.won <= best_constant.won else "beats it"
        print(f"  {entry.name} at {entry.won:.4f} {verdict}")


def _write(
    out: Path | None,
    seeds: list[int],
    seed_start: int,
    chance: float,
    error: float,
    table: list[Result],
) -> None:
    """Write what is scored so far, so that a stopped run can continue."""
    if out is None:
        return
    out.write_text(
        json.dumps(
            {
                "seeds": len(seeds),
                "seed_start": seed_start,
                "chance": chance,
                "standard_error": error,
                "policies": [vars(entry) for entry in table],
            },
            indent=2,
        )
        + "\n"
    )


def _sample_world(config: EnvConfig) -> World:
    """Build one world of this shape, to read the schemas it publishes.

    The action table and the observation layout are functions of the world
    parameters, so any world of the shape a run plays answers for all of them.
    """
    world = World(
        width=config.width,
        height=config.height,
        seed=1,
        faction_count=config.faction_count,
    )
    world.seed_world()
    return world


if __name__ == "__main__":
    main()
