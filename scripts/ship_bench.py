#!/usr/bin/env python3
"""Load a benchmark result file into the local observability stack.

The benchmark writes a tab separated table and never opens a socket. This
script is the step that ships it. Keeping the two apart means a benchmark run
is reproducible with nothing listening, and a run that nobody ships is still a
complete record on disk.

The script reads the result file that `scripts/graviton-benchmark.sh` writes,
turns each row into a row of `bench.rows`, and posts them to ClickHouse over
HTTP. It also copies the run log into the directory that the collector reads,
so the log reaches Loki.

Usage:

    scripts/ship_bench.py RESULT_FILE [--log LOG_FILE]
    scripts/ship_bench.py RESULT_FILE --print

`--print` writes the rows and sends nothing, which is how a person checks the
parse without starting the stack.

The result file holds comment lines that start with a hash, one or more header
lines, and the rows under each header. A header names its own columns, so a
new column in the benchmark reaches this script without an edit here.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

CLICKHOUSE = os.environ.get("CACHETTE_CLICKHOUSE", "http://127.0.0.1:8123")
USER = os.environ.get("CACHETTE_CLICKHOUSE_USER", "cachette")
PASSWORD = os.environ.get("CACHETTE_CLICKHOUSE_PASSWORD", "cachette")
RUNS = Path(__file__).resolve().parent.parent / "observability" / "runs"

# The columns of bench.rows, and what each one defaults to. A row that a
# header does not mention takes the default, so a timing row and a memory row
# both fit the one table.
COLUMNS: dict[str, Any] = {
    "taken_at": "1970-01-01 00:00:00",
    "run_id": "",
    "commit_sha": "",
    "working_tree": "",
    "instance_type": "",
    "region": "",
    "cpu_count": 0,
    "cache_line": 0,
    "memory_kb": 0,
    "rustc": "",
    "profile": "",
    "seed": 0,
    "faction_count": 0,
    "settlements": 0,
    "bench": "",
    "tiles": 0,
    "units": 0,
    "threads": 0,
    "samples": 0,
    "min_ns": 0,
    "median_ns": 0,
    "max_ns": 0,
    "empty_bytes": 0,
    "resident_bytes": 0,
    "peak_bytes": 0,
}

# A comment line in the result file names a fact about the run. The name on
# the left is what the benchmark and the script print; the name on the right
# is the column. A fact with no column here is kept out of the table rather
# than guessed at.
RUN_FACTS = {
    "run_id": "run_id",
    "commit": "commit_sha",
    "working_tree": "working_tree",
    "instance_type": "instance_type",
    "region": "region",
    "cpu_count": "cpu_count",
    "cache_line_bytes": "cache_line",
    "memory_kb": "memory_kb",
    "rustc": "rustc",
    "profile": "profile",
    "seed": "seed",
    "faction_count": "faction_count",
    "settlements": "settlements",
}

NUMERIC = {
    name
    for name, default in COLUMNS.items()
    if isinstance(default, int)
}


def parse(text: str) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Returns the facts about the run, and one dictionary for each row."""
    facts: dict[str, Any] = {}
    rows: list[dict[str, Any]] = []
    header: list[str] | None = None

    for line in text.splitlines():
        if not line.strip():
            continue
        if line.startswith("#"):
            parts = line.lstrip("#").strip().split("\t")
            if len(parts) == 2 and parts[0] in RUN_FACTS:
                # The result file repeats its preamble once for each sweep it
                # ran. The first value wins, because a later sweep restates
                # the same run rather than describing a new one.
                facts.setdefault(RUN_FACTS[parts[0]], parts[1].strip())
            if parts[0] == "taken_utc" and len(parts) == 2:
                facts.setdefault("taken_at", parts[1].strip().replace("T", " ").rstrip("Z"))
            continue

        fields = line.split("\t")
        if fields[0] == "bench":
            header = fields
            continue
        if header is None or len(fields) != len(header):
            continue
        row = dict(zip(header, fields, strict=True))
        rows.append(row)

    return facts, rows


def shape(facts: dict[str, Any], row: dict[str, Any]) -> dict[str, Any]:
    """Returns one row of bench.rows, with every column present."""
    shaped = dict(COLUMNS)
    for key, value in facts.items():
        if key in shaped:
            shaped[key] = value
    for key, value in row.items():
        if key in shaped:
            shaped[key] = value
    for key in NUMERIC:
        try:
            shaped[key] = int(str(shaped[key]).strip() or 0)
        except ValueError:
            shaped[key] = 0
    return shaped


def send(rows: list[dict[str, Any]]) -> None:
    """Posts the rows to ClickHouse as newline delimited JSON."""
    body = "\n".join(json.dumps(row) for row in rows).encode()
    query = "INSERT INTO bench.rows FORMAT JSONEachRow"
    request = urllib.request.Request(
        f"{CLICKHOUSE}/?query={urllib.parse.quote(query)}",
        data=body,
        method="POST",
    )
    request.add_header("X-ClickHouse-User", USER)
    request.add_header("X-ClickHouse-Key", PASSWORD)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            response.read()
    except urllib.error.URLError as error:
        raise SystemExit(
            f"Could not reach ClickHouse at {CLICKHOUSE}: {error}\n"
            "Start the stack with `just obs-up`."
        ) from error


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result", type=Path, help="the file the benchmark run produced")
    parser.add_argument("--log", type=Path, help="the run log, which goes to Loki")
    parser.add_argument(
        "--print",
        dest="dry",
        action="store_true",
        help="write the rows and send nothing",
    )
    arguments = parser.parse_args()

    facts, rows = parse(arguments.result.read_text())
    if not rows:
        print(f"{arguments.result}: no row found. Is this a benchmark result?", file=sys.stderr)
        return 1

    shaped = [shape(facts, row) for row in rows]

    if arguments.dry:
        for row in shaped:
            print(json.dumps(row))
        print(f"# {len(shaped)} rows, run {facts.get('run_id', 'unnamed')}", file=sys.stderr)
        return 0

    send(shaped)
    print(f"Loaded {len(shaped)} rows from {arguments.result}")

    log = arguments.log
    if log is None and arguments.result.with_suffix(".log").exists():
        log = arguments.result.with_suffix(".log")
    if log is not None and log.exists():
        RUNS.mkdir(parents=True, exist_ok=True)
        destination = RUNS / f"{facts.get('run_id', log.stem)}.log"
        shutil.copyfile(log, destination)
        print(f"Copied the log to {destination}. The collector reads it within a minute")

    print("Open http://127.0.0.1:3000 for the dashboards")
    return 0


if __name__ == "__main__":
    sys.exit(main())
