"""Start a generation run in the background, and report what it is doing.

One round of four variants takes about seventy seconds, and a whole asset
set is eleven subjects. A request must therefore never wait for a run. This
module starts the tool as a child process, returns at once, and records the
state of the work in a JSON file that any page can read again.

The module calls the tool through its command line. It imports nothing from
the tool package, so a change inside the tool cannot break the server.[^1]

## One job, one style, many subjects

A job holds one style and a list of subjects. The worker runs the subjects
one after another, because the endpoint is slow and one machine carries the
whole load. One worker thread serves every job in the order that the person
started them.

## What happens when the server dies

The child process runs in its own session, so it survives the server. The
supervisor thread does not. A job record therefore holds the identifier of
the process that supervises it. A reader that finds a running job with
another supervisor reports the job as abandoned, because nothing is left to
write the end of it.

The drawings are safe in that case. The tool writes every file to a
temporary name and renames it, so the session directory holds whole files
only. A person restarts the subject that was in flight and keeps the rest.

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import os
import queue
import subprocess
import sys
import threading
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path

from store import read_json, safe_name, write_json_atomically

# The states of one subject inside a job.
ITEM_STATES = ("queued", "running", "done", "failed", "stopped")

# The states of a whole job.
JOB_STATES = ("queued", "running", "done", "failed", "stopped")


def now_iso() -> str:
    """Give the present time as an ISO 8601 string in UTC."""
    return datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def stamp() -> str:
    """Give a time stamp that sorts by time, as a directory name does."""
    return datetime.now(UTC).strftime("%Y%m%d-%H%M%S")


@dataclass(frozen=True)
class Item:
    """One subject of one job."""

    slug: str | None
    subject: str
    session_id: str
    state: str = "queued"
    started: str | None = None
    finished: str | None = None
    returncode: int | None = None
    log: str = ""

    @property
    def name(self) -> str:
        """Give the name to show for this subject."""
        return self.slug or self.subject

    def as_json(self) -> dict:
        """Give the record that goes on disk."""
        return {
            "slug": self.slug,
            "subject": self.subject,
            "session": self.session_id,
            "state": self.state,
            "started": self.started,
            "finished": self.finished,
            "returncode": self.returncode,
            "log": self.log,
        }


@dataclass(frozen=True)
class Job:
    """One request to draw a list of subjects in one style."""

    job_id: str
    style: str
    rounds: int
    variants: int
    started: str
    supervisor: int
    state: str = "queued"
    finished: str | None = None
    stop_requested: bool = False
    items: list[Item] = field(default_factory=list)

    @property
    def state_now(self) -> str:
        """Give the state, and say when nothing supervises the job.

        A job that claims to run under another process is abandoned. The
        server that started it is gone, so no thread will ever end the job.
        """
        if self.state in ("queued", "running") and self.supervisor != os.getpid():
            return "abandoned"
        return self.state

    @property
    def active(self) -> bool:
        """Report whether this job still has work that will run."""
        return self.state_now in ("queued", "running")

    @property
    def done_count(self) -> int:
        """Count the subjects that finished, however they finished."""
        return len(
            [item for item in self.items if item.state in ("done", "failed", "stopped")]
        )

    def as_json(self) -> dict:
        """Give the record that goes on disk."""
        return {
            "job": self.job_id,
            "style": self.style,
            "rounds": self.rounds,
            "variants": self.variants,
            "started": self.started,
            "finished": self.finished,
            "state": self.state,
            "supervisor": self.supervisor,
            "stop_requested": self.stop_requested,
            "items": [item.as_json() for item in self.items],
        }


def job_from_json(value: dict) -> Job | None:
    """Read one job record, or give `None` when the record is unusable.

    The worker writes this file while a page reads it. A half-written or
    broken record becomes `None`, and the page shows a gap.
    """
    job_id = value.get("job")
    style = value.get("style")
    if not isinstance(job_id, str) or not isinstance(style, str):
        return None
    items = []
    raw_items = value.get("items")
    if isinstance(raw_items, list):
        for raw in raw_items:
            if not isinstance(raw, dict):
                continue
            session_id = raw.get("session")
            if not isinstance(session_id, str):
                continue
            items.append(
                Item(
                    slug=raw.get("slug") if isinstance(raw.get("slug"), str) else None,
                    subject=raw.get("subject")
                    if isinstance(raw.get("subject"), str)
                    else "",
                    session_id=session_id,
                    state=raw.get("state")
                    if raw.get("state") in ITEM_STATES
                    else "queued",
                    started=raw.get("started"),
                    finished=raw.get("finished"),
                    returncode=raw.get("returncode")
                    if isinstance(raw.get("returncode"), int)
                    else None,
                    log=raw.get("log") if isinstance(raw.get("log"), str) else "",
                )
            )
    return Job(
        job_id=job_id,
        style=style,
        rounds=value.get("rounds") if isinstance(value.get("rounds"), int) else 0,
        variants=value.get("variants") if isinstance(value.get("variants"), int) else 0,
        started=value.get("started") if isinstance(value.get("started"), str) else "",
        supervisor=value.get("supervisor")
        if isinstance(value.get("supervisor"), int)
        else -1,
        state=value.get("state") if value.get("state") in JOB_STATES else "queued",
        finished=value.get("finished"),
        stop_requested=bool(value.get("stop_requested")),
        items=items,
    )


def default_command(
    python: str, style: str, subject: str, session_id: str, rounds: int, variants: int
) -> list[str]:
    """Build the command line that draws one subject.

    The server calls the tool this way and no other way. The two halves of
    the tool share a directory layout, and nothing else.
    """
    return [
        python,
        "-m",
        "direct_die",
        "run",
        "--asset",
        style,
        "--subject",
        subject,
        "--session",
        session_id,
        "--rounds",
        str(rounds),
        "--variants",
        str(variants),
    ]


class RunManager:
    """Start jobs, run them one at a time, and record what they did."""

    def __init__(
        self,
        runs_root: Path,
        sessions_root: Path,
        tool_directory: Path,
        command: Callable[..., list[str]] = default_command,
        python: str | None = None,
    ) -> None:
        """Hold the directories and the command that a job runs."""
        self.runs_root = Path(runs_root)
        self.sessions_root = Path(sessions_root)
        self.tool_directory = Path(tool_directory)
        self.command = command
        self.python = python or sys.executable
        self._queue: queue.Queue[str] = queue.Queue()
        self._worker: threading.Thread | None = None
        self._lock = threading.Lock()
        self._children: dict[str, subprocess.Popen] = {}

    # -- reading -----------------------------------------------------------

    def job_directory(self, job_id: str) -> Path:
        """Give the directory of one job."""
        return self.runs_root / safe_name(job_id)

    def load(self, job_id: str) -> Job | None:
        """Read one job record, or give `None` when there is none."""
        value = read_json(self.job_directory(job_id) / "job.json")
        return job_from_json(value) if value is not None else None

    def jobs(self) -> list[Job]:
        """List every job, newest first. A broken record drops out."""
        try:
            entries = sorted(
                (entry for entry in self.runs_root.iterdir() if entry.is_dir()),
                key=lambda path: path.name,
                reverse=True,
            )
        except OSError:
            return []
        found = []
        for entry in entries:
            job = self.load(entry.name)
            if job is not None:
                found.append(job)
        return found

    def active_jobs(self) -> list[Job]:
        """List the jobs that still have work that will run."""
        return [job for job in self.jobs() if job.active]

    def running_item(self, style: str, slug: str) -> tuple[Job, Item] | None:
        """Find the job and item that draw this slug now, if one does."""
        for job in self.jobs():
            if job.style != style or not job.active:
                continue
            for item in job.items:
                if item.slug == slug and item.state in ("queued", "running"):
                    return job, item
        return None

    def item_for_session(self, style: str, session_id: str) -> tuple[Job, Item] | None:
        """Find the job and item that write one session now, if one does."""
        for job in self.jobs():
            if job.style != style or not job.active:
                continue
            for item in job.items:
                if item.session_id == session_id and item.state in (
                    "queued",
                    "running",
                ):
                    return job, item
        return None

    def log_tail(self, job_id: str, name: str, lines: int = 40) -> str:
        """Give the last lines of one subject log, or "" when it is absent."""
        try:
            safe_name(name)
        except ValueError:
            return ""
        path = self.job_directory(job_id) / name
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""
        return "\n".join(text.splitlines()[-lines:])

    # -- starting ----------------------------------------------------------

    def start_job(
        self,
        style: str,
        requests: list[tuple[str | None, str]],
        rounds: int,
        variants: int,
    ) -> Job:
        """Record a job, put it in the queue, and give the record.

        The call returns as soon as the record is on disk. The work starts
        on the worker thread, which may already carry another job.
        """
        safe_name(style)
        if not requests:
            raise ValueError("a job needs at least one subject")
        with self._lock:
            job_id = self._free_job_id(style)
            directory = self.runs_root / job_id
            directory.mkdir(parents=True, exist_ok=True)
            items = []
            for index, (slug, subject) in enumerate(requests):
                session_id = self._free_session_id(style, slug, index)
                items.append(
                    Item(
                        slug=slug,
                        subject=subject,
                        session_id=session_id,
                        log=f"{session_id}.log",
                    )
                )
            job = Job(
                job_id=job_id,
                style=style,
                rounds=rounds,
                variants=variants,
                started=now_iso(),
                supervisor=os.getpid(),
                state="queued",
                items=items,
            )
            self._write(job)
            self._ensure_worker()
        self._queue.put(job_id)
        return job

    def request_stop(self, job_id: str) -> None:
        """Ask a job to stop, and end the subject that runs now."""
        job = self.load(job_id)
        if job is None:
            return
        self._write(_replace_job(job, stop_requested=True))
        child = self._children.get(job_id)
        if child is not None and child.poll() is None:
            child.terminate()

    # -- the worker --------------------------------------------------------

    def _ensure_worker(self) -> None:
        if self._worker is not None and self._worker.is_alive():
            return
        self._worker = threading.Thread(
            target=self._serve, name="direct-die-runs", daemon=True
        )
        self._worker.start()

    def _serve(self) -> None:
        while True:
            job_id = self._queue.get()
            try:
                self._run_job(job_id)
            except Exception as error:  # the thread must not die
                job = self.load(job_id)
                if job is not None:
                    self._write(_replace_job(job, state="failed", finished=now_iso()))
                self._append_log(job_id, "job.log", f"the supervisor failed: {error}\n")
            finally:
                self._queue.task_done()

    def _run_job(self, job_id: str) -> None:
        job = self.load(job_id)
        if job is None:
            return
        job = _replace_job(job, state="running")
        self._write(job)
        for index in range(len(job.items)):
            current = self.load(job_id) or job
            if current.stop_requested:
                job = _replace_item(current, index, state="stopped")
                self._write(job)
                continue
            job = _replace_item(current, index, state="running", started=now_iso())
            self._write(job)
            code = self._run_item(job, job.items[index])
            current = self.load(job_id) or job
            state = "done" if code == 0 else "failed"
            if current.stop_requested and code != 0:
                state = "stopped"
            job = _replace_item(
                current, index, state=state, finished=now_iso(), returncode=code
            )
            self._write(job)
        final = self.load(job_id) or job
        states = {item.state for item in final.items}
        if final.stop_requested:
            end = "stopped"
        elif "failed" in states:
            end = "failed"
        else:
            end = "done"
        self._write(_replace_job(final, state=end, finished=now_iso()))

    def _run_item(self, job: Job, item: Item) -> int:
        command = self.command(
            self.python,
            job.style,
            item.subject,
            item.session_id,
            job.rounds,
            job.variants,
        )
        log_path = self.job_directory(job.job_id) / (item.log or "job.log")
        self._append_log(
            job.job_id,
            item.log or "job.log",
            f"{now_iso()} {' '.join(command)}\n",
        )
        try:
            with log_path.open("a", encoding="utf-8") as stream:
                child = subprocess.Popen(
                    command,
                    cwd=str(self.tool_directory),
                    stdout=stream,
                    stderr=subprocess.STDOUT,
                    stdin=subprocess.DEVNULL,
                    start_new_session=True,
                )
                self._children[job.job_id] = child
                return child.wait()
        except OSError as error:
            self._append_log(
                job.job_id,
                item.log or "job.log",
                f"the command did not start: {error}\n",
            )
            return -1
        finally:
            self._children.pop(job.job_id, None)

    # -- writing -----------------------------------------------------------

    def _write(self, job: Job) -> None:
        directory = self.job_directory(job.job_id)
        directory.mkdir(parents=True, exist_ok=True)
        write_json_atomically(directory / "job.json", job.as_json())

    def _append_log(self, job_id: str, name: str, text: str) -> None:
        directory = self.job_directory(job_id)
        directory.mkdir(parents=True, exist_ok=True)
        with (directory / name).open("a", encoding="utf-8") as stream:
            stream.write(text)

    def _free_job_id(self, style: str) -> str:
        base = f"{stamp()}-{style}"
        candidate = base
        counter = 2
        while (self.runs_root / candidate).exists():
            candidate = f"{base}-{counter}"
            counter += 1
        return candidate

    def _free_session_id(self, style: str, slug: str | None, offset: int) -> str:
        """Give a session identifier that names the slug it draws.

        The identifier carries the slug, so the matrix reads the subject of
        a session from the directory name. Nothing stores that name twice.
        """
        base = stamp()
        if offset:
            base = f"{base}-{offset:02d}"
        candidate = f"{base}-{slug}" if slug else base
        counter = 2
        while (self.sessions_root / style / candidate).exists():
            candidate = f"{base}-{counter}-{slug}" if slug else f"{base}-{counter}"
            counter += 1
        return candidate


def _replace_job(job: Job, **changes: object) -> Job:
    values = job.as_json()
    del values["items"]
    mapping = {
        "job_id": values["job"],
        "style": values["style"],
        "rounds": values["rounds"],
        "variants": values["variants"],
        "started": values["started"],
        "supervisor": values["supervisor"],
        "state": values["state"],
        "finished": values["finished"],
        "stop_requested": values["stop_requested"],
        "items": list(job.items),
    }
    mapping.update(changes)
    return Job(**mapping)  # type: ignore[arg-type]


def _replace_item(job: Job, index: int, **changes: object) -> Job:
    items = list(job.items)
    current = items[index]
    mapping = {
        "slug": current.slug,
        "subject": current.subject,
        "session_id": current.session_id,
        "state": current.state,
        "started": current.started,
        "finished": current.finished,
        "returncode": current.returncode,
        "log": current.log,
    }
    mapping.update(changes)
    items[index] = Item(**mapping)  # type: ignore[arg-type]
    return _replace_job(job, items=items)
