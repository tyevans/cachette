---
id: 0516
title: Give a faction one flat observation array, and declare its layout in a schema
status: proposed
created: 2026-09-06
implements: [ADR-0154 D1, ADR-0154 D2, ADR-0154 D3]
changes: []
creates: []
serves: [PRD-0056, PRD-0001]
blocked-by: [BLK-007]
---

## Why

**A learner reads one array on every decision, and the engine offers none.** An
accepted record says what that array is. The observation of one faction is one
flat array of signed integers, its length is a function of the world parameters
and never of the population, and the engine returns a schema that names each
field, where it starts, how many positions it holds, its width and the bound of
each position.[^1] [^2] No file outside the engine may state an offset, a length
or a bound.[^1]

**Nothing in the engine holds either half.** No observation reader exists, no
schema call exists for anything but the declared event layouts, and nothing
about a faction's own state crosses to the control plane as one array. A survey
of the tree found the whole contract unbuilt.[^3]

The record also says the reader starts no pass over the tiles or the units. It
reads the aggregates the engine already keeps and the summary the pyramid
already rebuilt.[^2] A survey read that clause against the code and found the
inputs present: the fog and the summary level share one lattice, and the
per-cell visibility answer is already a faction mask beside each summary
cell.[^3] [^4]

**This item is the pattern the engine already proves.** The binding derives one
column for each declared field of an event, and it names no field.[^5] The
observation schema is the same shape applied to a different table, and the
column builder is the model to copy.[^3]

**It depends on item 0495.** The map block of the observation must hold only
what the faction observes, and the readers that answer for one faction are the
work of that item.[^6] The blocks that hold a faction's own standing, its own
relation row, the public board and its own weights need no mask, and the record
says so in its own consequences.[^1] Whether this item ships those blocks first
and the map block second is a question refining it must answer.

## What is missing before this can be refined

- **Which fields the array holds, and in what order.** The record fixes the
  form and no document fixes the list. One design document names a map block of
  the two block extents with fields for each cell, and it is a design and not a
  record.[^7] The work must decide the field list, and every length and every
  bound it states becomes a row of the reference tables rather than a literal in
  the code.[^8]

- **What the array says about a place the faction has never seen.** Item 0495
  raises the same question for a reader and does not settle it: a refusal, a
  zero, or a second mask that says which positions the caller may read.[^6] An
  observation cannot refuse one position of an array, so this item has fewer
  options than that one, and it must say which it takes.

- **Whether the version integer is this item or a later one.** The record says
  the engine carries one version integer beside the schema, that a learner which
  loads a policy under a different version stops with an error naming both, and
  that the version and its test are code rather than a record.[^1] The work must
  say whether it builds the version now or defers it, and a deferral leaves a
  trained policy with nothing that says which layout produced it.

- **What the reader costs at the target scale.** The record forbids a pass over
  the tiles or the units, so the cost should follow the lattice.[^9] No figure
  for it exists, and one blocker says every cost figure of this project stays
  derived until the target platform measures it.[^10] The work must express the
  cost parametrically and cite that blocker rather than state a number.

- **How a test proves the schema and the array cannot disagree.** The record
  asks for a test that builds the observation, reads the schema, and asserts
  that each field sits where its row says.[^1] A test that reads both from one
  constant proves nothing. The work must say what it perturbs to make the test
  fail, and it must put the defect back and watch the test stay green before it
  claims the case is covered.[^11]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
[^6]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/proposed/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^7]: Design, one environment core serves every learning stack. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^8]: Reference registers. `docs/reference/`
[^9]: ADR-0096, cost follows the lattice, not the population. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Testing Rules, sections 1 and 2a. `.agents/rules/testing.md`
