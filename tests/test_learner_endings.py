"""A generation says why its episodes ended and how near the losing ones came.

A run reported the share of episodes a seat won and nothing else. A policy
that reaches almost every wonder and a policy that reaches none both report a
win share of zero, and the two call for opposite decisions: run for longer, or
change the reward.

**Three endings reach the instrument by three paths, and a fixture must supply
all three.** A game that a win reader ended early names that reader. A game
that ran to the tick limit names the territory reader, because the engine
compares held ground at the limit and records a winner there. An episode that
the horizon truncated holds no end record at all, and it reads the unfinished
name.

A fixture that only ran to the tick limit proves nothing, because an
instrument that answered the territory name for everything would pass it.[^1]

# References

[^1]: Testing Rules, a fixture supplies the input.
``.agents/rules/testing.md``
"""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import win_paths
from cachette.learn.endings import (
    PATH_LEADER,
    PATH_PROGRESS,
    UNFINISHED,
    end_shares,
    summarise_endings,
)
from cachette.learn.env import Env, EnvConfig
from cachette.learn.reward import Weighting
from cachette.learn.rollout import run_population
from cachette.learn.train import TrainConfig, train

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from pathlib import Path

    from cachette.learn.record import EpisodeRecord, PopulationRecord


class NoOp:
    """Take the no-op row of the action table on every decision.

    This measures how a game ends and not how a policy plays, so the policy
    chooses nothing. Row zero is the no-op and it is always legal.
    """

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return the no-op row for every world of the batch."""
        del masks
        return [0] * len(observations)


# The extent is small so that whole games run in a test. The horizon covers
# the tick limit exactly, so no episode of this configuration truncates.
TICK_LIMIT = 400
INTERVAL = 5
WHOLE = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    tick_limit=TICK_LIMIT,
    horizon=TICK_LIMIT // INTERVAL,
    decision_interval=INTERVAL,
)

# The same worlds under a horizon far below the tick limit. Every episode of
# this configuration stops before a reader fires, so every one holds no end
# record.
TRUNCATED = replace(WHOLE, horizon=10)

# A configuration the whole training loop runs in a test. A run of the real
# size takes hours.
TRAINED = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

# One seed whose game a win reader ends before the tick limit, and three whose
# games run to the limit. The early ending is the case that separates the end
# path from a constant.
#
# **The early seed is a fixture and it decays.** Which seed a reader ends early
# is a property of the engine. The seed was 9 until the campaign raise took the
# renown weight, and that game then ran to the limit. The test of the early
# ending asserts it, so a seed that stops ending early fails there first.
EARLY_SEED = 116

# One seed whose game runs to the tick limit while a rival still builds a
# wonder. The wonder test needs a leader that holds wonder work at the end, and
# the instrument reads the observation of the last decision.
#
# **This seed is a fixture and it decays too.** Part-built wonder work that no
# builder attends now loses work on every tick, and it stops at nothing.[^1]
# Seed 7 built toward a wonder for thirty decisions and then left it, so its
# work was gone before the limit. The seeds 0 and 3 held no work at the end
# either. The rival of this seed attends its work up to the last decision. The
# wonder test asserts the raw reading of this seed first, so a seed that stops
# holding work fails there with a message about the fixture.
#
# References
# ----------
# [^1]: ADR-0206, a part-built wonder decays when nobody works it, decision D1.
# ``docs/adrs/draft/adr-0206-a-part-built-wonder-decays-when-nobody-works-it.md``
WONDER_SEED = 98
SEEDS = [0, EARLY_SEED, WONDER_SEED, 7]

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


@pytest.fixture(scope="module")
def whole_games() -> PopulationRecord:
    """Play every seed to the end of its game."""
    return run_population(WHOLE, WEIGHTING, [NoOp()], SEEDS, 1, "")


@pytest.fixture(scope="module")
def truncated_games() -> PopulationRecord:
    """Play every seed under a horizon that stops it before any reader."""
    return run_population(TRUNCATED, WEIGHTING, [NoOp()], SEEDS, 1, "")


@pytest.fixture(scope="module")
def catalogue() -> Env:
    """Give back an environment whose catalogue names every published signal."""
    return Env(WHOLE, WEIGHTING)


def by_seed(record: PopulationRecord, seed: int) -> EpisodeRecord:
    """Return the one episode of a record that played a seed."""
    return record.by_seed()[seed][0]


def test_the_join_names_every_path_the_engine_holds() -> None:
    """The two mappings hold one entry for each win path and no other.

    The engine names the paths, and this module joins each one to a signal.
    **A join is two declarations that must agree**, so this is the check that
    fails when one of them moves.
    """
    assert set(PATH_PROGRESS) == set(win_paths())
    assert set(PATH_LEADER) == set(win_paths())


def test_every_joined_signal_is_a_scalar_of_the_schema(catalogue: Env) -> None:
    """Each signal the join names holds one position of the observation.

    A name the schema stopped carrying would read as a missing value and the
    instrument would report zero. This refuses that.
    """
    published = {signal.name for signal in catalogue.signals.scalars()}
    assert set(PATH_PROGRESS.values()) <= published
    assert set(PATH_LEADER.values()) <= published


def test_a_game_a_reader_ended_early_names_that_reader(
    whole_games: PopulationRecord,
) -> None:
    """An episode a win reader ended before the limit names the path."""
    episode = by_seed(whole_games, EARLY_SEED)
    assert episode.end_tick < TICK_LIMIT
    assert episode.end_path in win_paths()


def test_a_game_that_ran_to_the_limit_ends_on_territory(
    whole_games: PopulationRecord,
) -> None:
    """The engine compares held ground at the tick limit and records a winner.

    An episode that reached the limit therefore holds an end record, and the
    reader that wrote it is the territory reader.
    """
    reached = [row for row in whole_games.episodes if row.end_tick == TICK_LIMIT]
    assert reached
    assert {row.end_path for row in reached} == {"territory"}


def test_a_truncated_episode_holds_no_end_record(
    truncated_games: PopulationRecord,
) -> None:
    """A horizon below the tick limit stops the episode before any reader."""
    assert {row.end_path for row in truncated_games.episodes} == {UNFINISHED}


def test_the_shares_name_every_path_and_sum_to_one(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """The shares hold one entry for each path and one for the unfinished.

    A path no episode reached reads zero rather than being absent, so a reader
    that compares two generations never reads a missing key as a path that
    stopped existing.
    """
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    assert set(summary.shares) == {*win_paths(), UNFINISHED}
    assert sum(summary.shares.values()) == pytest.approx(1.0)


def test_the_shares_count_the_episodes_of_each_ending(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """Each share is the count of the episodes that ended that way.

    **This is the assertion that a constant cannot pass.** The fixture holds
    one game a reader ended early and three that ran to the limit, so two
    entries are above zero and they differ.
    """
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    counted = {
        name: sum(1 for row in whole_games.episodes if row.end_path == name)
        / len(whole_games.episodes)
        for name in summary.shares
    }
    assert summary.shares == counted
    assert sum(1 for share in summary.shares.values() if share > 0.0) >= 2


def test_an_empty_set_names_every_path_and_shares_nothing() -> None:
    """A set of no episode reads zero on every ending rather than refusing."""
    shares = end_shares([], win_paths())
    assert set(shares) == {*win_paths(), UNFINISHED}
    assert set(shares.values()) == {0.0}


def test_every_reach_is_a_fraction(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """The engine publishes a share as a fixed-point integer, and this scales it.

    A reading that skipped the scale would report tens of thousands. Each
    entry here runs from zero to one, and the three entries of one path are in
    order.
    """
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    assert set(summary.own) == set(win_paths())
    for reach in (*summary.own.values(), *summary.leader.values()):
        assert 0.0 <= reach.median <= 1.0
        assert reach.median <= reach.upper <= reach.highest <= 1.0


def test_the_leader_reaches_at_least_as_far_as_the_seat(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """The leading faction is the seat while the seat leads, and never below it.

    A reader that asks whether the opponents are near a wonder reads the
    leader, so this is the entry that must never fall under the own reading.
    """
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    for name, own in summary.own.items():
        assert summary.leader[name].highest >= own.highest


def test_a_wonder_that_nobody_started_reaches_nothing(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """The instrument separates a path nobody reached from a path somebody won.

    No episode of this fixture ends on the wonder path, and the seat builds
    none, so the wonder reach of the seat is zero. **A generation that read the
    same zero for a policy at the edge of a wonder would say nothing**, and the
    leader entry of the same path is above zero here, which proves the reading
    is not a constant zero.

    The first assertion reads the raw signal of one episode, and it checks the
    fixture. The last assertion reads the summary, and it checks the
    instrument. An instrument that read a constant zero passes the first and
    fails the last.
    """
    wonder_game = by_seed(whole_games, WONDER_SEED)
    assert wonder_game.signals[PATH_LEADER["wonder"]] > 0.0, (
        f"seed {WONDER_SEED} ends with no wonder work for any faction, so the "
        "fixture no longer supplies a leader that started a wonder. Choose "
        "another seed whose rival still builds at the tick limit."
    )
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    assert summary.shares["wonder"] == 0.0
    assert summary.own["wonder"].highest == 0.0
    assert summary.leader["wonder"].highest > 0.0


def test_the_columns_carry_every_ending_and_every_path(
    whole_games: PopulationRecord, catalogue: Env
) -> None:
    """A history row holds one flat column for each ending and each path."""
    summary = summarise_endings(whole_games.episodes, catalogue.signals)
    columns = summary.columns()
    for name in (*win_paths(), UNFINISHED):
        assert columns[f"ended_{name}"] == summary.shares[name]
    for name in win_paths():
        assert columns[f"reach_{name}"] == summary.own[name].upper
        assert columns[f"reach_leader_{name}"] == summary.leader[name].upper


def test_the_report_of_an_episode_carries_the_end_path(
    whole_games: PopulationRecord,
) -> None:
    """The record a report file stores names the ending of the episode."""
    stored = by_seed(whole_games, EARLY_SEED).as_dict()
    assert stored["end_path"] == by_seed(whole_games, EARLY_SEED).end_path


def test_a_training_run_prints_and_stores_the_instrument(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """The training loop reaches the instrument, prints it, and records it.

    **A test that built the summary itself would prove nothing about the
    run.** The engine and the loop are obligated to reach this, so the test
    starts at the loop.[^1]

    References
    ----------
    [^1]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``
    """
    result = train(
        "t",
        TRAINED,
        WEIGHTING,
        TrainConfig(
            generations=1, population=4, seeds_per_generation=1, workers=2, seed=0
        ),
        tmp_path,
        [0],
        validation=[],
    )

    printed = capsys.readouterr().out
    assert " ended " in printed, printed
    assert " reach " in printed, printed
    for name in (*win_paths(), UNFINISHED):
        assert name in printed, printed

    history = result["history"][0]
    for name in (*win_paths(), UNFINISHED):
        assert f"ended_{name}" in history
    for name in win_paths():
        assert f"reach_{name}" in history
        assert f"reach_leader_{name}" in history

    endings = result["generations"][0]["endings"]
    assert isinstance(endings, dict)
    assert set(endings["shares"]) == {*win_paths(), UNFINISHED}
    assert set(endings["own"]) == set(win_paths())
    assert set(endings["leader"]) == set(win_paths())
