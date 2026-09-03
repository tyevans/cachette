---
id: 0312
title: Write the tutorial from nothing to a running world
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**A reader who has never seen the package must build a world and run one tick
from the documentation alone.** That is the first checkable statement of the
product record, and nothing in the tree serves it.[^1] The orientation document
holds a worked example, and an example is not a tutorial: it shows a reader who
already knows the shape of the thing what the calls look like.

A tutorial teaches by doing. One page holds it, and one page is enough, because
the smallest useful run is short.

## What the work does

Write one tutorial page. The reader starts with a machine and ends holding a
number that came out of a simulation they built.

The page names each thing the reader must already have. It installs the package.
It builds a world, spawns a set of units, gives one order over that set, steps
the world, and reads the tick, the state hash and one column of the event log.

Every line of code on the page is executed by the harness, so the page fails
when the package moves.[^2]

## What it deliberately does not cover

It does not open a window. It does not explain the pyramid, the selector, the
fixed-point scale or the threading model. It does not tune anything. It states
no performance figure, because a blocker governs every figure in this
project.[^3] It does not mention the agent server, because that surface serves a
different audience and another record holds it.[^4] It does not explain why any
part of the engine is the way it is. A reader who asks why is reading the wrong
quadrant, and the page says so once at the end and sends them to the
explanation.

## Why this is not refined

The page cites the reference for every call it names, and no reference is
published yet.[^5] The harness that runs the code does not exist yet either.[^2]
Refining this item means fixing the exact call sequence against the published
reference, so that the page and the reference cannot disagree.

## References

[^1]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: Backlog item 0311, execute every documentation example from the test suite. `docs/backlog/proposed/0311-execute-every-documentation-example-from-the-test-suite.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Product requirement record 0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^5]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/refined/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
