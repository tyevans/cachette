---
id: 0229
title: Explain the frame cost at the smallest extent
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The first run of the benchmark on the target platform measured one frame at
five extents and at three thread counts. Four of the five extents agree with
each other. The smallest does not.

At 4,096 tiles a frame costs 369 ns for each tile on one thread, 619 ns on two
threads, and 59 ns on four threads. Every other extent costs between 37 and
78 ns for each tile at every thread count. The machine holds two hardware
threads, so the four-thread column is the fastest column on a machine that
cannot run four threads at once.

The result is not noise. Nine samples produced a spread under one fifth in
each of the three rows. The register holds the figures and marks the rows as
unusable.[^1]

**A world of 4,096 tiles is the size that most tests use.** If a stage costs
more at a low tile count than at a high one, the suite pays it on every test,
and no test can see it. That is why this question is worth answering rather
than filing as a curiosity.

## What is missing before this is refined

- **Nobody knows which stage costs the time.** A step runs a tile pass, a
  choice pass, movement, admission, a holder spread, a rate pass, a
  consumption pass, a death scan, a position settle, a level 1 rebuild and an
  influence solve. The benchmark measures the whole step and separates none of
  them. Something must measure the stages before anything can explain the
  rows.
- **A stage timer is not free to add.** A timer inside the step would read the
  clock inside the simulation, and a lint forbids that across this
  workspace.[^2] The benchmark holds the one allowance, and it holds it
  outside the engine. Decide how a stage is measured before measuring one.
- **The candidate explanations are untested.** The influence solve runs a
  fixed iteration count over the whole plane, and a small world has few cells.
  The level 1 rebuild takes a thread count. Neither has been checked against
  the rows.
- **Two figures are missing.** Nobody has run the same sweep on a second
  instance, and nobody has run it at a sixth extent between 4,096 and 65,536
  tiles. Either would say whether the effect is a step or a curve.

## Done when

- Something measures the stages of a step, and the way it does so does not put
  a clock inside the engine.
- The three rows at 4,096 tiles are explained, or the item states plainly that
  they are not and names what was ruled out.
- The register stops marking those rows as unusable, or it says why they stay
  unusable.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, one frame against the tile count. `docs/reference/graviton-costs.md`
[^2]: The lint configuration. `clippy.toml`
