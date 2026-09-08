"""The batch must report what the verb refused, and the record must keep it.

``World.act`` says whether the verb took the action. The single-world loop kept
that answer in the info of the step, and the batch used to throw it away. Every
training run goes through the batch, so nothing measured what share of a
policy's chosen actions the engine carried out. A policy that the engine mostly
refuses is close to a no-op whatever it chooses.

These tests drive the batch and the population runner, which are the callers a
run uses. A test that built a record by hand would prove that the record
holds numbers.
"""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING

import numpy as np

from cachette.learn.config import TrainConfig
from cachette.learn.env import Env, EnvConfig, VectorEnv, viable_seeds
from cachette.learn.reward import Weighting
from cachette.learn.rollout import run_population
from cachette.learn.train import train

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=8,
    decision_interval=10,
)

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


class FixedRow:
    """Take one row of the action table on every decision, legal or not.

    **This ignores the mask on purpose.** A policy that only ever chooses a
    legal row gives the verb nothing to refuse, so a fixture built from one
    would leave the refusal count at zero whatever the code did.
    """

    def __init__(self, row: int) -> None:
        """Take the row this policy sends on every decision."""
        self._row = row

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return the same row for every world of the batch."""
        del masks
        return [self._row] * len(observations)


def an_illegal_row(config: EnvConfig, seed: int) -> int:
    """Return a row that the seat may not take on the first decision."""
    env = Env(config, WEIGHTING)
    env.reset(seed)
    illegal = np.flatnonzero(env.action_mask() == 0)
    assert illegal.size, "every row is legal, so this world can refuse nothing"
    return int(illegal[-1])


def applied_alone(config: EnvConfig, seed: int, rows: Sequence[int]) -> list[bool]:
    """Play one world through the single-world loop, and keep every answer."""
    env = Env(config, WEIGHTING)
    env.reset(seed)
    answers = []
    for row in rows:
        if env.done:
            break
        answers.append(bool(env.step(row).info["applied"]))
    return answers


def applied_batched(config: EnvConfig, seed: int, rows: Sequence[int]) -> list[bool]:
    """Play the same world through the batch, and keep every answer."""
    vector = VectorEnv(config, WEIGHTING, count=1, workers=1)
    vector.reset([seed])
    answers = []
    for row in rows:
        if vector.done:
            break
        result = vector.step([row])[0]
        if result.info.get("skipped"):
            break
        answers.append(bool(result.info["applied"]))
    return answers


def test_the_batch_reports_the_refusal_the_single_world_loop_reports() -> None:
    """The batch discarded this answer, and every training run uses the batch.

    The two loops apply the same rows to the same world, so the engine gives
    the same answer to each. A batch that dropped the answer would report
    nothing here, and a batch that invented one would disagree.
    """
    seed = 3
    illegal = an_illegal_row(WORLD, seed)
    rows = [illegal, 0, illegal, 0, illegal, 0]
    alone = applied_alone(WORLD, seed, rows)
    batched = applied_batched(WORLD, seed, rows)
    assert alone == batched
    # The fixture must reach the case the assertion is for. A run in which the
    # verb took every action would pass this test and prove nothing.
    assert False in alone, "no action was refused, so this fixture proves nothing"


def test_a_population_run_counts_what_the_verbs_refused() -> None:
    """A run that refuses most of what a policy chooses is close to a no-op."""
    seeds = [3, 4]
    illegal = an_illegal_row(WORLD, seeds[0])
    played = run_population(WORLD, WEIGHTING, [FixedRow(illegal)], seeds, workers=1)
    assert played.chosen > 0
    assert played.refused > 0
    assert 0.0 < played.refusal_share <= 1.0
    for row in played.episodes:
        assert row.chosen == row.decisions
        assert row.refused <= row.chosen
        assert row.applied == row.chosen - row.refused


def test_a_policy_that_reads_the_mask_is_refused_less_often() -> None:
    """The count must move with the choices, and not only be non-zero.

    A count that came from the number of decisions rather than from the
    answers of the engine would report the same share for both policies.
    """
    seeds = [3, 4]
    illegal = an_illegal_row(WORLD, seeds[0])
    ignoring = run_population(WORLD, WEIGHTING, [FixedRow(illegal)], seeds, workers=1)
    obeying = run_population(WORLD, WEIGHTING, [FixedRow(0)], seeds, workers=1)
    assert obeying.refusal_share < ignoring.refusal_share


def test_a_seat_the_learner_does_not_hold_chooses_nothing() -> None:
    """The controller baseline sends no action, so it refuses none.

    A refusal share taken over zero choices must read as zero rather than
    fail, because that baseline is measured on every run.
    """
    played = run_population(
        replace(WORLD, controlled=False),
        WEIGHTING,
        [FixedRow(0)],
        [3],
        workers=1,
    )
    assert played.chosen == 0
    assert played.refused == 0
    assert played.refusal_share == 0.0


def test_every_episode_names_the_seed_and_the_candidate_it_played() -> None:
    """A comparison over shared seeds needs the outcome of each seed.

    A run that stored only the mean over the seeds could be compared only
    unpaired, which throws away most of the power of the comparison.
    """
    seeds = [3, 4, 5]
    played = run_population(
        WORLD, WEIGHTING, [FixedRow(0), FixedRow(1)], seeds, workers=1
    )
    assert len(played.episodes) == 2 * len(seeds)
    for index, row in enumerate(played.episodes):
        candidate, position = divmod(index, len(seeds))
        assert row.candidate == candidate
        assert row.seed == seeds[position]
        assert row.total_reward == played.returns[candidate, position]
    grouped = played.by_seed()
    assert list(grouped) == seeds
    for seed, rows in grouped.items():
        assert [row.candidate for row in rows] == [0, 1]
        assert all(row.seed == seed for row in rows)


def test_an_episode_reading_holds_every_quantity_the_engine_publishes() -> None:
    """The reading came from a tuple of seven names, and the schema holds more.

    A name the reading left out was a quantity no measurement could reach,
    and a name the reading spelled differently read as a missing value.
    """
    probe = Env(WORLD, WEIGHTING)
    published = {signal.name for signal in probe.signals.scalars()}
    played = run_population(WORLD, WEIGHTING, [FixedRow(0)], [3], workers=1)
    row = played.episodes[0]
    assert set(row.signals) == published
    reading = row.as_row()
    assert published <= set(reading)
    # The engine spells the tick of the end ``tick``, and a report spells it
    # ``end_tick``. Both names carry the same number.
    assert reading["end_tick"] == reading["tick"]
    assert (
        reading["won"] + reading["lost"] + reading["drawn"] + reading["unresolved"]
        == 1.0
    )


def test_a_run_records_the_score_of_every_candidate_and_every_episode(
    tmp_path: Path,
) -> None:
    """A run that logged only the best, the mean and the worst cannot be reread.

    No reward experiment can be read afterwards from three summary numbers.
    The record holds the whole score vector, the seeds of the generation and
    the reading of every episode.
    """
    train_config = TrainConfig(
        generations=1, population=4, seeds_per_generation=1, workers=1, seed=0
    )
    seeds = viable_seeds(WORLD, 2, 900)
    result = train("t", WORLD, WEIGHTING, train_config, tmp_path, seeds, validation=[])

    assert len(result["generations"]) == 1
    row = result["generations"][0]
    assert row["generation"] == 0
    assert len(row["ranked"]) == train_config.population
    assert len(row["absolute"]) == train_config.population
    assert row["seeds"] == seeds[:1]
    episodes = row["episodes"]
    assert len(episodes) == train_config.population
    assert [entry["candidate"] for entry in episodes] == list(
        range(train_config.population)
    )
    assert all(entry["seed"] == seeds[0] for entry in episodes)
    assert row["chosen"] == sum(entry["chosen"] for entry in episodes)
    assert row["refused"] == sum(entry["refused"] for entry in episodes)

    # The summary row carries the refusal figures too, so a reader of the
    # history meets them without opening the whole record.
    summary = result["history"][0]
    assert summary["chosen"] == row["chosen"]
    assert summary["refused"] == row["refused"]
