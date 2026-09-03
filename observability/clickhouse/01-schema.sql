-- The benchmark rows.
--
-- One row is one measurement: one operation, at one extent, at one unit
-- count, at one thread count, on one machine, at one commit. The columns that
-- describe the run repeat on every row on purpose, so that a query needs no
-- join and a row means something on its own.
--
-- A timing row fills the sample columns and leaves the memory columns at
-- zero. A memory row does the opposite. The `bench` column says which.

CREATE DATABASE IF NOT EXISTS bench;

CREATE TABLE IF NOT EXISTS bench.rows
(
    -- What produced the row.
    taken_at        DateTime,
    run_id          LowCardinality(String),
    commit_sha      LowCardinality(String),
    working_tree    LowCardinality(String),

    -- The machine.
    instance_type   LowCardinality(String),
    region          LowCardinality(String),
    cpu_count       UInt16,
    cache_line      UInt16,
    memory_kb       UInt64,
    rustc           LowCardinality(String),

    -- The world.
    profile         LowCardinality(String),
    seed            UInt64,
    faction_count   UInt16,
    settlements     UInt32,

    -- The measurement.
    bench           LowCardinality(String),
    tiles           UInt64,
    units           UInt32,
    threads         UInt16,

    -- A timing row.
    samples         UInt16,
    min_ns          UInt64,
    median_ns       UInt64,
    max_ns          UInt64,

    -- A memory row.
    empty_bytes     UInt64,
    resident_bytes  UInt64,
    peak_bytes      UInt64
)
ENGINE = ReplacingMergeTree
ORDER BY (bench, tiles, units, threads, instance_type, commit_sha, taken_at);

-- The quantities a reader actually asks for.
--
-- The cost for each tile and for each unit are the two constants that the
-- register states, and a division in a dashboard query is a second place to
-- get them wrong. They are computed once, here.
CREATE VIEW IF NOT EXISTS bench.derived AS
SELECT
    *,
    if(tiles > 0, median_ns / tiles, NULL)              AS ns_for_each_tile,
    if(units > 0, median_ns / units, NULL)              AS ns_for_each_unit,
    median_ns / 1000000                                 AS median_ms,
    -- The engine runs at ten ticks for each second, so a frame has 100
    -- milliseconds. That rate is a target the project chose. It is not
    -- derived and it is not measured, and the register says so.
    median_ns / 100000000                               AS budget_multiple,
    if(median_ns > 0, 1000000000 / median_ns, NULL)     AS ticks_for_each_second,
    if(tiles > 0 AND resident_bytes > 0,
       (resident_bytes - empty_bytes) / tiles, NULL)    AS bytes_for_each_tile
FROM bench.rows;
