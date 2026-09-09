"""The recorded end tick is the tick the game ended at.

A record of one episode reports how long the game ran. The column read the
signals of the engine under the name ``tick`` and took a default of zero, and
the engine publishes no signal of that name. Every episode the project
recorded therefore reported zero, and nothing failed.[^1]

**Three endings reach the column by three paths, and a fixture must supply
all three.** A game that a win reader ended early holds an end record, and
the clock of the world stands past that tick by up to one decision interval.
A game that ran to the tick limit holds an end record at the limit. An
episode that the horizon truncated holds no end record at all.

A fixture that only ends at the tick limit proves nothing, because a column
that reported the limit constant would pass it. The early ending is the case
that separates the three sources from each other.[^2]

# References

[^1]: Findings register, FND-689. ``docs/FINDINGS.md``
[^2]: Testing Rules, a fixture supplies the input.
``.agents/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig
from cachette.learn.record import EpisodeRecord
from cachette.learn.reward import Weighting
from cachette.learn.train import run_population


class NoOp:
    """Take the no-op row of the action table on every decision.

    This measures the clock of the game and not the play, so the policy
    chooses nothing. Row zero is the no-op and it is always legal.
    """

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return the no-op row for every world of the batch."""
        del masks
        return [0] * len(observations)


# The extent is small so that a whole game runs in a test, and the horizon
# covers the tick limit exactly so that no episode truncates.
TICK_LIMIT = 400
INTERVAL = 5
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    tick_limit=TICK_LIMIT,
    horizon=TICK_LIMIT // INTERVAL,
    decision_interval=INTERVAL,
)

# The seed of a game that a win reader ends before the tick limit. The engine
# fires the reader inside the interval one decision runs, so the clock of the
# world stands past the end of this game.
EARLY_SEED = 9

# The seed of a game that runs to the tick limit, where the engine compares
# held ground and records a winner.
LIMIT_SEED = 0

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


def play(config: EnvConfig, seed: int) -> tuple[Env, EpisodeRecord]:
    """Drive one whole episode of the no-op policy, and record it.

    The policy takes the no-op at every decision, because this measures the
    clock of the game and not the play. The engine drives every other seat.
    """
    env = Env(config, WEIGHTING)
    env.reset(seed)
    while not env.done:
        env.step(0)
    record = EpisodeRecord.of_env(
        env, candidate=0, seed=seed, total_reward=0.0, chosen=0, refused=0
    )
    return env, record


def test_a_game_that_ends_early_records_the_tick_the_reader_fired_at() -> None:
    """The end record is the source, and the clock of the world is not.

    This is the case that separates the two. The reader fires inside the
    interval one decision runs, so the clock stands past the end. A column
    that read the clock would report the later number, and a column that
    reported the tick limit would report a third.
    """
    env, record = play(WORLD, EARLY_SEED)
    end = env.world.game_end()
    assert end is not None, "this seed no longer ends by a reader, so it proves nothing"
    assert end["tick"] < TICK_LIMIT, "this seed no longer ends before the tick limit"
    assert env.world.tick > end["tick"], (
        "the clock no longer stands past the end, so this seed cannot tell "
        "the end record from the clock"
    )

    assert record.end_tick == end["tick"]
    assert record.as_row()["end_tick"] == float(end["tick"])
    assert record.as_dict()["end_tick"] == end["tick"]


def test_a_game_that_reaches_the_tick_limit_records_the_limit() -> None:
    """The tick limit is a rule of the game, so the limit is a real ending.

    The engine compares held ground at the limit and records a winner there,
    so an episode that reaches it holds an end record like any other.
    """
    env, record = play(WORLD, LIMIT_SEED)
    end = env.world.game_end()
    assert end is not None, "this seed no longer ends at all, so it proves nothing"
    assert end["tick"] == TICK_LIMIT, "this seed no longer runs to the tick limit"

    assert record.end_tick == TICK_LIMIT
    assert record.as_row()["end_tick"] == float(TICK_LIMIT)


def test_a_truncated_episode_records_the_clock_of_the_world() -> None:
    """A horizon shorter than the tick limit ends the episode with no record.

    The clock is then the only statement of how far the episode ran. A
    default of zero here would report an episode that ran no tick.
    """
    truncating = EnvConfig(
        width=WORLD.width,
        height=WORLD.height,
        faction_count=WORLD.faction_count,
        tick_limit=TICK_LIMIT,
        horizon=6,
        decision_interval=INTERVAL,
    )
    env, record = play(truncating, EARLY_SEED)
    assert env.truncated, "the horizon no longer truncates, so this proves nothing"
    assert env.world.game_end() is None, "this episode now holds an end record"

    assert record.end_tick == env.world.tick
    assert record.end_tick > 0


def test_the_engine_publishes_no_signal_that_carries_the_tick_of_the_end() -> None:
    """This is the belief that put a zero in every recorded episode.

    A record read the signals under the name ``tick``. No schema of the
    engine carries that name, so the read returned its default forever. The
    row therefore holds no such column either.
    """
    probe = Env(WORLD, WEIGHTING)
    assert "tick" not in probe.signals
    with pytest.raises(KeyError, match="names no signal"):
        probe.signals.signal("tick")

    _env, record = play(WORLD, EARLY_SEED)
    assert "tick" not in record.signals
    assert "tick" not in record.as_row()


def test_a_played_population_carries_the_end_tick_of_every_episode() -> None:
    """The batch is the caller the training run drives, so the test starts there.

    A test that builds one record proves the record works. It does not prove
    that the path a run takes reaches the same reader.
    """
    seeds = [EARLY_SEED, LIMIT_SEED]
    played = run_population(WORLD, WEIGHTING, [NoOp()], seeds, workers=1)

    assert len(played.episodes) == len(seeds)
    for episode, row in zip(played.episodes, played.rows(), strict=True):
        assert episode.end_tick > 0
        assert row["end_tick"] == float(episode.end_tick)

    ends = {episode.seed: episode.end_tick for episode in played.episodes}
    assert ends[LIMIT_SEED] == TICK_LIMIT
    assert ends[EARLY_SEED] < TICK_LIMIT, (
        "both episodes ran to the limit, so this fixture cannot tell a real "
        "end tick from the limit constant"
    )
