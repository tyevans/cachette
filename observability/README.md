# The Observability Stack (Index)

This document is an **index**. It says what the local observability stack
holds, how to start it, and which decisions it rests on. The detail of each
figure lives in the register that holds the measurements.[^1]

The stack exists so that a person explores a benchmark result repeatedly and
in detail. A register holds the figures and the reasoning, and a register
answers the questions somebody thought of when they wrote it. The stack
answers the rest.

## Start it

```
just obs-up
just obs-load /tmp/some-benchmark-result.txt
just obs-down
```

Grafana runs at `http://127.0.0.1:3000`. It asks for no password, because it
listens on the loopback address of one machine.

`just obs-clean` stops the stack and deletes every stored row and log.

## What runs

| Service | What it holds | Port |
|---|---|---|
| ClickHouse | The benchmark rows | 8123 |
| Loki | The log of each benchmark run | 3100 |
| OpenTelemetry collector | The log pipeline, and an OTLP port | 4317, 4318 |
| Grafana | The dashboards | 3000 |

Every port binds to `127.0.0.1`. Nothing is reachable from another machine.

## Three rules this stack obeys

**No timing value reaches the simulation.** The engine gains no
instrumentation, and no code in the core crate reads a clock. A benchmark
reads one, at one function, under one allowance.[^2] A timing value that
reached the whole-world hash or an event type would put nondeterminism into
the one property this project cannot recover, and that path looks harmless at
review.[^3]

**Nothing here gates a commit.** The gate command does not start a container
and does not need one. A gate that needs Docker is a gate that somebody cannot
run, and a gate nobody can run is a gate everybody learns to skip.

**Nothing here bills, and nothing here listens outside this machine.**

## Two design choices worth knowing

**The benchmark writes a file. A separate step ships it.** The benchmark opens
no socket. A run is therefore reproducible with nothing listening, and a run
that nobody ships is still a complete record on disk. The shipper reads the
file the run produced.[^4]

**The benchmark rows go to SQL, and not through the metric pipeline.** A
metric is a live stream, keyed by the wall clock, and a store for one rejects
or reorders a sample that arrives late. A benchmark row is the opposite: it is
an experiment result, keyed by a commit and a machine, loaded long after the
moment it describes, and read by comparing one commit against another. SQL is
what that comparison is made of. The collector still holds an OTLP port, and
it is the address for any telemetry this project sends later.

## The table

One row is one measurement: one operation, at one extent, at one unit count,
at one thread count, on one machine, at one commit.

A timing row fills the sample columns and leaves the memory columns at zero. A
memory row does the opposite. The `bench` column says which. The `derived`
view adds the quantities a reader asks for, so that a dashboard does not
divide by a tile count in five places and get it wrong in one.[^5]

The frame budget in that view is 100 milliseconds. **It is a target the
project chose. No record derives it and no measurement produced it.** The
register says so, and a ratio against it is only as good as it is.[^1]

## A caution about the log pipeline

The collector reads a log file once and remembers how far it read. A file
copied into the run directory a second time under the same name does not
arrive twice, and a file that changes after it is read arrives only from the
point it changed. Copy a finished log, not a log a run is still writing.

## References

[^1]: Target platform costs. `docs/reference/graviton-costs.md`
[^2]: The benchmark. `crates/cachette-core/benches/target_cost.rs`
[^3]: Recurring defect shapes, section 4. `.claude/rules/recurring-defects.md`
[^4]: The shipper. `scripts/ship_bench.py`
[^5]: The schema. `observability/clickhouse/01-schema.sql`
