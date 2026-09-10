"""Measure the controller baseline once, and read it back every time after.

# What the baseline is, and why it is worth keeping

A training run reports every policy against the built-in controller playing
the learner's own seat. That number is the bar the run must beat, and the run
measures it by playing the controller over the whole held-out seed set.

**The controller's play is a function of four things and nothing else**: the
engine build, the world it plays, the seed set, and the objective the reading
is weighted by. It holds no weights that a run trains, and it draws nothing
from a run. Two runs that agree on those four compute the same number twice.

That number is expensive. One run on a rented machine of sixty four cores
paid over nine minutes for it before the first generation started, and it
paid for it once in every process the run started.

# The key states every input, and the file states them again

The key below is a digest of the four inputs. The stored file also holds each
input in full, and a read compares them one by one before it trusts the
number. **A cache that answers for a different seed set reports a wrong
number with the authority of a measurement**, and a digest alone cannot rule
that out. The comparison can, and it is the check that fails when two
declarations of one fact disagree.

A seed set that holds a subset or a superset of a stored one is a different
seed set. It gets a different key, it fails the comparison, and it misses. A
mean over some of the worlds is a different quantity from a mean over all of
them, so there is nothing here to reuse.

An absent file misses. A file that does not parse misses. A file that parses
and disagrees with the request misses. Every miss measures.

# The engine key is derived in one place

The key of the engine build comes from the shell script that the remote
runner already uses for its wheel cache.[^1] This module runs that script and
does not derive the key itself, because two derivations of one key would
serve a stale baseline after an engine change and nothing would fail.

A machine with no git repository cannot run that script. The remote runner
has no repository, so the launcher passes the value it derived in the
``CACHETTE_ENGINE_KEY`` variable and this module reads that first. A machine
with neither takes no key, and a run with no key measures every time.

# The observation schema version sits beside the build key, not instead of it

The build key answers for the sources. The schema version answers for the
binary that is loaded. They catch different mistakes, so the key holds both.

The build key covers every rule of the engine, and the schema version covers
only the layout of the observation. A change to a movement rule moves the
baseline and leaves the schema version alone, so the schema version alone
would serve a stale number.

The build key comes from the tree, and the tree is not always what is
installed. A development machine holds a compiled extension that a previous
build produced, and it can be older than the sources beside it. The schema
version comes from the loaded extension, so it catches the part of that
mismatch a reader of the tree cannot see.

# Six processes that miss at once

One run starts one trainer process for each strategy, within a second of each
other. On a cold cache several of them ask for the same number.

The first to arrive takes a lock and measures. The others wait for the file,
and they say so while they wait. **A waiter never waits for ever.** It gives
up when the deadline passes or when the lock is older than the deadline, and
then it measures the number itself. The worst case is what the project does
today, so the fallback cannot make a run slower than no cache at all.

A measurement is written to a temporary file and renamed into place. Rename
is atomic on one filesystem, so no reader ever sees half a file.

# Six objectives that miss at once share one set of games

A baseline return separates into two things. The episodes are the games the
controller plays, and they come from the engine build, the world and the seed
set. The objective weights the readings of those games into one number.

A pass that misses on several objectives therefore plays one batch and scores
it once for each objective. It never plays the same games twice. Two
objectives that state the same weighting share one measurement as well: the
first of them measures and writes, and the second reads what the first wrote.

# References

[^1]: The engine build key. ``scripts/build-key.sh``
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import time
from dataclasses import asdict, is_dataclass
from pathlib import Path
from threading import Lock
from typing import TYPE_CHECKING, Any, cast

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Callable, Mapping, Sequence

    from .env import EnvConfig
    from .measure import MeasurementPass
    from .policy import Policy
    from .record import PopulationRecord
    from .reward import Scoring
    from .shard import ShardPool

# Where a measurement is kept. The wheel cache of the remote runner keeps its
# files under the same root, so this follows that convention rather than
# opening a second one. The directory sits outside the tree, so nothing here
# can be committed by accident.
DEFAULT_CACHE = Path.home() / ".cache" / "cachette-baselines"

# How long a process waits for another process that is measuring the same
# number. A pass over the held-out seeds took over nine minutes on a rented
# machine, so the wait must be longer than that. A waiter that reaches this
# measures the number itself.
WAIT_SECONDS = 1800.0

# How often a waiting process says that it is still waiting. The dashboard
# reads a line to tell a working process from a stopped one, so a process
# that says nothing for half an hour looks stopped.
WAIT_HEARTBEAT_SECONDS = 30.0

# How often a waiter looks for the file the measuring process will write.
POLL_SECONDS = 2.0


class BaselineCache:
    """Read and write one measured baseline, keyed on everything it depends on.

    A caller builds one of these for a run and asks it for the baseline. The
    cache measures the number when it does not hold it, and it returns the
    stored number when it does.

    A cache with no directory is disabled. It answers every request by
    measuring, and it writes nothing. That is the behaviour a run with no
    engine key gets, because a stored number that cannot name its engine
    could answer for any engine at all.
    """

    def __init__(
        self,
        directory: Path | None = None,
        engine_key: str | None = None,
        wait_seconds: float = WAIT_SECONDS,
    ) -> None:
        """Build the cache over one directory, for one engine build."""
        self._directory = directory
        self._engine_key = engine_key
        self._wait_seconds = wait_seconds

    @classmethod
    def of_environment(cls) -> BaselineCache:
        """Build the cache the environment describes.

        ``CACHETTE_BASELINE_CACHE`` names the directory, and an empty value
        turns the cache off. ``CACHETTE_ENGINE_KEY`` names the engine build,
        and an absent value asks the shell script for it.
        """
        named = os.environ.get("CACHETTE_BASELINE_CACHE")
        if named is not None and not named.strip():
            return cls(directory=None)
        directory = Path(named) if named else DEFAULT_CACHE
        return cls(directory=directory, engine_key=engine_key())

    @property
    def directory(self) -> Path | None:
        """Where this cache keeps its files, or nothing when it keeps none."""
        return self._directory

    @property
    def enabled(self) -> bool:
        """Whether this cache stores anything.

        A cache with no directory or no engine key stores nothing. The engine
        key is required because a stored number that cannot name the engine
        that produced it would answer for an engine that never played.
        """
        return self._directory is not None and bool(self._engine_key)

    def inputs(
        self,
        config: EnvConfig,
        scoring: Scoring,
        seeds: Sequence[int],
        schema_version: int,
    ) -> dict[str, Any] | None:
        """Return every input the baseline depends on, or nothing.

        The result is ``None`` when one input cannot be stated as data. A
        scoring that is not a dataclass is such an input: this module cannot
        say when two of them are the same, so a run under one measures every
        time rather than risk answering under another.
        """
        if not self.enabled:
            return None
        described = _describe(scoring)
        if described is None:
            return None
        return {
            "engine": self._engine_key,
            "observation_version": int(schema_version),
            "world": asdict(config),
            "seeds": [int(seed) for seed in seeds],
            "scoring": described,
        }

    def path_of(self, inputs: dict[str, Any]) -> Path:
        """Return the file that holds the measurement of these inputs."""
        assert self._directory is not None
        return self._directory / f"{key_of(inputs)}.json"

    def read(self, inputs: dict[str, Any]) -> dict[str, float] | None:
        """Return the stored measurement of these inputs, or nothing.

        Every input is compared against the stored copy. A file that names
        another seed set, another world, another engine or another objective
        is not an answer to this request, whatever its name says.
        """
        path = self.path_of(inputs)
        try:
            stored = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return None
        if not isinstance(stored, dict):
            return None
        for name, value in inputs.items():
            if stored.get(name) != value:
                return None
        summary = stored.get("summary")
        if not isinstance(summary, dict) or "return" not in summary:
            return None
        return {name: float(value) for name, value in summary.items()}

    def write(self, inputs: dict[str, Any], summary: dict[str, float]) -> None:
        """Store one measurement, so that no later run pays for it again.

        The file is written beside its target and renamed onto it. Rename is
        atomic on one filesystem, so a reader sees the whole file or no file.
        """
        path = self.path_of(inputs)
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(f"{path.name}.{os.getpid()}.part")
        payload = {**inputs, "summary": summary, "measured_at": time.time()}
        temporary.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        os.replace(temporary, path)

    def measuring(self, inputs: dict[str, Any]) -> bool:
        """Take the right to measure these inputs, and say whether it was free.

        A process that gets ``True`` measures and writes. A process that gets
        ``False`` waits for the process that got ``True``.
        """
        path = self._lock_of(inputs)
        path.parent.mkdir(parents=True, exist_ok=True)
        try:
            handle = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        except FileExistsError:
            return False
        with os.fdopen(handle, "w", encoding="utf-8") as file:
            file.write(f"{os.getpid()} {time.time()}\n")
        return True

    def release(self, inputs: dict[str, Any]) -> None:
        """Give up the right to measure, whether the measurement worked or not.

        A process that died without this leaves the lock behind, and the
        waiters fall back to measuring when the lock passes the deadline.
        """
        try:
            self._lock_of(inputs).unlink()
        except OSError:
            pass

    def wait(
        self,
        inputs: dict[str, Any],
        say: Callable[[float], None] | None = None,
    ) -> dict[str, float] | None:
        """Wait for another process to write these inputs, and read them.

        Returns the stored measurement, or ``None`` when the wait ended
        without one. **The caller then measures the number itself**, so a
        process that died holding the lock costs one duplicated measurement
        and never a run that hangs.

        The say entry is called with one line each time the wait wants to
        report itself, so that a dashboard can tell a waiting process from a
        stopped one.
        """
        started = time.monotonic()
        spoke = started
        while True:
            found = self.read(inputs)
            if found is not None:
                return found
            waited = time.monotonic() - started
            if waited >= self._wait_seconds:
                return None
            if self._lock_is_abandoned(inputs):
                return None
            now = time.monotonic()
            if say is not None and now - spoke >= WAIT_HEARTBEAT_SECONDS:
                spoke = now
                say(waited)
            time.sleep(POLL_SECONDS)

    def _lock_of(self, inputs: dict[str, Any]) -> Path:
        """Return the lock that names the right to measure these inputs."""
        return self.path_of(inputs).with_suffix(".lock")

    def _lock_is_abandoned(self, inputs: dict[str, Any]) -> bool:
        """Whether the lock has stood longer than a measurement may take."""
        try:
            held = time.time() - self._lock_of(inputs).stat().st_mtime
        except OSError:
            return True
        return held > self._wait_seconds


def key_of(inputs: dict[str, Any]) -> str:
    """Return the digest of a set of inputs.

    The digest names the file. It is not the check: a read compares every
    input against the stored copy, so the digest only has to spread the
    files apart.
    """
    text = json.dumps(inputs, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:32]


def engine_key() -> str | None:
    """Return the key of the engine build, or nothing when it cannot be had.

    The value of ``CACHETTE_ENGINE_KEY`` answers first, because a machine
    that unpacked the sources from an archive holds no repository to derive
    it from and the launcher passes it in.

    **This module derives no key of its own.** It runs the script that the
    wheel cache of the remote runner already reads, so the two cannot part
    company after an engine change.
    """
    passed = os.environ.get("CACHETTE_ENGINE_KEY")
    if passed and passed.strip():
        return passed.strip()
    script = Path(__file__).resolve().parents[3] / "scripts" / "build-key.sh"
    if not script.exists():
        return None
    try:
        found = subprocess.run(
            ["/usr/bin/env", "bash", str(script)],
            capture_output=True,
            text=True,
            timeout=60,
            check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if found.returncode != 0:
        return None
    return found.stdout.strip() or None


def _describe(scoring: Scoring) -> dict[str, Any] | None:
    """Return the objective as plain data, or nothing when it has no such form.

    A weighting and an objective scoring are both frozen dataclasses, so both
    state themselves. Anything else states nothing this module can compare,
    and a request under it is answered by measuring.
    """
    if not is_dataclass(scoring) or isinstance(scoring, type):
        return None
    try:
        return {
            "type": type(scoring).__name__,
            "fields": json.loads(json.dumps(asdict(scoring), sort_keys=True)),
        }
    except (TypeError, ValueError):
        return None


def available_workers() -> int:
    """Return how many engine workers this machine can give one pass.

    The answer comes from the machine and never from a constant. A constant
    written here would hold a pass to a tenth of a rented machine of sixty
    four cores, and nothing would fail.

    The affinity set answers first, because a container or a batch scheduler
    may give a process fewer cores than the machine holds. A platform that
    reports no affinity set falls back to the core count.
    """
    if hasattr(os, "sched_getaffinity"):
        return max(1, len(os.sched_getaffinity(0)))
    return max(1, os.cpu_count() or 1)


def controller_baselines(
    config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policy: Policy,
    seeds: Sequence[int],
    workers: int,
    schema_version: int,
    label: str,
    cache: BaselineCache | None = None,
    pool: ShardPool | None = None,
) -> dict[str, tuple[dict[str, float], str]]:
    """Return the controller baseline of each objective, and wait for it.

    This starts the pass and reads it at once. A caller whose own work does
    not depend on the answer starts the pass itself and reads it later, and
    the queue then holds the baseline beside whatever else the run plays.
    """
    return start_controller_baselines(
        config,
        scorings,
        policy,
        seeds,
        workers,
        schema_version,
        label,
        cache,
        pool,
    ).results()


def start_controller_baselines(
    config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policy: Policy,
    seeds: Sequence[int],
    workers: int,
    schema_version: int,
    label: str,
    cache: BaselineCache | None = None,
    pool: ShardPool | None = None,
) -> BaselinePass:
    """Start the controller baseline of each objective, and return before it ends.

    The pass reads the cache now, and it submits only the objectives the
    cache does not hold. **A caller that finds every number stored therefore
    enqueues nothing**, and the pass answers at once.

    The result of the pass holds one entry for each name the caller gave,
    under that name. Each entry holds the summary and the word ``cached`` or
    ``measured``, so a reader of a log knows which pass paid for the figure.

    **One set of episodes answers for every objective.** The episodes are the
    games the seat plays, and they come from the world, the seed set and the
    engine build. The objective weights the readings of those games. A pass
    that missed on four objectives therefore plays one batch and scores it
    four times, rather than playing the same games four times over.

    Two objectives that state the same weighting share one measurement. The
    first of them measures, and the second reads what the first wrote, which
    is the behaviour a run had before this pass existed.

    The label names the pass in the progress line. A measurement prints that
    line while it runs, so a dashboard can tell a working pass from a stopped
    one.

    **The world must be the one that gives the seat to the built-in
    controller.** A world that the learner holds plays the policy, and the
    policy is not part of the key, so a stored number would answer for a
    policy that never played. This refuses such a world rather than storing a
    number nobody can trust.

    The pool entry is the queue the pass plays its episodes in. **One episode
    is one task of it**, so a pass over 256 seeds fills a machine rather than
    reaching about a seventh of it. A pass that holds no pool steps one batch
    of worlds in this process.
    """
    if config.controlled:
        message = (
            "the controller baseline plays a world whose seat the built-in "
            "controller holds, and this world gives the seat to the learner"
        )
        raise ValueError(message)
    if not scorings:
        message = "a baseline pass measures at least one objective"
        raise ValueError(message)

    held = cache if cache is not None else BaselineCache.of_environment()
    inputs = {
        name: held.inputs(config, scoring, seeds, schema_version)
        for name, scoring in scorings.items()
    }
    groups = _groups(scorings)

    results: dict[str, tuple[dict[str, float], str]] = {}
    measuring: dict[str, Scoring] = {}
    owned: set[str] = set()
    for first, members in groups.items():
        request = inputs[first]
        stored = None if request is None else held.read(request)
        if stored is None and request is not None:
            if held.measuring(request):
                owned.add(first)
            else:
                # Another process holds the right to measure this number.
                # **A waiter never waits for ever.** It gives up when the
                # deadline passes or when the lock is older than the
                # deadline, and this pass then measures the number itself.
                # The worst case is what the project paid before the cache
                # existed, so the fallback cannot make a run slower.
                stored = held.wait(request, _waiting(label, first))
        if stored is not None:
            for name in members:
                results[name] = (stored, "cached")
            continue
        measuring[first] = scorings[first]

    return BaselinePass(
        config=config,
        scorings=scorings,
        policy=policy,
        seeds=list(seeds),
        workers=workers,
        label=label,
        cache=held,
        inputs=inputs,
        groups=groups,
        cached=results,
        measuring=measuring,
        owned=owned,
        pool=pool,
    )


class BaselinePass:
    """The controller baseline of each objective, and where each one came from.

    **A caller may start this pass and read it later.** The bar is a
    reporting quantity: a run ranks its candidates against each other by win
    share, and nothing that trains a weight reads a baseline. A run that
    waited for this pass therefore spent minutes with an empty queue before
    its first generation, and it spent them for a number no generation needs.

    A pass whose numbers the cache holds is already finished. It submits
    nothing, and it answers at once.

    A pass with no pool plays its episodes when it starts, in the process
    that started it. That is what a run of one core does, and it is what
    every run did before the queue existed.
    """

    def __init__(
        self,
        config: EnvConfig,
        scorings: Mapping[str, Scoring],
        policy: Policy,
        seeds: list[int],
        workers: int,
        label: str,
        cache: BaselineCache,
        inputs: Mapping[str, dict[str, Any] | None],
        groups: Mapping[str, list[str]],
        cached: Mapping[str, tuple[dict[str, float], str]],
        measuring: Mapping[str, Scoring],
        owned: set[str],
        pool: ShardPool | None,
    ) -> None:
        """Resolve what the cache holds, and start what it does not.

        The pass takes the right to measure each number it owns, so it holds
        that right until it gives its results back. A second process that
        wants the same number waits for the file this pass will write.
        """
        from .measure import start_measurement
        from .rollout import run_objectives

        self._scorings = scorings
        self._cache = cache
        self._inputs = inputs
        self._groups = groups
        self._results = dict(cached)
        self._measuring = dict(measuring)
        self._owned = owned
        self._label = label
        self._pending: object | None = None
        self._played: dict[str, PopulationRecord] | None = None
        # **Several strategies of one run read one pass.** Each of them runs
        # in a thread of the trainer process and asks for the bar when its
        # own training ends. The first one waits and writes the cache, and
        # the others read what it kept, so the games are played once.
        self._lock = Lock()
        if not self._measuring:
            self._release()
            return
        if pool is None:
            try:
                self._played = run_objectives(
                    config, self._measuring, [policy], seeds, workers, label
                )
            except BaseException:
                self._release()
                raise
        else:
            self._pending = start_measurement(
                pool, config, self._measuring, [policy], seeds
            )

    @property
    def measuring(self) -> tuple[str, ...]:
        """The objectives this pass measures, in the order the caller gave."""
        return tuple(self._measuring)

    @property
    def done(self) -> bool:
        """Whether every episode of this pass has finished.

        A pass that the cache answered is done at once, and so is a pass that
        played its episodes when it started.
        """
        if self._pending is None:
            return True
        pending = cast("MeasurementPass", self._pending)
        return pending.done

    def results(self) -> dict[str, tuple[dict[str, float], str]]:
        """Wait for the episodes, and return the baseline of each objective.

        The result holds one entry for each name the caller gave, under that
        name. Each entry holds the summary and the word ``cached`` or
        ``measured``, so a reader of a log knows which pass paid for the
        figure.

        **Two callers may read one pass.** The first one waits, writes the
        cache and keeps the answer. Every caller after it reads what the
        first kept, so a run of several strategies plays one set of games.
        """
        from .train import summarise

        with self._lock:
            if self._measuring:
                try:
                    played = self._take()
                    for first in self._measuring:
                        summary = summarise([played[first]])
                        request = self._inputs[first]
                        if first in self._owned and request is not None:
                            self._cache.write(request, summary)
                        self._results[first] = (summary, "measured")
                        for name in self._groups[first][1:]:
                            self._results[name] = _answered(
                                self._cache, self._inputs[name], summary
                            )
                finally:
                    self._release()
                    self._measuring = {}
            return {name: self._results[name] for name in self._scorings}

    def _take(self) -> dict[str, PopulationRecord]:
        """Return the records of this pass, waiting for the queue if it holds them."""
        if self._played is not None:
            return self._played
        pending = cast("MeasurementPass", self._pending)
        self._played = {
            name: held[0] for name, held in pending.records(self._label).items()
        }
        self._pending = None
        return self._played

    def _release(self) -> None:
        """Give up the right to measure each number this pass owns."""
        for first in self._owned:
            request = self._inputs[first]
            if request is not None:
                self._cache.release(request)
        self._owned = set()


def controller_baseline(
    config: EnvConfig,
    scoring: Scoring,
    policy: Policy,
    seeds: Sequence[int],
    workers: int,
    schema_version: int,
    label: str,
    cache: BaselineCache | None = None,
    pool: ShardPool | None = None,
) -> tuple[dict[str, float], str]:
    """Return the controller baseline of one objective, and where it came from.

    The second entry of the result is ``cached`` for a number a previous pass
    measured, and ``measured`` for one this call measured. A caller prints it,
    so a reader of a log knows which pass paid for the figure.

    The label names the pass in the progress line, for example
    ``conquer baseline``. A measurement prints that line while it runs, so a
    dashboard can tell a working pass from a stopped one. A cached answer
    prints nothing, because it takes no time.

    **This asks for one objective through the pass that asks for several.** A
    second path for one objective would be one rule stored twice, and the
    caller that wanted six numbers would drift away from the caller that
    wanted one. The label names the one objective there, so a waiting line
    still says which number the pass waits for.
    """
    found = controller_baselines(
        config,
        {label: scoring},
        policy,
        seeds,
        workers,
        schema_version,
        label,
        cache,
        pool,
    )
    return found[label]


def _groups(scorings: Mapping[str, Scoring]) -> dict[str, list[str]]:
    """Group the names that state the same objective, keyed on the first name.

    Two objectives that describe themselves the same way give the same
    reading of the same episodes, so one measurement answers for both. The
    cache key holds the same description, so this grouping agrees with the
    cache by construction rather than by a second rule.

    An objective that cannot state itself as data gets a group of its own,
    because nothing here can say that two of them are the same.

    **The result keeps the order the caller gave.** The order of a combined
    result therefore comes from a key the caller stated, and never from which
    world or worker finished first.
    """
    grouped: dict[object, str] = {}
    members: dict[str, list[str]] = {}
    for name, scoring in scorings.items():
        described = _describe(scoring)
        key: object = (
            (False, name)
            if described is None
            else (True, json.dumps(described, sort_keys=True))
        )
        first = grouped.get(key)
        if first is None:
            grouped[key] = name
            members[name] = [name]
        else:
            members[first].append(name)
    return members


def _waiting(label: str, name: str) -> Callable[[float], None]:
    """Return the line a waiting pass prints while it waits.

    A dashboard tells a working process from a stopped one by the age of its
    last line, so a wait that can run for half an hour must give it one.
    """

    def say(waited: float) -> None:
        print(
            f"  {label} waiting {waited:.0f}s for another process to measure {name}",
            flush=True,
        )

    return say


def _answered(
    held: BaselineCache,
    request: dict[str, Any] | None,
    summary: dict[str, float],
) -> tuple[dict[str, float], str]:
    """Say where the answer for one objective came from, and give it.

    A name whose number this pass wrote reads it back from the file, so it
    reports ``cached`` in the way a later process would. A name the cache
    cannot store reports ``measured``, because nothing stored it.
    """
    if request is not None:
        found = held.read(request)
        if found is not None:
            return found, "cached"
    return summary, "measured"


__all__ = [
    "DEFAULT_CACHE",
    "WAIT_SECONDS",
    "BaselineCache",
    "BaselinePass",
    "available_workers",
    "controller_baseline",
    "controller_baselines",
    "engine_key",
    "key_of",
    "start_controller_baselines",
]
