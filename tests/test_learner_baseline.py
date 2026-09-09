"""The controller baseline must be measured once and read back after.

A training run plays the built-in controller over the whole held-out seed set
to set the bar the trained policy must beat. That pass took over nine minutes
of a rented machine of sixty four cores, and it printed nothing while it ran.
The run then paid for it again in every process it started.

The tests below fix both halves of the repair. **The test that matters is the
one for a changed key**, because a cache that answers for another seed set,
another world or another engine reports a wrong number with the authority of
a measurement, and no later reader can tell.

The world here is small and the seed set is short, so the whole file runs in
a few seconds. A large world would measure the same code and cost minutes.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from dataclasses import replace
from pathlib import Path
from types import ModuleType

import pytest

from cachette.learn.baseline import BaselineCache, controller_baseline, key_of
from cachette.learn.env import Env, EnvConfig
from cachette.learn.policy import LinearPolicy
from cachette.learn.reward import Weighting

ROOT = Path(__file__).resolve().parent.parent

# A world small enough for a test and large enough to seat three factions.
# The tick limit is short, because these tests fix the cache and not the play.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=40,
    horizon=4,
    decision_interval=10,
    controlled=False,
)

# A weighting that scores the outcome and one shaped term. Both reach the
# return, so a change to either changes the number the cache stores.
SCORING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-10.0, drawn=0.0)

SEEDS = [50_000, 50_001]


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


@pytest.fixture(name="cache")
def _cache(tmp_path: Path) -> BaselineCache:
    """Return a cache in its own directory, for a named engine build."""
    return BaselineCache(directory=tmp_path, engine_key="an-engine-build")


@pytest.fixture(name="probe", scope="module")
def _probe() -> Env:
    """Return one environment of the test world, for its lengths."""
    return Env(WORLD, SCORING)


def _measure(
    cache: BaselineCache, probe: Env, **changed: object
) -> tuple[dict[str, float], str]:
    """Ask for the baseline, with any input of the request changed."""
    request: dict[str, object] = {
        "config": WORLD,
        "scoring": SCORING,
        "seeds": SEEDS,
        "schema_version": probe.observation_version,
    }
    request.update(changed)
    return controller_baseline(
        request["config"],  # type: ignore[arg-type]
        request["scoring"],  # type: ignore[arg-type]
        LinearPolicy.zeros(probe.action_length, probe.observation_length),
        request["seeds"],  # type: ignore[arg-type]
        1,
        request["schema_version"],  # type: ignore[arg-type]
        "test baseline",
        cache,
    )


def test_a_cache_hit_returns_the_number_the_measurement_returned(
    cache: BaselineCache, probe: Env
) -> None:
    """A stored number must equal the measured one, field by field."""
    measured, first = _measure(cache, probe)
    cached, second = _measure(cache, probe)
    assert first == "measured"
    assert second == "cached"
    assert measured == cached


def test_a_changed_seed_set_misses_rather_than_serving_the_old_number(
    cache: BaselineCache, probe: Env
) -> None:
    """A mean over other worlds is another quantity, so it must not be served."""
    _measure(cache, probe)
    _, source = _measure(cache, probe, seeds=[50_002, 50_003])
    assert source == "measured"


def test_a_seed_set_that_holds_a_subset_of_the_stored_one_misses(
    cache: BaselineCache, probe: Env
) -> None:
    """A subset is a different seed set, and a partial mean is not the mean."""
    _measure(cache, probe)
    _, source = _measure(cache, probe, seeds=SEEDS[:1])
    assert source == "measured"


def test_a_seed_set_that_holds_a_superset_of_the_stored_one_misses(
    cache: BaselineCache, probe: Env
) -> None:
    """A superset adds worlds the stored mean never played."""
    _measure(cache, probe)
    _, source = _measure(cache, probe, seeds=[*SEEDS, 50_004])
    assert source == "measured"


def test_a_changed_world_shape_misses(cache: BaselineCache, probe: Env) -> None:
    """Another world is another game, whatever the seeds are."""
    _measure(cache, probe)
    other = replace(WORLD, width=WORLD.width + 2, height=WORLD.height + 2)
    narrow = Env(other, SCORING)
    _, source = _measure(cache, narrow, config=other)
    assert source == "measured"


def test_a_changed_faction_count_misses(cache: BaselineCache, probe: Env) -> None:
    """The faction count sets the chance line, so it must reach the key."""
    _measure(cache, probe)
    other = replace(WORLD, faction_count=2)
    _, source = _measure(cache, Env(other, SCORING), config=other)
    assert source == "measured"


def test_a_changed_decision_interval_misses(cache: BaselineCache, probe: Env) -> None:
    """The interval decides how much of the game the learner's seat holds."""
    _measure(cache, probe)
    other = replace(WORLD, decision_interval=WORLD.decision_interval * 2)
    _, source = _measure(cache, probe, config=other)
    assert source == "measured"


def test_a_changed_engine_key_misses(cache: BaselineCache, probe: Env) -> None:
    """An engine change moves the play, and the cache must not hide that."""
    _measure(cache, probe)
    other = BaselineCache(directory=cache.directory, engine_key="a-later-engine-build")
    _, source = _measure(other, probe)
    assert source == "measured"


def test_a_changed_observation_version_misses(cache: BaselineCache, probe: Env) -> None:
    """The loaded binary may be older than the sources beside it."""
    _measure(cache, probe)
    _, source = _measure(cache, probe, schema_version=probe.observation_version + 1)
    assert source == "measured"


def test_a_changed_objective_misses(cache: BaselineCache, probe: Env) -> None:
    """The reading is weighted, so the weighting reaches the stored return."""
    _measure(cache, probe)
    other = Weighting(terms={"held_tiles": 2.0}, won=100.0, lost=-10.0, drawn=0.0)
    _, source = _measure(cache, probe, scoring=other)
    assert source == "measured"


def test_a_corrupt_file_misses_rather_than_raising(
    cache: BaselineCache, probe: Env
) -> None:
    """A half-written file must read as no answer, not as an error."""
    measured, _ = _measure(cache, probe)
    assert cache.directory is not None
    stored = next(cache.directory.glob("*.json"))
    stored.write_text("{not json", encoding="utf-8")
    again, source = _measure(cache, probe)
    assert source == "measured"
    assert again == measured


def test_a_file_that_names_other_seeds_is_refused_whatever_its_name_says(
    cache: BaselineCache, probe: Env
) -> None:
    """The digest names the file. The comparison decides whether to trust it.

    A digest alone cannot rule out a wrong answer, because a file may be
    edited or a digest may collide. The read compares every input against the
    stored copy, and this test drives that comparison.
    """
    measured, _ = _measure(cache, probe)
    assert cache.directory is not None
    stored = next(cache.directory.glob("*.json"))
    payload = json.loads(stored.read_text(encoding="utf-8"))
    payload["seeds"] = [1, 2, 3]
    payload["summary"]["return"] = 999_999.0
    stored.write_text(json.dumps(payload), encoding="utf-8")
    again, source = _measure(cache, probe)
    assert source == "measured"
    assert again["return"] != 999_999.0
    assert again == measured


def test_a_cache_with_no_engine_key_stores_nothing(probe: Env, tmp_path: Path) -> None:
    """A number that cannot name its engine could answer for any engine."""
    cache = BaselineCache(directory=tmp_path, engine_key=None)
    assert cache.enabled is False
    _measure(cache, probe)
    _, source = _measure(cache, probe)
    assert source == "measured"
    assert list(tmp_path.glob("*.json")) == []


def test_a_waiting_process_gives_up_and_measures_rather_than_hanging(
    cache: BaselineCache, probe: Env
) -> None:
    """A process that dies holding the lock must not stop the run.

    The lock is taken here and never released, which is what a process that
    died leaves behind. The waiter reaches its deadline and measures.
    """
    patient = BaselineCache(
        directory=cache.directory,
        engine_key="an-engine-build",
        wait_seconds=0.0,
    )
    inputs = patient.inputs(WORLD, SCORING, SEEDS, probe.observation_version)
    assert inputs is not None
    assert patient.measuring(inputs) is True
    _, source = _measure(patient, probe)
    assert source == "measured"


def test_a_second_process_reads_what_the_first_one_wrote(
    cache: BaselineCache, probe: Env
) -> None:
    """The cache must answer across processes, not only inside one.

    The launcher starts one trainer for each strategy. The test builds a
    second cache over the same directory, which is what a second process
    holds.
    """
    measured, _ = _measure(cache, probe)
    second = BaselineCache(directory=cache.directory, engine_key="an-engine-build")
    cached, source = _measure(second, probe)
    assert source == "cached"
    assert cached == measured


def test_the_world_must_give_the_seat_to_the_built_in_controller(
    cache: BaselineCache, probe: Env
) -> None:
    """A world the learner holds plays a policy the key does not name."""
    with pytest.raises(ValueError, match="built-in controller"):
        _measure(cache, probe, config=replace(WORLD, controlled=True))


def test_two_requests_that_differ_in_one_input_take_two_keys() -> None:
    """The digest must separate two requests that differ anywhere."""
    one = {"engine": "a", "seeds": [1, 2], "world": {"width": 4}}
    two = {"engine": "a", "seeds": [1, 3], "world": {"width": 4}}
    assert key_of(one) != key_of(two)
    assert key_of(one) == key_of(dict(reversed(list(one.items()))))


def test_the_measurement_prints_a_progress_line(
    cache: BaselineCache,
    probe: Env,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A silent pass of nine minutes reads as a stopped process.

    The heartbeat interval is set to nothing here, so the first decision
    prints. The real interval is thirty seconds, and a test that waited for
    it would assert on the clock.
    """
    monkeypatch.setattr("cachette.learn.rollout.HEARTBEAT_SECONDS", 0.0)
    _measure(cache, probe)
    printed = capsys.readouterr().out
    assert "test baseline working" in printed
    assert " t/s " in printed


def test_the_dashboard_renders_the_progress_line(
    cache: BaselineCache,
    probe: Env,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A heartbeat the dashboard drops on the floor is not a heartbeat.

    The log the dashboard reads is the stdout of the trainer, and that seam
    has no other guard. The line here is the line the trainer wrote.
    """
    monkeypatch.setattr("cachette.learn.rollout.HEARTBEAT_SECONDS", 0.0)
    _measure(cache, probe)
    printed = capsys.readouterr().out
    line = next(row for row in printed.splitlines() if "baseline working" in row)
    watch = _load("train_watch")
    reading = watch.read(line.replace("test baseline", "conquer baseline"))
    assert "conquer" in reading.strategies
    rendered = watch.render(reading, watch.Facts(heading="a run", generations=60))
    assert "baseline 0% of " in rendered
    assert "no word yet" not in rendered


def test_a_waiter_that_gives_up_and_works_stops_reading_as_idle() -> None:
    """The last word decides, so a working line must replace a waiting one."""
    watch = _load("train_watch")
    log = (
        "  land-net baseline waiting 60s for another process\n"
        "  land-net baseline working  decisions 12 live 8/16 ticks 400 "
        "rate 91.1 t/s [31s]\n"
    )
    rendered = watch.render(watch.read(log), watch.Facts(heading="a run"))
    assert "baseline 50% of 16 worlds 91t/s d12 [31s]" in rendered
    assert "waiting on another process" not in rendered


def test_the_dashboard_renders_a_waiting_process() -> None:
    """A process that waits must not read as a process that stopped."""
    watch = _load("train_watch")
    line = (
        "  land-net baseline waiting 60s for another process to measure the same number"
    )
    rendered = watch.render(watch.read(line), watch.Facts(heading="a run"))
    assert "waiting on another process" in rendered
