# ADR Registry (Index)

This document is an **index**. It lists every architecture decision record in
this project, assigns their numbers, and records what depends on what.

Rule 4 of the documentation rule exempts a table that lists files and paths as
data. The tables below are that table. They are this document's reference
list.

## How to use this registry

**Assign the number here before you write the record.** Add the row first,
with status `Proposed` and no file. Then write the file.

This registry is the allocator. Three number collisions occurred during the
research phase because agents chose their own numbers. That cannot happen if
the number comes from here.

**Never reuse the number of an accepted record.** A superseded record keeps
its number forever, because other records cite it.

A number may be **reclaimed** only when the record was never accepted and
nothing cites it. This has happened once: an omnibus draft held number 0001
before the research was done. It was deleted, not superseded, and 0001 was
reassigned. The reasoning is in git history.

**An author may set `Draft`. Only a reviewer may set anything beyond it.**

`Proposed` means the number is reserved and no file exists. `Draft` means a
file exists and is under review. Moving from one to the other is a statement
of fact, so the author makes it when they write the file.

`Accepted`, `Superseded` and `Rejected` are judgements. A reviewer makes
those. An author who accepts their own record has reviewed their own work.

### Who reviews

The project owner holds review rights and has delegated them to the agent
that writes the records, for the current phase of work. The delegation is
recorded here because a reader who sees one name in both roles would
otherwise read it as the failure this section warns about.

**A delegated reviewer is a weaker reviewer than a second person, and the
process compensates for it.** An author who reviews their own record cannot
be surprised by it, so the review must find what surprise would have found:

1. **Review reads the record against the code, not against the intent.** A
   record is accepted after the work that implements it exists, or its
   acceptance says plainly that nothing implements it yet.
2. **A record is reviewed by an agent that did not write it.** The reviewer
   gets the record and the rule, not the reasoning that produced it.
3. **The review states what it tried to reject.** A review that lists no
   attempted objection did not happen.

The owner may withdraw the delegation at any time, and a review by a second
person supersedes a delegated one.

**Decision numbers are local to their record.** Write `ADR-0004 D3`, not `D3`.
Flat global decision numbering collided three times during research and will
do so again.

## Which record comes next

A separate index states which records wait for review and which reserved rows
the project means to write next.[^PRI] It holds no status. This registry holds
the status, and it is the only document that does.

## Status vocabulary

| Status | Meaning |
|---|---|
| `Proposed` | The number is reserved. No file exists yet. |
| `Draft` | The file exists and is under review. Not binding. |
| `Accepted` | Binding. Cite it. Do not edit it except to change status. |
| `Superseded` | Replaced. The row names the replacement. Keep the file. |
| `Rejected` | Considered and declined. Keep the file; the reasoning is useful. |

An accepted record does not change. To change a decision, write a new record
that supersedes it.

### The retcon window

**The freeze protects a record's dependents, not the record.** A record that
nothing cites, that no code implements, and that no other record was written
against has no dependents, so amending it breaks nothing and a supersession
record would say only that the author changed their mind an hour later.

Amend an accepted record in place when all three hold:

1. **Nothing depends on it yet.** No other record cites it, no source file
   cites it, and no backlog item was refined against the claim you are
   changing.
2. **The amendment repairs the record rather than reversing it.** Correcting
   false reasoning, a stale reference, or a claim that contradicts a sibling
   record is a repair. Deciding the opposite thing is not, and that needs a
   supersession however recent the acceptance.
3. **The commit says what changed and why the freeze did not apply.** The
   commit message is the audit trail. A silent edit to an accepted record is
   the failure this whole section is about.

Outside that window, supersede.

**A draft is not covered by any of this.** A draft exists to be edited. Edit
it, and do not revert its status to justify doing so.

The window closes the moment someone builds on the record, which is usually
the next commit. When in doubt, assume it closed.

### Repairing a citation is not an amendment

**The three conditions above govern a change to what a record claims. They do
not govern a change to where a pointer goes.**

A record cites a number that is later retired, or a path that later moves. The
citation is then false, and it became false through no fault of the record. Fix
it. Do not test it against the window, and do not write a superseding record
whose whole content is a corrected pointer, which is a record for a topic and
not for a constraint.[^SCOPE]

The test is one question. **Does the edit change what a reader would decide?**

- Repointing a footnote at the row that now holds the material is a repair.
  Nothing a reader decides from the record changes.
- Rewording the sentence that carries the footnote marker is an amendment, and
  the window governs it. The exception is a sentence whose own content the
  register made false, and the section on an imported fact states it.
- Adding, removing or reversing a claim, a force or a rejected alternative is
  an amendment, whatever else the commit touches. A consequence the record
  derived from its own claim is the one exception, and the next section states
  it.

The commit still says what changed and why, because that is the audit trail and
a silent edit to an accepted record is the failure this whole section is about.

This has happened once, and the finding holds the case and the reasoning.[^CITE]

### Repairing a derived consequence is not an amendment

**A record states what follows from its claim. What follows changes when the
project learns, and the claim does not.**

A consequence that a record derived from its own claim may be struck or
corrected in place, in an accepted record, whatever depends on it. Do not test
it against the window. Do not write a superseding record to change one derived
clause, because that record's subject is a topic and not a constraint.[^SCOPE]

The test is the same question. **Does the edit change what a reader would
decide?** A reviewer refuses a change because of the claim. A reviewer does not
refuse one because of a cost clause. Striking the cost clause therefore leaves
every refusal the record supports.

- A cost clause, a complexity claim, or a sentence that follows "therefore" and
  restates the claim in another unit is derived. Strike it.
- A sentence that a reader would cite on its own is a claim, however late in the
  record it sits. The window governs it.
- The record that stales the consequence binds first. A draft stales nothing, so
  the strike waits for the acceptance.

The commit says which sentence went, which record staled it, and why the edit is
a repair. A register holds the reasoning and the case that raised it.[^DERIVED]

### Repairing a fact a record imported from a register is not an amendment

**A record sometimes states the condition of the project rather than its own
reasoning.** It says that no measurement exists, or that a question is open, and
it cites the register that held that condition. The register then moves, and the
sentence is false through no fault of the record.

This is the sibling of the section above and it is not the same case. A derived
consequence follows from the record's own claim. An imported fact follows from a
register. Do not read one section into the other, because "derived" would then
mean "anything a later fact made false", and that reading swallows the whole
amendment rule.

**Reword the sentence in place, in an accepted record, whatever depends on it.**
Keep the footnote marker, because the register the record cites is still the
register that holds the condition. Correcting the sentence and dropping the
citation are two edits, and only the first one is a repair.

The test is the same question. **Does the edit change what a reader would
decide?**

- A clause that states the condition of a register is imported. Reword it to the
  condition that now holds, or strike it when nothing is left to say.
- **A record does not gain a figure in exchange.** The rule that keeps a
  measured figure out of a record body does not depend on whether the
  measurement exists.[^SCOPE] A sweep that adds a number while it repairs a
  sentence has made the record worse.
- **A claim that rested on the condition is not imported.** Ask whether the
  record would have decided differently if the register had said the other
  thing. If it would, the sentence is a force and not a fact, and the window
  governs it.
- **Reword each record against its own subject, and never once for the group.**
  A register narrows in parts. A sentence that names the whole project is false
  everywhere, and the narrow sentence beneath it often survives. A sweep that
  applies one replacement to every file states a new false thing in the files
  where the narrow sentence was still true.[^SWEEP]

The commit says which sentence changed, which register row moved, and why the
freeze did not apply. A register holds the reasoning and the case that raised
it.[^IMPORTED]

## What does not belong in a record

A record is a historical document. It must not hold material that changes.

Put these in the registers and cite them:

- Per-tick cost budgets. They change with every measurement.
- The byte budget table.
- Constant values that depend on an unanswered question.
- Dependency versions and named releases. Put the version you read in a
  footnote, where it records what you consulted.
- Counts, file tables, and survivor lists. Put these in the commit message,
  which is immutable and scoped to one moment.
- A module arrangement, unless the arrangement is itself the constraint.
- A capability that nothing invokes. Do not record an intent as a fact.

The **rules** are decisions. The **numbers** are living reference.

The full rule states the test for whether a decision needs a record at all, and
gives the evidence for each category above. Read it before you write a record.
It is derived from a measurement of 106 records in two sibling projects.

Two results from that measurement bear on this project. Record length predicts
churn, with a correlation near 0.71 in both reference corpora. A title that
states a claim bounds a record; a title that names a topic does not, and a
topic title churned twice as often. Every row below therefore states a claim,
and a record that grows past one claim is split rather than extended.

A script checks the mechanical part of this rule and can run in continuous
integration. It fails when a record misses a required section, holds a version
pin or a volatile figure, cites a number that no record has, or disagrees with
this registry.

| Path | Purpose |
|---|---|
| `.claude/rules/adr-scope.md` | The record scope rule |
| `docs/research/adr-scope-findings.md` | The measurement behind the rule |
| `scripts/check-adrs.sh` | The record check |

## The records

Every row states a **claim**, not a topic. A title that names a topic has no
boundary, so any material about the topic can be added to the file. A claim
title has a boundary, and material that does not support the claim has
nowhere to go. In one reference corpus a topic-titled record churned 4.50
times on average against 2.13 for a claim-titled one.[^MEASURE]

**`Proposed` reserves the number. It does not promise the record.** Before you
write the file, apply the three-condition test in the scope rule.[^SCOPE] If the
claim fails it, drop the row and retire the number. A record that binds work
without stating a constraint is worse than no record.

### Determinism

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0001 | One binary gives one answer at any thread count | Accepted | — | 03, 07, 13 |
| 0002 | Simulated and aggregated state holds no floating point number | Accepted | 0001 | 03, 13 |
| 0003 | Every random draw is keyed, never stateful | Accepted | 0001 | 03 |
| 0004 | Iteration order is explicit, and unordered reductions need slots | Accepted | 0001 | 03, 06 |
| 0005 | A solver runs a fixed iteration count, never a convergence test | Accepted | 0001 | 06, 13 |
| 0006 | An event is plain data and applying it is pure | Accepted | 0001 | 03 |
| 0007 | Content supplies a key vector, never a comparator | Accepted | 0001 | 04 |

### Platform and value types

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0008 | The primary target is aarch64 | Accepted | 0001 | 07 |
| 0009 | Parallel stages write disjoint outputs, because the memory model is weak | Accepted | 0001, 0008 | 07 |
| 0010 | The cache line size is a compile-time constant | Proposed | 0008 | 07 |
| 0011 | Every value type is a newtype | Accepted | 0002, 0008 | 07 |

### Storage

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0012 | Tiles are dense columns and units are a generational arena | Accepted | 0001 | 01 |
| 0013 | The project writes its own entity storage rather than adopting an ECS | Proposed | 0012 | 01 |
| 0014 | Entity identity is an index plus a generation | Accepted | 0012 | 01 |
| 0015 | A tile column is narrow, with bitplanes and sparse side tables | Proposed | 0012 | 01 |
| 0016 | Tiles are stored in block-tiled order at the aggregation block size. The tiles are stored row by row today, and FND-217 records a draft that read this row as settled | Proposed | 0012, 0022 | 01, 02 |
| 0017 | The world is a rhombus, so a tile index is raw axial | Accepted | 0016 | 02 |
| 0018 | The unit-to-tile bridge is derived, and it rebuilds at the barrier | Accepted | 0012 | 01, 02 |
| 0019 | Change detection is per chunk, never per entity | Proposed | 0012 | 01, 02 |
| 0020 | Structural change batches at the barrier and applies by tombstone and compact | Proposed | 0001, 0012 | 01 |
| 0021 | A layout claim names one structure and one pass, and never a tier | Draft | 0012, 0054, 0066 | `draft/adr-0021-layout-follows-the-access-pattern.md` |
| 0066 | Entity storage holds four fixed shapes | Accepted | 0012, 0020 | 01, 05 |
| 0090 | A tile upgrade is stored sparsely, as the difference from the generated world | Draft | 0012, 0015, 0056, 0066, 0068, 0072, 0074 | `draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md` |

### The pyramid

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0022 | Level 0 is the only truth, and every level above it is derived | Accepted | 0012 | 02 |
| 0023 | An aggregate combines exactly, in any order | Accepted | 0002, 0022 | 02 |
| 0024 | Every summary field is declared extensive or intensive | Accepted | 0023 | 02 |
| 0025 | The pyramid carries two update paths, chosen by a threshold | Proposed | 0022, 0019 | 02 |
| 0026 | The world holds two pyramids, not one | Proposed | 0022 | 02, 08 |
| 0027 | The pyramid is the query index and the statistics catalogue | Proposed | 0022 | 02, 04 |
| 0028 | Descent has a cost model and a flat fallback | Proposed | 0027 | 02 |
| 0029 | An operator does not commute with aggregation | Proposed | 0023 | 02, 13 |

### The log

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0031 | Events live in type-segregated arenas of plain data | Draft | 0001, 0004, 0006 | 03, `draft/adr-0031-events-live-in-type-segregated-arenas.md` |
| 0032 | The log holds a fact no solver reproduces, never derived state | Draft | 0001, 0003, 0005, 0006 | 03, `draft/adr-0032-the-log-holds-a-fact-no-solver-reproduces.md` |

### The Python boundary

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0040 | Python is a control plane, not a data plane | Draft | 0001 | `draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md` |
| 0041 | A crate split enforces the boundary at compile time | Draft | 0001, 0040 | 05, `draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md` |
| 0042 | The interpreter is released for the whole step | Draft | 0001, 0040, 0041 | 05, `draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md` |
| 0043 | A declared tier enforces the no-loop rule, and the API refuses the loop | Draft | 0040 | `draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md` |
| 0044 | What copies and what does not is declared at the call site | Draft | 0012, 0040 | 05, `draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md` |
| 0046 | Every error is typed | Draft | 0040, 0085 | 05, `draft/adr-0046-every-error-is-typed.md` |
| 0047 | Many worlds live in one interpreter | Draft | 0040, 0042 | 05, `draft/adr-0047-many-worlds-live-in-one-interpreter.md` |
| 0085 | An entity crosses to Python as one opaque identity that the engine resolves | Accepted | 0006, 0011, 0014, 0040, 0044 | `accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md` |

### Subsystems

| No. | Claim | Status | Depends on | Source |
|---|---|---|---|---|
| 0048 | A verb declares a pipeline of kernels | Proposed | 0001 | 06, 13 |
| 0049 | A quantity is a rate, a constraint or a set, and each composes by one law | Proposed | 0023 | 13 |
| 0050 | The frame schedule is static and known before the frame runs | Proposed | 0001, 0009 | 06 |
| 0051 | A selector is a lazy expression tree that Rust evaluates | Accepted | 0040, 0043 | `accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md` |
| 0052 | A selector result may be a range, not only an enumerated set | Draft | 0051, 0028 | `draft/adr-0052-a-selector-result-may-be-a-range.md` |
| 0053 | A faction is a bit in a mask, and a relation is a plane | Accepted | 0011, 0012, 0023, 0024 | 08 |
| 0054 | An entity belongs to one of three tiers, declared at creation | Accepted | 0012, 0043 | 14, 15, 16 |
| 0055 | An effective stat comes from an ordered modifier pipeline | Proposed | 0002, 0054 | 12 |
| 0056 | Movement is tile-discrete and admitted by sort-then-admit | Accepted | 0004, 0018 | 06, 10 |
| 0058 | A field update is a flux pair on an edge, so quantity is conserved exactly | Proposed | 0023, 0029 | 13 |
| 0059 | Fog storage grows with observed area, not with world area | Proposed | 0026, 0053 | 08 |
| 0060 | An influence map is stored as a shared basis, not one plane per faction | Draft | 0022, 0053 | 09, `draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md` |
| 0061 | Trade solves a flow, never a path for each cart | Proposed | 0049, 0058 | 11 |
| 0062 | Production and upkeep are rates attached to a site | Accepted | 0055 | 12, `accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md` |
| 0063 | A need is a rate with a threshold, and crossing it is a fact | Accepted | 0032, 0062 | 15, `accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md` |
| 0064 | A unit chooses by scoring a small fixed option set | Accepted | 0001, 0002, 0003, 0004, 0007, 0022, 0063 | 16, `accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md` |
| 0065 | A group is a site membership, not a region | Draft | 0004, 0009, 0014, 0040, 0054, 0056, 0066 | 17, `draft/adr-0065-a-group-is-a-site-membership-not-a-region.md` |
| 0067 | The viewer reads the world and never writes to it | Accepted | 0001 | — |
| 0068 | Terrain is generated from the seed and is never stored as a map | Accepted | 0001, 0002, 0003, 0012 | — |
| 0069 | Weather is a field the world integrates, never a table it reads | Reserved | 0058, 0068 | — |
| 0070 | The head-up display reports what the drawing pass read | Accepted | 0067, 0018 | `accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md` |
| 0093 | The window shows what changes, and the record of a moment goes to the inspection path | Draft | 0067, 0070 | `draft/adr-0093-the-window-shows-what-changes.md` |
| 0094 | The caller owns the camera and the pixels, and one command fills them | Draft | 0022, 0041, 0044, 0067, 0093 | `draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md` |
| 0071 | The bridge rebuild orders on one thread | Accepted | 0001, 0004, 0007, 0018 | `accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md` |
| 0072 | A tile stock is generated, and only what was taken is stored | Accepted | 0002, 0003, 0068 | `accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md` |
| 0073 | Gathering is admitted by sort-then-admit against the tile | Accepted | 0004, 0018, 0056, 0072 | `accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md` |
| 0074 | A spawn may over-fill a tile, and only admission enforces the capacity | Accepted | 0018, 0056 | `accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md` |
| 0075 | The founding choice reads a bounded sample of the world | Accepted | 0002, 0003, 0004, 0068 | `accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md` |
| 0076 | A founding keeps a fixed distance from the foundings before it | Accepted | 0003, 0004, 0053, 0075 | `accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md` |
| 0077 | The golden state hash is exact, order-sensitive and stable across build platforms | Reserved | 0001 | — |
| 0078 | Descent is a bounded record, and a relation is a bounded recursion | Draft | 0002, 0004, 0014, 0054 | 14 |
| 0079 | Succession is filter, then sort by a key vector, then allocate | Proposed | 0004, 0007, 0014, 0078 | 14 |
| 0080 | A depleted deposit recovers by ageing the stored take, never by a pass over the world | Accepted | 0002, 0003, 0004, 0072, 0073 | `accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md` |
| 0081 | A residence is a stored column and occupancy is a maintained count | Draft | 0004, 0014, 0018, 0063, 0066, 0074 | `draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md` |
| 0082 | The store sets the rate of a birth and the housing admits it | Draft | 0003, 0014, 0056, 0062, 0063, 0074, 0081 | `draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md` |
| 0083 | The gate build checks every integer overflow | Draft | 0001, 0002 | `draft/adr-0083-the-gate-build-checks-every-integer-overflow.md` |
| 0084 | The world reserves the unit columns at construction, and a spawn past the reservation is refused | Draft | 0012, 0014, 0066 | `draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md` |
| 0088 | A tile field is a generated base and a stored change | Draft | 0002, 0003, 0004, 0009, 0012, 0068, 0072 | `draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md` |
| 0087 | An influence solve runs a fixed iteration count over the whole plane | Draft | 0001, 0002, 0004, 0009, 0022, 0060 | 09, `draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md` |
| 0091 | Movement takes its direction from a per-cell field, never from a per-unit search | Draft | 0004, 0018, 0022, 0024, 0056, 0064 | `draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md` |
| 0092 | The agent tool surface grows one tool at a time, against a stated need | Draft | 0040, 0085 | `draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md` |
| 0095 | A behavioural strategy arrives as a field over cells, never as a search from a unit | Draft | 0004, 0022, 0064, 0091 | `draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md` |
| 0096 | Cost follows the lattice, not the population, and a unit is a reader | Draft | 0009, 0014, 0022, 0064, 0091, 0095 | `draft/adr-0096-cost-follows-the-lattice-not-the-population.md` |
| 0098 | The choice is decided for each cell and each bucket of need | Draft | 0001, 0009, 0063, 0064, 0096 | `draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md` |
| 0099 | A site fills its positions by one sort and one scan | Draft | 0004, 0007, 0014, 0065 | 16, `draft/adr-0099-a-site-fills-its-positions-by-one-sort-and-one-scan.md` |
| 0097 | The toolchain is a dated nightly, pinned to a date and never to a channel | Draft | 0001, 0002, 0008, 0041 | `draft/adr-0097-the-toolchain-is-a-dated-nightly.md` |
| 0103 | The tile value field stores a dense delta, never a sparse change list | Draft | 0001, 0002, 0009, 0022, 0088 | `draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md` |
| 0104 | A soldier is promoted from a level that never falls | Draft | 0004, 0007, 0014, 0054 | 14, `draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md` |
| 0105 | A total order needs no repeated identifier, only no repeated key | Draft | 0001, 0004, 0007 | `draft/adr-0105-a-total-order-needs-no-repeated-key.md` |
| 0106 | A cohort serves whole rations to a keyed subset, never an equal share to everybody | Draft | 0002, 0003, 0004, 0009, 0023, 0063 | `draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md` |
| 0107 | The Python reference is generated from the compiled module, and its prose lives in the Rust doc comment | Draft | 0040, 0041, 0046 | `draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md` |
| 0109 | The choice key holds a bounded class of the unit's own state | Draft | 0004, 0064, 0096, 0098 | `draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md` |
| 0110 | A unit returns by climbing a reach field seeded at every site of its faction | Draft | 0004, 0005, 0022, 0053, 0091, 0095 | `draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md` |
| 0111 | The presence relation is derived at the end of the step and never stored as a fact | Draft | 0001, 0004, 0009, 0018, 0023, 0040, 0053 | `draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md` |
| 0125 | The control plane names the seed set of a destination field | Draft | 0004, 0005, 0022, 0085, 0091, 0095, 0110 | 20, 21, `draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md` |
| 0120 | A unit carries a type, and the type is an index into a table the world is built with | Draft | 0011, 0012, 0014, 0066 | 21, `draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md` |
| 0121 | A meeting between two factions resolves at the tile, never at a level 1 cell | Draft | 0004, 0009, 0018, 0022, 0053, 0120 | 21, `draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md` |
| 0122 | An attacker whose attack does not exceed the defender's armour contributes exactly zero | Draft | 0002, 0023, 0120, 0121 | 21, `draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md` |
| 0123 | Casualties are whole units served to a keyed subset, never a fraction of everybody | Draft | 0001, 0002, 0003, 0004, 0009, 0106, 0121 | 21, `draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md` |
| 0126 | A trade negotiation is engine state, and the words are not | Draft | 0001, 0002, 0004, 0053, 0090 | 21, `draft/adr-0126-a-trade-negotiation-is-engine-state.md` |
| 0127 | A terminal refusal closes an ordered pair until a named tick | Draft | 0120 | `draft/adr-0127-a-terminal-refusal-closes-a-pair-until-a-named-tick.md` |
| 0128 | A contract moves a quantity only when a unit carries it onto the ground of the other party | Draft | 0004, 0062, 0063, 0072, 0073, 0120, 0121 | `draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md` |
| 0129 | A trade negotiation is engine state, and the words are not. Superseded by ADR-0126, the older identical copy | Superseded | 0126 | 21, `draft/adr-0129-a-trade-negotiation-is-engine-state.md` |
| 0130 | A terminal refusal closes an ordered pair until a named tick. Superseded by ADR-0127, the older identical copy | Superseded | 0127 | `draft/adr-0130-a-terminal-refusal-closes-a-pair-until-a-named-tick.md` |
| 0131 | A contract moves a quantity only when a unit carries it onto the ground of the other party. Superseded by ADR-0128, the older identical copy | Superseded | 0128 | `draft/adr-0131-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md` |
| 0132 | Conversion changes the faction of a unit, and adds no second allegiance | Draft | 0014, 0065, 0070, 0072, 0099, 0104, 0111, 0134 | `draft/adr-0132-conversion-changes-the-faction-of-a-unit.md` |
| 0133 | A unit converts to the faction that leads the influence field at its cell | Draft | 0002, 0003, 0004, 0087, 0096, 0111, 0123, 0128, 0132, 0134 | 09, `draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md` |
| 0134 | A god reads conversion as an event log and as the counts it already reads | Draft | 0006, 0064, 0065, 0070, 0132, 0133 | `draft/adr-0134-a-god-reads-conversion-as-an-event-log.md` |
| 0140 | Weather is a field over the level 1 cell lattice | Draft | 0001, 0004, 0009, 0022, 0087, 0121 | `draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md` |
| 0141 | A weather pass moves water and never scales it | Draft | 0002, 0004, 0009, 0023, 0072, 0087 | `draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md` |
| 0142 | A god inflicts weather only on ground its own faction holds | Draft | 0004, 0040, 0053, 0111, 0125, 0128, 0140 | `draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md` |
| 0143 | Wet ground yields more to a gatherer | Draft | 0056, 0072, 0073, 0140 | `draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md` |
| 0144 | A faction controller runs inside the step and acts only through the caller's verbs | Accepted | 0001, 0003, 0004, 0040, 0042, 0043, 0053 | `accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md` |
| 0145 | A unit type is a row of capability columns, and zero means cannot | Accepted | 0002, 0011, 0120, 0121 | `accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md` |
| 0146 | A faction relation is one signed integer per ordered pair, and a pass reads a threshold | Draft | 0001, 0002, 0004, 0006, 0053, 0121, 0126, 0133, 0145 | `draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md` |
| 0147 | A contract consideration is a tagged kind | Accepted | 0002, 0111, 0126, 0127, 0128, 0146 | `accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md` |
| 0148 | A game end is recorded once and stops the controllers | Accepted | 0001, 0004, 0023, 0053, 0144 | `accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md` |

### Retired numbers

A retired number is never reused. The row states what the number held and why
the project stopped holding it.

| No. | Held | Retired because |
|---|---|---|
| 0057 | A long path follows a portal graph and a flow tile | It described a subsystem nobody had built, for a need nobody had stated |
| 0030 | Classic event sourcing is rejected | It stated the alternative that ADR-0031 refuses, and a rejected alternative belongs in the record that refuses it |
| 0033 | Threads write local buffers and the barrier concatenates them in a fixed order | ADR-0009 D1 already forbids a shared output and ADR-0009 D2 already fixes the combine order, so it added no constraint |
| 0034 | A command queues during the Python phase and seals at the barrier | No command queue exists and nothing has chosen one, and ADR-0042 D3 holds the durable part |
| 0035 | Rejection is reported through a closed enumeration | It described a set-valued command layer that does not exist, and it disagreed with how the bindings refuse today |
| 0036 | A snapshot copies dirty chunks, not the world | Nothing snapshots the world and no product record asks for one, so there was no decision to preserve |
| 0037 | The log is transient, and retention is additive | Its own argument was that adding retention later is cheap, which fails the second condition of the scope test |
| 0038 | The aggregate is the region, never the entity | It named a structure this project does not have, and ADR-0009 holds the parallel rule it was carrying |
| 0039 | The save format is hand-written | No save format exists and no serialisation library was ever adopted, so nothing had chosen anything |
| 0045 | View safety needs three layers | Every read across the boundary copies, so the record guarded a hazard the project does not have |

**A retired number is mentioned, never cited.** Write it in a code span. A
citation says "follow this for the claim", and a retired record holds no
claim to follow. The citation check enforces this, and it caught the first
attempt.

**Nine of these rows went in one review, and that is the expected yield.** Rows
0030 to 0039 and 0040 to 0047 were reserved together, by deriving a row for each
claim of two long drafts. A row that reserves a number is a guess that a
constraint exists, and eighteen guesses made at once are not eighteen
constraints. Seven of the eighteen were written, two already had files, and the
rest are above. The finding holds what the review believed at the start and what
it found.[^YIELD]

The research that supports the retired rows is not lost. Reports 03 and 05 hold
the reasoning, and a future record that needs one of these claims starts from the
report and takes a fresh number, as the row for `ADR-0057` already says.

**ADR-0057 was never accepted and nothing cited it.** It specified a portal
graph, a flow tile cache and a coarse biasing field, in detail, before any
path-finding existed and before a product record asked for a long path. It
failed the first condition of the scope test in the strongest way: there was
no decision to preserve, because nothing had chosen anything.

The research that supports it is not lost. Report 10 holds the reasoning, and
a future record that needs a long path starts from the report and takes a
fresh number.

The project keeps this row because retiring a number is the cheap outcome and
the row is what makes it cheap. A reader who finds a citation of ADR-0057 in
an old branch learns here that it went, and why, rather than concluding the
registry lost a record.

## Source reports

The `Source` column gives report numbers under `docs/research/reports/`. Those
reports hold the evidence. A record states the decision and cites the report;
it does not repeat the research.

The reports were written concurrently by agents that could not see each
other's work. They conflict in places. **Read the registers below before
writing any record.**

## Registers

These change over time. Records cite them; records do not contain them.

| Path | Purpose |
|---|---|
| `docs/BLOCKERS.md` | Work that is stopped, and what would start it |
| `docs/DECISIONS.md` | Choices that are open, with a recommendation |
| `docs/FINDINGS.md` | What the project believed and had to correct |
| `docs/research/reports/MERGE-NOTES.md` | Conflicts between the reports |
| `docs/research/research-agenda.md` | Fields still to investigate |
| `docs/reference/budgets.md` | Cost and storage tables |

`FINDINGS.md` is precedent. When two sources disagree, look there first — the
conflict may already be settled.

## Layout

| Path | Contents |
|---|---|
| `docs/adrs/draft/` | Records under review |
| `docs/adrs/accepted/` | Accepted records |
| `docs/research/reports/` | Research that supports the records |

A file is named `adr-NNNN-<slug>.md` and moves between directories as its
status changes. The number never changes.

## Writing order

Write the core and the foundations first, records 0001 to 0006. Every
subsystem record cites them, so their vocabulary must settle before the rest
is written.

Then the cross-cutting models, 0007 to 0013. Then the subsystems.
[^PRI]: Decision record priority index. `docs/adrs/PRIORITY.md`
[^MEASURE]: Findings, the scope of a decision record. `docs/research/adr-scope-findings.md`
[^SCOPE]: Decision Record Scope, the test for whether a decision needs a record. `.claude/rules/adr-scope.md`
[^YIELD]: Findings register, FND-214. `docs/FINDINGS.md`
[^CITE]: Findings register, FND-218. `docs/FINDINGS.md`
[^DERIVED]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^IMPORTED]: Decisions register, DEC-097. `docs/DECISIONS.md`
[^SWEEP]: Recurring Defect Shapes, shape 2. `.claude/rules/recurring-defects.md`
