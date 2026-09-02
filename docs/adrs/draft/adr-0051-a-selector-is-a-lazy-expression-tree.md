# ADR-0051: A selector is a lazy expression tree that Rust evaluates

## Context

The control plane must say where to act. Today it cannot. A caller can name a
unit it already holds, and it can name an address it chose. It cannot ask for
the places or the units that answer a description.

A finding records the cost of that gap. A test needed one gather event. The
gather resolves against the ground a unit stands on, so the test put a unit on
every open tile of a small world and ordered each one, and let the engine find
the ground.[^1] The need under that test is ordinary: a developer wants a unit
to have a reason to be somewhere, which means naming the ground that holds
something worth taking.[^2]

The boundary rule is already fixed. Python sends an instruction and receives an
answer, and the number of crossings does not grow with the number of entities
the command touches.[^3] [^4] A caller who names a set therefore cannot name it
by listing it, because the list is the population.

The project owner closed the shape of the answer on the command side. A verb
takes a set.[^5] The open part is how a caller describes the set it means. Two
families of answer exist. The caller can compute the set and pass it, or the
caller can describe the set and let the engine compute it.

The research surveyed five libraries that already solve this in Python.[^6] Each
one builds a deferred tree by operator overloading and executes it elsewhere.
The survey also names the failure mode: an interface that pretends to be eager
gets used eagerly, and the abstraction then leaks under performance rather than
under correctness.

The engine has one property it cannot recover if it loses it. The same binary
gives the same answer at any thread count.[^7] Anything that evaluates over the
world in parallel must therefore fix its order.

## Decision

### D1. A selector describes a set. It is not a set

Building a selector reads no tile, no unit and no column. It allocates nodes and
nothing else.

A predicate, a combination of predicates and a spatial restriction are all
nodes. The value of the expression is the tree, and the tree holds no member of
the set it describes.

This is what makes the boundary affordable. A caller composes a description of
any complexity without paying for the world once.

### D2. Python builds the tree, and Rust evaluates it, and the tree crosses once

The whole expression crosses the boundary in one call, together with the verb
or the terminal operation that consumes it.

Rust walks the tree against the columns. Python never sees an intermediate
result, and Python never receives a set so that it can narrow it further.

A selector that crossed the boundary node by node would put the crossing count
back in proportion to the expression, and a selector that returned an
intermediate result would put it in proportion to the population.[^4]

### D3. A selector carries its domain, and a mixed combination fails when the tree is built

A selector over units and a selector over tiles are different types. Combining
them directly is an error at the moment the caller writes it.

The two domains meet through named bridges, in both directions. A caller names
the tiles that hold a set of units, or names the units that stand on a set of
tiles. Each bridge is explicit, so a reader can see where the engine consults
the unit-to-tile bridge.[^8]

The alternative is an empty result at run time, which teaches the caller
nothing and which a test may not distinguish from a correct empty answer.

### D4. A selector holds no snapshot, and the engine evaluates it when the caller uses it

The engine evaluates the tree against the world at the moment a verb or a
terminal operation runs. The same tree, used in two frames, gives two answers.

A selector is therefore reusable and it is not a result. A caller that wants the
answer of one frame asks for the answer, and holds that.

The alternative binds a selector to the frame that built it. The project rejects
it because the engine would then hold a result for every live selector, and the
size of that result is a function of the population.

### D5. Evaluation fixes its order, and the order never comes from a thread

The engine may evaluate a tree in parallel. The result order is fixed by a
stable key over the entities or the addresses, and never by the order in which
threads finished or in which a work-stealing queue handed out work.[^9]

Two runs of one tree over one world state give the same answer, in the same
order, at any thread count.[^7]

No predicate uses a floating point value. A comparison against a fixed-point
field compares the raw integer, in the scale the field declares.[^10] A
selector that compared floats would decide differently on two machines, and the
difference would reach the simulation through the verb that consumed the set.

### D6. A selector reports the evaluation it will perform

A caller can ask a selector what the engine will do with it, and gets an answer
that names each node, how the engine narrows at each node, and where it cannot
narrow at all.

A predicate over a field that carries no summary cannot prune, so the engine
reads every candidate. Nothing about that is wrong, and nothing about it is
visible. Without a report the caller sees only slowness, and reports a
performance defect that is really a schema defect.[^11]

The report is derived from the plan the engine runs. It is not a second
description of that plan, because a second description can disagree with the
first and nothing fails when it does.[^12]

## The alternatives this rejects

**The caller computes the set and passes it.** Python would read the columns,
build a mask with an array library, and hand the result to a verb. This is the
data plane. It also puts the layout of the world into Python, which makes two
declaration sites for one fact and gives nothing that fails when they
disagree.[^12] The boundary record forbids it.[^3]

**The caller sends a query string, and Rust parses it.** A string is easy to
carry and easy to extend. The project rejects it because every error becomes a
run-time error, the caller gets no completion and no type checking, and a string
invites a caller to build one in a loop.

**The engine offers a fixed set of named reads instead of a tree.** Each
question would be one method. The project rejects it as the general answer,
because the set of questions is not closed and each new one is a new crossing to
design, a new type to declare and a new thing to keep in step. A named read
remains correct for a question the engine answers better than a general tree
can, and this record does not forbid one.

**Eager evaluation of each predicate.** Each comparison would return a set
immediately, and combination would be set algebra over results. The project
rejects it because each step is a crossing and each intermediate result has the
size of the population it matched, and because an optimiser cannot then reorder
anything.

## Consequences

**The engine owns the evaluation strategy, and the caller cannot override it.**
A caller describes what it wants. The engine chooses how to find it. A caller
who needs a particular strategy has no way to ask for one, and that refusal is
deliberate.

**Every predicate needs a field the engine can read, and pruning needs a
summary of that field.** A selector makes the summary schema a part of the
control plane interface. Adding a predicate over a field with no summary is
allowed and it is not free.

**A caller can build a tree that is too large.** A caller who composes nodes in
a loop creates an expression that costs more than the query. The engine caps the
node count and refuses beyond it, and the refusal is typed.

**A selector surprises a caller who expects a snapshot.** D4 makes the same
name give different answers in different frames. The interface documents this,
and the reporting operation shows the caller what will be evaluated.

**This record states no cost.** No measurement exists on the target platform,
and every cost figure in this project is derived rather than measured.[^13] The
argument here is about the shape of the traffic and the shape of the work.

**The tree is affordable only because the answer need not be a list.** A
description that costs nothing to build is worth little if evaluating it always
produces one index for every member. A separate record carries that
argument.[^14]

**Nothing implements this record.** No selector type exists, in Python or in
Rust. The record binds the verbs written after it, and it describes no code
today. Building the tree before a caller needs it is the shape this project
avoids, so this record waits for a caller rather than for an author.[^15] [^5]

## References

[^1]: Findings register, FND-147. `docs/FINDINGS.md`
[^2]: PRD-0007, the world holds things worth taking. `docs/product/accepted/prd-0007-the-world-holds-things-worth-taking.md`
[^3]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^4]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^5]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^6]: Selector engine and verbs, section 1.1. `docs/research/reports/04-selector-engine-and-verbs.md`
[^7]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^9]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: Selector engine and verbs, section 1.7. `docs/research/reports/04-selector-engine-and-verbs.md`
[^12]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: ADR-0052, a selector result may be a range, not only an enumerated set. `docs/adrs/draft/adr-0052-a-selector-result-may-be-a-range.md`
[^15]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
