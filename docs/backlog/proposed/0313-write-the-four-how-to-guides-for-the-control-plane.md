---
id: 0313
title: Write the four how-to guides for the control plane
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**A reader who has finished the tutorial arrives with a goal, and a tutorial
does not serve a goal.** A how-to guide names one goal and reaches it. The
product record names three of the goals directly: install the package, learn the
rule that governs every program on this engine, and get an answer out of a run
without walking the population.[^1]

## What the work does

Write four pages. Each names one goal in its title, and each ends with the
reader having reached it.

1. **Install the package and build the extension.** The reader has a machine and
   no environment. The page names the interpreter floor, the Rust toolchain, the
   package manager and the command runner, and it ends with an import that
   works. The product record requires the instruction to name each thing the
   reader must already have.[^1]
2. **Repeat a run exactly.** The reader wants two runs to agree. The page names
   the seed, the thread count, the state hash and the event log, and it states
   what the guarantee covers: one binary gives one answer at any thread
   count.[^2]
3. **Read what one tick did, without walking the population.** The reader wants
   the result of a step. The page reads the event columns and the gather columns
   into arrays and works on the arrays.[^3]
4. **Order a set of units with one command.** The reader wants many units to do
   one thing. The page builds the set, sends one command, and says plainly that
   a loop is the thing the engine exists to make unnecessary.

Every line of code on every page is executed by the harness.[^4]

## What it does not do

It does not draw a frame. `Camera` is a public name and the reference covers it,
but a reader who cannot install the package cannot draw anything, so that guide
waits.

It does not document the agent server. That surface serves a contributor to this
repository, another record holds that audience, and the product record excludes
it.[^1] [^5]

It states no performance figure, and it does not say whether an upgrade changes
hands when the ground does. A blocker governs each.[^6] [^7]

## Why this is not refined

Each page cites the reference for the calls it names, and no reference is
published yet.[^8] The fourth page also depends on how much of the rule the
explanation quadrant states, and that page is not written.[^9] Refining this
item draws the line between the two.

## References

[^1]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: Decisions register, DEC-060. `docs/DECISIONS.md`
[^4]: Backlog item 0311, execute every documentation example from the test suite. `docs/backlog/proposed/0311-execute-every-documentation-example-from-the-test-suite.md`
[^5]: Product requirement record 0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^8]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/refined/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
[^9]: Backlog item 0314, write the explanation pages and link out rather than copy. `docs/backlog/proposed/0314-write-the-explanation-pages-and-link-out-rather-than-copy.md`
