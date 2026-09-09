"""Send the lines of one strategy to the file that a reader opens by name.

A run trains several strategies at once in one process, and every one of them
prints to the standard output. The combined log holds every line, and a reader
who wants one strategy wants a file of its own. The launcher wrote those files
by starting one process for each strategy and copying its output, and a run
that holds every strategy in one process writes them here instead.

**A line goes to the standard output whatever else happens to it.** The
dashboard reads the combined log, and it reads the strategy name inside each
line, so the routing here adds a file and removes nothing.

The route reads the thread that printed the line. One thread drives one
strategy, so the thread identity is the strategy identity, and a line needs no
prefix and no argument to find its file.
"""

from __future__ import annotations

import sys
import threading
from contextlib import contextmanager
from typing import TYPE_CHECKING, Any, TextIO

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Iterator
    from pathlib import Path


class ThreadJournal:
    """A text stream that copies the lines of a thread into its own file.

    Every write reaches the stream this wraps. A write from a thread that
    registered a file reaches that file as well.

    **A line reaches the stream whole, or it does not reach it yet.** The
    print function writes the text and the line break in two calls, so two
    threads that printed at once would put the text of one inside the line of
    the other. A reader of the log matches a line by shape, and a spliced
    line matches nothing. This holds the text of each thread until a line
    break arrives, and writes whole lines under one lock.

    The route reads the thread that wrote. One thread drives one strategy, so
    the buffer of a thread holds one strategy and never two.
    """

    def __init__(self, stream: TextIO) -> None:
        """Wrap one text stream, with no thread registered."""
        self._stream = stream
        self._files: dict[int, TextIO] = {}
        self._held: dict[int, str] = {}
        self._lock = threading.Lock()

    def register(self, file: TextIO) -> None:
        """Copy the lines of the calling thread into this file."""
        self._files[threading.get_ident()] = file

    def forget(self) -> None:
        """Write what this thread held, and stop copying its lines."""
        thread = threading.get_ident()
        rest = self._held.pop(thread, "")
        if rest:
            self._emit(rest + "\n", self._files.get(thread))
        self._files.pop(thread, None)

    def write(self, text: str) -> int:
        """Hold the text of this thread, and write the lines it completes."""
        thread = threading.get_ident()
        held = self._held.get(thread, "") + text
        cut = held.rfind("\n")
        if cut >= 0:
            self._emit(held[: cut + 1], self._files.get(thread))
        self._held[thread] = held[cut + 1 :]
        return len(text)

    def flush(self) -> None:
        """Flush the wrapped stream, and the file of this thread."""
        with self._lock:
            self._stream.flush()
            file = self._files.get(threading.get_ident())
            if file is not None:
                file.flush()

    def _emit(self, lines: str, file: TextIO | None) -> None:
        """Write whole lines to the stream, and to the file of a thread."""
        with self._lock:
            self._stream.write(lines)
            if file is not None:
                file.write(lines)

    def __getattr__(self, name: str) -> Any:  # noqa: ANN401 - a stream is untyped
        """Answer every other stream question from the wrapped stream."""
        return getattr(self._stream, name)


@contextmanager
def strategy_logs() -> Iterator[ThreadJournal]:
    """Route the standard output through a journal, and put it back after.

    A caller that trains one strategy needs none of this. A caller that
    trains several in one process opens this once and registers one file for
    each strategy thread.
    """
    held = sys.stdout
    journal = ThreadJournal(held)
    sys.stdout = journal
    try:
        yield journal
    finally:
        sys.stdout = held


@contextmanager
def strategy_log(journal: ThreadJournal | None, path: Path) -> Iterator[None]:
    """Copy the lines of this thread into one file, and close it after.

    The file is appended to, because a strategy that resumes continues the
    log of the run it continues.
    """
    if journal is None:
        yield
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as file:
        journal.register(file)
        try:
            yield
        finally:
            journal.forget()


__all__ = ["ThreadJournal", "strategy_log", "strategy_logs"]
