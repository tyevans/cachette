---
id: 0153
title: Let Python read an event without repeating its layout
status: complete
created: 2026-09-01
implements: [ADR-0002 D1, ADR-0006 D1, ADR-0014 D1, ADR-0014 D2]
changes: []
creates: [ADR-0085, ADR-0163]
serves: []
blocked-by: []
---

## Why

The bindings hand Python the event log as raw bytes. The layout of an event
lives in the Rust source, which declares the field order, the field widths and
the padding.[^1] Python holds no description of it.

A reader in Python must therefore repeat the layout. That is a second
declaration site for one fact, and nothing fails when the two copies
disagree.[^2] The recurring defect rule names this exact pair as a place the
shape will recur in this project.

The consequence is visible now. The agent-facing protocol server returns the
bytes and a digest of them, because it refuses to decode them.[^3] An agent can
prove that two runs emitted the same log. It cannot see which tile changed, who
owned it, or by how much.

## What the work does

Give Python the events, from the one place that declares them. The project
owner closed the choice and took the column form.[^4] The bindings return one
array for each field of an event, in the way the tile column already
crosses.[^7] Python never sees a byte offset, a field width or a field order.

The engine holds two logs. One reports a tile change. One reports a grant of
the gather resolve, and it names the unit that took the amount.[^1] Both cross.

## Impact review

**Governed by.** ADR-0006 D1 makes an event plain data with a declared layout,
so a column is a read of that layout and not a second copy of it.[^5] ADR-0002
D1 bans a floating point number in simulated state, so the fixed-point value
field crosses as its raw integer.[^6] ADR-0014 D1 makes an identity one opaque
value that only the entity storage builds. ADR-0014 D2 makes resolution able to
fail.[^8] ADR-0044 asks a copying call to say that it copies.[^9]

**Creates.** ADR-0085. The record states that an entity crosses to Python as
one opaque identity, and that the engine resolves it against the arena or
refuses. The registry row is allocated and the status is `Draft`. The author is
not the reviewer.

**Changes.** None. No accepted record is superseded.

**Blockers.** None. No value in this work sits behind an open question.

**Precedent.** FND-137 records what the project believed about the event log
and what is true.[^10] The testing rule records a real defect of this shape: a
soldier respawned into one slot at a later generation drew the direction of the
soldier that died there.[^11]

**What this work adds beyond a reader.** Nothing in Python puts a unit in the
world today, so the gather log is always empty on this side of the boundary. A
column that nothing can fill is an inert capability.[^12] The work therefore
adds the verbs that fill it: put units in the world, tell them to gather, and
remove them.

**Those verbs take a set.** The project owner closed the question they opened,
and he closed it by removing it: a verb that takes a collection has no per-unit
form for a caller to repeat.[^13] The founding run was weighed and does not
replace them, because a founding never frees a slot that a later founding
reuses, so it cannot produce the case this work exists to refuse.[^15] The
destination is a selector that says where to act, and a separate item holds
it.[^16]

## What it must not do

It must not add a `struct` format string to Python. That is the defect.

It must not hand out a slot index as the name of an entity. An identity packs
an index and a generation, and only both together name one entity over time.
An index alone survives the death of what it named, so a reader that holds one
reports on a later occupant of the slot and nothing fails.[^8]

It must not convert a fixed-point field to a float on the way out. A float in
simulated state is banned, and a float that enters through an interface is the
same defect one layer further out.[^6]

## Done when

- A method of the bindings returns one array for each field of the tile change
  event. The names are the field names.
- A method of the bindings returns one array for each field of the gather
  event, including the unit identity.
- The fixed-point value field arrives as a signed 32-bit integer. No column has
  a floating point element type.
- No Python file holds a byte offset, a field width or a field order of an
  event. A whole-tree search proves it.
- The bindings resolve an identity that Python hands back. A live identity
  gives the soldier. A stale identity raises a typed error.
- Every verb that acts on units takes a set and answers once. No test in the
  repository shows a caller repeating a verb over the mass tier.
- A set that the engine refuses leaves the world as it found it.
- A test hands back an identity for a slot whose generation has moved on, and
  asserts the failure. Removing the generation check makes that test fail.
- The agent protocol server answers which tile changed, and by how much,
  through the columns. A test drives it through a real protocol client.
- The type stubs describe every new method.
- ADR-0085 exists with status `Draft`, and the registry row states it.
- The whole check command runs green.

## Outcome

**Closed twice, for two different reasons, and both readings stand.** Two
auditors read the code on 5 September 2026, separately, and both found every
statement of the list above satisfied. A later worker found two declaration
sites that the audit could not see, and closed those.

**The reading half was already done.** Both logs cross as named columns. The
event log gives one array for each field, and the names are the field names.
The gather log does the same, and it carries the whole unit identity rather
than a slot index. No column holds a float: the fixed-point value arrives as a
signed 32-bit integer array. The bindings resolve an identity that Python hands
back, and the generation check refuses a stale one.[^20] A real protocol client
drives that refusal.[^21] No Python module reads a byte of an event, unpacks a
structure or builds a structured element type. ADR-0085 is accepted, which is
stronger than the list asks for.

**The declaring half was not.** The type stub of the compiled module held one
typed dictionary for each event log, listing the field names and the element
widths in declaration order. The binding layer held one method for each log
that listed the fields by hand. Both restate the layout without consuming it,
which is why a search for a byte read could not reach them. A finding records
the general shape, because the precedent is worth more than the item.[^17]

**What implements the item now.** Each event type carries one declaration of
its fields. The declaration states no offset and no width, because the compiler
reads both from the type. Every column method builds its arrays from that
declaration and names no field. The compiled module reports the declaration,
and a script writes the column classes of the type stub from that report. The
agent server builds a row from the keys the engine gave, so it names no field
either. A decision record states the constraint.[^18]

**The failure is loud, and it was proved in both directions.** A field added to
an event and left out of the declaration compiles, and the layout test goes red
and names the event and the bytes it does not cover. A field added and declared
makes the stub check go red until the stub follows.

**The copies agreed when this was written.** The mechanism found no existing
mismatch. This is prevention, not repair.

**What the check covers.** It catches a field of the type that the declaration
omits, including an undeclared padding field, because the declared fields must
fill the type exactly. It does not need to catch implicit padding in the type,
because the plain-data derive rejects that at compile time.

**Left undone.** The type stub generates only its event column classes. Every
other declaration in it stays hand-written and unchecked against the
module.[^19]

## References

[^1]: The event types. `crates/cachette-core/src/event.rs`
[^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: The agent protocol server. `python/cachette/agent/server.py`
[^4]: Decisions register, DEC-060. `docs/DECISIONS.md`
[^5]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: The Python bindings. `crates/cachette-py/src/lib.rs`
[^8]: ADR-0014, entity identity is an index plus a generation, decisions D1 and D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^9]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
[^10]: Findings register, FND-137. `docs/FINDINGS.md`
[^11]: Testing Rules, section 2. `.claude/rules/testing.md`
[^12]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^13]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^15]: The founded group tests. `crates/cachette-core/tests/founded_group_survives.rs`
[^16]: Backlog item 0161. `docs/backlog/proposed/0161-let-a-selector-say-where-to-act.md`
[^17]: Findings register, FND-520. `docs/FINDINGS.md`
[^18]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
[^19]: Findings register, FND-521. `docs/FINDINGS.md`
[^20]: The identity resolution tests. `crates/cachette-core/tests/identity_resolution.rs`
[^21]: The protocol client tests. `tests/test_agent_mcp.py`
