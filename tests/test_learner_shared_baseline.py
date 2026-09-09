"""One set of games must answer for every objective of a baseline pass.

A training run measures a yardstick before it trains. The built-in controller
holds every seat, and its return under each objective is the bar a policy must
beat. A run of six strategies measured six of these, and four of them replayed
the same games.

The games are the same because a scoring reaches nothing the games depend on.
The world comes from the world shape and the seed. The seat takes no action,
because the built-in controller holds it. The end of an episode comes from the
outcome reader, which reads the observation of the faction and the recorded end
of the game. A scoring weights the readings of an episode and never moves it.

**The test that matters is the one that compares the two paths.** A shared pass
that gave another number would be a cheaper measurement of a different
quantity, and no reader of a report could tell.

The world here is small and the seed set is short, so the whole file runs in a
few seconds. The seeds were chosen for what they hold rather than for how they
look: they hold wins and losses, and one of them ends with a faction of no
people at all. A fixture of one outcome would let a scoring that ignored the
outcome pass.

# References

[^1]: Testing Rules, a fixture supplies the input.
``.agents/rules/testing.md``
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from cachette.learn import __main__ as runner
from cachette.learn import rollout
from cachette.learn.baseline import (
    BaselineCache,
    available_workers,
    controller_baseline,
    controller_baselines,
)
from cachette.learn.env import Env, EnvConfig
from cachette.learn.policy import LinearPolicy
from cachette.learn.record import PopulationRecord
from cachette.learn.reward import Weighting

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterator, Mapping, Sequence

# A world small enough for a test and long enough to resolve. The seeds below
# end in a win, in a loss, and once with a faction that holds no people, so
# every weight of every objective below reaches a value that differs.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=300,
    horizon=30,
    decision_interval=10,
    controlled=False,
)

SEEDS = [50_000, 50_001, 50_002, 50_003, 50_004, 50_005]

# Six names over five objectives. Two of them state the same weighting, which
# is the shape a run of six strategies has: a linear policy and a network
# policy trained against one reward.
#
# The objectives reach different quantities on purpose. One reads the ground,
# one reads the stores, one reads the people, and one reads only the outcome.
# A pass that scored every name under one of them would still agree with
# itself, and a fixture of one objective could not find that.
OBJECTIVES: dict[str, Weighting] = {
    "conquer": Weighting(terms={"held_tiles": 0.1}, won=2000.0, lost=-200.0, drawn=0.0),
    "conquer-net": Weighting(
        terms={"held_tiles": 0.1}, won=2000.0, lost=-200.0, drawn=0.0
    ),
    "land": Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-10.0, drawn=0.0),
    "wealth": Weighting(
        terms={"store_total": 1.0e-5, "held_tiles": 0.5},
        won=100.0,
        lost=-10.0,
        drawn=0.0,
    ),
    "people": Weighting(
        terms={"population": 3.0, "held_tiles": 0.25},
        won=100.0,
        lost=-10.0,
        drawn=0.0,
    ),
    # Every shaped weight is absent, so this pays the outcome and nothing
    # else. It is the extreme of the distribution of objectives.
    "outcome": Weighting(terms={}, won=100.0, lost=-10.0, drawn=0.0),
}


@pytest.fixture(name="probe", scope="module")
def _probe() -> Env:
    """Return one environment of the test world, for its lengths."""
    return Env(WORLD, OBJECTIVES["land"])


@pytest.fixture(name="policy", scope="module")
def _policy(probe: Env) -> LinearPolicy:
    """Return the policy the baseline seats.

    The seat belongs to the built-in controller in this world, so the engine
    never asks this policy for anything it carries out. It is here because the
    measurement takes a policy.
    """
    return LinearPolicy.zeros(probe.action_length, probe.observation_length)


@pytest.fixture(name="cache")
def _cache(tmp_path: Path) -> BaselineCache:
    """Return a cache in its own directory, for a named engine build."""
    return BaselineCache(directory=tmp_path, engine_key="an-engine-build")


@pytest.fixture(name="no_cache")
def _no_cache() -> BaselineCache:
    """Return a cache that stores nothing, so every request measures."""
    return BaselineCache(directory=None)


class Spy:
    """Count the batches a pass plays, and record what each one scored.

    A pass that replays the games gives the right answer at four times the
    cost, and no assertion over the numbers can find that. This counts the
    calls instead.
    """

    def __init__(self) -> None:
        """Start with no call recorded."""
        self.calls: list[tuple[str, ...]] = []

    def install(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """Put this between the baseline pass and the batch it plays."""
        played = rollout.run_objectives

        def watched(
            config: EnvConfig,
            scorings: Mapping[str, Weighting],
            policies: Sequence[LinearPolicy],
            seeds: Sequence[int],
            workers: int,
            label: str = "",
        ) -> dict[str, PopulationRecord]:
            self.calls.append(tuple(scorings))
            return played(config, scorings, policies, seeds, workers, label)

        monkeypatch.setattr(rollout, "run_objectives", watched)


@pytest.fixture(name="spy")
def _spy(monkeypatch: pytest.MonkeyPatch) -> Iterator[Spy]:
    """Return a spy that counts the batches the baseline pass plays."""
    watcher = Spy()
    watcher.install(monkeypatch)
    yield watcher


def _one(
    cache: BaselineCache, probe: Env, policy: LinearPolicy, name: str
) -> dict[str, float]:
    """Measure one objective through the pass that measures one."""
    summary, _ = controller_baseline(
        WORLD,
        OBJECTIVES[name],
        policy,
        SEEDS,
        1,
        probe.observation_version,
        f"{name} baseline",
        cache,
    )
    return summary


def _many(
    cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> dict[str, tuple[dict[str, float], str]]:
    """Measure every objective through the pass that shares one set of games."""
    return controller_baselines(
        WORLD,
        OBJECTIVES,
        policy,
        SEEDS,
        1,
        probe.observation_version,
        "baseline",
        cache,
    )


def test_the_shared_pass_gives_what_a_pass_for_each_objective_gives(
    no_cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """This is the test the change rests on: one play, and the same numbers.

    Each objective is measured twice. Once through the pass that plays the
    games again for every objective, and once through the pass that plays them
    once. The two summaries must agree entry by entry.
    """
    shared = _many(no_cache, probe, policy)
    for name in OBJECTIVES:
        alone = _one(no_cache, probe, policy, name)
        assert shared[name][0] == alone, name


def test_the_objectives_do_not_all_score_the_same_number(
    no_cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """A fixture whose objectives agree would pass the test above for nothing.

    The five distinct objectives must give five distinct returns. Two names
    that state the same weighting must give one return between them.
    """
    shared = _many(no_cache, probe, policy)
    returns = {name: summary["return"] for name, (summary, _) in shared.items()}
    assert returns["conquer"] == returns["conquer-net"]
    distinct = {returns[name] for name in ("conquer", "land", "wealth", "people")}
    assert len(distinct) == 4
    assert returns["outcome"] != returns["land"]


def test_the_episodes_of_every_objective_are_one_set_of_games(
    no_cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """The readings of the games must not move when the objective moves.

    Every entry of the summary except the return comes from the engine. A
    scoring that reached the play would move one of them, and the return alone
    could not say so.
    """
    shared = _many(no_cache, probe, policy)
    readings = []
    for summary, _ in shared.values():
        readings.append({name: summary[name] for name in summary if name != "return"})
    for reading in readings[1:]:
        assert reading == readings[0]


def test_five_objectives_cost_one_batch(
    spy: Spy, no_cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """A pass over six names must play the games once, not six times."""
    _many(no_cache, probe, policy)
    assert len(spy.calls) == 1
    assert spy.calls[0] == ("conquer", "land", "wealth", "people", "outcome")


def test_two_objectives_that_share_a_reward_cost_one_measurement(
    spy: Spy, cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """The behaviour the cache already had must not regress.

    Two strategies that state the same weighting share one number. The first
    of them measures and writes, and the second reads what the first wrote.
    """
    shared = _many(cache, probe, policy)
    assert spy.calls[0].count("conquer") == 1
    assert "conquer-net" not in spy.calls[0]
    assert shared["conquer"][1] == "measured"
    assert shared["conquer-net"][1] == "cached"
    assert shared["conquer"][0] == shared["conquer-net"][0]


def test_a_second_pass_measures_nothing(
    spy: Spy, cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """Every objective of a warm cache reads, and the pass plays no batch."""
    first = _many(cache, probe, policy)
    second = _many(cache, probe, policy)
    assert len(spy.calls) == 1
    for name in OBJECTIVES:
        assert second[name][1] == "cached"
        assert second[name][0] == first[name][0]


def test_a_pass_measures_only_the_objectives_the_cache_misses(
    spy: Spy, cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """One stored objective must not make the others read a stored number."""
    _one(cache, probe, policy, "land")
    spy.calls.clear()
    shared = _many(cache, probe, policy)
    assert len(spy.calls) == 1
    assert "land" not in spy.calls[0]
    assert shared["land"][1] == "cached"
    assert shared["wealth"][1] == "measured"


def test_the_pass_that_fills_the_cache_shares_one_set_of_games(
    spy: Spy,
    probe: Env,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Drive the caller the launcher runs, and not only the pass beneath it.

    The launcher runs the fill pass once before it starts a trainer for each
    strategy. A shared pass that nothing reached would ship inert, so the test
    starts at the caller.
    """
    monkeypatch.setenv("CACHETTE_BASELINE_CACHE", str(tmp_path))
    monkeypatch.setenv("CACHETTE_ENGINE_KEY", "an-engine-build")
    monkeypatch.setattr(
        runner,
        "STRATEGIES",
        {name: (WORLD, weighting, "linear") for name, weighting in OBJECTIVES.items()},
    )
    assert runner.fill_baseline_cache(list(OBJECTIVES), SEEDS, 1, probe) == 0
    assert len(spy.calls) == 1
    printed = capsys.readouterr().out
    assert "conquer controller measured" in printed
    assert "conquer-net controller cached" in printed


def test_the_fill_pass_refuses_a_table_of_two_worlds(
    probe: Env, monkeypatch: pytest.MonkeyPatch
) -> None:
    """One batch plays one world, so two worlds must fail rather than blend."""
    from dataclasses import replace

    monkeypatch.setattr(
        runner,
        "STRATEGIES",
        {
            "land": (WORLD, OBJECTIVES["land"], "linear"),
            "wealth": (replace(WORLD, width=32), OBJECTIVES["wealth"], "linear"),
        },
    )
    with pytest.raises(ValueError, match="name another world"):
        runner.fill_baseline_cache(["land", "wealth"], SEEDS, 1, probe)


def test_a_pass_reads_the_workers_from_the_machine() -> None:
    """The worker count must come from what is available, not from a constant.

    A constant held the pass that fills the cache to a tenth of a rented
    machine of sixty four cores, and nothing failed.
    """
    import os

    assert available_workers() >= 1
    if hasattr(os, "sched_getaffinity"):
        assert available_workers() == len(os.sched_getaffinity(0))


def test_a_shared_pass_refuses_a_world_the_learner_holds(
    no_cache: BaselineCache, probe: Env, policy: LinearPolicy
) -> None:
    """A world the learner holds plays the policy, and the key holds no policy."""
    from dataclasses import replace

    with pytest.raises(ValueError, match="built-in controller holds"):
        controller_baselines(
            replace(WORLD, controlled=True),
            OBJECTIVES,
            policy,
            SEEDS,
            1,
            probe.observation_version,
            "baseline",
            no_cache,
        )
