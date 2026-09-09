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
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Callable, Sequence

    from .env import EnvConfig
    from .policy import Policy
    from .reward import Scoring

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


def controller_baseline(
    config: EnvConfig,
    scoring: Scoring,
    policy: Policy,
    seeds: Sequence[int],
    workers: int,
    schema_version: int,
    label: str,
    cache: BaselineCache | None = None,
) -> tuple[dict[str, float], str]:
    """Return the controller baseline, and say where the number came from.

    The second entry of the result is ``cached`` for a number a previous pass
    measured, and ``measured`` for one this call measured. A caller prints it,
    so a reader of a log knows which pass paid for the figure.

    The label names the pass in the progress line, for example
    ``conquer baseline``. A measurement prints that line while it runs, so a
    dashboard can tell a working pass from a stopped one. A cached answer
    prints nothing, because it takes no time.

    **The world must be the one that gives the seat to the built-in
    controller.** A world that the learner holds plays the policy, and the
    policy is not part of the key, so the stored number would answer for a
    policy that never played. This refuses such a world rather than storing a
    number nobody can trust.
    """
    if config.controlled:
        message = (
            "the controller baseline plays a world whose seat the built-in "
            "controller holds, and this world gives the seat to the learner"
        )
        raise ValueError(message)
    from .train import evaluate

    def measure() -> dict[str, float]:
        return evaluate(config, scoring, policy, list(seeds), workers, label=label)

    held = cache if cache is not None else BaselineCache.of_environment()
    inputs = held.inputs(config, scoring, seeds, schema_version)
    if inputs is None:
        return measure(), "measured"

    found = held.read(inputs)
    if found is not None:
        return found, "cached"

    if not held.measuring(inputs):

        def say(waited: float) -> None:
            print(
                f"  {label} waiting {waited:.0f}s for another process to "
                f"measure the same number",
                flush=True,
            )

        waited = held.wait(inputs, say)
        if waited is not None:
            return waited, "cached"
        # The other process gave up, died, or is slower than the deadline.
        # Measuring here costs one duplicated pass, which is what the
        # project pays today, and it cannot hang.
        return measure(), "measured"

    try:
        summary = measure()
        held.write(inputs, summary)
    finally:
        held.release(inputs)
    return summary, "measured"


__all__ = [
    "DEFAULT_CACHE",
    "WAIT_SECONDS",
    "BaselineCache",
    "controller_baseline",
    "engine_key",
    "key_of",
]
